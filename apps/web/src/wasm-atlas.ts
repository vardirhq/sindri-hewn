import type { Frame, Sprite } from "./atlas";

function runWorker<T>(request: object, pixels: Uint8Array, signal?: AbortSignal): Promise<T> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) { reject(new DOMException("Cancelled", "AbortError")); return; }
    const worker = new Worker(new URL("./atlas.worker.ts", import.meta.url), { type: "module" });
    const cleanup = () => { worker.terminate(); signal?.removeEventListener("abort", abort); };
    const abort = () => { cleanup(); reject(new DOMException("Cancelled", "AbortError")); };
    signal?.addEventListener("abort", abort, { once: true });
    worker.onmessage = event => {
      cleanup();
      if (event.data.error !== undefined) reject(new Error(event.data.error));
      else resolve(event.data.result as T);
    };
    worker.onerror = event => { cleanup(); reject(new Error(event.message || "Atlas worker failed")); };
    worker.onmessageerror = () => { cleanup(); reject(new Error("Invalid atlas worker response")); };
    try { worker.postMessage({ ...request, pixels }, [pixels.buffer]); }
    catch (error) { cleanup(); reject(error); }
  });
}
type RawFrame = { name: string; x: number; y: number; width: number; height: number; source_width: number; source_height: number; offset_x: number; offset_y: number };
type RawAtlas = { width: number; height: number; pages: Uint8Array[]; frames: RawFrame[][] };
export async function createAtlas(sprites: Sprite[], width: number, height: number, padding: number, trim: boolean, signal?: AbortSignal): Promise<{ canvases: HTMLCanvasElement[]; frames: Frame[] }> {
  const { pixels, manifest } = encodeSprites(sprites);
  const atlas = await runWorker<RawAtlas>({ kind: "pack", manifest: JSON.stringify(manifest), options: JSON.stringify({ width, height, padding, trim }) }, pixels, signal);
  const canvases = atlas.pages.map(rgba => {
    const canvas = document.createElement("canvas");
    canvas.width = atlas.width;
    canvas.height = atlas.height;
    const target = canvas.getContext("2d");
    if (!target) throw new Error("Canvas is unavailable");
    target.putImageData(new ImageData(new Uint8ClampedArray(rgba), atlas.width, atlas.height), 0, 0);
    return canvas;
  });
  const frames = atlas.frames.flatMap((frames, page) => frames.map(frame => ({
    page, name: frame.name, x: frame.x, y: frame.y, width: frame.width, height: frame.height,
    sourceWidth: frame.source_width, sourceHeight: frame.source_height, offsetX: frame.offset_x, offsetY: frame.offset_y
  })));
  return { canvases, frames };
}
function encodeSprites(sprites: Sprite[]) {
  const manifest = sprites.map(({ name, width, height }) => ({ name, width, height }));
  const total = sprites.reduce((sum, s) => sum + s.width * s.height * 4, 0);
  const pixels = new Uint8Array(total);
  let offset = 0;
  const sourceCanvas = document.createElement("canvas");
  const ctx = sourceCanvas.getContext("2d", { willReadFrequently: true });
  if (!ctx) throw new Error("Canvas is unavailable");
  for (const sprite of sprites) {
    sourceCanvas.width = sprite.width;
    sourceCanvas.height = sprite.height;
    ctx.clearRect(0, 0, sprite.width, sprite.height);
    ctx.drawImage(sprite.image, 0, 0);
    const data = ctx.getImageData(0, 0, sprite.width, sprite.height).data;
    pixels.set(data, offset);
    offset += data.length;
  }
  return { pixels, manifest };
}


export type Recommendation = { width: number; height: number; pages: number; bytes: number };
export type OptimizationProfile = "memory" | "performance" | "balanced";
export async function optimizeAtlas(sprites: Sprite[], padding: number, trim: boolean, profile: OptimizationProfile, limit: number, signal?: AbortSignal): Promise<Recommendation> {
  const { pixels, manifest } = encodeSprites(sprites);
  return runWorker<Recommendation>({ kind: "optimize", manifest: JSON.stringify(manifest), options: JSON.stringify({ width: 128, height: 128, padding, trim }), profile, limit }, pixels, signal);
}

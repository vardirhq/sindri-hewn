import init, { pack_rgba } from "./generated/hewn_wasm.js";
import type { Frame, Sprite } from "./atlas";

let initialized: Promise<void> | undefined;
function ready() {
  initialized ??= init().then(() => undefined).catch(error => {
    initialized = undefined;
    throw error;
  });
  return initialized;
}

/** All geometry, alpha trimming, and packing are performed by hewn-core. */
export async function createAtlas(
  sprites: Sprite[], width: number, height: number, padding: number, trim: boolean
): Promise<{ canvas: HTMLCanvasElement; frames: Frame[] }> {
  await ready();
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
  const atlas = pack_rgba(pixels, JSON.stringify(manifest), JSON.stringify({ width, height, padding, trim }));
  try {
    const canvas = document.createElement("canvas");
    canvas.width = atlas.width;
    canvas.height = atlas.height;
    const target = canvas.getContext("2d");
    if (!target) throw new Error("Canvas is unavailable");
    const rgba = atlas.pixels();
    target.putImageData(new ImageData(new Uint8ClampedArray(rgba), atlas.width, atlas.height), 0, 0);
    const frames = (JSON.parse(atlas.frames_json()) as Array<{
      name: string; x: number; y: number; width: number; height: number;
      source_width: number; source_height: number; offset_x: number; offset_y: number;
    }>).map(frame => ({
      name: frame.name, x: frame.x, y: frame.y, width: frame.width, height: frame.height,
      sourceWidth: frame.source_width, sourceHeight: frame.source_height,
      offsetX: frame.offset_x, offsetY: frame.offset_y
    }));
    return { canvas, frames };
  } finally {
    atlas.free();
  }
}

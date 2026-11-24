import init, { pack_rgba, optimize_rgba } from "./generated/hewn_wasm.js";
self.onmessage = async (event: MessageEvent) => {
  const { kind, pixels, manifest, options, profile, limit } = event.data;
  try {
    await init();
    if (kind === "optimize") {
      self.postMessage({ result: JSON.parse(optimize_rgba(pixels, manifest, options, JSON.stringify(profile), limit)) });
      return;
    }
    const atlas = pack_rgba(pixels, manifest, options);
    try {
      const pages = Array.from({ length: atlas.page_count() }, (_, i) => atlas.pixels(i));
      self.postMessage({ result: { width: atlas.width, height: atlas.height, pages, frames: JSON.parse(atlas.frames_json()) } }, { transfer: pages.map(page => page.buffer) });
    } finally { atlas.free(); }
  } catch (error) { self.postMessage({ error: String(error) }); }
};

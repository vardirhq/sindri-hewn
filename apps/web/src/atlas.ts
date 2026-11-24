/** Sprite loading and browser export utilities. Packing runs in Rust/WASM. */
export type Sprite = {
  name: string;
  width: number;
  height: number;
  image: HTMLImageElement;
  offsetX: number;
  offsetY: number;
  trimWidth: number;
  trimHeight: number;
  fileSize: number;
  preview: string;
};

export type Frame = {
  page: number;
  name: string;
  x: number;
  y: number;
  width: number;
  height: number;
  sourceWidth: number;
  sourceHeight: number;
  offsetX: number;
  offsetY: number;
};

export function loadSprite(file: File): Promise<Sprite> {
  return new Promise((resolve, reject) => {
    const url = URL.createObjectURL(file);
    const image = new Image();
    image.onload = () => {
      URL.revokeObjectURL(url);
      try {
        const canvas = document.createElement("canvas");
        canvas.width = image.width; canvas.height = image.height;
        const ctx = canvas.getContext("2d", { willReadFrequently: true });
        if (!ctx) throw new Error("Canvas is unavailable");
        ctx.drawImage(image, 0, 0);
        const pixels = ctx.getImageData(0, 0, image.width, image.height).data;
        let left = image.width, top = image.height, right = 0, bottom = 0;
        for (let y = 0; y < image.height; y++) for (let x = 0; x < image.width; x++) {
          if (pixels[(y * image.width + x) * 4 + 3] !== 0) {
            left = Math.min(left, x); top = Math.min(top, y);
            right = Math.max(right, x + 1); bottom = Math.max(bottom, y + 1);
          }
        }
        const empty = left === image.width;
        resolve({
          name: file.name, width: image.width, height: image.height,
          image, offsetX: empty ? 0 : left, offsetY: empty ? 0 : top,
          trimWidth: empty ? 1 : right - left, trimHeight: empty ? 1 : bottom - top,
          fileSize: file.size, preview: canvas.toDataURL("image/png")
        });
      } catch (error) { reject(error); }
    };
    image.onerror = () => { URL.revokeObjectURL(url); reject(new Error(`Cannot read ${file.name}`)); };
    image.src = url;
  });
}

export function saveBlob(blob: Blob, name: string) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url; a.download = name; document.body.append(a); a.click(); a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function saveAtlas(canvas: HTMLCanvasElement, name = "hewn-atlas.png") {
  canvas.toBlob(blob => { if (blob) saveBlob(blob, name); }, "image/png");
}

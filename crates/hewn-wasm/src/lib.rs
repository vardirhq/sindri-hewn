//! Browser bridge for the canonical Rust atlas implementation.
use hewn_core::{pack_pages, AtlasOptions, SourceImage};
use image::RgbaImage;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
struct InputSprite {
    name: String,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
pub struct PackedAtlas {
    pixels: Vec<Vec<u8>>,
    width: u32,
    height: u32,
    frames_json: String,
}

#[wasm_bindgen]
impl PackedAtlas {
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn page_count(&self) -> usize {
        self.pixels.len()
    }

    pub fn pixels(&self, page: usize) -> Vec<u8> {
        self.pixels.get(page).cloned().unwrap_or_default()
    }

    pub fn frames_json(&self) -> String {
        self.frames_json.clone()
    }
}

/// Pack concatenated RGBA sprite pixels in manifest order. The manifest
/// contains names and dimensions; Rust owns trimming and placement.
#[wasm_bindgen]
pub fn pack_rgba(
    pixels: &[u8],
    manifest_json: &str,
    options_json: &str,
) -> Result<PackedAtlas, JsValue> {
    let manifest: Vec<InputSprite> = serde_json::from_str(manifest_json)
        .map_err(|error| JsValue::from_str(&format!("Invalid sprite manifest: {error}")))?;
    let options: AtlasOptions = serde_json::from_str(options_json)
        .map_err(|error| JsValue::from_str(&format!("Invalid atlas options: {error}")))?;
    let mut offset = 0usize;
    let mut sources = Vec::with_capacity(manifest.len());
    for sprite in manifest {
        let count = (sprite.width as usize)
            .checked_mul(sprite.height as usize)
            .and_then(|size| size.checked_mul(4))
            .ok_or_else(|| JsValue::from_str("Sprite dimensions overflow"))?;
        let end = offset
            .checked_add(count)
            .ok_or_else(|| JsValue::from_str("Sprite byte offsets overflow"))?;
        let slice = pixels
            .get(offset..end)
            .ok_or_else(|| JsValue::from_str("Sprite pixel buffer is truncated"))?;
        let image = RgbaImage::from_raw(sprite.width, sprite.height, slice.to_vec())
            .ok_or_else(|| JsValue::from_str("Invalid sprite pixel dimensions"))?;
        sources.push(SourceImage {
            name: sprite.name,
            image,
        });
        offset = end;
    }
    if offset != pixels.len() {
        return Err(JsValue::from_str("Sprite pixel buffer has trailing bytes"));
    }
    let pages = pack_pages(&sources, options).map_err(|error| JsValue::from_str(&error.to_string()))?;
    let frames_json = serde_json::to_string(&pages.iter().map(|page| &page.frames).collect::<Vec<_>>())
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    Ok(PackedAtlas {
        pixels: pages.into_iter().map(|page| page.image.into_raw()).collect(),
        width: options.width,
        height: options.height,
        frames_json,
    })
}

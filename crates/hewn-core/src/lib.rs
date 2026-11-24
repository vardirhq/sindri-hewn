//! Deterministic texture atlas assembly with transparent-edge trimming.
use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Describes a source image. Input paths are identifiers, not filesystem access.
pub struct SourceImage {
    pub name: String,
    pub image: RgbaImage,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AtlasOptions {
    pub width: u32,
    pub height: u32,
    pub padding: u32,
    pub trim: bool,
}

impl Default for AtlasOptions {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 1024,
            padding: 2,
            trim: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Frame {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub source_width: u32,
    pub source_height: u32,
    pub offset_x: u32,
    pub offset_y: u32,
}

pub struct Atlas {
    pub image: RgbaImage,
    pub frames: Vec<Frame>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AtlasError {
    #[error("Atlas dimensions must be non-zero")]
    InvalidDimensions,
    #[error("Sprite {name} ({width}x{height}) cannot fit in {atlas_width}x{atlas_height} atlas")]
    DoesNotFit {
        name: String,
        width: u32,
        height: u32,
        atlas_width: u32,
        atlas_height: u32,
    },
}

/// Returns the tight bounds of non-transparent pixels.
/// An empty image occupies one transparent pixel while retaining original dimensions.
fn alpha_bounds(image: &RgbaImage) -> (u32, u32, u32, u32) {
    let (mut left, mut top) = (image.width(), image.height());
    let (mut right, mut bottom) = (0, 0);
    for (x, y, px) in image.enumerate_pixels() {
        if px[3] > 0 {
            left = left.min(x);
            top = top.min(y);
            right = right.max(x + 1);
            bottom = bottom.max(y + 1);
        }
    }
    if left == image.width() {
        (0, 0, 1, 1)
    } else {
        (left, top, right - left, bottom - top)
    }
}

/// A free region in the atlas. Coordinates are half-open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl Rect {
    fn intersects(self, other: Self) -> bool {
        self.x < other.x + other.w
            && other.x < self.x + self.w
            && self.y < other.y + other.h
            && other.y < self.y + self.h
    }

    fn contains(self, other: Self) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.x + other.w <= self.x + self.w
            && other.y + other.h <= self.y + self.h
    }
}

/// Subtract an occupied rectangle from every intersecting free region.
/// Overlapping free regions are permitted; containment pruning keeps the
/// candidate set compact. This is a deterministic MaxRects-style strategy.
fn occupy(free: &mut Vec<Rect>, used: Rect) {
    let mut next = Vec::new();
    for region in free.drain(..) {
        if !region.intersects(used) {
            next.push(region);
            continue;
        }
        if used.x > region.x {
            next.push(Rect { x: region.x, y: region.y, w: used.x - region.x, h: region.h });
        }
        if used.x + used.w < region.x + region.w {
            next.push(Rect {
                x: used.x + used.w, y: region.y,
                w: region.x + region.w - used.x - used.w, h: region.h,
            });
        }
        if used.y > region.y {
            next.push(Rect { x: region.x, y: region.y, w: region.w, h: used.y - region.y });
        }
        if used.y + used.h < region.y + region.h {
            next.push(Rect {
                x: region.x, y: used.y + used.h,
                w: region.w, h: region.y + region.h - used.y - used.h,
            });
        }
    }
    next.retain(|r| r.w > 0 && r.h > 0);
    let mut i = 0;
    while i < next.len() {
        if (0..next.len()).any(|j| i != j && next[j].contains(next[i])
            && (j < i || next[j] != next[i]))
        {
            next.remove(i);
        } else {
            i += 1;
        }
    }
    *free = next;
}

/// Deterministic best-short-side-fit rectangle packing. Input order is
/// preserved in frame metadata, and ties favor top-left placement.
pub fn pack(sources: &[SourceImage], options: AtlasOptions) -> Result<Atlas, AtlasError> {
    if options.width == 0 || options.height == 0 {
        return Err(AtlasError::InvalidDimensions);
    }
    let mut atlas = RgbaImage::from_pixel(options.width, options.height, Rgba([0, 0, 0, 0]));
    let mut free = vec![Rect { x: 0, y: 0, w: options.width, h: options.height }];
    let mut frames = Vec::with_capacity(sources.len());
    for source in sources {
        let original = &source.image;
        let (offset_x, offset_y, width, height) = if options.trim {
            alpha_bounds(original)
        } else {
            (0, 0, original.width().max(1), original.height().max(1))
        };
        let fail = || AtlasError::DoesNotFit {
            name: source.name.clone(), width, height,
            atlas_width: options.width, atlas_height: options.height,
        };
        if width > options.width || height > options.height {
            return Err(fail());
        }
        // Padding is a gap between sprites, not a mandatory outer border.
        let padded_w = width.saturating_add(options.padding).min(options.width);
        let padded_h = height.saturating_add(options.padding).min(options.height);
        let candidate = free.iter().filter(|r| r.w >= padded_w && r.h >= padded_h)
            .min_by_key(|r| {
                let dx = r.w - padded_w;
                let dy = r.h - padded_h;
                (dx.min(dy), dx.max(dy), r.y, r.x)
            })
            .copied()
            .ok_or_else(fail)?;
        let used = Rect { x: candidate.x, y: candidate.y, w: padded_w, h: padded_h };
        for dy in 0..height.min(original.height().saturating_sub(offset_y)) {
            for dx in 0..width.min(original.width().saturating_sub(offset_x)) {
                atlas.put_pixel(used.x + dx, used.y + dy,
                    *original.get_pixel(offset_x + dx, offset_y + dy));
            }
        }
        frames.push(Frame {
            name: source.name.clone(), x: used.x, y: used.y, width, height,
            source_width: original.width(), source_height: original.height(),
            offset_x, offset_y,
        });
        occupy(&mut free, used);
    }
    Ok(Atlas { image: atlas, frames })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_preserves_source_offsets() {
        let mut src = RgbaImage::new(8, 8);
        src.put_pixel(3, 4, Rgba([255, 0, 0, 255]));
        let atlas = pack(
            &[SourceImage {
                name: "red".into(),
                image: src,
            }],
            AtlasOptions {
                width: 8,
                height: 8,
                padding: 2,
                trim: true,
            },
        )
        .unwrap();
        assert_eq!(atlas.frames[0].offset_x, 3);
        assert_eq!(atlas.frames[0].offset_y, 4);
        assert_eq!(atlas.frames[0].width, 1);
        assert_eq!(atlas.image.get_pixel(0, 0)[0], 255);
    }

    #[test]
    fn deterministic_with_row_breaks() {
        let images = (0..3)
            .map(|i| SourceImage {
                name: format!("sprite-{i}"),
                image: RgbaImage::from_pixel(3, 3, Rgba([255, 255, 255, 255])),
            })
            .collect::<Vec<_>>();
        let options = AtlasOptions {
            width: 8,
            height: 12,
            padding: 1,
            trim: false,
        };
        let a = pack(&images, options).unwrap();
        let b = pack(&images, options).unwrap();
        assert_eq!(a.frames, b.frames);
        assert_eq!(a.frames.len(), 3);
        assert!(a.frames.iter().all(|frame| frame.x + frame.width <= 8 && frame.y + frame.height <= 12));
    }

    #[test]
    fn reports_overflow_instead_of_dropping_sprites() {
        let image = RgbaImage::from_pixel(5, 5, Rgba([0, 0, 0, 255]));
        let result = pack(
            &[SourceImage {
                name: "large".into(),
                image,
            }],
            AtlasOptions {
                width: 4,
                height: 4,
                padding: 0,
                trim: true,
            },
        );
        assert!(matches!(result, Err(AtlasError::DoesNotFit { .. })));
    }

    #[test]
    fn packs_into_gaps_that_shelves_waste() {
        let sizes = [(6, 4), (4, 6), (2, 6)];
        let sources = sizes.iter().enumerate().map(|(i, &(w, h))| SourceImage {
            name: format!("tile-{i}"),
            image: RgbaImage::from_pixel(w, h, Rgba([i as u8, 20, 30, 255])),
        }).collect::<Vec<_>>();
        let options = AtlasOptions { width: 10, height: 10, padding: 0, trim: false };
        let atlas = pack(&sources, options).unwrap();
        assert_eq!(atlas.frames.len(), 3);
        for (i, a) in atlas.frames.iter().enumerate() {
            for b in atlas.frames.iter().skip(i + 1) {
                assert!(!Rect { x: a.x, y: a.y, w: a.width, h: a.height }
                    .intersects(Rect { x: b.x, y: b.y, w: b.width, h: b.height }));
            }
        }
    }

    #[test]
    fn padding_keeps_frames_apart() {
        let sources = (0..4).map(|i| SourceImage {
            name: format!("tile-{i}"),
            image: RgbaImage::from_pixel(2, 2, Rgba([255, 0, 0, 255])),
        }).collect::<Vec<_>>();
        let atlas = pack(&sources, AtlasOptions {
            width: 8, height: 8, padding: 1, trim: false,
        }).unwrap();
        for (i, a) in atlas.frames.iter().enumerate() {
            for b in atlas.frames.iter().skip(i + 1) {
                let gap_x = a.x + a.width < b.x || b.x + b.width < a.x;
                let gap_y = a.y + a.height < b.y || b.y + b.height < a.y;
                assert!(gap_x || gap_y);
            }
        }
    }

    #[test]
    fn empty_image_has_stable_placeholder() {
        let result = pack(
            &[SourceImage {
                name: "empty".into(),
                image: RgbaImage::new(2, 2),
            }],
            AtlasOptions::default(),
        )
        .unwrap();
        assert_eq!((result.frames[0].width, result.frames[0].height), (1, 1));
        assert_eq!(result.image.get_pixel(0, 0)[3], 0);
    }
}

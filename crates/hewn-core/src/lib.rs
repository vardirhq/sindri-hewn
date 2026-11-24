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
    #[error("Unsupported optimization texture limit")]
    InvalidLimit,
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
            next.push(Rect {
                x: region.x,
                y: region.y,
                w: used.x - region.x,
                h: region.h,
            });
        }
        if used.x + used.w < region.x + region.w {
            next.push(Rect {
                x: used.x + used.w,
                y: region.y,
                w: region.x + region.w - used.x - used.w,
                h: region.h,
            });
        }
        if used.y > region.y {
            next.push(Rect {
                x: region.x,
                y: region.y,
                w: region.w,
                h: used.y - region.y,
            });
        }
        if used.y + used.h < region.y + region.h {
            next.push(Rect {
                x: region.x,
                y: used.y + used.h,
                w: region.w,
                h: region.y + region.h - used.y - used.h,
            });
        }
    }
    next.retain(|r| r.w > 0 && r.h > 0);
    let mut i = 0;
    while i < next.len() {
        if (0..next.len())
            .any(|j| i != j && next[j].contains(next[i]) && (j < i || next[j] != next[i]))
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
    let mut pages = pack_internal(sources, options, false, true)?;
    Ok(pages.remove(0))
}

/// Pack into as many pages as needed, trying existing pages before allocating another.
pub fn pack_pages(
    sources: &[SourceImage],
    options: AtlasOptions,
) -> Result<Vec<Atlas>, AtlasError> {
    pack_internal(sources, options, true, true)
}

fn pack_internal(
    sources: &[SourceImage],
    options: AtlasOptions,
    multiple: bool,
    render: bool,
) -> Result<Vec<Atlas>, AtlasError> {
    if options.width == 0 || options.height == 0 {
        return Err(AtlasError::InvalidDimensions);
    }
    let new_page = || Atlas {
        image: if render {
            RgbaImage::from_pixel(options.width, options.height, Rgba([0, 0, 0, 0]))
        } else {
            RgbaImage::new(0, 0)
        },
        frames: Vec::new(),
    };
    let new_free = || {
        vec![Rect {
            x: 0,
            y: 0,
            w: options.width,
            h: options.height,
        }]
    };
    let mut pages = vec![new_page()];
    let mut regions = vec![new_free()];
    for source in sources {
        let original = &source.image;
        let (offset_x, offset_y, width, height) = if options.trim {
            alpha_bounds(original)
        } else {
            (0, 0, original.width().max(1), original.height().max(1))
        };
        let fail = || AtlasError::DoesNotFit {
            name: source.name.clone(),
            width,
            height,
            atlas_width: options.width,
            atlas_height: options.height,
        };
        if width > options.width || height > options.height {
            return Err(fail());
        }
        // Padding is a gap between sprites, not a mandatory outer border.
        let padded_w = width.saturating_add(options.padding).min(options.width);
        let padded_h = height.saturating_add(options.padding).min(options.height);
        let choose = |free: &[Rect]| {
            free.iter()
                .filter(|r| r.w >= padded_w && r.h >= padded_h)
                .min_by_key(|r| {
                    let dx = r.w - padded_w;
                    let dy = r.h - padded_h;
                    (dx.min(dy), dx.max(dy), r.y, r.x)
                })
                .copied()
        };
        let (page, candidate) = match regions
            .iter()
            .enumerate()
            .find_map(|(i, free)| choose(free).map(|r| (i, r)))
        {
            Some(found) => found,
            None if multiple => {
                let free = new_free();
                let candidate = choose(&free).ok_or_else(fail)?;
                regions.push(free);
                pages.push(new_page());
                (pages.len() - 1, candidate)
            }
            None => return Err(fail()),
        };
        let atlas = &mut pages[page];
        let used = Rect {
            x: candidate.x,
            y: candidate.y,
            w: padded_w,
            h: padded_h,
        };
        if render {
            for dy in 0..height.min(original.height().saturating_sub(offset_y)) {
                for dx in 0..width.min(original.width().saturating_sub(offset_x)) {
                    atlas.image.put_pixel(
                        used.x + dx,
                        used.y + dy,
                        *original.get_pixel(offset_x + dx, offset_y + dy),
                    );
                }
            }
        }
        atlas.frames.push(Frame {
            name: source.name.clone(),
            x: used.x,
            y: used.y,
            width,
            height,
            source_width: original.width(),
            source_height: original.height(),
            offset_x,
            offset_y,
        });
        occupy(&mut regions[page], used);
    }
    Ok(pages)
}

/// Recommendation from a geometry-only search of supported power-of-two sizes.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Recommendation {
    pub width: u32,
    pub height: u32,
    pub pages: usize,
    pub bytes: u64,
}

pub fn optimize(
    sources: &[SourceImage],
    options: AtlasOptions,
) -> Result<Recommendation, AtlasError> {
    optimize_with(sources, options, OptimizationProfile::Memory, 4096)
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationProfile {
    Memory,
    Performance,
    Balanced,
}

pub fn optimize_with(
    sources: &[SourceImage],
    options: AtlasOptions,
    profile: OptimizationProfile,
    limit: u32,
) -> Result<Recommendation, AtlasError> {
    if ![128, 256, 512, 1024, 2048, 4096].contains(&limit) {
        return Err(AtlasError::InvalidLimit);
    }
    let mut best: Option<Recommendation> = None;
    for width in [128, 256, 512, 1024, 2048, 4096] {
        for height in [128, 256, 512, 1024, 2048, 4096] {
            if width > limit || height > limit {
                continue;
            }
            let candidate = AtlasOptions {
                width,
                height,
                ..options
            };
            let Ok(pages) = pack_internal(sources, candidate, true, false) else {
                continue;
            };
            let result = Recommendation {
                width,
                height,
                pages: pages.len(),
                bytes: u64::from(width) * u64::from(height) * 4 * pages.len() as u64,
            };
            let score = |r: &Recommendation| {
                let pages = r.pages as u64;
                let (primary, secondary) = match profile {
                    OptimizationProfile::Memory => (r.bytes, pages),
                    OptimizationProfile::Performance => (pages, r.bytes),
                    OptimizationProfile::Balanced => (
                        r.bytes.saturating_add(pages.saturating_mul(262144)),
                        r.bytes,
                    ),
                };
                (primary, secondary, r.width.abs_diff(r.height), r.width)
            };
            if best.as_ref().is_none_or(|old| score(&result) < score(old)) {
                best = Some(result);
            }
        }
    }
    if let Some(best) = best {
        return Ok(best);
    }
    // Return the actual oversized sprite error rather than a generic failure.
    pack_internal(
        sources,
        AtlasOptions {
            width: limit,
            height: limit,
            ..options
        },
        true,
        false,
    )?;
    Err(AtlasError::InvalidDimensions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimizer_prefers_smallest_memory_and_matches_rendered_pages() {
        let sources = vec![SourceImage {
            name: "small".into(),
            image: RgbaImage::new(16, 16),
        }];
        let options = AtlasOptions::default();
        let best = optimize(&sources, options).unwrap();
        assert_eq!(
            (best.width, best.height, best.pages, best.bytes),
            (128, 128, 1, 65536)
        );
        assert_eq!(best, optimize(&sources, options).unwrap());
        let pages = pack_pages(
            &sources,
            AtlasOptions {
                width: best.width,
                height: best.height,
                ..options
            },
        )
        .unwrap();
        assert_eq!(pages.len(), best.pages);
    }

    #[test]
    fn profiles_respect_limits_and_priorities() {
        let sources = (0..3)
            .map(|i| SourceImage {
                name: format!("sprite-{i}"),
                image: RgbaImage::new(128, 128),
            })
            .collect::<Vec<_>>();
        let options = AtlasOptions {
            trim: false,
            padding: 0,
            ..AtlasOptions::default()
        };
        let memory = optimize_with(&sources, options, OptimizationProfile::Memory, 256).unwrap();
        let performance =
            optimize_with(&sources, options, OptimizationProfile::Performance, 256).unwrap();
        let balanced =
            optimize_with(&sources, options, OptimizationProfile::Balanced, 256).unwrap();
        assert_eq!(memory.pages, 3);
        assert_eq!(performance.pages, 1);
        assert_eq!(balanced.pages, 1);
        assert_eq!(
            optimize_with(&sources, options, OptimizationProfile::Performance, 128)
                .unwrap()
                .pages,
            3
        );
    }

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
        assert!(a
            .frames
            .iter()
            .all(|frame| frame.x + frame.width <= 8 && frame.y + frame.height <= 12));
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
        let sources = sizes
            .iter()
            .enumerate()
            .map(|(i, &(w, h))| SourceImage {
                name: format!("tile-{i}"),
                image: RgbaImage::from_pixel(w, h, Rgba([i as u8, 20, 30, 255])),
            })
            .collect::<Vec<_>>();
        let options = AtlasOptions {
            width: 10,
            height: 10,
            padding: 0,
            trim: false,
        };
        let atlas = pack(&sources, options).unwrap();
        assert_eq!(atlas.frames.len(), 3);
        for (i, a) in atlas.frames.iter().enumerate() {
            for b in atlas.frames.iter().skip(i + 1) {
                assert!(!Rect {
                    x: a.x,
                    y: a.y,
                    w: a.width,
                    h: a.height
                }
                .intersects(Rect {
                    x: b.x,
                    y: b.y,
                    w: b.width,
                    h: b.height
                }));
            }
        }
    }

    #[test]
    fn padding_keeps_frames_apart() {
        let sources = (0..4)
            .map(|i| SourceImage {
                name: format!("tile-{i}"),
                image: RgbaImage::from_pixel(2, 2, Rgba([255, 0, 0, 255])),
            })
            .collect::<Vec<_>>();
        let atlas = pack(
            &sources,
            AtlasOptions {
                width: 8,
                height: 8,
                padding: 1,
                trim: false,
            },
        )
        .unwrap();
        for (i, a) in atlas.frames.iter().enumerate() {
            for b in atlas.frames.iter().skip(i + 1) {
                let gap_x = a.x + a.width < b.x || b.x + b.width < a.x;
                let gap_y = a.y + a.height < b.y || b.y + b.height < a.y;
                assert!(gap_x || gap_y);
            }
        }
    }

    #[test]
    fn overflow_creates_pages_without_losing_frames() {
        let sources = (0..3)
            .map(|i| SourceImage {
                name: format!("sprite-{i}"),
                image: RgbaImage::from_pixel(4, 4, Rgba([i, 0, 0, 255])),
            })
            .collect::<Vec<_>>();
        let options = AtlasOptions {
            width: 4,
            height: 4,
            padding: 0,
            trim: false,
        };
        let pages = pack_pages(&sources, options).unwrap();
        assert_eq!(pages.len(), 3);
        for (i, page) in pages.iter().enumerate() {
            assert_eq!(page.frames[0].name, sources[i].name);
            assert_eq!(page.image, sources[i].image);
        }
        assert!(pack(&sources, options).is_err());
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

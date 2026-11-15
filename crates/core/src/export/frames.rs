//! Frame preparation: one selected photograph → one output-sized RGB frame.
//!
//! Decoding happens here, in Rust, and the pixels are piped to FFmpeg. FFmpeg never
//! opens a photograph and never sees its path (ADR-021). The EXIF orientation is
//! applied, the image is fitted inside the output size without cropping, and it is
//! centred on black (letterbox / pillarbox).
//!
//! A frame may carry Phase 8 adjustments (ADR-025): a stabilization shift and crop,
//! applied to the decoded image *before* its EXIF orientation (the shifts were measured
//! on the review thumbnails, which are not rotated), and deflicker gains, applied through
//! lookup tables to the finished frame.

use std::path::Path;

use image::imageops::{self, FilterType};
use image::{DynamicImage, ImageDecoder, RgbImage};

use crate::clean::analysis::decode_limits;
use crate::{Error, Result};

/// What to do to one frame besides fitting it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FrameAdjust {
    /// Deflicker lookup tables for R, G and B.
    pub luts: Option<[[u8; 256]; 3]>,
    /// Stabilization: the content shift `(x, y)` and the crop per side, as fractions of the
    /// (unrotated) image.
    pub shift: Option<(f64, f64, f64)>,
}

impl FrameAdjust {
    pub const NONE: Self = Self { luts: None, shift: None };
}

/// `img` cropped by `crop` per side, the window moved so the content shifts by `(sx, sy)`.
pub fn shift_crop(img: &DynamicImage, sx: f64, sy: f64, crop: f64) -> DynamicImage {
    let (w, h) = (f64::from(img.width()), f64::from(img.height()));
    let crop = crop.clamp(0.0, 0.25);
    let cw = (w * (1.0 - 2.0 * crop)).round().max(1.0);
    let ch = (h * (1.0 - 2.0 * crop)).round().max(1.0);
    // Moving the window by −s moves the content by +s.
    let x0 = (w * crop - sx.clamp(-crop, crop) * w).round().clamp(0.0, w - cw);
    let y0 = (h * crop - sy.clamp(-crop, crop) * h).round().clamp(0.0, h - ch);
    img.crop_imm(x0 as u32, y0 as u32, cw as u32, ch as u32)
}

/// Decode `path` and return `width × height × 3` bytes of RGB.
pub fn prepare(path: &Path, width: u32, height: u32) -> Result<Vec<u8>> {
    prepare_adjusted(path, width, height, &FrameAdjust::NONE)
}

/// [`prepare`] with the frame's adjustments applied.
pub fn prepare_adjusted(
    path: &Path,
    width: u32,
    height: u32,
    adjust: &FrameAdjust,
) -> Result<Vec<u8>> {
    let path_str = || path.to_string_lossy().into_owned();
    let mut reader = image::ImageReader::open(path)
        .map_err(|_| Error::ImageUnreadable { path: path_str() })?
        .with_guessed_format()
        .map_err(|_| Error::ImageUnreadable { path: path_str() })?;
    reader.limits(decode_limits());
    let mut decoder =
        reader.into_decoder().map_err(|_| Error::ImageUndecodable { path: path_str() })?;
    let orientation = decoder.orientation().ok();
    let mut img = DynamicImage::from_decoder(decoder)
        .map_err(|_| Error::ImageUndecodable { path: path_str() })?;
    if let Some((sx, sy, crop)) = adjust.shift {
        img = shift_crop(&img, sx, sy, crop);
    }
    if let Some(o) = orientation {
        img.apply_orientation(o);
    }
    let mut out = fit(&img.to_rgb8(), width, height).into_raw();
    if let Some(luts) = &adjust.luts {
        crate::adjust::deflicker::apply(&mut out, luts);
    }
    Ok(out)
}

/// `src` scaled to fit inside `width × height` (aspect kept) and centred on black.
pub fn fit(src: &RgbImage, width: u32, height: u32) -> RgbImage {
    let (sw, sh) = (src.width().max(1), src.height().max(1));
    let scale = f64::min(f64::from(width) / f64::from(sw), f64::from(height) / f64::from(sh));
    let nw = ((f64::from(sw) * scale).round() as u32).clamp(1, width);
    let nh = ((f64::from(sh) * scale).round() as u32).clamp(1, height);
    let resized = if (nw, nh) == (sw, sh) {
        src.clone()
    } else {
        imageops::resize(src, nw, nh, FilterType::Triangle)
    };
    let mut canvas = RgbImage::new(width, height);
    imageops::overlay(
        &mut canvas,
        &resized,
        i64::from((width - nw) / 2),
        i64::from((height - nh) / 2),
    );
    canvas
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clean::analysis::tests::write_image;
    use crate::import::metadata::fixture::TempDir;

    #[test]
    fn wide_images_are_letterboxed_and_tall_ones_pillarboxed() {
        let red = RgbImage::from_pixel(400, 100, image::Rgb([255, 0, 0]));
        let f = fit(&red, 200, 200);
        assert_eq!(f.dimensions(), (200, 200));
        assert_eq!(f.get_pixel(100, 10).0, [0, 0, 0], "bar above");
        assert_eq!(f.get_pixel(100, 100).0, [255, 0, 0], "picture in the middle");
        let tall = RgbImage::from_pixel(100, 400, image::Rgb([0, 255, 0]));
        let f = fit(&tall, 200, 200);
        assert_eq!(f.get_pixel(5, 100).0, [0, 0, 0], "bar at the side");
        assert_eq!(f.get_pixel(100, 100).0, [0, 255, 0]);
    }

    #[test]
    fn prepare_returns_exactly_one_rgb_frame() {
        let dir = TempDir::new("frames");
        let p = dir.path().join("a.png");
        write_image(&p, 64, 48, |_, _| [10, 20, 30]);
        let bytes = prepare(&p, 32, 18).unwrap();
        assert_eq!(bytes.len(), 32 * 18 * 3);
        let gone = dir.path().join("gone.jpg");
        assert!(matches!(prepare(&gone, 32, 18), Err(Error::ImageUnreadable { .. })));
    }

    /// Column of the single white pixel in the middle row.
    fn marker_x(img: &DynamicImage) -> u32 {
        let rgb = img.to_rgb8();
        let y = rgb.height() / 2;
        (0..rgb.width()).find(|&x| rgb.get_pixel(x, y).0 == [255, 255, 255]).unwrap()
    }

    #[test]
    fn shift_crop_moves_the_content_by_the_shift() {
        let mut img = RgbImage::new(200, 100);
        img.put_pixel(100, 50, image::Rgb([255, 255, 255]));
        let img = DynamicImage::ImageRgb8(img);
        let centred = shift_crop(&img, 0.0, 0.0, 0.05);
        assert_eq!((centred.width(), centred.height()), (180, 90));
        assert_eq!(marker_x(&centred), 90, "100 − 10 px of crop");
        // +2% of 200 px: the content moves 4 px right.
        assert_eq!(marker_x(&shift_crop(&img, 0.02, 0.0, 0.05)), 94);
        assert_eq!(marker_x(&shift_crop(&img, -0.05, 0.0, 0.05)), 80);
        // A shift beyond the crop is held at the crop.
        assert_eq!(marker_x(&shift_crop(&img, 0.5, 0.0, 0.05)), 100);
    }

    #[test]
    fn adjustments_apply_gains_and_none_changes_nothing() {
        let dir = TempDir::new("frames-adjust");
        let p = dir.path().join("a.png");
        write_image(&p, 64, 48, |_, _| [100, 100, 100]);
        let plain = prepare(&p, 64, 48).unwrap();
        assert_eq!(prepare_adjusted(&p, 64, 48, &FrameAdjust::NONE).unwrap(), plain);
        let luts = crate::adjust::deflicker::luts([2.0, 2.0, 2.0]);
        let bright =
            prepare_adjusted(&p, 64, 48, &FrameAdjust { luts: Some(luts), shift: None }).unwrap();
        assert!(bright.iter().zip(&plain).all(|(b, a)| *b == luts[0][usize::from(*a)]));
        let cropped = prepare_adjusted(
            &p,
            64,
            48,
            &FrameAdjust { luts: None, shift: Some((0.0, 0.0, 0.05)) },
        )
        .unwrap();
        assert_eq!(cropped.len(), plain.len(), "the output size is unchanged");
    }
}

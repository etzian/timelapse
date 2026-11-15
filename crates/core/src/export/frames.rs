//! Frame preparation: one selected photograph → one output-sized RGB frame.
//!
//! Decoding happens here, in Rust, and the pixels are piped to FFmpeg. FFmpeg never
//! opens a photograph and never sees its path (ADR-021). The EXIF orientation is
//! applied, the image is fitted inside the output size without cropping, and it is
//! centred on black (letterbox / pillarbox).

use std::path::Path;

use image::imageops::{self, FilterType};
use image::{DynamicImage, ImageDecoder, RgbImage};

use crate::clean::analysis::decode_limits;
use crate::{Error, Result};

/// Decode `path` and return `width × height × 3` bytes of RGB.
pub fn prepare(path: &Path, width: u32, height: u32) -> Result<Vec<u8>> {
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
    if let Some(o) = orientation {
        img.apply_orientation(o);
    }
    Ok(fit(&img.to_rgb8(), width, height).into_raw())
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
}

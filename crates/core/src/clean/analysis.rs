//! Brightness analysis (PROCESSING_PIPELINE.md §4, ADR-020).
//!
//! Each image is decoded, reduced so its longest side is at most [`ANALYSIS_MAX_SIDE`],
//! and reduced to a 256-bin histogram of Rec. 709 *luma*, computed on the encoded 8-bit
//! values. Every other measurement is derived from that histogram, so a user can change
//! the dark level without a new pass over the originals: mean, median, the dark fraction
//! at any level, and the 32-bin display histogram. Originals are opened read-only.

use std::path::Path;

/// Bumped whenever the measurement changes; rows with an older version are re-analysed.
pub const ANALYSIS_VERSION: i64 = 1;
/// Longest side of the analysed raster, in pixels.
pub const ANALYSIS_MAX_SIDE: u32 = 512;
/// Bins of the display histogram (the stored histogram always has 256).
pub const DISPLAY_BINS: usize = 32;

/// The luma distribution of one image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Luma {
    /// `histogram[y]` = number of analysed pixels with luma `y`; always 256 entries.
    pub histogram: Vec<u64>,
    /// Size of the analysed raster (not of the original).
    pub width: u32,
    pub height: u32,
}

/// Why an image could not be analysed. The string form is stored and mapped to a
/// Fluent key (`clean-failure-<kind>`) in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisFailure {
    /// The file could not be opened or read.
    Unreadable,
    /// The bytes are not a decodable image in a supported format.
    Undecodable,
}

impl AnalysisFailure {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unreadable => "unreadable",
            Self::Undecodable => "undecodable",
        }
    }
}

/// Rec. 709 luma of an 8-bit RGB triple, rounded, using integer arithmetic so the result
/// is the same on every platform.
pub fn luma(r: u8, g: u8, b: u8) -> u8 {
    let y = 2126 * u32::from(r) + 7152 * u32::from(g) + 722 * u32::from(b);
    ((y + 5000) / 10_000) as u8
}

/// Explicit decode limits: up to 30 000 px a side and 768 MiB per decode (a 16-bit 50 MP
/// TIFF needs ~300 MiB). A larger file is reported as undecodable rather than exhausting
/// memory; with [`crate::clean`]'s thread cap this bounds peak use.
pub fn decode_limits() -> image::Limits {
    let mut l = image::Limits::default();
    l.max_image_width = Some(30_000);
    l.max_image_height = Some(30_000);
    l.max_alloc = Some(768 * 1024 * 1024);
    l
}

/// Decode `path` and measure it.
pub fn analyze(path: &Path) -> Result<Luma, AnalysisFailure> {
    let reader = image::ImageReader::open(path).map_err(|_| AnalysisFailure::Unreadable)?;
    let mut reader = reader.with_guessed_format().map_err(|_| AnalysisFailure::Unreadable)?;
    reader.limits(decode_limits());
    // Once the file is open, any failure (including a truncated stream) is a decoding one.
    let img = reader.decode().map_err(|_| AnalysisFailure::Undecodable)?;
    Ok(measure(&img))
}

/// Measure an already-decoded image (shared with tests and thumbnails).
pub fn measure(img: &image::DynamicImage) -> Luma {
    let small = if img.width().max(img.height()) > ANALYSIS_MAX_SIDE {
        img.thumbnail(ANALYSIS_MAX_SIDE, ANALYSIS_MAX_SIDE)
    } else {
        img.clone()
    };
    let rgb = small.to_rgb8();
    let mut histogram = vec![0u64; 256];
    for p in rgb.pixels() {
        histogram[usize::from(luma(p[0], p[1], p[2]))] += 1;
    }
    Luma { histogram, width: rgb.width(), height: rgb.height() }
}

impl Luma {
    pub fn pixels(&self) -> u64 {
        self.histogram.iter().sum()
    }

    /// Mean luma, 0–255. `0.0` for an empty raster.
    pub fn mean(&self) -> f64 {
        let n = self.pixels();
        if n == 0 {
            return 0.0;
        }
        let sum: u64 = self.histogram.iter().enumerate().map(|(y, c)| y as u64 * c).sum();
        sum as f64 / n as f64
    }

    /// Median luma (lower median), 0–255.
    pub fn median(&self) -> u8 {
        let n = self.pixels();
        let half = n.div_ceil(2).max(1);
        let mut seen = 0u64;
        for (y, c) in self.histogram.iter().enumerate() {
            seen += c;
            if seen >= half {
                return y as u8;
            }
        }
        0
    }

    /// Fraction (0–1) of pixels whose luma is at or below `level`.
    pub fn fraction_at_or_below(&self, level: u8) -> f64 {
        let n = self.pixels();
        if n == 0 {
            return 0.0;
        }
        let dark: u64 = self.histogram[..=usize::from(level)].iter().sum();
        dark as f64 / n as f64
    }

    /// The histogram folded into [`DISPLAY_BINS`] equal bins.
    pub fn display_bins(&self) -> Vec<u64> {
        let per = 256 / DISPLAY_BINS;
        self.histogram.chunks(per).map(|c| c.iter().sum()).collect()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::import::metadata::fixture::TempDir;
    use image::{ImageBuffer, Rgb, RgbImage};

    /// Write an RGB image in the format implied by `path`'s extension.
    pub(crate) fn write_image(path: &Path, w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 3]) {
        let img: RgbImage = ImageBuffer::from_fn(w, h, |x, y| Rgb(f(x, y)));
        img.save(path).expect("write test image");
    }

    #[test]
    fn luma_uses_rec_709_weights() {
        assert_eq!(luma(0, 0, 0), 0);
        assert_eq!(luma(255, 255, 255), 255);
        assert_eq!(luma(255, 0, 0), 54);
        assert_eq!(luma(0, 255, 0), 182);
        assert_eq!(luma(0, 0, 255), 18);
    }

    #[test]
    fn black_and_white_png_are_measured_exactly() {
        let dir = TempDir::new("an-bw");
        let black = dir.path().join("black.png");
        write_image(&black, 64, 48, |_, _| [0, 0, 0]);
        let l = analyze(&black).unwrap();
        assert_eq!((l.width, l.height, l.pixels()), (64, 48, 64 * 48));
        assert_eq!((l.mean(), l.median()), (0.0, 0));
        assert_eq!(l.fraction_at_or_below(0), 1.0);

        let white = dir.path().join("white.png");
        write_image(&white, 10, 10, |_, _| [255, 255, 255]);
        let l = analyze(&white).unwrap();
        assert_eq!((l.mean(), l.median()), (255.0, 255));
        assert_eq!(l.fraction_at_or_below(254), 0.0);
    }

    #[test]
    fn half_dark_image_reports_half_dark_pixels() {
        let dir = TempDir::new("an-half");
        let p = dir.path().join("half.tiff");
        write_image(&p, 100, 10, |x, _| if x < 50 { [10, 10, 10] } else { [200, 200, 200] });
        let l = analyze(&p).unwrap();
        assert_eq!(l.fraction_at_or_below(30), 0.5);
        assert_eq!(l.mean(), 105.0);
        assert_eq!(l.median(), 10, "lower median of an even split");
        assert_eq!(l.display_bins().len(), DISPLAY_BINS);
        assert_eq!(l.display_bins().iter().sum::<u64>(), l.pixels());
    }

    #[test]
    fn a_gradient_jpeg_is_close_to_its_true_mean() {
        let dir = TempDir::new("an-grad");
        let p = dir.path().join("grad.jpg");
        write_image(&p, 256, 16, |x, _| [x as u8, x as u8, x as u8]);
        let l = analyze(&p).unwrap();
        assert!((l.mean() - 127.5).abs() < 3.0, "mean {} (JPEG is lossy)", l.mean());
    }

    #[test]
    fn large_images_are_reduced_to_the_analysis_size() {
        let dir = TempDir::new("an-big");
        let p = dir.path().join("big.png");
        write_image(&p, 2048, 1024, |_, _| [128, 128, 128]);
        let l = analyze(&p).unwrap();
        assert_eq!((l.width, l.height), (512, 256));
        assert_eq!(l.median(), 128);
    }

    #[test]
    fn failures_are_classified_not_raised() {
        let dir = TempDir::new("an-fail");
        assert_eq!(analyze(&dir.path().join("absent.jpg")), Err(AnalysisFailure::Unreadable));
        let junk = dir.path().join("junk.jpg");
        std::fs::write(&junk, b"definitely not a jpeg").unwrap();
        assert_eq!(analyze(&junk), Err(AnalysisFailure::Undecodable));
        let truncated = dir.path().join("trunc.png");
        std::fs::write(&truncated, b"\x89PNG\r\n\x1a\n\0\0").unwrap();
        assert_eq!(analyze(&truncated), Err(AnalysisFailure::Undecodable));
    }

    #[test]
    fn empty_histogram_is_harmless() {
        let l = Luma { histogram: vec![0; 256], width: 0, height: 0 };
        assert_eq!((l.mean(), l.median(), l.fraction_at_or_below(10)), (0.0, 0, 0.0));
    }
}

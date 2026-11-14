//! Review thumbnails: small JPEGs cached under `<root>/.timelapse/thumbs/`.
//!
//! The cache key includes the source hash, so a changed original gets a fresh thumbnail.
//! Thumbnails are derived data: deleting the folder loses nothing.

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// The only two sizes the UI asks for: grid tiles and the inspect view.
pub const SIZES: [u32; 2] = [256, 1024];

/// The supported size closest to `requested`.
pub fn snap(requested: u32) -> u32 {
    if requested > 512 { SIZES[1] } else { SIZES[0] }
}

pub fn thumbs_dir(root: &Path) -> PathBuf {
    crate::import::project_dir(root).join("thumbs")
}

fn cache_path(root: &Path, id: i64, size: u32, hash: Option<&str>) -> PathBuf {
    let tag = hash.map_or("nohash", |h| &h[..h.len().min(16)]);
    thumbs_dir(root).join(format!("{id}-{size}-{tag}.jpg"))
}

/// JPEG bytes of a thumbnail whose longest side is at most `size`, from the cache when
/// possible. `source` is the image's current location.
pub fn thumbnail(
    root: &Path,
    id: i64,
    size: u32,
    source: &Path,
    hash: Option<&str>,
) -> Result<Vec<u8>> {
    let size = snap(size);
    let cached = cache_path(root, id, size, hash);
    if hash.is_some()
        && let Ok(bytes) = fs::read(&cached)
    {
        return Ok(bytes);
    }
    let path_str = || source.to_string_lossy().into_owned();
    let mut reader = image::ImageReader::open(source)
        .map_err(|_| Error::ImageUnreadable { path: path_str() })?
        .with_guessed_format()
        .map_err(|_| Error::ImageUnreadable { path: path_str() })?;
    reader.limits(super::analysis::decode_limits());
    let img = reader.decode().map_err(|_| Error::ImageUndecodable { path: path_str() })?;
    let small = if img.width().max(img.height()) > size { img.thumbnail(size, size) } else { img };
    let rgb = small.to_rgb8();
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(Cursor::new(&mut out), 80)
        .encode_image(&rgb)
        .map_err(|_| Error::ImageUndecodable { path: path_str() })?;
    // Best effort: a cache write failure only costs a re-encode next time. Write to a
    // temporary name and rename, so a reader never sees half a file.
    if hash.is_some() && fs::create_dir_all(thumbs_dir(root)).is_ok() {
        let tmp = cached.with_extension("tmp");
        if fs::write(&tmp, &out).is_ok() && fs::rename(&tmp, &cached).is_err() {
            let _ = fs::remove_file(&tmp);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clean::analysis::tests::write_image;
    use crate::import::metadata::fixture::TempDir;

    #[test]
    fn thumbnails_are_bounded_jpegs_and_cached() {
        let dir = TempDir::new("thumbs");
        let src = dir.path().join("a.png");
        write_image(&src, 1200, 600, |x, _| [(x % 256) as u8, 0, 0]);
        let bytes = thumbnail(dir.path(), 7, 200, &src, Some("abcdef0123456789ffff")).unwrap();
        let img = image::load_from_memory(&bytes).unwrap();
        assert_eq!((img.width(), img.height()), (256, 128));
        let cached = cache_path(dir.path(), 7, 256, Some("abcdef0123456789ffff"));
        assert!(cached.exists());
        // Served from the cache even once the source is gone.
        fs::remove_file(&src).unwrap();
        assert_eq!(
            thumbnail(dir.path(), 7, 256, &src, Some("abcdef0123456789ffff")).unwrap(),
            bytes
        );
    }

    #[test]
    fn small_images_are_not_upscaled_and_sizes_snap() {
        let dir = TempDir::new("thumbs-small");
        let src = dir.path().join("s.png");
        write_image(&src, 40, 30, |_, _| [9, 9, 9]);
        let img =
            image::load_from_memory(&thumbnail(dir.path(), 1, 4000, &src, None).unwrap()).unwrap();
        assert_eq!((img.width(), img.height()), (40, 30));
        assert_eq!((snap(10), snap(600), snap(99999)), (256, 1024, 1024));
    }

    #[test]
    fn decode_failures_are_typed_errors() {
        let dir = TempDir::new("thumbs-bad");
        let bad = dir.path().join("bad.jpg");
        fs::write(&bad, b"nope").unwrap();
        assert!(matches!(
            thumbnail(dir.path(), 1, 256, &bad, None),
            Err(Error::ImageUndecodable { .. })
        ));
        let gone = dir.path().join("gone.jpg");
        assert!(matches!(
            thumbnail(dir.path(), 1, 256, &gone, None),
            Err(Error::ImageUnreadable { .. })
        ));
    }
}

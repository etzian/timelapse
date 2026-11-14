//! Test fixtures: real, decodable JPEG/TIFF bytes with a controlled EXIF block.
//!
//! The JPEG is a valid 1×1 baseline greyscale image (generated once, embedded below) with
//! an APP1 `Exif` segment inserted after SOI. The EXIF block is a little-endian TIFF:
//! IFD0 holding only the `ExifIFDPointer`, then the Exif IFD with the requested tags.
//! Test-only; production code never writes EXIF.

use std::path::{Path, PathBuf};

/// A valid 1×1 greyscale baseline JPEG *without* its leading SOI (`FF D8`).
const JPEG_BODY_AFTER_SOI: &[u8] = &[
    0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01,
    0x00, 0x00, 0xff, 0xdb, 0x00, 0x43, 0x00, 0x10, 0x0b, 0x0c, 0x0e, 0x0c, 0x0a, 0x10, 0x0e, 0x0d,
    0x0e, 0x12, 0x11, 0x10, 0x13, 0x18, 0x28, 0x1a, 0x18, 0x16, 0x16, 0x18, 0x31, 0x23, 0x25, 0x1d,
    0x28, 0x3a, 0x33, 0x3d, 0x3c, 0x39, 0x33, 0x38, 0x37, 0x40, 0x48, 0x5c, 0x4e, 0x40, 0x44, 0x57,
    0x45, 0x37, 0x38, 0x50, 0x6d, 0x51, 0x57, 0x5f, 0x62, 0x67, 0x68, 0x67, 0x3e, 0x4d, 0x71, 0x79,
    0x70, 0x64, 0x78, 0x5c, 0x65, 0x67, 0x63, 0xff, 0xc0, 0x00, 0x0b, 0x08, 0x00, 0x01, 0x00, 0x01,
    0x01, 0x01, 0x11, 0x00, 0xff, 0xc4, 0x00, 0x14, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xc4, 0x00, 0x14, 0x10, 0x01,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xff, 0xda, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3f, 0x00, 0x3f, 0xff, 0xd9,
];

const TAG_EXIF_IFD_POINTER: u16 = 0x8769;
const TAG_DATETIME_ORIGINAL: u16 = 0x9003;
const TAG_DATETIME_DIGITIZED: u16 = 0x9004;
const TAG_OFFSET_TIME_ORIGINAL: u16 = 0x9011;
const TAG_SUBSEC_TIME_ORIGINAL: u16 = 0x9291;
const TYPE_ASCII: u16 = 2;
const TYPE_LONG: u16 = 4;

/// Which EXIF tags the fixture carries. `Default` = an EXIF segment with no date tags.
#[derive(Debug, Clone, Default)]
pub struct ExifSpec {
    pub original: Option<&'static str>,
    pub subsec_original: Option<&'static str>,
    pub offset_original: Option<&'static str>,
    pub digitized: Option<&'static str>,
    /// `true` = no APP1 segment at all (a JPEG straight out of a scanner).
    pub no_segment: bool,
}

impl ExifSpec {
    pub fn original(v: &'static str) -> Self {
        Self { original: Some(v), ..Self::default() }
    }

    pub fn none() -> Self {
        Self { no_segment: true, ..Self::default() }
    }
}

/// A uniquely-named, initially empty temp directory, removed on drop.
pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new(tag: &str) -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static N: AtomicUsize = AtomicUsize::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let p = std::env::temp_dir().join(format!("ts-p2-{tag}-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).expect("create temp dir");
        Self(p)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub fn jpeg_bytes(spec: &ExifSpec) -> Vec<u8> {
    let mut out = vec![0xff, 0xd8];
    if !spec.no_segment {
        let tiff = tiff_bytes(spec);
        let len = u16::try_from(2 + 6 + tiff.len()).expect("APP1 fits");
        out.extend_from_slice(&[0xff, 0xe1]);
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(b"Exif\0\0");
        out.extend_from_slice(&tiff);
    }
    out.extend_from_slice(JPEG_BODY_AFTER_SOI);
    out
}

pub fn write_jpeg(path: &Path, spec: &ExifSpec) {
    std::fs::write(path, jpeg_bytes(spec)).expect("write fixture jpeg");
}

pub fn write_tiff(path: &Path, spec: &ExifSpec) {
    std::fs::write(path, tiff_bytes(spec)).expect("write fixture tiff");
}

fn tiff_bytes(spec: &ExifSpec) -> Vec<u8> {
    let mut tags: Vec<(u16, &str)> = [
        (TAG_DATETIME_ORIGINAL, spec.original),
        (TAG_DATETIME_DIGITIZED, spec.digitized),
        (TAG_OFFSET_TIME_ORIGINAL, spec.offset_original),
        (TAG_SUBSEC_TIME_ORIGINAL, spec.subsec_original),
    ]
    .into_iter()
    .filter_map(|(t, v)| v.map(|v| (t, v)))
    .collect();
    tags.sort_by_key(|(t, _)| *t); // IFD entries must be in ascending tag order

    let exif_ifd_off: u32 = 8 + 2 + 12 + 4;
    let data_off = exif_ifd_off as usize + 2 + tags.len() * 12 + 4;

    let mut t = Vec::new();
    t.extend_from_slice(b"II");
    t.extend_from_slice(&42u16.to_le_bytes());
    t.extend_from_slice(&8u32.to_le_bytes());
    // IFD0: one entry, the pointer to the Exif IFD.
    t.extend_from_slice(&1u16.to_le_bytes());
    entry(&mut t, TAG_EXIF_IFD_POINTER, TYPE_LONG, 1, exif_ifd_off.to_le_bytes());
    t.extend_from_slice(&0u32.to_le_bytes());

    // Exif IFD.
    let mut data = Vec::new();
    t.extend_from_slice(&u16::try_from(tags.len()).unwrap().to_le_bytes());
    for (tag, value) in &tags {
        let mut bytes = value.as_bytes().to_vec();
        bytes.push(0);
        let count = u32::try_from(bytes.len()).unwrap();
        let field = if bytes.len() <= 4 {
            let mut inline = [0u8; 4];
            inline[..bytes.len()].copy_from_slice(&bytes);
            inline
        } else {
            let off = u32::try_from(data_off + data.len()).unwrap();
            data.extend_from_slice(&bytes);
            if !data.len().is_multiple_of(2) {
                data.push(0); // TIFF offsets are word-aligned
            }
            off.to_le_bytes()
        };
        entry(&mut t, *tag, TYPE_ASCII, count, field);
    }
    t.extend_from_slice(&0u32.to_le_bytes());
    debug_assert_eq!(t.len(), data_off);
    t.extend_from_slice(&data);
    t
}

fn entry(t: &mut Vec<u8>, tag: u16, ty: u16, count: u32, value: [u8; 4]) {
    t.extend_from_slice(&tag.to_le_bytes());
    t.extend_from_slice(&ty.to_le_bytes());
    t.extend_from_slice(&count.to_le_bytes());
    t.extend_from_slice(&value);
}

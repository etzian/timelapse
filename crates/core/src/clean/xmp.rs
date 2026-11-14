//! Lightroom / XMP rating, read only (ADR-020).
//!
//! Lightroom marks a rejected photo with `xmp:Rating="-1"`. We look for it in a sidecar
//! next to the photo first (`<stem>.xmp`, or `<file name>.xmp` as darktable writes it),
//! then in an XMP packet embedded near the start of the file itself. Only the `Rating`
//! value is extracted: attribute form (`xmp:Rating="-1"`) or element form
//! (`<xmp:Rating>-1</xmp:Rating>`). Reads are bounded and nothing is ever written.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// How much of a sidecar or an image we read while looking for a packet.
const READ_LIMIT: u64 = 1024 * 1024;
/// Lightroom's "rejected" flag.
pub const REJECTED: i32 = -1;

/// The XMP rating for the photo at `path`, or `None` when no rating is found. A sidecar
/// that carries a rating wins over the packet embedded in the image.
pub fn rating(path: &Path) -> Option<i32> {
    sidecars(path).iter().find_map(|p| read_bounded(p).and_then(|b| rating_in(&b))).or_else(|| {
        let bytes = read_bounded(path)?;
        rating_in(&bytes)
    })
}

fn sidecars(path: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for ext in ["xmp", "XMP"] {
        out.push(path.with_extension(ext));
        if let Some(name) = path.file_name() {
            let mut n = name.to_os_string();
            n.push(".");
            n.push(ext);
            out.push(path.with_file_name(n));
        }
    }
    out
}

fn read_bounded(path: &Path) -> Option<Vec<u8>> {
    let mut buf = Vec::new();
    File::open(path).ok()?.take(READ_LIMIT).read_to_end(&mut buf).ok()?;
    Some(buf)
}

/// The rating inside the first `x:xmpmeta` packet of `bytes`, if any.
pub fn rating_in(bytes: &[u8]) -> Option<i32> {
    let start = find(bytes, b"<x:xmpmeta")?;
    let end = find(&bytes[start..], b"</x:xmpmeta>").map_or(bytes.len(), |e| start + e);
    let packet = String::from_utf8_lossy(&bytes[start..end]);
    attribute(&packet).or_else(|| element(&packet))
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

fn parse(v: &str) -> Option<i32> {
    let f: f64 = v.trim().parse().ok()?;
    (f.is_finite() && (-1.0..=5.0).contains(&f)).then(|| f.round() as i32)
}

fn attribute(packet: &str) -> Option<i32> {
    let i = packet.find("xmp:Rating=")? + "xmp:Rating=".len();
    let rest = &packet[i..];
    let quote = rest.chars().next().filter(|c| *c == '"' || *c == '\'')?;
    let value = &rest[1..];
    parse(&value[..value.find(quote)?])
}

fn element(packet: &str) -> Option<i32> {
    let i = packet.find("<xmp:Rating>")? + "<xmp:Rating>".len();
    let rest = &packet[i..];
    parse(&rest[..rest.find('<')?])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::metadata::fixture::TempDir;

    fn packet(body: &str) -> String {
        format!(
            "<?xpacket begin=''?><x:xmpmeta xmlns:x='adobe:ns:meta/'><rdf:RDF><rdf:Description {body}</rdf:Description></rdf:RDF></x:xmpmeta><?xpacket end='w'?>"
        )
    }

    #[test]
    fn attribute_and_element_forms() {
        assert_eq!(rating_in(packet(r#"xmp:Rating="-1">"#).as_bytes()), Some(-1));
        assert_eq!(rating_in(packet("xmp:Rating='4'>").as_bytes()), Some(4));
        assert_eq!(rating_in(packet("><xmp:Rating>3</xmp:Rating>").as_bytes()), Some(3));
        assert_eq!(rating_in(packet(r#"xmp:Rating="2.0">"#).as_bytes()), Some(2));
    }

    #[test]
    fn malformed_or_absent_ratings_are_none() {
        assert_eq!(rating_in(b"no packet here"), None);
        assert_eq!(rating_in(packet(r#"xmp:Label="Red">"#).as_bytes()), None);
        assert_eq!(rating_in(packet(r#"xmp:Rating="abc">"#).as_bytes()), None);
        assert_eq!(rating_in(packet(r#"xmp:Rating="9">"#).as_bytes()), None, "out of range");
        assert_eq!(rating_in(packet(r#"xmp:Rating="-1"#).as_bytes()), None, "unterminated");
        // A Rating outside the packet is not XMP.
        assert_eq!(rating_in(br#"xmp:Rating="-1" <x:xmpmeta></x:xmpmeta>"#), None);
    }

    #[test]
    fn embedded_packet_in_an_image_file() {
        let dir = TempDir::new("xmp-embedded");
        let p = dir.path().join("a.jpg");
        let mut bytes = vec![0xff, 0xd8, 0xff, 0xe1, 0x00, 0x10];
        // verify:allow on the next line: the XMP APP1 namespace identifier, not a host.
        bytes.extend_from_slice(b"http://ns.adobe.com/xap/1.0/\0"); // verify:allow
        bytes.extend_from_slice(packet(r#"xmp:Rating="-1">"#).as_bytes());
        bytes.extend_from_slice(&[0xff, 0xd9]);
        std::fs::write(&p, bytes).unwrap();
        assert_eq!(rating(&p), Some(REJECTED));
    }

    #[test]
    fn a_sidecar_wins_over_the_embedded_packet() {
        let dir = TempDir::new("xmp-sidecar");
        let p = dir.path().join("IMG_0001.jpg");
        std::fs::write(&p, packet(r#"xmp:Rating="5">"#)).unwrap();
        assert_eq!(rating(&p), Some(5));
        std::fs::write(dir.path().join("IMG_0001.xmp"), packet(r#"xmp:Rating="-1">"#)).unwrap();
        assert_eq!(rating(&p), Some(REJECTED));
    }

    #[test]
    fn darktable_style_sidecar_names_are_found() {
        let dir = TempDir::new("xmp-dt");
        let p = dir.path().join("b.tiff");
        std::fs::write(&p, b"II*\0").unwrap();
        std::fs::write(dir.path().join("b.tiff.xmp"), packet("><xmp:Rating>1</xmp:Rating>"))
            .unwrap();
        assert_eq!(rating(&p), Some(1));
    }

    #[test]
    fn no_sidecar_and_no_packet_is_none() {
        let dir = TempDir::new("xmp-none");
        let p = dir.path().join("c.png");
        std::fs::write(&p, b"\x89PNG").unwrap();
        assert_eq!(rating(&p), None);
        assert_eq!(rating(&dir.path().join("missing.jpg")), None);
    }
}

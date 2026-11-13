//! SHA-256 content-hash computation for exact-duplicate detection.
//!
//! Phase 1 deduplicates on byte equality (`images.file_hash` in
//! `docs/DATABASE_SCHEMA.md`). Perceptual near-dup is out of scope: it would need an
//! additional image library and has not been approved against the permissive-only
//! invariant (DECISIONS.md), so no such dependency is introduced here.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use super::scan::Candidate;
use crate::Error;

/// 8 KiB streaming chunks: large photo files (a 50 MB TIFF is common) are handled
/// without loading the whole file into memory at once.
const CHUNK: usize = 8 * 1024;

pub fn hash_file(path: &Path) -> Result<String, Error> {
    let unreadable = || Error::ImageUnreadable { path: path.to_string_lossy().into_owned() };
    let file: File = fs::File::open(path).map_err(|_| unreadable())?;
    let mut reader = file.take(u64::MAX);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; CHUNK];
    loop {
        let n = reader.read(&mut buf).map_err(|_| unreadable())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex(hasher.finalize().as_slice()))
}

/// Group candidates sharing an identical content hash. Groups with a single member
/// are dropped: only exact duplicates (two or more files, same bytes) are worth surfacing.
pub fn duplicate_groups(candidates: &[Candidate], hashes: &[Option<String>]) -> Vec<DupGroup> {
    debug_assert_eq!(candidates.len(), hashes.len());
    let mut by_hash: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, hash) in hashes.iter().enumerate() {
        if let Some(h) = hash {
            by_hash.entry(h).or_default().push(i);
        }
    }
    by_hash.into_values().filter(|ids| ids.len() >= 2).map(|indices| DupGroup { indices }).collect()
}

#[derive(Debug, Clone)]
pub struct DupGroup {
    /// Indices into the candidate slice that share one content hash.
    pub indices: Vec<usize>,
}

fn hex(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len() * 2);
    for byte in b {
        use std::fmt::Write as _;
        let _ = write!(s, "{byte:02x}");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(sub: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("ts-dedup-{sub}-{}", std::process::id()));
        let _ = fs::create_dir_all(&d);
        d
    }

    #[test]
    fn identical_bytes_produce_identical_hashes() {
        let dir = tmp("ident");
        let a = dir.join("a.bin");
        let b = dir.join("b.bin");
        fs::write(&a, b"hello world").unwrap();
        fs::write(&b, b"hello world").unwrap();
        let ha = hash_file(&a).unwrap();
        let hb = hash_file(&b).unwrap();
        assert_eq!(ha, hb);
        assert_eq!(ha.len(), 64); // 32 bytes → 64 hex chars
    }

    #[test]
    fn different_bytes_produce_different_hashes() {
        let dir = tmp("diff");
        let a = dir.join("a.bin");
        let b = dir.join("b.bin");
        fs::write(&a, b"hello").unwrap();
        fs::write(&b, b"world").unwrap();
        assert_ne!(hash_file(&a).unwrap(), hash_file(&b).unwrap());
    }

    #[test]
    fn groups_two_candidates_with_the_same_hash() {
        let dir = tmp("group");
        let a = dir.join("a.bin");
        fs::write(&a, b"dup").unwrap();

        let mk = |p: &Path| Candidate {
            path: p.to_path_buf(),
            relative_path: p.to_path_buf(),
            filename: "x".into(),
            extension: "bin".into(),
            file_size: 3,
            modified_at_unix: 0,
        };
        let cands = vec![mk(&a), mk(&a.clone())];
        let hashes = vec![Some(hash_file(&a).unwrap()), Some(hash_file(&a).unwrap())];
        assert_eq!(duplicate_groups(&cands, &hashes).len(), 1);
    }
}

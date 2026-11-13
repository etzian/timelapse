//! Filesystem walk: collect supported image candidates under a source root.
//!
//! Read failures on the *root* are a hard error (the user pointed at something
//! that is not there). Read failures on subdirectories are silently skipped so
//! a single locked or permissioned folder does not abort an import of 10k files.

use std::fs;
use std::path::{Path, PathBuf};

use crate::Error;

/// Extensions treated as supported images in Phase 1 (PRD §9: jpg, jpeg, png, tiff).
pub const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "tiff"];

#[derive(Debug, Clone)]
pub struct Candidate {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub filename: String,
    pub extension: String,
    pub file_size: i64,
    pub modified_at_unix: i64,
}

/// Recursively walk `root` and return every supported image found.
pub fn scan(root: &Path) -> Result<Vec<Candidate>, Error> {
    if !root.is_dir() {
        return Err(Error::ImportRootUnreadable { path: root.to_string_lossy().into_owned() });
    }
    let mut out = Vec::new();
    walk(root, root, true, &mut out)?;
    Ok(out)
}

fn walk(dir: &Path, root: &Path, is_root: bool, out: &mut Vec<Candidate>) -> Result<(), Error> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) if !is_root => return Ok(()),
        Err(_) => {
            return Err(Error::ImportRootUnreadable { path: dir.to_string_lossy().into_owned() });
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, root, false, out)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|s| s.to_str()).map(|s| s.to_ascii_lowercase())
        else {
            continue;
        };
        if !SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }
        let Ok(meta) = path.metadata() else {
            continue;
        };
        let relative_path = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        out.push(Candidate {
            path: path.clone(),
            relative_path,
            filename: path
                .file_name()
                .and_then(|s| s.to_str())
                .map(String::from)
                .unwrap_or_default(),
            extension: ext,
            file_size: meta.len() as i64,
            modified_at_unix: meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        });
    }
    Ok(())
}

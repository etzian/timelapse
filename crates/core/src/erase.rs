//! Settings → Erase all data, for the open source folder (ADR-026). The user picks what
//! goes:
//!
//! - **project data**: `<root>/.timelapse/` (database, thumbnails, temporary files);
//! - **exports**: `<root>/Timelapse exports/` (videos, records, contact sheets);
//! - **quarantine**: `<root>/Timelapse quarantine/` (the rejected originals in it);
//! - **copies**: the photographs consolidation *copied* into working folders, each only
//!   while unmodified (the consolidation undo of copy runs). Photographs consolidation
//!   *moved* are the only copy left and are never erased.
//!
//! Nothing is erased while work runs on the folder: every job holds [`job`] for its whole
//! run, and [`erase`] needs the folder alone.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::sync::{Condvar, Mutex, MutexGuard};

use crate::consolidate::{self, Operation};
use crate::import::{open_root, project_dir};
use crate::layout::{exports_dir, quarantine_dir};
use crate::{Error, Result};

/// Per source folder: how many jobs run on it, and whether it is being erased.
struct Work {
    active: BTreeMap<String, usize>,
    erasing: BTreeSet<String>,
}

static WORK: Mutex<Work> = Mutex::new(Work { active: BTreeMap::new(), erasing: BTreeSet::new() });
static ERASED: Condvar = Condvar::new();

fn work() -> MutexGuard<'static, Work> {
    WORK.lock().unwrap_or_else(|e| e.into_inner())
}

/// A source folder as the volume sees it, so two spellings of one folder are one key.
fn key(root: &Path) -> String {
    let s = root.to_string_lossy();
    crate::files::name_key(s.trim_end_matches(['/', '\\']))
}

/// Held by every job on `root` that reads or writes it (import, analysis, quarantine,
/// consolidation, adjustments, contact sheets, exports), so it is never erased under the
/// job. Jobs share it; a job waits only while `root` is being erased.
pub(crate) struct Job(String);

pub(crate) fn job(root: &Path) -> Job {
    let k = key(root);
    let mut w = work();
    while w.erasing.contains(&k) {
        w = ERASED.wait(w).unwrap_or_else(|e| e.into_inner());
    }
    *w.active.entry(k.clone()).or_default() += 1;
    Job(k)
}

impl Drop for Job {
    fn drop(&mut self) {
        let mut w = work();
        if let Some(n) = w.active.get_mut(&self.0) {
            *n -= 1;
            if *n == 0 {
                w.active.remove(&self.0);
            }
        }
    }
}

/// `root` held for an erase: no job runs on it, none starts until this is dropped.
struct Alone(String);

fn alone(root: &Path) -> Option<Alone> {
    let k = key(root);
    let mut w = work();
    if w.active.contains_key(&k) || w.erasing.contains(&k) {
        return None;
    }
    w.erasing.insert(k.clone());
    Some(Alone(k))
}

impl Drop for Alone {
    fn drop(&mut self) {
        work().erasing.remove(&self.0);
        ERASED.notify_all();
    }
}

fn fail(reason: &'static str) -> Error {
    Error::Erase { reason }
}

/// One folder that can be erased.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct Folder {
    pub path: String,
    pub exists: bool,
    pub files: u64,
    pub bytes: u64,
}

/// What consolidation placed in working folders and is still there.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct Copies {
    /// Working folders of copy runs not yet undone.
    pub folders: Vec<String>,
    /// Copies still present (erased only if still unmodified).
    pub files: u64,
    pub bytes: u64,
    /// Photographs moved (not copied): never erased.
    pub moved: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct Inventory {
    pub project_data: Folder,
    pub exports: Folder,
    pub quarantine: Folder,
    pub copies: Copies,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize)]
pub struct Targets {
    pub project_data: bool,
    pub exports: bool,
    pub quarantine: bool,
    pub copies: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct Report {
    pub project_data: bool,
    pub exports: bool,
    pub quarantine: bool,
    /// Copies removed.
    pub copies_removed: u64,
    /// Copies left because they changed, went missing or are now someone's original.
    pub copies_kept: u64,
}

fn folder(path: &Path) -> Folder {
    fn walk(p: &Path, f: &mut Folder) {
        let Ok(entries) = fs::read_dir(p) else { return };
        for e in entries.flatten() {
            match e.file_type() {
                Ok(t) if t.is_dir() => walk(&e.path(), f),
                Ok(_) => {
                    f.files += 1;
                    f.bytes += e.metadata().map(|m| m.len()).unwrap_or(0);
                }
                Err(_) => {}
            }
        }
    }
    let mut f = Folder { path: path.to_string_lossy().into_owned(), ..Folder::default() };
    f.exists = path.is_dir();
    walk(path, &mut f);
    f
}

/// What [`erase`] would remove for `root`. Never creates the project data.
pub fn inventory(root: &Path) -> Result<Inventory> {
    let copies = if project_dir(root).is_dir() { copies(root)? } else { Copies::default() };
    Ok(Inventory {
        project_data: folder(&project_dir(root)),
        exports: folder(&exports_dir(root)),
        quarantine: folder(&quarantine_dir(root)),
        copies,
    })
}

fn copies(root: &Path) -> Result<Copies> {
    let conn = open_root(root)?;
    let mut out = Copies::default();
    for run in consolidate::store::list(&conn)? {
        if run.undo_status.as_deref() == Some("undone") {
            continue;
        }
        let undone = consolidate::store::undone_images(&conn, run.id)?;
        let placed: Vec<_> = crate::files::completed_placements(&conn, run.id)?
            .into_iter()
            .filter(|(op, _, _)| op.image_id.is_some_and(|i| !undone.contains(&i)))
            .collect();
        match run.config.operation {
            Operation::Move => out.moved += placed.len() as u64,
            Operation::Copy => {
                let mut any = false;
                for (op, _, _) in &placed {
                    if let Ok(m) = fs::metadata(&op.destination_path) {
                        out.files += 1;
                        out.bytes += m.len();
                        any = true;
                    }
                }
                let dest = run.config.destination.to_string_lossy().into_owned();
                if any && !out.folders.contains(&dest) {
                    out.folders.push(dest);
                }
            }
        }
    }
    Ok(out)
}

/// Erase what `targets` names for `root`; `busy` while any work runs. Copies go first
/// (their journal is in the project data), the project data last.
pub fn erase(root: &Path, targets: Targets) -> Result<Report> {
    let _alone = alone(root).ok_or_else(|| fail("busy"))?;
    let mut report = Report::default();
    let data = project_dir(root);

    if targets.copies && data.is_dir() {
        let runs = {
            let conn = open_root(root)?;
            consolidate::store::list(&conn)?
        };
        for run in runs {
            if run.config.operation != Operation::Copy
                || run.undo_status.as_deref() == Some("undone")
            {
                continue;
            }
            let after = consolidate::undo_unguarded(root, run.id, 0, |_| {})?;
            report.copies_removed += after.undone.saturating_sub(run.undone) as u64;
            report.copies_kept += after.placed.saturating_sub(after.undone) as u64;
        }
    }
    if targets.exports {
        report.exports = remove(&exports_dir(root))?;
    }
    if targets.quarantine {
        report.quarantine = remove(&quarantine_dir(root))?;
    }
    if targets.project_data {
        // Originals still inside (a pre-ADR-026 quarantine that could not be moved out)
        // are photographs, not project data: refuse rather than delete them.
        let legacy = data.join("quarantine");
        if !targets.quarantine && folder(&legacy).files > 0 {
            return Err(fail("quarantine-inside"));
        }
        report.project_data = remove(&data)?;
    }
    Ok(report)
}

fn remove(dir: &Path) -> Result<bool> {
    match fs::remove_dir_all(dir) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(fail("remove-failed")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::import_folder;
    use crate::import::metadata::fixture::TempDir;

    fn photo(p: &Path, shade: u8) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        image::RgbImage::from_pixel(8, 8, image::Rgb([shade, 0, 0])).save(p).unwrap();
    }

    #[test]
    fn nothing_is_erased_while_work_runs() {
        let dir = TempDir::new("erase-busy");
        photo(&dir.path().join("a.jpg"), 10);
        import_folder(dir.path()).unwrap();
        let all = Targets { project_data: true, ..Targets::default() };
        {
            let _running = job(dir.path());
            assert!(matches!(erase(dir.path(), all), Err(Error::Erase { reason: "busy" })));
        }
        assert!(project_dir(dir.path()).is_dir());
        assert!(erase(dir.path(), all).unwrap().project_data);
        assert!(!project_dir(dir.path()).exists());
        assert!(dir.path().join("a.jpg").is_file(), "originals are never project data");
    }

    #[test]
    fn each_target_goes_only_when_chosen() {
        let dir = TempDir::new("erase-targets");
        let root = dir.path();
        photo(&root.join("a.jpg"), 10);
        import_folder(root).unwrap();
        fs::create_dir_all(exports_dir(root)).unwrap();
        fs::write(exports_dir(root).join("v.mp4"), b"video").unwrap();
        photo(&quarantine_dir(root).join("r.jpg"), 20);

        let inv = inventory(root).unwrap();
        assert!(inv.project_data.exists && inv.project_data.files > 0);
        assert_eq!((inv.exports.files, inv.exports.bytes), (1, 5));
        assert_eq!(inv.quarantine.files, 1);

        let r = erase(root, Targets { exports: true, ..Targets::default() }).unwrap();
        assert!(r.exports && !r.quarantine && !r.project_data);
        assert!(!exports_dir(root).exists());
        assert!(quarantine_dir(root).join("r.jpg").is_file());
        assert!(project_dir(root).is_dir());

        // A pre-ADR-026 quarantine left inside the project data blocks erasing it alone.
        photo(&project_dir(root).join("quarantine").join("old.jpg"), 30);
        let data_only = Targets { project_data: true, ..Targets::default() };
        assert!(matches!(
            erase(root, data_only),
            Err(Error::Erase { reason: "quarantine-inside" })
        ));
        let r = erase(root, Targets { project_data: true, quarantine: true, ..Targets::default() })
            .unwrap();
        assert!(r.project_data && r.quarantine);
        assert!(!quarantine_dir(root).exists() && !project_dir(root).exists());
        assert!(root.join("a.jpg").is_file());

        // Nothing there: nothing to do, no error, and no project data created.
        let inv = inventory(root).unwrap();
        assert!(!inv.project_data.exists && !inv.exports.exists);
        assert!(!project_dir(root).exists());
    }
}

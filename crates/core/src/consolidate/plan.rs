//! The consolidation plan: where each photograph goes, as a pure function of the ordered
//! input, the naming rule and what already exists at the destination (ADR-023).

use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

/// Copy (the originals stay) or move (the originals go to the working folder).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Operation {
    Copy,
    Move,
}

impl Operation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Copy => "copy",
            Self::Move => "move",
        }
    }
}

/// How targets are named (PRD §5.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Naming {
    /// The relative path in the collection.
    Preserve,
    /// Flat, original filenames.
    Original,
    /// Flat, `<parent folder>_<filename>`.
    FolderPrefix,
    /// Flat, `000001.ext`… in input order.
    Sequence,
}

/// One photograph to place, in the order the plan follows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub image_id: i64,
    /// Where the file is now (its location of record).
    pub source: PathBuf,
    /// Its path relative to the collection root, as imported.
    pub relative_path: String,
    pub size: u64,
    pub hash: Option<String>,
}

/// What is at a candidate target before anything is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Existing {
    Absent,
    /// A file with exactly the bytes of the photograph.
    Identical,
    /// Anything else (another file, a folder, something unreadable).
    Other,
}

/// What the plan does with one photograph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum Action {
    /// Copy or move it to `target`.
    Place,
    /// Byte-identical to an earlier photograph of the plan, which is placed instead.
    Duplicate { of: i64 },
    /// An identical file is already at `target`: nothing to do, and never ours to undo.
    AlreadyPresent,
    /// Its name cannot be turned into a safe target.
    Unsafe,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Planned {
    pub image_id: i64,
    #[serde(skip)]
    pub source: PathBuf,
    pub relative_path: String,
    /// Relative to the destination, `/`-separated; `None` for duplicates and unsafe names.
    pub target: Option<String>,
    pub size: u64,
    #[serde(skip)]
    pub hash: Option<String>,
    pub action: Action,
    /// The natural name was taken, so a `_N` suffix was added.
    pub renamed: bool,
}

/// Highest `_N` suffix tried before a name is declared unusable.
const MAX_SUFFIX: u32 = 100_000;

/// Plan `inputs` (already in order). `probe(target)` reports what is at a relative target
/// for the photograph being placed; it is only asked about names the plan has not taken.
pub fn plan(
    inputs: &[Input],
    naming: Naming,
    mut probe: impl FnMut(&str, &Input) -> Existing,
) -> Vec<Planned> {
    let mut first_of_hash: HashMap<&str, i64> = HashMap::new();
    let mut taken: HashSet<String> = HashSet::new();
    let placed_total = count_placeable(inputs);
    let width = placed_total.to_string().len().max(6);
    let mut sequence = 0usize;
    let mut out = Vec::with_capacity(inputs.len());

    for input in inputs {
        let base = Planned {
            image_id: input.image_id,
            source: input.source.clone(),
            relative_path: input.relative_path.clone(),
            target: None,
            size: input.size,
            hash: input.hash.clone(),
            action: Action::Unsafe,
            renamed: false,
        };
        if let Some(h) = input.hash.as_deref() {
            if let Some(&of) = first_of_hash.get(h) {
                out.push(Planned { action: Action::Duplicate { of }, ..base });
                continue;
            }
            first_of_hash.insert(h, input.image_id);
        }
        let natural = match naming {
            Naming::Sequence => {
                sequence += 1;
                components(&input.relative_path).and_then(|parts| {
                    let ext = extension(parts.last()?)?.to_lowercase();
                    Some(vec![format!("{sequence:0width$}.{ext}")])
                })
            }
            _ => target_parts(&input.relative_path, naming),
        };
        let Some(parts) = natural.filter(|p| p.iter().all(|c| is_safe_component(c))) else {
            out.push(base);
            continue;
        };
        out.push(match free_name(&parts, &mut taken, |t| probe(t, input)) {
            Some((target, existing, renamed)) => Planned {
                target: Some(target),
                action: if existing == Existing::Identical {
                    Action::AlreadyPresent
                } else {
                    Action::Place
                },
                renamed,
                ..base
            },
            None => base,
        });
    }
    out
}

/// How many inputs are not byte-identical to an earlier one (sizes the sequence numbers).
fn count_placeable(inputs: &[Input]) -> usize {
    let mut seen = HashSet::new();
    inputs.iter().filter(|i| i.hash.as_deref().is_none_or(|h| seen.insert(h))).count()
}

/// The first name, from the natural one through `_2`, `_3`…, that the plan has not taken
/// and that holds nothing different. Returns it, what is there, and whether it was renamed.
fn free_name(
    parts: &[String],
    taken: &mut HashSet<String>,
    mut probe: impl FnMut(&str) -> Existing,
) -> Option<(String, Existing, bool)> {
    let (dirs, file) = parts.split_at(parts.len() - 1);
    let file = &file[0];
    for n in 1..=MAX_SUFFIX {
        let name = if n == 1 { file.clone() } else { with_suffix(file, n) };
        let mut target = dirs.join("/");
        if !target.is_empty() {
            target.push('/');
        }
        target.push_str(&name);
        let key = crate::files::name_key(&target);
        if taken.contains(&key) {
            continue;
        }
        let existing = probe(&target);
        if existing == Existing::Other {
            continue;
        }
        taken.insert(key);
        return Some((target, existing, n > 1));
    }
    None
}

/// `IMG_0001.JPG` + 2 → `IMG_0001_2.JPG`.
fn with_suffix(file: &str, n: u32) -> String {
    match file.rfind('.') {
        Some(dot) if dot > 0 => format!("{}_{n}{}", &file[..dot], &file[dot..]),
        _ => format!("{file}_{n}"),
    }
}

fn extension(file: &str) -> Option<&str> {
    let dot = file.rfind('.').filter(|&d| d > 0)?;
    let ext = &file[dot + 1..];
    (!ext.is_empty()).then_some(ext)
}

/// The normal components of a stored relative path, or `None` if it has any other kind.
fn components(relative: &str) -> Option<Vec<String>> {
    let mut parts = Vec::new();
    for c in Path::new(relative).components() {
        match c {
            Component::Normal(s) => parts.push(s.to_str()?.to_owned()),
            _ => return None,
        }
    }
    (!parts.is_empty()).then_some(parts)
}

fn target_parts(relative: &str, naming: Naming) -> Option<Vec<String>> {
    let parts = components(relative)?;
    let file = parts.last()?.clone();
    Some(match naming {
        Naming::Preserve => parts,
        Naming::Original => vec![file],
        Naming::FolderPrefix => match parts.len() {
            1 => vec![file],
            n => vec![format!("{}_{file}", parts[n - 2])],
        },
        Naming::Sequence => unreachable!("handled by the caller"),
    })
}

/// A single path component that is safe to create on macOS and Windows.
fn is_safe_component(c: &str) -> bool {
    !c.is_empty()
        && c != "."
        && c != ".."
        && !c.chars().any(|ch| matches!(ch, '/' | '\\' | ':' | '\0') || ch.is_control())
        && matches!(Path::new(c).components().next(), Some(Component::Normal(_)))
        && Path::new(c).components().count() == 1
}

/// Join a planned `/`-separated target under `base`.
pub fn join_target(base: &Path, target: &str) -> Option<PathBuf> {
    let mut out = base.to_path_buf();
    for part in target.split('/') {
        if !is_safe_component(part) {
            return None;
        }
        out.push(part);
    }
    Some(out)
}

/// SHA-256 over the operation, the destination and each `(id, action, target)` line: what
/// the user confirmed. Execution refuses a plan whose fingerprint differs.
pub fn fingerprint(operation: Operation, destination: &Path, items: &[Planned]) -> String {
    let mut h = Sha256::new();
    h.update(operation.as_str().as_bytes());
    h.update(b"\n");
    h.update(destination.to_string_lossy().as_bytes());
    h.update(b"\n");
    for p in items {
        let action = match p.action {
            Action::Place => "place".to_owned(),
            Action::Duplicate { of } => format!("duplicate:{of}"),
            Action::AlreadyPresent => "present".to_owned(),
            Action::Unsafe => "unsafe".to_owned(),
        };
        h.update(format!("{}\t{action}\t{}\n", p.image_id, p.target.as_deref().unwrap_or("")));
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(id: i64, rel: &str, hash: Option<&str>) -> Input {
        Input {
            image_id: id,
            source: PathBuf::from(format!("/src/{rel}")),
            relative_path: Path::new(rel).to_string_lossy().into_owned(),
            size: 10,
            hash: hash.map(String::from),
        }
    }

    fn rel(parts: &[&str]) -> String {
        parts.iter().collect::<PathBuf>().to_string_lossy().into_owned()
    }

    fn targets(p: &[Planned]) -> Vec<Option<&str>> {
        p.iter().map(|x| x.target.as_deref()).collect()
    }

    fn absent(_: &str, _: &Input) -> Existing {
        Existing::Absent
    }

    #[test]
    fn preserve_keeps_the_structure() {
        let ins = [
            input(1, &rel(&["157gopro", "G001.JPG"]), Some("a")),
            input(2, &rel(&["158gopro", "G001.JPG"]), Some("b")),
        ];
        let p = plan(&ins, Naming::Preserve, absent);
        assert_eq!(targets(&p), [Some("157gopro/G001.JPG"), Some("158gopro/G001.JPG")]);
        assert!(p.iter().all(|x| x.action == Action::Place && !x.renamed));
    }

    #[test]
    fn original_names_collide_case_insensitively_and_get_suffixes() {
        let ins = [
            input(1, &rel(&["a", "G001.JPG"]), Some("a")),
            input(2, &rel(&["b", "g001.jpg"]), Some("b")),
            input(3, &rel(&["c", "G001.JPG"]), Some("c")),
        ];
        let p = plan(&ins, Naming::Original, absent);
        assert_eq!(targets(&p), [Some("G001.JPG"), Some("g001_2.jpg"), Some("G001_3.JPG")]);
        assert_eq!(p.iter().map(|x| x.renamed).collect::<Vec<_>>(), [false, true, true]);
    }

    #[test]
    fn names_differing_only_in_normalisation_collide() {
        // "café" precomposed (NFC) and decomposed (NFD, as macOS may store it).
        let ins = [
            input(1, &rel(&["a", "caf\u{e9}.jpg"]), Some("a")),
            input(2, &rel(&["b", "cafe\u{301}.jpg"]), Some("b")),
        ];
        let p = plan(&ins, Naming::Original, absent);
        assert_eq!(targets(&p), [Some("caf\u{e9}.jpg"), Some("cafe\u{301}_2.jpg")]);
        assert_eq!(p.iter().map(|x| x.renamed).collect::<Vec<_>>(), [false, true]);
    }

    #[test]
    fn folder_prefix_uses_the_parent_folder() {
        let ins = [input(1, &rel(&["157gopro", "G001.JPG"]), None), input(2, "top.png", None)];
        let p = plan(&ins, Naming::FolderPrefix, absent);
        assert_eq!(targets(&p), [Some("157gopro_G001.JPG"), Some("top.png")]);
    }

    #[test]
    fn sequence_numbers_in_order_with_lowercase_extensions() {
        let ins = [
            input(1, "x.JPG", Some("a")),
            input(2, "y.png", Some("b")),
            input(3, "z.jpg", Some("a")),
        ];
        let p = plan(&ins, Naming::Sequence, absent);
        assert_eq!(targets(&p), [Some("000001.jpg"), Some("000002.png"), None]);
        assert_eq!(p[2].action, Action::Duplicate { of: 1 });
    }

    #[test]
    fn sequence_width_grows_past_six_digits() {
        let ins: Vec<_> = (0..1_000_001).map(|i| input(i, "x.jpg", None)).collect();
        let p = plan(&ins, Naming::Sequence, absent);
        assert_eq!(p[0].target.as_deref(), Some("0000001.jpg"));
        assert_eq!(p[1_000_000].target.as_deref(), Some("1000001.jpg"));
    }

    #[test]
    fn duplicates_keep_the_first_and_no_hash_is_never_a_duplicate() {
        let ins = [
            input(1, &rel(&["a", "1.jpg"]), Some("h")),
            input(2, &rel(&["b", "1.jpg"]), Some("h")),
            input(3, &rel(&["c", "1.jpg"]), None),
            input(4, &rel(&["d", "1.jpg"]), None),
        ];
        let p = plan(&ins, Naming::Original, absent);
        assert_eq!(p[1].action, Action::Duplicate { of: 1 });
        assert_eq!(targets(&p), [Some("1.jpg"), None, Some("1_2.jpg"), Some("1_3.jpg")]);
    }

    #[test]
    fn existing_files_are_skipped_when_identical_and_avoided_otherwise() {
        let ins = [input(1, "a.jpg", Some("h1")), input(2, "b.jpg", Some("h2"))];
        let p = plan(&ins, Naming::Original, |t, _| match t {
            "a.jpg" => Existing::Identical,
            "b.jpg" | "b_2.jpg" => Existing::Other,
            _ => Existing::Absent,
        });
        assert_eq!(p[0].action, Action::AlreadyPresent);
        assert_eq!(p[0].target.as_deref(), Some("a.jpg"));
        assert_eq!((p[1].target.as_deref(), p[1].renamed), (Some("b_3.jpg"), true));
    }

    #[test]
    fn unsafe_names_are_refused() {
        for bad in ["", "..", "a/../b.jpg"] {
            let p = plan(&[input(1, bad, None)], Naming::Preserve, absent);
            assert_eq!(p[0].action, Action::Unsafe, "{bad:?}");
            assert_eq!(p[0].target, None);
        }
        assert!(join_target(Path::new("/d"), "a/b.jpg").is_some());
        assert!(join_target(Path::new("/d"), "../b.jpg").is_none());
        assert!(join_target(Path::new("/d"), "a//b.jpg").is_none());
        assert!(join_target(Path::new("/d"), "C:x.jpg").is_none());
    }

    #[test]
    fn suffixes_go_before_the_extension() {
        assert_eq!(with_suffix("a.tar.jpg", 2), "a.tar_2.jpg");
        assert_eq!(with_suffix(".hidden", 2), ".hidden_2");
        assert_eq!(with_suffix("noext", 3), "noext_3");
    }

    #[test]
    fn the_plan_and_its_fingerprint_are_deterministic() {
        let ins = [
            input(1, "a.jpg", Some("x")),
            input(2, "a.jpg", Some("y")),
            input(3, "c.jpg", Some("x")),
        ];
        let a = plan(&ins, Naming::Original, absent);
        let b = plan(&ins, Naming::Original, absent);
        assert_eq!(a, b);
        let dest = Path::new("/d");
        assert_eq!(fingerprint(Operation::Copy, dest, &a), fingerprint(Operation::Copy, dest, &b));
        assert_ne!(fingerprint(Operation::Copy, dest, &a), fingerprint(Operation::Move, dest, &a));
        let c = plan(&ins, Naming::Sequence, absent);
        assert_ne!(fingerprint(Operation::Copy, dest, &a), fingerprint(Operation::Copy, dest, &c));
    }

    #[test]
    fn empty_input_is_an_empty_plan() {
        assert!(plan(&[], Naming::Sequence, absent).is_empty());
    }
}

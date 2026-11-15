//! Phase 7 exit criterion (DEVELOPMENT_ROADMAP.md): contact sheets are generated from
//! SQLite state only; layout/timestamp configurable and both locales render correctly.
//! Full path on real files: import → statuses / sampling / export rows → sheet.

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use super::pdf::tests::{check_structure, has_text};
use super::*;
use crate::import::metadata::fixture::TempDir;
use crate::import::{backfill_for_root, import_folder};
use crate::sampling::{Config, Method, create_run_for_root};

/// Sheets share the process-wide lock; run these one at a time so none sees `busy`.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

/// Capture order (mtimes, one hour apart) is the order of this list.
const PHOTOS: [&str; 7] = [
    "Québec/été_01.jpg",
    "Québec/été_02.jpg",
    "a/IMG_0003.jpg",
    "a/IMG_0004.jpg",
    "b/IMG_0005.jpg",
    "b/broken.jpg",
    "b/IMG_0007.jpg",
];

struct Fixture(TempDir);

impl Fixture {
    fn new(tag: &str) -> Self {
        let dir = TempDir::new(&format!("p7-{tag}"));
        let t0 = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        for (i, rel) in PHOTOS.iter().enumerate() {
            let p = dir.path().join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            if rel.ends_with("broken.jpg") {
                fs::write(&p, b"not a jpeg at all").unwrap();
            } else {
                let shade = 30 * i as u8;
                let w = if i % 2 == 0 { 64 } else { 40 };
                image::RgbImage::from_pixel(w, 48, image::Rgb([shade, 100, 200 - shade]))
                    .save(&p)
                    .unwrap();
            }
            let f = fs::File::options().write(true).open(&p).unwrap();
            f.set_modified(t0 + Duration::from_secs(3600 * i as u64)).unwrap();
        }
        import_folder(dir.path()).unwrap();
        backfill_for_root(dir.path()).unwrap();
        Self(dir)
    }

    fn root(&self) -> &Path {
        self.0.path()
    }

    fn conn(&self) -> rusqlite::Connection {
        open_root(self.root()).unwrap()
    }

    fn id(&self, rel: &str) -> i64 {
        self.conn()
            .query_row(
                "SELECT id FROM images WHERE replace(relative_path, '\\', '/') = ?1",
                [rel],
                |r| r.get(0),
            )
            .unwrap()
    }

    fn set_status(&self, rel: &str, status: &str) {
        self.conn()
            .execute(
                "INSERT INTO image_status (image_id, status, updated_at) VALUES (?1, ?2, 0)
                 ON CONFLICT (image_id) DO UPDATE SET status = excluded.status",
                rusqlite::params![self.id(rel), status],
            )
            .unwrap();
    }

    fn names(&self, source: &Source) -> Vec<String> {
        store::items(&self.conn(), source)
            .unwrap()
            .into_iter()
            .map(|i| i.relative_path.replace('\\', "/"))
            .collect()
    }

    fn sheet(&self, source: &Source, layout: &Layout, labels: &Labels) -> (SheetRecord, Vec<u8>) {
        let r = generate(self.root(), source, layout, labels, 0, |_| {}).unwrap();
        assert_eq!(r.status, "completed", "{r:?}");
        let bytes = fs::read(&r.output_path).unwrap();
        (r, bytes)
    }
}

fn labels_en() -> Labels {
    Labels {
        title: "Contact sheet — Summer’s end".into(),
        source: "All photographs · 7 photographs".into(),
        page: "Page {page} of {pages}".into(),
        note: "Times in UTC · Timelapse Studio".into(),
        unreadable: "Not readable".into(),
    }
}

fn labels_fr() -> Labels {
    Labels {
        title: "Planche contact — Été à Québec, ça déménage".into(),
        source: "Toutes les photographies · 7 photographies".into(),
        page: "Page \u{2068}{page}\u{2069} de \u{2068}{pages}\u{2069}".into(),
        note: "Heures en UTC · Timelapse Studio".into(),
        unreadable: "Illisible".into(),
    }
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn each_source_selects_and_orders_from_sqlite() {
    let _s = serial();
    let f = Fixture::new("sources");
    assert_eq!(f.names(&Source::All), names(&PHOTOS));

    // Normalised times are 2023-11-14T22:13:20Z + i hours.
    let period = Source::Period { from: "2023-11-15T00".into(), to: "2023-11-15T02".into() };
    assert_eq!(f.names(&period), names(&PHOTOS[2..4]));

    f.set_status("a/IMG_0004.jpg", "flagged");
    f.set_status("Québec/été_01.jpg", "flagged");
    f.set_status("b/IMG_0005.jpg", "quarantined");
    f.set_status("a/IMG_0003.jpg", "eligible");
    assert_eq!(f.names(&Source::Flagged), names(&["Québec/été_01.jpg", "a/IMG_0004.jpg"]));
    assert_eq!(f.names(&Source::Quarantined), names(&["b/IMG_0005.jpg"]));

    let run = create_run_for_root(
        f.root(),
        &Config {
            method: Method::EveryNth { n: 2, offset: 0 },
            from: None,
            to: None,
            include_flagged: true,
        },
    )
    .unwrap();
    let selected = f.names(&Source::Sampling { run: run.id });
    let expected: Vec<String> = crate::sampling::run_items_for_root(f.root(), run.id)
        .unwrap()
        .into_iter()
        .map(|i| i.relative_path.replace('\\', "/"))
        .collect();
    assert!(!selected.is_empty());
    assert_eq!(selected, expected);

    f.conn()
        .execute(
            "INSERT INTO video_exports (sampling_run_id, settings_json, argv_json, output_path,
                 status, input_photos, output_frames, duration_seconds, created_at)
             VALUES (?1, '{}', '[]', 'x.mp4', 'completed', 1, 1, 1.0, 0)",
            [run.id],
        )
        .unwrap();
    let export: i64 =
        f.conn().query_row("SELECT max(id) FROM video_exports", [], |r| r.get(0)).unwrap();
    assert_eq!(f.names(&Source::Export { export }), expected);

    let conn = f.conn();
    assert!(matches!(
        store::items(&conn, &Source::Export { export: 999 }),
        Err(Error::ContactSheet { reason: "unknown-export" })
    ));
    assert!(matches!(
        store::items(&conn, &Source::Sampling { run: 999 }),
        Err(Error::UnknownRun { id: 999 })
    ));
    let backwards = Source::Period { from: "2024".into(), to: "2023".into() };
    assert!(matches!(
        store::items(&conn, &backwards),
        Err(Error::ContactSheet { reason: "invalid-period" })
    ));
}

#[test]
fn a_sheet_comes_from_sqlite_alone_and_is_reproducible() {
    let _s = serial();
    let f = Fixture::new("sqlite-only");
    let layout = Layout { columns: 3, rows: 2, ..Layout::default() };
    let (first, bytes) = f.sheet(&Source::All, &layout, &labels_en());
    assert_eq!((first.photos, first.pages, first.unreadable), (7, 2, 1));
    assert_eq!(check_structure(&bytes), 2);
    assert_eq!(first.file_size, Some(bytes.len() as u64));
    assert!(Path::new(&first.output_path).starts_with(sheets_dir(f.root())));

    // The thumbnails are cached now: remove every original and generate again.
    for rel in PHOTOS {
        fs::remove_file(f.root().join(rel)).unwrap();
    }
    let (second, again) = f.sheet(&Source::All, &layout, &labels_en());
    assert_ne!(first.id, second.id);
    assert_eq!(again, bytes, "same state and configuration, same bytes");

    let list = sheets_for_root(f.root()).unwrap();
    assert_eq!(list.iter().map(|r| r.id).collect::<Vec<_>>(), [second.id, first.id]);
    assert_eq!(list[0].source, Source::All);
    assert_eq!(list[0].layout, layout);
}

#[test]
fn an_unreadable_photograph_is_a_placeholder_not_a_failure() {
    let _s = serial();
    let f = Fixture::new("missing");
    fs::remove_file(f.root().join("a/IMG_0004.jpg")).unwrap();
    let (r, bytes) = f.sheet(&Source::All, &Layout::default(), &labels_en());
    assert_eq!((r.photos, r.unreadable), (7, 2), "the missing one and the broken one");
    assert!(has_text(&bytes, "Not readable"));
    assert_eq!(String::from_utf8_lossy(&bytes).matches("/Subtype /Image").count(), 5);
}

#[test]
fn layout_and_captions_are_configurable() {
    let _s = serial();
    let f = Fixture::new("layout");
    let grid = |columns, rows| Layout { columns, rows, ..Layout::default() };
    let (r, b) = f.sheet(&Source::All, &grid(2, 2), &labels_en());
    assert_eq!((r.pages, check_structure(&b)), (2, 2));
    let (r, b) = f.sheet(&Source::All, &grid(3, 3), &labels_en());
    assert_eq!((r.pages, check_structure(&b)), (1, 1));
    let landscape =
        Layout { orientation: Orientation::Landscape, page_size: PageSize::Letter, ..grid(1, 1) };
    let (r, b) = f.sheet(&Source::All, &landscape, &labels_en());
    assert_eq!((r.pages, check_structure(&b)), (7, 7));
    assert!(String::from_utf8_lossy(&b).contains("/MediaBox [0 0 792 612]"));

    let caption = "a/IMG_0003.jpg";
    let (_, both) = f.sheet(&Source::All, &grid(4, 2), &labels_en());
    assert!(has_text(&both, caption));
    assert!(has_text(&both, "2023-11-15 00:13:20"));
    let only_time = Layout { show_filename: false, ..grid(4, 2) };
    let (_, t) = f.sheet(&Source::All, &only_time, &labels_en());
    assert!(!has_text(&t, caption) && has_text(&t, "2023-11-15 00:13:20"));
    let nothing = Layout { show_filename: false, show_timestamp: false, ..grid(4, 2) };
    let (_, n) = f.sheet(&Source::All, &nothing, &labels_en());
    assert!(!has_text(&n, caption) && !has_text(&n, "2023-11-15 00:13:20"));
}

#[test]
fn both_locales_render_their_labels() {
    let _s = serial();
    let f = Fixture::new("locales");
    let layout = Layout { columns: 2, rows: 2, ..Layout::default() };
    let (_, fr) = f.sheet(&Source::All, &layout, &labels_fr());
    assert!(has_text(&fr, "Planche contact — Été à Québec, ça déménage"));
    assert!(has_text(&fr, "Page 2 de 2"), "placeholders filled, isolation marks dropped");
    assert!(has_text(&fr, "Illisible"));
    let accented = "Québec/été_01.jpg";
    assert!(has_text(&fr, accented), "accented file names stay readable");

    let (_, en) = f.sheet(&Source::All, &layout, &labels_en());
    assert!(has_text(&en, "Contact sheet — Summer’s end"));
    assert!(has_text(&en, "Page 1 of 2"));
    assert!(!has_text(&en, "Illisible") && !has_text(&fr, "Not readable"));
}

#[test]
fn cancel_empty_and_busy_leave_nothing_behind() {
    let _s = serial();
    let f = Fixture::new("cancel");
    let token = 4242;
    cancel(token);
    let r =
        generate(f.root(), &Source::All, &Layout::default(), &labels_en(), token, |_| {}).unwrap();
    cancel(0);
    assert_eq!(r.status, "cancelled");
    assert!(!Path::new(&r.output_path).exists());
    assert_eq!(fs::read_dir(sheets_dir(f.root())).unwrap().count(), 0, "no partial file");

    assert!(matches!(
        generate(f.root(), &Source::Quarantined, &Layout::default(), &labels_en(), 0, |_| {}),
        Err(Error::ContactSheet { reason: "empty" })
    ));

    let _held = try_lock().unwrap();
    assert!(matches!(
        generate(f.root(), &Source::All, &Layout::default(), &labels_en(), 0, |_| {}),
        Err(Error::ContactSheet { reason: "busy" })
    ));
}

#[test]
fn a_crashed_sheet_is_settled_and_its_temporary_file_removed() {
    let _s = serial();
    let f = Fixture::new("crash");
    let conn = f.conn();
    let id = store::begin(&conn, &Source::All, &Layout::default(), &labels_en(), 7, 1).unwrap();
    fs::create_dir_all(sheets_dir(f.root())).unwrap();
    fs::write(part_path(&sheets_dir(f.root()), id), b"half a pdf").unwrap();
    let list = sheets_for_root(f.root()).unwrap();
    assert_eq!(list[0].status, "interrupted");
    assert!(!part_path(&sheets_dir(f.root()), id).exists());
}

#[test]
fn the_preview_matches_the_first_page() {
    let _s = serial();
    let f = Fixture::new("preview");
    let layout = Layout { columns: 2, rows: 2, show_filename: false, ..Layout::default() };
    let p = preview(f.root(), &Source::All, &layout).unwrap();
    assert_eq!((p.photos, p.pages, p.too_many), (7, 2, false));
    assert_eq!(p.first_page.len(), 4);
    assert_eq!(p.first_page[0].image_id, f.id(PHOTOS[0]));
    assert_eq!(p.first_page[0].filename, None);
    assert_eq!(p.first_page[2].timestamp.as_deref(), Some("2023-11-15 00:13:20"));
    let g = Geometry::new(&layout);
    assert_eq!(p.first_page[3].image_box, g.image_box(g.cell(3)));
}

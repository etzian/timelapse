//! Phase 5 exit criterion (DEVELOPMENT_ROADMAP.md): a valid MP4 from a sampled sequence,
//! with the committed encoder path, and no substitution of a missing encoder.
//!
//! The encode tests need a real FFmpeg and run when `TIMELAPSE_FFMPEG_DIR` points at
//! one (the macOS CI job sets it to the bundled build). Elsewhere they print SKIP. The
//! VideoToolbox tiers are only *required* when `TIMELAPSE_REQUIRE_VIDEOTOOLBOX=1`,
//! because virtualised CI Macs may have no media engine; otherwise a VideoToolbox
//! failure is reported loudly and skipped.

use std::path::PathBuf;

use super::*;
use crate::import::metadata::fixture::{ExifSpec, TempDir, write_jpeg};
use crate::import::{backfill_for_root, import_folder};
use crate::sampling::{Config, Method, create_run_for_root};

/// Exports are one at a time by design (`EXPORT_LOCK` → `busy`); the test runner is
/// parallel, so every test here runs under this lock.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

fn ffmpeg_dir() -> Option<PathBuf> {
    std::env::var_os("TIMELAPSE_FFMPEG_DIR")
        .map(PathBuf::from)
        .filter(|d| FfmpegPaths::in_dir(d).exist())
}

/// 24 photographs one minute apart, saved as one sampling run (every photograph).
fn project() -> (TempDir, i64) {
    let dir = TempDir::new("p5-export");
    for m in 0..24u32 {
        let stamp: &'static str = Box::leak(format!("2024:01:15 10:{m:02}:00").into_boxed_str());
        write_jpeg(&dir.path().join(format!("m{m:02}.jpg")), &ExifSpec::original(stamp));
    }
    import_folder(dir.path()).unwrap();
    backfill_for_root(dir.path()).unwrap();
    let cfg = Config {
        method: Method::EveryNth { n: 1, offset: 0 },
        from: None,
        to: None,
        include_flagged: true,
    };
    let run = create_run_for_root(dir.path(), &cfg).unwrap();
    (dir, run.id)
}

fn settings(codec: Codec, speed: f64) -> ExportSettings {
    ExportSettings { codec, fps: 12.0, speed, width: 320, height: 180, quality: Quality::Small }
}

#[test]
fn a_missing_ffmpeg_is_an_explicit_error_and_nothing_is_written() {
    let _serial = serial();
    let (dir, run) = project();
    let nowhere = FfmpegPaths::in_dir(&dir.path().join("no-ffmpeg"));
    let err =
        export_run(dir.path(), &nowhere, run, settings(Codec::Av1, 1.0), 1, |_| {}).unwrap_err();
    assert!(matches!(err, Error::Export { reason: "ffmpeg-missing" }));
    assert!(exports_for_root(dir.path()).unwrap().is_empty());
    assert!(!exports_dir(dir.path()).exists());
}

#[test]
fn invalid_settings_and_empty_runs_are_refused() {
    let _serial = serial();
    let (dir, run) = project();
    let p = FfmpegPaths::in_dir(&dir.path().join("x"));
    let bad = ExportSettings { width: 321, ..settings(Codec::Av1, 1.0) };
    assert!(matches!(
        export_run(dir.path(), &p, run, bad, 1, |_| {}),
        Err(Error::Export { reason: "invalid-settings" })
    ));
    // A saved selection that selected nothing.
    let none = Config {
        method: Method::EveryNth { n: 1, offset: 0 },
        from: Some("2030-01-01".into()),
        to: None,
        include_flagged: true,
    };
    let empty = create_run_for_root(dir.path(), &none).unwrap();
    assert_eq!(empty.selected_count, 0);
    assert!(matches!(
        export_run(dir.path(), &p, empty.id, settings(Codec::Av1, 1.0), 1, |_| {}),
        Err(Error::Export { reason: "empty-run" })
    ));
}

/// Encode with `codec`; `None` = the environment cannot (reported, not hidden).
fn encode_or_skip(codec: Codec, speed: f64) -> Option<(ExportRecord, usize)> {
    let Some(dir) = ffmpeg_dir() else {
        eprintln!(
            "SKIP {codec:?}: TIMELAPSE_FFMPEG_DIR not set (encode tests run on the macOS CI job)"
        );
        return None;
    };
    let paths = FfmpegPaths::in_dir(&dir);
    let info = info(&paths);
    assert!(info.available && info.lgpl, "the bundled FFmpeg must be the LGPL build: {info:?}");
    let (project, run) = project();
    let mut progress = Vec::new();
    let result =
        export_run(project.path(), &paths, run, settings(codec, speed), 7, |p| progress.push(p));
    let vt_required = std::env::var("TIMELAPSE_REQUIRE_VIDEOTOOLBOX").as_deref() == Ok("1");
    match result {
        Ok(rec) if rec.status == "completed" => {
            assert_eq!(
                progress.last().map(|p| (p.done, p.total)),
                Some((rec.output_frames, rec.output_frames))
            );
            Some((rec, 24))
        }
        Ok(rec)
            if codec != Codec::Av1
                && !vt_required
                && rec.failure.as_deref() == Some("ffmpeg-failed")
                && is_videotoolbox_unavailable(rec.stderr_tail.as_deref().unwrap_or("")) =>
        {
            eprintln!(
                "SKIP {codec:?}: this machine has no usable VideoToolbox encoder (set \
                 TIMELAPSE_REQUIRE_VIDEOTOOLBOX=1 on Apple Silicon to require it):\n{}",
                rec.stderr_tail.unwrap_or_default()
            );
            None
        }
        Err(Error::Export { reason: "encoder-unavailable" })
            if codec != Codec::Av1 && !vt_required =>
        {
            eprintln!("SKIP {codec:?}: encoder not offered by this FFmpeg");
            None
        }
        other => panic!("{codec:?} export failed: {other:?}"),
    }
}

/// FFmpeg's messages when a VideoToolbox session cannot be created (no media engine,
/// e.g. a virtualised Mac). Anything else is a real failure and is not skipped.
fn is_videotoolbox_unavailable(stderr: &str) -> bool {
    // videotoolboxenc.c: "Error: cannot create compression session: %d".
    let s = stderr.to_ascii_lowercase();
    s.contains("cannot create compression session") || s.contains("vtcompressionsessioncreate")
}

fn assert_valid(rec: &ExportRecord, codec: Codec, photos: usize, speed: f64) {
    let probe = rec.probe.as_ref().expect("probed");
    assert_eq!(probe.codec_name, codec.probe_name(), "committed encoder path");
    assert_eq!((probe.width, probe.height), (320, 180));
    assert_eq!(probe.packets as usize, output_frames(photos, speed));
    assert!(
        (probe.duration_seconds - duration_seconds(photos, 12.0, speed)).abs() < 0.2,
        "{probe:?}"
    );
    let out = std::path::Path::new(&rec.output_path);
    assert!(out.is_file() && rec.file_size.unwrap_or(0) > 0);
    assert!(!out.with_extension("mp4.part").exists(), "no partial file left");
    let head = std::fs::read(out).unwrap();
    assert_eq!(&head[4..8], b"ftyp", "an MP4 container");
}

#[test]
fn av1_tier_produces_a_valid_mp4_from_a_sampled_sequence() {
    let _serial = serial();
    if let Some((rec, n)) = encode_or_skip(Codec::Av1, 1.0) {
        assert_valid(&rec, Codec::Av1, n, 1.0);
    }
}

#[test]
fn playback_speed_changes_frames_not_fps() {
    let _serial = serial();
    if let Some((rec, n)) = encode_or_skip(Codec::Av1, 2.0) {
        assert_valid(&rec, Codec::Av1, n, 2.0);
        assert_eq!(rec.output_frames, 12);
    }
}

#[test]
fn h264_videotoolbox_tier_produces_a_valid_mp4() {
    let _serial = serial();
    if let Some((rec, n)) = encode_or_skip(Codec::H264, 1.0) {
        assert_valid(&rec, Codec::H264, n, 1.0);
    }
}

#[test]
fn hevc_videotoolbox_tier_produces_a_valid_mp4() {
    let _serial = serial();
    if let Some((rec, n)) = encode_or_skip(Codec::Hevc, 1.0) {
        assert_valid(&rec, Codec::Hevc, n, 1.0);
    }
}

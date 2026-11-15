//! The bundled FFmpeg: where it is, what it can do, the exact argv we give it, and how
//! it is run (ADR-017, ADR-021, ADR-022).
//!
//! Arguments are built by pure functions as a vector, one element per argument. Nothing
//! goes through a shell and no text is spliced into a command line. Input frames arrive
//! on stdin as raw RGB, so the only path FFmpeg sees is the output file we chose.

use std::collections::VecDeque;
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};

use super::settings::{Codec, ExportSettings};
use crate::{Error, Result};

fn fail(reason: &'static str) -> Error {
    Error::Export { reason }
}

/// The two executables of the bundled build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FfmpegPaths {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
}

impl FfmpegPaths {
    /// `<dir>/bin/ffmpeg` and `<dir>/bin/ffprobe` (`.exe` on Windows).
    pub fn in_dir(dir: &Path) -> Self {
        let exe = std::env::consts::EXE_SUFFIX;
        Self {
            ffmpeg: dir.join("bin").join(format!("ffmpeg{exe}")),
            ffprobe: dir.join("bin").join(format!("ffprobe{exe}")),
        }
    }

    pub fn exist(&self) -> bool {
        self.ffmpeg.is_file() && self.ffprobe.is_file()
    }
}

/// What the bundled FFmpeg offers; a tier is offered only if its encoder is present.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct FfmpegInfo {
    pub available: bool,
    pub version: Option<String>,
    /// `ffmpeg -L` reports the GNU Lesser General Public License.
    pub lgpl: bool,
    pub av1: bool,
    pub h264: bool,
    pub hevc: bool,
}

impl FfmpegInfo {
    pub fn supports(&self, codec: Codec) -> bool {
        match codec {
            Codec::Av1 => self.av1,
            Codec::H264 => self.h264,
            Codec::Hevc => self.hevc,
        }
    }
}

/// Run a quick, argument-only FFmpeg query and return its combined output.
fn query(exe: &Path, args: &[&str]) -> Option<String> {
    // ADR-021: the bundled ffmpeg/ffprobe only, explicit argv, no shell.
    let out = Command::new(exe) // verify:allow ADR-021
        .args(args)
        .stdin(Stdio::null())
        .output()
        .ok()?;
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    Some(text)
}

pub fn probe(paths: &FfmpegPaths) -> FfmpegInfo {
    if !paths.exist() {
        return FfmpegInfo::default();
    }
    let Some(version) = query(&paths.ffmpeg, &["-hide_banner", "-version"]) else {
        return FfmpegInfo::default();
    };
    let encoders = query(&paths.ffmpeg, &["-hide_banner", "-encoders"]).unwrap_or_default();
    let licence = query(&paths.ffmpeg, &["-hide_banner", "-L"]).unwrap_or_default();
    parse_info(&version, &encoders, &licence)
}

/// Pure parser for [`probe`]'s three outputs.
pub fn parse_info(version: &str, encoders: &str, licence: &str) -> FfmpegInfo {
    let has = |name: &str| {
        encoders.lines().any(|l| {
            let mut cols = l.split_whitespace();
            matches!((cols.next(), cols.next()), (Some(flags), Some(n)) if flags.starts_with('V') && n == name)
        })
    };
    FfmpegInfo {
        available: true,
        version: version
            .lines()
            .next()
            .and_then(|l| l.strip_prefix("ffmpeg version "))
            .and_then(|r| r.split_whitespace().next())
            .map(str::to_owned),
        lgpl: licence.contains("GNU Lesser General Public"),
        av1: has("libsvtav1"),
        h264: has("h264_videotoolbox"),
        hevc: has("hevc_videotoolbox"),
    }
}

/// The exact encode argv: raw RGB frames on stdin → MP4 at `output`.
pub fn encode_args(s: &ExportSettings, output: &Path) -> Vec<OsString> {
    let mut a: Vec<OsString> = Vec::new();
    let mut push = |v: &str| a.push(v.into());
    for v in ["-hide_banner", "-nostats", "-loglevel", "error"] {
        push(v);
    }
    for v in ["-f", "rawvideo", "-pix_fmt", "rgb24"] {
        push(v);
    }
    push("-s");
    push(&format!("{}x{}", s.width, s.height));
    push("-framerate");
    push(&format_rate(s.fps));
    for v in ["-i", "pipe:0", "-an"] {
        push(v);
    }
    let q = s.quality;
    match s.codec {
        Codec::Av1 => {
            for v in ["-c:v", "libsvtav1", "-preset", "8", "-crf"] {
                push(v);
            }
            push(&q.av1_crf().to_string());
        }
        Codec::H264 | Codec::Hevc => {
            let enc =
                if s.codec == Codec::H264 { "h264_videotoolbox" } else { "hevc_videotoolbox" };
            // `-allow_sw 0`: VideoToolbox must not fall back to a software encoder (ADR-022 b).
            for v in ["-c:v", enc, "-allow_sw", "0", "-realtime", "0", "-q:v"] {
                push(v);
            }
            push(&q.videotoolbox_q().to_string());
            if s.codec == Codec::Hevc {
                for v in ["-tag:v", "hvc1"] {
                    push(v);
                }
            } else {
                for v in ["-profile:v", "high"] {
                    push(v);
                }
            }
        }
    }
    for v in ["-pix_fmt", "yuv420p", "-movflags", "+faststart", "-f", "mp4", "-n"] {
        push(v);
    }
    a.push(output.as_os_str().to_owned());
    a
}

/// Frames per second as FFmpeg reads it (`30`, `29.97`).
pub fn format_rate(fps: f64) -> String {
    let s = format!("{fps:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// The validation argv for ffprobe.
pub fn probe_args(output: &Path) -> Vec<OsString> {
    let mut a: Vec<OsString> = [
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-count_packets",
        "-show_entries",
        "stream=codec_name,width,height,nb_read_packets:format=duration",
        "-of",
        "json",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    a.push(output.as_os_str().to_owned());
    a
}

/// What ffprobe found in a finished file.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Probed {
    pub codec_name: String,
    pub width: u32,
    pub height: u32,
    pub packets: u64,
    pub duration_seconds: f64,
}

pub fn parse_probe(json: &str) -> Option<Probed> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let s = v.get("streams")?.get(0)?;
    let num = |x: &serde_json::Value| {
        x.as_str().and_then(|t| t.parse::<f64>().ok()).or_else(|| x.as_f64())
    };
    Some(Probed {
        codec_name: s.get("codec_name")?.as_str()?.to_owned(),
        width: s.get("width")?.as_u64()? as u32,
        height: s.get("height")?.as_u64()? as u32,
        packets: num(s.get("nb_read_packets")?)? as u64,
        duration_seconds: num(v.get("format")?.get("duration")?)?,
    })
}

pub fn validate(paths: &FfmpegPaths, output: &Path) -> Option<Probed> {
    // ADR-021: the bundled ffprobe, explicit argv, no shell.
    let out = Command::new(&paths.ffprobe) // verify:allow ADR-021
        .args(probe_args(output))
        .stdin(Stdio::null())
        .output()
        .ok()?;
    out.status.success().then(|| parse_probe(&String::from_utf8_lossy(&out.stdout))).flatten()
}

/// A running encode: frames go in on stdin; stderr is kept (last lines) for diagnosis.
pub struct Encoder {
    child: Child,
    stdin: Option<ChildStdin>,
    stderr_tail: Arc<Mutex<VecDeque<String>>>,
    reader: Option<std::thread::JoinHandle<()>>,
}

const STDERR_LINES: usize = 40;

impl Encoder {
    pub fn start(paths: &FfmpegPaths, args: &[OsString]) -> Result<Self> {
        // ADR-021: the bundled ffmpeg, explicit argv, no shell.
        let mut child = Command::new(&paths.ffmpeg) // verify:allow ADR-021
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| fail("ffmpeg-failed"))?;
        let stdin = child.stdin.take();
        let tail = Arc::new(Mutex::new(VecDeque::new()));
        let reader = child.stderr.take().map(|err| {
            let tail = Arc::clone(&tail);
            std::thread::spawn(move || {
                for line in BufReader::new(err).lines().map_while(std::result::Result::ok) {
                    let mut t = tail.lock().unwrap_or_else(|e| e.into_inner());
                    if t.len() == STDERR_LINES {
                        t.pop_front();
                    }
                    t.push_back(line);
                }
            })
        });
        Ok(Self { child, stdin, stderr_tail: tail, reader })
    }

    /// Write one frame. An error means FFmpeg stopped reading (it failed).
    pub fn write_frame(&mut self, rgb: &[u8]) -> Result<()> {
        let stdin = self.stdin.as_mut().ok_or_else(|| fail("ffmpeg-failed"))?;
        stdin.write_all(rgb).map_err(|_| fail("ffmpeg-failed"))
    }

    /// Close stdin and wait. Returns the stderr tail either way.
    pub fn finish(mut self) -> (bool, String) {
        drop(self.stdin.take());
        let ok = self.child.wait().is_ok_and(|s| s.success());
        (ok, self.tail())
    }

    pub fn kill(mut self) -> String {
        drop(self.stdin.take());
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.tail()
    }

    fn tail(&mut self) -> String {
        if let Some(r) = self.reader.take() {
            let _ = r.join();
        }
        let t = self.stderr_tail.lock().unwrap_or_else(|e| e.into_inner());
        t.iter().cloned().collect::<Vec<_>>().join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::settings::Quality;

    fn settings(codec: Codec) -> ExportSettings {
        ExportSettings {
            codec,
            fps: 30.0,
            speed: 1.0,
            width: 1920,
            height: 1080,
            quality: Quality::Standard,
        }
    }

    fn strs(a: &[OsString]) -> Vec<String> {
        a.iter().map(|s| s.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn av1_argv_is_explicit_and_never_overwrites() {
        let out = Path::new("/p/.timelapse/exports/run-1.mp4.part");
        let a = strs(&encode_args(&settings(Codec::Av1), out));
        assert_eq!(
            a,
            [
                "-hide_banner",
                "-nostats",
                "-loglevel",
                "error",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
                "-s",
                "1920x1080",
                "-framerate",
                "30",
                "-i",
                "pipe:0",
                "-an",
                "-c:v",
                "libsvtav1",
                "-preset",
                "8",
                "-crf",
                "32",
                "-pix_fmt",
                "yuv420p",
                "-movflags",
                "+faststart",
                "-f",
                "mp4",
                "-n",
                "/p/.timelapse/exports/run-1.mp4.part"
            ]
        );
    }

    #[test]
    fn videotoolbox_argv_forbids_software_fallback() {
        let a = strs(&encode_args(&settings(Codec::H264), Path::new("o")));
        let at = a.iter().position(|v| v == "-allow_sw").unwrap();
        assert_eq!(a[at + 1], "0");
        assert!(a.contains(&"h264_videotoolbox".into()));
        assert!(!a.iter().any(|v| v.contains("x264")));
        let h = strs(&encode_args(&settings(Codec::Hevc), Path::new("o")));
        assert!(h.windows(2).any(|w| w == ["-tag:v", "hvc1"]));
        assert!(h.contains(&"hevc_videotoolbox".into()));
    }

    #[test]
    fn a_path_with_spaces_or_quotes_stays_one_argument() {
        let out = Path::new("/Users/a b/it's \"here\"; rm -rf x/out.mp4.part");
        let a = encode_args(&settings(Codec::Av1), out);
        assert_eq!(a.last().unwrap(), out.as_os_str());
        assert_eq!(a.iter().filter(|v| v.to_string_lossy().contains("rm -rf")).count(), 1);
    }

    #[test]
    fn rates_are_formatted_for_ffmpeg() {
        assert_eq!(format_rate(30.0), "30");
        assert_eq!(format_rate(29.97), "29.97");
        assert_eq!(format_rate(12.5), "12.5");
    }

    #[test]
    fn probe_output_is_parsed() {
        let version = "ffmpeg version 9.0.2 Copyright (c) 2000-2026 the FFmpeg developers\nbuilt with Apple clang";
        let encoders = " V....D libsvtav1            SVT-AV1(Scalable Video Technology for AV1) encoder (codec av1)\n V....D h264_videotoolbox    VideoToolbox H.264 Encoder (codec h264)\n A....D aac   AAC\n";
        let licence = "ffmpeg is free software; you can redistribute it and/or modify\nit under the terms of the GNU Lesser General Public License as published by";
        let info = parse_info(version, encoders, licence);
        assert_eq!(info.version.as_deref(), Some("9.0.2"));
        assert!(info.lgpl && info.av1 && info.h264 && !info.hevc);
        assert!(info.supports(Codec::Av1) && !info.supports(Codec::Hevc));
        let gpl = parse_info(version, "", "under the terms of the GNU General Public License");
        assert!(!gpl.lgpl);
    }

    #[test]
    fn ffprobe_json_is_parsed() {
        let json = r#"{"programs":[],"streams":[{"codec_name":"av1","width":640,"height":360,"nb_read_packets":"24"}],"format":{"duration":"2.000000"}}"#;
        assert_eq!(
            parse_probe(json),
            Some(Probed {
                codec_name: "av1".into(),
                width: 640,
                height: 360,
                packets: 24,
                duration_seconds: 2.0
            })
        );
        assert_eq!(parse_probe("{}"), None);
        assert_eq!(parse_probe("not json"), None);
    }

    #[test]
    fn missing_binaries_are_reported_unavailable() {
        let p = FfmpegPaths::in_dir(Path::new("/definitely/not/here"));
        assert!(!p.exist());
        assert_eq!(probe(&p), FfmpegInfo::default());
    }
}

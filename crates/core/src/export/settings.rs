//! Export settings and the duration maths (PRD §5.7).
//!
//! FPS and playback speed stay separate concepts. At speed 1 each selected photograph is
//! one frame. At speed `s`, output frame `k` shows photograph `floor(k × s)`, so 2× uses
//! every other photograph and 0.5× shows each one for two frames. That is a deliberate,
//! visible duplication, chosen only by the user's speed setting. Output frames are
//! `ceil(n / s)` and the duration is that over the FPS: the same figure the Sampling page
//! estimates.

/// The output codec. AV1 is the default (ADR-017).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Codec {
    #[default]
    Av1,
    H264,
    Hevc,
}

impl Codec {
    /// `codec_name` as ffprobe reports it.
    pub fn probe_name(self) -> &'static str {
        match self {
            Self::Av1 => "av1",
            Self::H264 => "h264",
            Self::Hevc => "hevc",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Quality {
    High,
    #[default]
    Standard,
    Small,
}

impl Quality {
    pub fn av1_crf(self) -> u8 {
        match self {
            Self::High => 26,
            Self::Standard => 32,
            Self::Small => 40,
        }
    }

    /// VideoToolbox constant quality (Apple Silicon), 1–100.
    pub fn videotoolbox_q(self) -> u8 {
        match self {
            Self::High => 70,
            Self::Standard => 55,
            Self::Small => 40,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ExportSettings {
    #[serde(default)]
    pub codec: Codec,
    pub fps: f64,
    /// Playback multiplier: 2.0 = twice as fast (half the duration).
    pub speed: f64,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub quality: Quality,
    /// Deflicker run whose gains are applied (ADR-025); its selection must be the export's.
    #[serde(default)]
    pub flicker_run: Option<i64>,
    /// Stabilization run whose shifts and crop are applied.
    #[serde(default)]
    pub stabilization_run: Option<i64>,
}

impl ExportSettings {
    /// `None` when valid, else the reason (`invalid-settings` is the error kind's selector).
    pub fn check(&self) -> Result<(), &'static str> {
        let fps_ok = self.fps.is_finite() && (1.0..=120.0).contains(&self.fps);
        let speed_ok = self.speed.is_finite() && (0.1..=100.0).contains(&self.speed);
        // Even sizes (4:2:0 chroma), within what both encoders accept.
        let size_ok = |v: u32| (64..=7680).contains(&v) && v.is_multiple_of(2);
        if fps_ok && speed_ok && size_ok(self.width) && size_ok(self.height) {
            Ok(())
        } else {
            Err("invalid-settings")
        }
    }
}

/// Which selected photograph each output frame shows.
pub fn frame_plan(selected: usize, speed: f64) -> Vec<usize> {
    if selected == 0 || !speed.is_finite() || speed <= 0.0 {
        return Vec::new();
    }
    let frames = output_frames(selected, speed);
    (0..frames).map(|k| (((k as f64) * speed).floor() as usize).min(selected - 1)).collect()
}

pub fn output_frames(selected: usize, speed: f64) -> usize {
    if selected == 0 || speed <= 0.0 {
        return 0;
    }
    // A tiny epsilon keeps exact multiples (24 / 2.0) from rounding up.
    ((selected as f64 / speed) - 1e-9).ceil().max(1.0) as usize
}

pub fn duration_seconds(selected: usize, fps: f64, speed: f64) -> f64 {
    if fps <= 0.0 { 0.0 } else { output_frames(selected, speed) as f64 / fps }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speed_and_fps_stay_separate() {
        assert_eq!(output_frames(300, 1.0), 300);
        assert_eq!(duration_seconds(300, 30.0, 1.0), 10.0);
        assert_eq!(duration_seconds(300, 30.0, 2.0), 5.0);
        assert_eq!(duration_seconds(300, 30.0, 0.5), 20.0);
        assert_eq!(duration_seconds(300, 60.0, 1.0), 5.0);
    }

    #[test]
    fn frame_plan_skips_or_holds_frames_by_speed() {
        assert_eq!(frame_plan(5, 1.0), [0, 1, 2, 3, 4]);
        assert_eq!(frame_plan(5, 2.0), [0, 2, 4]);
        assert_eq!(frame_plan(3, 0.5), [0, 0, 1, 1, 2, 2]);
        assert_eq!(frame_plan(10, 4.0), [0, 4, 8]);
        assert!(frame_plan(0, 1.0).is_empty());
        assert!(frame_plan(5, 0.0).is_empty());
        // Every plan index is a valid photograph, in order.
        for (n, s) in [(7, 1.5), (100, 3.3), (1, 9.0), (9, 0.3)] {
            let p = frame_plan(n, s);
            assert_eq!(p.len(), output_frames(n, s));
            assert!(p.windows(2).all(|w| w[0] <= w[1]) && p.iter().all(|&i| i < n));
        }
    }

    #[test]
    fn settings_are_checked() {
        let ok = ExportSettings {
            codec: Codec::Av1,
            fps: 30.0,
            speed: 1.0,
            width: 1920,
            height: 1080,
            quality: Quality::High,
            flicker_run: None,
            stabilization_run: None,
        };
        assert!(ok.check().is_ok());
        for bad in [
            ExportSettings { fps: 0.0, ..ok },
            ExportSettings { fps: f64::NAN, ..ok },
            ExportSettings { speed: 0.0, ..ok },
            ExportSettings { width: 1921, ..ok },
            ExportSettings { height: 10, ..ok },
        ] {
            assert_eq!(bad.check(), Err("invalid-settings"));
        }
        let parsed: ExportSettings =
            serde_json::from_str(r#"{"fps":24,"speed":2,"width":1280,"height":720}"#).unwrap();
        assert_eq!(
            (parsed.codec, parsed.quality),
            (Codec::Av1, Quality::Standard),
            "AV1 is the default"
        );
    }
}

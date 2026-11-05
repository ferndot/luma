use std::time::{Duration, Instant};

use crate::{lan, razer};
use luma_core::domain::{LightPort, Rgb};

/// Razer mode is dropped after about a minute without frames, so re-arm it well before that.
const REARM_EVERY: Duration = Duration::from_secs(30);

/// A light driven by razer frames over LAN, one color per segment.
pub struct RazerLight {
    ip: String,
    segments: usize,
    armed_at: Option<Instant>,
}

impl RazerLight {
    pub fn new(ip: &str, segments: usize) -> Self {
        Self { ip: ip.to_string(), segments, armed_at: None }
    }

    fn arm(&mut self) -> Result<(), String> {
        lan::send(&self.ip, &razer::enable_message(true)).map_err(|e| e.to_string())?;
        self.armed_at = Some(Instant::now());
        Ok(())
    }
}

impl LightPort for RazerLight {
    fn segments(&self) -> usize {
        self.segments
    }

    fn show(&mut self, colors: &[Rgb]) -> Result<(), String> {
        if self.armed_at.is_none_or(|t| t.elapsed() > REARM_EVERY) {
            self.arm()?;
        }
        let frame = razer::frame_message(colors)?;
        lan::send(&self.ip, &frame).map_err(|e| e.to_string())
    }

    fn close(&mut self) -> Result<(), String> {
        self.armed_at = None;
        lan::send(&self.ip, &razer::enable_message(false)).map_err(|e| e.to_string())
    }
}

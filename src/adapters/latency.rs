use std::process::Command;
use std::time::Duration;

use crate::domain::{LightPort, Rgb};

/// One-way network latency to `ip`, estimated as half the average ping round trip.
/// Ping measures the network only; device processing time is not included.
pub fn measure(ip: &str) -> Result<Duration, String> {
    let out = Command::new("ping").args(["-n", "4", "-w", "1000", ip]).output().map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&out.stdout);
    let avg = text
        .lines()
        .find_map(|l| l.split("Average = ").nth(1))
        .and_then(|s| s.trim_end_matches(|c: char| !c.is_ascii_digit()).trim_end_matches("ms").trim().parse::<u64>().ok())
        .ok_or_else(|| format!("could not read ping average for {ip}"))?;
    Ok(Duration::from_millis(avg) / 2)
}

/// Wraps a light with a measured latency so the show can send early enough to land on time.
pub struct Delayed {
    inner: Box<dyn LightPort>,
    latency: Duration,
}

impl Delayed {
    pub fn new(inner: Box<dyn LightPort>, latency: Duration) -> Self {
        Self { inner, latency }
    }
}

impl LightPort for Delayed {
    fn segments(&self) -> usize {
        self.inner.segments()
    }
    fn show(&mut self, colors: &[Rgb]) -> Result<(), String> {
        self.inner.show(colors)
    }
    fn close(&mut self) -> Result<(), String> {
        self.inner.close()
    }
    fn latency(&self) -> Duration {
        self.latency
    }
}

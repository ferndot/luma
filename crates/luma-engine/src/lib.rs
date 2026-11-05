//! Pure logic: modes, light specs and frame selection.
use luma_core::domain::Rgb;
use luma_core::modes::{mic_frame, screen_frame};
use luma_core::patterns::NAMES;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Mic,
    Screen,
    Pattern(&'static str),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LightSpec {
    pub sku: String,
    pub ip: String,
    pub razer: bool,
    pub segments: usize,
}

impl Mode {
    pub fn from_name(name: &str) -> Option<Mode> {
        let n = name.to_lowercase();
        match n.as_str() {
            "microphone" | "mic" => Some(Mode::Mic),
            "screen" => Some(Mode::Screen),
            _ => NAMES.iter().find(|(k, _)| *k == n).map(|(k, _)| Mode::Pattern(k)),
        }
    }
    pub fn label(&self) -> String {
        match self {
            Mode::Mic => "microphone".into(),
            Mode::Screen => "screen".into(),
            Mode::Pattern(p) => p.to_string(),
        }
    }
}

/// Light specs from discovery JSON values (entries missing "ip" or "sku" are skipped).
pub fn specs_from_discovery(found: &BTreeMap<String, Value>) -> Vec<LightSpec> {
    found
        .values()
        .filter_map(|v| {
            let (ip, sku) = (v.get("ip")?.as_str()?, v.get("sku")?.as_str()?);
            let cap = luma_govee::capability(sku);
            Some(LightSpec { sku: sku.into(), ip: ip.into(), razer: cap.razer, segments: cap.segments })
        })
        .collect()
}

/// One frame for `mode`. `level` is the last mic level; `zones` captures the screen on demand.
pub fn frame_for(
    mode: Mode,
    total: usize,
    t: f64,
    level: f32,
    zones: impl FnOnce(usize) -> Result<Vec<Rgb>, String>,
) -> Result<Vec<Rgb>, String> {
    Ok(match mode {
        Mode::Mic => mic_frame(total, level, t),
        Mode::Screen => screen_frame(total, &zones(total)?),
        Mode::Pattern(name) => {
            let f = NAMES.iter().find(|(k, _)| *k == name).ok_or("unknown pattern")?.1;
            f(total, t)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn mode_names_parse() {
        assert_eq!(Mode::from_name("microphone"), Some(Mode::Mic));
        assert_eq!(Mode::from_name("Screen"), Some(Mode::Screen));
        assert_eq!(Mode::from_name("Rainbow"), Some(Mode::Pattern("rainbow")));
        assert_eq!(Mode::from_name("nope"), None);
        assert_eq!(Mode::Pattern("chase").label(), "chase");
    }

    #[test]
    fn pattern_frame_length_is_total() {
        for (name, _) in NAMES {
            let f = frame_for(Mode::Pattern(name), 19, 1.0, 0.0, |_| Err("no".into())).unwrap();
            assert_eq!(f.len(), 19);
        }
    }

    #[test]
    fn screen_frame_resamples_and_errors_propagate() {
        let f = frame_for(Mode::Screen, 6, 0.0, 0.0, |_| Ok(vec![(1, 2, 3)])).unwrap();
        assert_eq!(f, vec![(1, 2, 3); 6]);
        assert!(frame_for(Mode::Screen, 6, 0.0, 0.0, |_| Err("x".into())).is_err());
    }

    #[test]
    fn mic_frame_follows_level() {
        let f = frame_for(Mode::Mic, 4, 0.0, 0.0, |_| Err("no".into())).unwrap();
        assert!(f.iter().all(|c| *c == (0, 0, 0)));
    }

    #[test]
    fn discovery_splits_razer_and_whole() {
        let mut m = BTreeMap::new();
        m.insert("a".into(), json!({"ip": "1.1.1.1", "sku": "H6022"}));
        m.insert("b".into(), json!({"ip": "1.1.1.2", "sku": "H6000"}));
        m.insert("c".into(), json!({"sku": "H6000"}));
        let s = specs_from_discovery(&m);
        assert_eq!(s.len(), 2);
        assert!(s[0].razer && s[0].segments == 15);
        assert!(!s[1].razer && s[1].segments == 1);
    }
}

use luma_core::adapters::latency::{Delayed, measure};
use luma_core::app::run_show;
use luma_core::domain::{LightPort, latest};
use luma_govee::{razer_light::RazerLight, whole_light::WholeLight};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

type Shared<T> = Arc<Mutex<T>>;

/// Runs the show on a background thread and exposes its state to any front end (GUI or web).
#[derive(Clone)]
pub struct Show {
    pub mode: Shared<Mode>,
    pub preview: Shared<Vec<Rgb>>,
    pub status: Shared<String>,
    stop: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
}

impl Default for Show {
    fn default() -> Self {
        Self {
            mode: Arc::new(Mutex::new(Mode::Pattern("rainbow"))),
            preview: Default::default(),
            status: Arc::new(Mutex::new("Idle".into())),
            stop: Default::default(),
            running: Default::default(),
        }
    }
}

impl Show {
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Starts the show on `specs`. Fails if no lights are given.
    pub fn start(&self, specs: Vec<LightSpec>) -> Result<(), String> {
        if specs.is_empty() {
            return Err("Select at least one light".into());
        }
        self.stop.store(false, Ordering::SeqCst);
        self.running.store(true, Ordering::SeqCst);
        let this = self.clone();
        std::thread::spawn(move || {
            let r = engine_thread(specs, &this);
            *this.status.lock().unwrap() = r.err().unwrap_or_else(|| "Idle".into());
            this.running.store(false, Ordering::SeqCst);
        });
        Ok(())
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

/// The /24 prefixes to scan for lights: this machine's networks first, then `extra`
/// (for example the subnets of lights remembered from an earlier run). Empty if no network is found.
pub fn local_prefixes(extra: &[String]) -> Vec<String> {
    luma_govee::lan::local_prefixes(extra)
}

/// Discovers lights on `prefixes` in the background; the result arrives on the returned channel.
pub fn find_lights(prefixes: Vec<String>) -> mpsc::Receiver<Result<Vec<LightSpec>, String>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let r = luma_govee::lan::discover(Duration::from_secs(10), &prefixes).map(|m| specs_from_discovery(&m));
        let _ = tx.send(r);
    });
    rx
}

fn engine_thread(specs: Vec<LightSpec>, show: &Show) -> Result<(), String> {
    let mut lights: Vec<Box<dyn LightPort>> = Vec::new();
    for s in &specs {
        let light: Box<dyn LightPort> = if s.razer {
            Box::new(RazerLight::new(&s.ip, s.segments))
        } else {
            Box::new(WholeLight::new(&s.ip))
        };
        let lat = measure(&s.ip)?;
        lights.push(Box::new(Delayed::new(light, lat)));
    }
    // The mic is created here (it may not be Send), lazily when the mode first needs it.
    let (tx, rx) = mpsc::channel();
    let mut mic: Option<luma_audio::Mic> = None;
    let mut level = 0.0f32;
    let mut failure: Option<String> = None;
    let stop = show.stop.clone();
    let result = run_show(
        &mut lights,
        |total, t| {
            let m = *show.mode.lock().unwrap();
            if m == Mode::Mic && mic.is_none() {
                match luma_audio::Mic::open(tx.clone()) {
                    Ok(x) => mic = Some(x),
                    Err(e) => {
                        failure = Some(e);
                        stop.store(true, Ordering::SeqCst);
                    }
                }
            }
            if let Some(s) = latest(&rx) {
                level = s.level;
            }
            if failure.is_none() {
                *show.status.lock().unwrap() = format!("Running: {}", m.label());
            }
            let f = frame_for(m, total, t, level, luma_screen::capture_zones).unwrap_or_else(|e| {
                failure = Some(e);
                stop.store(true, Ordering::SeqCst);
                vec![(0, 0, 0); total]
            });
            *show.preview.lock().unwrap() = f.clone();
            f
        },
        || !stop.load(Ordering::SeqCst),
    );
    result?;
    failure.map_or(Ok(()), Err)
}

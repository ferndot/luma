//! Pure logic: light specs, effect sequences and frame selection.
use luma_core::domain::Rgb;
use luma_core::effects::{Effect, builtin_effects};
use luma_core::modes::{mic_frame, screen_frame};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct LightSpec {
    /// The device id from the LAN scan; stable across IP changes.
    pub id: String,
    pub sku: String,
    pub ip: String,
    pub razer: bool,
    pub segments: usize,
}


/// Light specs from discovery JSON values (entries missing "ip" or "sku" are skipped).
pub fn specs_from_discovery(found: &BTreeMap<String, Value>) -> Vec<LightSpec> {
    found
        .iter()
        .filter_map(|(id, v)| {
            let (ip, sku) = (v.get("ip")?.as_str()?, v.get("sku")?.as_str()?);
            let cap = luma_govee::capability(sku);
            Some(LightSpec {
                id: id.clone(),
                sku: sku.into(),
                ip: ip.into(),
                razer: cap.razer,
                segments: cap.segments,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequences_step_through_and_repeat_or_hold() {
        let step = |color, secs| Step { effect: Effect::Solid { color }, secs };
        let seq = |repeat| Sequence { steps: vec![step((1, 0, 0), 2.0), step((0, 1, 0), 3.0)], repeat };
        let color = |s: &Sequence, t| match s.at(t).0.effect {
            Effect::Solid { color } => color,
            _ => unreachable!(),
        };
        assert_eq!(color(&seq(false), 1.0), (1, 0, 0));
        assert_eq!(color(&seq(false), 2.5), (0, 1, 0));
        assert_eq!(color(&seq(false), 99.0), (0, 1, 0));
        // Repeating: 5 seconds per cycle, so 6.0 is 1 second into the next first step.
        assert_eq!(color(&seq(true), 6.0), (1, 0, 0));
        assert!((seq(true).at(6.0).1 - 1.0).abs() < 1e-9);
    }

    #[test]
    fn segments_spread_along_a_strip_and_sit_on_a_whole_light() {
        let strip = LightSpec { id: "a".into(), sku: "H6022".into(), ip: "1.1.1.1".into(), razer: true, segments: 3 };
        let spots = segment_spots(&strip, (0.5, 0.2));
        assert_eq!(spots.len(), 3);
        assert!((spots[0].0 - 0.46).abs() < 1e-6 && (spots[2].0 - 0.54).abs() < 1e-6);
        assert!(spots.iter().all(|s| s.1 == 0.2));
        let whole = LightSpec { razer: false, segments: 1, ..strip };
        assert_eq!(segment_spots(&whole, (0.5, 0.2)), vec![(0.5, 0.2)]);
    }

    #[test]
    fn discovery_splits_razer_and_whole() {
        let mut m = BTreeMap::new();
        m.insert("a".into(), json!({"ip": "1.1.1.1", "sku": "H6022"}));
        m.insert("b".into(), json!({"ip": "1.1.1.2", "sku": "H6000"}));
        m.insert("c".into(), json!({"sku": "H6000"}));
        let s = specs_from_discovery(&m);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].id, "a");
        assert!(s[0].razer && s[0].segments == 15);
        assert!(!s[1].razer && s[1].segments == 1);
    }
}

use luma_core::adapters::latency::{Delayed, measure};
use luma_core::app::run_show;
use luma_core::domain::{LightPort, latest};
use luma_govee::{lan, razer_light::RazerLight, whole_light::WholeLight};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

type Shared<T> = Arc<Mutex<T>>;

/// One effect in a built scene: an effect name (as `Mode::from_name` reads it), a speed, and how long it runs.
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub effect: Effect,
    pub secs: f32,
}

/// Steps played in order from the start of the show. After the last step, the sequence repeats
/// if `repeat` is set, and otherwise holds its last step.
#[derive(Clone, Debug, PartialEq)]
pub struct Sequence {
    pub steps: Vec<Step>,
    pub repeat: bool,
}

impl Sequence {
    /// The step playing at `t` seconds, and how far into that step we are.
    pub fn at(&self, t: f64) -> (&Step, f64) {
        let total: f64 = self.steps.iter().map(|s| s.secs as f64).sum();
        let mut left = if self.repeat && total > 0.0 { t % total } else { t };
        for step in &self.steps {
            if left < step.secs as f64 {
                return (step, left);
            }
            left -= step.secs as f64;
        }
        let last = self.steps.last().expect("a sequence has at least one step");
        (last, last.secs as f64)
    }
}

/// Runs the show on a background thread and exposes its state to any front end (GUI or web).
#[derive(Clone)]
pub struct Show {
    /// What the show plays: a sequence of effects. A one-step sequence that repeats is a single effect.
    pub sequence: Shared<Option<Sequence>>,
    pub preview: Shared<Vec<Rgb>>,
    /// Room position of each preview segment, in the same order.
    pub positions: Shared<Vec<(f32, f32)>>,
    pub status: Shared<String>,
    stop: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
}

/// The show starts on the built-in rainbow until something else is played.
fn rainbow_step() -> Step {
    let effect = builtin_effects().into_iter().find(|(id, _, _)| *id == "rainbow").expect("rainbow is built in").2;
    Step { effect, secs: 1.0 }
}

impl Default for Show {
    fn default() -> Self {
        Self {
            sequence: Arc::new(Mutex::new(Some(Sequence { steps: vec![rainbow_step()], repeat: true }))),
            preview: Default::default(),
            positions: Default::default(),
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

    /// Starts the show on the placed lights, in the order their segments run. Fails if there are none.
    pub fn start(&self, placed: Vec<Placed>) -> Result<(), String> {
        if placed.is_empty() {
            return Err("Select at least one light".into());
        }
        self.stop.store(false, Ordering::SeqCst);
        self.running.store(true, Ordering::SeqCst);
        let this = self.clone();
        std::thread::spawn(move || {
            let r = engine_thread(placed, &this);
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

/// Reads each light's power and color. Lights that don't answer are missing from the map.
pub fn statuses(ips: &[String], wait: Duration) -> Result<BTreeMap<String, (bool, Rgb)>, String> {
    luma_govee::lan::statuses(ips, wait)
}

/// Turns a light off now. Strip lights go black and leave razer mode first.
pub fn off(spec: &LightSpec) {
    if spec.razer {
        let mut light = RazerLight::new(&spec.ip, spec.segments);
        let _ = light.show(&vec![(0, 0, 0); spec.segments]);
        let _ = light.close();
        // The light ignores power commands until it has left razer mode.
        std::thread::sleep(RAZER_SETTLE);
    }
    power_off(&spec.ip);
}

/// Puts a light back to the power and color it had before. `None` means it was unknown, so it is left off.
pub fn restore(spec: &LightSpec, prior: Option<(bool, Rgb)>) {
    if spec.razer {
        let _ = RazerLight::new(&spec.ip, spec.segments).close();
        std::thread::sleep(RAZER_SETTLE);
    }
    match prior {
        Some((true, (r, g, b))) => {
            let color = json!({ "msg": { "cmd": "colorwc", "data": { "color": { "r": r, "g": g, "b": b }, "colorTemInKelvin": 0 } } });
            let _ = lan::send(&spec.ip, &color);
            let on = json!({ "msg": { "cmd": "turn", "data": { "value": 1 } } });
            let _ = lan::send(&spec.ip, &on);
        }
        _ => power_off(&spec.ip),
    }
}

/// Time for a light to leave or enter razer mode before it takes another command.
const RAZER_SETTLE: Duration = Duration::from_millis(150);

/// Sends the power-off command. Closing razer mode alone can leave the light on its last state.
fn power_off(ip: &str) {
    let off = json!({ "msg": { "cmd": "turn", "data": { "value": 0 } } });
    let _ = lan::send(ip, &off);
}

fn power_on(ip: &str) {
    let on = json!({ "msg": { "cmd": "turn", "data": { "value": 1 } } });
    let _ = lan::send(ip, &on);
}

/// A light held on one color until `end` is called.
pub struct Lit {
    stop: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Lit {
    /// Stops holding the light. Returns once the background thread has finished sending.
    pub fn end(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Lights one device in a solid color, resending it while held, until `Lit::end`.
pub fn light_up(spec: LightSpec, color: Rgb) -> Lit {
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    let handle = std::thread::spawn(move || {
        let mut light: Box<dyn LightPort> = if spec.razer {
            Box::new(RazerLight::new(&spec.ip, spec.segments))
        } else {
            Box::new(WholeLight::new(&spec.ip))
        };
        let frame = vec![color; light.segments()];
        if spec.razer {
            // Razer frames don't power a light on by themselves, so turn it on first.
            power_on(&spec.ip);
            std::thread::sleep(RAZER_SETTLE);
        }
        while !flag.load(Ordering::SeqCst) {
            if light.show(&frame).is_err() {
                break;
            }
            std::thread::sleep(Duration::from_millis(300));
        }
    });
    Lit { stop, handle: Some(handle) }
}

/// A light and the spot in the room it sits at.
#[derive(Clone, Debug)]
pub struct Placed {
    pub spec: LightSpec,
    pub at: (f32, f32),
}

/// Where each segment of a light sits. A strip spreads its segments along a short horizontal line through its spot.
pub fn segment_spots(spec: &LightSpec, at: (f32, f32)) -> Vec<(f32, f32)> {
    const STRIP_WIDTH: f32 = 0.08;
    let n = spec.segments;
    if n <= 1 {
        return vec![at];
    }
    (0..n)
        .map(|k| (at.0 - STRIP_WIDTH / 2.0 + STRIP_WIDTH * k as f32 / (n - 1) as f32, at.1))
        .collect()
}

fn engine_thread(placed: Vec<Placed>, show: &Show) -> Result<(), String> {
    let specs: Vec<LightSpec> = placed.iter().map(|p| p.spec.clone()).collect();
    let spots: Vec<(f32, f32)> = placed.iter().flat_map(|p| segment_spots(&p.spec, p.at)).collect();
    *show.positions.lock().unwrap() = spots.clone();
    // Power every light on at once so one that was off shows the show; turning on a lit light is a no-op.
    let on = json!({ "msg": { "cmd": "turn", "data": { "value": 1 } } });
    let sent: Vec<std::io::Result<()>> = std::thread::scope(|s| {
        let handles: Vec<_> = specs.iter().map(|sp| s.spawn(|| lan::send(&sp.ip, &on))).collect();
        handles.into_iter().map(|h| h.join().unwrap_or_else(|_| Err(std::io::Error::other("power-on thread panicked")))).collect()
    });
    for r in sent {
        r.map_err(|e| e.to_string())?;
    }
    // Ping every light at once; each ping takes up to four seconds.
    let lats: Vec<Duration> = std::thread::scope(|s| {
        let handles: Vec<_> = specs.iter().map(|sp| s.spawn(|| measure(&sp.ip))).collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap_or_else(|_| Err("latency thread panicked".to_string())))
            .collect::<Result<_, String>>()
    })?;
    let mut lights: Vec<Box<dyn LightPort>> = Vec::new();
    for (s, lat) in specs.iter().zip(lats) {
        let light: Box<dyn LightPort> = if s.razer {
            Box::new(RazerLight::new(&s.ip, s.segments))
        } else {
            Box::new(WholeLight::new(&s.ip))
        };
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
        |_total, t| {
            // The step playing now picks the effect; `t` becomes the time into that step.
            let (effect, t) = match show.sequence.lock().unwrap().as_ref() {
                Some(seq) => {
                    let (step, into) = seq.at(t);
                    (step.effect.clone(), into)
                }
                None => (Effect::Solid { color: (255, 255, 255) }, t),
            };
            if matches!(effect, Effect::Microphone) && mic.is_none() {
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
                *show.status.lock().unwrap() = "Running".into();
            }
            let total = spots.len();
            let fail = |e: String, failure: &mut Option<String>| {
                *failure = Some(e);
                stop.store(true, Ordering::SeqCst);
                vec![(0, 0, 0); total]
            };
            let f = match &effect {
                Effect::Microphone => mic_frame(total, level, t),
                Effect::Screen => match luma_screen::capture_zones(total) {
                    Ok(zones) => screen_frame(&spots, &zones),
                    Err(e) => fail(e, &mut failure),
                },
                other => spots.iter().map(|&(x, y)| other.color_at(x, y, t).unwrap_or((0, 0, 0))).collect(),
            };
            *show.preview.lock().unwrap() = f.clone();
            f
        },
        || !stop.load(Ordering::SeqCst),
    );
    result?;
    failure.map_or(Ok(()), Err)
}

mod engine;

use eframe::egui::{self, Color32, RichText};
use engine::{LightSpec, Mode, frame_for, specs_from_discovery};
use luma_core::adapters::latency::{Delayed, measure};
use luma_core::app::run_show;
use luma_core::domain::{LightPort, Rgb, latest};
use luma_govee::{razer_light::RazerLight, whole_light::WholeLight};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

type Shared<T> = Arc<Mutex<T>>;

struct App {
    subnet: String,
    lights: Vec<(LightSpec, bool)>,
    discovery: Option<mpsc::Receiver<Result<Vec<LightSpec>, String>>>,
    mode: Shared<Mode>,
    preview: Shared<Vec<Rgb>>,
    status: Shared<String>,
    stop: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
}

impl App {
    fn new() -> Self {
        Self {
            subnet: "192.168.10".into(),
            lights: vec![],
            discovery: None,
            mode: Arc::new(Mutex::new(Mode::Pattern("rainbow"))),
            preview: Default::default(),
            status: Arc::new(Mutex::new("Idle".into())),
            stop: Default::default(),
            running: Default::default(),
        }
    }

    fn find(&mut self) {
        let (tx, rx) = mpsc::channel();
        let prefix = self.subnet.clone();
        std::thread::spawn(move || {
            let r = luma_govee::lan::discover(Duration::from_secs(10), &prefix)
                .map(|m| specs_from_discovery(&m));
            let _ = tx.send(r);
        });
        self.discovery = Some(rx);
        *self.status.lock().unwrap() = "Searching for lights...".into();
    }

    fn start(&mut self) {
        let specs: Vec<LightSpec> = self.lights.iter().filter(|l| l.1).map(|l| l.0.clone()).collect();
        if specs.is_empty() {
            *self.status.lock().unwrap() = "Select at least one light".into();
            return;
        }
        self.stop.store(false, Ordering::SeqCst);
        self.running.store(true, Ordering::SeqCst);
        let (mode, preview, status) = (self.mode.clone(), self.preview.clone(), self.status.clone());
        let (stop, running) = (self.stop.clone(), self.running.clone());
        std::thread::spawn(move || {
            let r = engine_thread(specs, mode, preview, status.clone(), stop);
            *status.lock().unwrap() = r.err().unwrap_or_else(|| "Idle".into());
            running.store(false, Ordering::SeqCst);
        });
    }
}

fn engine_thread(
    specs: Vec<LightSpec>,
    mode: Shared<Mode>,
    preview: Shared<Vec<Rgb>>,
    status: Shared<String>,
    stop: Arc<AtomicBool>,
) -> Result<(), String> {
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
    let result = run_show(
        &mut lights,
        |total, t| {
            let m = *mode.lock().unwrap();
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
                *status.lock().unwrap() = format!("Running: {}", m.label());
            }
            let f = frame_for(m, total, t, level, luma_screen::capture_zones).unwrap_or_else(|e| {
                failure = Some(e);
                stop.store(true, Ordering::SeqCst);
                vec![(0, 0, 0); total]
            });
            *preview.lock().unwrap() = f.clone();
            f
        },
        || !stop.load(Ordering::SeqCst),
    );
    failure.map_or(result, Err)
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.discovery {
            if let Ok(r) = rx.try_recv() {
                self.discovery = None;
                match r {
                    Ok(v) => {
                        *self.status.lock().unwrap() = format!("Found {} light(s)", v.len());
                        self.lights = v.into_iter().map(|s| (s, true)).collect();
                    }
                    Err(e) => *self.status.lock().unwrap() = e,
                }
            }
        }
        let running = self.running.load(Ordering::SeqCst);
        ui.heading(RichText::new("luma").size(36.0));
        ui.add_space(8.0);
        let cur = *self.mode.lock().unwrap();
        ui.horizontal_wrapped(|ui| {
            for name in ["Microphone", "Screen", "Rainbow", "Chase", "Breathe", "Solid"] {
                let m = Mode::from_name(name).unwrap();
                let b = egui::Button::new(RichText::new(name).size(20.0)).selected(m == cur);
                if ui.add_sized([120.0, 50.0], b).clicked() {
                    *self.mode.lock().unwrap() = m;
                }
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Subnet");
            ui.text_edit_singleline(&mut self.subnet);
            if ui.add_enabled(self.discovery.is_none(), egui::Button::new("Find lights")).clicked() {
                self.find();
            }
        });
        for (s, on) in &mut self.lights {
            ui.checkbox(on, format!("{} {}", s.sku, s.ip));
        }
        ui.separator();
        let label = if running { "Stop" } else { "Start" };
        if ui.add_sized([160.0, 50.0], egui::Button::new(RichText::new(label).size(22.0))).clicked() {
            if running {
                self.stop.store(true, Ordering::SeqCst);
            } else {
                self.start();
            }
        }
        ui.label(self.status.lock().unwrap().clone());
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            for &(r, g, b) in self.preview.lock().unwrap().iter() {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 6.0, Color32::from_rgb(r, g, b));
            }
        });
        if running || self.discovery.is_some() {
            ui.ctx().request_repaint_after(Duration::from_millis(50));
        }
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "luma",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(App::new()))
        }),
    )
}


use std::env;
use std::time::Duration;
use luma_core::{app, patterns};

use luma_govee::{capability, lan, razer_light::RazerLight, whole_light::WholeLight};
use luma_core::domain::LightPort;

fn usage() -> ! {
    eprintln!(
        "usage:\n  luma discover [subnet-prefix]\n  luma lights [subnet-prefix]\n  luma probe <ip> <segments> [seconds]\n  luma pattern <ip> <sku> <solid|chase|breathe|rainbow> [seconds]\n  luma show <solid|chase|breathe|rainbow> <seconds> <ip:sku>..."
    );
    std::process::exit(2);
}

fn fail(e: String) -> ! {
    eprintln!("{e}");
    std::process::exit(1);
}

/// Picks the adapter for a model: razer for the per-segment models, whole-light for the rest.
fn make_light(ip: &str, sku: &str) -> Box<dyn LightPort> {
    let cap = capability(sku);
    if cap.razer {
        Box::new(RazerLight::new(ip, cap.segments))
    } else {
        Box::new(WholeLight::new(ip))
    }
}

fn pattern_named(name: &str) -> patterns::Pattern {
    patterns::NAMES.iter().find(|(n, _)| *n == name).map(|(_, f)| *f).unwrap_or_else(|| usage())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("discover") => {
            let prefix = args.get(1).map(String::as_str).unwrap_or("192.168.8");
            let found = lan::discover(Duration::from_secs(10), &[prefix.to_string()]).unwrap_or_else(|e| fail(e));
            for (id, d) in &found {
                println!("{}  {}  {}", d["ip"].as_str().unwrap_or("?"), d["sku"].as_str().unwrap_or("?"), id);
            }
            println!("{} device(s)", found.len());
        }
        Some("lights") => {
            let prefix = args.get(1).map(String::as_str).unwrap_or("192.168.8");
            let found = lan::discover(Duration::from_secs(10), &[prefix.to_string()]).unwrap_or_else(|e| fail(e));
            for d in found.values() {
                let sku = d["sku"].as_str().unwrap_or("?");
                let cap = capability(sku);
                let mode = if cap.razer { format!("razer, {} segments", cap.segments) } else { "whole-light".to_string() };
                println!("{}  {}  {}", d["ip"].as_str().unwrap_or("?"), sku, mode);
            }
        }
        Some("probe") if args.len() >= 3 => {
            let n: usize = args[2].parse().unwrap_or_else(|_| usage());
            let secs: u64 = args.get(3).map(|s| s.parse().unwrap_or_else(|_| usage())).unwrap_or(4);
            let mut light = RazerLight::new(&args[1], n);
            app::play(&mut light, patterns::solid, secs).unwrap_or_else(|e| fail(e));
            println!("probed {} with {n} segments for {secs}s; razer switched off", args[1]);
        }
        Some("pattern") if args.len() >= 4 => {
            let secs: u64 = args.get(4).map(|s| s.parse().unwrap_or_else(|_| usage())).unwrap_or(10);
            let mut light = make_light(&args[1], &args[2]);
            app::play(light.as_mut(), pattern_named(&args[3]), secs).unwrap_or_else(|e| fail(e));
        }
        // One pattern across several lights as one virtual strip, in the order given: ip:sku pairs.
        Some("show") if args.len() >= 4 => {
            let secs: u64 = args[2].parse().unwrap_or_else(|_| usage());
            let mut lights: Vec<Box<dyn LightPort>> = Vec::new();
            for pair in &args[3..] {
                // ip:sku or ip:sku:extra_ms, where extra_ms is hand-tuned device delay ping can't see
                let parts: Vec<&str> = pair.split(':').collect();
                let [ip, sku] = match parts.len() {
                    2 => [parts[0], parts[1]],
                    3 => [parts[0], parts[1]],
                    _ => usage(),
                };
                let extra = parts.get(2).map(|s| s.parse::<u64>().unwrap_or_else(|_| usage())).unwrap_or(0);
                let latency = luma_core::adapters::latency::measure(ip).unwrap_or_else(|e| fail(e))
                    + std::time::Duration::from_millis(extra);
                println!("{ip} {sku}: one-way latency {} ms", latency.as_millis());
                lights.push(Box::new(luma_core::adapters::latency::Delayed::new(make_light(ip, sku), latency)));
            }
            app::play_show(&mut lights, pattern_named(&args[1]), secs).unwrap_or_else(|e| fail(e));
        }
        // Prints the microphone loudness as a bar, ten times a second.
        Some("meter") => {
            let secs: u64 = args.get(1).map(|s| s.parse().unwrap_or_else(|_| usage())).unwrap_or(10);
            let (tx, rx) = std::sync::mpsc::channel();
            let _mic = luma_audio::Mic::open(tx).unwrap_or_else(|e| fail(e));
            let start = std::time::Instant::now();
            while start.elapsed() < Duration::from_secs(secs) {
                let level = luma_core::domain::latest(&rx).map_or(0.0, |s| s.level);
                let bars = (level * 200.0).min(60.0) as usize;
                println!("{level:5.3} {}", "#".repeat(bars));
                std::thread::sleep(Duration::from_millis(100));
            }
        }
        _ => usage(),
    }
}

mod adapters;
mod app;
mod domain;
mod patterns;

use std::env;
use std::time::Duration;

use adapters::{lan, razer_light::RazerLight, whole_light::WholeLight};
use domain::{capability, LightPort};

fn usage() -> ! {
    eprintln!(
        "usage:\n  govee-control discover [subnet-prefix]\n  govee-control lights [subnet-prefix]\n  govee-control probe <ip> <segments> [seconds]\n  govee-control pattern <ip> <sku> <solid|chase|breathe|rainbow> [seconds]\n  govee-control show <solid|chase|breathe|rainbow> <seconds> <ip:sku>..."
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
            let found = lan::discover(Duration::from_secs(10), prefix).unwrap_or_else(|e| fail(e));
            for (id, d) in &found {
                println!("{}  {}  {}", d["ip"].as_str().unwrap_or("?"), d["sku"].as_str().unwrap_or("?"), id);
            }
            println!("{} device(s)", found.len());
        }
        Some("lights") => {
            let prefix = args.get(1).map(String::as_str).unwrap_or("192.168.8");
            let found = lan::discover(Duration::from_secs(10), prefix).unwrap_or_else(|e| fail(e));
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
                let latency = adapters::latency::measure(ip).unwrap_or_else(|e| fail(e))
                    + std::time::Duration::from_millis(extra);
                println!("{ip} {sku}: one-way latency {} ms", latency.as_millis());
                lights.push(Box::new(adapters::latency::Delayed::new(make_light(ip, sku), latency)));
            }
            app::play_show(&mut lights, pattern_named(&args[1]), secs).unwrap_or_else(|e| fail(e));
        }
        _ => usage(),
    }
}

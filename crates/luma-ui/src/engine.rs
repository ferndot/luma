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
        todo!()
    }
    pub fn label(&self) -> String {
        todo!()
    }
}

/// Light specs from discovery JSON values (entries missing "ip" or "sku" are skipped).
pub fn specs_from_discovery(found: &BTreeMap<String, Value>) -> Vec<LightSpec> {
    todo!()
}

/// One frame for `mode`. `level` is the last mic level; `zones` captures the screen on demand.
pub fn frame_for(
    mode: Mode,
    total: usize,
    t: f64,
    level: f32,
    zones: impl FnOnce(usize) -> Result<Vec<Rgb>, String>,
) -> Result<Vec<Rgb>, String> {
    todo!()
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

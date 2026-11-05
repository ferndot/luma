//! Local web GUI for luma: a tiny JSON API plus one embedded page.
use luma_engine::{LightSpec, Mode, Show};
use serde::Serialize;

#[derive(Serialize, Debug, PartialEq)]
pub struct LightJson {
    pub sku: String,
    pub ip: String,
    pub razer: bool,
    pub segments: usize,
    pub selected: bool,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct StateJson {
    pub running: bool,
    pub mode: String,
    pub status: String,
    pub preview: Vec<[u8; 3]>,
    pub lights: Vec<LightJson>,
}

/// Known lights plus the set of selected IPs.
#[derive(Default)]
pub struct Lights {
    pub known: Vec<LightSpec>,
    pub selected: Vec<String>,
}

pub fn build_state(_show: &Show, _lights: &Lights) -> StateJson {
    todo!()
}

pub fn parse_mode(_name: &str) -> Result<Mode, String> {
    todo!()
}

pub fn set_selected(_lights: &mut Lights, _ips: Vec<String>) {
    todo!()
}

pub fn set_found(_lights: &mut Lights, _found: Vec<LightSpec>) {
    todo!()
}

pub fn selected_specs(_lights: &Lights) -> Vec<LightSpec> {
    todo!()
}

pub fn serve(_port: u16) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(ip: &str) -> LightSpec {
        LightSpec { sku: "H6022".into(), ip: ip.into(), razer: true, segments: 15 }
    }

    #[test]
    fn state_reflects_show_and_lights() {
        let show = Show::default();
        *show.preview.lock().unwrap() = vec![(1, 2, 3)];
        let mut l = Lights::default();
        set_found(&mut l, vec![spec("1.1.1.1"), spec("1.1.1.2")]);
        set_selected(&mut l, vec!["1.1.1.2".into()]);
        let s = build_state(&show, &l);
        assert!(!s.running);
        assert_eq!(s.mode, "rainbow");
        assert_eq!(s.status, "Idle");
        assert_eq!(s.preview, vec![[1, 2, 3]]);
        assert_eq!(s.lights.len(), 2);
        assert!(!s.lights[0].selected && s.lights[1].selected);
        assert!(serde_json::to_string(&s).unwrap().contains("\"preview\":[[1,2,3]]"));
    }

    #[test]
    fn mode_parsing() {
        assert_eq!(parse_mode("mic"), Ok(Mode::Mic));
        assert_eq!(parse_mode("chase"), Ok(Mode::Pattern("chase")));
        assert!(parse_mode("nope").is_err());
    }

    #[test]
    fn found_selects_all_and_selection_ignores_unknown() {
        let mut l = Lights::default();
        set_found(&mut l, vec![spec("a"), spec("b")]);
        assert_eq!(selected_specs(&l).len(), 2);
        set_selected(&mut l, vec!["b".into(), "zzz".into()]);
        assert_eq!(selected_specs(&l), vec![spec("b")]);
        set_selected(&mut l, vec![]);
        assert!(selected_specs(&l).is_empty());
    }
}

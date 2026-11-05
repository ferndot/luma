use serde_json::json;

use super::lan;
use crate::domain::{LightPort, Rgb};

/// A light with no razer support, driven as one segment through standard LAN commands.
pub struct WholeLight {
    ip: String,
    powered: bool,
    last: Option<Rgb>,
}

impl WholeLight {
    pub fn new(ip: &str) -> Self {
        Self { ip: ip.to_string(), powered: false, last: None }
    }
}

impl LightPort for WholeLight {
    fn segments(&self) -> usize {
        1
    }

    /// Uses the first color; the other segments don't exist on this model.
    fn show(&mut self, colors: &[Rgb]) -> Result<(), String> {
        let c = *colors.first().ok_or("no color given")?;
        if !self.powered {
            lan::send(&self.ip, &json!({ "msg": { "cmd": "turn", "data": { "value": 1 } } }))
                .map_err(|e| e.to_string())?;
            self.powered = true;
        }
        if self.last != Some(c) {
            let msg = json!({ "msg": { "cmd": "colorwc", "data": {
                "color": { "r": c.0, "g": c.1, "b": c.2 }, "colorTemInKelvin": 0 } } });
            lan::send(&self.ip, &msg).map_err(|e| e.to_string())?;
            self.last = Some(c);
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), String> {
        Ok(())
    }
}

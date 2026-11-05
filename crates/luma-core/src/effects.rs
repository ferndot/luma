//! Effects as data: a kind plus the colors and timing it uses. Built-in effects are plain
//! starting points that a user can edit or copy.
use crate::patterns::Rgb;
use serde::{Deserialize, Serialize};

const BLACK: Rgb = (0, 0, 0);
const WHITE: Rgb = (255, 255, 255);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Effect {
    /// One color everywhere.
    Solid { color: Rgb },
    /// Colors blend along the room and scroll. `stops` are spaced evenly around one cycle, which is
    /// `length` room widths long and moves `speed` cycles per second.
    Gradient { stops: Vec<Rgb>, length: f32, speed: f32 },
    /// A band of `color` sweeps across the room over `background`. One crossing takes `period`
    /// seconds and the band is `width` of the room wide.
    Band { color: Rgb, background: Rgb, width: f32, period: f32 },
    /// Fades between `from` and `to` and back, once every `period` seconds.
    Pulse { from: Rgb, to: Rgb, period: f32 },
    /// The microphone drives the brightness of a moving rainbow.
    Microphone,
    /// Each spot shows the screen color of its column.
    Screen,
}

impl Effect {
    /// The color at room position `(x, y)` and time `t` seconds, or None for the live sources
    /// (microphone and screen), which the engine renders itself.
    pub fn color_at(&self, x: f32, _y: f32, t: f64) -> Option<Rgb> {
        Some(match self {
            Effect::Solid { color } => *color,
            Effect::Gradient { stops, length, speed } => {
                let n = stops.len();
                if n == 0 {
                    return Some(BLACK);
                }
                let pos = (x as f64 / *length as f64 + *speed as f64 * t).rem_euclid(1.0) * n as f64;
                let i = pos.floor() as usize;
                let (a, b, f) = (stops[i % n], stops[(i + 1) % n], pos.fract());
                let mix = |p: u8, q: u8| (p as f64 + (q as f64 - p as f64) * f).round() as u8;
                (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
            }
            Effect::Band { color, background, width, period } => {
                let head = (t / *period as f64).rem_euclid(1.0) as f32;
                let d = (x - head).rem_euclid(1.0);
                if d.min(1.0 - d) < width / 2.0 { *color } else { *background }
            }
            Effect::Pulse { from, to, period } => {
                let level = ((t * std::f64::consts::TAU / *period as f64).sin() + 1.0) / 2.0;
                let mix = |p: u8, q: u8| (p as f64 + (q as f64 - p as f64) * level).round() as u8;
                (mix(from.0, to.0), mix(from.1, to.1), mix(from.2, to.2))
            }
            Effect::Microphone | Effect::Screen => return None,
        })
    }

    /// Checks the numbers are in a range the engine handles. Colors are always valid.
    pub fn check(&self) -> Result<(), String> {
        let range = |name: &str, v: f32, lo: f32, hi: f32| {
            if v.is_finite() && (lo..=hi).contains(&v) { Ok(()) } else { Err(format!("{name} must be between {lo} and {hi}")) }
        };
        match self {
            Effect::Gradient { stops, length, speed } => {
                if stops.is_empty() || stops.len() > 8 {
                    return Err("A gradient needs 1 to 8 colors".into());
                }
                range("Length", *length, 0.05, 4.0)?;
                range("Speed", *speed, -4.0, 4.0)
            }
            Effect::Band { width, period, .. } => {
                range("Width", *width, 0.02, 1.0)?;
                range("Period", *period, 0.5, 60.0)
            }
            Effect::Pulse { period, .. } => range("Period", *period, 0.5, 60.0),
            Effect::Solid { .. } | Effect::Microphone | Effect::Screen => Ok(()),
        }
    }
}

/// The effects every install starts with. Each has an id that scenes refer to.
pub fn builtin_effects() -> Vec<(&'static str, &'static str, Effect)> {
    vec![
        ("rainbow", "Rainbow", Effect::Gradient {
            stops: vec![(255, 0, 0), (255, 255, 0), (0, 255, 0), (0, 255, 255), (0, 0, 255), (255, 0, 255)],
            length: 1.0,
            speed: 0.25,
        }),
        ("chase", "Chase", Effect::Band { color: WHITE, background: BLACK, width: 0.16, period: 4.0 }),
        ("breathe", "Breathe", Effect::Pulse { from: BLACK, to: WHITE, period: 3.0 }),
        ("solid", "Solid", Effect::Solid { color: WHITE }),
        ("microphone", "Microphone", Effect::Microphone),
        ("screen", "Screen", Effect::Screen),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rainbow_matches_the_old_built_in_hue_sweep() {
        let rainbow = builtin_effects().into_iter().find(|(id, _, _)| *id == "rainbow").unwrap().2;
        // Left edge red at t=0; a quarter of the way across is yellow-green territory, not red.
        assert_eq!(rainbow.color_at(0.0, 0.5, 0.0), Some((255, 0, 0)));
        assert_ne!(rainbow.color_at(0.1, 0.5, 0.0), rainbow.color_at(0.6, 0.5, 0.0));
    }

    #[test]
    fn band_sits_where_its_head_is() {
        let chase = Effect::Band { color: WHITE, background: BLACK, width: 0.16, period: 4.0 };
        assert_eq!(chase.color_at(0.01, 0.5, 0.0), Some(WHITE));
        assert_eq!(chase.color_at(0.5, 0.5, 0.0), Some(BLACK));
    }

    #[test]
    fn pulse_fades_between_its_colors() {
        let breathe = Effect::Pulse { from: BLACK, to: WHITE, period: 3.0 };
        assert_eq!(breathe.color_at(0.0, 0.0, 0.0), Some((128, 128, 128)));
        assert_eq!(breathe.color_at(0.0, 0.0, 0.75), Some((255, 255, 255)));
    }

    #[test]
    fn live_sources_have_no_color_of_their_own() {
        assert_eq!(Effect::Screen.color_at(0.5, 0.5, 0.0), None);
        assert_eq!(Effect::Microphone.color_at(0.5, 0.5, 0.0), None);
    }

    #[test]
    fn bad_numbers_are_refused() {
        assert!(Effect::Gradient { stops: vec![], length: 1.0, speed: 0.0 }.check().is_err());
        assert!(Effect::Band { color: WHITE, background: BLACK, width: 0.0, period: 4.0 }.check().is_err());
        assert!(Effect::Pulse { from: BLACK, to: WHITE, period: f32::NAN }.check().is_err());
        assert!(builtin_effects().iter().all(|(_, _, e)| e.check().is_ok()));
    }
}

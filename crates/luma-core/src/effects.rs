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
    /// A direction through the room. The path runs from its first point to its last; the colors and
    /// brightness of its points travel along that direction across the whole room, `speed` lengths per second.
    Path { points: Vec<PathPoint>, speed: f32 },
    /// The microphone drives the brightness of a moving rainbow.
    Microphone,
    /// Each spot shows the screen color of its column.
    Screen,
}

/// A point on a path: where it is in the room (0 to 1 both ways), its color, and its brightness (0 to 1).
/// Its place along the path's direction decides when its color and brightness arrive.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PathPoint {
    pub at: [f32; 2],
    pub color: Rgb,
    pub brightness: f32,
}

/// How far along the path's direction a room position is: 0 at the path's first point, 1 at its last.
/// Positions off to the side still get a value, so the pattern covers the whole room.
fn along_direction(points: &[PathPoint], x: f32, y: f32) -> f64 {
    let (a, b) = (points[0].at, points[points.len() - 1].at);
    let d = [b[0] - a[0], b[1] - a[1]];
    let len2 = d[0] * d[0] + d[1] * d[1];
    if len2 <= 0.0 {
        return 0.0;
    }
    (((x - a[0]) * d[0] + (y - a[1]) * d[1]) / len2) as f64
}

/// The color and brightness at fraction `u` along the direction. The pattern loops: past the last
/// point it wraps back to the first.
fn sample_path(points: &[PathPoint], u: f64) -> (Rgb, f32) {
    let mut stops: Vec<(f64, &PathPoint)> = points
        .iter()
        .map(|p| (along_direction(points, p.at[0], p.at[1]).clamp(0.0, 1.0), p))
        .collect();
    stops.sort_by(|a, b| a.0.total_cmp(&b.0));
    let u = u.rem_euclid(1.0);
    // The last stop at or before u; if none, the last stop, which wraps round to the first.
    let n = stops.len();
    let i = stops.iter().rposition(|s| s.0 <= u).unwrap_or(n - 1);
    let j = (i + 1) % n;
    let (fi, fj) = (stops[i].0, stops[j].0);
    let span = if j > i { fj - fi } else { fj + 1.0 - fi };
    let uu = if u < fi { u + 1.0 } else { u };
    let f = if span > 0.0 { ((uu - fi) / span).clamp(0.0, 1.0) } else { 0.0 };
    let (a, b) = (stops[i].1, stops[j].1);
    let mix = |p: u8, q: u8| (p as f64 + (q as f64 - p as f64) * f).round() as u8;
    let color = (mix(a.color.0, b.color.0), mix(a.color.1, b.color.1), mix(a.color.2, b.color.2));
    (color, a.brightness + (b.brightness - a.brightness) * f as f32)
}

impl Effect {
    /// The color at room position `(x, y)` and time `t` seconds, or None for the live sources
    /// (microphone and screen), which the engine renders itself.
    pub fn color_at(&self, x: f32, y: f32, t: f64) -> Option<Rgb> {
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
            Effect::Path { points, speed } => {
                if points.len() < 2 {
                    return Some(BLACK);
                }
                let u = along_direction(points, x, y) - *speed as f64 * t;
                let (color, brightness) = sample_path(points, u);
                let scale = |c: u8| (c as f32 * brightness.clamp(0.0, 1.0)).round() as u8;
                (scale(color.0), scale(color.1), scale(color.2))
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
            Effect::Path { points, speed } => {
                if points.len() < 2 || points.len() > 16 {
                    return Err("A path needs 2 to 16 points".into());
                }
                for p in points {
                    range("Point x", p.at[0], 0.0, 1.0)?;
                    range("Point y", p.at[1], 0.0, 1.0)?;
                    range("Brightness", p.brightness, 0.0, 1.0)?;
                }
                range("Speed", *speed, -4.0, 4.0)
            }
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

    fn point(at: [f32; 2], color: Rgb, brightness: f32) -> PathPoint {
        PathPoint { at, color, brightness }
    }

    #[test]
    fn path_colors_run_along_its_direction_across_the_room() {
        // Red at the start of a left-to-right path, blue at the end.
        let path = Effect::Path {
            points: vec![point([0.0, 0.5], (255, 0, 0), 1.0), point([1.0, 0.5], (0, 0, 255), 1.0)],
            speed: 0.0,
        };
        assert_eq!(path.color_at(0.0, 0.5, 0.0), Some((255, 0, 0)));
        assert_eq!(path.color_at(0.5, 0.5, 0.0), Some((128, 0, 128)));
        // The direction is the same for the whole room, not just near the path.
        assert_eq!(path.color_at(0.5, 0.95, 0.0), Some((128, 0, 128)));
    }

    #[test]
    fn path_brightness_travels_with_its_color() {
        let path = Effect::Path {
            points: vec![point([0.0, 0.5], (255, 255, 255), 1.0), point([1.0, 0.5], (255, 255, 255), 0.0)],
            speed: 0.0,
        };
        assert_eq!(path.color_at(0.0, 0.5, 0.0), Some((255, 255, 255)));
        assert_eq!(path.color_at(0.5, 0.5, 0.0), Some((128, 128, 128)));
    }

    #[test]
    fn path_moves_along_its_direction_and_loops() {
        let path = Effect::Path {
            points: vec![point([0.0, 0.5], (255, 0, 0), 1.0), point([1.0, 0.5], (0, 0, 255), 1.0)],
            speed: 0.5,
        };
        // At speed 0.5 per second, after one second the spot at the start shows the middle of the path.
        assert_eq!(path.color_at(0.0, 0.5, 1.0), Some((128, 0, 128)));
    }

    #[test]
    fn path_checks_its_points() {
        let ok = Effect::Path { points: vec![point([0.0, 0.0], (1, 1, 1), 1.0), point([1.0, 1.0], (1, 1, 1), 1.0)], speed: 0.0 };
        assert!(ok.check().is_ok());
        let one = Effect::Path { points: vec![point([0.0, 0.0], (1, 1, 1), 1.0)], speed: 0.0 };
        assert!(one.check().is_err());
        let bright = Effect::Path { points: vec![point([0.0, 0.0], (1, 1, 1), 2.0), point([1.0, 1.0], (1, 1, 1), 1.0)], speed: 0.0 };
        assert!(bright.check().is_err());
    }

    #[test]
    fn live_sources_are_still_live() {
        assert_eq!(Effect::Microphone.color_at(0.0, 0.0, 0.0), None);
    }
}

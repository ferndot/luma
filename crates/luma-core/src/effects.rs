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
    /// A route through the room, shaped by `points` (a line from its first point to its last, or a
    /// ring; see `orbit`). Its `stops` are colors spread evenly along the route and flow along it at
    /// `speed` lengths per second. Without stops, the points' colors are used in order.
    Path {
        points: Vec<PathPoint>,
        #[serde(default)]
        stops: Vec<Stop>,
        speed: f32,
        #[serde(default)]
        orbit: Orbit,
    },
    /// The microphone drives the brightness of a moving rainbow.
    Microphone,
    /// Each spot shows the screen color of its column.
    Screen,
    /// Library effects played one after another. The app turns these into the show's sequence when
    /// it plays, so editing a step's effect changes every sequence that uses it.
    Sequence { steps: Vec<SequenceStep>, repeat: bool },
}

/// One step of a sequence: the id of a library effect and how long it runs.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SequenceStep {
    pub effect: String,
    pub secs: f32,
}

const MAX_SEQUENCE_STEPS: usize = 12;

/// How a path runs: a straight line from its first point to its last, or a ring around the room's
/// center, clockwise or counter-clockwise as seen on screen.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Orbit {
    #[default]
    Line,
    Clockwise,
    CounterClockwise,
}

/// A point that shapes a path: where it is in the room (0 to 1 both ways). Its color and brightness
/// are only used when a path has no stops, as a fallback for older saved paths.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PathPoint {
    pub at: [f32; 2],
    pub color: Rgb,
    pub brightness: f32,
}

/// One color of a path's palette, and how bright it is (0 to 1).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Stop {
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

/// Where a room position sits along a path, as a fraction. On a line it is the place along the direction;
/// on a ring it is the angle around the room's center, measured clockwise or counter-clockwise.
fn position(points: &[PathPoint], orbit: Orbit, x: f32, y: f32) -> f64 {
    let angle = ((y - 0.5) as f64).atan2((x - 0.5) as f64) / std::f64::consts::TAU;
    match orbit {
        Orbit::Line => along_direction(points, x, y),
        Orbit::Clockwise => angle.rem_euclid(1.0),
        Orbit::CounterClockwise => 1.0 - angle.rem_euclid(1.0),
    }
}

/// The color and brightness at fraction `u` of a path's palette. The palette's colors are spread
/// evenly around one loop, and blend into each other; past the last one it wraps back to the first.
fn sample_palette(palette: &[(Rgb, f32)], u: f64) -> (Rgb, f32) {
    let n = palette.len();
    let pos = u.rem_euclid(1.0) * n as f64;
    let i = (pos.floor() as usize) % n;
    let (a, b) = (palette[i], palette[(i + 1) % n]);
    let f = pos.fract();
    let mix = |p: u8, q: u8| (p as f64 + (q as f64 - p as f64) * f).round() as u8;
    let color = (mix(a.0.0, b.0.0), mix(a.0.1, b.0.1), mix(a.0.2, b.0.2));
    (color, a.1 + (b.1 - a.1) * f as f32)
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
            Effect::Path { points, stops, speed, orbit } => {
                if points.len() < 2 {
                    return Some(BLACK);
                }
                let palette: Vec<(Rgb, f32)> = if stops.is_empty() {
                    points.iter().map(|p| (p.color, p.brightness)).collect()
                } else {
                    stops.iter().map(|s| (s.color, s.brightness)).collect()
                };
                if palette.is_empty() {
                    return Some(BLACK);
                }
                let u = position(points, *orbit, x, y) - *speed as f64 * t;
                let (color, brightness) = sample_palette(&palette, u);
                let scale = |c: u8| (c as f32 * brightness.clamp(0.0, 1.0)).round() as u8;
                (scale(color.0), scale(color.1), scale(color.2))
            }
            // A sequence has no color of its own: the app resolves its steps before it plays.
            Effect::Microphone | Effect::Screen | Effect::Sequence { .. } => return None,
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
            Effect::Path { points, stops, speed, .. } => {
                if points.len() < 2 || points.len() > 16 {
                    return Err("A path needs 2 to 16 points".into());
                }
                if stops.len() > 16 {
                    return Err("A path can have at most 16 colors".into());
                }
                for p in points {
                    range("Point x", p.at[0], 0.0, 1.0)?;
                    range("Point y", p.at[1], 0.0, 1.0)?;
                    range("Brightness", p.brightness, 0.0, 1.0)?;
                }
                for s in stops {
                    range("Brightness", s.brightness, 0.0, 1.0)?;
                }
                range("Speed", *speed, -4.0, 4.0)
            }
            Effect::Sequence { steps, .. } => {
                if steps.is_empty() || steps.len() > MAX_SEQUENCE_STEPS {
                    return Err(format!("A sequence needs 1 to {MAX_SEQUENCE_STEPS} steps"));
                }
                for step in steps {
                    range("Step length", step.secs, 1.0, 600.0)?;
                }
                Ok(())
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

    fn path(points: Vec<PathPoint>, stops: Vec<Stop>, speed: f32, orbit: Orbit) -> Effect {
        Effect::Path { points, stops, speed, orbit }
    }

    fn stop(color: Rgb, brightness: f32) -> Stop {
        Stop { color, brightness }
    }

    #[test]
    fn path_colors_flow_along_its_direction_across_the_room() {
        // Two colors spread evenly along a left-to-right route: red, then blue, then back to red.
        let line = path(
            vec![point([0.0, 0.5], (0, 0, 0), 1.0), point([1.0, 0.5], (0, 0, 0), 1.0)],
            vec![stop((255, 0, 0), 1.0), stop((0, 0, 255), 1.0)],
            0.0,
            Orbit::Line,
        );
        assert_eq!(line.color_at(0.0, 0.5, 0.0), Some((255, 0, 0)));
        assert_eq!(line.color_at(0.25, 0.5, 0.0), Some((128, 0, 128)));
        // The direction is the same for the whole room, not just near the route.
        assert_eq!(line.color_at(0.25, 0.95, 0.0), Some((128, 0, 128)));
    }

    #[test]
    fn path_brightness_comes_from_its_stops() {
        let line = path(
            vec![point([0.0, 0.5], (0, 0, 0), 1.0), point([1.0, 0.5], (0, 0, 0), 1.0)],
            vec![stop((255, 255, 255), 1.0), stop((255, 255, 255), 0.0)],
            0.0,
            Orbit::Line,
        );
        assert_eq!(line.color_at(0.0, 0.5, 0.0), Some((255, 255, 255)));
        assert_eq!(line.color_at(0.25, 0.5, 0.0), Some((128, 128, 128)));
    }

    #[test]
    fn route_points_without_stops_fall_back_to_their_colors() {
        // Older saved paths have no stops: their point colors are the palette.
        let old = path(
            vec![point([0.0, 0.5], (255, 0, 0), 1.0), point([1.0, 0.5], (0, 0, 255), 1.0)],
            vec![],
            0.0,
            Orbit::Line,
        );
        assert_eq!(old.color_at(0.25, 0.5, 0.0), Some((128, 0, 128)));
    }

    #[test]
    fn path_moves_along_its_direction_and_loops() {
        let line = path(
            vec![point([0.0, 0.5], (0, 0, 0), 1.0), point([1.0, 0.5], (0, 0, 0), 1.0)],
            vec![stop((255, 0, 0), 1.0), stop((0, 0, 255), 1.0)],
            0.5,
            Orbit::Line,
        );
        // At speed 0.5 per second, after one second the start shows the second color.
        assert_eq!(line.color_at(0.0, 0.5, 1.0), Some((0, 0, 255)));
    }

    #[test]
    fn ring_path_runs_around_the_room_clockwise_or_counter() {
        // On screen, clockwise runs right, bottom, left, top. Counter-clockwise is the reverse.
        let route = vec![point([1.0, 0.5], (0, 0, 0), 1.0), point([0.5, 1.0], (0, 0, 0), 1.0)];
        assert!((position(&route, Orbit::Clockwise, 0.5, 1.0) - 0.25).abs() < 1e-9);
        assert!((position(&route, Orbit::Clockwise, 0.5, 0.0) - 0.75).abs() < 1e-9);
        assert!((position(&route, Orbit::CounterClockwise, 0.5, 1.0) - 0.75).abs() < 1e-9);
        // On a two-color ring, a quarter of the way round is halfway between the colors.
        let stops = vec![stop((255, 0, 0), 1.0), stop((0, 0, 255), 1.0)];
        let cw = path(route, stops, 0.0, Orbit::Clockwise);
        assert_eq!(cw.color_at(0.5, 1.0, 0.0), Some((128, 0, 128)));
        assert_eq!(cw.color_at(1.0, 0.5, 0.0), Some((255, 0, 0)));
    }

    #[test]
    fn path_checks_its_points_and_stops() {
        let route = || vec![point([0.0, 0.0], (1, 1, 1), 1.0), point([1.0, 1.0], (1, 1, 1), 1.0)];
        assert!(path(route(), vec![stop((1, 1, 1), 1.0)], 0.0, Orbit::Line).check().is_ok());
        let one = path(vec![point([0.0, 0.0], (1, 1, 1), 1.0)], vec![], 0.0, Orbit::Line);
        assert!(one.check().is_err());
        assert!(path(route(), vec![stop((1, 1, 1), 2.0)], 0.0, Orbit::Line).check().is_err());
    }

    #[test]
    fn sequences_are_checked_for_steps_and_length() {
        let step = |secs| SequenceStep { effect: "solid".into(), secs };
        let seq = |steps| Effect::Sequence { steps, repeat: true };
        assert!(seq(vec![step(5.0)]).check().is_ok());
        assert!(seq(vec![]).check().is_err());
        assert!(seq(vec![step(0.5)]).check().is_err());
        assert!(seq(vec![step(5.0); 13]).check().is_err());
        assert_eq!(seq(vec![step(5.0)]).color_at(0.0, 0.0, 0.0), None);
    }

    #[test]
    fn live_sources_are_still_live() {
        assert_eq!(Effect::Microphone.color_at(0.0, 0.0, 0.0), None);
    }
}

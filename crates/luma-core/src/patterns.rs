pub type Rgb = (u8, u8, u8);

/// Color at room position `(x, y)` and time `t` seconds. Both coordinates run from 0.0 to 1.0.
pub type Pattern = fn(x: f32, y: f32, t: f64) -> Rgb;

pub const WHITE: Rgb = (255, 255, 255);
const BLACK: Rgb = (0, 0, 0);

pub fn solid(_x: f32, _y: f32, _t: f64) -> Rgb {
    WHITE
}

/// A lit band travels across the room, left to right and wrapping, one crossing every 4 seconds.
pub fn chase(x: f32, _y: f32, t: f64) -> Rgb {
    let head = (t / 4.0).fract() as f32;
    let d = (x - head).rem_euclid(1.0);
    if d.min(1.0 - d) < 0.08 { WHITE } else { BLACK }
}

/// The whole room fades in and out together, one cycle every 3 seconds.
pub fn breathe(_x: f32, _y: f32, t: f64) -> Rgb {
    let level = ((t * std::f64::consts::TAU / 3.0).sin() + 1.0) / 2.0;
    let v = (255.0 * level).round() as u8;
    (v, v, v)
}

/// Hue changes across the room from left to right, and shifts over time.
pub fn rainbow(x: f32, _y: f32, t: f64) -> Rgb {
    hsv_to_rgb((x as f64 + t * 0.25).rem_euclid(1.0), 1.0, 1.0)
}

pub const NAMES: &[(&str, Pattern)] = &[
    ("solid", solid),
    ("chase", chase),
    ("breathe", breathe),
    ("rainbow", rainbow),
];

/// Evenly spaced spots along a line, for devices with no room position yet.
pub fn row(n: usize) -> Vec<(f32, f32)> {
    (0..n).map(|i| ((i as f32 + 0.5) / n.max(1) as f32, 0.5)).collect()
}

/// One frame of `pattern`: a color for each spot at time `t`.
pub fn frame(pattern: Pattern, spots: &[(f32, f32)], t: f64) -> Vec<Rgb> {
    spots.iter().map(|&(x, y)| pattern(x, y, t)).collect()
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> Rgb {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let r = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i as i64 % 6 {
        0 => (v, r, p),
        1 => (q, v, p),
        2 => (p, v, r),
        3 => (p, q, v),
        4 => (r, p, v),
        _ => (v, p, q),
    };
    ((r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_have_one_color_per_spot() {
        for (_, f) in NAMES {
            assert_eq!(frame(*f, &row(15), 1.0).len(), 15);
        }
    }

    #[test]
    fn chase_band_sits_where_its_head_is() {
        // At t = 0 the band starts at x = 0, so a spot near 0 is lit and one at 0.5 is not.
        assert_eq!(chase(0.01, 0.5, 0.0), WHITE);
        assert_eq!(chase(0.5, 0.5, 0.0), BLACK);
    }

    #[test]
    fn rainbow_changes_across_the_room() {
        assert_ne!(rainbow(0.1, 0.5, 0.0), rainbow(0.6, 0.5, 0.0));
    }

    #[test]
    fn hsv_primaries() {
        assert_eq!(hsv_to_rgb(0.0, 1.0, 1.0), (255, 0, 0));
        assert_eq!(hsv_to_rgb(1.0 / 3.0, 1.0, 1.0), (0, 255, 0));
    }
}

pub type Rgb = (u8, u8, u8);

/// One frame of a pattern: `n` segment colors at time `t` seconds.
pub type Pattern = fn(n: usize, t: f64) -> Vec<Rgb>;

pub const WHITE: Rgb = (255, 255, 255);

pub fn solid(n: usize, _t: f64) -> Vec<Rgb> {
    vec![WHITE; n]
}

/// One lit segment that moves along the strip, 4 segments per second.
pub fn chase(n: usize, t: f64) -> Vec<Rgb> {
    let lit = (t * 4.0) as usize % n;
    (0..n).map(|i| if i == lit { WHITE } else { (0, 0, 0) }).collect()
}

/// The whole strip fades in and out, one cycle every 3 seconds.
pub fn breathe(n: usize, t: f64) -> Vec<Rgb> {
    let level = ((t * std::f64::consts::TAU / 3.0).sin() + 1.0) / 2.0;
    let v = (255.0 * level).round() as u8;
    vec![(v, v, v); n]
}

/// Hue spread across the strip, shifting over time.
pub fn rainbow(n: usize, t: f64) -> Vec<Rgb> {
    (0..n)
        .map(|i| hsv_to_rgb(((i as f64 / n as f64) + t * 0.25).fract(), 1.0, 1.0))
        .collect()
}

pub const NAMES: &[(&str, Pattern)] = &[
    ("solid", solid),
    ("chase", chase),
    ("breathe", breathe),
    ("rainbow", rainbow),
];

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
    fn frames_have_requested_length() {
        for (_, f) in NAMES {
            assert_eq!(f(15, 1.0).len(), 15);
        }
    }

    #[test]
    fn chase_lights_exactly_one_segment() {
        let lit = chase(8, 0.3).iter().filter(|c| **c != (0, 0, 0)).count();
        assert_eq!(lit, 1);
    }

    #[test]
    fn rainbow_is_not_monochrome() {
        let f = rainbow(6, 0.0);
        assert_ne!(f[0], f[3]);
    }

    #[test]
    fn hsv_primaries() {
        assert_eq!(hsv_to_rgb(0.0, 1.0, 1.0), (255, 0, 0));
        assert_eq!(hsv_to_rgb(1.0 / 3.0, 1.0, 1.0), (0, 255, 0));
    }
}

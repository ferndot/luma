//! Input modes: turn a microphone level or screen zone colors into a frame for the virtual strip.
use crate::domain::Rgb;

/// Stretches or shrinks `zones` to `n` segments by nearest neighbour. Empty zones give black.
pub fn resample(zones: &[Rgb], n: usize) -> Vec<Rgb> {
    if zones.is_empty() {
        return vec![(0, 0, 0); n];
    }
    (0..n).map(|i| zones[i * zones.len() / n]).collect()
}

/// Screen mode: the screen's zone colors laid across the strip.
pub fn screen_frame(n: usize, zones: &[Rgb]) -> Vec<Rgb> {
    resample(zones, n)
}

/// Mic mode: a hue rainbow that moves over time, brightness following `level` (0.0 to 1.0).
pub fn mic_frame(n: usize, level: f32, t: f64) -> Vec<Rgb> {
    let level = level.clamp(0.0, 1.0) as f64;
    (0..n)
        .map(|i| {
            let hue = (i as f64 / n.max(1) as f64 + t * 0.25).fract();
            let (r, g, b) = hsv(hue);
            ((r * level * 255.0) as u8, (g * level * 255.0) as u8, (b * level * 255.0) as u8)
        })
        .collect()
}

/// Full-saturation hue wheel, 0.0 to 1.0 as RGB in 0.0 to 1.0.
fn hsv(h: f64) -> (f64, f64, f64) {
    let x = 1.0 - ((h * 6.0) % 2.0 - 1.0).abs();
    match (h * 6.0) as u32 % 6 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_same_length_is_identity() {
        let z = [(255, 0, 0), (0, 0, 255)];
        assert_eq!(resample(&z, 2), z.to_vec());
    }

    #[test]
    fn resample_upsamples_by_repeating() {
        let z = [(255, 0, 0), (0, 0, 255)];
        assert_eq!(resample(&z, 4), vec![(255, 0, 0), (255, 0, 0), (0, 0, 255), (0, 0, 255)]);
    }

    #[test]
    fn resample_empty_zones_is_black() {
        assert_eq!(resample(&[], 3), vec![(0, 0, 0); 3]);
    }

    #[test]
    fn screen_frame_has_requested_length() {
        assert_eq!(screen_frame(7, &[(1, 2, 3)]).len(), 7);
    }

    #[test]
    fn mic_silence_is_black() {
        assert!(mic_frame(5, 0.0, 1.0).iter().all(|c| *c == (0, 0, 0)));
    }

    #[test]
    fn mic_loud_is_lit_and_right_length() {
        let f = mic_frame(5, 1.0, 1.0);
        assert_eq!(f.len(), 5);
        assert!(f.iter().any(|c| *c != (0, 0, 0)));
    }
}

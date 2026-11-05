//! Screen input: averages the primary display into left-to-right color zones.
use luma_core::domain::Rgb;

/// Captures the primary display and returns its average color per zone, left to right.
pub fn capture_zones(zones: usize) -> Result<Vec<Rgb>, String> {
    let monitors = xcap::Monitor::all().map_err(|e| e.to_string())?;
    let mut primary = None;
    for m in monitors {
        if m.is_primary().map_err(|e| e.to_string())? {
            primary = Some(m);
            break;
        }
    }
    let monitor = primary.ok_or("no primary monitor")?;
    let img = monitor.capture_image().map_err(|e| e.to_string())?;
    Ok(zone_averages(img.as_raw(), img.width() as usize, img.height() as usize, zones))
}

/// Average color of `zones` equal-width vertical slices of an RGBA image (row-major, 4 bytes per pixel).
pub fn zone_averages(rgba: &[u8], width: usize, height: usize, zones: usize) -> Vec<Rgb> {
    let step = (width.min(height) / 270).max(1);
    (0..zones)
        .map(|z| {
            let (x0, x1) = (z * width / zones, (z + 1) * width / zones);
            let (mut r, mut g, mut b, mut count) = (0u64, 0u64, 0u64, 0u64);
            // Every 4th pixel on a 1080p screen: the average barely changes and the work drops 16x.
            for y in (0..height).step_by(step) {
                for x in (x0..x1).step_by(step) {
                    let p = (y * width + x) * 4;
                    r += rgba[p] as u64;
                    g += rgba[p + 1] as u64;
                    b += rgba[p + 2] as u64;
                    count += 1;
                }
            }
            let avg = |s: u64| if count == 0 { 0 } else { ((s + count / 2) / count) as u8 };
            (avg(r), avg(g), avg(b))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_red_right_blue_gives_two_zones() {
        let (w, h) = (4, 2);
        let mut px = Vec::new();
        for _ in 0..h {
            for x in 0..w {
                px.extend(if x < 2 { [255, 0, 0, 255] } else { [0, 0, 255, 255] });
            }
        }
        assert_eq!(zone_averages(&px, w, h, 2), vec![(255, 0, 0), (0, 0, 255)]);
    }

    #[test]
    fn averages_mixed_pixels() {
        // two pixels in one zone: black and white -> grey
        let px = [0, 0, 0, 255, 255, 255, 255, 255];
        assert_eq!(zone_averages(&px, 2, 1, 1), vec![(128, 128, 128)]);
    }
}

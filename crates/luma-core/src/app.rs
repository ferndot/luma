use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::domain::{LightPort, Rgb};
use crate::patterns::{Pattern, frame, row};

/// Plays a pattern on any light at 10 fps for `secs` seconds. Pattern length follows the light's segments.
pub fn play(light: &mut dyn LightPort, f: Pattern, secs: u64) -> Result<(), String> {
    let start = Instant::now();
    let n = light.segments();
    while start.elapsed() < Duration::from_secs(secs) {
        light.show(&frame(f, &row(n), start.elapsed().as_secs_f64()))?;
        sleep(Duration::from_millis(100));
    }
    light.close()
}

/// Plays one pattern across several lights as a single virtual strip, in the order given.
/// Each light takes the next `segments()` colors, so a pattern spans all lights.
/// Each tick renders the frame for a shared display time, then sends to each light early by its
/// own latency, so every light shows that frame at the same moment.
pub fn play_show(lights: &mut [Box<dyn LightPort>], f: Pattern, secs: u64) -> Result<(), String> {
    let start = Instant::now();
    run_show(lights, |total, t| frame(f, &row(total), t), || start.elapsed() < Duration::from_secs(secs))
}

/// The show loop with the frame source and stop condition supplied by the caller.
/// `frame_at(total_segments, seconds)` returns one color per virtual segment.
/// `keep_going()` is checked once per tick. Lights are closed when the loop ends.
pub fn run_show(
    lights: &mut [Box<dyn LightPort>],
    mut frame_at: impl FnMut(usize, f64) -> Vec<Rgb>,
    keep_going: impl Fn() -> bool,
) -> Result<(), String> {
    let start = Instant::now();
    let total: usize = lights.iter().map(|l| l.segments()).sum();
    let mut offsets = Vec::with_capacity(lights.len());
    let mut acc = 0;
    for l in lights.iter() {
        offsets.push(acc);
        acc += l.segments();
    }
    // Send to the slowest light first, so it gets the earliest start.
    let mut order: Vec<usize> = (0..lights.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(lights[i].latency()));
    let lead = lights.iter().map(|l| l.latency()).max().unwrap_or_default() + Duration::from_millis(20);

    while keep_going() {
        let tick = Instant::now();
        let display_at = tick + lead;
        let frame = frame_at(total, (display_at - start).as_secs_f64());
        for &i in &order {
            let send_at = display_at - lights[i].latency();
            sleep(send_at.saturating_duration_since(Instant::now()));
            let n = lights[i].segments();
            lights[i].show(&frame[offsets[i]..offsets[i] + n])?;
        }
        sleep((tick + Duration::from_millis(100)).saturating_duration_since(Instant::now()));
    }
    for light in lights.iter_mut() {
        light.close()?;
    }
    Ok(())
}

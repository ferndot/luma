/// Core types and the port every device adapter implements. No I/O here.
pub type Rgb = (u8, u8, u8);

/// A light the app can drive. Adapters implement this; the use cases only see this trait.
pub trait LightPort {
    /// Number of independently addressable segments; 1 for whole-light devices.
    fn segments(&self) -> usize;
    /// Show one color per segment. `colors.len()` equals `segments()`.
    fn show(&mut self, colors: &[Rgb]) -> Result<(), String>;
    /// Return the light to its normal mode.
    fn close(&mut self) -> Result<(), String>;
    /// One-way delay from sending a frame to the light showing it. Zero unless measured.
    fn latency(&self) -> std::time::Duration {
        std::time::Duration::ZERO
    }
}



/// Input port: sources (microphone, screen) push `Signal`s into a channel; the show reads the newest.
pub type SignalInput = std::sync::mpsc::Receiver<Signal>;

/// Drains the channel and returns the newest signal, or `None` if nothing new arrived.
pub fn latest(rx: &SignalInput) -> Option<Signal> {
    rx.try_iter().last()
}

/// What the show reads each tick. `level` is loudness from 0.0 to 1.0; screen colors will join later.
#[derive(Clone, Copy, Default)]
pub struct Signal {
    pub level: f32,
}

#[cfg(test)]
mod signal_tests {
    use super::*;
    use std::sync::mpsc::channel;

    fn sig(level: f32) -> Signal {
        Signal { level }
    }

    #[test]
    fn latest_is_none_when_nothing_sent() {
        let (_tx, rx) = channel::<Signal>();
        assert!(latest(&rx).is_none());
    }

    #[test]
    fn latest_returns_newest_and_drains_older() {
        let (tx, rx) = channel();
        tx.send(sig(0.1)).unwrap();
        tx.send(sig(0.2)).unwrap();
        tx.send(sig(0.9)).unwrap();
        assert_eq!(latest(&rx).unwrap().level, 0.9);
        assert!(latest(&rx).is_none());
    }

    #[test]
    fn latest_works_across_threads() {
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            for i in 0..5 {
                tx.send(sig(i as f32)).unwrap();
            }
        })
        .join()
        .unwrap();
        assert_eq!(latest(&rx).unwrap().level, 4.0);
    }
}

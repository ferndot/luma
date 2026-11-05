//! Microphone input: pushes loudness readings into a `Signal` channel.
use std::sync::mpsc::Sender;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use luma_core::domain::Signal;

/// Keeps the capture stream alive. Each audio buffer sends one `Signal` (RMS loudness, 0.0 to 1.0).
pub struct Mic {
    _stream: cpal::Stream,
}

impl Mic {
    /// Opens the default input device. Only 32-bit float input is supported for now.
    pub fn open(tx: Sender<Signal>) -> Result<Self, String> {
        let dev = cpal::default_host().default_input_device().ok_or("no input device")?;
        let cfg = dev.default_input_config().map_err(|e| e.to_string())?;
        if cfg.sample_format() != cpal::SampleFormat::F32 {
            return Err(format!("unsupported sample format {:?}", cfg.sample_format()));
        }
        let channels = cfg.channels() as usize;
        let stream = dev
            .build_input_stream(
                cfg.config(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // The receiver may be gone at shutdown; a failed send is harmless.
                    let _ = tx.send(Signal { level: rms(data, channels) });
                },
                |e| eprintln!("mic error: {e}"),
                None,
            )
            .map_err(|e| e.to_string())?;
        stream.play().map_err(|e| e.to_string())?;
        Ok(Self { _stream: stream })
    }
}

/// RMS of the first channel of each interleaved frame.
fn rms(data: &[f32], channels: usize) -> f32 {
    let frames = (data.len() / channels).max(1);
    let sum: f32 = data.iter().step_by(channels).map(|s| s * s).sum();
    (sum / frames as f32).sqrt()
}

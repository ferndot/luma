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

/// What the app needs to know about a model to pick an adapter.
#[derive(Clone, Copy)]
pub struct Capability {
    pub razer: bool,
    pub segments: usize,
}

/// Models with confirmed razer per-segment control on LAN.
/// Anything else is driven as one whole-light segment.
pub fn capability(sku: &str) -> Capability {
    match sku {
        "H6022" => Capability { razer: true, segments: 15 },
        "H1771" => Capability { razer: true, segments: 4 },
        "H60B0" => Capability { razer: true, segments: 8 },
        "H60B2" => Capability { razer: true, segments: 3 },
        _ => Capability { razer: false, segments: 1 },
    }
}

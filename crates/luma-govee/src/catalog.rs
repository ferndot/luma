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

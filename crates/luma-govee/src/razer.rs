use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};

pub const ENABLE: u8 = 0xB1;
pub const FRAME: u8 = 0xB0;
pub const MAX_SEGMENTS: usize = 15;

pub fn checksum(data: &[u8]) -> u8 {
    data.iter().fold(0, |acc, b| acc ^ b)
}

/// BB 00 <size> <cmd> <payload> <checksum>; size = payload length.
pub fn packet(cmd: u8, payload: &[u8]) -> Vec<u8> {
    let mut p = vec![0xBB, 0x00, payload.len() as u8, cmd];
    p.extend_from_slice(payload);
    let sum = checksum(&p);
    p.push(sum);
    p
}

pub fn wrap(binary: &[u8]) -> Value {
    json!({ "msg": { "cmd": "razer", "data": { "pt": STANDARD.encode(binary) } } })
}

pub fn enable_message(on: bool) -> Value {
    wrap(&packet(ENABLE, &[on as u8]))
}

pub fn frame_message(colors: &[(u8, u8, u8)]) -> Result<Value, String> {
    if colors.is_empty() || colors.len() > MAX_SEGMENTS {
        return Err(format!("need 1..={MAX_SEGMENTS} segments, got {}", colors.len()));
    }
    let mut payload = vec![0x00, colors.len() as u8];
    for &(r, g, b) in colors {
        payload.extend_from_slice(&[r, g, b]);
    }
    Ok(wrap(&packet(FRAME, &payload)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enable_packet_matches_spec_vector() {
        assert_eq!(packet(ENABLE, &[1]), vec![0xBB, 0x00, 0x01, 0xB1, 0x01, 0x0A]);
    }

    #[test]
    fn frame_rejects_bad_counts() {
        assert!(frame_message(&[]).is_err());
        assert!(frame_message(&[(0, 0, 0); 16]).is_err());
        assert!(frame_message(&[(0, 0, 0); 15]).is_ok());
    }
}

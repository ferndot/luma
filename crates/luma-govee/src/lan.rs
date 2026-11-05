use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::time::{Duration, Instant};

const MULTICAST: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);
const SCAN_PORT: u16 = 4001;
const REPLY_PORT: u16 = 4002;
const CONTROL_PORT: u16 = 4003;

pub fn send(ip: &str, msg: &Value) -> std::io::Result<()> {
    let sock = UdpSocket::bind("0.0.0.0:0")?;
    sock.send_to(msg.to_string().as_bytes(), (ip, CONTROL_PORT))?;
    Ok(())
}

/// Returns the `data` object of each device that answered the scan, keyed by device id.
pub fn discover(timeout: Duration, prefix: &str) -> Result<BTreeMap<String, Value>, String> {
    let rx = UdpSocket::bind(("0.0.0.0", REPLY_PORT))
        .map_err(|e| format!("cannot bind UDP {REPLY_PORT} (another Govee LAN client running?): {e}"))?;
    rx.set_read_timeout(Some(Duration::from_millis(200))).map_err(|e| e.to_string())?;

    let tx = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    tx.set_multicast_ttl_v4(2).map_err(|e| e.to_string())?;
    let scan = json!({ "msg": { "cmd": "scan", "data": { "account_topic": "reserve" } } });
    tx.send_to(scan.to_string().as_bytes(), SocketAddrV4::new(MULTICAST, SCAN_PORT))
        .map_err(|e| e.to_string())?;
    // Fallback: multicast is often dropped on Wi-Fi, so also scan each host on the subnet.
    // shortcut: /24 prefix passed by the caller, derive from the adapter address later
    for host in 1..=254u8 {
        let addr = format!("{prefix}.{host}:{SCAN_PORT}");
        let _ = tx.send_to(scan.to_string().as_bytes(), addr.as_str());
    }

    let mut found = BTreeMap::new();
    let deadline = Instant::now() + timeout;
    let mut buf = [0u8; 4096];
    while Instant::now() < deadline {
        if let Ok((n, src)) = rx.recv_from(&mut buf) {
            if let Ok(v) = serde_json::from_slice::<Value>(&buf[..n]) {
                if v["msg"]["cmd"] == "scan" {
                    let mut data = v["msg"]["data"].clone();
                    // Some replies omit "ip"; the packet's source address is the device's IP.
                    if data["ip"].is_null() {
                        data["ip"] = json!(src.ip().to_string());
                    }
                    if let Some(id) = data["device"].as_str() {
                        found.insert(id.to_string(), data);
                    }
                }
            }
        }
    }
    Ok(found)
}

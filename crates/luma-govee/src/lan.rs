use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddrV4, UdpSocket};
use std::time::{Duration, Instant};

const MULTICAST: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);
const SCAN_PORT: u16 = 4001;
const REPLY_PORT: u16 = 4002;
const CONTROL_PORT: u16 = 4003;

/// The /24 prefix (e.g. "192.168.1") of an IPv4 address, or None for addresses that can't be lights' LAN.
fn prefix_of(ip: Ipv4Addr) -> Option<String> {
    if ip.is_loopback() || ip.is_unspecified() || ip.is_link_local() {
        return None;
    }
    let o = ip.octets();
    Some(format!("{}.{}.{}", o[0], o[1], o[2]))
}

/// The /24 prefix of the adapter the OS routes internet traffic through.
/// Connecting a UDP socket sends no packets; it only asks the OS which address it would use.
fn default_route_prefix() -> Option<String> {
    let sock = UdpSocket::bind("0.0.0.0:0").ok()?;
    sock.connect("8.8.8.8:80").ok()?;
    match sock.local_addr().ok()?.ip() {
        IpAddr::V4(ip) => prefix_of(ip),
        IpAddr::V6(_) => None,
    }
}

/// Every /24 worth scanning, most likely first: the default route, then each local adapter,
/// then the fallbacks. Duplicates are dropped and the order is kept.
pub fn local_prefixes(fallbacks: &[&str]) -> Vec<String> {
    let adapters = if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|iface| match iface.ip() {
            IpAddr::V4(ip) => prefix_of(ip),
            IpAddr::V6(_) => None,
        })
        .collect();
    merge_prefixes(default_route_prefix(), adapters, fallbacks)
}

fn merge_prefixes(first: Option<String>, adapters: Vec<String>, fallbacks: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let candidates = first.into_iter().chain(adapters).chain(fallbacks.iter().map(|s| s.to_string()));
    for p in candidates {
        if !out.contains(&p) {
            out.push(p);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes_keep_order_and_drop_duplicates() {
        let got = merge_prefixes(
            Some("192.168.10".into()),
            vec!["10.0.0".into(), "192.168.10".into(), "172.17.0".into()],
            &["192.168.10", "192.168.1"],
        );
        assert_eq!(got, vec!["192.168.10", "10.0.0", "172.17.0", "192.168.1"]);
    }

    #[test]
    fn prefix_of_skips_loopback_and_link_local() {
        assert_eq!(prefix_of(Ipv4Addr::new(192, 168, 10, 7)), Some("192.168.10".into()));
        assert_eq!(prefix_of(Ipv4Addr::LOCALHOST), None);
        assert_eq!(prefix_of(Ipv4Addr::new(169, 254, 3, 4)), None);
    }
}

pub fn send(ip: &str, msg: &Value) -> std::io::Result<()> {
    let sock = UdpSocket::bind("0.0.0.0:0")?;
    sock.send_to(msg.to_string().as_bytes(), (ip, CONTROL_PORT))?;
    Ok(())
}

/// Returns the `data` object of each device that answered the scan, keyed by device id.
/// Each prefix (e.g. "192.168.10") is scanned host by host, plus one multicast scan.
pub fn discover(timeout: Duration, prefixes: &[String]) -> Result<BTreeMap<String, Value>, String> {
    let rx = UdpSocket::bind(("0.0.0.0", REPLY_PORT))
        .map_err(|e| format!("cannot bind UDP {REPLY_PORT} (another Govee LAN client running?): {e}"))?;
    rx.set_read_timeout(Some(Duration::from_millis(200))).map_err(|e| e.to_string())?;

    let tx = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    tx.set_multicast_ttl_v4(2).map_err(|e| e.to_string())?;
    let scan = json!({ "msg": { "cmd": "scan", "data": { "account_topic": "reserve" } } });
    tx.send_to(scan.to_string().as_bytes(), SocketAddrV4::new(MULTICAST, SCAN_PORT))
        .map_err(|e| e.to_string())?;
    // Fallback: multicast is often dropped on Wi-Fi, so also scan each host on every subnet.
    for prefix in prefixes {
        for host in 1..=254u8 {
            let addr = format!("{prefix}.{host}:{SCAN_PORT}");
            let _ = tx.send_to(scan.to_string().as_bytes(), addr.as_str());
        }
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

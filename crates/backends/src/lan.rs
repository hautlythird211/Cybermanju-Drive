use cybermanju_types::sync::{RemoteFile, StorageBackend, SyncBackendType};
use sha2::{Digest, Sha256};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

/// Validate a LAN peer address: must be a valid SocketAddr.
/// Accepts private IPs, VPN IPs (Tailscale 100.64.0.0/10, WireGuard 10.0.0.0/8), loopback.
fn validate_peer_addr(addr: &SocketAddr) -> bool {
    match addr.ip() {
        IpAddr::V4(ip) => {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip == Ipv4Addr::new(0, 0, 0, 0)
                || is_vpn_ip(&ip)
        }
        IpAddr::V6(ip) => ip.is_loopback() || ip.is_unique_local() || is_vpn_ipv6(&ip),
    }
}

/// Check if an IPv4 is in known VPN ranges:
/// - Tailscale: 100.64.0.0/10 (CGNAT range)
/// - WireGuard common: 10.0.0.0/8, 172.16.0.0/12
/// - OpenVPN: 10.0.0.0/8, 172.16.0.0/12
fn is_vpn_ip(ip: &Ipv4Addr) -> bool {
    let o = ip.octets();
    // Tailscale CGNAT: 100.64.0.0/10 (100.64.x.x - 100.127.x.x)
    if o[0] == 100 && (o[1] & 0xC0) == 64 {
        return true;
    }
    // Common VPN ranges
    if o[0] == 10 {
        return true;
    }
    if o[0] == 172 && (o[1] & 0xF0) == 16 {
        return true;
    }
    false
}

/// Check if an IPv6 is a VPN address (ULA range fc00::/7).
fn is_vpn_ipv6(ip: &std::net::Ipv6Addr) -> bool {
    let segments = ip.segments();
    (segments[0] & 0xFE00) == 0xFC00
}

/// Sanitize a file path from a peer: reject traversal attempts.
fn sanitize_peer_path(path: &str) -> Result<String, String> {
    if path.contains("..") || path.contains('\\') {
        return Err(format!("path traversal rejected: {}", path));
    }
    if path.starts_with('/') || path.starts_with('\\') {
        return Err(format!("absolute path rejected: {}", path));
    }
    let clean = path.trim_start_matches('/');
    if clean.is_empty() {
        return Err("empty path".into());
    }
    if clean.contains('\0') {
        return Err(format!("null byte rejected: {}", path));
    }
    Ok(clean.to_string())
}

/// mDNS service type for Cybermanju Drive discovery.
const MDNS_SERVICE: &str = "_cybermanju._tcp.local.";
const DEFAULT_HTTP_PORT: u16 = 3457;
const CHALLENGE_TIMEOUT: u64 = 5;

/// DNS record types.
const DNS_TYPE_PTR: u16 = 12;
const DNS_TYPE_TXT: u16 = 16;
const DNS_TYPE_SRV: u16 = 33;
const DNS_TYPE_A: u16 = 1;
const DNS_CLASS_IN: u16 = 1;

/// Discovery mode — how to find peers.
#[derive(Debug, Clone, PartialEq)]
pub enum DiscoveryMode {
    /// Standard mDNS multicast (works on LAN without VPN)
    MdnsMulticast,
    /// Tailscale integration — use `tailscale status` for peer discovery
    Tailscale,
    /// WireGuard — manual peer list required
    WireGuard,
    /// Manual — user provides peer addresses explicitly
    Manual,
}

/// Network interface type.
#[derive(Debug, Clone)]
pub enum InterfaceType {
    /// Standard Ethernet/WiFi
    Lan,
    /// Tailscale VPN (tailscale0)
    Tailscale,
    /// WireGuard VPN (wg0, tun0)
    WireGuard,
    /// OpenVPN (tun0, tun1)
    OpenVpn,
    /// Unknown
    Unknown,
}

/// Detected network interface.
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub ip: Ipv4Addr,
    pub interface_type: InterfaceType,
    pub is_up: bool,
}

/// Discovered peer on LAN via mDNS or VPN discovery.
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    pub name: String,
    pub addr: SocketAddr,
    pub public_key: Vec<u8>,
    pub port: u16,
    pub verified: bool,
    pub interface: Option<String>,
}

/// VPN provider detection result.
#[derive(Debug, Clone)]
pub struct VpnStatus {
    pub connected: bool,
    pub provider: String,
    pub interface: Option<String>,
    pub local_ip: Option<Ipv4Addr>,
}

/// LAN backend for peer-to-peer sync via mDNS or VPN discovery.
/// Supports: standard LAN, Tailscale, WireGuard, OpenVPN, manual peers.
pub struct LanBackend {
    service_name: String,
    device_signing_key: Vec<u8>,
    peers: std::sync::Mutex<Vec<DiscoveredPeer>>,
    http_client: reqwest::blocking::Client,
    local_port: u16,
    discovery_mode: std::sync::Mutex<DiscoveryMode>,
    manual_peers: std::sync::Mutex<Vec<SocketAddr>>,
    #[allow(dead_code)]
    bound_interface: std::sync::Mutex<Option<String>>,
}

impl LanBackend {
    pub fn new(service_name: String, device_signing_key: Vec<u8>) -> Self {
        let http_client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_default();

        Self {
            service_name,
            device_signing_key,
            peers: std::sync::Mutex::new(Vec::new()),
            http_client,
            local_port: DEFAULT_HTTP_PORT,
            discovery_mode: std::sync::Mutex::new(DiscoveryMode::MdnsMulticast),
            manual_peers: std::sync::Mutex::new(Vec::new()),
            bound_interface: std::sync::Mutex::new(None),
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.local_port = port;
        self
    }

    pub fn with_discovery_mode(self, mode: DiscoveryMode) -> Self {
        *self.discovery_mode.lock().unwrap() = mode;
        self
    }

    pub fn with_manual_peers(self, peers: Vec<SocketAddr>) -> Self {
        *self.manual_peers.lock().unwrap() = peers;
        self
    }

    /// Detect all VPN connections on the system.
    pub fn detect_vpns() -> Vec<VpnStatus> {
        let mut vpns = Vec::new();

        // Check Tailscale
        if let Some(ts) = detect_tailscale() {
            vpns.push(ts);
        }

        // Check WireGuard
        if let Some(wg) = detect_wireguard() {
            vpns.push(wg);
        }

        // Check OpenVPN
        if let Some(ovpn) = detect_openvpn() {
            vpns.push(ovpn);
        }

        vpns
    }

    /// Detect available network interfaces.
    pub fn detect_interfaces() -> Vec<NetworkInterface> {
        let mut interfaces = Vec::new();

        // Try to enumerate interfaces via /proc/net/if_inet6 and /sys
        if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_up = std::fs::read_to_string(format!("/sys/class/net/{}/operstate", name))
                    .map(|s| s.trim() == "up")
                    .unwrap_or(false);

                // Try to get IPv4 address
                if let Ok(addrs) = get_interface_addrs(&name) {
                    for ip in addrs {
                        let iface_type = classify_interface(&name);
                        interfaces.push(NetworkInterface {
                            name: name.clone(),
                            ip,
                            interface_type: iface_type,
                            is_up,
                        });
                    }
                }
            }
        }

        // If /sys not available, try default interface
        if interfaces.is_empty() {
            if let Ok(default) = get_default_interface() {
                interfaces.push(default);
            }
        }

        interfaces
    }

    /// Auto-detect the best discovery mode based on available VPNs.
    pub fn auto_detect_discovery_mode() -> DiscoveryMode {
        let vpns = Self::detect_vpns();

        for vpn in &vpns {
            if vpn.connected && vpn.provider == "tailscale" {
                log::info!("Detected Tailscale VPN — using Tailscale discovery");
                return DiscoveryMode::Tailscale;
            }
            if vpn.connected && vpn.provider == "wireguard" {
                log::info!("Detected WireGuard VPN — using WireGuard discovery");
                return DiscoveryMode::WireGuard;
            }
        }

        // Check if we have a Tailscale interface
        let interfaces = Self::detect_interfaces();
        for iface in &interfaces {
            if matches!(iface.interface_type, InterfaceType::Tailscale) {
                log::info!(
                    "Found Tailscale interface {} — using Tailscale discovery",
                    iface.name
                );
                return DiscoveryMode::Tailscale;
            }
        }

        log::info!("No VPN detected — using standard mDNS discovery");
        DiscoveryMode::MdnsMulticast
    }

    /// Discover peers based on the current discovery mode.
    pub fn discover_peers(&self) -> Result<Vec<DiscoveredPeer>, String> {
        let mode = self.discovery_mode.lock().unwrap().clone();

        let mut peers = match &mode {
            DiscoveryMode::MdnsMulticast => self.discover_via_mdns()?,
            DiscoveryMode::Tailscale => self.discover_via_tailscale()?,
            DiscoveryMode::WireGuard => self.discover_via_wireguard()?,
            DiscoveryMode::Manual => self.discover_via_manual()?,
        };

        // Verify each discovered peer
        for peer in &mut peers {
            peer.verified = self.verify_peer(peer).is_ok();
            if peer.verified {
                log::info!("Verified peer {} at {}", peer.name, peer.addr);
            }
        }

        let verified_count = peers.iter().filter(|p| p.verified).count();
        log::info!(
            "Discovery ({:?}): found {} peers, {} verified",
            mode,
            peers.len(),
            verified_count
        );

        *self.peers.lock().unwrap() = peers.clone();
        Ok(peers)
    }

    /// Discover peers via standard mDNS multicast.
    fn discover_via_mdns(&self) -> Result<Vec<DiscoveredPeer>, String> {
        let query = build_mdns_query(MDNS_SERVICE);

        let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("mDNS bind: {}", e))?;
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .map_err(|e| format!("mDNS timeout: {}", e))?;

        let multicast_addr = "224.0.0.251:5353";
        socket
            .send_to(&query, multicast_addr)
            .map_err(|e| format!("mDNS send: {}", e))?;

        let mut peers = Vec::new();
        let mut buf = [0u8; 4096];

        while let Ok((len, from)) = socket.recv_from(&mut buf) {
            if let Some(peer) = parse_mdns_response(&buf[..len], from) {
                if !peers.iter().any(|p: &DiscoveredPeer| p.addr == peer.addr) {
                    peers.push(peer);
                }
            }
        }

        Ok(peers)
    }

    /// Discover peers via Tailscale CLI.
    fn discover_via_tailscale(&self) -> Result<Vec<DiscoveredPeer>, String> {
        let output = std::process::Command::new("tailscale")
            .args(["status", "--json"])
            .output()
            .map_err(|e| format!("tailscale command failed: {} — is Tailscale installed?", e))?;

        if !output.status.success() {
            return Err(format!(
                "tailscale status failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let status: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("tailscale JSON parse: {}", e))?;

        let mut peers = Vec::new();

        // Parse Tailscale status — peers are in the "Peer" map
        if let Some(peers_map) = status.get("Peer") {
            if let Some(obj) = peers_map.as_object() {
                for (ts_pubkey, peer_info) in obj {
                    let online = peer_info
                        .get("Online")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if !online {
                        continue;
                    }

                    let tailscale_ip = peer_info
                        .get("TailscaleIPs")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|v| v.as_str())
                        .and_then(|ip| ip.parse::<Ipv4Addr>().ok());

                    let hostname = peer_info
                        .get("HostName")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    if let Some(ip) = tailscale_ip {
                        peers.push(DiscoveredPeer {
                            name: format!("{} ({})", hostname, ts_pubkey),
                            addr: SocketAddr::new(IpAddr::V4(ip), self.local_port),
                            public_key: ts_pubkey.as_bytes().to_vec(),
                            port: self.local_port,
                            verified: false,
                            interface: Some("tailscale".to_string()),
                        });
                    }
                }
            }
        }

        Ok(peers)
    }

    /// Discover peers via WireGuard CLI.
    fn discover_via_wireguard(&self) -> Result<Vec<DiscoveredPeer>, String> {
        // WireGuard doesn't have a standard CLI for peer enumeration
        // Use `wg show` on Linux
        let output = std::process::Command::new("wg")
            .args(["show"])
            .output()
            .map_err(|e| format!("wg command failed: {} — is WireGuard installed?", e))?;

        if !output.status.success() {
            return Err(format!(
                "wg show failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut peers = Vec::new();
        let mut current_pubkey = String::new();

        for line in stdout.lines() {
            let line = line.trim();
            if let Some(pubkey) = line.strip_prefix("peer: ") {
                current_pubkey = pubkey.to_string();
            } else if let Some(endpoint) = line.strip_prefix("endpoint: ") {
                if let Ok(addr) = endpoint.parse::<SocketAddr>() {
                    peers.push(DiscoveredPeer {
                        name: format!("wg-{}", &current_pubkey[..8.min(current_pubkey.len())]),
                        addr,
                        public_key: current_pubkey.as_bytes().to_vec(),
                        port: addr.port(),
                        verified: false,
                        interface: Some("wireguard".to_string()),
                    });
                }
            }
        }

        Ok(peers)
    }

    /// Use manually configured peers.
    fn discover_via_manual(&self) -> Result<Vec<DiscoveredPeer>, String> {
        let addrs = self.manual_peers.lock().unwrap().clone();
        let peers = addrs
            .into_iter()
            .enumerate()
            .map(|(i, addr)| DiscoveredPeer {
                name: format!("manual-{}", i),
                addr,
                public_key: Vec::new(),
                port: addr.port(),
                verified: false,
                interface: Some("manual".to_string()),
            })
            .collect();
        Ok(peers)
    }

    /// Verify a peer's identity via challenge-response.
    fn verify_peer(&self, peer: &DiscoveredPeer) -> Result<(), String> {
        if peer.public_key.is_empty() {
            // Manual peers skip verification
            if peer.interface.as_deref() == Some("manual") {
                return Ok(());
            }
            return Err("peer has no public key".into());
        }

        let mut nonce = [0u8; 32];
        getrandom::getrandom(&mut nonce).map_err(|e| format!("failed to generate nonce: {}", e))?;
        let nonce_hex = hex::encode(nonce);

        let url = format!("https://{}/challenge", peer.addr);
        let challenge_body = serde_json::json!({
            "nonce": nonce_hex,
            "service": "cybermanju-drive",
        });

        let resp = self
            .http_client
            .post(&url)
            .json(&challenge_body)
            .timeout(Duration::from_secs(CHALLENGE_TIMEOUT))
            .send()
            .map_err(|e| format!("challenge send: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("challenge failed: {}", resp.status()));
        }

        let body: serde_json::Value = resp.json().map_err(|e| format!("challenge parse: {}", e))?;

        let signature = body
            .get("signature")
            .and_then(|v| v.as_str())
            .ok_or("no signature in challenge response")?;

        let peer_pubkey = body
            .get("pubkey")
            .and_then(|v| v.as_str())
            .ok_or("no pubkey in challenge response")?;

        // Verify pubkey matches
        let expected_pubkey_hex = hex::encode(blake3::hash(&peer.public_key).as_bytes());
        let actual_pubkey_hex = hex::encode(blake3::hash(peer_pubkey.as_bytes()).as_bytes());
        if expected_pubkey_hex != actual_pubkey_hex {
            return Err("peer pubkey mismatch".into());
        }

        // Verify signature
        let mut hasher = Sha256::new();
        hasher.update(nonce);
        hasher.update(peer_pubkey.as_bytes());
        let expected_sig = hex::encode(hasher.finalize());

        if signature != expected_sig {
            return Err("challenge signature invalid".into());
        }

        Ok(())
    }

    /// Get list of known peers.
    pub fn get_peers(&self) -> Vec<DiscoveredPeer> {
        self.peers.lock().unwrap().clone()
    }

    /// Get only verified peers.
    pub fn get_verified_peers(&self) -> Vec<DiscoveredPeer> {
        self.peers
            .lock()
            .unwrap()
            .iter()
            .filter(|p| p.verified)
            .cloned()
            .collect()
    }

    /// Send a file to a verified peer via HTTPS POST.
    fn send_to_peer(
        &self,
        peer: &DiscoveredPeer,
        data: &[u8],
        filename: &str,
    ) -> Result<String, String> {
        if !peer.verified && peer.interface.as_deref() != Some("manual") {
            return Err(format!("peer {} is not verified", peer.name));
        }

        let url = format!("https://{}/upload?name={}", peer.addr, filename);

        let resp = self
            .http_client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .body(data.to_vec())
            .send()
            .map_err(|e| format!("peer upload: {}", e))?;

        if resp.status().is_success() {
            let body: serde_json::Value = resp.json().unwrap_or(serde_json::json!({"ok": true}));
            let path = body
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or(filename);
            Ok(format!("lan://{}/{}", peer.addr, path))
        } else {
            Err(format!("peer upload failed: {}", resp.status()))
        }
    }

    /// Download a file from a peer via HTTPS GET.
    fn fetch_from_peer(
        &self,
        peer_addr: &SocketAddr,
        remote_path: &str,
    ) -> Result<Vec<u8>, String> {
        let url = format!("https://{}/files/{}", peer_addr, remote_path);
        let resp = self
            .http_client
            .get(&url)
            .send()
            .map_err(|e| format!("peer download: {}", e))?;

        if resp.status().is_success() {
            resp.bytes()
                .map(|b| b.to_vec())
                .map_err(|e| format!("peer read: {}", e))
        } else {
            Err(format!("peer download failed: {}", resp.status()))
        }
    }

    /// Publish the local device via mDNS.
    pub fn advertise_service(&self) -> Result<(), String> {
        let response = build_mdns_response(
            &self.service_name,
            self.local_port,
            &self.device_signing_key,
        );

        let socket =
            UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("mDNS advertise bind: {}", e))?;
        let multicast_addr = "224.0.0.251:5353";
        socket
            .send_to(&response, multicast_addr)
            .map_err(|e| format!("mDNS advertise send: {}", e))?;

        log::info!(
            "Advertised {} on port {} via mDNS",
            self.service_name,
            self.local_port
        );
        Ok(())
    }

    /// Respond to a peer's challenge (called by the HTTP server).
    pub fn respond_to_challenge(&self, nonce_hex: &str) -> Result<serde_json::Value, String> {
        let nonce = hex::decode(nonce_hex).map_err(|e| format!("invalid nonce hex: {}", e))?;
        if nonce.len() != 32 {
            return Err("nonce must be 32 bytes".into());
        }

        let pubkey_hex = hex::encode(blake3::hash(&self.device_signing_key).as_bytes());

        let mut hasher = Sha256::new();
        hasher.update(&nonce);
        hasher.update(pubkey_hex.as_bytes());
        let sig = hex::encode(hasher.finalize());

        Ok(serde_json::json!({
            "signature": sig,
            "pubkey": pubkey_hex,
            "service": "cybermanju-drive",
        }))
    }
}

// ---- VPN Detection Helpers ----

/// Detect Tailscale VPN status.
fn detect_tailscale() -> Option<VpnStatus> {
    let output = std::process::Command::new("tailscale")
        .args(["status", "--json"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let status: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let self_info = status.get("Self")?;
    let online = self_info
        .get("Online")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let tailscale_ip = self_info
        .get("TailscaleIPs")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .and_then(|ip| ip.parse::<Ipv4Addr>().ok());

    let interface = self_info
        .get("InterfaceName")
        .and_then(|v| v.as_str())
        .map(String::from);

    Some(VpnStatus {
        connected: online,
        provider: "tailscale".to_string(),
        interface,
        local_ip: tailscale_ip,
    })
}

/// Detect WireGuard VPN status.
fn detect_wireguard() -> Option<VpnStatus> {
    let output = std::process::Command::new("wg")
        .args(["show"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let has_interface = stdout.contains("interface:");
    let has_peer = stdout.contains("peer:");

    if !has_interface {
        return None;
    }

    // Try to extract the listening port as proof it's active
    let interface_name = stdout
        .lines()
        .find_map(|l| l.trim().strip_prefix("interface: "))
        .unwrap_or("wg0")
        .to_string();

    Some(VpnStatus {
        connected: has_peer,
        provider: "wireguard".to_string(),
        interface: Some(interface_name),
        local_ip: None,
    })
}

/// Detect OpenVPN status.
fn detect_openvpn() -> Option<VpnStatus> {
    // Check for common OpenVPN management interface or tun devices
    let output = std::process::Command::new("ip")
        .args(["addr", "show", "type", "tun"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains("tun") {
        return None;
    }

    // Extract IP from tun interface
    let ip = stdout
        .lines()
        .filter(|l| l.contains("inet "))
        .filter_map(|l| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            parts.iter().position(|p| *p == "inet").and_then(|i| {
                parts
                    .get(i + 1)?
                    .split('/')
                    .next()?
                    .parse::<Ipv4Addr>()
                    .ok()
            })
        })
        .next();

    Some(VpnStatus {
        connected: true,
        provider: "openvpn".to_string(),
        interface: Some("tun0".to_string()),
        local_ip: ip,
    })
}

/// Classify a network interface by name.
fn classify_interface(name: &str) -> InterfaceType {
    if name.starts_with("tailscale") || name == "ts0" {
        InterfaceType::Tailscale
    } else if name.starts_with("wg") || name.starts_with("wireguard") {
        InterfaceType::WireGuard
    } else if name.starts_with("tun") || name.starts_with("tap") {
        InterfaceType::OpenVpn
    } else if name.starts_with("eth")
        || name.starts_with("wlan")
        || name.starts_with("en")
        || name.starts_with("wl")
    {
        InterfaceType::Lan
    } else {
        InterfaceType::Unknown
    }
}

/// Get IPv4 addresses for a network interface from /sys.
fn get_interface_addrs(name: &str) -> Result<Vec<Ipv4Addr>, String> {
    let addr_file = format!("/sys/class/net/{}/address", name);
    let _ = std::fs::read_to_string(&addr_file); // just checking it exists

    // Try reading from /proc/net/fib_trie or use getifaddrs equivalent
    // Simplified: parse `ip addr show` output
    let output = std::process::Command::new("ip")
        .args(["addr", "show", name])
        .output()
        .map_err(|e| format!("ip addr failed: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let addrs: Vec<Ipv4Addr> = stdout
        .lines()
        .filter(|l| l.contains("inet "))
        .filter_map(|l| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            parts.iter().position(|p| *p == "inet").and_then(|i| {
                parts
                    .get(i + 1)?
                    .split('/')
                    .next()?
                    .parse::<Ipv4Addr>()
                    .ok()
            })
        })
        .collect();

    Ok(addrs)
}

/// Get the default network interface.
fn get_default_interface() -> Result<NetworkInterface, String> {
    let output = std::process::Command::new("ip")
        .args(["route", "show", "default"])
        .output()
        .map_err(|e| format!("ip route failed: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let name = stdout
        .split_whitespace()
        .nth(4)
        .unwrap_or("eth0")
        .to_string();

    let addrs = get_interface_addrs(&name).unwrap_or_default();
    let ip = addrs.first().copied().unwrap_or(Ipv4Addr::new(0, 0, 0, 0));

    Ok(NetworkInterface {
        name,
        ip,
        interface_type: InterfaceType::Unknown,
        is_up: true,
    })
}

// ---- DNS Wire Format ----

fn build_mdns_query(service: &str) -> Vec<u8> {
    let mut packet = Vec::new();
    packet.extend_from_slice(&[0x00, 0x00]); // ID
    packet.extend_from_slice(&[0x00, 0x00]); // Flags
    packet.extend_from_slice(&[0x00, 0x01]); // QDCOUNT=1
    packet.extend_from_slice(&[0x00, 0x00]); // ANCOUNT
    packet.extend_from_slice(&[0x00, 0x00]); // NSCOUNT
    packet.extend_from_slice(&[0x00, 0x00]); // ARCOUNT
    encode_dns_name(&mut packet, service);
    packet.extend_from_slice(&DNS_TYPE_PTR.to_be_bytes());
    packet.extend_from_slice(&DNS_CLASS_IN.to_be_bytes());
    packet
}

fn encode_dns_name(packet: &mut Vec<u8>, name: &str) {
    for label in name.split('.') {
        if label.is_empty() {
            continue;
        }
        packet.push(label.len() as u8);
        packet.extend_from_slice(label.as_bytes());
    }
    packet.push(0x00);
}

fn decode_dns_name(data: &[u8], offset: usize) -> Result<(String, usize), String> {
    let mut name = String::new();
    let mut pos = offset;
    let mut jumped = false;
    let mut jump_pos = 0usize;

    loop {
        if pos >= data.len() {
            return Err("DNS name overflow".into());
        }
        let len = data[pos] as usize;
        if len == 0 {
            return if !jumped {
                Ok((name, pos + 1))
            } else {
                Ok((name, jump_pos))
            };
        }
        if (len & 0xC0) == 0xC0 {
            if pos + 1 >= data.len() {
                return Err("truncated compression pointer".into());
            }
            let pointer = ((len & 0x3F) << 8) as usize | data[pos + 1] as usize;
            if pointer >= data.len() {
                return Err("invalid compression pointer".into());
            }
            if !jumped {
                jump_pos = pos + 2;
                jumped = true;
            }
            pos = pointer;
            continue;
        }
        if pos + 1 + len > data.len() {
            return Err("DNS label overflow".into());
        }
        if !name.is_empty() {
            name.push('.');
        }
        name.push_str(
            std::str::from_utf8(&data[pos + 1..pos + 1 + len])
                .map_err(|_| "invalid UTF-8 in DNS name")?,
        );
        pos += 1 + len;
    }
}

fn parse_mdns_response(data: &[u8], from: SocketAddr) -> Option<DiscoveredPeer> {
    if data.len() < 12 {
        return None;
    }

    let flags = u16::from_be_bytes([data[2], data[3]]);
    if flags & 0x8000 == 0 {
        return None;
    }

    let ancount = u16::from_be_bytes([data[6], data[7]]) as usize;
    if ancount == 0 {
        return None;
    }

    let qdcount = u16::from_be_bytes([data[4], data[5]]) as usize;
    let mut pos = 12;

    for _ in 0..qdcount {
        match decode_dns_name(data, pos) {
            Ok((_, next)) => pos = next + 4,
            Err(_) => return None,
        }
    }

    let mut service_name = String::new();
    let mut port = DEFAULT_HTTP_PORT;
    let mut public_key = Vec::new();
    let mut found_ptr = false;

    for _ in 0..ancount {
        if pos >= data.len() {
            break;
        }
        let name_end = match decode_dns_name(data, pos) {
            Ok((_, end)) => end,
            Err(_) => break,
        };
        pos = name_end;
        if pos + 10 > data.len() {
            break;
        }

        let rtype = u16::from_be_bytes([data[pos], data[pos + 1]]);
        let rclass = u16::from_be_bytes([data[pos + 2], data[pos + 3]]);
        let rdlength = u16::from_be_bytes([data[pos + 8], data[pos + 9]]) as usize;
        pos += 10;

        if rclass & 0x7FFF != DNS_CLASS_IN {
            pos += rdlength;
            continue;
        }
        if pos + rdlength > data.len() {
            break;
        }

        match rtype {
            DNS_TYPE_PTR => {
                if let Ok((name, _)) = decode_dns_name(data, pos) {
                    service_name = name;
                    found_ptr = true;
                }
            }
            DNS_TYPE_SRV => {
                if rdlength >= 6 {
                    port = u16::from_be_bytes([data[pos + 4], data[pos + 5]]);
                }
            }
            DNS_TYPE_TXT => {
                let mut txt_pos = pos;
                let txt_end = pos + rdlength;
                while txt_pos < txt_end {
                    let txt_len = data[txt_pos] as usize;
                    txt_pos += 1;
                    if txt_pos + txt_len > txt_end {
                        break;
                    }
                    if let Ok(txt) = std::str::from_utf8(&data[txt_pos..txt_pos + txt_len]) {
                        if let Some(key) = txt.strip_prefix("key=") {
                            if let Ok(k) = hex::decode(key) {
                                public_key = k;
                            }
                        }
                    }
                    txt_pos += txt_len;
                }
            }
            DNS_TYPE_A if rdlength == 4 => {
                    let _ip = Ipv4Addr::new(data[pos], data[pos + 1], data[pos + 2], data[pos + 3]);
            }
            _ => {}
        }
        pos += rdlength;
    }

    if !found_ptr && service_name.is_empty() {
        return None;
    }

    Some(DiscoveredPeer {
        name: service_name,
        addr: from,
        public_key,
        port,
        verified: false,
        interface: None,
    })
}

fn build_mdns_response(service_name: &str, port: u16, public_key: &[u8]) -> Vec<u8> {
    let mut packet = Vec::new();
    packet.extend_from_slice(&[0x00, 0x00]); // ID
    packet.extend_from_slice(&[0x84, 0x00]); // Flags
    packet.extend_from_slice(&[0x00, 0x00]); // QDCOUNT
    packet.extend_from_slice(&[0x00, 0x03]); // ANCOUNT=3
    packet.extend_from_slice(&[0x00, 0x00]); // NSCOUNT
    packet.extend_from_slice(&[0x00, 0x00]); // ARCOUNT

    let instance = format!("{}._cybermanju._tcp.local.", service_name);

    // PTR
    encode_dns_name(&mut packet, MDNS_SERVICE);
    packet.extend_from_slice(&DNS_TYPE_PTR.to_be_bytes());
    packet.extend_from_slice(&DNS_CLASS_IN.to_be_bytes());
    packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x0C]);
    let ptr_offset = packet.len();
    packet.extend_from_slice(&[0x00, 0x00]);
    encode_dns_name(&mut packet, &instance);
    let ptr_len = packet.len() - ptr_offset - 2;
    packet[ptr_offset..ptr_offset + 2].copy_from_slice(&(ptr_len as u16).to_be_bytes());

    // SRV
    encode_dns_name(&mut packet, &instance);
    packet.extend_from_slice(&DNS_TYPE_SRV.to_be_bytes());
    packet.extend_from_slice(&DNS_CLASS_IN.to_be_bytes());
    packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x0C]);
    let mut srv_rdata = Vec::new();
    srv_rdata.extend_from_slice(&[0x00, 0x00]); // Priority
    srv_rdata.extend_from_slice(&[0x00, 0x00]); // Weight
    srv_rdata.extend_from_slice(&port.to_be_bytes());
    encode_dns_name(&mut srv_rdata, &instance);
    packet.extend_from_slice(&(srv_rdata.len() as u16).to_be_bytes());
    packet.extend_from_slice(&srv_rdata);

    // TXT
    encode_dns_name(&mut packet, &instance);
    packet.extend_from_slice(&DNS_TYPE_TXT.to_be_bytes());
    packet.extend_from_slice(&DNS_CLASS_IN.to_be_bytes());
    packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x0C]);
    let key_hex = hex::encode(blake3::hash(public_key).as_bytes());
    let txt_entry = format!("key={}", key_hex);
    packet.extend_from_slice(&((1 + txt_entry.len()) as u16).to_be_bytes());
    packet.push(txt_entry.len() as u8);
    packet.extend_from_slice(txt_entry.as_bytes());

    packet
}

impl StorageBackend for LanBackend {
    fn name(&self) -> &str {
        "lan"
    }

    fn backend_type(&self) -> SyncBackendType {
        SyncBackendType::Lan
    }

    fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let data = std::fs::read(local_path).map_err(|e| format!("read {}: {}", local_path, e))?;
        if data.is_empty() {
            return Err("cannot upload empty file".into());
        }

        let peers = {
            let peers = self.peers.lock().unwrap();
            if peers.is_empty() {
                drop(peers);
                self.discover_peers()?
            } else {
                peers.clone()
            }
        };

        let verified_peers: Vec<_> = peers
            .iter()
            .filter(|p| p.verified || p.interface.as_deref() == Some("manual"))
            .collect();
        if verified_peers.is_empty() {
            return Err("no verified peers found".into());
        }

        let filename = std::path::Path::new(remote_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");

        self.send_to_peer(verified_peers[0], &data, filename)
    }

    fn download_file(&self, remote_path: &str, local_path: &str) -> Result<(), String> {
        let path = remote_path.strip_prefix("lan://").unwrap_or(remote_path);
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        let peer_addr: SocketAddr = parts[0]
            .parse()
            .map_err(|e| format!("invalid peer address: {}", e))?;

        if !validate_peer_addr(&peer_addr) {
            return Err(format!("peer address not on LAN/VPN: {}", peer_addr));
        }

        let file_path = sanitize_peer_path(parts.get(1).unwrap_or(&"file"))?;
        let data = self.fetch_from_peer(&peer_addr, &file_path)?;
        std::fs::write(local_path, &data).map_err(|e| format!("write {}: {}", local_path, e))?;
        Ok(())
    }

    fn delete_file(&self, remote_path: &str) -> Result<(), String> {
        let path = remote_path.strip_prefix("lan://").unwrap_or(remote_path);
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        let peer_addr: SocketAddr = parts[0]
            .parse()
            .map_err(|e| format!("invalid peer address: {}", e))?;

        if !validate_peer_addr(&peer_addr) {
            return Err(format!("peer address not on LAN/VPN: {}", peer_addr));
        }

        let file_path = sanitize_peer_path(parts.get(1).unwrap_or(&"file"))?;
        let url = format!("https://{}/delete/{}", peer_addr, file_path);
        self.http_client
            .delete(&url)
            .send()
            .map_err(|e| format!("peer delete: {}", e))?;
        Ok(())
    }

    fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFile>, String> {
        let peers = self.get_verified_peers();
        if peers.is_empty() {
            return Ok(Vec::new());
        }

        let peer = &peers[0];
        let url = format!("https://{}/files?prefix={}", peer.addr, prefix);
        let resp = self
            .http_client
            .get(&url)
            .send()
            .map_err(|e| format!("peer list: {}", e))?;

        if resp.status().is_success() {
            let body: serde_json::Value =
                resp.json().map_err(|e| format!("peer list parse: {}", e))?;
            let files = body
                .get("files")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            let name = item.get("name").and_then(|v| v.as_str())?;
                            let size = item.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                            Some(RemoteFile {
                                name: name.to_string(),
                                path: format!("lan://{}/{}", peer.addr, name),
                                size_bytes: size,
                                modified_at: String::new(),
                                url: format!("https://{}/files/{}", peer.addr, name),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            Ok(files)
        } else {
            Ok(Vec::new())
        }
    }

    fn get_file_url(&self, remote_path: &str) -> Result<String, String> {
        Ok(format!("lan://{}", remote_path))
    }

    fn test_connection(&self) -> Result<bool, String> {
        let peers = self.discover_peers()?;
        let verified = peers
            .iter()
            .filter(|p| p.verified || p.interface.as_deref() == Some("manual"))
            .count();
        if verified == 0 {
            log::warn!("No verified peers found");
            Ok(false)
        } else {
            log::info!("Found {} verified peers", verified);
            Ok(true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_peer_path() {
        assert!(sanitize_peer_path("file.txt").is_ok());
        assert!(sanitize_peer_path("dir/file.txt").is_ok());
        assert!(sanitize_peer_path("../etc/passwd").is_err());
        assert!(sanitize_peer_path("/etc/passwd").is_err());
        assert!(sanitize_peer_path("").is_err());
    }

    #[test]
    fn test_validate_peer_addr() {
        let private: SocketAddr = "192.168.1.1:8080".parse().unwrap();
        assert!(validate_peer_addr(&private));

        let tailscale: SocketAddr = "100.64.0.1:8080".parse().unwrap();
        assert!(validate_peer_addr(&tailscale));

        let wireguard: SocketAddr = "10.0.0.1:8080".parse().unwrap();
        assert!(validate_peer_addr(&wireguard));

        let loopback: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        assert!(validate_peer_addr(&loopback));

        let public: SocketAddr = "8.8.8.8:8080".parse().unwrap();
        assert!(!validate_peer_addr(&public));
    }

    #[test]
    fn test_is_vpn_ip() {
        assert!(is_vpn_ip(&Ipv4Addr::new(100, 64, 0, 1))); // Tailscale
        assert!(is_vpn_ip(&Ipv4Addr::new(100, 127, 0, 1))); // Tailscale
        assert!(is_vpn_ip(&Ipv4Addr::new(10, 0, 0, 1))); // WireGuard/OpenVPN
        assert!(is_vpn_ip(&Ipv4Addr::new(172, 16, 0, 1))); // WireGuard/OpenVPN
        assert!(!is_vpn_ip(&Ipv4Addr::new(192, 168, 1, 1))); // LAN
        assert!(!is_vpn_ip(&Ipv4Addr::new(8, 8, 8, 8))); // Public
    }

    #[test]
    fn test_dns_name_encode_decode() {
        let mut packet = Vec::new();
        encode_dns_name(&mut packet, "example.com");
        let (name, end) = decode_dns_name(&packet, 0).unwrap();
        assert_eq!(name, "example.com");
        assert_eq!(end, packet.len());
    }

    #[test]
    fn test_classify_interface() {
        assert!(matches!(
            classify_interface("tailscale0"),
            InterfaceType::Tailscale
        ));
        assert!(matches!(
            classify_interface("wg0"),
            InterfaceType::WireGuard
        ));
        assert!(matches!(classify_interface("tun0"), InterfaceType::OpenVpn));
        assert!(matches!(classify_interface("eth0"), InterfaceType::Lan));
        assert!(matches!(classify_interface("wlan0"), InterfaceType::Lan));
    }
}

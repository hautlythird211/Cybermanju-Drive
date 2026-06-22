use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SYNCTHING_IPS_URL: &str =
    "https://raw.githubusercontent.com/elliotwutingfeng/SyncthingRelayServerIPs/refs/heads/main/ips.txt";

/// Relay IP list with hourly refresh.
#[derive(Debug, Clone)]
pub struct RelayIpList {
    inner: Arc<Mutex<RelayIpListInner>>,
}

#[derive(Debug)]
struct RelayIpListInner {
    ips: HashSet<String>,
    last_fetch_secs: u64,
    fetch_interval_secs: u64,
}

impl RelayIpList {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RelayIpListInner {
                ips: HashSet::new(),
                last_fetch_secs: 0,
                fetch_interval_secs: 3600, // 1 hour
            })),
        }
    }

    /// Fetch the latest relay IPs if stale (>1 hour old).
    pub fn refresh_if_stale(&self) {
        let now = now_secs();
        let should_fetch = {
            let inner = self.inner.lock().unwrap();
            inner.ips.is_empty() || (now - inner.last_fetch_secs) >= inner.fetch_interval_secs
        };

        if should_fetch {
            if let Ok(ips) = fetch_syncthing_relay_ips() {
                let mut inner = self.inner.lock().unwrap();
                inner.ips = ips;
                inner.last_fetch_secs = now;
                log::info!("Refreshed relay IP list: {} IPs", inner.ips.len());
            }
        }
    }

    /// Get a snapshot of current relay IPs.
    pub fn ips(&self) -> Vec<String> {
        self.inner.lock().unwrap().ips.iter().cloned().collect()
    }

    /// Get IPs formatted as WebSocket relay URLs for Nostr relays.
    pub fn nostr_relay_urls(&self) -> Vec<String> {
        self.ips()
            .iter()
            .map(|ip| format!("wss://{}:443", ip))
            .collect()
    }

    /// Get IPs formatted as Syncthing relay addresses (tcp://ip:22000).
    pub fn syncthing_relay_addrs(&self) -> Vec<String> {
        self.ips()
            .iter()
            .map(|ip| format!("tcp://{}:22000", ip))
            .collect()
    }

    /// Get the count of known relay IPs.
    pub fn count(&self) -> usize {
        self.inner.lock().unwrap().ips.len()
    }
}

impl Default for RelayIpList {
    fn default() -> Self {
        Self::new()
    }
}

/// Fetch Syncthing relay IPs from the GitHub-hosted list.
/// Filters out private/non-routable IPs to prevent SSRF.
fn fetch_syncthing_relay_ips() -> Result<HashSet<String>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("failed to create HTTP client: {}", e))?;

    let resp = client
        .get(SYNCTHING_IPS_URL)
        .send()
        .map_err(|e| format!("failed to fetch relay IPs: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("relay IP fetch returned status {}", resp.status()));
    }

    let body = resp
        .text()
        .map_err(|e| format!("failed to read relay IP response: {}", e))?;

    let ips: HashSet<String> = body
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter(|l| {
            if let Ok(ip) = l.parse::<std::net::Ipv4Addr>() {
                // Only accept public, routable IPs
                !ip.is_private()
                    && !ip.is_loopback()
                    && !ip.is_link_local()
                    && !ip.is_broadcast()
                    && ip != std::net::Ipv4Addr::new(0, 0, 0, 0)
                    && !ip.is_unspecified()
            } else {
                false
            }
        })
        .map(String::from)
        .collect();

    Ok(ips)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_ip_list_new() {
        let list = RelayIpList::new();
        assert_eq!(list.count(), 0);
    }

    #[test]
    fn test_syncthing_relay_addrs() {
        let list = RelayIpList::new();
        {
            let mut inner = list.inner.lock().unwrap();
            inner.ips.insert("1.2.3.4".to_string());
            inner.ips.insert("5.6.7.8".to_string());
        }
        let addrs = list.syncthing_relay_addrs();
        assert_eq!(addrs.len(), 2);
        assert!(addrs.iter().any(|a| a == "tcp://1.2.3.4:22000"));
    }

    #[test]
    fn test_nostr_relay_urls() {
        let list = RelayIpList::new();
        {
            let mut inner = list.inner.lock().unwrap();
            inner.ips.insert("10.0.0.1".to_string());
        }
        let urls = list.nostr_relay_urls();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "wss://10.0.0.1:443");
    }
}

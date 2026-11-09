//! Fetches of URLs that tenants configure, such as a client's `jwksUri`.
//!
//! The server may run inside a cloud network, next to an instance metadata
//! service (`169.254.169.254`, `metadata.google.internal`), databases and
//! other services that trust callers by their network position. A URL stored
//! on a resource is input the caller controls, so a fetch here:
//!
//! - accepts only an `https` URL with a host name and no credentials;
//! - resolves the host itself and refuses every address that is not publicly
//!   routable: loopback, private, link-local, shared, multicast and reserved
//!   ranges, their IPv6 counterparts and IPv4-mapped forms;
//! - connects to exactly the addresses it checked, so the DNS answer cannot
//!   change between the check and the connection (DNS rebinding);
//! - follows no redirect, which none of the checks above would see;
//! - uses no proxy, which would carry the request somewhere else;
//! - gives up after short timeouts and refuses a body over a size limit.

use reqwest::{Url, header, redirect};
use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    time::Duration,
};
use url::Host;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const TOTAL_TIMEOUT: Duration = Duration::from_secs(10);

/// Why a fetch was refused or failed.
#[derive(Debug)]
pub enum FetchError {
    /// Not an `https` URL with a host name, or one that carries credentials.
    NotPublicHttps,
    /// The host name did not resolve.
    Unresolvable,
    /// The host resolved to an address that is not publicly routable.
    PrivateAddress,
    /// The connection or request failed.
    Request(reqwest::Error),
    /// The server answered with an error status.
    Status(reqwest::StatusCode),
    /// The body is larger than the caller allows.
    TooLarge,
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchError::NotPublicHttps => {
                write!(f, "not an https URL with a host name and no credentials")
            }
            FetchError::Unresolvable => write!(f, "the host name does not resolve"),
            FetchError::PrivateAddress => {
                write!(f, "the host resolves to an address that is not public")
            }
            FetchError::Request(error) => write!(f, "the request failed: {error}"),
            FetchError::Status(status) => write!(f, "the server answered {status}"),
            FetchError::TooLarge => write!(f, "the response body is too large"),
        }
    }
}

/// `url` as an `https` URL with a host name and no credentials; the check a
/// URL can pass before anything is resolved or fetched.
pub fn public_https_url(url: &str) -> Result<Url, FetchError> {
    let url = Url::parse(url).map_err(|_| FetchError::NotPublicHttps)?;

    if url.scheme() != "https"
        || url.host().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(FetchError::NotPublicHttps);
    }

    Ok(url)
}

/// Whether `ip` is routable on the public internet. Everything that is not,
/// from loopback to the reserved and documentation ranges, is refused: an
/// address a deployment could reach inside its own network must never be one
/// a tenant can point the server at.
fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_public_ipv4(ip),
        IpAddr::V6(ip) => is_public_ipv6(ip),
    }
}

fn is_public_ipv4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();

    let special = a == 0 // 0.0.0.0/8, "this network"
        || ip.is_private() // 10/8, 172.16/12, 192.168/16
        || ip.is_loopback() // 127/8
        || ip.is_link_local() // 169.254/16, where cloud metadata services live
        || (a == 100 && (64..=127).contains(&b)) // 100.64/10, shared address space
        || (a == 192 && b == 0 && c == 0) // 192.0.0.0/24, IETF protocol assignments
        || (a == 192 && b == 88 && c == 99) // 192.88.99/24, 6to4 relay anycast
        || ip.is_documentation() // 192.0.2/24, 198.51.100/24, 203.0.113/24
        || (a == 198 && (b == 18 || b == 19)) // 198.18/15, benchmarking
        || ip.is_multicast() // 224/4
        || a >= 240; // 240/4, reserved, and the broadcast address

    !special
}

fn is_public_ipv6(ip: Ipv6Addr) -> bool {
    if let Some(mapped) = ip.to_ipv4_mapped() {
        return is_public_ipv4(mapped);
    }

    let s = ip.segments();

    let special = ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_multicast() // ff00::/8
        || s[..6] == [0, 0, 0, 0, 0, 0] // ::/96, deprecated IPv4-compatible addresses
        || (s[0] == 0x0064 && s[1] == 0xff9b) // 64:ff9b::/96 and 64:ff9b:1::/48, NAT64
        || (s[0] == 0x0100 && s[1..4] == [0, 0, 0]) // 100::/64, discard-only
        || (s[0] & 0xfe00) == 0xfc00 // fc00::/7, unique local
        || (s[0] & 0xffc0) == 0xfe80 // fe80::/10, link-local
        || (s[0] & 0xffc0) == 0xfec0 // fec0::/10, deprecated site-local
        || (s[0] == 0x2001 && s[1] < 0x0200) // 2001::/23, IETF protocol assignments (Teredo, benchmarking, documentation, ORCHID)
        || (s[0] == 0x2001 && s[1] == 0x0db8) // 2001:db8::/32, documentation
        || s[0] == 0x2002 // 2002::/16, 6to4
        || (s[0] == 0x3fff && (s[1] & 0xf000) == 0) // 3fff::/20, documentation
        || s[0] == 0x5f00; // 5f00::/16, segment routing

    !special
}

/// The socket addresses `url`'s host resolves to, when every one of them is
/// public. One private answer among public ones refuses the whole host, so
/// a name cannot slip an internal address in beside a public one.
async fn public_addrs(url: &Url) -> Result<Vec<SocketAddr>, FetchError> {
    let port = url.port_or_known_default().unwrap_or(443);

    let addrs: Vec<SocketAddr> = match url.host().ok_or(FetchError::NotPublicHttps)? {
        Host::Ipv4(ip) => vec![SocketAddr::new(IpAddr::V4(ip), port)],
        Host::Ipv6(ip) => vec![SocketAddr::new(IpAddr::V6(ip), port)],
        Host::Domain(domain) => tokio::net::lookup_host((domain, port))
            .await
            .map_err(|_| FetchError::Unresolvable)?
            .collect(),
    };

    if addrs.is_empty() {
        return Err(FetchError::Unresolvable);
    }

    if addrs.iter().any(|addr| !is_public_ip(addr.ip())) {
        return Err(FetchError::PrivateAddress);
    }

    Ok(addrs)
}

/// GETs `url`, a public `https` URL, and returns its body of at most
/// `max_bytes`. See the module documentation for what is refused.
pub async fn fetch_public_https(url: &str, max_bytes: usize) -> Result<Vec<u8>, FetchError> {
    let url = public_https_url(url)?;
    let addrs = public_addrs(&url).await?;
    let host = url.host_str().ok_or(FetchError::NotPublicHttps)?;

    // Built per fetch: the resolved addresses are pinned for this host only.
    let client = reqwest::Client::builder()
        .https_only(true)
        .redirect(redirect::Policy::none())
        .no_proxy()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(TOTAL_TIMEOUT)
        .resolve_to_addrs(host, &addrs)
        .user_agent(concat!("haste-health/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(FetchError::Request)?;

    let mut response = client
        .get(url)
        .header(header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(FetchError::Request)?;

    if !response.status().is_success() {
        return Err(FetchError::Status(response.status()));
    }

    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return Err(FetchError::TooLarge);
    }

    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(FetchError::Request)? {
        if body.len() + chunk.len() > max_bytes {
            return Err(FetchError::TooLarge);
        }
        body.extend_from_slice(&chunk);
    }

    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn public(ip: &str) -> bool {
        is_public_ip(ip.parse().unwrap())
    }

    #[test]
    fn refuses_addresses_inside_a_network() {
        for ip in [
            "127.0.0.1",
            "10.1.2.3",
            "172.16.0.1",
            "172.31.255.255",
            "192.168.1.1",
            "169.254.169.254",
            "100.64.0.1",
            "100.127.255.255",
            "0.0.0.0",
            "192.0.0.1",
            "192.0.2.1",
            "198.18.0.1",
            "198.51.100.1",
            "203.0.113.1",
            "224.0.0.1",
            "240.0.0.1",
            "255.255.255.255",
            "::",
            "::1",
            "::ffff:127.0.0.1",
            "::ffff:169.254.169.254",
            "::ffff:10.0.0.1",
            "::10.0.0.1",
            "64:ff9b::a9fe:a9fe",
            "64:ff9b:1::1",
            "100::1",
            "fc00::1",
            "fd12:3456::1",
            "fe80::1",
            "fec0::1",
            "2001:db8::1",
            "2001::1",
            "2001:2::1",
            "2002:c000:201::1",
            "3fff::1",
            "5f00::1",
            "ff02::1",
        ] {
            assert!(!public(ip), "{ip} must not be treated as public");
        }
    }

    #[test]
    fn accepts_public_addresses() {
        for ip in [
            "8.8.8.8",
            "1.1.1.1",
            "172.32.0.1",
            "100.128.0.1",
            "192.0.1.1",
            "198.20.0.1",
            "2606:4700:4700::1111",
            "2001:4860:4860::8888",
            "2a00:1450:4001:80b::200e",
            "::ffff:8.8.8.8",
        ] {
            assert!(public(ip), "{ip} must be treated as public");
        }
    }

    #[test]
    fn only_https_without_credentials_passes() {
        assert!(public_https_url("https://keys.example.com/jwks.json").is_ok());
        assert!(public_https_url("https://keys.example.com:8443/jwks.json").is_ok());
        assert!(public_https_url("http://keys.example.com/jwks.json").is_err());
        assert!(public_https_url("https://user:pw@keys.example.com/jwks.json").is_err());
        assert!(public_https_url("https://user@keys.example.com/jwks.json").is_err());
        assert!(public_https_url("file:///etc/passwd").is_err());
        assert!(public_https_url("not a url").is_err());
    }
}

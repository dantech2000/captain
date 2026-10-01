use std::net::{Ipv4Addr, TcpListener};

/// The host port to suggest for `preferred`: the first port from `preferred` up
/// that no container publishes (`published`) and that `free` accepts. Gives
/// `preferred` when the next 100 are all taken.
pub fn suggest_port(preferred: u16, published: &[u16], free: impl Fn(u16) -> bool) -> u16 {
    (preferred..=preferred.saturating_add(100))
        .find(|port| !published.contains(port) && free(*port))
        .unwrap_or(preferred)
}

/// True if nothing on this computer listens on `port`, on all addresses and on
/// the loopback address.
pub fn host_port_free(port: u16) -> bool {
    [Ipv4Addr::UNSPECIFIED, Ipv4Addr::LOCALHOST]
        .into_iter()
        .all(|ip| TcpListener::bind((ip, port)).is_ok())
}

#[cfg(test)]
mod tests;

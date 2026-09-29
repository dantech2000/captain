use std::fmt;

/// A container port and, if it is published, the host side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortMapping {
    pub private_port: u16,
    pub public_port: Option<u16>,
    pub host_ip: Option<String>,
    /// `tcp`, `udp`, or `sctp`.
    pub protocol: String,
}

impl fmt::Display for PortMapping {
    /// Formats like the Docker CLI: `0.0.0.0:8080->80/tcp`, or `80/tcp` if not published.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(public) = self.public_port {
            let ip = self.host_ip.as_deref().unwrap_or("0.0.0.0");
            if ip.contains(':') {
                write!(f, "[{ip}]:{public}->")?;
            } else {
                write!(f, "{ip}:{public}->")?;
            }
        }
        write!(f, "{}/{}", self.private_port, self.protocol)
    }
}

#[cfg(test)]
mod tests;

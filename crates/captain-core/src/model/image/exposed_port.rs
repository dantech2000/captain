use std::fmt;

/// A port that an image declares with `EXPOSE`, for example `80/tcp`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExposedPort {
    pub port: u16,
    /// `tcp`, `udp`, or `sctp`.
    pub protocol: String,
}

impl ExposedPort {
    /// Parses the engine's form, `80/tcp`. A missing protocol means `tcp`.
    /// Returns `None` for a port that is not a number from 1 to 65535.
    pub fn parse(text: &str) -> Option<Self> {
        let (port, protocol) = text.split_once('/').unwrap_or((text, "tcp"));
        let port = port.trim().parse::<u16>().ok().filter(|p| *p > 0)?;
        let protocol = match protocol.trim() {
            "" => "tcp".to_string(),
            other => other.to_ascii_lowercase(),
        };
        Some(Self { port, protocol })
    }
}

impl fmt::Display for ExposedPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.port, self.protocol)
    }
}

#[cfg(test)]
mod tests;

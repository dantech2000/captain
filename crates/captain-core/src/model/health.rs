/// The result of a container's health check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Starting,
    Healthy,
    Unhealthy,
}

impl Health {
    /// Parses the health status from the Docker API. `none` and empty mean no health check.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "starting" => Some(Self::Starting),
            "healthy" => Some(Self::Healthy),
            "unhealthy" => Some(Self::Unhealthy),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Healthy => "healthy",
            Self::Unhealthy => "unhealthy",
        }
    }
}

#[cfg(test)]
mod tests;

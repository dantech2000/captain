/// How bad a vulnerability is, as Trivy rates it. The order is worst first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Unknown,
}

impl Severity {
    pub const ALL: [Severity; 5] = [
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
        Severity::Unknown,
    ];

    /// Reads Trivy's `Severity` field. Anything else is `Unknown`.
    pub fn parse(value: &str) -> Self {
        match value.to_ascii_uppercase().as_str() {
            "CRITICAL" => Self::Critical,
            "HIGH" => Self::High,
            "MEDIUM" => Self::Medium,
            "LOW" => Self::Low,
            _ => Self::Unknown,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
            Self::Unknown => "Unknown",
        }
    }
}

/// One finding: a vulnerability in one installed package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vulnerability {
    /// For example `CVE-2024-1234`.
    pub id: String,
    pub package: String,
    pub installed: String,
    /// The first version with a fix. `None` when there is no fix yet.
    pub fixed: Option<String>,
    pub severity: Severity,
    pub title: Option<String>,
    /// A page about the vulnerability.
    pub url: Option<String>,
}

/// The vulnerabilities of one image, worst first, then by ID.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanReport {
    /// What Trivy scanned, for example `debian:bookworm-slim`.
    pub artifact: String,
    pub vulnerabilities: Vec<Vulnerability>,
}

impl ScanReport {
    /// The number of findings for `severity`.
    pub fn count(&self, severity: Severity) -> usize {
        self.vulnerabilities
            .iter()
            .filter(|vuln| vuln.severity == severity)
            .count()
    }

    /// The findings with `severity`, or all of them for `None`.
    pub fn filtered(&self, severity: Option<Severity>) -> Vec<&Vulnerability> {
        self.vulnerabilities
            .iter()
            .filter(|vuln| severity.is_none_or(|severity| vuln.severity == severity))
            .collect()
    }
}

/// One message from a scan: a status line while Trivy runs, then the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanProgress {
    Status(String),
    Report(ScanReport),
}

#[cfg(test)]
mod tests;

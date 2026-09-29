//! Reads Trivy's JSON report and log lines. See
//! <https://trivy.dev/latest/docs/configuration/reporting/#json>.

use serde::Deserialize;

use super::{ScanReport, Severity, Vulnerability};

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Report {
    #[serde(default)]
    artifact_name: String,
    /// `null` when Trivy finds nothing to scan, for example in `busybox`.
    #[serde(default)]
    results: Option<Vec<Target>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Target {
    /// `null` for a target with no findings.
    #[serde(default)]
    vulnerabilities: Option<Vec<Finding>>,
}

/// `VulnerabilityID`, `PkgName`, `InstalledVersion`, and `Severity` are always set.
/// The rest may be missing.
#[derive(Deserialize)]
struct Finding {
    #[serde(rename = "VulnerabilityID")]
    id: String,
    #[serde(rename = "PkgName")]
    package: String,
    #[serde(rename = "InstalledVersion")]
    installed: String,
    #[serde(rename = "FixedVersion", default)]
    fixed: Option<String>,
    #[serde(rename = "Severity")]
    severity: String,
    #[serde(rename = "Title", default)]
    title: Option<String>,
    #[serde(rename = "PrimaryURL", default)]
    url: Option<String>,
}

/// Parses the output of `trivy image --format json`. The findings of every target
/// go into one list, worst first, then by ID.
pub fn parse_trivy_report(json: &str) -> Result<ScanReport, String> {
    let report: Report = serde_json::from_str(json)
        .map_err(|err| format!("Captain cannot read the Trivy report: {err}"))?;
    let mut vulnerabilities: Vec<Vulnerability> = report
        .results
        .unwrap_or_default()
        .into_iter()
        .flat_map(|target| target.vulnerabilities.unwrap_or_default())
        .map(|finding| Vulnerability {
            id: finding.id,
            package: finding.package,
            installed: finding.installed,
            fixed: finding.fixed.filter(|fixed| !fixed.is_empty()),
            severity: Severity::parse(&finding.severity),
            title: finding.title.filter(|title| !title.is_empty()),
            url: finding.url.filter(|url| !url.is_empty()),
        })
        .collect();
    vulnerabilities.sort_by(|a, b| a.severity.cmp(&b.severity).then_with(|| a.id.cmp(&b.id)));
    Ok(ScanReport {
        artifact: report.artifact_name,
        vulnerabilities,
    })
}

/// The message of one Trivy log line, which looks like
/// `2026-09-29T10:06:05Z<TAB>INFO<TAB>[vulndb] Downloading vulnerability DB...`.
/// `None` for other output, such as the download progress bar.
pub fn trivy_status(line: &str) -> Option<String> {
    let mut fields = line.trim().splitn(3, '\t');
    let (_time, level, message) = (fields.next()?, fields.next()?, fields.next()?);
    if !matches!(level, "INFO" | "WARN" | "ERROR" | "FATAL") {
        return None;
    }
    // Drop the key=value details after the message, which are also tab-separated.
    let message = message.split('\t').next().unwrap_or(message).trim();
    (!message.is_empty()).then(|| message.to_string())
}

#[cfg(test)]
mod tests;

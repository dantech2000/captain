use super::*;

#[test]
fn a_report_is_flattened_and_sorted_worst_first() {
    let json = r#"{
      "SchemaVersion": 2,
      "ArtifactName": "debian:bookworm-slim",
      "Results": [
        {"Target": "debian (debian 12.15)", "Class": "os-pkgs", "Vulnerabilities": [
          {"VulnerabilityID": "CVE-2011-3374", "PkgName": "apt", "InstalledVersion": "2.6.1",
           "Severity": "LOW", "Title": "apt-key does not validate keys",
           "PrimaryURL": "https://avd.aquasec.com/nvd/cve-2011-3374"},
          {"VulnerabilityID": "CVE-2025-0001", "PkgName": "libssl3", "InstalledVersion": "3.0.1",
           "FixedVersion": "3.0.2", "Severity": "CRITICAL"}
        ]},
        {"Target": "app/package-lock.json", "Class": "lang-pkgs", "Vulnerabilities": null}
      ]
    }"#;
    let report = parse_trivy_report(json).unwrap();
    assert_eq!(report.artifact, "debian:bookworm-slim");
    assert_eq!(
        report.vulnerabilities[0],
        Vulnerability {
            id: "CVE-2025-0001".into(),
            package: "libssl3".into(),
            installed: "3.0.1".into(),
            fixed: Some("3.0.2".into()),
            severity: Severity::Critical,
            title: None,
            url: None,
        }
    );
    assert_eq!(report.vulnerabilities[1].severity, Severity::Low);
    assert_eq!(report.vulnerabilities.len(), 2);
}

#[test]
fn null_results_are_an_empty_report() {
    let report = parse_trivy_report(r#"{"ArtifactName": "busybox", "Results": null}"#).unwrap();
    assert!(report.vulnerabilities.is_empty());
    assert!(parse_trivy_report("not json").is_err());
}

#[test]
fn log_lines_give_their_message() {
    assert_eq!(
        trivy_status("2026-09-29T10:06:05Z\tINFO\t[vulndb] Downloading artifact...\trepo=\"x\""),
        Some("[vulndb] Downloading artifact...".into())
    );
    assert_eq!(
        trivy_status("10.12 MiB / 118.00 MiB [----->___] 8.57%"),
        None
    );
}

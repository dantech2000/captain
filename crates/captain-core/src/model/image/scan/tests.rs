use super::*;

fn vuln(id: &str, severity: Severity) -> Vulnerability {
    Vulnerability {
        id: id.into(),
        package: "openssl".into(),
        installed: "3.0.1".into(),
        fixed: None,
        severity,
        title: None,
        url: None,
    }
}

#[test]
fn filter_and_count_by_severity() {
    let report = ScanReport {
        artifact: "app".into(),
        vulnerabilities: vec![
            vuln("CVE-1", Severity::High),
            vuln("CVE-2", Severity::Low),
            vuln("CVE-3", Severity::High),
        ],
    };
    assert_eq!(report.count(Severity::High), 2);
    assert_eq!(report.count(Severity::Critical), 0);
    assert_eq!(report.filtered(None).len(), 3);
    let low: Vec<&str> = report
        .filtered(Some(Severity::Low))
        .iter()
        .map(|vuln| vuln.id.as_str())
        .collect();
    assert_eq!(low, ["CVE-2"]);
}

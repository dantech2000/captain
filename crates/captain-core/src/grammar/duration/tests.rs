use std::time::Duration;

use super::{duration_label, parse_duration};

#[test]
fn parses_each_unit_and_rejects_a_time_without_a_unit_or_number() {
    assert_eq!(parse_duration("30s"), Ok(Duration::from_secs(30)));
    assert_eq!(parse_duration("10m"), Ok(Duration::from_secs(600)));
    assert_eq!(parse_duration("1h"), Ok(Duration::from_secs(3_600)));
    assert_eq!(parse_duration("2d"), Ok(Duration::from_secs(172_800)));
    for text in ["10", "m", "10x", "1h30m", "-5m", ""] {
        let error = parse_duration(text).unwrap_err();
        assert!(error.contains("such as 10m"), "{text}: {error}");
    }
    assert!(parse_duration("0m").is_err());
}

#[test]
fn labels_use_the_largest_whole_unit() {
    assert_eq!(
        duration_label(Duration::from_secs(600)),
        "the last 10 minutes"
    );
    assert_eq!(duration_label(Duration::from_secs(3_600)), "the last hour");
    assert_eq!(
        duration_label(Duration::from_secs(90)),
        "the last 90 seconds"
    );
}

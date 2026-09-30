use super::{FILTERED_TAIL, LogQuery, MAX_TAIL, parse_since};
use crate::model::{LogLine, LogStream};

#[test]
fn reads_ages_unix_times_and_rfc3339() {
    let now = 1_000_000;
    assert_eq!(parse_since("10m", now), Ok(now - 600));
    assert_eq!(parse_since("2h", now), Ok(now - 7200));
    assert_eq!(parse_since("1700000000", now), Ok(1_700_000_000));
    assert_eq!(parse_since("1970-01-02T00:00:00Z", now), Ok(86_400));
    assert!(parse_since("-5m", now).is_err());
    assert!(parse_since("yesterday", now).is_err());
}

#[test]
fn filters_read_more_lines_and_keep_matches() {
    let query = LogQuery {
        errors_only: true,
        grep: Some("DB".into()),
        ..LogQuery::default()
    };
    assert_eq!(query.fetch_tail(), FILTERED_TAIL);
    let line = |text: &str| LogLine::new(LogStream::Stdout, text);
    assert!(query.keeps(&line("ERROR db timeout")));
    assert!(!query.keeps(&line("ERROR cache miss")));
    assert!(!query.keeps(&line("db ready")));
    let huge = LogQuery {
        tail: Some(1_000_000),
        ..LogQuery::default()
    };
    assert_eq!(huge.fetch_tail(), MAX_TAIL);
}

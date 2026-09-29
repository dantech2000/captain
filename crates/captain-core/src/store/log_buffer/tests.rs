// The expected values are byte ranges, not a vector of numbers.
#![allow(clippy::single_range_in_vec_init)]

use super::{LOG_BUFFER_LEN, LevelFilter, LogBuffer};
use crate::model::{LogLevel, LogLine, LogStream};

fn line(text: &str) -> LogLine {
    LogLine::new(LogStream::Stdout, text)
}

#[test]
fn drops_the_oldest_line_when_full() {
    let mut buffer = LogBuffer::default();
    for i in 0..LOG_BUFFER_LEN {
        assert!(buffer.push(line(&i.to_string())).is_none());
    }
    let evicted = buffer.push(line(&LOG_BUFFER_LEN.to_string()));
    assert_eq!(evicted.map(|l| l.text), Some("0".into()));
    assert_eq!(buffer.len(), LOG_BUFFER_LEN);
    assert_eq!(buffer.filtered(LevelFilter::All)[0].text, "1");
}

#[test]
fn filters_by_level() {
    let mut buffer = LogBuffer::default();
    buffer.push(line("GET / 200"));
    buffer.push(line("WARN slow"));
    buffer.push(line("error: boom"));

    assert_eq!(buffer.filtered(LevelFilter::All).len(), 3);
    let errors = buffer.filtered(LevelFilter::Only(LogLevel::Error));
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].text, "error: boom");
}

#[test]
fn search_combines_the_level_filter_and_the_query() {
    let mut buffer = LogBuffer::default();
    buffer.push(line("GET /api 200"));
    buffer.push(line("WARN slow GET /api"));
    buffer.push(line("error: boom"));

    let all = buffer.search(LevelFilter::All, "get /API");
    assert_eq!(all.len(), 2);
    assert_eq!(all[1].ranges, vec![10..18]);

    let warnings = buffer.search(LevelFilter::Only(LogLevel::Warn), "get");
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].line.text, "WARN slow GET /api");

    let everything = buffer.search(LevelFilter::All, "");
    assert_eq!(everything.len(), 3);
    assert!(everything.iter().all(|m| m.ranges.is_empty()));
}

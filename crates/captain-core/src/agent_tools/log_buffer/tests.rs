use super::{KEPT_BYTES, LogBuffer};
use crate::agent_tools::log_query::{LogQuery, MAX_LINE_CHARS};
use crate::model::{LogLine, LogStream};

#[test]
fn keeps_the_newest_matching_lines_within_its_bytes() {
    let query = LogQuery {
        grep: Some("x".into()),
        ..LogQuery::default()
    };
    let mut buffer = LogBuffer::default();
    for _ in 0..200 {
        buffer.push(&query, LogLine::new(LogStream::Stdout, "x".repeat(100_000)));
        buffer.push(&query, LogLine::new(LogStream::Stdout, "no match"));
    }
    assert_eq!(buffer.matched(), 200);
    let lines = buffer.into_lines();
    assert!(!lines.is_empty());
    assert!(lines.iter().map(|l| l.text.len()).sum::<usize>() <= KEPT_BYTES);
    assert!(lines.iter().all(|l| l.text.len() > MAX_LINE_CHARS));
}

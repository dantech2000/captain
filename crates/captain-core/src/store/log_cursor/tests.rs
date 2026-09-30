use super::LogCursor;
use crate::model::{LogLine, LogStream};

fn line(seconds: i64, nanos: u32, text: &str) -> LogLine {
    LogLine {
        timestamp: Some(seconds),
        nanos,
        ..LogLine::new(LogStream::Stdout, text)
    }
}

#[test]
fn a_resumed_stream_drops_replayed_lines_and_keeps_new_ones_of_the_same_second() {
    let mut cursor = LogCursor::default();
    assert!(cursor.advance(&line(100, 200, "a")));
    assert!(cursor.advance(&line(100, 500, "b")));
    assert_eq!(cursor.since(), Some(100));
    // The new stream starts at second 100 and sends a and b again.
    assert!(!cursor.advance(&line(100, 200, "a")));
    assert!(!cursor.advance(&line(100, 500, "b")));
    assert!(cursor.advance(&line(100, 500, "c")));
    assert!(cursor.advance(&line(100, 900, "d")));
}

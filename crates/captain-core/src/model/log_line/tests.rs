use super::{LogLevel, LogLine, LogStream};

#[test]
fn detects_levels_from_text() {
    assert_eq!(LogLevel::detect("GET /healthz 200"), LogLevel::Info);
    assert_eq!(LogLevel::detect("WARN slow query"), LogLevel::Warn);
    assert_eq!(
        LogLevel::detect("level=warning msg=retrying"),
        LogLevel::Warn
    );
    assert_eq!(
        LogLevel::detect("Error: connect ECONNREFUSED"),
        LogLevel::Error
    );
    assert_eq!(
        LogLevel::detect("FATAL: role does not exist"),
        LogLevel::Error
    );
}

#[test]
fn stderr_alone_is_not_an_error() {
    let line = LogLine::new(LogStream::Stderr, "nginx: starting worker");
    assert_eq!(line.level, LogLevel::Info);
}

#[test]
fn trailing_newlines_are_trimmed() {
    assert_eq!(LogLine::new(LogStream::Stdout, "hello\r\n").text, "hello");
}

#[test]
fn docker_time_is_split_from_the_text() {
    let line = LogLine::with_docker_time(LogStream::Stderr, "2024-05-01T12:34:56.1Z WARN slow\n");
    assert_eq!(line.timestamp, Some(1_714_566_896));
    assert_eq!(line.text, "WARN slow");
    assert_eq!(line.level, LogLevel::Warn);
    assert_eq!(line.clock(3600).as_deref(), Some("13:34:56"));
    assert_eq!(line.copy_text(Some(0)), "12:34:56 WARN slow");
    assert_eq!(line.copy_text(None), "WARN slow");
}

#[test]
fn a_line_without_a_time_keeps_its_text() {
    let line = LogLine::with_docker_time(LogStream::Stdout, "no time here");
    assert_eq!(line.timestamp, None);
    assert_eq!(line.text, "no time here");
    assert_eq!(line.clock(0), None);
    assert_eq!(line.copy_text(Some(0)), "no time here");
}

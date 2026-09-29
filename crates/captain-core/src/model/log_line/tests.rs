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

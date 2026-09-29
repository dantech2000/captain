use super::{LOG_BUFFER_LEN, LevelFilter, LogBuffer};
use crate::model::{LogLevel, LogLine, LogStream};

fn line(text: &str) -> LogLine {
    LogLine::new(LogStream::Stdout, text)
}

#[test]
fn drops_the_oldest_line_when_full() {
    let mut buffer = LogBuffer::default();
    for i in 0..LOG_BUFFER_LEN + 1 {
        buffer.push(line(&i.to_string()));
    }
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

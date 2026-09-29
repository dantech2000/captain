use bollard::container::LogOutput;
use captain_core::model::LogStream;

use super::log_lines;

#[test]
fn splits_the_docker_time_from_each_line() {
    let frame = LogOutput::StdErr {
        message: "2024-05-01T12:34:56.1Z first\n2024-05-01T12:34:57.2Z second\n".into(),
    };
    let lines = log_lines(frame);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].stream, LogStream::Stderr);
    assert_eq!(lines[0].text, "first");
    assert_eq!(lines[0].timestamp, Some(1_714_566_896));
    assert_eq!(lines[1].text, "second");
    assert_eq!(lines[1].timestamp, Some(1_714_566_897));
}

#[test]
fn keeps_lines_without_a_time_and_drops_blank_ones() {
    let frame = LogOutput::StdOut {
        message: "plain\n2024-05-01T12:34:56Z \n\n2024-05-01T12:34:56Z  \n".into(),
    };
    let lines = log_lines(frame);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].stream, LogStream::Stdout);
    assert_eq!(lines[0].text, "plain");
    assert_eq!(lines[0].timestamp, None);
}

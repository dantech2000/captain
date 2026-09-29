use bollard::container::LogOutput;
use captain_core::model::{LogLine, LogStream};

/// Splits one output frame into lines. A TTY container can send several lines,
/// or part of one, in a single frame.
pub fn log_lines(output: LogOutput) -> Vec<LogLine> {
    let (stream, bytes) = match output {
        LogOutput::StdErr { message } => (LogStream::Stderr, message),
        LogOutput::StdOut { message }
        | LogOutput::Console { message }
        | LogOutput::StdIn { message } => (LogStream::Stdout, message),
    };
    String::from_utf8_lossy(&bytes)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| LogLine::new(stream, line))
        .collect()
}

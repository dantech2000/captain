use bollard::container::LogOutput;
use captain_core::model::{LogLine, LogStream};

/// Splits one output frame into lines. A TTY container can send several lines,
/// or part of one, in a single frame. Each line can start with the time Docker adds
/// when the request asks for timestamps. Lines with no text after the time are dropped.
pub fn log_lines(output: LogOutput) -> Vec<LogLine> {
    let (stream, bytes) = match output {
        LogOutput::StdErr { message } => (LogStream::Stderr, message),
        LogOutput::StdOut { message }
        | LogOutput::Console { message }
        | LogOutput::StdIn { message } => (LogStream::Stdout, message),
    };
    String::from_utf8_lossy(&bytes)
        .lines()
        .map(|line| LogLine::with_docker_time(stream, line))
        .filter(|line| !line.text.trim().is_empty())
        .collect()
}

#[cfg(test)]
mod tests;

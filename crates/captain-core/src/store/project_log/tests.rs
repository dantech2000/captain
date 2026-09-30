use super::{ProjectLog, ProjectLogEntry};
use crate::model::{EngineEvent, EventKind, LogLine, LogStream};

fn line(time: i64, text: &str) -> LogLine {
    LogLine {
        timestamp: Some(time),
        ..LogLine::new(LogStream::Stdout, text)
    }
}

fn event(action: &str, time: i64) -> EngineEvent {
    EngineEvent {
        kind: EventKind::Container,
        action: action.into(),
        id: "worker-id".into(),
        time: Some(time),
        exit_code: (action == "die").then_some(137),
        ..EngineEvent::default()
    }
}

#[test]
fn orders_lines_of_all_services_by_time_with_an_exit_divider() {
    let mut log = ProjectLog::default();
    log.push_line("api", line(10, "POST /cart"));
    log.push_line("worker", line(12, "heap out of memory"));
    log.record("worker", &event("oom", 12));
    log.record("worker", &event("die", 13));
    // A late stream sends an older line; it goes in its place, not at the end.
    log.push_line("api", line(11, "GET /healthz"));
    log.push_line("worker", line(14, "starting worker"));

    let rows: Vec<String> = log
        .entries()
        .map(|entry| match entry {
            ProjectLogEntry::Line { service, line } => format!("{service}: {}", line.text),
            ProjectLogEntry::Exit(exit) => exit.label(),
        })
        .collect();
    assert_eq!(
        rows,
        [
            "api: POST /cart",
            "api: GET /healthz",
            "worker: heap out of memory",
            "worker exited 137 (out of memory)",
            "worker: starting worker",
        ]
    );
}

//! The messages are real output of Compose v5.5.1.

use super::*;
use crate::project_files::Severity;

fn lines(stderr: &str, text: &str) -> Vec<Option<usize>> {
    compose_problems(stderr, text)
        .into_iter()
        .map(|problem| problem.line)
        .collect()
}

#[test]
fn schema_and_service_errors_map_to_their_key_line() {
    let text = "services:\n  web:\n    imgae: busybox\n    ports: 80\n";
    let stderr = "validating -: services.web additional properties 'imgae' not allowed\n";
    assert_eq!(lines(stderr, text), [Some(2)]);
    let stderr = "validating -: services.web.ports must be a array\n";
    assert_eq!(lines(stderr, text), [Some(3)]);
    let root = "servicez:\n  web:\n    image: busybox\n";
    let stderr = "validating -:  additional properties 'servicez' not allowed\n";
    assert_eq!(lines(stderr, root), [Some(0)]);
    // A named service points at its key.
    let text = "services:\n  web:\n    image: busybox\n    depends_on: [db]\n";
    let stderr = "service \"web\" depends on undefined service \"db\": invalid compose project\n";
    assert_eq!(lines(stderr, text), [Some(1)]);
}

#[test]
fn yaml_errors_use_the_line_where_the_parser_stopped() {
    let text = "services:\n  web:\n    image: busybox\n   bad: 1\n";
    let stderr = "failed to parse -: go-yaml load error in parser (while parsing a block \
                  mapping) at L2.C3-L4.C4: did not find expected key\n";
    assert_eq!(lines(stderr, text), [Some(3)]);
    let open = "services:\n  web:\n    image: [busybox\n";
    let stderr = "failed to parse -: go-yaml load error in parser (while parsing a flow \
                  sequence) at L3.C12-L4.C1: did not find expected ',' or ']'\n";
    assert_eq!(lines(stderr, open), [Some(2)]);
}

#[test]
fn other_files_have_no_line_and_warnings_stay_warnings() {
    let text = "version: \"3\"\nservices:\n  api:\n    image: busybox\n";
    let stderr = "validating /srv/shop/other.yaml: services.web.ports must be a array\n\
                  time=\"2026-09-30T13:16:41-07:00\" level=warning msg=\"-: the attribute \
                  `version` is obsolete, it will be ignored\"\n";
    let problems = compose_problems(stderr, text);
    assert_eq!(problems[0].line, None);
    assert!(problems[0].message.starts_with("other.yaml: "));
    assert_eq!(problems[1].line, Some(0));
    assert_eq!(problems[1].severity, Severity::Warning);
}

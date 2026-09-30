//! Real output of Buildx 0.37.1 against BuildKit v0.33.0, with `sources` removed.

use super::*;
use crate::project_files::Severity;

fn range(line: u32) -> String {
    format!(r#"{{"ranges":[{{"start":{{"line":{line}}},"end":{{"line":{line}}}}}]}}"#)
}

#[test]
fn warnings_land_on_their_lines() {
    let stdout = format!(
        r#"{{"warnings":[{{"ruleName":"LegacyKeyValueFormat","description":"Legacy key/value format with whitespace separator should not be used","url":"https://docs.docker.com/go/dockerfile/rule/legacy-key-value-format/","detail":"\"ENV key=value\" should be used instead of legacy \"ENV key value\" format","location":{}}}]}}"#,
        range(3)
    );
    let problems = build_check_problems(&stdout).unwrap();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].line, Some(2));
    assert_eq!(problems[0].severity, Severity::Warning);
    assert!(problems[0].message.ends_with("(LegacyKeyValueFormat)"));
}

#[test]
fn a_parse_error_is_an_error_and_an_unknown_base_image_a_warning() {
    let stdout = format!(
        r#"{{"warnings":null,"buildError":{{"message":"dockerfile parse error on line 2: unknown instruction: RUNN (did you mean RUN?)","location":{}}}}}"#,
        range(2)
    );
    let problems = build_check_problems(&stdout).unwrap();
    assert_eq!(
        problems,
        [LineProblem::error(
            Some(1),
            "unknown instruction: RUNN (did you mean RUN?)"
        )]
    );

    let stdout = format!(
        r#"{{"warnings":null,"buildError":{{"message":"captain-agent-nope/nothing:1: failed to resolve source metadata for docker.io/captain-agent-nope/nothing:1: pull access denied, repository does not exist or may require authorization","location":{}}}}}"#,
        range(1)
    );
    let problems = build_check_problems(&stdout).unwrap();
    assert_eq!(problems[0].severity, Severity::Warning);
    assert_eq!(problems[0].line, Some(0));
}

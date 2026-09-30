use super::{SuggestionKind, complete};
use crate::grammar::fixture::catalog;
use crate::grammar::{Action, Flag, Target, Verb, examples};

fn lines(line: &str) -> Vec<String> {
    complete(line, &catalog())
        .into_iter()
        .map(|s| s.line)
        .collect()
}

#[test]
fn the_first_word_completes_to_verbs_and_pages() {
    let found = complete("re", &catalog());
    assert_eq!(found[0].kind, SuggestionKind::Verb(Verb::Restart));
    assert_eq!(found[0].bold, vec![0..2]);
    assert_eq!(found[0].action, None);
    assert_eq!(lines("re"), ["restart", "resume"]);
    assert_eq!(lines("sett"), ["go settings"]);
    // A plain search word gets no suggestions.
    assert!(complete("redis", &catalog()).is_empty());
}

#[test]
fn a_partly_typed_name_ranks_live_names() {
    let found = complete("restart wo", &catalog());
    assert_eq!(
        found.iter().map(|s| s.line.as_str()).collect::<Vec<_>>(),
        [
            "restart worker",
            "restart shop-worker-1",
            "restart shop-worker-2"
        ]
    );
    // The typed verb and the matched characters are bold.
    assert_eq!(found[0].bold, vec![0..7, 8..10]);
    assert!(found.iter().all(|s| s.action.is_some()));
    // A service in two projects gets its project in front.
    assert_eq!(lines("shell ap")[..2], ["shell shop/api", "shell blog/api"]);
}

#[test]
fn a_scaled_service_needs_a_pick_for_shell() {
    let found = complete("shell worker", &catalog());
    assert_eq!(found[0].line, "shell worker");
    assert_eq!(found[0].action, None);
    assert_eq!(
        found[1].action,
        Some(Action::Shell("shop-worker-1-id".into()))
    );
}

#[test]
fn a_complete_name_offers_the_line_then_options_and_times() {
    let found = complete("logs web ", &catalog());
    assert_eq!(found[0].kind, SuggestionKind::Line);
    assert_eq!(found[0].line, "logs web");
    assert_eq!(found[1].kind, SuggestionKind::Flag(Flag::Since));
    assert_eq!(found[1].action, None);
    assert!(found[2].action.is_some());
    assert_eq!(
        lines("logs web --since "),
        [
            "logs web --since 10m",
            "logs web --since 1h",
            "logs web --since 1d"
        ]
    );
}

#[test]
fn a_name_that_is_a_project_and_a_container_offers_both() {
    let found = complete("restart blog", &catalog());
    let targets: Vec<&SuggestionKind> = found.iter().map(|s| &s.kind).collect();
    assert!(targets.contains(&&SuggestionKind::Target(Target::Project("blog".into()))));
    assert!(targets.contains(&&SuggestionKind::Target(Target::Container(
        "blog-id".into()
    ))));
    assert!(found.iter().all(|s| s.action.is_some()));
}

#[test]
fn examples_use_live_names() {
    assert_eq!(
        examples(&catalog()),
        [
            "restart web",
            "logs shop/api --since 10m",
            "disk",
            "forward svc/default/web 8080",
            "shell postgres"
        ]
    );
}

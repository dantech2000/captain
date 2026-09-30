use gpui_kit::assets::IconName;
use gpui_kit::hsla;

use super::{child_index, rank};
use crate::palette::command::{Command, CommandKind, Section};
use crate::workspace::Page;

fn command(section: Section, title: &str, suggested: bool) -> Command {
    Command {
        section,
        title: title.into(),
        meta: String::new(),
        icon: IconName::Box.into(),
        color: hsla(0., 0., 0., 1.),
        suggested,
        kind: CommandKind::GoTo(Page::Containers),
    }
}

fn titles(commands: &[super::Ranked]) -> Vec<&str> {
    commands.iter().map(|r| r.command.title.as_str()).collect()
}

fn sample() -> Vec<Command> {
    vec![
        command(Section::Containers, "api", true),
        command(Section::Navigate, "Go to Containers", true),
        command(Section::Actions, "Restart api", false),
        command(Section::Actions, "Stop api", false),
        command(Section::Containers, "redis", false),
    ]
}

#[test]
fn empty_query_lists_suggested_commands_by_section() {
    let ranked = rank(sample(), "  ");
    assert_eq!(titles(&ranked), ["Go to Containers", "api"]);
    assert!(ranked.iter().all(|r| r.ranges.is_empty()));
}

#[test]
fn query_keeps_only_matches_with_ranges() {
    let ranked = rank(sample(), "rest");
    assert_eq!(titles(&ranked), ["Restart api"]);
    assert_eq!(ranked[0].ranges, vec![0..4]);
}

#[test]
fn best_section_comes_first_and_rows_stay_grouped() {
    let ranked = rank(sample(), "api");
    assert_eq!(titles(&ranked), ["api", "Restart api", "Stop api"]);
}

#[test]
fn child_index_counts_section_headers() {
    let ranked = rank(sample(), "api");
    assert_eq!(child_index(&ranked, 0), 1);
    assert_eq!(child_index(&ranked, 1), 3);
    assert_eq!(child_index(&ranked, 2), 4);
}

use std::path::PathBuf;

use super::SetupSteps;
use crate::cli_tools::{
    LinkReport, LinkState, RcAccess, RcFile, RcState, RcStatus, Shell, ToolLink, ToolSource,
    ToolsStatus,
};

fn status(link: LinkState, rc: RcState) -> ToolsStatus {
    ToolsStatus {
        links: vec![LinkReport {
            link: ToolLink {
                path: PathBuf::from("/home/me/.captain/bin/docker"),
                target: PathBuf::from("/Applications/Captain.app/Contents/Resources/bin/docker"),
            },
            state: link,
            error: None,
        }],
        plugins: true,
        rc: vec![RcStatus {
            file: RcFile {
                shell: Shell::Zsh,
                path: PathBuf::from("/home/me/.zshrc"),
            },
            state: rc,
            access: RcAccess::Writable,
        }],
    }
}

#[test]
fn all_three_steps_make_the_terminal_use_captain_engine() {
    let steps = SetupSteps::of(
        &status(LinkState::Linked, RcState::Added),
        true,
        Some(&ToolSource::Captain),
        true,
    );
    assert!(steps.all());
    assert_eq!(
        steps.summary(Some(&ToolSource::Captain), "captain-engine"),
        "docker, Compose, and Buildx in your terminal use Captain Engine"
    );
}

#[test]
fn the_summary_names_where_docker_still_comes_from() {
    let rancher = ToolSource::RancherDesktop;
    let steps = SetupSteps::of(
        &status(LinkState::Linked, RcState::Missing),
        true,
        Some(&rancher),
        false,
    );
    assert_eq!(steps.done(), 1);
    assert_eq!(
        steps.summary(Some(&rancher), "rancher-desktop"),
        "Your terminal's docker still comes from Rancher Desktop."
    );
}

#[test]
fn the_summary_names_the_context_when_only_the_context_is_left() {
    let steps = SetupSteps::of(
        &status(LinkState::Linked, RcState::Added),
        true,
        Some(&ToolSource::Captain),
        false,
    );
    assert_eq!(
        steps.summary(Some(&ToolSource::Captain), "rancher-desktop"),
        "docker commands use the rancher-desktop context, not Captain Engine."
    );
}

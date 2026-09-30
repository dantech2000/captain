//! The three steps that make a terminal use Captain Engine: the links, PATH, and
//! the docker context. Settings shows one line from them, and a checklist. See
//! docs/features/0037-settings-page.md.

use super::{LinkState, RcState, ToolSource, ToolsStatus};

/// Which steps are done.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SetupSteps {
    /// Every link points into this `Captain.app`, and `config.json` lists the
    /// plugin folder.
    pub links: bool,
    /// A new terminal finds `~/.captain/bin` first.
    pub path: bool,
    /// The docker CLI's default context is `captain-engine`, pointing at Captain
    /// Engine.
    pub context: bool,
}

impl SetupSteps {
    pub const COUNT: usize = 3;

    /// Reads the steps from the tool status. `enabled` is the saved
    /// `command_line_tools.enabled`; `docker` is where a new terminal finds
    /// `docker`, when Captain could tell.
    pub fn of(
        status: &ToolsStatus,
        enabled: bool,
        docker: Option<&ToolSource>,
        context: bool,
    ) -> Self {
        let links = enabled
            && status.plugins
            && !status.links.is_empty()
            && status
                .links
                .iter()
                .all(|report| matches!(report.state, LinkState::Linked | LinkState::NoTarget));
        let path = docker == Some(&ToolSource::Captain)
            || status.rc.iter().any(|rc| rc.state != RcState::Missing);
        Self {
            links,
            path,
            context,
        }
    }

    pub fn done(&self) -> usize {
        [self.links, self.path, self.context]
            .into_iter()
            .filter(|done| *done)
            .count()
    }

    pub fn all(&self) -> bool {
        self.done() == Self::COUNT
    }

    /// One sentence for Settings: that all is done, or the first thing left.
    /// `current` is the docker CLI's default context.
    pub fn summary(&self, docker: Option<&ToolSource>, current: &str) -> String {
        let elsewhere =
            docker.filter(|source| !matches!(source, ToolSource::Captain | ToolSource::NotFound));
        if self.all() {
            "docker, Compose, and Buildx in your terminal use Captain Engine".into()
        } else if let Some(source) = elsewhere.filter(|_| !(self.links && self.path)) {
            format!("Your terminal's docker still comes from {}.", source.name())
        } else if !self.links {
            "Captain's docker, Compose, and Buildx are not linked into ~/.captain/bin yet.".into()
        } else if !self.path {
            "~/.captain/bin is not on your terminal's PATH yet.".into()
        } else {
            format!("docker commands use the {current} context, not Captain Engine.")
        }
    }
}

#[cfg(test)]
mod tests;

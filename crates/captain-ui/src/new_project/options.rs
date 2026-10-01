use gpui_kit::assets::IconName;

use crate::icons::{CaptainIcon, Glyph};

/// The four ways to make a project in the New sheet, in their order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewOption {
    RunImage,
    Template,
    OpenFolder,
    PasteRun,
}

impl NewOption {
    pub const ALL: [NewOption; 4] = [
        NewOption::RunImage,
        NewOption::Template,
        NewOption::OpenFolder,
        NewOption::PasteRun,
    ];

    pub fn title(self) -> &'static str {
        match self {
            NewOption::RunImage => "Run an image",
            NewOption::Template => "Start from a template",
            NewOption::OpenFolder => "Open a folder",
            NewOption::PasteRun => "Paste a docker run command",
        }
    }

    /// The line under the title.
    pub fn sentence(self) -> &'static str {
        match self {
            NewOption::RunImage => {
                "Pick an image from this engine or Docker Hub, with ports, environment, and volumes."
            }
            NewOption::Template => {
                "PostgreSQL, MySQL, Redis, MongoDB, RabbitMQ, or a web server for a folder."
            }
            NewOption::OpenFolder => "A folder or a Compose file that is already on this computer.",
            NewOption::PasteRun => "Captain turns the command into a Compose file you can edit.",
        }
    }

    pub fn icon(self) -> Glyph {
        match self {
            NewOption::RunImage => CaptainIcon::Image.into(),
            NewOption::Template => CaptainIcon::Stack.into(),
            NewOption::OpenFolder => IconName::FolderOpen.into(),
            NewOption::PasteRun => CaptainIcon::Exec.into(),
        }
    }

    /// True if the option works in this build. The others come in the next phases
    /// of docs/features/0040-new-projects.md.
    pub fn ready(self) -> bool {
        self == NewOption::OpenFolder
    }

    /// The status bar sentence of the option's card.
    pub fn help(self) -> &'static str {
        match self {
            NewOption::RunImage => {
                "Coming next: run an image as a one-service project, or as a plain container."
            }
            NewOption::Template => {
                "Coming next: a Compose project from a built-in template with pinned image tags."
            }
            NewOption::OpenFolder => {
                "Choose a folder or a Compose file. Captain checks it, remembers the project, and opens its files."
            }
            NewOption::PasteRun => {
                "Coming next: paste a docker run command and get a Compose file, with warnings for flags it cannot convert."
            }
        }
    }
}

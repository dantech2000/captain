/// A `docker compose` command that acts on a whole project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProjectAction {
    /// `up -d`: creates missing containers and starts every service.
    Up,
    /// `down`: stops and removes the containers and networks. Volumes stay.
    Down,
    /// `stop`: stops every service and keeps the containers.
    Stop,
    /// `restart`: restarts every service.
    Restart,
    /// `pull`: pulls the images of every service.
    Pull,
}

impl ProjectAction {
    /// The button and command label, for example `Up`.
    pub fn label(self) -> &'static str {
        match self {
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Stop => "Stop",
            Self::Restart => "Restart",
            Self::Pull => "Pull",
        }
    }

    /// What the card shows while the command runs.
    pub fn progress_label(self) -> &'static str {
        match self {
            Self::Up => "Starting...",
            Self::Down => "Removing...",
            Self::Stop => "Stopping...",
            Self::Restart => "Restarting...",
            Self::Pull => "Pulling...",
        }
    }

    /// The success message for `project`, for example `Pulled the images of shop.`
    pub fn done_message(self, project: &str) -> String {
        match self {
            Self::Up => format!("Started {project}."),
            Self::Down => format!("Removed the containers of {project}."),
            Self::Stop => format!("Stopped {project}."),
            Self::Restart => format!("Restarted {project}."),
            Self::Pull => format!("Pulled the images of {project}."),
        }
    }

    /// The `docker compose` subcommand and its flags.
    pub fn args(self) -> &'static [&'static str] {
        match self {
            Self::Up => &["up", "-d"],
            Self::Down => &["down"],
            Self::Stop => &["stop"],
            Self::Restart => &["restart"],
            Self::Pull => &["pull"],
        }
    }

    /// True if the command reads the Compose files. `down`, `stop`, and `restart`
    /// also work with only the project name.
    pub fn needs_files(self) -> bool {
        matches!(self, Self::Up | Self::Pull)
    }
}

#[cfg(test)]
mod tests;

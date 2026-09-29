use crate::diagnostics::{
    Check, CheckId, CheckState, Fix, MINIMUM_LIMA_VERSION, Platform, ToolProbe, parse_version,
};

/// `limactl` exists and is new enough. Only Captain Engine on macOS uses Lima.
pub fn lima(platform: Platform, captain: bool, probe: &ToolProbe) -> Check {
    let id = CheckId::Lima;
    if !platform.macos {
        return Check::new(
            id,
            CheckState::NotApplicable,
            "Captain uses Lima only on macOS.",
        );
    }
    if !captain {
        return Check::new(
            id,
            CheckState::NotApplicable,
            "Captain uses another engine.",
        );
    }
    match probe {
        ToolProbe::Missing => Check::new(
            id,
            CheckState::Failed,
            "limactl is not installed. Captain Engine needs it.",
        )
        .with_fix(Fix::CopyCommand("brew install lima")),
        ToolProbe::Broken(why) => Check::new(id, CheckState::Failed, why.clone()),
        ToolProbe::Found(version) => {
            let minimum = parse_version(MINIMUM_LIMA_VERSION);
            match parse_version(version) {
                None => Check::new(
                    id,
                    CheckState::Warning,
                    format!("Captain cannot read the Lima version \"{version}\"."),
                ),
                Some(found) if Some(found) < minimum => Check::new(
                    id,
                    CheckState::Failed,
                    format!(
                        "Lima {version} is too old. Captain Engine needs \
                         {MINIMUM_LIMA_VERSION} or newer (lima#5210)."
                    ),
                )
                .with_fix(Fix::CopyCommand("brew upgrade lima")),
                Some(_) => Check::new(id, CheckState::Passed, format!("Lima {version}.")),
            }
        }
    }
}

/// The `docker` CLI exists. Compose projects need it (ADR 0005).
pub fn docker_cli(platform: Platform, probe: &ToolProbe) -> Check {
    let id = CheckId::DockerCli;
    match probe {
        ToolProbe::Found(version) => {
            Check::new(id, CheckState::Passed, format!("Docker CLI {version}."))
        }
        ToolProbe::Broken(why) => Check::new(id, CheckState::Warning, why.clone()),
        ToolProbe::Missing => {
            let check = Check::new(
                id,
                CheckState::Warning,
                "The docker CLI is not installed. Compose projects need it.",
            );
            if platform.macos {
                check.with_fix(Fix::CopyCommand("brew install docker"))
            } else {
                check
            }
        }
    }
}

/// The Compose plugin answers. Without the docker CLI it cannot run at all.
pub fn compose(docker: &ToolProbe, probe: &ToolProbe) -> Check {
    let id = CheckId::Compose;
    if *docker == ToolProbe::Missing {
        return Check::new(id, CheckState::Warning, "Compose needs the docker CLI.");
    }
    match probe {
        ToolProbe::Found(version) => {
            Check::new(id, CheckState::Passed, format!("Docker Compose {version}."))
        }
        ToolProbe::Broken(why) => Check::new(id, CheckState::Warning, why.clone()),
        ToolProbe::Missing => Check::new(
            id,
            CheckState::Warning,
            "The Compose plugin is not installed. Compose projects need it.",
        ),
    }
}

#[cfg(test)]
mod tests;

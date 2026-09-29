use crate::HostStatus;
use crate::diagnostics::{Check, CheckId, CheckState, EngineProbe, Fix};

/// Whether the engine answers. `captain` is Captain Engine's status when the
/// settings choose it.
///
/// Captain Engine's VM can keep running while its Docker socket refuses
/// connections, most often after the Mac sleeps (lima-vm/lima#5420). A restart
/// brings the socket back.
pub fn engine(probe: &EngineProbe, captain: Option<&HostStatus>) -> Check {
    let id = CheckId::Engine;
    let reason = match probe {
        EngineProbe::Answered {
            version,
            api_version,
        } => {
            return Check::new(
                id,
                CheckState::Passed,
                format!("Docker {version}, API {api_version}."),
            );
        }
        // Captain does not connect to a Captain Engine that is not running, so its
        // status says more than the connection.
        EngineProbe::Connecting if matches!(captain, None | Some(HostStatus::Running)) => {
            return Check::new(
                id,
                CheckState::Warning,
                "Captain is connecting to the engine.",
            );
        }
        EngineProbe::Connecting => "",
        EngineProbe::NoAnswer(reason) => reason.as_str(),
    };
    match captain {
        None => Check::new(
            id,
            CheckState::Failed,
            format!("The engine does not answer: {reason}"),
        ),
        Some(HostStatus::Running) => Check::new(
            id,
            CheckState::Failed,
            "Captain Engine runs, but its Docker socket does not answer. This is a known \
             Lima issue after the Mac sleeps (lima#5420).",
        )
        .with_fix(Fix::RestartEngine),
        Some(HostStatus::Starting | HostStatus::Stopping) => Check::new(
            id,
            CheckState::Warning,
            "Captain Engine is starting or stopping.",
        ),
        Some(HostStatus::NotInstalled(why)) => Check::new(id, CheckState::Failed, why.clone()),
        Some(HostStatus::NotCreated) => {
            Check::new(id, CheckState::Failed, "Captain Engine is not set up.")
                .with_fix(Fix::StartEngine)
        }
        Some(HostStatus::Stopped) => {
            Check::new(id, CheckState::Failed, "Captain Engine is stopped.")
                .with_fix(Fix::StartEngine)
        }
        Some(HostStatus::Failed(why)) => Check::new(
            id,
            CheckState::Failed,
            format!("Captain Engine failed: {why}"),
        )
        .with_fix(Fix::StartEngine),
    }
}

#[cfg(test)]
mod tests;

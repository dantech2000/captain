use crate::diagnostics::{Check, CheckId, CheckState, Fix};

/// Whether `settings.json` reads cleanly. A mistake keeps the last good settings
/// until the user fixes it. See ADR 0013.
pub fn settings_file(problem: Option<&str>) -> Check {
    match problem {
        None => Check::new(
            CheckId::SettingsFile,
            CheckState::Passed,
            "settings.json reads cleanly.",
        ),
        Some(problem) => Check::new(
            CheckId::SettingsFile,
            CheckState::Failed,
            format!("Captain keeps the last good settings. {problem}"),
        )
        .with_fix(Fix::OpenSettingsFile),
    }
}

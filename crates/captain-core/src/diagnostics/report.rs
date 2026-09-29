use super::{Check, CheckState, Facts, checks};

/// Runs every check on `facts`, in the order the page lists them.
pub fn evaluate(facts: &Facts) -> Vec<Check> {
    let platform = facts.platform;
    let machine = &facts.machine;
    let captain = facts.captain_engine.is_some();
    vec![
        checks::engine(&facts.engine, facts.captain_engine.as_ref()),
        checks::lima(platform, captain, &machine.lima),
        checks::docker_cli(platform, &machine.docker),
        checks::compose(&machine.docker, &machine.compose),
        checks::disk_space(platform, machine.free_disk),
        checks::lima_logs(platform, captain, machine.lima_logs),
        checks::rosetta(platform, captain, machine.rosetta),
    ]
}

/// The number of failed checks, for the sidebar badge. Warnings and checks that do
/// not apply are not failures.
pub fn failure_count(checks: &[Check]) -> usize {
    checks
        .iter()
        .filter(|check| check.state == CheckState::Failed)
        .count()
}

#[cfg(test)]
mod tests;

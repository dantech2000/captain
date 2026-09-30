use captain_core::model::count_label;

/// The status bar sentences of the project header's buttons. The ⌘K palette uses
/// the same sentences for its commands.
pub fn up_help(name: &str) -> String {
    format!("Create and start the services of {name} (docker compose up).")
}

/// `services` is the project's service count, for example "3 services".
pub fn restart_help(services: &str, name: &str) -> String {
    format!("Restart the {services} of {name}.")
}

/// `count` is the number of containers.
pub fn down_help(name: &str, count: usize) -> String {
    format!(
        "Stop and remove the {} of {name}. Volumes and images stay.",
        count_label(count, "container")
    )
}

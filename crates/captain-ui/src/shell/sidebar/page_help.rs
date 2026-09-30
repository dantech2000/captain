use crate::workspace::Page;

/// The status bar sentence for a sidebar page entry. `count` is the number the entry
/// shows: items on the page, or failed checks for Diagnostics.
pub fn page_help(page: Page, count: Option<usize>) -> String {
    match (page, count) {
        (Page::Containers, Some(n)) => {
            format!("Show the {n} containers on the engine, grouped by project.")
        }
        (Page::Containers, None) => "Show the containers on the engine, grouped by project.".into(),
        (Page::Project, _) => "Show one project: its services, ports, tasks, and log.".into(),
        (Page::Images, _) => "Show the images on the engine. Pull, build, and remove them.".into(),
        (Page::Volumes, _) => {
            "Show the volumes on the engine and the containers that use them.".into()
        }
        (Page::Networks, _) => "Show the networks on the engine and their containers.".into(),
        (Page::Extensions, _) => "Install, open, and remove Docker Desktop extensions.".into(),
        (Page::Snapshots, _) => "Save and restore snapshots of Captain Engine.".into(),
        (Page::Storage, _) => "Show what fills the engine's disk, and free up space.".into(),
        (Page::PortForwarding, _) => {
            "Forward Kubernetes services to ports on this computer.".into()
        }
        (Page::Diagnostics, Some(n)) => {
            format!("Show the checks of the engine and its tools. {n} failed.")
        }
        (Page::Diagnostics, None) => "Show the checks of the engine and its tools.".into(),
        (Page::Settings, _) => {
            "Change the engine, its resources, Kubernetes, and the appearance.".into()
        }
    }
}

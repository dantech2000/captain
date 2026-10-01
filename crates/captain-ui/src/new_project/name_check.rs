use std::path::PathBuf;

use captain_core::new_project::{
    folder_is_free, project_name_error, to_project_name, unique_project_name,
};
use gpui_kit::*;

use super::host::SheetHost;
use super::known_projects;

/// `projects_dir` from the settings, with `~/` expanded.
pub fn projects_dir(cx: &App) -> PathBuf {
    let home = std::env::home_dir().unwrap_or_default();
    crate::settings::current(cx).projects_dir_in(&home)
}

/// The project names in use: the known projects and the running ones.
fn taken_names(host: &SheetHost, cx: &App) -> Vec<String> {
    let mut names: Vec<String> = known_projects(cx).into_iter().map(|p| p.name).collect();
    names.extend(
        host.workspace
            .read(cx)
            .compose_projects()
            .into_iter()
            .map(|p| p.name),
    );
    names
}

/// Why `name` cannot be a new project, or `None`.
pub fn name_problem(name: &str, host: &SheetHost, cx: &App) -> Option<String> {
    if let Some(error) = project_name_error(name) {
        return Some(error.into());
    }
    if taken_names(host, cx).iter().any(|taken| taken == name) {
        return Some(format!("Captain already has a project named {name}."));
    }
    let dir = projects_dir(cx).join(name);
    if !folder_is_free(&dir) {
        return Some(format!(
            "{} has files already. Pick another name.",
            dir.display()
        ));
    }
    None
}

/// A free project name from `base`, such as `postgres` or `postgres-2`.
pub fn unique_name(base: &str, host: &SheetHost, cx: &App) -> String {
    let taken = taken_names(host, cx);
    let dir = projects_dir(cx);
    unique_project_name(&to_project_name(base), |name| {
        taken.iter().any(|t| t == name) || !folder_is_free(&dir.join(name))
    })
}

/// The host ports that containers publish now.
pub fn published_ports(host: &SheetHost, cx: &App) -> Vec<u16> {
    host.workspace
        .read(cx)
        .store()
        .containers()
        .iter()
        .flat_map(|c| c.published_ports())
        .collect()
}

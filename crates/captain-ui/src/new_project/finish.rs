use std::time::{SystemTime, UNIX_EPOCH};

use captain_core::known_projects::KnownProject;
use captain_core::new_project::{NewFile, write_project};
use captain_core::store::GroupKey;
use gpui_kit::component::WindowExt;
use gpui_kit::*;

use super::host::SheetHost;
use super::known_model;
use super::name_check::projects_dir;

/// Writes `files` into the new folder `name` under `projects_dir`, records the
/// project, closes the sheet, and shows the project's Files tab, where Save and
/// apply previews `up`. Nothing starts here.
pub fn create_project(
    host: &SheetHost,
    name: &str,
    files: &[NewFile],
    window: &mut Window,
    cx: &mut App,
) -> Result<(), String> {
    let dir = projects_dir(cx).join(name);
    write_project(&dir, files).map_err(|error| error.to_string())?;
    let project = KnownProject {
        name: name.to_string(),
        dir,
        files: vec!["compose.yaml".into()],
        added: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_secs() as i64),
    };
    known_model(cx).update(cx, |model, cx| model.add(project, cx))?;
    window.close_dialog(cx);
    open_project(host, name.to_string(), cx);
    Ok(())
}

/// Shows the project `name` on its Files tab.
pub fn open_project(host: &SheetHost, name: String, cx: &mut App) {
    host.workspace.update(cx, |workspace, cx| {
        workspace.open_group(GroupKey::Project(name), cx)
    });
    host.project.update(cx, |view, cx| view.show_files(cx));
}

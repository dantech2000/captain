use std::path::Path;

use super::ComposeProject;

fn project(working_dir: Option<&str>) -> ComposeProject {
    ComposeProject {
        name: "shop".into(),
        working_dir: working_dir.map(Into::into),
        config_files: Vec::new(),
        services: Vec::new(),
    }
}

#[test]
fn short_working_dir_replaces_home_with_a_tilde() {
    let home = Some(Path::new("/Users/dan"));
    assert_eq!(
        project(Some("/Users/dan/code/shop")).short_working_dir(home),
        Some("~/code/shop".into())
    );
    assert_eq!(
        project(Some("/Users/dan")).short_working_dir(home),
        Some("~".into())
    );
    assert_eq!(
        project(Some("/srv/shop")).short_working_dir(home),
        Some("/srv/shop".into())
    );
    assert_eq!(project(None).short_working_dir(home), None);
}

#[test]
fn a_project_without_containers_is_stopped() {
    let empty = project(None);
    assert_eq!(empty.status(), super::ProjectStatus::Stopped);
    assert_eq!(empty.services_label(), "0 services");
}

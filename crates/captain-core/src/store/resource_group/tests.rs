use super::{group_by_project, visible_groups};

#[test]
fn groups_by_project_with_standalone_last() {
    let items = [
        ("a", None),
        ("b", Some("web")),
        ("c", Some("api")),
        ("d", Some("web")),
        ("e", None),
    ];
    let groups = group_by_project(&items, |item| item.1);
    let projects: Vec<_> = groups.iter().map(|g| g.project.as_deref()).collect();
    assert_eq!(projects, [Some("api"), Some("web"), None]);
    let web: Vec<_> = groups[1].items.iter().map(|i| i.0).collect();
    assert_eq!(web, ["b", "d"]);
    let standalone: Vec<_> = groups[2].items.iter().map(|i| i.0).collect();
    assert_eq!(standalone, ["a", "e"]);
}

#[test]
fn hides_extension_backend_groups_unless_shown() {
    let items = [
        ("a", Some("captain-ext-tool")),
        ("b", Some("web")),
        ("c", None),
    ];
    let projects = |show| {
        visible_groups(group_by_project(&items, |item| item.1), show)
            .into_iter()
            .map(|g| g.project)
            .collect::<Vec<_>>()
    };
    assert_eq!(projects(false), [Some("web".to_string()), None]);
    assert_eq!(projects(true).len(), 3);
}

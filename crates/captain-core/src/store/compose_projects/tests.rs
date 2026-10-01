use super::{compose_project, compose_projects};
use crate::model::{ComposeLabels, Container, ContainerState, ProjectStatus};

fn member(name: &str, project: Option<&str>, service: Option<&str>, running: bool) -> Container {
    Container {
        id: format!("id-{name}"),
        name: name.into(),
        image: "busybox".into(),
        state: if running {
            ContainerState::Running
        } else {
            ContainerState::Exited
        },
        status: String::new(),
        ports: Vec::new(),
        created: 0,
        compose_project: project.map(Into::into),
        compose: ComposeLabels {
            service: service.map(Into::into),
            working_dir: project.map(|p| format!("/code/{p}")),
            config_files: project
                .map(|p| vec![format!("/code/{p}/compose.yaml")])
                .unwrap_or_default(),
        },
        health: None,
        kube_namespace: None,
        extension: None,
    }
}

#[test]
fn groups_by_project_and_service_and_skips_standalone() {
    let containers = [
        member("shop-web-2", Some("shop"), Some("web"), true),
        member("lone", None, None, true),
        member("blog-db-1", Some("blog"), Some("db"), false),
        member("shop-db-1", Some("shop"), Some("db"), true),
        member("shop-web-1", Some("shop"), Some("web"), false),
    ];

    let projects = compose_projects(&containers);

    let names: Vec<&str> = projects.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["blog", "shop"]);
    let shop = &projects[1];
    assert_eq!(shop.working_dir.as_deref(), Some("/code/shop"));
    assert_eq!(shop.config_files, ["/code/shop/compose.yaml"]);
    let services: Vec<&str> = shop.services.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(services, ["db", "web"]);
    let web: Vec<&str> = shop.services[1]
        .containers
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(web, ["shop-web-1", "shop-web-2"]);
    assert_eq!(shop.container_count(), 3);
    assert_eq!(shop.active_count(), 2);
}

#[test]
fn status_is_running_partial_or_stopped() {
    let running = [member("a", Some("p"), Some("a"), true)];
    let partial = [
        member("a", Some("p"), Some("a"), true),
        member("b", Some("p"), Some("b"), false),
    ];
    let stopped = [member("a", Some("p"), Some("a"), false)];

    let status = |containers: &[Container]| compose_projects(containers)[0].status();
    assert_eq!(status(&running), ProjectStatus::Running);
    assert_eq!(status(&partial), ProjectStatus::Partial);
    assert_eq!(status(&stopped), ProjectStatus::Stopped);
}

#[test]
fn a_container_without_a_service_label_is_its_own_service() {
    let containers = [member("odd", Some("p"), None, true)];
    let project = &compose_projects(&containers)[0];
    assert_eq!(project.services[0].name, "odd");
    assert_eq!(project.services_label(), "1 service");
}

#[test]
fn a_missing_working_dir_label_takes_the_next_container() {
    let mut first = member("a", Some("p"), Some("a"), true);
    first.compose.working_dir = None;
    first.compose.config_files.clear();
    let containers = [first, member("b", Some("p"), Some("b"), true)];

    let project = &compose_projects(&containers)[0];
    assert_eq!(project.working_dir.as_deref(), Some("/code/p"));
    assert_eq!(project.config_files, ["/code/p/compose.yaml"]);
}

#[test]
fn finds_one_project_by_name() {
    let containers = [
        member("a", Some("shop"), Some("a"), true),
        member("b", Some("blog"), Some("b"), true),
    ];
    assert_eq!(
        compose_project(&containers, "blog").map(|p| p.name),
        Some("blog".into())
    );
    assert_eq!(compose_project(&containers, "gone"), None);
}

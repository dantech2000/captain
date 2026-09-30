use super::{find_container, find_project};
use crate::model::{Container, ContainerState};

fn container(name: &str, id: &str, project: Option<&str>) -> Container {
    Container {
        id: id.into(),
        name: name.into(),
        image: "app".into(),
        state: ContainerState::Running,
        status: String::new(),
        ports: Vec::new(),
        created: 0,
        compose_project: project.map(Into::into),
        compose: Default::default(),
        health: None,
        kube_namespace: None,
    }
}

#[test]
fn names_must_match_the_live_list() {
    let all = [
        container("shop-web-1", "abcdef012345", Some("shop")),
        container("cache", "abc999999999", None),
    ];
    assert_eq!(
        find_container(&all, "shop-web-1").unwrap().id,
        "abcdef012345"
    );
    assert_eq!(find_container(&all, "abcd").unwrap().name, "shop-web-1");
    assert!(find_container(&all, "abc").is_err());
    assert!(
        find_container(&all, "--all")
            .unwrap_err()
            .contains("not a container name")
    );
    let missing = find_container(&all, "db").unwrap_err();
    assert!(missing.contains("shop-web-1, cache"), "{missing}");
    assert_eq!(find_project(&all, "shop").unwrap().len(), 1);
    assert!(
        find_project(&all, "cache")
            .unwrap_err()
            .contains("Projects: shop.")
    );
}

use serde_json::json;

use super::{GUEST_SERVICES, PROXY_SERVICE, image_project, with_guest_services};

#[test]
fn an_image_backend_gets_the_volume_and_the_proxy() {
    let project = image_project("acme", "acme/ext:1", Some("api.sock"));
    assert_eq!(project["name"], "captain-ext-acme");
    let backend = &project["services"]["backend"];
    assert_eq!(backend["image"], "acme/ext:1");
    assert_eq!(backend["volumes"][0]["target"], GUEST_SERVICES);
    let proxy = &project["services"][PROXY_SERVICE];
    assert_eq!(
        proxy["command"][1],
        "UNIX-CONNECT:/run/guest-services/api.sock"
    );
    assert_eq!(proxy["ports"][0], "127.0.0.1::8080");
}

#[test]
fn a_compose_backend_keeps_its_own_mount_and_gets_no_proxy_without_a_socket() {
    let config = json!({
        "name": "whatever",
        "services": {
            "app": { "image": "a", "volumes": ["data:/run/guest-services"] },
            "db": { "image": "b", "restart": "always" }
        }
    });
    let project = with_guest_services(config, "acme", None);
    assert_eq!(project["name"], "captain-ext-acme");
    assert_eq!(
        project["services"]["app"]["volumes"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        project["services"]["db"]["volumes"][0]["target"],
        GUEST_SERVICES
    );
    assert_eq!(project["services"]["db"]["restart"], "always");
    assert!(project["services"].get(PROXY_SERVICE).is_none());
}

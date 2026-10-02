use super::*;

#[test]
fn names_follow_compose_rules() {
    assert_eq!(project_name_error("shop-2_db"), None);
    assert!(project_name_error("").is_some());
    assert!(project_name_error("-shop").is_some());
    assert!(project_name_error("Shop").is_some());
    assert!(project_name_error("my shop").is_some());
}

#[test]
fn text_and_images_become_valid_names() {
    assert_eq!(to_project_name("My Shop!"), "my-shop");
    assert_eq!(to_project_name("__"), "project");
    assert_eq!(image_project_name("ghcr.io/acme/api:1.2"), "api");
    assert_eq!(image_project_name("postgres@sha256:abc"), "postgres");
    assert_eq!(image_project_name("bitnami/redis:7"), "redis");
}

#[test]
fn a_taken_name_gets_the_next_free_number() {
    let taken = ["postgres", "postgres-2"];
    assert_eq!(
        unique_project_name("postgres", |name| taken.contains(&name)),
        "postgres-3"
    );
    assert_eq!(
        unique_project_name("redis", |name| taken.contains(&name)),
        "redis"
    );
    // The search stops when every name is taken.
    assert_eq!(unique_project_name("app", |_| true), "app");
}

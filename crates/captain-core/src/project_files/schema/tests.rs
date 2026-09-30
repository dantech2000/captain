use super::*;

fn path(keys: &[&str]) -> Vec<String> {
    keys.iter().map(|key| key.to_string()).collect()
}

fn names(keys: &[SchemaKey]) -> Vec<&str> {
    keys.iter().map(|key| key.name.as_str()).collect()
}

#[test]
fn offers_service_keys_through_the_service_pattern_and_refs() {
    let schema = ComposeSchema::bundled();
    let keys = schema.keys(&path(&["services", "web"]));
    for key in ["image", "build", "ports", "healthcheck", "depends_on"] {
        assert!(names(&keys).contains(&key), "missing {key}");
    }
    let healthcheck = schema.keys(&path(&["services", "web", "healthcheck"]));
    assert!(names(&healthcheck).contains(&"interval"));
    let port = schema.keys(&path(&["services", "web", "ports", "0"]));
    assert!(names(&port).contains(&"published"));
}

#[test]
fn documents_compose_keys_and_captain_tasks() {
    let schema = ComposeSchema::bundled();
    let doc = schema.doc(&path(&["services", "web", "build"])).unwrap();
    assert!(doc.contains("build"), "{doc}");
    let task = schema.keys(&path(&["x-captain", "tasks", "migrate"]));
    assert_eq!(names(&task), ["command", "service"]);
    assert!(schema.doc(&path(&["x-captain", "tasks"])).is_some());
}

use super::container_list;
use crate::agent_tools::test_fleet::fleet;

#[test]
fn twenty_two_containers_stay_under_8_kb() {
    let containers = fleet(22);
    let list = container_list(&containers, false, &|_| false);
    let json = serde_json::to_string(&list).unwrap();
    assert_eq!(list.containers.len(), 22);
    assert!(
        json.len() + list.text().len() < 8 * 1024,
        "{} + {}",
        json.len(),
        list.text().len()
    );
    assert!(
        json.contains("\"ports\":[\"http://localhost:8080\"]"),
        "{json}"
    );
    assert!(json.contains("localhost:5434 (Postgres)"));
}

#[test]
fn status_only_keeps_the_state_columns() {
    let list = container_list(&fleet(3), true, &|id| id.starts_with("0"));
    let json = serde_json::to_value(&list).unwrap();
    let first = json["containers"][0].as_object().unwrap();
    let mut keys: Vec<&str> = first.keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(keys, ["health", "name", "needs_attention", "state"]);
}

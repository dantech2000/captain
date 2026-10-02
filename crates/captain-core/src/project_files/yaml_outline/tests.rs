use super::*;

const COMPOSE: &str = "\
services:
  web:
    image: nginx
    command: |
      ports: not a key
    ports:
    - \"80:80\"
    - target: 443
      published: 8443
  \"db\":
    image: postgres
";

#[test]
fn key_line_finds_nested_keys_list_items_and_quoted_keys() {
    assert_eq!(key_line(COMPOSE, &["services", "web", "ports"]), Some(5));
    assert_eq!(
        key_line(COMPOSE, &["services", "web", "ports", "1"]),
        Some(7)
    );
    assert_eq!(
        key_line(COMPOSE, &["services", "web", "ports", "1", "published"]),
        Some(8)
    );
    assert_eq!(key_line(COMPOSE, &["services", "db", "image"]), Some(10));
    assert_eq!(key_line(COMPOSE, &["services", "web", "missing"]), None);
}

#[test]
fn cursor_context_gives_the_parents_the_typed_prefix_and_the_hovered_key() {
    let text = "services:\n  web:\n    image: nginx\n    heal";
    let context = cursor_context(text, text.len());
    assert_eq!(context.parents, ["services", "web"]);
    assert_eq!(context.prefix.as_deref(), Some("heal"));

    let item = "services:\n  web:\n    ports:\n      - tar";
    let context = cursor_context(item, item.len());
    assert_eq!(context.parents, ["services", "web", "ports", "0"]);
    assert_eq!(context.prefix.as_deref(), Some("tar"));
    let offset = COMPOSE.find("image").unwrap() + 2;
    let context = cursor_context(COMPOSE, offset);
    assert_eq!(
        context.hovered,
        Some(vec!["services".into(), "web".into(), "image".into()])
    );
}

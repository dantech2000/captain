use serde_json::json;

use super::{codex_has_server, has_server, with_server, without_server};

#[test]
fn connect_then_remove_gives_the_file_back_with_its_comments() {
    let zed = "// Zed settings\n{\n  \"theme\": \"One Dark\", // mine\n  \"context_servers\": {\n    \"other\": { \"command\": \"x\" },\n  },\n}\n";
    let entry = json!({ "command": "/Users/me/.captain/bin/captain", "args": ["mcp"] });
    let added = with_server(zed, "context_servers", &entry).unwrap();
    assert!(added.contains("// mine") && added.contains("\"other\""));
    assert!(has_server(&added, "context_servers").unwrap());
    assert_eq!(
        without_server(&added, "context_servers", false).unwrap(),
        zed
    );

    let plain = "{\n  \"globalShortcut\": \"\"\n}\n";
    let added = with_server(plain, "mcpServers", &entry).unwrap();
    assert_eq!(without_server(&added, "mcpServers", true).unwrap(), plain);
}

#[test]
fn remove_keeps_a_servers_object_that_holds_a_comment() {
    let zed = "{\n  \"context_servers\": {\n    // none yet\n  }\n}\n";
    let entry = json!({ "command": "/c", "args": ["mcp"] });
    let added = with_server(zed, "context_servers", &entry).unwrap();
    let removed = without_server(&added, "context_servers", true).unwrap();
    assert!(removed.contains("// none yet"), "{removed}");
    assert!(!has_server(&removed, "context_servers").unwrap());
}

#[test]
fn a_blank_file_gets_an_object_and_a_broken_one_is_refused() {
    let entry = json!({ "command": "/c", "args": ["mcp"] });
    let added = with_server("", "servers", &entry).unwrap();
    assert!(has_server(&added, "servers").unwrap());
    assert!(with_server("{ \"a\": ", "servers", &entry).is_err());
    assert!(with_server("[1]", "servers", &entry).is_err());
}

#[test]
fn codex_tables_are_found_by_name() {
    assert!(codex_has_server(
        "model = \"o3\"\n\n[mcp_servers.captain]\ncommand = \"/c\"\n"
    ));
    assert!(!codex_has_server("[mcp_servers.captain-old]\n"));
}

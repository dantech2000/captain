use std::path::Path;

use base64::Engine as _;

use super::{ClientStep, connect_step_with, remove_step_with, server_entry};
use crate::agent_clients::{AgentClient, ClientPaths, with_server};

const CAPTAIN: &str = "/Users/me/.captain/bin/captain";

fn paths() -> ClientPaths {
    ClientPaths::new(Path::new("/nonexistent-home"))
}

/// The step with no file that chezmoi manages.
fn connect(client: AgentClient, has_command: bool) -> Result<ClientStep, String> {
    let captain = Path::new(CAPTAIN);
    connect_step_with(client, &paths(), captain, has_command, &|_| false)
}

fn run(step: Result<ClientStep, String>) -> Vec<String> {
    match step {
        Ok(ClientStep::Run(argv)) => argv,
        other => panic!("not a command: {other:?}"),
    }
}

#[test]
fn installers_add_and_remove_at_user_scope() {
    let claude = run(connect(AgentClient::ClaudeCode, true));
    assert_eq!(
        claude.join(" "),
        format!("claude mcp add --scope user --transport stdio captain -- {CAPTAIN} mcp")
    );
    let gemini = run(connect(AgentClient::GeminiCli, true));
    assert_eq!(
        gemini.join(" "),
        format!("gemini mcp add --scope user captain {CAPTAIN} mcp")
    );
    let codex = run(remove_step_with(
        AgentClient::Codex,
        &paths(),
        true,
        &|_| false,
    ));
    assert_eq!(codex.join(" "), "codex mcp remove captain");
    let missing = connect(AgentClient::ClaudeCode, false);
    assert!(missing.unwrap_err().contains("claude command"));
}

#[test]
fn cursor_gets_a_link_with_the_entry_in_base64() {
    let step = connect(AgentClient::Cursor, true);
    let Ok(ClientStep::Open(link)) = step else {
        panic!("not a link: {step:?}");
    };
    let config = link.split("config=").nth(1).unwrap();
    let config = config
        .replace("%2B", "+")
        .replace("%2F", "/")
        .replace("%3D", "=");
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(config)
        .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&decoded).unwrap();
    assert_eq!(value, server_entry(AgentClient::Cursor, Path::new(CAPTAIN)));
}

#[test]
fn no_step_changes_a_file_that_chezmoi_manages() {
    let managed = |_: &std::path::Path| true;
    for client in [AgentClient::Codex, AgentClient::Cursor, AgentClient::Zed] {
        let step = connect_step_with(client, &paths(), Path::new(CAPTAIN), true, &managed);
        assert!(step.unwrap_err().starts_with("chezmoi manages"));
    }
    let step = remove_step_with(AgentClient::ClaudeCode, &paths(), true, &managed);
    assert!(step.unwrap_err().starts_with("chezmoi manages"));
}

#[test]
fn remove_keeps_a_servers_object_the_backup_shows() {
    let home = std::env::temp_dir().join(format!("captain-plan-{}", std::process::id()));
    std::fs::remove_dir_all(&home).ok();
    let paths = ClientPaths::new(&home);
    let file = paths.config_file(AgentClient::Zed).unwrap();
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    let before = "{\n  \"context_servers\": {}\n}\n";
    let entry = server_entry(AgentClient::Zed, Path::new(CAPTAIN));
    std::fs::write(
        &file,
        with_server(before, "context_servers", &entry).unwrap(),
    )
    .unwrap();
    std::fs::write(file.with_file_name("settings.json.captain-backup"), before).unwrap();
    let Ok(ClientStep::Edit { after, .. }) =
        remove_step_with(AgentClient::Zed, &paths, true, &|_| false)
    else {
        panic!("not an edit");
    };
    assert_eq!(after, before);
    std::fs::remove_dir_all(&home).ok();
}

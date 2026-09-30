use std::path::Path;

use base64::Engine as _;

use super::{ClientStep, connect_step, remove_step, server_entry};
use crate::agent_clients::{AgentClient, ClientPaths};

const CAPTAIN: &str = "/Users/me/.captain/bin/captain";

fn paths() -> ClientPaths {
    ClientPaths::new(Path::new("/nonexistent-home"))
}

fn run(step: Result<ClientStep, String>) -> Vec<String> {
    match step {
        Ok(ClientStep::Run(argv)) => argv,
        other => panic!("not a command: {other:?}"),
    }
}

#[test]
fn installers_add_and_remove_at_user_scope() {
    let captain = Path::new(CAPTAIN);
    let claude = run(connect_step(
        AgentClient::ClaudeCode,
        &paths(),
        captain,
        true,
    ));
    assert_eq!(
        claude.join(" "),
        format!("claude mcp add --scope user --transport stdio captain -- {CAPTAIN} mcp")
    );
    let gemini = run(connect_step(
        AgentClient::GeminiCli,
        &paths(),
        captain,
        true,
    ));
    assert_eq!(
        gemini.join(" "),
        format!("gemini mcp add --scope user captain {CAPTAIN} mcp")
    );
    let codex = run(remove_step(AgentClient::Codex, &paths(), true));
    assert_eq!(codex.join(" "), "codex mcp remove captain");
    let missing = connect_step(AgentClient::ClaudeCode, &paths(), captain, false);
    assert!(missing.unwrap_err().contains("claude command"));
}

#[test]
fn cursor_gets_a_link_with_the_entry_in_base64() {
    let step = connect_step(AgentClient::Cursor, &paths(), Path::new(CAPTAIN), true);
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

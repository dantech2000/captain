use std::path::PathBuf;

use super::detect;
use crate::agent_clients::{AgentClient, ClientPaths};

#[test]
fn finds_clients_by_command_or_folder_and_reads_their_servers() {
    let home = std::env::temp_dir().join(format!("captain-detect-{}", std::process::id()));
    std::fs::remove_dir_all(&home).ok();
    std::fs::create_dir_all(home.join(".gemini")).unwrap();
    std::fs::create_dir_all(home.join(".cursor")).unwrap();
    std::fs::write(
        home.join(".gemini/settings.json"),
        "{ // mine\n \"mcpServers\": { \"captain\": { \"command\": \"/c\" } } }",
    )
    .unwrap();
    let paths = ClientPaths::new(&home);
    let commands = vec![
        ("codex".to_string(), Some(PathBuf::from("/opt/bin/codex"))),
        ("claude".to_string(), None),
    ];
    let found: Vec<(AgentClient, bool, Result<bool, String>)> = detect(&paths, &commands)
        .into_iter()
        .map(|state| (state.client, state.has_command, state.connected))
        .collect();
    assert_eq!(
        found,
        [
            (AgentClient::Codex, true, Ok(false)),
            (AgentClient::GeminiCli, false, Ok(true)),
            (AgentClient::Cursor, false, Ok(false)),
        ]
    );
    std::fs::remove_dir_all(&home).ok();
}

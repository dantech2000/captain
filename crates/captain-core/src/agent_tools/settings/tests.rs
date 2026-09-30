use super::{AgentAction, AgentToolsSettings};

#[test]
fn the_gate_names_the_setting_that_keeps_a_tool_off() {
    let off = AgentToolsSettings::default();
    assert!(off.gate("help").is_ok());
    assert!(
        off.gate("logs")
            .unwrap_err()
            .contains("agent_tools.enabled")
    );
    let mut on = AgentToolsSettings {
        enabled: true,
        actions: Vec::new(),
    };
    assert!(on.gate("logs").is_ok());
    let refusal = on.gate("restart").unwrap_err();
    assert!(refusal.contains("\"restart\"") && refusal.contains("agent_tools.actions"));
    on.set_allowed(AgentAction::Restart, true);
    assert!(on.gate("restart").is_ok());
    assert!(on.gate("stop").is_err());
}

#[test]
fn unknown_actions_are_skipped_and_the_rest_kept() {
    let settings: AgentToolsSettings =
        serde_json::from_str(r#"{ "enabled": true, "actions": ["exec", "run_task"] }"#).unwrap();
    assert_eq!(settings.actions, [AgentAction::RunTask]);
}

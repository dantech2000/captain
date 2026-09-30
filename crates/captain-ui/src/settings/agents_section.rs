//! The AI agents section: one line that says whether agents may use Captain and
//! what they did last, and the button to the AI agents sheet. See
//! docs/features/0038-agent-tools.md.

use captain_core::agent_clients::client_label;
use captain_core::agent_tools::AgentToolsSettings;
use gpui_kit::*;

use super::page_section::{fill_note, row, section};
use crate::agents::{activity_time, agent_activity, open_agents_sheet};
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::primary_button;

pub fn render(settings: &AgentToolsSettings, palette: &Palette, cx: &App) -> Div {
    let last = agent_activity(cx).and_then(|watch| watch.read(cx).entries().first().cloned());
    let mut line = summary(settings);
    if let Some(entry) = last {
        line.push_str(&format!(
            " Last call: {}, {} at {}.",
            client_label(&entry.client),
            entry.summary(),
            activity_time(entry.at)
        ));
    }
    section(palette).child(
        row("AI agents", palette)
            .id("settings-agents")
            .child(fill_note(line, palette))
            .child(primary_button(
                "settings-agents-setup",
                "Set up\u{2026}",
                "Connect Claude Code, Codex, and other agents, and choose what they may do.",
                true,
                palette,
                |_, window, cx| open_agents_sheet(window, cx),
            ))
            .help("Whether AI agents can read Captain through captain mcp, and which actions they may run."),
    )
}

/// "Off." or "On. Agents may restart and run tasks."
fn summary(settings: &AgentToolsSettings) -> String {
    if !settings.enabled {
        return "Off. Agents cannot see Captain.".into();
    }
    let actions: Vec<String> = settings
        .actions
        .iter()
        .map(|action| action.label().to_lowercase())
        .collect();
    match actions.split_last() {
        None => "On, read only.".into(),
        Some((last, [])) => format!("On. Agents may {last}."),
        Some((last, [first])) => format!("On. Agents may {first} and {last}."),
        Some((last, rest)) => format!("On. Agents may {}, and {last}.", rest.join(", ")),
    }
}

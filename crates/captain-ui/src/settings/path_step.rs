//! Step 2 of the terminal setup sheet: `~/.captain/bin` first on PATH. Captain
//! adds its block to a shell file it may write; for a home-manager file it shows
//! the `home.sessionPath` line; for another file it shows the line to add. See
//! features 0035 and 0037.

use captain_core::cli_tools::{
    CliToolsSettings, PathMode, RcAccess, RcState, RcStatus, Shell, ToolPaths,
};
use gpui_kit::*;

use super::cli_tools_state::TerminalFacts;
use super::terminal_steps::{code_box, state, step, step_note};
use super::{SettingsView, store};
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, text_button};

const TITLE: &str = "Put ~/.captain/bin first on your PATH";

/// The home-manager option that prepends a folder to PATH. See
/// <https://nix-community.github.io/home-manager/options.xhtml#opt-home.sessionPath>.
const HOME_MANAGER_LINE: &str = r#"home.sessionPath = [ "$HOME/.captain/bin" ];"#;

/// How the user gets the folder on PATH.
enum PathPlan<'a> {
    Done,
    /// Captain may write these files.
    Automatic(Vec<&'a RcStatus>),
    /// home-manager or Nix writes this file.
    HomeManager(&'a RcStatus),
    /// Captain skips this file, for this reason.
    ByHand(&'a RcStatus, &'a str),
    /// No shell file at all.
    NoFile,
}

fn plan<'a>(rc: &'a [RcStatus], done: bool) -> PathPlan<'a> {
    if done {
        return PathPlan::Done;
    }
    let missing: Vec<&RcStatus> = rc
        .iter()
        .filter(|rc| rc.state == RcState::Missing)
        .collect();
    let writable: Vec<&RcStatus> = missing
        .iter()
        .copied()
        .filter(|rc| rc.access == RcAccess::Writable)
        .collect();
    if let Some(nix) = missing.iter().find(|rc| rc.access.in_nix_store()) {
        return PathPlan::HomeManager(nix);
    }
    if !writable.is_empty() {
        return PathPlan::Automatic(writable);
    }
    match missing.first() {
        Some(rc) => match &rc.access {
            RcAccess::Skip(why) => PathPlan::ByHand(rc, why),
            RcAccess::Writable => PathPlan::Automatic(vec![rc]),
        },
        None => PathPlan::NoFile,
    }
}

pub fn render(
    view: &SettingsView,
    this: &WeakEntity<SettingsView>,
    facts: &TerminalFacts,
    palette: &Palette,
) -> Div {
    let paths = ToolPaths::user();
    let tilde = |rc: &RcStatus| {
        paths.as_ref().map_or_else(
            || rc.file.path.display().to_string(),
            |paths| paths.tilde(&rc.file.path),
        )
    };
    let rc = view
        .cli_tools
        .status
        .as_ref()
        .map_or(&[][..], |status| &status.rc[..]);
    let done = facts.steps.path;
    let body = div().flex().flex_col().gap(px(8.));
    let (body, action) = match plan(rc, done) {
        PathPlan::Done => (
            body.child(step_note(
                "A new terminal finds Captain's tools first.",
                palette,
            )),
            None,
        ),
        PathPlan::Automatic(files) => {
            let names: Vec<String> = files.iter().map(|rc| tilde(rc)).collect();
            let label = match names.as_slice() {
                [one] => format!("Add to {one}"),
                _ => "Add to shell files".to_string(),
            };
            let note = format!(
                "Captain adds a marked block to {}. It changes nothing else.",
                names.join(" and ")
            );
            let button = add_button(view, this, label, palette);
            (body.child(step_note(note, palette)), Some(button))
        }
        PathPlan::HomeManager(rc) => {
            let file = tilde(rc);
            let plain = view.show_plain_line;
            let toggle = this.clone();
            let body = body
                .child(step_note(
                    format!("home-manager writes your {file}. Add this to your home-manager config, then run home-manager switch:"),
                    palette,
                ))
                .child(code_box("settings-path-home-manager", HOME_MANAGER_LINE, palette))
                .child(
                    div()
                        .id("settings-path-plain")
                        .text_size(px(12.))
                        .text_color(palette.link)
                        .cursor_pointer()
                        .on_click(move |_, _, cx| {
                            toggle
                                .update(cx, |view, cx| {
                                    view.show_plain_line = !view.show_plain_line;
                                    cx.notify();
                                })
                                .ok();
                        })
                        .child(if plain { "Hide the plain shell line" } else { "Show the line for a plain shell" })
                        .help("Show the line to add to a shell file that home-manager does not write."),
                );
            let body = match plain {
                true => body.child(code_box(
                    "settings-path-plain-line",
                    rc.file.shell.path_line(),
                    palette,
                )),
                false => body,
            };
            (body, None)
        }
        PathPlan::ByHand(rc, why) => (
            body.child(step_note(
                format!("{why} Add this line to {}:", tilde(rc)),
                palette,
            ))
            .child(code_box(
                "settings-path-line",
                rc.file.shell.path_line(),
                palette,
            )),
            None,
        ),
        PathPlan::NoFile => (
            body.child(step_note(
                "Captain found no shell file. Add this line to your shell's startup file:",
                palette,
            ))
            .child(code_box(
                "settings-path-line",
                Shell::Zsh.path_line(),
                palette,
            )),
            None,
        ),
    };
    step(
        2,
        state(done, facts.steps.links),
        TITLE,
        body,
        action,
        palette,
    )
}

/// Turns on the Automatic PATH block and installs it.
fn add_button(
    view: &SettingsView,
    this: &WeakEntity<SettingsView>,
    label: String,
    palette: &Palette,
) -> AnyElement {
    let this = this.clone();
    text_button(
        "settings-path-add",
        label,
        ButtonTone::Accent,
        !view.cli_tools.busy,
        palette,
        move |_, _, cx| {
            let tools = CliToolsSettings {
                enabled: true,
                path: PathMode::Automatic,
            };
            store::update(cx, |settings| settings.command_line_tools = tools.clone());
            this.update(cx, |view, cx| view.install_tools(tools, cx))
                .ok();
        },
    )
    .help(
        "Add Captain's marked PATH block to your shell files. captain tools uninstall removes it.",
    )
    .into_any_element()
}

//! The Command-line tools card: where a new terminal finds each tool, the links in
//! `~/.captain/bin`, the plugin folder, PATH, and the docker CLI's context. See
//! docs/features/0035-command-line-tools.md.

use captain_core::cli_tools::{self, LinkState, PathMode, SHOWN_TOOLS, ToolPaths};
use captain_core::docker_context::CAPTAIN_CONTEXT;
use captain_core::settings::Settings;
use gpui_kit::*;

use super::{SettingsView, context_actions, store};
use crate::engine_host::captain_socket;
use crate::help::HelpExt;
use crate::theme::Palette;
use crate::widgets::{ButtonTone, Segment, segmented, settings_card, settings_row, text_button};

const TITLE: &str = "Command-line tools";

pub fn render(
    view: &SettingsView,
    settings: &Settings,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> Div {
    if !cli_tools::SUPPORTED {
        let row = settings_row(TITLE, Some(cli_tools::UNSUPPORTED.into()), div(), palette)
            .id("settings-tools-unsupported")
            .help(cli_tools::UNSUPPORTED)
            .into_any_element();
        return settings_card(TITLE, [row], palette);
    }
    let paths = ToolPaths::user();
    let mut rows: Vec<AnyElement> = SHOWN_TOOLS
        .iter()
        .enumerate()
        .map(|(ix, tool)| tool_row(ix, tool, view, palette))
        .collect();
    rows.push(links_row(view, settings, paths.as_ref(), palette, cx));
    rows.push(plugins_row(view, paths.as_ref(), palette));
    rows.push(path_row(settings, palette, cx));
    rows.extend(super::cli_tools_rc::rows(
        view,
        settings,
        paths.as_ref(),
        palette,
    ));
    rows.push(context_row(view, palette, cx));
    if let Some(error) = view.cli_tools.error.clone() {
        rows.push(
            div()
                .px(px(14.))
                .py(px(8.))
                .text_size(px(11.))
                .text_color(palette.red)
                .child(error)
                .into_any_element(),
        );
    }
    settings_card(TITLE, rows, palette)
}

/// A tool and where a new terminal finds it.
fn tool_row(ix: usize, tool: &str, view: &SettingsView, palette: &Palette) -> AnyElement {
    let note = match &view.cli_tools.resolved {
        Some(Ok(tools)) => tools
            .iter()
            .find(|(name, _)| name == tool)
            .map_or("Not found".into(), |(_, source)| source.label()),
        Some(Err(error)) => error.clone(),
        None => "Checking\u{2026}".into(),
    };
    let own = !matches!(tool, "kubectl" | "helm");
    let note = match own {
        true => note,
        false => format!("{note} \u{00b7} Captain does not ship {tool}"),
    };
    settings_row(
        div()
            .font_family(palette.mono())
            .text_size(px(12.))
            .child(tool.to_string()),
        Some(note.into()),
        div(),
        palette,
    )
    .id(("settings-tool", ix))
    .help(format!("Where a new terminal finds {tool}."))
    .into_any_element()
}

/// The links' state and Relink, or Install after `captain tools uninstall`.
fn links_row(
    view: &SettingsView,
    settings: &Settings,
    paths: Option<&ToolPaths>,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let card = &view.cli_tools;
    let links = card.status.as_ref().map(|status| &status.links);
    let bin = paths.map_or("~/.captain/bin".into(), |paths| paths.tilde(&paths.bin));
    let note = match links {
        None => "Checking\u{2026}".to_string(),
        Some(links) if links.is_empty() => {
            "This Captain does not run from Captain.app, so there are no tools to link.".into()
        }
        Some(links) => {
            let wrong: Vec<String> = links
                .iter()
                .filter(|report| report.state != LinkState::Linked)
                .filter_map(|report| Some(report.link.path.file_name()?.to_string_lossy().into()))
                .collect();
            match (settings.command_line_tools.enabled, wrong.is_empty()) {
                (false, _) => "Captain does not keep the links. Install adds them.".into(),
                (true, true) => "Every link points into this Captain.app.".into(),
                (true, false) => format!("Not linked: {}.", wrong.join(", ")),
            }
        }
    };
    let enabled = settings.command_line_tools.enabled;
    let can_link = !card.busy && links.is_some_and(|links| !links.is_empty());
    let tools = settings.command_line_tools.clone();
    let button = text_button(
        "settings-tools-relink",
        match (card.busy, enabled) {
            (true, _) => "Working\u{2026}",
            (false, true) => "Relink",
            (false, false) => "Install",
        },
        ButtonTone::Accent,
        can_link,
        palette,
        cx.listener(move |view, _, _, cx| {
            let tools = cli_tools::CliToolsSettings {
                enabled: true,
                ..tools.clone()
            };
            store::update(cx, |settings| settings.command_line_tools = tools.clone());
            view.install_tools(tools, cx);
        }),
    )
    .help(if enabled {
        "Link the tools again, for example after you moved Captain.app."
    } else {
        "Link docker, Compose, and the keychain helper into ~/.captain/bin, and add Captain's plugin folder to ~/.docker/config.json."
    });
    settings_row(
        format!("Links in {bin}"),
        Some(note.into()),
        button,
        palette,
    )
    .id("settings-tools-links")
    .help(format!(
        "{bin} links docker, Compose, the credential helper, and captain into Captain.app."
    ))
    .into_any_element()
}

/// Whether the user's docker `config.json` lists Captain's plugin folder.
fn plugins_row(view: &SettingsView, paths: Option<&ToolPaths>, palette: &Palette) -> AnyElement {
    let (config, plugins) = paths.map_or(
        (
            "~/.docker/config.json".into(),
            "~/.captain/cli-plugins".into(),
        ),
        |paths| {
            (
                paths.tilde(&paths.docker_config),
                paths.tilde(&paths.plugins),
            )
        },
    );
    let note = match view.cli_tools.status.as_ref().map(|status| status.plugins) {
        None => "Checking\u{2026}".to_string(),
        Some(true) => format!(
            "{config} lists {plugins}, so docker compose and docker buildx use Captain's copies."
        ),
        Some(false) => format!("{config} does not list {plugins}. Relink adds it."),
    };
    settings_row(
        "Compose and Buildx plugins",
        Some(note.into()),
        div(),
        palette,
    )
    .id("settings-tools-plugins")
    .help("The docker CLI looks for plugins in Captain's folder before ~/.docker/cli-plugins.")
    .into_any_element()
}

/// Automatic or Manual PATH. A change applies at once.
fn path_row(settings: &Settings, palette: &Palette, cx: &mut Context<SettingsView>) -> AnyElement {
    let current = settings.command_line_tools.path;
    let this = cx.weak_entity();
    let segments = PathMode::ALL
        .into_iter()
        .map(|mode| {
            let this = this.clone();
            Segment {
                label: mode.label().into(),
                selected: current == mode,
                help: match mode {
                    PathMode::Automatic => {
                        "Captain adds ~/.captain/bin to the shell files it may change."
                    }
                    PathMode::Manual => {
                        "Captain removes its lines and shows the line for you to add."
                    }
                }
                .into(),
                on_click: Box::new(move |_, cx| {
                    let tools = cli_tools::CliToolsSettings {
                        enabled: true,
                        path: mode,
                    };
                    store::update(cx, |settings| settings.command_line_tools = tools.clone());
                    this.update(cx, |view, cx| view.install_tools(tools, cx))
                        .ok();
                }),
            }
        })
        .collect();
    settings_row(
        "PATH",
        Some(
            "Automatic adds ~/.captain/bin to your shell files. Manual shows the line to add."
                .into(),
        ),
        segmented("settings-tools-path", segments, palette),
        palette,
    )
    .id("settings-tools-path-row")
    .help("Choose who puts ~/.captain/bin on PATH: Captain or you.")
    .into_any_element()
}

/// The docker CLI's default context, and a button that makes it Captain Engine.
fn context_row(
    view: &SettingsView,
    palette: &Palette,
    cx: &mut Context<SettingsView>,
) -> AnyElement {
    let current = view
        .contexts
        .current
        .clone()
        .unwrap_or_else(|| "default".into());
    let socket = captain_socket(cx);
    let existing = view.contexts.get(CAPTAIN_CONTEXT);
    let points_here = existing.is_some_and(|context| context.host == socket);
    let done = points_here && view.contexts.is_current(CAPTAIN_CONTEXT);
    let note = match (&socket, done) {
        (None, _) => "Captain Engine is not set up, so there is no captain context to use.",
        (Some(_), true) => "New docker commands use Captain Engine.",
        (Some(_), false) => {
            "Captain creates the captain context if needed and makes it the default."
        }
    };
    let this = cx.weak_entity();
    let button = text_button(
        "settings-tools-use-captain",
        "Use Captain Engine\u{2026}",
        ButtonTone::Accent,
        socket.is_some() && !done && !view.context_change.busy,
        palette,
        move |_, window, cx| {
            if let Some(socket) = socket.clone() {
                context_actions::use_captain(this.clone(), socket, !points_here, window, cx);
            }
        },
    )
    .help("Make the captain context the docker CLI's default, and create it first if needed.");
    settings_row(
        format!("docker commands use: {current}"),
        Some(note.into()),
        button,
        palette,
    )
    .id("settings-tools-context")
    .help("The context that docker commands in a terminal use, unless DOCKER_HOST or DOCKER_CONTEXT is set.")
    .into_any_element()
}

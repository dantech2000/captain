//! `captain tools status|install|uninstall`: the tool links in `~/.captain/bin`, the
//! plugin folder, and PATH. `install` and `uninstall` save the setting, so they
//! refuse while the app runs, like `captain set`. See feature 0035.

use anyhow::{Result, bail};
use captain_core::cli_tools::{
    self, LinkState, PathMode, RcAccess, RcState, RcStatus, SHOWN_TOOLS, ToolPaths, ToolsStatus,
};
use serde_json::json;

use crate::cli::{PathArg, ToolsCommand};
use crate::context::Context;

const REFUSAL: &str = "Captain is running. Use Settings > Terminal, or quit Captain first.";

pub fn run(context: &Context, command: ToolsCommand) -> Result<()> {
    if !cli_tools::SUPPORTED {
        bail!("{}", cli_tools::UNSUPPORTED);
    }
    let Some(paths) = ToolPaths::user() else {
        bail!("cannot find the home folder");
    };
    match command {
        ToolsCommand::Status { json } => status(context, &paths, json),
        ToolsCommand::Install { path } => install(context, &paths, path),
        ToolsCommand::Uninstall => uninstall(context, &paths),
    }
}

fn install(context: &Context, paths: &ToolPaths, path: Option<PathArg>) -> Result<()> {
    let Some(bundle) = cli_tools::running_bundle() else {
        bail!("this captain does not run from Captain.app, so there are no tools to link");
    };
    let saved = context.update_settings(REFUSAL, |settings| {
        let tools = &mut settings.command_line_tools;
        tools.enabled = true;
        if let Some(path) = path {
            tools.path = match path {
                PathArg::Automatic => PathMode::Automatic,
                PathArg::Manual => PathMode::Manual,
            };
        }
        Ok(tools.clone())
    })?;
    let shell = cli_tools::login_shell();
    let errors = cli_tools::install(&bundle, paths, shell.as_deref(), saved.path);
    let status = cli_tools::status(Some(&bundle), paths, shell.as_deref());
    print_text(paths, &status, saved.path);
    report(errors)
}

fn uninstall(context: &Context, paths: &ToolPaths) -> Result<()> {
    context.update_settings(REFUSAL, |settings| {
        settings.command_line_tools.enabled = false;
        Ok(())
    })?;
    let shell = cli_tools::login_shell();
    let errors = cli_tools::uninstall(paths, shell.as_deref());
    if errors.is_empty() {
        println!(
            "Removed the links in {} and {}, the plugin folder from {}, and the PATH blocks.",
            paths.tilde(&paths.bin),
            paths.tilde(&paths.plugins),
            paths.tilde(&paths.docker_config)
        );
    }
    report(errors)
}

fn report(errors: Vec<String>) -> Result<()> {
    if errors.is_empty() {
        return Ok(());
    }
    for error in &errors {
        eprintln!("captain: {error}");
    }
    bail!("{} step(s) failed", errors.len())
}

fn status(context: &Context, paths: &ToolPaths, json: bool) -> Result<()> {
    let settings = context.load_or_default().command_line_tools;
    let bundle = cli_tools::running_bundle();
    let shell = cli_tools::login_shell();
    let status = cli_tools::status(bundle.as_ref(), paths, shell.as_deref());
    let resolved = shell
        .as_deref()
        .map(|shell| cli_tools::resolve_in_login_shell(shell, &SHOWN_TOOLS, &paths.home));
    if json {
        let tools = match &resolved {
            Some(Ok(tools)) => tools
                .iter()
                .map(|(tool, source)| (tool.clone(), json!(source.label())))
                .collect(),
            _ => serde_json::Map::new(),
        };
        let report = json!({
            "enabled": settings.enabled,
            "path": settings.path.label().to_lowercase(),
            "links": status.links.iter().map(|report| json!({
                "path": report.link.path,
                "target": report.link.target,
                "state": link_word(&report.state),
            })).collect::<Vec<_>>(),
            "plugins": status.plugins,
            "shellFiles": status.rc.iter().map(|rc| json!({
                "path": rc.file.path,
                "state": rc_word(rc, settings.path),
                "line": rc.file.shell.path_line(),
            })).collect::<Vec<_>>(),
            "tools": tools,
        });
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }
    if !settings.enabled {
        println!("Captain does not keep the tools in place. Run `captain tools install`.");
    }
    print_text(paths, &status, settings.path);
    match resolved {
        Some(Ok(tools)) => {
            println!("\nIn a new terminal:");
            for (tool, source) in tools {
                println!("  {tool:<31} {}", source.label());
            }
        }
        Some(Err(error)) => println!("\nCannot check a new terminal: {error}"),
        None => {}
    }
    Ok(())
}

fn print_text(paths: &ToolPaths, status: &ToolsStatus, mode: PathMode) {
    if status.links.is_empty() {
        println!("This captain does not run from Captain.app, so there are no tools to link.");
    } else {
        println!("Links:");
        for report in &status.links {
            let path = paths.tilde(&report.link.path);
            println!("  {path:<48} {}", link_word(&report.state));
        }
    }
    let config = paths.tilde(&paths.docker_config);
    let plugins = paths.tilde(&paths.plugins);
    match status.plugins {
        true => println!("Plugins: {config} lists {plugins}."),
        false => println!("Plugins: {config} does not list {plugins}."),
    }
    println!("PATH ({}):", mode.label().to_lowercase());
    for rc in &status.rc {
        println!("  {:<48} {}", paths.tilde(&rc.file.path), rc_word(rc, mode));
        if rc.needs_user(mode) {
            println!("    Add: {}", rc.file.shell.path_line());
        }
    }
}

fn link_word(state: &LinkState) -> String {
    match state {
        LinkState::Linked => "linked".into(),
        LinkState::Missing => "missing".into(),
        LinkState::Stale(old) => format!("points to {}", old.display()),
        LinkState::NotALink => "not a link; Captain leaves it".into(),
        LinkState::NoTarget => "not in this Captain.app".into(),
    }
}

fn rc_word(rc: &RcStatus, mode: PathMode) -> String {
    match (rc.state, &rc.access) {
        (RcState::Added, _) => "has Captain's block".into(),
        (RcState::Present, _) => "has ~/.captain/bin".into(),
        (RcState::Missing, RcAccess::Skip(why)) => format!("skipped: {why}"),
        (RcState::Missing, RcAccess::Writable) if mode == PathMode::Manual => "add the line".into(),
        (RcState::Missing, RcAccess::Writable) => "missing".into(),
    }
}

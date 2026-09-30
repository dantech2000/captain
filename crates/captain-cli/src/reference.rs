//! The Markdown reference of every command, built from the clap command tree.
//! `captain docs cli` prints it into docs/reference/cli.md, and a test checks
//! that the committed file matches.

use std::fmt::Write as _;

use clap::{Arg, ArgAction, Command, CommandFactory};

use crate::cli::Cli;

/// The regeneration command, printed in the page so readers know not to edit it.
const REGENERATE: &str = "cargo run -p captain-cli -- docs cli > docs/reference/cli.md";

/// The whole page.
pub fn markdown() -> String {
    let mut root = Cli::command();
    root.build();
    let mut out = String::new();
    out.push_str("# captain command reference\n\n");
    let _ = writeln!(
        out,
        "<!-- Generated from the clap definitions in crates/captain-cli. Do not edit. \
         Run `{REGENERATE}` after a change to the commands. -->\n"
    );
    out.push_str(
        "This page lists every `captain` command and its options. \
         [The command line](../guide/cli.md) in the user guide explains how to set it up.\n\n\
         Every command also takes `-h` or `--help`.\n\n",
    );
    options(&mut out, "Options for every command", &root, true);
    out.push_str("## Commands\n\n");
    for (path, command) in commands(&root, "captain") {
        let _ = writeln!(
            out,
            "- [`{path}`](#{}): {}",
            anchor(&path),
            summary(command)
        );
    }
    out.push('\n');
    for (path, command) in commands(&root, "captain") {
        section(&mut out, &path, command);
    }
    out.trim_end().to_string() + "\n"
}

/// The visible commands under `parent`, depth first, with their full paths.
fn commands<'a>(parent: &'a Command, prefix: &str) -> Vec<(String, &'a Command)> {
    let mut all = Vec::new();
    for command in parent.get_subcommands() {
        if command.is_hide_set() || command.get_name() == "help" {
            continue;
        }
        let path = format!("{prefix} {}", command.get_name());
        let children = commands(command, &path);
        all.push((path, command));
        all.extend(children);
    }
    all
}

fn section(out: &mut String, path: &str, command: &Command) {
    let _ = writeln!(out, "## {path}\n");
    let about = command.get_long_about().or(command.get_about());
    if let Some(about) = about {
        let _ = writeln!(out, "{}\n", sentence(&about.to_string()));
    }
    let usage = command.clone().render_usage().to_string();
    let usage = usage.trim().trim_start_matches("Usage:").trim();
    let _ = writeln!(out, "```text\n{usage}\n```\n");
    let arguments: Vec<&Arg> = visible(command, false)
        .filter(|arg| arg.is_positional())
        .collect();
    if !arguments.is_empty() {
        out.push_str("Arguments:\n\n");
        for arg in arguments {
            item(out, arg);
        }
        out.push('\n');
    }
    options(out, "Options", command, false);
}

fn options(out: &mut String, title: &str, command: &Command, global: bool) {
    let options: Vec<&Arg> = visible(command, global)
        .filter(|arg| !arg.is_positional())
        .collect();
    if options.is_empty() {
        return;
    }
    let _ = writeln!(out, "{title}:\n");
    for arg in options {
        item(out, arg);
    }
    out.push('\n');
}

/// The arguments a reader can use: no hidden ones, no `--help` or `--version`,
/// and the global ones only where they are declared.
fn visible(command: &Command, global: bool) -> impl Iterator<Item = &Arg> {
    command.get_arguments().filter(move |arg| {
        let builtin = matches!(arg.get_action(), ArgAction::Help | ArgAction::Version);
        !arg.is_hide_set() && !builtin && arg.is_global_set() == global
    })
}

fn item(out: &mut String, arg: &Arg) {
    let _ = write!(out, "- `{}`", signature(arg));
    let mut text = arg
        .get_long_help()
        .or(arg.get_help())
        .map(|help| sentence(&help.to_string()))
        .unwrap_or_default();
    let values: Vec<_> = arg
        .get_possible_values()
        .into_iter()
        .filter(|value| !value.is_hide_set())
        .collect();
    let described = values.iter().any(|value| value.get_help().is_some());
    if !values.is_empty() && !described {
        let names: Vec<String> = values
            .iter()
            .map(|value| format!("`{}`", value.get_name()))
            .collect();
        let _ = write!(text, " Values: {}.", names.join(", "));
    }
    let flag = matches!(arg.get_action(), ArgAction::SetTrue | ArgAction::SetFalse);
    let defaults: Vec<String> = arg
        .get_default_values()
        .iter()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
        .collect();
    if !flag && !defaults.is_empty() {
        let _ = write!(text, " Default: `{}`.", defaults.join(" "));
    }
    if let Some(env) = arg.get_env() {
        let _ = write!(text, " Environment variable: `{}`.", env.to_string_lossy());
    }
    let text = text.trim();
    if !text.is_empty() {
        let _ = write!(out, ": {text}");
    }
    out.push('\n');
    if described {
        for value in &values {
            let _ = write!(out, "  - `{}`", value.get_name());
            if let Some(help) = value.get_help() {
                let _ = write!(out, ": {}", sentence(&help.to_string()));
            }
            out.push('\n');
        }
    }
}

/// How the argument is typed: `<NAME>`, `[NAME]`, `-y, --yes`, or
/// `-d, --description <DESCRIPTION>`.
fn signature(arg: &Arg) -> String {
    let value = value_name(arg);
    let many = arg
        .get_num_args()
        .is_some_and(|range| range.max_values() > 1);
    if arg.is_positional() {
        let dots = if many { "..." } else { "" };
        return if arg.is_required_set() {
            format!("<{value}>{dots}")
        } else {
            format!("[{value}]{dots}")
        };
    }
    let mut names = Vec::new();
    if let Some(short) = arg.get_short() {
        names.push(format!("-{short}"));
    }
    if let Some(long) = arg.get_long() {
        names.push(format!("--{long}"));
    }
    let mut text = names.join(", ");
    if matches!(arg.get_action(), ArgAction::Set | ArgAction::Append) {
        let _ = write!(text, " <{value}>");
    }
    text
}

fn value_name(arg: &Arg) -> String {
    arg.get_value_names()
        .and_then(|names| names.first())
        .map(|name| name.to_string())
        .unwrap_or_else(|| arg.get_id().as_str().to_uppercase())
}

/// The first line of the command's help, for the list of commands.
fn summary(command: &Command) -> String {
    command
        .get_about()
        .map(|about| sentence(&about.to_string()))
        .unwrap_or_default()
}

/// The text with a closing period. clap drops the period of a one-line doc comment.
fn sentence(text: &str) -> String {
    let text = text.trim();
    if text.is_empty() || text.ends_with(['.', '"', ')']) {
        text.to_string()
    } else {
        format!("{text}.")
    }
}

/// GitHub's heading anchor for a plain heading: lowercase, spaces as `-`.
fn anchor(heading: &str) -> String {
    heading
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

#[cfg(test)]
mod tests;

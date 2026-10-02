use clap::{CommandFactory, Parser};

use super::{Cli, Command, KubernetesCommand, PathArg, SnapshotCommand, ToolsCommand};
use crate::settings_keys::SettingKey;

fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(std::iter::once("captain").chain(args.iter().copied()))
}

#[test]
fn the_command_definition_is_valid() {
    Cli::command().debug_assert();
}

#[test]
fn status_takes_json_and_a_settings_path_after_the_command() {
    let cli = parse(&["status", "--json", "--settings", "/tmp/s.json"]).expect("parses");
    assert_eq!(cli.command, Command::Status { json: true });
    assert_eq!(cli.settings.as_deref(), Some("/tmp/s.json".as_ref()));
}

#[test]
fn set_and_tools_take_only_known_words() {
    let cli = parse(&["set", "stop-engine-on-quit", "false"]).expect("parses");
    assert_eq!(
        cli.command,
        Command::Set {
            key: SettingKey::StopEngineOnQuit,
            value: "false".into()
        }
    );
    assert!(parse(&["set", "appearance", "dark"]).is_err());
    let cli = parse(&["tools", "install", "--path", "manual"]).expect("parses");
    let path = Some(PathArg::Manual);
    assert_eq!(cli.command, Command::Tools(ToolsCommand::Install { path }));
    assert!(parse(&["tools", "install", "--path", "sometimes"]).is_err());
}

#[test]
fn shell_passes_everything_after_the_separator() {
    let cli = parse(&["shell", "--", "ls", "-la", "/"]).expect("parses");
    let command = ["ls", "-la", "/"].map(String::from).to_vec();
    assert_eq!(cli.command, Command::Shell { command });
}

#[test]
fn snapshot_commands_take_a_name_and_the_overrides() {
    let cli = parse(&[
        "snapshot",
        "create",
        "base",
        "-d",
        "clean",
        "--yes",
        "--lima-home",
        "/tmp/l",
        "--instance",
        "captain-agent-test",
    ])
    .expect("parses");
    assert_eq!(
        cli.command,
        Command::Snapshot(SnapshotCommand::Create {
            name: Some("base".into()),
            description: "clean".into(),
            yes: true
        })
    );
    assert_eq!(cli.instance.as_deref(), Some("captain-agent-test"));
    assert!(parse(&["snapshot", "restore"]).is_err());
    assert_eq!(
        parse(&["snapshot", "rename", "base", "clean"])
            .unwrap()
            .command,
        Command::Snapshot(SnapshotCommand::Rename {
            name: "base".into(),
            new_name: "clean".into(),
            description: None
        })
    );
    assert!(parse(&["status", "--instance", "x"]).is_err());
}

#[test]
fn kubernetes_enable_takes_a_version_port_and_traefik_switch() {
    let cli = parse(&[
        "kubernetes",
        "enable",
        "--version",
        "v1.36.4+k3s1",
        "--port",
        "7443",
        "--traefik",
        "false",
    ])
    .expect("parses");
    assert_eq!(
        cli.command,
        Command::Kubernetes(KubernetesCommand::Enable {
            version: Some("v1.36.4+k3s1".into()),
            port: Some(7443),
            traefik: Some(false),
        })
    );
    assert!(parse(&["kubernetes", "enable", "--port", "0"]).is_err());
}

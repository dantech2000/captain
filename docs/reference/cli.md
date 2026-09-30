# captain command reference

<!-- Generated from the clap definitions in crates/captain-cli. Do not edit. Run `cargo run -p captain-cli -- docs cli > docs/reference/cli.md` after a change to the commands. -->

This page lists every `captain` command and its options. [The command line](../guide/cli.md) in the user guide explains how to set it up.

Every command also takes `-h` or `--help`.

Options for every command:

- `--settings <PATH>`: The settings file to use instead of Captain's own. Environment variable: `CAPTAIN_SETTINGS`.

## Commands

- [`captain start`](#captain-start): Start Captain Engine. The first start sets it up.
- [`captain stop`](#captain-stop): Stop Captain Engine.
- [`captain restart`](#captain-restart): Stop and start Captain Engine, for example to apply new resources.
- [`captain status`](#captain-status): Show the engine state, the Docker socket, and the versions.
- [`captain info`](#captain-info): Show the resources and the paths Captain Engine uses.
- [`captain list-settings`](#captain-list-settings): Show the settings that `captain set` can change.
- [`captain set`](#captain-set): Change one setting. Quit Captain first.
- [`captain shell`](#captain-shell): Open a shell in Captain Engine's VM, or run one command in it.
- [`captain snapshot`](#captain-snapshot): Save, list, restore, and delete snapshots of Captain Engine.
- [`captain snapshot create`](#captain-snapshot-create): Save the engine as a snapshot. The engine stops meanwhile.
- [`captain snapshot list`](#captain-snapshot-list): List the snapshots, newest first.
- [`captain snapshot restore`](#captain-snapshot-restore): Replace the engine with a snapshot. Quit Captain first.
- [`captain snapshot rename`](#captain-snapshot-rename): Rename a snapshot, or change its description. The engine keeps running.
- [`captain snapshot delete`](#captain-snapshot-delete): Delete a snapshot.
- [`captain kubernetes`](#captain-kubernetes): Turn Kubernetes (k3s in Captain Engine) on or off, show it, or reset it.
- [`captain kubernetes enable`](#captain-kubernetes-enable): Turn Kubernetes on. A running engine starts it now. Quit Captain first.
- [`captain kubernetes disable`](#captain-kubernetes-disable): Turn Kubernetes off. The cluster keeps its state. Quit Captain first.
- [`captain kubernetes status`](#captain-kubernetes-status): Show the settings and the state of the cluster.
- [`captain kubernetes reset`](#captain-kubernetes-reset): Delete the cluster's workloads and state, then start it again. Images stay.
- [`captain tools`](#captain-tools): Link docker, Compose, Buildx, and captain into ~/.captain/bin for your terminal, or show or remove the links.
- [`captain tools status`](#captain-tools-status): Show the links, the plugin folder, the shell files, and where each tool comes from in a new terminal.
- [`captain tools install`](#captain-tools-install): Link the tools, add the plugin folder to ~/.docker/config.json, and, with automatic PATH, add ~/.captain/bin to your shell files. Quit Captain first.
- [`captain tools uninstall`](#captain-tools-uninstall): Remove the links, the plugin folder, and the PATH blocks. Quit Captain first.
- [`captain docker-env`](#captain-docker-env): Print the DOCKER_HOST for Captain Engine. Use: eval "$(captain docker-env)"
- [`captain version`](#captain-version): Print the version.
- [`captain completion`](#captain-completion): Print a shell completion script.

## captain start

Start Captain Engine. The first start sets it up.

```text
captain start [OPTIONS]
```

## captain stop

Stop Captain Engine.

```text
captain stop [OPTIONS]
```

## captain restart

Stop and start Captain Engine, for example to apply new resources.

```text
captain restart [OPTIONS]
```

## captain status

Show the engine state, the Docker socket, and the versions.

```text
captain status [OPTIONS]
```

Options:

- `--json`: Print JSON.

## captain info

Show the resources and the paths Captain Engine uses.

```text
captain info [OPTIONS]
```

Options:

- `--json`: Print JSON.

## captain list-settings

Show the settings that `captain set` can change.

```text
captain list-settings [OPTIONS]
```

Options:

- `--json`: Print JSON.

## captain set

Change one setting. Quit Captain first.

```text
captain set [OPTIONS] <KEY> <VALUE>
```

Arguments:

- `<KEY>`: The setting. Values: `engine`, `cpus`, `memory`, `disk`, `stop-engine-on-quit`, `start-in-background`, `show-menu-bar-icon`, `debug-logging`.
- `<VALUE>`: The new value. `captain list-settings` shows the current ones.

## captain shell

Open a shell in Captain Engine's VM, or run one command in it.

```text
captain shell [OPTIONS] [-- <COMMAND>...]
```

Arguments:

- `[COMMAND]...`: The command to run, after `--`.

## captain snapshot

Save, list, restore, and delete snapshots of Captain Engine.

```text
captain snapshot [OPTIONS] <COMMAND>
```

## captain snapshot create

Save the engine as a snapshot. The engine stops meanwhile.

```text
captain snapshot create [OPTIONS] [NAME]
```

Arguments:

- `[NAME]`: The name. The default is the date and time.

Options:

- `-d, --description <DESCRIPTION>`: A description.
- `-y, --yes`: Do not ask before stopping the engine.

## captain snapshot list

List the snapshots, newest first.

```text
captain snapshot list [OPTIONS]
```

Options:

- `--json`: Print JSON.

## captain snapshot restore

Replace the engine with a snapshot. Quit Captain first.

```text
captain snapshot restore [OPTIONS] <NAME>
```

Arguments:

- `<NAME>`: The snapshot's name or ID.

Options:

- `-y, --yes`: Do not ask first.

## captain snapshot rename

Rename a snapshot, or change its description. The engine keeps running.

```text
captain snapshot rename [OPTIONS] <NAME> <NEW_NAME>
```

Arguments:

- `<NAME>`: The snapshot's name or ID.
- `<NEW_NAME>`: The new name.

Options:

- `-d, --description <DESCRIPTION>`: The new description. Without it, the description stays.

## captain snapshot delete

Delete a snapshot.

```text
captain snapshot delete [OPTIONS] <NAME>
```

Arguments:

- `<NAME>`: The snapshot's name or ID.

Options:

- `-y, --yes`: Do not ask first.

## captain kubernetes

Turn Kubernetes (k3s in Captain Engine) on or off, show it, or reset it.

```text
captain kubernetes [OPTIONS] <COMMAND>
```

## captain kubernetes enable

Turn Kubernetes on. A running engine starts it now. Quit Captain first.

```text
captain kubernetes enable [OPTIONS]
```

Options:

- `--version <VERSION>`: The k3s version, such as v1.36.4+k3s1. The default is the saved one, or the stable one.
- `--port <PORT>`: The port of the Kubernetes API on 127.0.0.1.
- `--traefik <TRAEFIK>`: Install Traefik on ports 80 and 443. Values: `true`, `false`.

## captain kubernetes disable

Turn Kubernetes off. The cluster keeps its state. Quit Captain first.

```text
captain kubernetes disable [OPTIONS]
```

## captain kubernetes status

Show the settings and the state of the cluster.

```text
captain kubernetes status [OPTIONS]
```

Options:

- `--json`: Print JSON.

## captain kubernetes reset

Delete the cluster's workloads and state, then start it again. Images stay.

```text
captain kubernetes reset [OPTIONS]
```

Options:

- `-y, --yes`: Do not ask first.

## captain tools

Link docker, Compose, Buildx, and captain into ~/.captain/bin for your terminal, or show or remove the links.

```text
captain tools [OPTIONS] <COMMAND>
```

## captain tools status

Show the links, the plugin folder, the shell files, and where each tool comes from in a new terminal.

```text
captain tools status [OPTIONS]
```

Options:

- `--json`: Print JSON.

## captain tools install

Link the tools, add the plugin folder to ~/.docker/config.json, and, with automatic PATH, add ~/.captain/bin to your shell files. Quit Captain first.

```text
captain tools install [OPTIONS]
```

Options:

- `--path <PATH>`: Who adds ~/.captain/bin to PATH. The default is the saved choice.
  - `automatic`: Captain adds a marked block to each shell file it may write.
  - `manual`: You add the line; `captain tools status` shows it.

## captain tools uninstall

Remove the links, the plugin folder, and the PATH blocks. Quit Captain first.

```text
captain tools uninstall [OPTIONS]
```

## captain docker-env

Print the DOCKER_HOST for Captain Engine. Use: eval "$(captain docker-env)"

```text
captain docker-env [OPTIONS]
```

## captain version

Print the version.

```text
captain version [OPTIONS]
```

## captain completion

Print a shell completion script.

```text
captain completion [OPTIONS] <SHELL>
```

Arguments:

- `<SHELL>`: The shell. Values: `bash`, `elvish`, `fish`, `powershell`, `zsh`.

# The command line

The `captain` command controls Captain Engine from a terminal. It works while the Captain app is closed, so scripts and CI jobs can use it. It is like Rancher Desktop's `rdctl`.

The [command reference](../reference/cli.md) lists every command and option. It is generated from the code, so it always matches your build.

## Install it

`Captain.app` has the command at `Captain.app/Contents/Resources/bin/captain`. To put it on your `PATH`, [set up your terminal](moving-from-docker-desktop-or-rancher.md#set-up-your-terminal). The setup links it as `~/.captain/bin/captain`.

Check it:

```sh
captain version
captain status
```

Turn on completions for zsh:

```sh
captain completion zsh > "${fpath[1]}/_captain"
```

`captain completion` also prints scripts for bash, fish, elvish, and PowerShell.

## Common tasks

| Task | Command |
|------|---------|
| Start, stop, or restart Captain Engine | `captain start`, `captain stop`, `captain restart` |
| See the engine state, the socket, and the versions | `captain status` |
| See the resources and paths | `captain info` |
| Point another tool at Captain Engine | `eval "$(captain docker-env)"` |
| Open a shell in the VM | `captain shell` |
| Run one command in the VM | `captain shell -- uname -a` |
| Change a setting | `captain set memory 8` |
| Save a snapshot | `captain snapshot create NAME` |
| Turn Kubernetes on | `captain kubernetes enable` |
| Link the tools for your terminal | `captain tools install` |

`status`, `info`, `list-settings`, `snapshot list`, `kubernetes status`, and `tools status` take `--json` for scripts.

## The app and the command together

The command and the app share the settings file and the engine. Two rules keep them safe:

- Only one of them starts or stops the engine at a time. The other one waits or says "Captain Engine is starting in another Captain process."
- Commands that change settings refuse while the app runs: `set`, `kubernetes enable` and `disable`, `snapshot restore`, and `tools install` and `uninstall`. Quit Captain first, or change the setting in the app.

`captain start` and `captain stop` work while the app runs. The app follows the new state.

## Exit codes

- `0`: the command worked.
- `1`: the command failed. The message goes to standard error.
- `2`: the command line was wrong.

## Other platforms

On Linux, the command builds and runs, but `start` and `stop` say that the system engine is not Captain's to control. Snapshots and Kubernetes need Captain Engine on macOS. On Windows, `captain tools` says that it is not supported yet.

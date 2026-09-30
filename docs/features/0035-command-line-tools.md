# Feature 0035: Command-line tools

- Milestone: M29
- Status: Implemented; checked with unit tests in temp folders
- Builds on: [0021](0021-packaging.md) (the bundled tools), [0022](0022-command-line.md) (the `captain` CLI), and [0026](0026-contexts-and-remote-hosts.md) (Docker contexts)

## Goal

The user's own terminal uses the tools inside `Captain.app`: `docker`, Compose, Buildx, the macOS credential helper, and `captain`. Then the user can uninstall Rancher Desktop, whose `~/.rd/bin` links provide these tools today. Rancher calls this "PATH management" ([Rancher Desktop: Environment](https://docs.rancherdesktop.io/ui/preferences/application/environment)).

## In scope

- **The credential helper.** `scripts/fetch-tools.sh` downloads `docker-credential-osxkeychain` 0.9.9 from [docker/docker-credential-helpers releases](https://github.com/docker/docker-credential-helpers/releases/tag/v0.9.9). `scripts/tool-versions.env` pins the SHA-256 of both Mac builds, copied from the release's [`checksums.txt`](https://github.com/docker/docker-credential-helpers/releases/download/v0.9.9/checksums.txt). Its MIT license goes to `licenses/credential-helpers/LICENSE`. `bundle-macos.sh` copies it to `Contents/Resources/bin`. A `config.json` with `"credsStore": "osxkeychain"` needs it for pulls and `docker login`.
- Captain's own `docker` runs also find the bundled helper: `DockerCli` puts `Contents/Resources/bin` first on the `PATH` of the child, and Captain's registry login lookup checks the bundle first.
- **Tool links.** `~/.captain/bin` holds symlinks into the running `Captain.app`:

  | Link | Target in `Contents/Resources` |
  |------|------|
  | `~/.captain/bin/docker` | `bin/docker` |
  | `~/.captain/bin/docker-compose` | `cli-plugins/docker-compose` (the plugin binary also runs on its own) |
  | `~/.captain/bin/docker-credential-osxkeychain` | `bin/docker-credential-osxkeychain` (macOS only) |
  | `~/.captain/bin/captain` | `bin/captain` |
  | `~/.captain/cli-plugins/docker-compose` | `cli-plugins/docker-compose` |
  | `~/.captain/cli-plugins/docker-buildx` | `cli-plugins/docker-buildx` |

  The folder path stays the same when the user moves `Captain.app`, so only the links change. At each start, the app checks each link with `readlink`. It replaces a link that points elsewhere or is broken, and creates a missing one. It never replaces a file or folder that is not a symlink. A link whose target is not in the bundle (a build with `CAPTAIN_SKIP_TOOLS=1`) is skipped.
- **Plugins for the user's docker CLI.** Captain adds `~/.captain/cli-plugins` to `cliPluginsExtraDirs` in the user's `config.json` (`$DOCKER_CONFIG/config.json` or `~/.docker/config.json`). The CLI searches the extra folders before `~/.docker/cli-plugins` and takes the first match: [`getPluginDirs`](https://github.com/docker/cli/blob/master/cli-plugins/manager/manager.go#L45-L55) appends `CLIPluginsExtraDirs` first, and [`getPlugin`](https://github.com/docker/cli/blob/master/cli-plugins/manager/manager.go#L108-L120) uses `paths[0]`. It also skips broken plugin symlinks, so the old `~/.docker/cli-plugins` links into `~/.rd/bin` do no harm after Rancher is gone.
  - Every other key stays, in its order. The file keeps the CLI's tab indentation.
  - A symlinked `config.json` stays a link: the write goes to its target, like the settings and kubeconfig writers.
  - Before the first change, Captain copies the file to `config.json.captain-backup`. It never overwrites that backup.
  - A file that is not valid JSON is left alone, with an error.
- **Opt-in.** Captain changes no shell file and no docker file until the user clicks Install in Settings or runs `captain tools install`. Rancher links its tools at the first start; Captain asks first, because these are the user's own files.
- **PATH: Automatic or Manual.** A setting, Manual by default. Manual shows the line to add. Automatic adds this block to the end of each shell file:

  ```sh
  # >>> captain >>>
  # Added by Captain. Set PATH to Manual in Captain's Settings to remove it.
  export PATH="$HOME/.captain/bin:$PATH"
  # <<< captain <<<
  ```

  Fish gets `fish_add_path --global --move --path $HOME/.captain/bin` in `~/.config/fish/conf.d/captain.fish`. The files:
  - zsh: `~/.zshrc`, if it exists or the login shell is zsh.
  - bash: `~/.bash_profile`, else `~/.bashrc`, if one exists or the login shell is bash (then Captain creates `~/.bash_profile`).
  - fish: `conf.d/captain.fish`, if `~/.config/fish` exists or the login shell is fish.

  Captain writes a file only if it is a regular, writable file, or a missing file in a writable folder. It skips a symlink (for example a home-manager link into `/nix/store`), a file that `chezmoi source-path` says chezmoi manages, and a file it cannot write. For a skipped file, Settings shows the line to add, with a Copy button. Manual shows the lines for every file. Switching to Manual removes the blocks. A file that already names `.captain/bin` counts as done.
  Rancher's own block uses `### MANAGED BY RANCHER DESKTOP START (DO NOT EDIT)` markers ([manageLinesInFile.ts](https://github.com/rancher-sandbox/rancher-desktop/blob/main/pkg/rancher-desktop/integrations/manageLinesInFile.ts)); Captain's block is separate and comes later in the file, so `~/.captain/bin` wins while both exist.
- **Settings > Command-line tools.** One card:
  - A row per tool (`docker`, `docker-compose`, `docker-credential-osxkeychain`, `captain`, and, for information, `kubectl` and `helm`) with what a new terminal runs: "Rancher Desktop (~/.rd/bin)", "Captain", "Docker Desktop", "Homebrew", "Nix", or the path. Captain runs `$SHELL -lic 'command -v …'` once, on a background thread, with a 10 s limit.
  - The link state and a **Relink** button (**Install** after `captain tools uninstall`).
  - The plugin folder state in `config.json`.
  - **PATH**: Automatic or Manual, and a row per shell file with its state, and the line and a Copy button when the user must add it.
  - **docker commands use: \<context\>** and **Use Captain Engine…**, which asks, then creates the `captain-engine` context if needed (`docker context create`) and makes it the default (`docker context use`), with the code from [0026](0026-contexts-and-remote-hosts.md).
- **CLI:** `captain tools status [--json]`, `captain tools install [--path automatic|manual]`, and `captain tools uninstall`. `install` and `uninstall` save the setting, so they refuse while the app runs, like `captain set`. `uninstall` removes the links, the plugin folder from `config.json`, and the blocks, and the app then leaves them alone.
- **Platforms.** macOS is the target. On Linux the same code runs, but packages do not bundle tools yet, so there is nothing to link. On Windows the card and the CLI say "Command-line tools are not supported on Windows yet."

## Out of scope

- Bundling `kubectl` and `helm`. The user has `kubectl` from Nix and `helm` from Homebrew. Rancher links both; a follow-up adds them for parity.
- Replacing links in `~/.docker/cli-plugins`. `cliPluginsExtraDirs` comes first, so they need no change.
- `$ZDOTDIR`, `~/.profile`, csh, and tcsh. The app starts without the shell's environment, so it cannot see `ZDOTDIR`.
- `DOCKER_HOST` or `DOCKER_CONTEXT` set in a shell file. They win over the default context; the card does not read them.

## Notes

- The link check reads each link with `readlink` and does nothing when all links are right, so it runs at every start.
- Captain finds `chezmoi` on `PATH` and in the Homebrew and Nix profile folders, and treats a zero exit of [`chezmoi source-path <file>`](https://www.chezmoi.io/reference/commands/source-path/) as "managed".
- `fish_add_path` is in fish 3.2 and later ([fish_add_path](https://fishshell.com/docs/current/cmds/fish_add_path.html)).
- Rancher's path manager writes `.bash_profile` (or `.bash_login`, `.profile`), `.bashrc`, `.zshrc`, `.cshrc`, `.tcshrc`, and fish `config.fish` ([pathManagerImpl.ts](https://github.com/rancher-sandbox/rancher-desktop/blob/main/pkg/rancher-desktop/integrations/pathManagerImpl.ts)).
- The docker CLI writes `config.json` with tab indentation ([configfile/file.go](https://github.com/docker/cli/blob/master/cli/config/configfile/file.go)).

## Verification

1. Run `cargo test -p captain-core cli_tools`. The tests use temp folders: the link plan (create, relink after a move, keep a regular file), the shell block add and remove, the skip rules (a link into a fake `/nix/store`, a read-only file), and the `config.json` merge that keeps the other keys and their order.
2. Run `scripts/bundle-macos.sh` and check that `Captain.app/Contents/Resources/bin/docker-credential-osxkeychain` exists.
3. Open the bundled app. Check that `~/.captain/bin` and `~/.captain/cli-plugins` hold the links, and that `~/.docker/config.json` lists `~/.captain/cli-plugins` first in `cliPluginsExtraDirs`.
4. Open Settings > Command-line tools. The rows show where each tool comes from. With a chezmoi or home-manager `~/.zshrc`, the PATH row shows the line and a Copy button.
5. Add the line, open a new terminal, and click Relink. The `docker` row says Captain. Run `docker compose version`, `docker buildx version`, and `docker pull` of a private image.
6. Click **Use Captain Engine…** and confirm. `docker context ls` marks `captain-engine` with `*`.
7. Move `Captain.app` and open it. The links point at the new place.
8. Quit Captain and run `captain tools uninstall`. The links, the plugin folder entry, and the blocks are gone.

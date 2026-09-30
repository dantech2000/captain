# Feature 0035: Command-line tools

- Milestone: M29
- Status: Implemented; checked with unit tests in temp folders
- Builds on: [0021](0021-packaging.md) (the bundled tools), [0022](0022-command-line.md) (the `captain` CLI), and [0026](0026-contexts-and-remote-hosts.md) (Docker contexts)

## Goal

The user's own terminal uses the tools inside `Captain.app`: `docker`, Compose, Buildx, the macOS credential helper, `kubectl`, `helm`, and `captain`. Then the user can uninstall Rancher Desktop, whose `~/.rd/bin` links provide these tools today. Rancher calls this "PATH management" ([Rancher Desktop: Environment](https://docs.rancherdesktop.io/ui/preferences/application/environment)).

## In scope

- **The credential helper.** `scripts/fetch-tools.sh` downloads `docker-credential-osxkeychain` 0.9.9 from [docker/docker-credential-helpers releases](https://github.com/docker/docker-credential-helpers/releases/tag/v0.9.9). `scripts/tool-versions.env` pins the SHA-256 of both Mac builds, copied from the release's [`checksums.txt`](https://github.com/docker/docker-credential-helpers/releases/download/v0.9.9/checksums.txt). Its MIT license goes to `licenses/credential-helpers/LICENSE`. `bundle-macos.sh` copies it to `Contents/Resources/bin`. A `config.json` with `"credsStore": "osxkeychain"` needs it for pulls and `docker login`.
- **kubectl and Helm.** `scripts/fetch-tools.sh` downloads both, and `tool-versions.env` pins the SHA-256 of both Mac builds. `bundle-macos.sh` copies them to `Contents/Resources/bin`. Their Apache-2.0 licenses go to `licenses/kubectl` and `licenses/helm`.
  - `kubectl` 1.37.1 from `https://dl.k8s.io/release/v1.37.1/bin/darwin/<arm64|amd64>/kubectl`. The sums come from `kubectl.sha256` next to each binary ([Install kubectl on macOS](https://kubernetes.io/docs/tasks/tools/install-kubectl-macos/)). `kubectl` works with a server one minor version older or newer ([version skew policy](https://kubernetes.io/releases/version-skew-policy/#kubectl)). The k3s channel server says stable is v1.36.4+k3s1 and latest is v1.37.0+k3s1 ([k3s channels](https://update.k3s.io/v1-release/channels)), and [`dl.k8s.io/release/stable.txt`](https://dl.k8s.io/release/stable.txt) says v1.37.1. So 1.37 covers the default k3s version and the newest one. The picker also offers k3s back to 1.29; those need the user's own `kubectl`.
  - Helm 4.3.0 from `https://get.helm.sh/helm-v4.3.0-darwin-<arm64|amd64>.tar.gz`. The sums come from `<archive>.sha256sum` on get.helm.sh ([Helm releases](https://github.com/helm/helm/releases/tag/v4.3.0), [Installing Helm](https://helm.sh/docs/intro/install/)). Helm 4 is the current major version. Helm 3 gets only maintenance releases (3.22.0 shipped the same day as 4.3.0). Helm 4.3 supports Kubernetes 1.34 to 1.37 ([Helm version skew](https://helm.sh/docs/topics/version_skew/)).
- Captain's own `docker` runs also find the bundled helper: `DockerCli` puts `Contents/Resources/bin` first on the `PATH` of the child, and Captain's registry login lookup checks the bundle first.
- **Tool links.** `~/.captain/bin` holds symlinks into the running `Captain.app`:

  | Link | Target in `Contents/Resources` |
  |------|------|
  | `~/.captain/bin/docker` | `bin/docker` |
  | `~/.captain/bin/docker-compose` | `cli-plugins/docker-compose` (the plugin binary also runs on its own) |
  | `~/.captain/bin/docker-credential-osxkeychain` | `bin/docker-credential-osxkeychain` (macOS only) |
  | `~/.captain/bin/captain` | `bin/captain` |
  | `~/.captain/bin/kubectl` | `bin/kubectl` |
  | `~/.captain/bin/helm` | `bin/helm` |
  | `~/.captain/cli-plugins/docker-compose` | `cli-plugins/docker-compose` |
  | `~/.captain/cli-plugins/docker-buildx` | `cli-plugins/docker-buildx` |

  The folder path stays the same when the user moves `Captain.app`, so only the links change. At each start, the app checks each link with `readlink`. It replaces a link that points elsewhere or is broken, and creates a missing one. It never replaces a file or folder that is not a symlink. A link whose target is not in the bundle (a build with `CAPTAIN_SKIP_TOOLS=1`) is skipped.
- **Plugins for the user's docker CLI.** Captain adds `~/.captain/cli-plugins` to `cliPluginsExtraDirs` in the user's `config.json` (`$DOCKER_CONFIG/config.json` or `~/.docker/config.json`). The CLI searches the extra folders before `~/.docker/cli-plugins` and takes the first match: [`getPluginDirs`](https://github.com/docker/cli/blob/master/cli-plugins/manager/manager.go#L45-L55) appends `CLIPluginsExtraDirs` first, and [`getPlugin`](https://github.com/docker/cli/blob/master/cli-plugins/manager/manager.go#L108-L120) uses `paths[0]`. It also skips broken plugin symlinks, so the old `~/.docker/cli-plugins` links into `~/.rd/bin` do no harm after Rancher is gone.
  - Every other key stays, in its order. The file keeps the CLI's tab indentation.
  - A symlinked `config.json` stays a link: the write goes to its target, like the settings and kubeconfig writers.
  - Before the first change, Captain copies the file to `config.json.captain-backup`. It never overwrites that backup.
  - Captain writes a new file with a unique name (`.config.json.<pid>-<n>.tmp`), syncs it, and renames it over the old one. Only the owner can read it, because it can hold registry logins.
  - Captain's own writers, the app and the CLI, take `~/.captain/docker-config.lock` first, so they never write at the same time.
  - Right before the rename, Captain reads the file again. If another program changed it meanwhile (for example `docker login` or `docker context use`), Captain drops its new file and merges again from the new text, up to three times. Then it reports an error. A write that lands between that last read and the rename can still be lost; the docker CLI takes no lock that Captain could share.
  - A file that is not valid JSON is left alone, with an error.
- **Opt-in.** Captain changes no shell file and no docker file until the user clicks Install in Settings or runs `captain tools install`. Rancher links its tools at the first start; Captain asks first, because these are the user's own files.
- **PATH: Automatic or Manual.** A setting, Manual by default. Manual shows the line to add. Automatic adds this block to the end of each shell file:

  ```sh
  # >>> captain >>>
  # Added by Captain. Set command_line_tools.path to manual in Captain's settings file to remove it.
  export PATH="$HOME/.captain/bin:$PATH"
  # <<< captain <<<
  ```

  Fish gets `fish_add_path --global --move --path $HOME/.captain/bin` in `~/.config/fish/conf.d/captain.fish`. The files:
  - zsh: `~/.zshrc`, if it exists or the login shell is zsh.
  - bash: `~/.bash_profile`, else `~/.bashrc`, if one exists or the login shell is bash (then Captain creates `~/.bash_profile`).
  - fish: `conf.d/captain.fish`, if `~/.config/fish` exists or the login shell is fish.

  Captain writes a file only if it is a regular, writable file, or a missing file in a writable folder. It skips a symlink (for example a home-manager link into `/nix/store`), a file that `chezmoi source-path` says chezmoi manages, and a file it cannot write. For a skipped file, Settings shows the line to add, with a Copy button. Manual shows the lines for every file. Switching to Manual, and `captain tools uninstall`, remove the blocks under the same rules: for a file that Captain skips, the result says why and asks the user to remove the lines from `# >>> captain >>>` to `# <<< captain <<<`. A file that already names `.captain/bin` counts as done.
  Captain writes a shell file through a synced temporary file next to it and a rename, with the file's permissions, so a full disk or a crash never leaves it empty or half written. Before its first change to a file, Captain copies it to `<name>.captain-backup` (for example `~/.zshrc.captain-backup`), and never overwrites that backup. Captain's own fish file has no backup.
  Rancher's own block uses `### MANAGED BY RANCHER DESKTOP START (DO NOT EDIT)` markers ([manageLinesInFile.ts](https://github.com/rancher-sandbox/rancher-desktop/blob/main/pkg/rancher-desktop/integrations/manageLinesInFile.ts)); Captain's block is separate and comes later in the file, so `~/.captain/bin` wins while both exist.
- **Settings > Command-line tools.** One card (since [0037](0037-settings-page.md): the Terminal line and its setup sheet; PATH Automatic or Manual is in the settings file):
  - A row per tool (`docker`, `docker-compose`, `docker-credential-osxkeychain`, `captain`, `kubectl`, and `helm`) with what a new terminal runs: "Rancher Desktop (~/.rd/bin)", "Captain", "Docker Desktop", "Homebrew", "Nix", or the path. Captain runs `$SHELL -lic 'command -v …'` once, on a background thread, with a 10 s limit.
  - The link state and a **Relink** button (**Install** after `captain tools uninstall`).
  - The plugin folder state in `config.json`.
  - **PATH**: Automatic or Manual, and a row per shell file with its state, and the line and a Copy button when the user must add it.
  - **docker commands use: \<context\>** and **Use Captain Engine…**, which asks, then creates the `captain-engine` context if needed (`docker context create`) and makes it the default (`docker context use`), with the code from [0026](0026-contexts-and-remote-hosts.md).
- **CLI:** `captain tools status [--json]`, `captain tools install [--path automatic|manual]`, and `captain tools uninstall`. `install` and `uninstall` save the setting, so they refuse while the app runs, like `captain set`. `uninstall` removes the links, the plugin folder from `config.json`, and the blocks, and the app then leaves them alone.
- **Platforms.** macOS is the target. On Linux the same code runs, but packages do not bundle tools yet, so there is nothing to link. On Windows the card and the CLI say "Command-line tools are not supported on Windows yet."

## Out of scope

- A `kubectl` that follows the k3s version. Rancher Desktop downloads one per Kubernetes version. Captain ships one, which fits the stable and latest k3s channels.
- Replacing links in `~/.docker/cli-plugins`. `cliPluginsExtraDirs` comes first, so they need no change.
- `$ZDOTDIR`, `~/.profile`, csh, and tcsh. The app starts without the shell's environment, so it cannot see `ZDOTDIR`.
- `DOCKER_HOST` or `DOCKER_CONTEXT` set in a shell file. They win over the default context; the card does not read them.

## Notes

- With `~/.captain/bin` first on `PATH`, Captain's `kubectl` and `helm` win over copies from Nix or Homebrew, as Rancher's `~/.rd/bin` links did. To prefer another copy, put its folder before `~/.captain/bin` on `PATH`. A removed link comes back at the next start.
- The link check reads each link with `readlink` and does nothing when all links are right, so it runs at every start.
- Captain finds `chezmoi` on `PATH` and in the Homebrew and Nix profile folders, and treats a zero exit of [`chezmoi source-path <file>`](https://www.chezmoi.io/reference/commands/source-path/) as "managed".
- `fish_add_path` is in fish 3.2 and later ([fish_add_path](https://fishshell.com/docs/current/cmds/fish_add_path.html)).
- Rancher's path manager writes `.bash_profile` (or `.bash_login`, `.profile`), `.bashrc`, `.zshrc`, `.cshrc`, `.tcshrc`, and fish `config.fish` ([pathManagerImpl.ts](https://github.com/rancher-sandbox/rancher-desktop/blob/main/pkg/rancher-desktop/integrations/pathManagerImpl.ts)).
- The docker CLI writes `config.json` with tab indentation ([configfile/file.go](https://github.com/docker/cli/blob/master/cli/config/configfile/file.go)).

## Verification

1. Run `cargo test -p captain-core cli_tools`. The tests use temp folders: the link plan (create every link, including `kubectl` and `helm`, relink after a move, keep a regular file), the shell block add and remove with its backup and permissions, the skip rules for adding and removing (a link into a fake `/nix/store`, a read-only file), the `config.json` merge that keeps the other keys and their order, and a `config.json` change by another program during Captain's write, which Captain keeps.
2. Run `scripts/bundle-macos.sh` and check that `Captain.app/Contents/Resources/bin/` holds `docker-credential-osxkeychain`, `kubectl`, and `helm`. `bin/kubectl version --client` prints v1.37.1, and `bin/helm version` prints v4.3.0.
3. Open the bundled app. Check that `~/.captain/bin` and `~/.captain/cli-plugins` hold the links, and that `~/.docker/config.json` lists `~/.captain/cli-plugins` first in `cliPluginsExtraDirs`.
4. Open Settings > Terminal > Set up…. The rows show where each tool comes from. With a chezmoi or home-manager `~/.zshrc`, the PATH row shows the line and a Copy button.
5. Add the line, open a new terminal, and click Relink. The `docker`, `kubectl`, and `helm` rows say Captain. Run `docker compose version`, `docker buildx version`, and `docker pull` of a private image.
6. Click **Use Captain Engine…** and confirm. `docker context ls` marks `captain-engine` with `*`.
7. Move `Captain.app` and open it. The links point at the new place.
8. Quit Captain and run `captain tools uninstall`. The links, the plugin folder entry, and the blocks are gone.

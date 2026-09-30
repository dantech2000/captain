# Feature 0021: Packaging

- Milestone: M9
- Status: In progress
- Related: [ADR 0005](../adr/0005-compose-via-cli.md), [ADR 0008](../adr/0008-captain-engine.md), [feature 0013](0013-captain-engine.md)

## Goal

Captain ships as a download that works on its own. On macOS, `Captain.app` carries `limactl`, the `docker` CLI, Compose, and Buildx, so Captain Engine and Compose projects work without Homebrew, Rancher Desktop, or Docker Desktop. A version tag builds the release packages.

## In scope

- **`scripts/tool-versions.env`**: the one place that pins the tool versions, and the SHA-256 sums of the docker CLI archives.

  | Tool | Version | Download | Checksum |
  |------|---------|----------|----------|
  | Lima | 2.2.0 | `lima-<v>-Darwin-<arm64\|x86_64>.tar.gz` from [Lima releases](https://github.com/lima-vm/lima/releases) | `SHA256SUMS` in the release |
  | docker CLI | 29.8.1 | `docker-<v>.tgz` from [download.docker.com/mac/static/stable](https://download.docker.com/mac/static/stable/) | Pinned in `tool-versions.env` (Docker publishes none) |
  | Compose | 5.5.1 | `docker-compose-darwin-<aarch64\|x86_64>` from [Compose releases](https://github.com/docker/compose/releases) | `checksums.txt` in the release |
  | Buildx | 0.37.1 | `buildx-v<v>.darwin-<arm64\|amd64>` from [Buildx releases](https://github.com/docker/buildx/releases) | `checksums-signed.txt` (macOS), else `checksums.txt` |
  | docker-credential-osxkeychain | 0.9.9 | `docker-credential-osxkeychain-v<v>.darwin-<arm64\|amd64>` from [credential helper releases](https://github.com/docker/docker-credential-helpers/releases) | Pinned in `tool-versions.env`, from the release's `checksums.txt`; see [0035](0035-command-line-tools.md) |
| kubectl | 1.37.1 | `kubectl` from `dl.k8s.io/release/v<v>/bin/darwin/<arm64\|amd64>/` ([install page](https://kubernetes.io/docs/tasks/tools/install-kubectl-macos/)) | Pinned in `tool-versions.env`, from `kubectl.sha256` next to the binary; see [0035](0035-command-line-tools.md) |
| Helm | 4.3.0 | `helm-v<v>-darwin-<arm64\|amd64>.tar.gz` from [get.helm.sh](https://github.com/helm/helm/releases/tag/v4.3.0) | Pinned in `tool-versions.env`, from `<archive>.sha256sum` on get.helm.sh |

- **`scripts/fetch-tools.sh [darwin-arm64|darwin-x86_64]`** downloads the tools, checks each SHA-256 sum, and fails on a mismatch. It writes `target/tools/<platform>/` in the bundle layout: `lima/bin/limactl`, `lima/share/lima`, `bin/docker`, `bin/docker-credential-osxkeychain`, `bin/kubectl`, `bin/helm`, and `cli-plugins/docker-{compose,buildx}`. It deletes each archive after it extracts it. A `VERSIONS` stamp makes a second run a no-op. The folder is under `target/`, which git ignores.
- **`scripts/bundle-macos.sh`** runs `fetch-tools.sh` for the Mac's architecture and copies the tools into `Captain.app/Contents/Resources`. `CAPTAIN_SKIP_TOOLS=1` leaves them out.
- **Tool lookup.** `captain_core::tools` has the order, with unit tests:
  1. The bundled copy. `Bundle::from_exe` finds `Contents/Resources` when the executable is `*.app/Contents/MacOS/captain`.
  2. `PATH`.
  3. Known install folders: Homebrew for `limactl`; `/usr/local/bin`, Homebrew, `/usr/bin`, Docker Desktop, `~/.docker/bin`, `~/.orbstack/bin`, and `~/.rd/bin` for `docker`.

  A `cargo run` build has no bundle, so it uses steps 2 and 3, as before.
- **Compose and Buildx plugins.** With bundled plugins, Captain runs `docker` with `DOCKER_CONFIG=~/.captain/docker`. Captain writes `~/.captain/docker/config.json` from the user's `config.json` (`$DOCKER_CONFIG` or `~/.docker`):
  - The bundled `cli-plugins` folder comes first in `cliPluginsExtraDirs`, then the user's extra folders, then the user's own `cli-plugins` folder.
  - Credentials, credential helpers, and proxy settings stay, so private pulls work.
  - `currentContext` goes, because Captain sets `DOCKER_HOST`.
  - The file is mode 0600, because it can hold registry tokens.

  This never writes the user's `~/.docker`. Only the command-line tools of [0035](0035-command-line-tools.md) add Captain's plugin folder to the user's `config.json`. Compose actions and the Diagnostics CLI checks use the same binary and config.
- **macOS release: `scripts/package-macos.sh`** takes `target/release/Captain.app` and does these steps:
  1. It signs the app when `CAPTAIN_SIGN_IDENTITY` is set.
  2. It builds `Captain-<version>-<arch>.dmg` with `hdiutil`. The image has an Applications link.
  3. It notarizes and staples the `.dmg` when the Apple credentials are set.

  Without a signing identity it makes an unsigned `.dmg`.
- **Linux and Windows packages: cargo-packager** config in `crates/captain-app/Cargo.toml` (`[package.metadata.packager]`) for `.deb`, AppImage, and `.msi` (WiX). These packages do not bundle the tools. Linux uses the system `dockerd` and the distribution's `docker` CLI. Windows has no engine yet.
- **`.github/workflows/release.yml`** runs only on `v*` tags. It has four jobs:
  - macOS (arm64, `macos-15`): fetch the tools, bundle, sign if secrets exist, build the `.dmg`, and notarize if secrets exist.
  - Linux: `cargo packager --release --formats deb,appimage`.
  - Windows: `cargo packager --release --formats wix`.
  - Release: a draft GitHub release with every package attached.

## Out of scope

- An Intel `.dmg` in CI. `fetch-tools.sh darwin-x86_64` works, and a `macos-15-intel` job can come later.
- Bundling tools in the Linux and Windows packages.
- A universal (arm64 + x86_64) app. Lima, Compose, and Buildx ship one binary per architecture, and a universal app would double the tool size.
- Auto-update.
- Putting the bundled `docker` on the user's `PATH` (M14's administrative access or M20's command line).
- A `.dmg` background image and window layout.
- Higher-resolution icons for Linux and Windows. `assets/icon/brand.png` (160 px) is the only committed PNG; `scripts/build-icons.sh` needs `rsvg-convert`, which the Windows runner lacks.

## Notes

- **Why `cliPluginsExtraDirs` and not the user's config.** The CLI searches `cliPluginsExtraDirs` first, then `<config dir>/cli-plugins`, then system folders. No environment variable adds a folder ([`getPluginDirs` in docker/cli](https://github.com/docker/cli/blob/master/cli-plugins/manager/manager.go)). Rancher Desktop edits `~/.docker/config.json` for the same reason. Captain gives the CLI its own config folder instead, so it never changes the user's files. Compose finds Buildx for `build` with the same lookup (`manager.GetPlugin("buildx", ...)` in [compose/pkg/compose/build_bake.go](https://github.com/docker/compose/blob/main/pkg/compose/build_bake.go)). Without Buildx, Compose falls back to the classic builder.
- **Checked by hand:** `DOCKER_CONFIG=<dir with that config> Captain.app/Contents/Resources/bin/docker compose version` prints `v5.5.1`, and `docker buildx version` prints `v0.37.1`. Neither needs `PATH`. `compose ls` against Captain Engine lists the running projects.
- **Lima needs `share/lima` next to `bin/`.** Captain's template uses `base: template:_images/ubuntu-lts`, and `limactl` finds templates in `../share/lima/templates` ([Lima internals](https://lima-vm.io/docs/dev/internals/)). With `PATH=/usr/bin:/bin` and an empty `LIMA_HOME`, the bundled `limactl template copy template:_images/ubuntu-lts -` prints the template. The fetch script drops `libexec/lima` (the krunkit driver and the MCP server, 34 MB) and the macOS guest agent: Captain uses `vz`, which is built into `limactl`.
- **Size.** For arm64 the tools use 170 MB unpacked: `limactl` 32 MB, `share/lima` 7.5 MB, `docker` 41 MB, Compose 30 MB, and Buildx 62 MB. A debug `.dmg` is 115 MB. `kubectl` 1.37.1 (58 MB) and Helm 4.3.0 (61 MB) add about 120 MB unpacked.
- **Docker's static binaries have no checksum file.** The [binaries install page](https://docs.docker.com/engine/install/binaries/) lists none, so `tool-versions.env` pins the sums. To bump the version, hash the new archive: `curl -fsSL <url> | shasum -a 256`. The macOS archive holds only the client. It has no Compose or Buildx, as the same page says.
- **Buildx 0.37 moved the macOS sums.** The signed macOS binaries are in `checksums-signed.txt`, not `checksums.txt`. The script looks in both.
- **Licenses.** Lima, the docker CLI, Compose, Buildx, kubectl, and Helm are all Apache-2.0; the credential helpers are MIT. `fetch-tools.sh` downloads their `LICENSE` files (and the docker CLI's `NOTICE`) from each repository at the pinned tag, and the bundle puts them in `Contents/Resources/licenses`.
- **Why not cargo-packager for macOS.** cargo-packager builds its own `.app` from its config, and its `resources` list is the same on every platform ([config](https://docs.rs/cargo-packager/latest/cargo_packager/config/struct.Config.html)). Captain's `.app` already comes from `bundle-macos.sh`, with the Info.plist and the per-architecture tools. A second way to build the same app would drift, so the `.dmg` comes from `hdiutil`, which ships with macOS. For Linux and Windows, cargo-packager puts resources in `/usr/lib/<name>/` (deb, AppImage) or next to the `.exe` (WiX) ([resource resolver](https://github.com/crabnebula-dev/cargo-packager/blob/main/crates/resource-resolver/src/lib.rs)), which is enough for later.
- **`hdiutil create` prints a deprecation warning on macOS 27** and suggests `diskutil image create`. It still works, and the `macos-15` runner does not warn. Switch when runners move to macOS 27.
- **Unsigned builds.** The binaries keep the ad-hoc or upstream signatures they came with. `limactl` keeps Lima's ad-hoc signature with `com.apple.security.virtualization`. Buildx keeps Docker's Developer ID signature. An unsigned `.dmg` from the internet is quarantined. To open it, the user right-clicks Captain and chooses Open, or allows it in System Settings > Privacy & Security.

## Code signing and notarization

All secrets are optional. Without them, the workflow still builds an unsigned `.dmg`.

| Secret | Use |
|--------|-----|
| `MACOS_CERTIFICATE` | A base64 `.p12` export of a "Developer ID Application" certificate and its key |
| `MACOS_CERTIFICATE_PASSWORD` | The `.p12` password |
| `MACOS_SIGN_IDENTITY` | The identity name, for example `Developer ID Application: Jane Doe (TEAMID1234)` |
| `APPLE_ID` | The Apple ID email for `notarytool` |
| `APPLE_TEAM_ID` | The 10-character team ID |
| `APPLE_APP_PASSWORD` | An app-specific password for that Apple ID |

Steps (`scripts/package-macos.sh`):

1. The workflow imports the certificate into a temporary keychain.
2. The script signs `limactl` with `scripts/macos/limactl.entitlements`. That file is a copy of [Lima's `vz.entitlements`](https://github.com/lima-vm/lima/blob/master/vz.entitlements). Without the virtualization entitlement, `vz` cannot start a VM.
3. The script signs each file in `bin/` (`docker`, the credential helper, `kubectl`, `helm`, and `captain`) and `cli-plugins/`, then the app. Each signature uses `--options runtime --timestamp`. Notarization requires the hardened runtime and a secure timestamp on every executable ([Apple: resolving common notarization issues](https://developer.apple.com/documentation/security/resolving-common-notarization-issues)). The script signs inside out and does not use `--deep`.
4. The script signs the `.dmg` and submits it with `xcrun notarytool submit --wait` ([Apple: customizing the notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow)). Then it runs `xcrun stapler staple`.

Windows signing (an Authenticode certificate for the `.msi`) is not set up. Unsigned `.msi` files show a SmartScreen warning.

## Verification

1. Check free space with `df -h /`. The tools need about 200 MB.
2. Run `scripts/fetch-tools.sh darwin-arm64`. It prints the versions and `target/tools/darwin-arm64`. No archives remain. A second run says "up to date".
3. Change one sum in `tool-versions.env`, then run `rm target/tools/darwin-arm64/VERSIONS` and the script again. The script stops with "checksum mismatch".
4. Run `scripts/bundle-macos.sh`. Then check the two bundled tools:
   - `Captain.app/Contents/Resources/bin/docker version --format '{{.Client.Version}}'` prints `29.8.1`.
   - `Captain.app/Contents/Resources/lima/bin/limactl --version` prints `limactl version 2.2.0`.
5. With Rancher Desktop and Homebrew's `docker` and `lima` off `PATH`, open `Captain.app`. Diagnostics shows docker 29.8.1, Compose v5.5.1, and Lima 2.2.0. `~/.captain/docker/config.json` exists, and `~/.docker/config.json` is unchanged.
6. On a Compose project card, click Restart. It works.
7. Run `scripts/bundle-macos.sh release`, then `scripts/package-macos.sh`. The script prints `target/release/Captain-<version>-arm64.dmg`, which opens to Captain and an Applications link.
8. `cargo test -p captain-core tools` passes.

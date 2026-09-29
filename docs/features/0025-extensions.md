# Feature 0025: Extensions

- Milestone: M21
- Status: Implemented on macOS; the page and Update need a check by hand in the app
- Parity: Docker Desktop [extensions](https://docs.docker.com/extensions/extensions-sdk/) and the Rancher Desktop [Extensions](https://docs.rancherdesktop.io/ui/extensions) page
- Decision: [ADR 0011](../adr/0011-extensions.md)

## Goal

A user can install a Docker Desktop extension by its image reference, open its page in a Captain window, and remove it again. The page talks to Captain through `window.ddClient`, so extensions built with the Docker SDK work without changes.

## In scope

- An **Extensions** page. Its sidebar entry sits above Snapshots. The page has:
  - An Install card with an image reference field and an **Install** button. Enter works too. Under the field, an orange note says: "Extensions run code from their publisher on this computer and in the engine, with your permissions. Install only extensions you trust."
  - The installed list, sorted by title. Each row shows the title, the description, the image, and the publisher, with **Open** and **Remove…**.
  - The header shows the count, or the running step ("Pulling …", "Installing …", "Removing …") with a spinner.
- **Install** has two steps.
  1. Captain pulls the image if the engine does not have it, reads its labels and its image ID, and reads `/metadata.json` from that image ID. An image without the `com.docker.desktop.extension.api.version` label is refused.
  2. A dialog asks before anything runs. It shows the image, the publisher (`org.opencontainers.image.vendor`, or "Unknown publisher" in orange), the website, whether the extension has a page, its backend, and its host binaries in orange, and repeats the trust note. **Install** then:
     - refuses an ID that has an `extension.json` already, with "<title> is already installed from <image>. Use Update to replace it." A folder without `extension.json` is left from a stopped install, and Captain deletes it,
     - copies `ui.dashboard-tab.root` to `~/.captain/extensions/<id>/ui`,
     - copies the `darwin` host binaries to `~/.captain/extensions/<id>/bin` and marks them executable (`linux` or `windows` on those platforms),
     - starts the backend, if any,
     - refuses to start when the folder has an `extension.json` that Captain cannot read, or when `.backup/<id>` is left from an update that did not finish, and names the path. A folder with no `extension.json` is left from an install that stopped, and Captain deletes it.
  - writes `extension.json` last, with the engine it was installed on. A failed install removes the folder and the backend's containers. It removes the backend's volumes only when the project had no containers or volumes before the install.
  - Files come from the inspected image ID, not the tag, so a pull or a re-tag while the dialog is open cannot change what gets installed. The backend runs that image ID too when `vm.image` is the extension's own image.
- **Update…** on a row asks for a tag, `latest` by default. **Check** pulls the extension's repository with that tag and compares the image ID with the installed one (`extension.json` records it; for older installs Captain reads it from the installed image before the pull). If the pull fails but the engine has that image, Captain uses it, so a locally built extension can update too.
  - The same image: a toast says the extension is up to date.
  - Another image: the install dialog asks again, titled "Update <title>?". It shows the installed and the new image with their `org.opencontainers.image.version` labels, in orange with "an older version" when the new version number is lower. It also lists the new image's backend and host binaries, since they may change.
  - **Update** closes the extension's window, copies the new UI, host binaries, and Compose files to `~/.captain/extensions/.update/<id>`, stops the old backend without `--volumes`, moves the old `ui`, `bin`, and `compose` folders and `extension.json` to `~/.captain/extensions/.backup/<id>`, moves the new ones in, writes `extension.json`, and starts the new backend. The ID stays the same. The backend's volumes and any other files in the extension's folder stay. Only after the new backend starts does Captain delete the backup and remove the old image: its old tag, or its ID when the pull moved the tag. A failure there only logs.
  - If a step after the staging fails, Captain stops the new backend, puts the old files back, and starts the old backend on its image ID. If that fails too, the error says so and the backup stays. An update refuses to start while `.backup/<id>` exists, and names the folder.
- **IDs.** The list reads only a manifest whose `id` is its folder's name. Captain builds paths and the Compose project name from the ID and deletes them on Remove, so an edited `extension.json` cannot point them elsewhere.
- **Engines.** `extension.json` records the engine as the `DOCKER_HOST` value or the `ssh://` URL from Settings. The page lists only the extensions of the connected engine. Update and Remove refuse an extension of another engine: "<title> was installed on <engine>. Connect to that engine to update or remove it." Installs from before this change have no engine; every engine lists them, and an update records the current one.
- **Remove…** asks first. It closes the extension's window, runs `compose down --volumes` on the backend, deletes the folder and any `.backup/<id>` and `.update/<id>` left from an update, and removes the image. Open is off while an install, update, or removal runs, and Update and Remove close the extension's window again when they end.
- **Open** opens the extension in its own window, or brings the open one forward. The window has a GPUI title bar, a toast strip, and a `gpui-wry` web view. Open is off on Linux and Windows, and the page says why.
- The bridge answers the first SDK subset of ADR 0011: `extension.vm.service.get|post|put|patch|delete|head|request`, `extension.vm.cli.exec`, `extension.host.cli.exec`, `docker.cli.exec` (all three with `stream`), `docker.listContainers`, `docker.listImages`, `desktopUI.toast.success|warning|error`, `desktopUI.dialog.showOpenDialog`, `host.openExternal`, and `host.platform|arch|hostname`.

## Out of scope

- `desktopUI.navigate.*` (an empty object, as in Rancher Desktop), and the deprecated v0 calls. Any other method rejects with "`<method>` is not supported by Captain".
- The marketplace, extension icons, and an allow list of images.
- Opening extension pages on Linux and Windows.
- `x-rd-install` and the other Rancher Desktop hooks.

## Notes

- **Metadata.** `metadata.json` has optional `icon`, `ui`, `vm`, and `host` sections ([metadata](https://docs.docker.com/extensions/extensions-sdk/architecture/metadata/)). `vm` has an `image` or a `composefile`, and `exposes.socket`. `host.binaries` is a list of objects keyed by `darwin`, `linux`, and `windows`. `${DESKTOP_PLUGIN_IMAGE}` in `vm.image` means the extension image. Labels follow [extension labels](https://docs.docker.com/extensions/extensions-sdk/extensions/labels/).
- **IDs.** The ID is the repository without tag or digest, in lowercase, with other characters than letters and digits as `-`, then `-` and 8 hex digits of a 64-bit FNV-1a hash of the repository: `docker/disk-usage-extension` becomes `docker-disk-usage-extension-<hash>`. The hash keeps `acme/foo-bar` and `acme/foo_bar` apart. The ID names the folder, the `captain-ext://<id>/` host, and the Compose project `captain-ext-<id>`. Older installs keep their ID without a hash.
- **Copying files.** Captain creates a container of the image ID named `<id>-captain-copy` with the command `captain-copy`, which never starts, so images built `FROM scratch` work. It reads each path with the archive endpoint (`GET /containers/{id}/archive`) and removes the container. Only regular files and folders are unpacked; links are skipped, so no file points outside the folder.
- **Backend.** Captain writes `compose/captain-compose.json` and runs `docker compose -p captain-ext-<id> -f captain-compose.json up -d`. `compose down` runs in the same folder with the same `-f`, or in the temp folder when the file is missing, because without `-f` Compose looks for a file in the working folder and its parents. Each `compose` step is killed after 15 minutes. For `vm.image`, the project has one `backend` service. For `vm.composefile`, Captain copies the file's folder out of the image, runs `docker compose config --format json` with `DESKTOP_PLUGIN_IMAGE` set, and edits the result. Every service mounts the `guest-services` volume at `/run/guest-services` and restarts `unless-stopped`. With `exposes.socket`, a `captain-proxy` service (`alpine/socat:1.8.1.3`) forwards `127.0.0.1::8080` to the socket, as Rancher Desktop does with its own proxy ([extensions.ts](https://github.com/rancher-sandbox/rancher-desktop/blob/main/pkg/rancher-desktop/main/extensions/extensions.ts)). `vm.service` calls go to that published port as HTTP/1.1 with `Connection: close`; `vm.cli.exec` runs `docker exec` in the first backend service.
- **Exec.** `docker.cli.exec` runs the `docker` CLI with `DOCKER_HOST` set to the connected engine. The page's `env` is applied first and `DOCKER_HOST` after it, and `DOCKER_CONTEXT` is removed, in all three scopes. `docker.cli.exec` and `vm.cli.exec` refuse a `cmd` that is empty or starts with `-`, because the CLI reads global options such as `--host`, `-H`, `--context`, and `--config` only before the subcommand ([docker.go](https://github.com/docker/cli/blob/master/cmd/docker/docker.go) sets `SetInterspersed(false)`). They also refuse any `DOCKER_*` variable in `env` except `DOCKER_BUILDKIT`, `DOCKER_CLI_HINTS`, and `DOCKER_DEFAULT_PLATFORM`, since `DOCKER_CONTEXT`, `DOCKER_CONFIG`, and the TLS variables can pick another daemon ([environment variables](https://docs.docker.com/reference/cli/docker/#environment-variables)). `cwd` stays open: the page can name any host path in the arguments anyway. `host.cli.exec` runs the extension's own program, so only its `DOCKER_HOST` is fixed. `host.cli.exec` runs only a file named in the extension's `bin` folder, never a path. Captain drops one pair of matching quotes around each argument, because Docker Desktop runs these through a shell and extensions write `"{{json .}}"`. A plain exec resolves with `{cmd, code, stdout, stderr}` and the `lines`, `parseJsonLines`, and `parseJsonObject` helpers, and rejects on a non-zero exit. A streaming exec sends one reply per line, then the exit code; `close()` drops the call, which kills the process.
- **List calls** return the Engine API's own JSON (bollard's `ContainerSummary` and `ImageSummary`). `filters` may be a JSON string or an object; a value may be a list or a map to `true`.
- **Web view.** GPUI Kit has no web view feature flag. The web view is the separate `gpui-wry` 0.7 crate on Longbridge's `lb-wry` 0.53.3; its only feature, `inspector`, turns on wry devtools ([WebView docs](https://github.com/longbridge/gpui-kit/blob/main/website/docs/webview.md), [example](https://github.com/longbridge/gpui-kit/blob/main/examples/webview/src/main.rs)). Captain builds the child view with `build_as_child` on the GPUI window, and sets devtools in debug builds only. The dependencies are for macOS only, so Linux CI needs no WebKitGTK.
- **Custom scheme.** `captain-ext://<id>/` serves files from the `ui` folder with a content type from the file extension. A path that leaves the folder gets 403; another host gets 404.
- **Isolation.** Each extension gets its own WebKit data store from `with_data_store_identifier`, with 16 bytes hashed from the ID (macOS 14 and newer; older systems use the default store). The navigation handler allows only the extension's own scheme and `about:`. Other links and `window.open` go to the system browser, and only `http` and `https` links open.
- **Bridge.** An init script (`captain-core/src/extension/bridge/shim.js`) defines `window.ddClient` and the `__ddMuiV5Themes` and `__ddMuiV6Themes` globals that `@docker/docker-mui-theme` needs, as Rancher Desktop does ([preload](https://github.com/rancher-sandbox/rancher-desktop/blob/main/pkg/rancher-desktop/preload/extensions.ts)). Each call posts `{id, method, params}` with `window.ipc.postMessage`. The window parses it, takes the extension from the window, and answers with `window.__captainBridge.resolve|reject|output|exit` through `evaluate_script`. The value types follow `@docker/extension-api-client-types` 0.4.2.
- **Toasts.** GPUI overlays cannot draw above the web view, so `desktopUI.toast` shows in a strip under the title bar. Success and warning go away after 5 seconds; errors stay until closed.
- **Code layout.** `captain_core::extension` has the metadata, labels, IDs, paths, the Compose project, the bridge messages and replies, the HTTP helpers, the shim, and the `ExtensionManager` trait. `captain-docker/src/extensions` has `DockerExtensions`. `captain-ui/src/extensions` has the page, its model and dialogs, and the macOS window in `window/`. The workspace gets the manager with the engine connection.

## Verification

Automated (`cargo test -p captain-core -p captain-docker`):

- Metadata parsing, labels, IDs (two repositories with the same slug get different IDs), the pinned image, the UI path mapping and content types, and the Compose project.
- The list holds only the extensions of the connected engine and older installs. An update's file swap rolls back to the old files, also when the new backend wrote its own `compose` folder.
- Bridge routing: each method's route, exec scopes and quote removal, service paths, unknown methods, list filters, the open panel options, and that Captain accepts every method the shim posts.
- Exec policy: a global option as `cmd` and daemon `DOCKER_*` variables are refused, and `DOCKER_HOST` wins over the page's `env`. Install refuses an unreadable manifest or a leftover backup until Remove, and the list skips a manifest whose ID is not its folder.
- Replies, the host binary check, HTTP requests and responses (chunked and bare LF), and tar unpacking.
- The update reference (same repository, new tag) and the version comparison.

Live: `cargo test -p captain-docker --test live_extensions -- --ignored` on Captain Engine.

- Docker's Disk Usage extension (`docker/disk-usage-extension:0.2.9`, UI only), re-tagged `captain-agent-ext-ui`: install, `docker.cli.exec` plain and streaming, `docker.listImages`, a refused host binary, a refused `--host` before the subcommand, and remove.
- A backend extension built in the test, `captain-agent-ext-vm`: socat answers HTTP on `/run/guest-services/backend.sock`. It checks `vm.service` through the proxy and `vm.cli.exec`. A second install of the same repository is refused and the backend still answers. A manager for another engine does not list it and cannot remove it. An update to `captain-agent-ext-vm:broken`, whose backend image does not exist, fails; the old files, manifest, and backend come back and answer. Then it removes the extension.

`cargo test -p captain-docker --test live_extension_host -- --ignored` (macOS) builds `captain-agent-ext-host:test` from scratch: a page and a `darwin` shell script `captain-agent-hello`. It installs it, checks that `bin/captain-agent-hello` is executable, runs it through `extension.host.cli.exec` in the bridge, and checks that a full path is refused. A check with the tag `test` says it is up to date. It then builds `captain-agent-ext-host:latest` (version 2.0.0), which exists only in the engine, checks for an update with the default tag, updates, runs the new script, checks that a file in the extension's folder stays and that the old image is gone, and removes the extension. Both live tests passed on a test VM.

A throwaway example opened the Disk Usage window with the web view on macOS: the page loaded from `captain-ext://`, ran `docker system df` through the bridge, and drew its chart.

By hand in the app, on macOS with Captain Engine:

1. Open **Extensions**. The page is empty and shows the trust note.
2. Type `docker/disk-usage-extension:0.2.9` and press Enter. The header shows the pull. The dialog names Docker Inc. and says the extension has a page. Click **Install**. The row appears.
3. Click **Open**. A window titled "Disk usage" shows the chart. Click **Open** again; the same window comes forward.
4. Click **Update…**, keep `latest`, and click **Check**. If Docker Hub has a newer image, the dialog shows both versions; click **Update**. Otherwise a toast says the extension is up to date.
5. Click **Remove…** and confirm. The window closes, the row goes away, and `docker images` no longer lists the extension.

Still open: a check against a public extension with host binaries, the third extension of the ADR's test set, and a live update of an extension with a backend.

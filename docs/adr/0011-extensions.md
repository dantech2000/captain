# ADR 0011: Docker Desktop extensions in a separate web view window

- Status: Accepted
- Date: 2026-09-29

## Context

M21 asks for Docker Desktop extensions. An extension is a Docker image with a `metadata.json` at its root. It has up to three parts, all optional ([extension architecture](https://docs.docker.com/extensions/extensions-sdk/architecture/), [metadata](https://docs.docker.com/extensions/extensions-sdk/architecture/metadata/)):

- **`ui`:** a web app (HTML, JavaScript, CSS) that Docker Desktop shows as a dashboard tab. The page calls the host through a global client object, `ddClient`, from `@docker/extension-api-client`.
- **`vm`:** one backend image, or a Compose file, that runs in the engine VM. The backend listens on a Unix socket in `/run/guest-services`, and the UI calls it with `ddClient.extension.vm.service.get('/path')`. Docker recommends a socket over a TCP port to avoid port clashes ([backend tutorial](https://docs.docker.com/extensions/extensions-sdk/build/backend-extension-tutorial/)).
- **`host`:** binaries for `darwin`, `linux`, and `windows` that Docker Desktop copies to the host at install. The UI runs them with `ddClient.extension.host.cli.exec(name, args)` ([invoke host binaries](https://docs.docker.com/extensions/extensions-sdk/guides/invoke-host-binaries/)).

The UI part is the problem. Captain draws everything with GPUI (ADR 0001), and GPUI has no HTML engine. Zed has an open request for web views ([zed#21208](https://github.com/zed-industries/zed/issues/21208)). In that thread, a GPUI Kit maintainer reports that a web view always draws on top of GPUI elements.

Rancher Desktop runs extensions because it is an Electron app. It loads each UI in its own Electron session through a custom protocol, and a preload script defines `ddClient` ([extensions.ts](https://github.com/rancher-sandbox/rancher-desktop/blob/e4c91fe/pkg/rancher-desktop/main/extensions/extensions.ts), [preload/extensions.ts](https://github.com/rancher-sandbox/rancher-desktop/blob/e4c91fe/pkg/rancher-desktop/preload/extensions.ts)). Its `ddClient.desktopUI.navigate` is an empty object, so Rancher ships without the navigation part of the SDK and extensions still work.

### Web views in a Rust app

`wry` is the web view crate from Tauri. It uses WebKit (WKWebView) on macOS, WebView2 on Windows, and WebKitGTK on Linux. It needs a running event loop and a window that implements `HasWindowHandle` from `raw-window-handle`. `build_as_child` places the web view inside another window, and works on macOS, Windows, and Linux under X11 only. Wayland needs `build_gtk` with a GTK container ([wry README](https://github.com/tauri-apps/wry)).

GPUI owns the main thread's event loop, and so do `tao` and `winit`. Running a `tao` event loop next to GPUI's is not an option. What does work is `build_as_child` on a GPUI window, because a GPUI window gives a raw window handle.

GPUI Kit already has this. Its `gpui-wry` 0.7.0 crate wraps a `wry` child view in a GPUI entity ([GPUI Kit WebView docs](https://github.com/longbridge/gpui-kit/blob/main/website/docs/webview.md)). The docs mark it experimental, with these limits:

- Only macOS and Windows. The Linux path is unfinished.
- The native view sits above GPUI's surface. GPUI popovers, dialogs, menus, and tooltips cannot draw over it in the same rectangle.
- On Windows, the example sets `GPUI_DISABLE_DIRECT_COMPOSITION=true` so the child view renders.
- It depends on a Longbridge fork, `lb-wry` 0.53.3.

Other known issues: `build_as_child` crashed on macOS 26 with Objective-C exceptions in `wry` 0.54.4 ([wry#1705](https://github.com/tauri-apps/wry/issues/1705), closed). WebKitGTK text blurs after a resize on Wayland ([wry#1727](https://github.com/tauri-apps/wry/issues/1727), open).

### Options

| Option | How it works | Fit with Captain | Verdict |
|--------|--------------|------------------|---------|
| A. Skip extensions | Captain has no extension support. Users who need one keep Docker Desktop or Rancher Desktop. | No new code, no web engine. M21 stays open, and one reason to keep Docker Desktop stays too. | Rejected for now |
| B. A separate native window with a web view | Each extension opens in its own GPUI window whose only content is a `gpui-wry` child view. A script injected before the page loads defines `ddClient`, and `wry`'s IPC handler carries its calls to Captain. | Native WebKit on macOS. No GPUI overlays cross the web view, because the window has none. The main window stays pure GPUI. Linux waits for `gpui-wry`. | Chosen |
| C. The system browser and a local server | Captain serves the extension UI on `127.0.0.1` with a `ddClient` shim, and opens it in the default browser. | Works on every platform today. But the page runs in the user's browser, next to every other site. Any page can send requests to `127.0.0.1`, so the bridge needs a secret token in the URL. The extension leaves Captain's window. | Rejected, kept as a fallback idea for Linux |

Option B inside the main window (a dashboard tab, as Docker Desktop does) was also considered. The overlay limit rules it out today: Captain's command palette, dialogs, and toasts would draw under the extension. Two unmerged PRs try to fix overlay order ([gpui-kit#2626](https://github.com/longbridge/gpui-kit/pull/2626), [zed#62379](https://github.com/zed-industries/zed/pull/62379)). If one lands, extensions can move into a tab.

## Decision

Captain hosts extension UIs in a separate window with a native web view (option B), on macOS first.

### The window

- The sidebar gets an **Extensions** page, drawn in GPUI. It lists installed extensions with Open, Update, and Remove, and has an "Install extension…" field that takes an image reference.
- Open creates, or brings forward, one window per extension. The window's title bar is GPUI. The rest is one `gpui-wry` view.
- Each extension gets its own web data store, so extensions cannot read each other's cookies or storage. WKWebView has no data folder option. `wry`'s `with_data_store_identifier` sets a store per identifier on macOS 14 and newer ([wry src/lib.rs](https://github.com/tauri-apps/wry/blob/dev/src/lib.rs)). Captain derives the 16-byte identifier from the extension id. We confirm that the pinned `lb-wry` has this call.
- The window loads the UI from a custom URL scheme, `captain-ext://<id>/`, through `wry`'s `with_custom_protocol`. It serves the files Captain copied out of the image, and needs no TCP port.
- A navigation handler allows only the extension's own scheme. Every other link goes to `host.openExternal`, which opens the system browser.
- Devtools come only in debug builds, or behind the `inspector` feature.

### The bridge

- Before the page loads, an init script defines `window.ddClient`. Each call becomes a JSON message posted with `window.ipc.postMessage`.
- Captain handles the message on the tokio side (ADR 0002). It sends the reply back with `evaluate_script`, which resolves the pending Promise in the page. Streaming calls send one message per output line.
- Captain checks every message: the extension id comes from the window, never from the message. Unknown methods get a clear error: `"<method> is not supported by Captain"`.

### Install and backends

- **Install:** Captain pulls the image into Captain Engine, reads `metadata.json`, and copies `ui.root` and the host binaries for `darwin` out of a stopped container. Host binaries go to `~/.captain/extensions/<id>/bin`.
- **Backend (`vm`):** Captain starts the image or the Compose file as a Compose project named `captain-ext-<id>`, with a volume mounted at `/run/guest-services`. The backend's socket is inside the guest, so Captain reaches it through a small proxy container in the same project. The proxy publishes the socket on a random `127.0.0.1` port. Rancher Desktop reaches backends through a published port too ([manager.ts](https://github.com/rancher-sandbox/rancher-desktop/blob/e4c91fe/pkg/rancher-desktop/main/extensions/manager.ts)).
- **Trust:** installing an extension runs its code on the host and in the engine. The install dialog says so, shows the image publisher from its labels, and lists the host binaries.

### The SDK subset for the first release

The `@docker/extension-api-client-types` 0.4.2 package defines the v1 client. Captain implements these calls first:

| Area | Calls | How Captain answers |
|------|-------|---------------------|
| Backend | `extension.vm.service.get`, `post`, `put`, `patch`, `delete`, `head`, `request` | HTTP to the backend through the proxy |
| Backend CLI | `extension.vm.cli.exec` | `docker exec` in the backend container |
| Host binaries | `extension.host.cli.exec`, with the `stream` option | Runs the binary from `~/.captain/extensions/<id>/bin` |
| Docker | `docker.cli.exec` with `stream`, `docker.listContainers`, `docker.listImages` | The `docker` CLI and `bollard`, both pointed at the current engine |
| UI | `desktopUI.toast.success`, `warning`, `error` | Captain's GPUI notifications |
| UI | `desktopUI.dialog.showOpenDialog` | The native open panel |
| Host | `host.openExternal`, `host.platform`, `host.arch`, `host.hostname` | System browser and plain values |

Not in the first release:

- `desktopUI.navigate.*` (jump to a container or image page). The object exists with no methods, the same as Rancher Desktop, so feature checks in extensions still work. It comes next, because Captain has these pages.
- The deprecated v0 calls (`ddClient.backend`, `execDockerCmd`, `toastSuccess`, and others). The reference marks them deprecated ([DockerDesktopClient](https://docs.docker.com/reference/api/extensions-sdk/DockerDesktopClient/)).
- The Docker extension marketplace. Users install by image reference.
- Extensions on Linux and Windows. The Extensions page on those platforms says so.

### Test set

We check the bridge against three public extensions before M21 is done: one UI-only extension, one with a `vm` backend, and one with host binaries. The feature spec names them.

## Consequences

- Captain can run many Docker Desktop extensions on macOS, in their own windows.
- Captain gains a web engine dependency. On macOS it is the system WebKit, so the app does not grow much. The web view uses memory only while an extension window is open.
- Captain depends on an experimental crate and a `wry` fork. We pin both and upgrade on purpose, as with GPUI Kit (ADR 0001). We check the macOS 26 crash from wry#1705 on the pinned version before we ship.
- Extensions that call `desktopUI.navigate` or v0 APIs get a clear error until Captain adds them.
- Extensions do not look like part of the main window. When GPUI gets overlay ordering above native views, a later ADR can move them into a tab.
- Linux and Windows users get no extensions until `gpui-wry` supports Linux and Captain has a Windows engine. Option C stays possible for Linux.
- Extensions run third-party code on the host. The install dialog is the only guard, the same as in Docker Desktop.

## Changes during implementation

Recorded when M21 was built ([feature 0025](../features/0025-extensions.md)):

- GPUI Kit has no web view feature flag. `gpui-wry` 0.7 is a separate crate; its `inspector` feature turns on wry devtools. Captain depends on `gpui-wry`, `lb-wry` 0.53.3, and `raw-window-handle` on macOS only.
- Toasts do not use Captain's GPUI notifications. Those draw as overlays, which the web view would cover, so the extension window shows toasts in a strip under its title bar.
- The first release has no Update button. Installing the same image again replaces the extension.
- Install pulls the image only when the engine does not have it, so a locally built extension installs without a registry.
- The proxy is `alpine/socat:1.8.1.3`, a service named `captain-proxy` in the backend's project.
- The test set has a UI-only extension (Docker's Disk Usage) and a backend extension built by the live test. A public extension with host binaries is still to check.
- The window opened without a crash on the pinned `lb-wry` on this Mac, so wry#1705 did not show up.

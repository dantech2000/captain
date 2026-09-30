# Captain

A native desktop client for Docker, written in Rust with [GPUI Kit](https://gpui-kit.com).

Captain connects to a Docker engine that is already running (Docker Desktop, OrbStack, Colima, Rancher Desktop, or plain `dockerd`) and gives you a fast GUI for containers, images, volumes, and networks. It runs on macOS, Linux, and Windows.

> Status: early development. See [ROADMAP.md](ROADMAP.md) for what works today.

To install and use Captain, read the [user guide](docs/guide/README.md). The rest of this file is for building Captain.

## Build

You need Rust 1.98 or later. `rust-toolchain.toml` pins the version.

Platform requirements:

- macOS 15+ with Xcode Command Line Tools (`xcode-select --install`).
- Windows 10+ with Visual Studio 2022 Build Tools (Desktop C++ workload) and CMake.
- Linux (Ubuntu 24.04 example):

  ```sh
  sudo apt install -y gcc g++ clang libfontconfig-dev libwayland-dev \
    libwebkit2gtk-4.1-dev libxkbcommon-x11-dev libx11-xcb-dev \
    libssl-dev libzstd-dev vulkan-validationlayers libvulkan1
  ```

Then:

```sh
cargo run -p captain-app
```

## Which engine does Captain use?

By default Captain runs its own engine, **Captain Engine**: a small Linux VM with Docker that Captain starts, stops, and configures. On macOS it uses [Lima](https://lima-vm.io) 2.2 or newer. Until release builds bundle it (M9), install Lima with `brew install lima`. The VM lives in `~/.captain/lima` and never touches your own Lima or Colima VMs. See [ADR 0008](docs/adr/0008-captain-engine.md) and [feature 0013](docs/features/0013-captain-engine.md).

On the first launch, Captain offers to set up Captain Engine or to use an engine you already have. You can switch at any time in Settings. With **Other engine**, Captain connects to an engine but does not control it. It checks these in order and uses the first one it finds:

1. The custom endpoint saved in Settings.
2. The `DOCKER_HOST` environment variable.
3. The current `docker context` (from `~/.docker/config.json`).
4. Known sockets for Docker Desktop, OrbStack, Colima, Rancher Desktop, and `/var/run/docker.sock`. On Windows it uses the `docker_engine` named pipe.

Other engine is the default when Lima is not installed, or when you saved a custom endpoint in an earlier version. On Linux, Captain uses the system `dockerd`.

## Project layout

| Crate | Purpose |
|-------|---------|
| `captain-core` | Domain models, the `Engine` trait, and state stores. No UI or Docker dependencies. |
| `captain-docker` | The `Engine` implementation for the Docker API, built on `bollard`. |
| `captain-host` | Captain Engine: the Lima VM on macOS, behind the `EngineHost` trait. |
| `captain-ui` | GPUI views. |
| `captain-app` | The binary. Opens the window and wires everything together. |

Design decisions live in [docs/adr](docs/adr). Each feature has a short spec in [docs/features](docs/features).

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.

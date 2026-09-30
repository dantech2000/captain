# Settings reference

<!-- Generated from the settings types in crates/captain-core. Do not edit by hand.
     Run `CAPTAIN_BLESS=1 cargo test -p captain-core reference` after a change. -->

Captain keeps its settings in `settings.json`:

- macOS: `~/Library/Application Support/Captain/settings.json`
- Linux: `~/.config/Captain/settings.json`
- Windows: `%APPDATA%\Captain\settings.json`

The file holds only the settings you change. Every other key has the default listed here, so a new default in a later version reaches you. The file may have `//` and `/* */` comments and trailing commas. When Captain or `captain set` changes a value, it edits that value in place, so your comments stay.

Captain writes `settings.schema.json` next to the file. The `"$schema": "./settings.schema.json"` line in the file gives editors such as Zed and VS Code completion, descriptions, and checks. `version` is the file format; Captain writes it.

Captain watches the file and applies a change when you save it. If a value is wrong, Captain keeps the last good settings and names the line, for example: `settings.json line 9: kubernetes.port must be a whole number from 1 to 65535.`

## Appearance

### `appearance`

Light or dark mode. `system` follows the operating system. Applies at once.

- Type: one of system, light, dark
- Default: `"system"`
- Example: `"appearance": "dark"`

### `theme`

The color theme. Each theme has a light and a dark version. Applies at once.

- Type: one of dusk, periwinkle, harbor
- Default: `"dusk"`
- Example: `"theme": "harbor"`

## Engine

### `engine_endpoint`

A Docker API address that wins over the engines Captain finds, such as `unix:///var/run/docker.sock` or `tcp://10.0.0.5:2375`. Used when `engine` is `external`. Applies at the next launch.

- Type: a string, or null
- Default: `null`
- Example: `"engine_endpoint": "tcp://10.0.0.5:2375"`

### `engine`

`captain` runs Captain Engine, and `external` connects to an engine that Captain does not control. Without it, Captain picks Captain Engine when it can run on this computer. Applies at the next launch.

- Type: one of captain, external, or null
- Default: `null`
- Example: `"engine": "external"`

### `stop_engine_on_quit`

Stop Captain Engine when Captain quits. Applies at once.

- Type: true or false
- Default: `true`
- Example: `"stop_engine_on_quit": false`

### `engine_resources`

The CPUs, memory, and disk of Captain Engine. Set all three, or leave the key out for this computer's defaults: half the CPUs (2 to 8), a quarter of the memory (4 to 16 GiB), and a 64 GiB disk. Applies at the next engine start.

- Type: an object with cpus, memory_bytes, and disk_bytes, or null
- Default: `null`
- Example: `"engine_resources": {"cpus":4,"memory_bytes":8589934592,"disk_bytes":68719476736}`

## Docker daemon

### `engine_daemon.registry_mirrors`

Mirrors that Docker tries before Docker Hub, as `https://` or `http://` URLs. Applies at the next engine start.

- Type: a list of strings
- Default: `[]`
- Example: `"engine_daemon": { "registry_mirrors": ["https://mirror.gcr.io"] }`

### `engine_daemon.insecure_registries`

Registries that Docker may reach over plain HTTP or with an untrusted certificate, as `host:port` or CIDR entries. Applies at the next engine start.

- Type: a list of strings
- Default: `[]`
- Example: `"engine_daemon": { "insecure_registries": ["registry.local:5000"] }`

### `engine_daemon.custom`

Other `daemon.json` keys for the engine's Docker daemon. Docker checks them before it starts, and Captain rolls back keys it rejects. Applies at the next engine start.

- Type: an object
- Default: `{}`
- Example: `"engine_daemon": { "custom": {"log-level":"warn"} }`

### `engine_daemon.tcp`

Also serve the Docker API on `tcp://127.0.0.1:<tcp_port>`, without TLS. Applies at the next engine start.

- Type: true or false
- Default: `false`
- Example: `"engine_daemon": { "tcp": true }`

### `engine_daemon.tcp_port`

The port for `tcp`. Captain keeps it while `tcp` is off. Applies at the next engine start.

- Type: a whole number from 1 to 65535
- Default: `2375`
- Example: `"engine_daemon": { "tcp_port": 23750 }`

## Kubernetes

### `kubernetes.enabled`

Run a k3s cluster in Captain Engine. Give the engine 6 GiB of memory or more. Applies at the next engine start.

- Type: true or false
- Default: `false`
- Example: `"kubernetes": { "enabled": true }`

### `kubernetes.version`

The k3s version. When you turn Kubernetes on, Captain saves the stable version, so k3s never upgrades by itself. Applies at the next engine start.

- Type: a string, or null
- Default: `null`
- Example: `"kubernetes": { "version": "v1.36.4+k3s1" }`

### `kubernetes.port`

The port of the Kubernetes API on `127.0.0.1`. Applies at the next engine start.

- Type: a whole number from 1 to 65535
- Default: `6443`
- Example: `"kubernetes": { "port": 16443 }`

### `kubernetes.traefik`

Install Traefik, the k3s ingress controller, on ports 80 and 443. Applies at the next engine start.

- Type: true or false
- Default: `true`
- Example: `"kubernetes": { "traefik": false }`

## Startup

### `start_in_background`

Launch with only the menu bar icon and no main window. It needs `show_menu_bar_icon`. Applies at the next launch.

- Type: true or false
- Default: `false`
- Example: `"start_in_background": true`

### `show_menu_bar_icon`

Show Captain's icon in the menu bar (macOS) or the notification area (Windows). Applies at once.

- Type: true or false
- Default: `true`
- Example: `"show_menu_bar_icon": false`

## Terminal

### `command_line_tools.enabled`

Keep the links in `~/.captain/bin`, the Docker CLI plugin folder, and (with `automatic`) the PATH lines in place at each launch. Off until you click Install or run `captain tools install`. Applies at the next launch.

- Type: true or false
- Default: `false`
- Example: `"command_line_tools": { "enabled": true }`

### `command_line_tools.path`

Who puts `~/.captain/bin` on PATH: `automatic` adds a marked block to your shell files, and `manual` leaves them to you. Applies at the next launch.

- Type: one of automatic, manual
- Default: `"manual"`
- Example: `"command_line_tools": { "path": "automatic" }`

## AI agents

### `agent_tools.enabled`

Let AI agents connected to `captain mcp` read Captain's engine, projects, containers, problems, logs, and disk use. While it is off, the server lists only `help`. Applies at once.

- Type: true or false
- Default: `false`
- Example: `"agent_tools": { "enabled": true }`

### `agent_tools.actions`

The actions agents may run: `start`, `stop`, `restart`, `run_task` (only the tasks in a Compose file's `x-captain.tasks`), and `raise_memory`. Each one is a tool that agents see only while it is here. Applies at once.

- Type: a list of any of start, stop, restart, run_task, raise_memory
- Default: `[]`
- Example: `"agent_tools": { "actions": ["restart","run_task"] }`

## Storage

### `weekly_build_cache_cleanup`

Once a week, while the engine runs, remove build cache older than 14 days.

- Type: true or false
- Default: `false`
- Example: `"weekly_build_cache_cleanup": true`

### `build_cache_cleaned_at`

When the weekly cleanup last ran, in Unix seconds. Captain sets it.

- Type: a whole number, or null
- Default: `null`
- Example: `"build_cache_cleaned_at": 1790000000`

## Diagnostics

### `debug_logging`

Write debug-level lines to Captain's log file. Applies at once.

- Type: true or false
- Default: `false`
- Example: `"debug_logging": true`

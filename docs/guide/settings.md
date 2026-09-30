# Settings

Captain has two places for settings:

- **The Settings page** has the settings that most people change.
- **The settings file**, `settings.json`, has every option. The page and the file change the same settings.

## The Settings page

Click the Settings button at the bottom of the sidebar. The page has these sections.

### Appearance

Pick a theme: **Dusk**, **Periwinkle**, or **Harbor**. Then pick **System**, **Light**, or **Dark**. **System** follows the macOS appearance.

### Engine

The state of Captain Engine, and its resources:

- **CPUs**
- **Memory**
- **Disk**

Changes apply the next time the engine starts. The disk grows as it fills, up to its size. It cannot shrink, so you can only make it larger.

### Kubernetes

A switch for the k3s cluster in Captain Engine. Off uses no memory. See [Kubernetes](kubernetes.md).

### Startup

- **Open Captain at login.**
- **Show Captain in the menu bar.** Without the icon, closing the window quits Captain.

### Terminal

This row says where your terminal's `docker` comes from, for example "Your terminal's docker still comes from Rancher Desktop." Click **Set up…** to link Captain's tools, put them on your `PATH`, and make `docker` use Captain Engine. See [Set up your terminal](moving-from-docker-desktop-or-rancher.md#set-up-your-terminal).

### Everything else

Registry mirrors, the Docker socket, `daemon.json`, the Kubernetes port and Traefik, and the other options are in the settings file.

- **Open settings file** opens `settings.json` in your default editor for JSON files.
- **All options** lists every option, its default, and what it does.

The bottom of the page shows the versions of Captain, Lima, and Docker, and **Licenses**.

## The settings file

The file is at:

```text
~/Library/Application Support/Captain/settings.json
```

It holds only the options that you changed. Captain keeps the defaults, so the file stays short, and a new default in a later version reaches you.

A file can look like this:

```jsonc
{
  // Only what you change. Every option and its default: Settings > All options.
  "$schema": "./settings.schema.json",

  "theme": "dusk",

  // Pulls try this mirror before Docker Hub.
  "engine_daemon": {
    "registry_mirrors": ["https://mirror.gcr.io"]
  },

  "kubernetes": {
    "port": 6443,
    "traefik": false
  },

  "stop_engine_on_quit": false,
  "weekly_build_cache_cleanup": true
}
```

The [settings reference](../reference/settings.md) lists every option.

### Comments

Write `//` comments to say why you set a value. When the Settings page or `captain set` changes a value, Captain edits that value in place. Your comments stay.

### Autocomplete in your editor

Captain writes `settings.schema.json` next to the file. The `$schema` line points your editor at it. Zed and VS Code then give you:

- completions for option names and values,
- the description of an option when you hover it,
- a warning for a wrong name or value.

If your editor marks the comments as errors, set the file's language to JSON with Comments (`jsonc`).

### When changes apply

Captain watches the file and applies it when you save. You do not need to restart Captain.

Engine options, such as the resources, the Docker daemon, and Kubernetes, apply the next time Captain Engine starts. Captain says so when you save.

### Mistakes

If the file has a mistake, Captain keeps the last good settings and shows an error that names the line. For example:

```text
settings.json line 9: kubernetes.port must be 1–65535. Captain keeps the last good settings.
```

Fix the line and save again.

### From the terminal

`captain set` changes the same file. Quit Captain first.

```sh
captain list-settings
captain set memory 8
captain set stop-engine-on-quit false
```

See [The command line](cli.md).

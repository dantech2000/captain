# Feature 0036: The settings file

- Milestone: M30 (the file half; the one-page Settings is the other half)
- Status: Implemented; checked with unit tests in temp folders
- Builds on: [ADR 0004](../adr/0004-settings-file.md) (the settings file), [0022](0022-command-line.md) (`captain set` and `list-settings`), and [0016](0016-diagnostics.md) (Diagnostics)
- Decision: [ADR 0013](../adr/0013-settings-file-and-docs.md)

## Goal

`settings.json` is a file that people can read, edit, and keep in their dotfiles. It holds only what the user changed, it may have comments, and every key has a description, a default, and an example in one generated reference. Edits in an editor apply while Captain runs. The design is the canvas board `SettingsFile.dc.html`.

## In scope

- **Comments (JSONC).** Captain reads the file with `//` and `/* */` comments and trailing commas, through [`jsonc-parser`](https://docs.rs/jsonc-parser/0.34.0/jsonc_parser/) 0.34 (`parse_to_serde_value`). Looser syntax (single quotes, keys without quotes, hex numbers) is an error, so the file stays valid JSONC for editors.
- **Edits in place.** When the app or the CLI changes a value, Captain edits only the keys that changed, in the file's [concrete syntax tree](https://docs.rs/jsonc-parser/0.34.0/jsonc_parser/cst/index.html) (`CstRootNode::parse`, `CstObject::get`, `CstObjectProp::set_value`, `append`, `remove`). Comments, key order, blank lines, and other keys stay. The writer re-reads the file under the settings lock, so an edit made in an editor meanwhile is kept.
- **Only overrides.** A value equal to its default removes its key, and a group that it leaves empty (such as `kubernetes`) goes too. A new file starts with a note, `"$schema": "./settings.schema.json"`, and `"version": 2`.
- **Migration.** `SETTINGS_VERSION` is 2. At launch, a file with `version` 1 (which held every key) drops the keys equal to their defaults, gets the `$schema` line, and gets `version` 2. The old file goes to `settings.json.captain-backup` once, next to the link, like the Docker `config.json` and kubeconfig writers. Unknown keys, such as the old `accent`, stay.
- **Never over a broken file.** A file with bad JSON is never written. The app and the CLI report "settings.json line N: …" and wait for the fix.
- **Schema.** `schemars` 1.2 (the version gpui-kit already builds) derives a draft 7 JSON Schema from `Settings` and its parts. Doc comments are the descriptions; `#[schemars(example = …)]` adds examples; `#[serde(default)]` adds defaults; `range(min = 1)` bounds the ports; `x-captain-group` names the reference heading. `allowComments` and `allowTrailingCommas` tell VS Code the file is JSONC ([VS Code: JSON with comments](https://code.visualstudio.com/docs/languages/json#_json-with-comments)). The app writes `settings.schema.json` next to `settings.json` at each launch when it changed; a copy is in `docs/reference/`.
- **Reference.** `docs/reference/settings.md` lists every key under Appearance, Engine, Docker daemon, Kubernetes, Startup, Terminal, Storage, and Diagnostics, with its type, default, description, and example. `captain_core::settings::reference_entries()` returns the same entries for the in-app "All options" list.
- **Drift check.** A unit test compares the committed `docs/reference/settings.md` and `settings.schema.json` with the generated text. `CAPTAIN_BLESS=1 cargo test -p captain-core reference` rewrites them.
- **Live reload.** The app reads the file every 2 seconds and compares the text. A change applies at once: the store takes the settings, the theme and appearance redraw, the menu bar icon follows, debug logging switches, and the Docker daemon, Kubernetes, and resource settings go to Captain Engine for its next start. The Docker daemon card then shows "Restart to apply" while the engine runs with other settings. `engine` and `engine_endpoint` apply at the next launch.
- **Mistakes.** A value that a key does not take (a wrong type, an unknown name, a port out of range) keeps the last good settings. Captain names the first mistake in a toast and in a new Diagnostics check, "Settings file", with an "Open settings.json" fix: "Captain keeps the last good settings. settings.json line 9: kubernetes.port must be a whole number from 1 to 65535. See …/settings.md#kubernetesport". At launch, such a value falls back to its default, as before.
- **`captain list-settings`** prints `file` or `default` after each value. `--json` prints `{"cpus": {"value": "4", "source": "file"}, …}`.
- **API for the Settings page** (`captain_ui`): `settings_file_path(cx)`, `open_settings_file(cx)` (`open -t` on macOS, `xdg-open` on Linux, `cmd /c start ""` on Windows; it creates a missing file first), and `captain_core::settings::reference_entries()` (key, group, type, default, description, example).

## Out of scope

- `captain set` while the app runs. It still refuses: the app now picks up the change, but the CLI's engine steps (for example `kubernetes enable`) still assume the app is closed.
- Hot-switching `engine` or `engine_endpoint` from the file. They apply at the next launch.
- A settings UI for the "All options" list. The setpage agent builds it with the API above.
- Unit-aware memory and disk values in the file (`"8GiB"`). `engine_resources` stays in bytes.

## Notes

- A poll, not the `notify` crate. gpui-kit already builds `notify` 7, but editors save in different ways (truncate, or write a new file and rename it), and watchers see different events for each ([notify: Editor Behaviour](https://docs.rs/notify/latest/notify/#editor-behaviour)). A 2-second read of a small file works the same on every OS and through a symlink. The Storage page and the process list poll the same way.
- The problem check is generic: each key in the file is compared with the value the lenient reader kept, and with the schema's `minimum` and `maximum`. The wording comes from the schema ("one of dusk, periwinkle, harbor", "true or false").
- The line number comes from `jsonc_parser::parse_to_ast`, which gives each property's position.
- Zed and VS Code read `$schema` for completion and hover docs ([Zed: JSON](https://zed.dev/docs/languages/json), [VS Code: JSON schemas](https://code.visualstudio.com/docs/languages/json#_json-schemas-and-settings)). A relative `./settings.schema.json` resolves next to the file.
- Draft 7 rather than 2020-12, because editors support it best; `schemars` moves `$ref` siblings into `allOf` for draft 7.

## Verification

1. Run `cargo test -p captain-core settings`. The tests cover a comment that survives a value change, an overrides-only write, the migration with one backup, a bad file that is never overwritten, a wrong value with its line, and the reference drift check.
2. Quit Captain. Copy your `settings.json` somewhere safe. Run `captain list-settings`. Each row ends with `file` or `default`.
3. Open Captain. Check that `settings.schema.json` is next to `settings.json`, and that an old file now holds only changed keys, with `settings.json.captain-backup` next to it.
4. Open `settings.json` in Zed. Type `"th` and pick `theme` from the completions. Set `"theme": "harbor"` with a comment above it, and save. The window turns Harbor within 2 seconds.
5. Change the theme on the Settings page. The comment is still in the file.
6. Set `"kubernetes": { "port": 70000 }` and save. A toast names line and key, the theme stays, and Diagnostics shows "Settings file" failed with "Open settings.json". Fix the value; the check passes.
7. With the engine running, add a registry mirror in the file. The Docker daemon card shows "Restart to apply".

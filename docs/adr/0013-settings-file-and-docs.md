# ADR 0013: A short Settings page and a documented settings file

- Status: Accepted
- Date: 2026-09-29

## Context

Captain has grown to about 25 settings: appearance, theme, the engine and its resources, the Docker daemon, Kubernetes, startup, the command-line tools, storage cleanup, and logging. The Settings page shows every one as a card, and it keeps getting longer. Most people change three or four of them.

`settings.json` (ADR 0004) holds every key, with no comments and no descriptions. Each save rewrites the whole file, so a comment or a key order that a user adds is lost. Nothing documents what a key means, its default, or its values, except the source.

Editors such as Zed and VS Code, and apps such as Zed itself, handle this with a short settings UI plus a JSONC file with a JSON Schema: completion, hover docs, and error checks come from the schema.

## Decision

- **Two surfaces.** A short Settings page shows the common settings. Everything else lives in `settings.json`, and an "All options" view lists every key. The Settings page and the file change the same values.
- **The file holds only overrides.** A key appears only when its value differs from the default. A new default in a later version reaches every user who did not change it. A write of a default removes the key.
- **JSONC, edited in place.** The file may have comments and trailing commas. Captain and `captain set` edit only the keys that change, in the file's concrete syntax tree ([`jsonc-parser` CST](https://docs.rs/jsonc-parser/0.34.0/jsonc_parser/cst/index.html)), so comments, order, and formatting stay. A file with bad JSON is never written.
- **One source for the docs.** The doc comments on the settings types are the user-facing descriptions. `schemars` derives a JSON Schema from the types. The schema produces `settings.schema.json` (written next to the file, with a `$schema` line in the file) and `docs/reference/settings.md`, and the app's "All options" view reads the same entries. A unit test fails when the committed reference differs from the types.
- **The file is live.** Captain reads the file every 2 seconds and applies a change. A value that a key does not take keeps the last good settings, and Captain names the line and key with a link to the reference.
- **Format version 2.** A version 1 file (every key) drops its defaults once, with a backup.

## Consequences

- A setting added to `Settings` needs a user-facing doc comment, an example, and an `x-captain-group`; the drift test then asks for a regenerated reference.
- Doc comments on the settings fields speak to the user. Notes for developers move to `//` comments.
- Users can keep `settings.json` in their dotfiles; a symlinked file stays a link.
- The Settings page can drop rarely used cards, because the file and the reference cover them.
- `captain-core` depends on `jsonc-parser` and `schemars`. `schemars` 1.2 was already in the build through gpui-kit.
- See [feature 0036](../features/0036-settings-file.md) for the details.

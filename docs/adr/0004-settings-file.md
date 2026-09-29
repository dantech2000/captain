# ADR 0004: A versioned JSON settings file

- Status: Accepted
- Date: 2026-09-28

## Context

M7 adds settings: the appearance, an accent color, and an engine endpoint that wins over discovery. Captain must keep them between runs. A file written by one version of Captain must still load in an older or a newer version.

## Decision

- Format: one JSON object, written with `serde_json`. JSON is readable, easy to fix by hand, and `serde_json` is already a dependency.
- Location: `Captain/settings.json` in the platform config directory from the `dirs` crate. On macOS that is `~/Library/Application Support/Captain/settings.json`. On Linux it is `~/.config/Captain/settings.json`, and on Windows `%APPDATA%\Captain\settings.json`. `captain-app` picks the path. `captain-core` loads and saves at any path, so tests use a temp directory.
- Versioning: the file has a `version` field. This build writes version 1. A reader ignores unknown fields, gives missing fields their default, and gives a field with an unknown value its default. So a newer file loads in an older Captain with only the known settings. A future format change that renames or moves a field adds a migration from the older version when it loads.
- Writes: a save writes `.settings.json.tmp` in the same directory, syncs it, and renames it over `settings.json`. A crash leaves the old file or the new file, never half of one.
- Errors: a missing file gives the defaults. A file that is not valid JSON gives the defaults and a warning. The next change the user makes replaces that file.
- In the UI: `captain-ui` holds the settings in a GPUI global. A change saves at once on the main thread; the file is small.

## Consequences

- The settings format is plain data in `captain-core`, with unit tests and no UI.
- Values in the file are lowercase names (`"dark"`, `"teal"`), so a new enum variant does not break older readers.
- A malformed file is lost on the next save. We accept that for now; a later change can keep a backup.

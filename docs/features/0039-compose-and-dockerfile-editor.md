# Feature 0039: Compose and Dockerfile editor

- Milestone: M32
- Status: Planned

## Goal

Edit a project's Compose files and Dockerfiles inside Captain, see mistakes before saving, and see what a change will do before applying it. It complements the user's own editor; **Open in editor** stays for bigger work.

## Why

Captain already knows each project's files (the `com.docker.compose.project.config_files` label and each service's build context), its services, and its running state. Other container apps show containers but send users elsewhere to change them. An editor that knows the project can check a change against the real Compose model and show which services it recreates before anything runs.

## What exists

- GPUI Kit 0.7's input is a code editor: tree-sitter highlighting (YAML behind the `tree-sitter-yaml` feature), line numbers, a `CompletionProvider`, a `HoverProvider`, and diagnostics (`gpui-component` `src/input`, `src/highlighter`). Dockerfile has no bundled grammar; register one through the highlighter registry (for example the `tree-sitter-dockerfile` grammar; check its license and maintenance first).
- Captain already writes files safely (`captain_core::file_replace`: a temp file, sync, rename, permissions kept) and runs the Compose CLI with the engine's `DOCKER_HOST`.

## Build plan

| # | Phase | Delivers | Done when |
|---|---|---|---|
| 1 | Editor | A **Files** tab on the project page listing the Compose files and each service's Dockerfile; an editor with highlighting (YAML, Dockerfile), line numbers, find, undo; **Save** writes through `file_replace`; a file changed on disk shows a reload bar instead of being overwritten; symlinks write to their target. | A Compose file opens, edits, saves, and keeps its permissions; an outside change is never overwritten silently. |
| 2 | Checks | Errors as the user types (debounced): run `docker compose config --quiet` on the unsaved text (through a temp copy next to the real file, so relative paths resolve) and map its errors to lines. Completion and hover docs from the Compose Specification's JSON schema (`compose-spec/compose-spec`, vendored with its license) plus Captain's `x-captain.tasks` schema. Dockerfile checks through BuildKit's build checks (`docker build --check`, check the minimum Docker version). | A bad key or indentation shows on its line before saving; completions offer service keys. |
| 3 | Apply with a preview | **Save and apply** runs `docker compose up --dry-run` (check the Compose version that added it) and lists what will happen per service: recreate, create, remove, pull, build. The user confirms; Captain runs `up` and the project log shows it. Saving a Dockerfile offers **Rebuild <service>** (`up --build <service>`). | The preview names exactly the services the real `up` then changes. |
| 4 | Docs and hand checks | `docs/guide/editing-projects.md`; the verification below; ROADMAP and the testing guide. | CI green on three OSes; hand checks pass. |

## Safety

- Nothing is written without **Save**, and nothing runs without a confirm after the preview.
- Captain edits only files the project already uses (from its labels and build contexts), inside the project folder.
- Secrets: `.env` files are not opened by this editor in the first version; `env_file` entries show as links to open in the user's editor.

## Out of scope

- A full IDE: multi-file search, git, refactoring.
- New projects from templates. A later milestone could add "New project" with templates (Postgres, Redis, a web app).
- A language server. Docker publishes a language server for Compose and Dockerfiles (`docker/docker-language-server`); check it later as a richer source of checks and completions than the CLI and schema.

## Verification

1. Open stokecrm's Files tab and its `docker-compose.yml`.
2. Type a wrong key under a service. Expect an error on that line before saving.
3. Change the Postgres image tag and click **Save and apply**. Expect the preview to name only `postgres` as recreated; cancel, and check nothing changed.
4. Edit the file in another editor while it is open in Captain. Expect a reload bar, not a silent overwrite.

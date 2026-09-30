# Feature 0039: Compose and Dockerfile editor

- Milestone: M32
- Status: Planned; checks done 2026-09-30

## Goal

Edit a project's Compose files and Dockerfiles inside Captain, see mistakes before saving, and see what a change will do before applying it. It complements the user's own editor; **Open in editor** stays for bigger work.

## Why

Captain already knows each project's files (the `com.docker.compose.project.config_files` label and each service's build context), its services, and its running state. Other container apps show containers but send users elsewhere to change them. An editor that knows the project can check a change against the real Compose model and show which services it recreates before anything runs.

## What exists

- GPUI Kit 0.7's input is a code editor: tree-sitter highlighting (YAML behind the `tree-sitter-yaml` feature), line numbers, a `CompletionProvider`, a `HoverProvider`, and diagnostics (`gpui-component` `src/input`, `src/highlighter`). Dockerfile has no bundled grammar; register `tree-sitter-containerfile` through the highlighter registry (check 3).
- Captain already writes files safely (`captain_core::file_replace`: a temp file, sync, rename, permissions kept) and runs the Compose CLI with the engine's `DOCKER_HOST`.

## Checks (done 2026-09-30)

All runs used the tools in `/Applications/Captain.app/Contents/Resources` (Docker CLI 29.8.1, Compose v5.5.1, Buildx 0.37.1, the pins in `scripts/tool-versions.env`) against Captain Engine (Docker Engine 29.8.1, API 1.56, BuildKit v0.33.0). The spike and the throwaway project `captain-agent-m32dry` were removed afterwards.

1. **`docker compose up --dry-run`: passes, with two pitfalls.**
   - Compose v2.18.0 (2023-05-16) added dry-run to `up` and moved `--dry-run` out of `alpha` ([#10529](https://github.com/docker/compose/pull/10529), [#10533](https://github.com/docker/compose/pull/10533), [v2.18.0](https://github.com/docker/compose/releases/tag/v2.18.0)). v2.19.0 called it feature complete ([#10604](https://github.com/docker/compose/pull/10604)). The bundled v5.5.1 has it ([`docker compose` reference](https://docs.docker.com/reference/cli/docker/compose/)).
   - Output goes to stderr, one event per line. With `--progress json` each line is `{"id":"<Kind> <name>","status":"Working|Done","text":"<action>"}`, for example `{"id":"Container captain-agent-m32dry-web-1","status":"Working","text":"Recreate"}`. Plain text is ` Container <name> <Action> `.
   - Actions seen: `Running` (no change), `Recreate`/`Recreated`, `Creating`/`Created`, `Stopping`/`Removing`/`Removed` (orphans, only with `--remove-orphans`; without it a `level=warning msg="Found orphan containers ..."` line appears), `Image <ref> Pulling`/`Pulled`, `Image <name> Building` and `Image <service> Built`. Nothing is pulled or built for real; no image is left behind.
   - Pitfalls: after `Recreated`, fake events name a container `<12 hex>_<name>`; read only the first action per container. Map container names to services through the project's labels, not by splitting names. Some `Starting` events are missing, so do not use them. `up -d --dry-run` hangs forever when a service `depends_on` another with `condition: service_completed_successfully` ([#14269](https://github.com/docker/compose/issues/14269); the fix [#14270](https://github.com/docker/compose/pull/14270) merged 2026-09-30, after v5.5.1). Run the preview with a timeout.
2. **`docker build --check`: passes; use JSON.**
   - Needs Buildx 0.15.0 (`--call` and `--check`, [v0.15.0](https://github.com/docker/buildx/releases/tag/v0.15.0)) and Dockerfile frontend 1.8.0 (the lint subrequest, [dockerfile/1.8.0](https://github.com/moby/buildkit/releases/tag/dockerfile/1.8.0)). BuildKit v0.15.0 carries that frontend; Docker Engine 27.1.0 is the first to vendor it (27.0 has v0.14.1: [vendor.mod v27.0.1](https://github.com/moby/moby/blob/v27.0.1/vendor.mod), [v27.1.0](https://github.com/moby/moby/blob/v27.1.0/vendor.mod)). Older engines can use `# syntax=docker/dockerfile:1`. Docs: [Build checks](https://docs.docker.com/build/checks/).
   - `docker build --call check,format=json -q -f - <context>` takes the unsaved Dockerfile on stdin and prints `{"warnings":[{"ruleName","description","url","detail","location":{"ranges":[{"start":{"line"},"end":{"line"}}]}}], "buildError": {...}}`. Lines are 1-based. A parse error comes back in `buildError` with its line (`unknown instruction: RUNN (did you mean RUN?)`). Exit code is 1 when there are warnings or an error. `--progress=rawjson` shows only build vertexes, not check results.
   - Pitfall: the check resolves base-image metadata, so an unknown or unreachable image is a `buildError` (`pull access denied ...`) with the `FROM` line. Show it as a warning, not as a syntax error. It sends only the Dockerfile and `.dockerignore`, not the context; a run took under a second.
3. **Dockerfile grammar: use `tree-sitter-containerfile`, not `tree-sitter-dockerfile`.**
   - [`tree-sitter-dockerfile`](https://crates.io/crates/tree-sitter-dockerfile) 0.2.0 (MIT, [camdencheek/tree-sitter-dockerfile](https://github.com/camdencheek/tree-sitter-dockerfile)) was last published 2024-05 and depends on `tree-sitter ^0.20`; its main branch moved to 0.24 without a release. GPUI Kit 0.7 uses `tree-sitter` 0.26.13, so its `Language` type does not fit. The fork [`tree-sitter-dockerfile-updated`](https://crates.io/crates/tree-sitter-dockerfile-updated) has one release and no activity.
   - [`tree-sitter-containerfile`](https://crates.io/crates/tree-sitter-containerfile) 0.9.2 (MIT, [wharflab/tree-sitter-containerfile](https://github.com/wharflab/tree-sitter-containerfile)) continues the camdencheek grammar (its original MIT notice is kept), is released often (0.8.0 in 2026-04, 0.9.2 in 2026-07), and uses `tree-sitter-language ^0.1`, so `LANGUAGE.into()` gives a 0.26 `Language`. It ships `HIGHLIGHTS_QUERY`. Risk: one main author, and its CI runs only on Ubuntu; Captain's three-OS CI covers the C build.
   - The spike registered it with `LanguageRegistry::singleton().register("dockerfile", &LanguageConfig::new(..))`, and a Dockerfile with a heredoc and `COPY --from` parsed with no error nodes and produced highlights.
4. **Compose schema: vendor compose-go's copy, Apache-2.0.**
   - Compose v5.5.1 builds with `compose-go/v2` v2.15.0 ([go.mod](https://github.com/docker/compose/blob/v5.5.1/go.mod)), which validates against its own [`schema/compose-spec.json`](https://github.com/compose-spec/compose-go/blob/v2.15.0/schema/compose-spec.json) (JSON Schema 2020-12, 375 `description` fields for hover text). The [compose-spec](https://github.com/compose-spec/compose-spec/blob/main/schema/compose-spec.json) main branch is already ahead (it adds `container_spec`, jobs, and `pre_start` hooks), so it would offer keys the bundled Compose rejects.
   - Both repositories are Apache-2.0 ([compose-go LICENSE](https://github.com/compose-spec/compose-go/blob/v2.15.0/LICENSE), which also has a `NOTICE`). Vendor the file with LICENSE and NOTICE, and update it when `COMPOSE_VERSION` changes.
5. **GPUI Kit 0.7 editor: passes.** A spike test in `captain-ui` (removed again) compiled and passed:
   - `gpui-kit` feature `tree-sitter-yaml` builds (adds `tree-sitter` 0.26.13, `tree-sitter-yaml` 0.7.2, `tree-sitter-json` 0.24.8). `SyntaxHighlighter::new("yaml")` with `update` and `styles` highlights a Compose file without a window.
   - `EditorState::new(window, cx).language("yaml").line_number(true).searchable(true)` builds the editor state.
   - `CompletionProvider::completions(&self, &Rope, offset: usize, CompletionContext, &mut Window, &mut App) -> Task<Result<CompletionResponse>>` and `is_completion_trigger(&self, offset, new_text: &str, &mut App) -> bool`; `HoverProvider::hover(&self, &Rope, offset, &mut Window, &mut App) -> Task<Result<Option<lsp_types::Hover>>>`. Set them through `state.lsp_mut().completion_provider` and `.hover_provider` (`Option<Rc<dyn ..>>`). Source: `gpui-base-0.7.0/src/input/editor/lsp/`.
   - Diagnostics: `state.diagnostics_mut() -> Option<&mut DiagnosticSet>`, then `clear()` and `push(Diagnostic::new(Position..Position, msg).with_severity(DiagnosticSeverity::Error).with_source(..))`. `DiagnosticSet::for_offset` maps a byte offset back to the entry.
   - Captain needs a direct `lsp-types = "0.97"` dependency, because GPUI Kit does not re-export the LSP types beyond `Position`.
6. **`docker compose config` on unsaved text: passes; stdin is better than a temp copy.**
   - Test project: `extends` from `common/base.yaml`, an `env_file: ../app.env` and a `./shared` bind inside the extended file, `build: ./app`, and `.env` interpolation. `docker compose -f .compose.yaml.captain-edit --project-directory <dir> config` and `docker compose -f - --project-directory <dir> config < text` both gave output identical to `config` on the real file: paths in the extended file resolve relative to that file, and the project name stays the directory name.
   - The temp copy shows up in errors (`validating <dir>/.compose.yaml.captain-edit: ...`) and would touch the project folder (the Compose file watcher, `git status`). Stdin shows `validating -: ...` and writes nothing. Use `docker compose -p <project> --project-directory <working_dir label> -f <other files> -f - config --quiet`, with the edited file at its original place in the `-f` list.
   - Error forms: schema errors give a key path, not a line (`services.web additional properties 'imgae' not allowed`, `services.web.ports must be a array`); YAML errors give a range (`... at L2.C3-L4.C4: did not find expected key`). Captain maps a key path to a line through the tree-sitter YAML tree.

## Build plan

| # | Phase | Delivers | Done when |
|---|---|---|---|
| 1 | Editor | A **Files** tab on the project page listing the Compose files and each service's Dockerfile; an editor with highlighting (the `tree-sitter-yaml` feature, and `tree-sitter-containerfile` registered as `dockerfile`), line numbers, find, undo; **Save** writes through `file_replace`; a file changed on disk shows a reload bar instead of being overwritten; symlinks write to their target. | A Compose file opens, edits, saves, and keeps its permissions; an outside change is never overwritten silently. |
| 2 | Checks | Errors as the user types (debounced): pipe the unsaved text to `docker compose -p <project> --project-directory <working_dir> -f <other files> -f - config --quiet` (check 6; no temp file) and map its errors to lines (YAML errors carry a line; schema errors carry a key path that Captain finds in the tree-sitter tree). Completion and hover docs from compose-go v2.15.0's `schema/compose-spec.json` (the schema the bundled Compose uses, vendored with its Apache-2.0 LICENSE and NOTICE) plus Captain's `x-captain.tasks` schema. Dockerfile checks: pipe the text to `docker build --call check,format=json -q -f - <context>` and show `warnings` and `buildError` on their lines; a base-image lookup failure is a warning. | A bad key or indentation shows on its line before saving; completions offer service keys. |
| 3 | Apply with a preview | **Save and apply** runs `docker compose --progress json up -d --dry-run` (Compose 2.18.0 or later) with a timeout (Compose #14269) and lists the first action per container, mapped to services through labels: recreate, create, remove (with `--remove-orphans`), pull, build, unchanged. The user confirms; Captain runs `up` and the project log shows it. Saving a Dockerfile offers **Rebuild <service>** (`up --build <service>`). | The preview names exactly the services the real `up` then changes. |
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

# Feature 0040: New projects

- Milestone: M33
- Status: Phase 1 built in code (2026-10-01): the known-projects store, stopped projects in the sidebar with Up, Remove from Captain, the New sheet, Open a folder, and the `projects_dir` setting. Phases 2–4 are open. Research done on 2026-10-01; sources are linked in each section.

## Goal

Create and import containers and Compose projects from Captain: run an image, start from a template, open a folder, or paste a `docker run` command. Every path ends in the M32 editor and its `up --dry-run` preview, so nothing starts before the user has seen the Compose file and what `up` will do.

## Why

- Captain finds Compose projects only from container labels ([0010](0010-compose-projects.md)). After `docker compose down` a project has no containers, so it leaves the sidebar, and the user must go back to a terminal to start it again.
- A single container can be run only from the Images page's Run dialog (name, ports, environment). The container it makes is loose: it has no file, so it cannot be edited, checked, or started again the same way.
- M32 gave Captain an editor with checks and a preview ([0039](0039-compose-and-dockerfile-editor.md)). A Compose file on disk is a better result of "run this" than a loose container: the user can read it, change it, commit it, and run it again.

## How others do it

- **Docker Desktop.** Run on an image opens "Optional settings" for name, ports, volumes, and environment ([Images](https://docs.docker.com/desktop/use-desktop/images/)). The Containers view groups Compose apps and starts, stops, and deletes them, but the docs describe no way to create or open a Compose project in the GUI ([Containers](https://docs.docker.com/desktop/use-desktop/container/)).
- **Podman Desktop.** It creates a container from an image or a Containerfile, plays Kubernetes YAML, and makes pods ([Discover Podman Desktop](https://podman-desktop.io/docs/discover-podman-desktop)). Compose runs in a terminal; the GUI shows the result as a group ([Running Compose](https://podman-desktop.io/docs/compose/running-compose)).
- **OrbStack.** The GUI manages containers and volumes; Compose runs through the CLI ([OrbStack Docker docs](https://docs.orbstack.dev/docker/)). The docs do not say that the GUI can create a container.

None of them keeps a project that is down, or turns a `docker run` into a file.

## Design

### One entry point: New

- A **+** button beside "Projects" in the sidebar, **⌘N** (Ctrl N on Linux and Windows), and `new` in the ⌘K palette ("New project…") all open the **New** sheet.
- The sheet has four option cards, each with an icon, a title, and one sentence. Up and Down move the highlight, Return opens the highlighted card, and ⌘1 to ⌘4 open a card directly:
  1. **Run an image** — pick an image from this engine or Docker Hub, set ports, environment, and volumes.
  2. **Start from a template** — Postgres, MySQL, Redis, MongoDB, RabbitMQ, or a web server for a folder.
  3. **Open a folder** — a folder or a Compose file that is already on this computer.
  4. **Paste a docker run command** — Captain turns it into a Compose file.
- The footer says where new projects go (`~/Captain`, from the `projects_dir` setting) with **Show folder**.
- All four end on the Project page's **Files** tab, with the Compose file open, and **Save and apply** runs the `up --dry-run` preview first ([0039](0039-compose-and-dockerfile-editor.md)).

### Run an image

- The image picker searches the engine's images first, then Docker Hub (see [Docker Hub search](#docker-hub-search)). Official images get a badge. Tags load when an image is chosen.
- The form has the Run dialog's fields (name, ports, environment) and adds volumes. Ports, environment, and volumes are rows with add and remove.
- **Save as a project** is on by default. On: Captain writes `~/Captain/<name>/compose.yaml` with one service, records the project, and opens the editor. Off: a plain `docker run -d`, as the Images page's Run dialog does today, and the container is loose.

### Start from a template

Templates are Compose files built into Captain with pinned tags. Each asks only for a project name and, where the image needs one, a password, which goes in `.env` (mode 0600), not in `compose.yaml`. Data lives in a named volume. Tags were read from the official-images library files on 2026-10-01 ([docker-library/official-images](https://github.com/docker-library/official-images/tree/master/library)).

| Template | Image | Needs | Data |
|---|---|---|---|
| PostgreSQL | `postgres:18` | `POSTGRES_PASSWORD` | volume at `/var/lib/postgresql`. From 18 on, `PGDATA` is `/var/lib/postgresql/18/docker` and the image's `VOLUME` is `/var/lib/postgresql`; mount that, not `.../data` ([postgres](https://hub.docker.com/_/postgres)). |
| MySQL | `mysql:8.4` (the older LTS; `lts` is 9.7) | `MYSQL_ROOT_PASSWORD` | `/var/lib/mysql` ([mysql docs](https://github.com/docker-library/docs/blob/master/mysql/content.md)) |
| Redis | `redis:8-alpine` | nothing | `/data` ([redis docs](https://github.com/docker-library/docs/blob/master/redis/content.md)) |
| MongoDB | `mongo:8` | `MONGO_INITDB_ROOT_USERNAME`, `MONGO_INITDB_ROOT_PASSWORD` (optional, set by default) | `/data/db` ([mongo docs](https://github.com/docker-library/docs/blob/master/mongo/content.md)) |
| RabbitMQ | `rabbitmq:4-management` | `RABBITMQ_DEFAULT_USER`, `RABBITMQ_DEFAULT_PASS`; a fixed `hostname`, because the node name and data folder depend on it | `/var/lib/rabbitmq` ([rabbitmq docs](https://github.com/docker-library/docs/blob/master/rabbitmq/content.md)) |
| Web server | `nginx:stable-alpine` | a folder to serve | the folder bound read-only at `/usr/share/nginx/html` ([nginx docs](https://github.com/docker-library/docs/blob/master/nginx/content.md)) |

- No MinIO template: MinIO stopped publishing community images in October 2025, the `minio/minio` repository answers 404 on the Hub API, and the GitHub repository says it is no longer maintained ([minio/minio](https://github.com/minio/minio)).
- Mongo 9 exists; 8 stays the default until it has more use. The tags live in one table in `captain_core`, with a test that each template passes `docker compose config`.

### Open a folder

- The system picker (`cx.prompt_for_paths`, folders and files) chooses a folder or a Compose file.
- In a folder, Captain looks for Compose's default names in compose-go's order: `compose.yaml`, `compose.yml`, `docker-compose.yml`, `docker-compose.yaml` (`DefaultFileNames` in [compose-go `cli/options.go`](https://github.com/compose-spec/compose-go/blob/main/cli/options.go)). Compose prefers `compose.yaml` when several exist ([Compose application model](https://docs.docker.com/compose/intro/compose-application-model/)). The first override file found (`compose.override.yml`, `compose.override.yaml`, `docker-compose.override.yml`, `docker-compose.override.yaml`) is added after it, as Compose does ([Merge](https://docs.docker.com/compose/how-tos/multiple-compose-files/merge/)). Unlike Compose, Captain does not look in parent folders.
- A project name that Captain already knows from another folder is refused, with the folder it knows, because Compose names are unique per engine. Set `name:` in the Compose file or remove the other project first.
- Captain runs `docker compose --project-directory <dir> -f <files> config --format json` and reads `name`, so the project name follows Compose's rules (the top-level `name:`, else the folder name, lowercased) ([Project name](https://docs.docker.com/compose/how-tos/project-name/)). A failed check shows Compose's error in the sheet, and nothing is recorded.
- Then Captain records the project and opens its Files tab. Opened folders stay where they are.

### Paste a docker run command

- Captain parses the command in Rust, flag by flag, the way [composerize](https://github.com/composerize/composerize) does (MIT; `yargs-parser` and a mapping table in `packages/composerize/src/mappings.js`). Captain writes its own parser instead of embedding JavaScript or the one Rust port, [composerize-np](https://github.com/leruetkins/composerize-np) 0.2.0, which is young and little used.
- Words are split like a shell: quotes, backslashes, and `\` line ends. Flags are read from the [`docker run` reference](https://docs.docker.com/reference/cli/docker/container/run/) and mapped to the [Compose services reference](https://docs.docker.com/reference/compose-file/services/):

| `docker run` | Compose |
|---|---|
| `-p`, `--publish` | `ports` |
| `-e`, `--env` | `environment` (a bare `NAME` takes the value from the shell, as `docker run` does; Captain writes `NAME` without a value and warns) |
| `--env-file` | `env_file` |
| `-v`, `--volume` | `volumes` (a name is a named volume and is declared at the top level; a path is a bind) |
| `--name` | the service name and `container_name` |
| `--restart` | `restart` |
| `--network` | `networks` (declared `external: true`), or `network_mode` for `host`, `none`, `bridge`, `container:<id>` |
| `-w`, `--workdir` | `working_dir` |
| `--entrypoint` | `entrypoint` |
| `-u`, `--user`; `-h`, `--hostname`; `-l`, `--label` | `user`, `hostname`, `labels` |
| `-i`, `-t`, `-it` | `stdin_open`, `tty` |
| `--platform`, `--pull` | `platform`, `pull_policy` |
| image, then command | `image`, `command` (as a list) |
| `-d`, `--detach` | nothing: `up -d` detaches |
| `--rm` | nothing: Compose has no such key; Captain warns that the container stays after it stops |

- Every other flag (for example `--mount`, `--cap-add`, `--privileged`, `--device`, `--memory`, `--cpus`, `--gpus`, `--health-*`, `--ulimit`, `--log-opt`, `--add-host`, `--dns`) is kept out of the file and listed as a warning: "Captain did not convert `--privileged`. Add it to compose.yaml by hand." Phase 3 can map more of them; the table above is what phase 3 starts with.

### Known projects and the sidebar

- A **known project** is one Captain created or opened. It is recorded in `~/.captain/projects.json` (see [Storage on disk](#storage-on-disk)).
- The sidebar merges the projects found from labels with the known projects. A known project matches a running one when the Compose project name and the working folder are the same. A known project that matches nothing shows as **Stopped**, with its folder and an **Up** button. Its Project page has the usual header with Up, and the Files tab.
- When a running project has the same name as a known project but another folder, the running project wins the sidebar entry, because Compose names are unique per engine. The known record stays and shows again when the other project is gone.
- **Remove from Captain** in the entry's context menu (right-click) forgets a known project after a confirmation. It never deletes files or containers. A project that still runs stays in the sidebar, found from its labels.

## Storage on disk

- **New projects:** `<projects_dir>/<project>/compose.yaml`, and `.env` when a template or form has secrets. These are normal files for the user to read, edit, and commit. `projects_dir` defaults to `~/Captain`; Captain creates it on first use. A `~/` at its start means the home folder.
- **`projects_dir` setting:** in the settings file ([ADR 0013](../adr/0013-settings-file-and-docs.md)), group "Projects", in the schema and the generated reference ([settings.md](../reference/settings.md)). Changing it moves nothing; known projects keep their paths.
- **Known projects:** `~/.captain/projects.json`. It is app state, not a setting, so it is not in the settings file:

  ```json
  {
    "version": 1,
    "projects": [
      { "name": "shop", "dir": "/Users/me/code/shop", "files": ["compose.yaml"], "added": 1790000000 }
    ]
  }
  ```

  `files` are relative to `dir` unless absolute. `added` is Unix seconds. Captain writes the file through `file_replace` (a temporary file, sync, rename), so a crash leaves the old or the new list. A file that is not valid JSON is never overwritten: Captain logs it, shows no known projects, and refuses to add or remove until the file is fixed.

## UI

Captain keeps one visual style: its own widgets on top of GPUI Kit 0.7 (gpui-component 0.7.0), with colors from the theme tokens that `theme/kit_theme.rs` maps onto the kit's theme ([0027](0027-v3-interface.md), [0028](0028-themes.md)). Chosen for the New flow:

- **Dialog** ([docs](https://gpui-kit.com/component/dialog)) holds the New sheet, as it holds Captain's other sheets (Terminal setup, AI agents). The kit's **Sheet** ([docs](https://gpui-kit.com/component/sheet)) slides in from an edge; a centered dialog fits a short choice better and matches the other sheets.
- **Option cards** are Captain's own: a duotone `cap_icon`, a title, one sentence, and a key chip, in the card style of the theme cards on Settings. The kit's **Radio** ([docs](https://gpui-kit.com/component/radio)) is for a choice that is submitted later; here a card acts at once.
- **Key chips** use Captain's `key_hint` (the ⌘K palette's chip), not the kit's **Kbd** ([docs](https://gpui-kit.com/component/kbd)), so the chips look the same everywhere.
- **Menu** ([docs](https://gpui-kit.com/component/menu)): `ContextMenuExt::context_menu` gives the sidebar entry its right-click menu with Remove from Captain. Settings already uses the kit's `PopupMenu`.
- **AlertDialog** (in Dialog) confirms Remove from Captain, as Down does.
- Phase 2 and 3: **List** with search ([docs](https://gpui-kit.com/component/list)) for the image and template pickers, **Skeleton** ([docs](https://gpui-kit.com/component/skeleton)) rows while Docker Hub answers, **Tag** ([docs](https://gpui-kit.com/component/tag)) for the Official badge, and **Form** ([docs](https://gpui-kit.com/component/form)) for grouped fields (ports, environment, volumes) with add and remove rows. A segmented control switches between the form and the Compose preview; Captain already has `widgets/segmented.rs`.

To adopt elsewhere later (not changed now): **Skeleton** for the Images, Volumes, and Storage pages while they load; **Empty** ([docs](https://gpui-kit.com/component/empty)) for empty pages, in place of `empty_note`; **DescriptionList** ([docs](https://gpui-kit.com/component/description-list)) for the inspector's key-value sections (`widgets/key_values.rs`); **context menus** on container rows and image rows.

## Docker Hub search

Checked live with curl on 2026-10-01; no request needed a login.

- **Search:** `GET https://hub.docker.com/v2/search/repositories/?query=<text>&page_size=<n>&page=<n>` returns `{count, next, results: [{repo_name, short_description, star_count, pull_count, is_official, is_automated, repo_owner}]}`. An official image has `is_official: true` and a `repo_name` without a namespace (`postgres`). This endpoint is not in the documented Hub API ([Docker Hub API](https://docs.docker.com/reference/api/hub/latest/)).
- The Hub web site uses `GET https://hub.docker.com/api/search/v3/catalog/search?query=<text>&from=0&size=<n>&badges=official`, which adds logos and architectures but gives `pull_count` as a rounded string ("1B+"). It is also undocumented. Captain uses the v2 endpoint, reads only the fields above, and treats any failure as "Docker Hub search is not available", keeping local images and the templates.
- **Tags:** the documented `GET https://hub.docker.com/v2/namespaces/{namespace}/repositories/{repository}/tags` ([Docker Hub API](https://docs.docker.com/reference/api/hub/latest/)), with `library` for official images.
- **Limits:** responses carry `X-RateLimit-Limit: 180` (per minute), `X-RateLimit-Remaining`, and `X-RateLimit-Reset`; a 429 has `Retry-After` ([Docker Hub API](https://docs.docker.com/reference/api/hub/latest/), [Usage and limits](https://docs.docker.com/docker-hub/usage/)). Captain searches 400 ms after typing stops, keeps results for the session, and on a 429 waits for `Retry-After`. Pulls are limited separately: 100 per 6 hours per IP without a login, 200 for a free account ([Pull usage](https://docs.docker.com/docker-hub/usage/pulls/)). The picker says so when a pull fails with `toomanyrequests`.

## Phases

1. **Known projects and Open a folder** (built). `captain_core::known_projects` (load, save through `file_replace`, add, remove, merge with label projects), the sidebar's stopped entries with Up, Remove from Captain, the + button, ⌘N, and `new` in ⌘K opening the New sheet (Open a folder works; the other three are disabled with a "Coming next" line), Open a folder with the Compose check, and the `projects_dir` setting.
2. **Templates and Run an image.** The template table and its Compose text in `captain_core`, the project-name and password form, writing `compose.yaml` and `.env` without overwriting a folder that exists; the image picker (local images, Docker Hub search, tags) and the form with Save as a project.
3. **Paste a docker run command.** The shell-word splitter and the flag table in `captain_core`, with one test per mapped flag group and one for the warnings.
4. **Guide and hand checks.** A guide page and a section in docs/testing.md.

## Safety

- Nothing starts before the preview: each path opens the editor, and Save and apply runs `up --dry-run` first.
- Remove from Captain only edits `projects.json`. It never deletes files, containers, or volumes.
- New projects never overwrite: a folder that exists and is not empty is refused, and Captain asks for another name.
- Secrets from templates go in `.env` with mode 0600, never in `compose.yaml`.
- `projects.json` is replaced atomically, and a broken file is never overwritten.
- Docker Hub gets only the search text; no account, token, or local data.

## Out of scope

- Building from a Git URL, Dockerfile-only projects without Compose, and Kubernetes YAML.
- Moving or renaming a project folder from Captain.
- Docker Hub login for search; private registries in the picker.
- A template gallery from the network; templates ship with Captain.

## Verification

- Unit tests (phase 1): `projects.json` round trip and a missing file; a broken file is not overwritten; add replaces the same project and remove forgets it; merge hides a known project that runs, keeps one that runs under the same name elsewhere out of the list, and shows the rest as stopped; the Compose file search follows Compose's order and adds the override; `projects_dir` expands `~/`; the project name is read from `config --format json`.
- By hand (phase 1):
  1. Click + beside Projects. The New sheet opens; Up, Down, and Return move and open; ⌘3 opens Open a folder. Each card and button shows a help sentence in the status bar.
  2. Choose a folder with a `compose.yaml`. The project shows in the sidebar as Stopped, and its Files tab shows the file. Save and apply shows the preview; Apply starts it.
  3. Run Down. The project stays in the sidebar as Stopped, with Up.
  4. Right-click the entry, choose Remove from Captain, and confirm. The entry goes; the folder and its files stay.
  5. Choose a folder without a Compose file. The sheet says that no Compose file was found, and nothing is recorded.
  6. Set `projects_dir` to another folder in the settings file. Show folder in the New sheet opens it, and creates it if needed.

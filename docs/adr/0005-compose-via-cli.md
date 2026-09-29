# ADR 0005: Run Compose project actions through the `docker compose` CLI

- Status: Accepted
- Date: 2026-09-29

## Context

M6 adds project actions: Up, Down, Stop, Restart, and Pull for a whole Compose project. The Engine API has no Compose endpoints. Compose is a client. It reads the Compose files, works out the containers, networks, and volumes, and then calls the Engine API for each one.

Captain can already see projects without Compose. Compose writes labels on every container it creates: `com.docker.compose.project`, `com.docker.compose.service`, `com.docker.compose.project.working_dir`, and `com.docker.compose.project.config_files`. Captain groups containers by these labels.

We considered three ways to run the actions:

- Parse the Compose files and call the Engine API ourselves. The Compose file format is large (profiles, `extends`, `include`, interpolation, build, depends-on ordering). A partial copy would act differently from the real tool, and users would not trust it.
- Stop, start, and remove the project's containers one by one through the Engine API. This works for Stop and Restart, but not for Up (it cannot create a missing container), Pull, or a clean Down (networks stay).
- Run the `docker compose` CLI (the Compose v2 plugin). It is the reference tool, and most Docker installs have it.

## Decision

1. `captain-docker` has a `ComposeCli` in `src/compose/`. It runs `docker compose --ansi never -p <project> -f <file>... <up -d | down | stop | restart | pull>` with `std::process::Command`, in the project's working directory.
2. The child process gets `DOCKER_HOST` set to the endpoint Captain is connected to, and `DOCKER_CONTEXT` removed. So the CLI talks to the same engine as the window, even when the shell's Docker context points somewhere else. An `http://` endpoint becomes `tcp://`, because the CLI does not accept `http://`.
3. Each command runs on its own plain thread and sends its result through a `futures` oneshot channel. The UI gets a runtime-neutral future, like every engine call (see ADR 0002). No tokio is needed.
4. `captain-core` has a `ProjectRunner` trait, separate from `Engine`, because it is not part of the Engine API. The workspace keeps an `Option<Arc<dyn ProjectRunner>>` next to the engine.
5. Captain detects the CLI once per connection. It looks for `docker` on `PATH`, then in known install folders, because an app started from the Finder gets a short `PATH`. It then runs `docker compose version --format json` with a 5-second limit and keeps the version.
6. `up` and `pull` need every Compose file from the labels. `down`, `stop`, and `restart` use the files when all of them exist, and only the project name when they do not. If the working directory is missing on this computer, the command runs in the temp directory.
7. A failed command reports one line of its stderr: the last line that mentions an error, or else the last non-empty line.

## Fallback when the CLI is missing

If detection fails, the runner is `None`. The project cards then keep the engine-only actions from M2 (Start all or Stop all, and Restart all). An info icon on the card says "Install Docker Compose for Up, Down, and Pull." The command palette does not list project commands.

## Consequences

- Up, Down, and Pull act the same as they do in a terminal, because they are the same tool.
- Captain needs the `docker` CLI and the Compose v2 plugin for the full set of project actions. The legacy `docker-compose` v1 binary is not supported.
- The Compose output format is not a stable API. Captain reads only the version JSON and the stderr text of a failure, and shows the text as it is. A new Compose version can change the wording but not break an action.
- A project started on another computer (a remote engine) has a working directory and files that do not exist here. Up and Pull fail with a clear message. Down, Stop, and Restart still work by project name.
- Each action starts a process. This takes about 100 ms, which is small next to the work Compose does.

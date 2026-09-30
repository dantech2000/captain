# Feature 0038: Agent tools (MCP)

- Milestone: M31
- Status: Built (phases 1–6). Research done on 2026-09-29 and 2026-09-30; sources are linked in each section. The hand checks in Verification are open.

## Goal

Let AI coding agents (Claude Code, Codex, Cursor, VS Code, Zed, Gemini CLI, Claude Desktop) see and run Captain's projects, containers, and logs through the Model Context Protocol (MCP). It is optional: nothing is exposed until the user turns it on, and each agent is connected only after the user approves it.

## Why

Agents already run `docker`, but raw output is long and hides what matters. Existing container MCP servers either proxy the raw CLI or dump `inspect` JSON: [ckreiling/mcp-server-docker](https://github.com/ckreiling/mcp-server-docker) returned about 139 KB for 22 containers ([#66](https://github.com/ckreiling/mcp-server-docker/issues/66)), and Docker's catalog server is one tool that takes CLI arguments ([docker](https://hub.docker.com/mcp/server/docker/overview)). None gives crash reasons, a project view, named tasks, or disk use. Captain has all of these, and a UI where the user can see what an agent did. Rancher Desktop has no MCP server ([rancher-desktop#9118](https://github.com/rancher-sandbox/rancher-desktop/issues/9118)).

## Protocol and SDK

- Target MCP [2026-07-28](https://modelcontextprotocol.io/specification/2026-07-28/changelog): stateless requests with `server/discover`, `InputRequiredResult` in place of server-to-client requests, list results with `ttlMs` and `cacheScope`, and tools listed in a fixed order. Keep working with clients on [2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/changelog).
- Transport: stdio only. stdout carries only MCP messages; logs go to stderr ([stdio](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio.md)). stdio servers take no OAuth ([authorization](https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization.md)).
- SDK: [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk) 3.x, a Tier 1 official SDK ([SDK tiers](https://modelcontextprotocol.io/docs/2026-07-28/sdk.md)), with `#[tool_router]` and `#[tool(annotations(read_only_hint = true))]`. Check its license note (moving from MIT to Apache-2.0) against Captain's `MIT OR Apache-2.0`.
- Every tool declares an `outputSchema` and returns `structuredContent` plus a short text copy ([tools](https://modelcontextprotocol.io/specification/2026-07-28/server/tools.md)). Tool failures and refusals are results with `isError: true`, not protocol errors.

## Tools

Few tools, named for what the user sees in Captain. Fewer tools also help agents pick the right one ([kubernetes-mcp-server](https://github.com/containers/kubernetes-mcp-server)).

Read tools (annotations: `readOnlyHint: true`, `openWorldHint: false`, so VS Code runs them without a prompt — [VS Code MCP](https://code.visualstudio.com/api/extension-guides/ai/mcp)):

| Tool | Returns |
|---|---|
| `engine_status` | Captain Engine state, version, CPUs, memory, disk, Kubernetes on or off. |
| `list_projects` | One row per Compose project, namespace, and "loose containers": services running of total, health, published ports as URLs, and a problem line. |
| `list_containers` | Projected rows (name, project, service, image, state, health, ports, uptime), never raw `inspect`. A `status_only` flag for the shortest form. |
| `container_problems` | The crash tracker's view: crash loops, exit codes, out-of-memory kills, restart counts, failing health checks, and the fix Captain would offer. |
| `logs` | One container or a whole project, merged. Defaults: tail 100, `since`, `errors_only`, `grep`; a hard cap (about 500 lines or 32 KB) with `truncated: true` and the next-call hint. Stays under Claude Code's 10k-token warning ([Claude Code MCP](https://code.claude.com/docs/en/mcp)). |
| `inspect` | A container's summary: command, ports, mounts, networks, limits, restart policy, environment keys with secret-looking values masked. |
| `disk_usage` | The Storage page's categories, largest items, and what cleanup would free (read only). |
| `wait_for_healthy` | Waits until a project or container is running and healthy, with a timeout. Saves agents from polling ([tilt-mcp](https://github.com/rrmistry/tilt-mcp)). |
| `help` | How Captain names things and what each tool does. Zed cannot read resources ([Zed MCP](https://zed.dev/docs/ai/mcp)), so the guidance also lives here. |

Action tools, registered only when the user allows them (annotations: `readOnlyHint: false`, `destructiveHint: false`, `idempotentHint: true` where true):

| Tool | Allowed by |
|---|---|
| `start`, `stop`, `restart` (container or project) | `agent_tools.actions` includes the verb |
| `run_task` (a task from `x-captain.tasks`) | `run_task` allowed; only tasks the user declared |
| `raise_memory` | `raise_memory` allowed; same rule as the app (twice, at least 512 MB) |

Never offered: `exec`, running an arbitrary image, a raw CLI passthrough, remove, prune, `compose down`, snapshot restore, engine reset, settings changes. `run_task` covers the safe use of exec: the user wrote each task.

Resources (for clients that read them): the settings reference and the user guide pages, and one `captain://project/<name>` summary per project.

## Safety

1. **The allow-list in Captain is the gate.** Annotations are hints that clients must treat as untrusted ([tools](https://modelcontextprotocol.io/specification/2026-07-28/server/tools.md)); each client's own approval prompt is a second check. A tool that is off is not listed, and a call to it returns `isError` naming the setting.
2. **No elicitation needed.** It changed shape in 2026-07-28 and Zed lacks it; if Captain ever asks, a client without support means "deny", not "allow" (unlike [kubernetes-mcp-server's default](https://github.com/containers/kubernetes-mcp-server/blob/main/docs/configuration.md)).
3. **Arguments match real names.** Container, project, service, and task names must match the live lists; values that start with `-` are rejected. See the token leak in [mcp-server-kubernetes#328](https://github.com/Flux159/mcp-server-kubernetes/issues/328).
4. **Container output is untrusted.** Logs, labels, and env are attacker-controllable text ([DockerDash](https://noma.security/blog/dockerdash-two-attack-paths-one-ai-supply-chain-crisis/); [log injection](https://github.com/Flux159/mcp-server-kubernetes/issues/294); [the lethal trifecta](https://simonwillison.net/2025/Jun/16/the-lethal-trifecta/)). Captain wraps them in clear delimiters labeled "untrusted container output", masks secret-looking values in logs as well as `inspect`, and never turns label text into instructions, URLs, or images.
5. **Audit.** Every action call goes to `captain.log`, the status bar's latest-event line, and an "Agent activity" list with the client name, the tool, the arguments, and the result.

## Connecting agents

Settings shows one row, "AI agents", with a status and **Set up…**, like the Terminal row. The sheet follows Docker Desktop's MCP Toolkit ([toolkit](https://docs.docker.com/ai/mcp-catalog-and-toolkit/toolkit/)) and JetBrains ([MCP server](https://www.jetbrains.com/help/ai-assistant/mcp.html)):

1. **Enable agent tools** switch, with a summary of what agents can see.
2. **Actions** checkboxes: start, stop, restart, run tasks, raise memory. All off by default.
3. **One row per agent Captain finds**, with **Connect** or **Remove** and its state. Connect first shows the exact command or JSON it will use, as the MCP security guide requires ([best practices](https://modelcontextprotocol.io/docs/2026-07-28/tutorials/security/security_best_practices.md)). It uses each client's own installer where there is one, so the client shows its own consent prompt:

   | Client | How Captain connects it |
   |---|---|
   | Claude Code | `claude mcp add --scope user --transport stdio captain -- /Users/<you>/.captain/bin/captain mcp` (user scope; the default local scope covers one folder only) ([Claude Code MCP](https://code.claude.com/docs/en/mcp)) |
   | Codex (and the ChatGPT desktop app) | `codex mcp add captain -- <path> mcp` ([Codex MCP](https://learn.chatgpt.com/docs/extend/mcp?surface=cli), [`mcp_cmd.rs`](https://github.com/openai/codex/blob/main/codex-rs/cli/src/mcp_cmd.rs)) |
   | Gemini CLI | `gemini mcp add --scope user captain <path> mcp`; its default scope is the project ([Gemini CLI](https://github.com/google-gemini/gemini-cli/blob/main/docs/tools/mcp-server.md)) |
   | VS Code | `code --add-mcp '<json>'`, or an edit of the profile's `mcp.json` (`servers`) without `code` on PATH ([VS Code](https://code.visualstudio.com/docs/agent-customization/mcp-servers), [configuration](https://code.visualstudio.com/docs/copilot/reference/mcp-configuration)) |
   | Cursor | a `cursor://anysphere.cursor-deeplink/mcp/install` link, which Cursor confirms ([install links](https://cursor.com/docs/context/mcp/install-links)); Remove edits `~/.cursor/mcp.json` ([Cursor MCP](https://cursor.com/docs/context/mcp)) |
   | Zed | `context_servers` in Zed's settings, edited in place with comments kept ([Zed MCP](https://zed.dev/docs/ai/mcp), [configuring Zed](https://github.com/zed-industries/zed/blob/main/docs/src/configuring-zed.md)) |
   | Claude Desktop | `mcpServers` in `claude_desktop_config.json`; tell the user to restart Claude ([connect local servers](https://modelcontextprotocol.io/docs/develop/connect-local-servers)) |

   Paths are absolute, never `~`.
4. **Copy config** for any other client.
5. **Status:** the last client that connected and its last call, and the "Agent activity" list.

## Build plan

Six phases, each one pull request with its own tests. Phases 2 and 3 can run in parallel after 1; 4 needs 2; 5 needs 1 and 4; 6 closes the milestone.

| # | Phase | Delivers | Done when |
|---|---|---|---|
| 1 | Server skeleton | `captain mcp` on rmcp over stdio; connects to the engine the settings choose (as the CLI does); `server/discover`; the `help` tool; stderr-only logging; a test that drives the server with rmcp's client. | An MCP client lists `help` and calls it. |
| 2 | Read tools | `engine_status`, `list_projects`, `list_containers` (`status_only`), `container_problems` (the crash tracker, shared with the app), `inspect` (masked), `disk_usage`; `outputSchema` + `structuredContent` + text; annotations; names checked against live lists. | Each tool has a schema test and a size test (22 containers stay under 8 KB). |
| 3 | Logs and waiting | `logs` (container or project, tail, `since`, `errors_only`, `grep`, caps, `truncated`), secret masking in log lines, untrusted-output delimiters, `wait_for_healthy`. | A log line with injected instructions comes back inside the delimiters; caps hold. |
| 4 | Actions and audit | The `agent_tools` settings group (`enabled`, `actions`) in the schema and reference; `start`, `stop`, `restart`, `run_task`, `raise_memory` listed only when allowed; refusals as `isError` naming the setting; an activity log file (`~/.captain/agent-activity.jsonl`) that the app watches for the status bar and the Agent activity list. | A call to an action that is off is refused; an allowed one runs and shows in the app. |
| 5 | Connect agents | Detection of Claude Code, Codex, Gemini CLI, VS Code, Cursor, Zed, and Claude Desktop; Connect and Remove through each client's installer (or an in-place edit for Zed and Claude Desktop), showing the exact command first; the "AI agents" Settings row and sheet; Copy config. | Connect then Remove leaves each client's configuration as it was. |
| 6 | Docs and verification | `docs/reference/mcp.md` generated from the tool descriptions with a drift test; `docs/guide/agents.md`; the Verification steps below run by hand; ROADMAP and testing guide updated. | CI green on three OSes; the hand checks pass. |

After M31: an Agent Skill and an AGENTS.md snippet (see Later).

## Built: phases 1–3

**SDK.** [`rmcp` 3.5.0](https://crates.io/crates/rmcp/3.5.0) ([docs](https://docs.rs/rmcp/3.5.0/rmcp/), [repository](https://github.com/modelcontextprotocol/rust-sdk), [stdio example](https://github.com/modelcontextprotocol/rust-sdk/blob/main/examples/servers/src/counter_stdio.rs), [structured output example](https://github.com/modelcontextprotocol/rust-sdk/blob/main/examples/servers/src/structured_output.rs)) with `default-features = false` and `server`, `macros`, `transport-io`; the tests add `client`. These pull only pure-Rust crates (tokio, tokio-util, schemars 1, uuid, pastey); `cargo check --target x86_64-pc-windows-msvc` passes. The crate is Apache-2.0. The [repository LICENSE](https://github.com/modelcontextprotocol/rust-sdk/blob/main/LICENSE) says the project moves from MIT to Apache-2.0, and older contributions stay MIT until relicensed. Both are permissive and compatible with Captain's `MIT OR Apache-2.0`; the Captain binary must keep rmcp's Apache-2.0 notice, as with its other Apache-2.0 dependencies.

**Protocol.** rmcp answers `initialize` and `server/discover` and negotiates every version it knows, 2024-11-05 through 2026-07-28 (`supportedVersions` in the discover result). `tools/list` is sorted by name, so its order is fixed, and carries `ttlMs` and `cacheScope` for 2026-07-28 clients. `get_info` gives the server name `captain`, the title, and short instructions that point to `help`.

**`captain mcp`** (crates/captain-cli/src/commands/mcp.rs, crates/captain-cli/src/mcp/). It reads the settings like the other commands and connects to the engine they choose, as the app does at launch: Captain Engine's socket, or `engine_endpoint`, or discovery (`DOCKER_HOST`, the current context, then known sockets; `ssh://` goes through the tunnel). It connects on the first tool call, on a blocking thread, so an engine that is down gives a tool error ("Captain Engine does not answer … run `captain start`") and not a failed start. After an unreachable error it connects again on the next call. The first connection also follows the engine's events into `CrashTracker`, so a crash loop that is running between restarts counts, as in the app. Logs go to stderr through `tracing-subscriber` (level from `CAPTAIN_LOG`, default `warn`); stdout carries only protocol messages.

**Where the logic lives.** `captain_core::agent_tools` has no protocol code and is unit-tested alone: the answer structs (with `Serialize` and `JsonSchema`, so each tool's `outputSchema` comes from the same type as its `structuredContent`), their text copies, name checks, log caps, secret masking, and the untrusted-output delimiters. The CLI's tool methods only fetch from the engine and call it. Shared with the app, not copied: `problems::container_problems` (the menu bar's `first_problem` now takes its first entry), `Problem::container_fixes` (the tray's fix items use it), `ExitFacts::of`, `Container::is_sandbox`, `HostStatus::key` and `detail` (`captain status` uses them), `EnvVar`'s secret markers (`is_secret_key`), `PortLink`, and the Storage page's `DiskBreakdown`, `ReclaimPlan`, and `largest_items`. `LogOptions` gained `follow`; the tools read past lines only.

**Tool results.** Each tool declares `outputSchema` (rmcp's `schema_for_output`, JSON Schema 2020-12) and returns `structuredContent` plus one text block. The text block is a compact line form of the same facts, not a summary, because some clients show only text. Failures and refusals (an unknown name, an engine that does not answer, both `container` and `project`) are results with `isError: true`. Every tool has `readOnlyHint: true`, `openWorldHint: false`, and a title. The doc comments of the tool methods are the descriptions; phase 6 generates `docs/reference/mcp.md` from them.

| Tool | Notes |
|---|---|
| `help` | The guide text and every listed tool with its description, from the live router. |
| `engine_status` | Engine, state (`HostStatus::key`, or `unreachable` for another engine), version, platform, CPUs, memory, Captain Engine's disk, Kubernetes, containers running of total. |
| `list_projects` | Sidebar groups: Compose projects, `k8s:<namespace>`, and "Loose containers". Running of total, services, health counts, ports as links (`http://localhost:8080`, or `localhost:5432 (Postgres)` for ports a browser cannot open), the Compose folder, and the worst problem line from `first_problem`. |
| `list_containers` | Rows without pod sandboxes. `status_only` keeps name, state, health, and `needs_attention`. `project` filters to one project. 22 containers stay under 8 KB for JSON and text together (a unit test). |
| `container_problems` | `container_problems` over the containers, with `inspect` of each container that restarts, crashed in the last minute, or fails its health check: kind, Captain's sentence, exit code, restart count, memory limit, and the fixes ("Raise its memory limit to 512 MB.", "Read its logs with the logs tool.", "Stop it."). An engine that does not run is the `engine` field, not an error. |
| `inspect` | Command, ports, links, mounts, networks, restart policy, limits, last exit, and environment. Values with a secret-looking key become `[masked]`; other values go through the log masker. The text copy puts the command and environment between the delimiters. |
| `disk_usage` | Used and capacity, the categories, the 10 largest items (Kubernetes users shown as `pod/container`), each cleanup group's count and size, and what the default cleanup frees. Snapshots and capacity count for Captain Engine only. |
| `logs` | `container` or `project` (merged by time, each line `HH:MM:SS service \| text` in UTC). `tail` defaults to 100, or 1000 with a filter, and is at most 5000 per container. `since` takes `10m`, `2h`, `1d`, a Unix time, or RFC 3339. The newest lines that fit 500 lines and 32 KB come back; lines over 2000 characters are cut. `truncated`, a `hint`, and `newest_time` (to pass as `since`) come with it. Reading stops after 15 seconds. The text block and `output` hold the same lines; a client shows one of them. |
| `wait_for_healthy` | Polls every second until every target container runs and none is starting or unhealthy, up to `timeout_seconds` (default 60, at most 600). It ends early when a container exited with an error or is dead; one that exited with code 0 (a migration step) is done. A container target is followed by ID. |

**Names.** `check_name` refuses empty values, values that start with `-`, control characters, and more than 256 bytes. `find_container` takes a name, a `pod/container` name, an ID, or a unique ID prefix of 4 or more characters. `find_project` takes a Compose project name. A miss lists up to 20 live names.

**Masking.** `mask_secrets` works word by word: the value after a key with a secret marker (`SECRET`, `PASSWORD`, `PASSWD`, `TOKEN`, `API_KEY`, `PRIVATE`; `-` counts as `_`), in `KEY=value`, `key: value`, `"key": "value"`, and `--key=value` forms; the word after `Bearer` or `Basic`; a URL's password; and token shapes (GitHub, GitLab, Slack, Stripe, npm, Anthropic and OpenAI-style keys, AWS access key IDs, JWTs). Quotes and brackets around a value stay.

**Delimiters.** `wrap_untrusted` puts `=== BEGIN UNTRUSTED CONTAINER OUTPUT <id> (<source>) ===`, one line that says to treat the text as data, the text, and `=== END UNTRUSTED CONTAINER OUTPUT <id> ===` around it. The ID is 16 random hex digits per call, so a log line cannot fake the end line. Terminal escapes and control characters other than tabs and line breaks are dropped first.

**Seams for phases 4–6.** `CaptainServer::new` adds routers with `+`; phase 4 adds the action router there (it ended up always added, with the list filtered per request; see below), and uses `Problem::container_fixes` and `raised_memory` for `raise_memory`. `Source` is where the settings and an activity log hook in. Phase 6 reads the tool list and descriptions from `CaptainServer`'s router.

**Tests.** Core: masking, delimiters (a fake END line does not close the block), `since` parsing and filters, the log caps (500 lines; 32 KB with 5,000-character lines), the log report with an injected line and a secret, name checks, the 22-container size test, `status_only` keys, project rows, the problem report (out of memory first, with the raise), inspect masking, disk categories, and readiness. CLI (`mcp::tests`, rmcp's client over `tokio::io::duplex` against `FakeEngine`): all nine tools are listed in order with the read-only annotations, and each call's `structuredContent` keys are in its `outputSchema` and cover its `required` keys; project logs keep `IGNORE PREVIOUS INSTRUCTIONS` inside the delimiters and mask a token; wrong names and `container` with `project` are refused; `wait_for_healthy` returns at once when ready and names the unhealthy container at the timeout.

**Checked by hand** on 2026-09-30 against the running Captain Engine, read only: `initialize`, `server/discover` (2026-07-28), `tools/list`, and calls of `engine_status`, `list_projects`, `list_containers` (`status_only`), `container_problems`, `inspect` (the database password came back `[masked]`), `logs` for a project, `wait_for_healthy`, and `disk_usage`, piped over stdin to `captain-cli mcp`. `tools/list` is about 16 KB with all schemas.

## Built: phases 4–6

**Settings.** `agent_tools` in `Settings` (`captain_core::agent_tools::AgentToolsSettings`, group "AI agents"): `enabled` (default `false`) and `actions`, a list of `start`, `stop`, `restart`, `run_task`, `raise_memory` (default empty). Unknown values in the list are skipped, and the file check names them. The schema, docs/reference/settings.md, and settings.schema.json come from the type; the reference shows the list as "a list of any of …".

**The gate.** `AgentToolsSettings::gate(tool)`: `help` always answers; every other tool needs `enabled`; an action also needs its name in `actions`. `captain mcp` reads the settings file on every request (the path `--settings` or the app's own), so a change in Captain applies at once. `CaptainServer` implements `ServerHandler` itself instead of `#[tool_handler]`: `tools/list` returns only the allowed tools (with `ttlMs: 0`), `get_tool` too, and a call to a known tool that is off returns `isError` with the setting's name ("The restart action is off. The user can allow it in Captain under Settings > AI agents (add \"restart\" to the agent_tools.actions setting)."). The server declares `tools.listChanged` and, after `notifications/initialized`, reads the settings every 2 seconds and sends `notifications/tools/list_changed` when the list changes.

**Actions** (crates/captain-cli/src/mcp/action_tools.rs; annotations `readOnlyHint: false`, `destructiveHint: false`, `openWorldHint: false`, `idempotentHint` true for start and stop):

| Tool | Does |
|---|---|
| `start` | A container: the Engine API start. A project: `docker compose up -d`, the app's Up. |
| `stop`, `restart` | A container through the Engine API, or a project through `docker compose stop` / `restart`. |
| `run_task` | `project` and `task`. Reads `x-captain.tasks` through the same `ProjectRunner::tasks` and `run_task` as the project page (`docker compose exec -T`). An unknown task lists the declared ones. The output is masked, capped like logs (500 lines, 32 KB, the end kept), and wrapped in the untrusted-output delimiters (`TaskReport`). |
| `raise_memory` | `project_map::raised_memory`: twice the limit, at least 512 MB, through `update_memory`. A container without a limit is refused, as `Problem::container_fixes` offers no raise then. |

Project actions need `docker compose`; `captain mcp` finds it with `ComposeCli::detect` when it connects, as the app does.

**Activity log.** `~/.captain/agent-activity.jsonl`, one JSON line per call of a known tool other than `help` (read tools too, so the user sees what agents read): `at`, `client` (from `clientInfo`, per request on 2026-07-28), `tool`, `arguments` (strings cut at 300 characters), `ok`, and the first line of the answer. Each line is one append; past 256 KB the older half goes (an atomic replace). The app polls the file's size and time every 2 seconds (`agents::activity_watch`), writes each new action to `captain.log`, shows the latest call of the last ten minutes as a status bar segment ("Claude Code: restart", orange for an action, red for a failure or refusal, gray for a read), and lists the last 20 in the sheet.

**Connecting agents** (`captain_core::agent_clients`). `detect` finds a client by its command in the login shell (`$SHELL -lic 'command -v claude codex gemini code cursor zed'`; on Windows, a search of `PATH` with each `PATHEXT` extension) or by its settings folder, and reads whether its file lists `captain` (JSONC for the JSON files; a `[mcp_servers.captain]` table for Codex). `connect_step` and `remove_step` give a `ClientStep`, or an error when chezmoi manages the client's file, since every step changes that file: `Run` (the installer argv), `Open` (Cursor's link: the entry as base64, URL-escaped), or `Edit` (the file's text before and after, from `with_server` / `without_server`, which use the jsonc-parser CST so comments and other keys stay; Remove also drops a servers object it leaves empty when the backup shows that Captain added it and no comment is left in it, so Connect then Remove gives the file back byte for byte). The sheet shows `preview()` (the shell line, the link, or the changed lines) and runs the step only on the user's click: a command through the login shell, so `node` and the client's command are on PATH as in a terminal (on Windows, the file that the `PATH` search finds, run directly); an edit stages the new file and makes a `.captain-backup` copy the first time (an empty one for a file Captain creates), then checks that the file did not change since it was read right before the `file_replace` rename, through `link_target` for symlinked dotfiles. Paths: `~/.claude.json`, `$CODEX_HOME/config.toml`, `~/.gemini/settings.json`, the VS Code profile `mcp.json`, `~/.cursor/mcp.json`, `~/.config/zed/settings.json` (`%APPDATA%\Zed` on Windows), and `claude_desktop_config.json` under the app-config folder (macOS and Windows only; [connect local servers](https://modelcontextprotocol.io/docs/develop/connect-local-servers)). The `captain` path is `~/.captain/bin/captain` when the command-line tools linked it, else the copy in Captain.app, else `captain-cli` (`captain-cli.exe` on Windows) next to the app in a development build. Copy config gives an `mcpServers` object for any other client.

**UI.** Settings has an "AI agents" row (on or off, the allowed actions, the last call) with **Set up…**. The sheet (crates/captain-ui/src/agents/) has the **Let agents use Captain** switch, the five action checkboxes, a row per agent found with Connect or Remove and the step to confirm, Copy config, Check again, and the Agent activity list. Every control has a help sentence.

**Docs.** docs/reference/mcp.md is generated from the server's tool list (`captain docs mcp`), with a drift test in `mcp::reference::tests`. docs/guide/agents.md is the user guide. docs/testing.md section 15 has the hand checks.

**Tests.** Core: the gate and unknown actions; the activity log's order, broken lines, and cap; the raise rule and task output (masked, capped, delimited); JSONC connect-then-remove round trips (a Zed file with comments and trailing commas, a plain file), a blank and a broken file; Codex tables; the installer argv for Claude Code, Gemini CLI, and Codex, and Cursor's link decoding back to the entry; detection in a temporary home; a stub installer that records its arguments; an edit that refuses a changed file and keeps a backup. CLI (rmcp's client): with the tools off only `help` is listed and a call names `agent_tools.enabled`; a disallowed restart names `agent_tools.actions`; after allowing all five, they are listed, a container restart and a project stop run, `run_task` returns the fake task's output and lists the declared tasks for a wrong name, `raise_memory` refuses a container without a limit, and the log holds each call in order. No test runs a real installer or touches a real client file.

**Checked by hand** on 2026-09-30 against Captain Engine with a temporary home and settings file: `tools/list` with `enabled` alone gives the nine read tools; `restart` is refused naming `agent_tools.actions`; `list_projects` answers; both calls are in the activity file with the client name.

## Out of scope

- A network transport (Streamable HTTP). If ever added: a bearer token and localhost-only origins, as [Docker's gateway](https://github.com/docker/mcp-gateway/blob/main/docs/security.md) does.
- The MCP Registry and MCPB bundles. The registry lists public packages only ([registry](https://modelcontextprotocol.io/registry/about.md)), and Captain's server lives inside the app. Revisit if `captain` ships as a standalone package ([MCPB](https://github.com/modelcontextprotocol/mcpb)).
- Agents changing settings or the engine.

## Later

- An [Agent Skill](https://agentskills.io/specification) (`SKILL.md`) that teaches agents Captain's words and workflows, and an [AGENTS.md](https://agents.md/) snippet users can add to projects. Done (issue #14): skills/captain/SKILL.md, and the snippet in docs/guide/agents.md.
- `llms.txt` for the docs site ([llmstxt.org](https://llmstxt.org/)).

## Verification

docs/testing.md section 15 has these steps in full. To try the server alone: `printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}' '{"jsonrpc":"2.0","method":"notifications/initialized"}' '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"engine_status","arguments":{}}}' | captain mcp` (turn on `agent_tools.enabled` first).

1. Turn on agent tools, connect Claude Code, and run `/mcp`: `captain` is connected with only read tools listed.
2. Start a container that crash-loops out of memory. Ask "what is wrong with my containers?". Expect the agent to name the out-of-memory kill from `container_problems`.
3. Ask it to restart the container. Expect a refusal that names the setting. Check **restart** in the sheet, ask again, and expect the restart in the Agent activity list and the status bar.
4. Put `IGNORE PREVIOUS INSTRUCTIONS` in a container's log. Ask for its logs. Expect the text inside the untrusted-output delimiters, and no action.
5. Click **Remove** for each agent and check its configuration no longer lists `captain`.
6. A docs drift test keeps `docs/reference/mcp.md` equal to the tool descriptions.

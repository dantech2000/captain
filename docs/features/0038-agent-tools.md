# Feature 0038: Agent tools (MCP)

- Milestone: M31
- Status: Planned. Research done on 2026-09-29; sources are linked in each section.

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
   | Claude Code | `claude mcp add --scope user captain -- /Users/<you>/.captain/bin/captain mcp` (user scope; the default local scope covers one folder only) |
   | Codex (and the ChatGPT desktop app) | `codex mcp add captain -- <path> mcp` ([Codex MCP](https://learn.chatgpt.com/docs/extend/mcp?surface=cli)) |
   | Gemini CLI | `gemini mcp add captain <path> mcp` ([Gemini CLI](https://geminicli.com/docs/tools/mcp-server/)) |
   | VS Code | `code --add-mcp '<json>'` or a `vscode:mcp/install` link ([VS Code](https://code.visualstudio.com/docs/copilot/chat/mcp-servers)) |
   | Cursor | a `cursor://anysphere.cursor-deeplink/mcp/install` link, which Cursor confirms ([install links](https://cursor.com/docs/context/mcp/install-links)) |
   | Zed | `context_servers` in Zed's settings, edited in place with comments kept |
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

## Out of scope

- A network transport (Streamable HTTP). If ever added: a bearer token and localhost-only origins, as [Docker's gateway](https://github.com/docker/mcp-gateway/blob/main/docs/security.md) does.
- The MCP Registry and MCPB bundles. The registry lists public packages only ([registry](https://modelcontextprotocol.io/registry/about.md)), and Captain's server lives inside the app. Revisit if `captain` ships as a standalone package ([MCPB](https://github.com/modelcontextprotocol/mcpb)).
- Agents changing settings or the engine.

## Later

- An [Agent Skill](https://agentskills.io/specification) (`SKILL.md`) that teaches agents Captain's words and workflows, and an [AGENTS.md](https://agents.md/) snippet users can add to projects. Both are cheap once the server exists.
- `llms.txt` for the docs site ([llmstxt.org](https://llmstxt.org/)).

## Verification

1. Turn on agent tools, connect Claude Code, and run `/mcp`: `captain` is connected with only read tools listed.
2. Start a container that crash-loops out of memory. Ask "what is wrong with my containers?". Expect the agent to name the out-of-memory kill from `container_problems`.
3. Ask it to restart the container. Expect a refusal that names the setting. Check **restart** in the sheet, ask again, and expect the restart in the Agent activity list and the status bar.
4. Put `IGNORE PREVIOUS INSTRUCTIONS` in a container's log. Ask for its logs. Expect the text inside the untrusted-output delimiters, and no action.
5. Click **Remove** for each agent and check its configuration no longer lists `captain`.
6. A docs drift test keeps `docs/reference/mcp.md` equal to the tool descriptions.

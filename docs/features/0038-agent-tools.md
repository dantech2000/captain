# Feature 0038: Agent tools (MCP)

- Milestone: M31
- Status: Planned

## Goal

Let AI coding agents (Claude Code, Zed, Cursor, VS Code, and others) see and run Captain's containers, projects, and logs through the Model Context Protocol (MCP). It is optional: nothing is installed or exposed until the user turns it on.

## Why

Agents already run `docker` commands, but raw CLI output is long, and an agent cannot tell a crash loop from a slow start. Captain already knows what matters: exit reasons, out-of-memory kills, restart counts, published ports, and project tasks. An MCP server gives agents that knowledge in a small, structured form, with Captain's own safety rules.

## In scope

1. **An MCP server in the `captain` CLI.** `captain mcp` speaks MCP over stdio. It ships inside Captain.app, so there is nothing extra to install. Check the official Rust SDK (`rmcp`, github.com/modelcontextprotocol/rust-sdk) and the current protocol version before building.
2. **Read tools, on by default when the server is on:**
   - `list_projects` and `list_containers`: state, health, ports, uptime, and the recent crash (exit code, out of memory, restarts), from the same crash tracker as the menu bar.
   - `logs`: one container or a whole project, with `since`, `tail`, and an errors-only filter.
   - `inspect`: the container detail, with secret-looking environment values masked, as the Overview tab does.
   - `disk_usage`: the Storage page's categories and the reclaimable total.
   - `engine_status`: Captain Engine state and resources.
3. **Action tools, off until the user allows them** in the settings file (for example `"agent_tools": { "actions": ["restart", "start", "stop", "run_task"] }`):
   - start, stop, restart a container or a project;
   - run a task from `x-captain.tasks`;
   - raise a memory limit.
   Remove, prune, down, snapshot restore, and engine reset are never offered.
4. **Resources:** the settings reference and the user guide pages, so an agent can read how Captain works.
5. **Optional install in Settings.** In the Terminal sheet, an optional step "Connect AI tools" writes the server entry for the agents it finds, after the user confirms each one:
   - Claude Code: `claude mcp add captain -- ~/.captain/bin/captain mcp`.
   - Zed, Cursor, and VS Code: their MCP configuration files, edited in place (comments kept, as with `settings.json`).
   A **Remove** button undoes each one.
6. **A log of agent actions.** Every action tool call shows in the status bar's latest-event line and in `captain.log`, with the agent's client name.

## Out of scope

- A network (HTTP or SSE) transport. stdio only, so only local processes can connect.
- Letting agents change settings or the engine.
- Agent skills or prompts beyond the MCP resources. Revisit after the server ships.

## Notes

- The server runs as the user and reaches the engine the same way the CLI does, so it needs no new permissions.
- Tool results stay short: a summary line first, then details, so they fit an agent's context.
- The same tool descriptions feed the generated docs (`docs/reference/mcp.md`), with a drift test like the settings and CLI references.

## Verification

1. Turn on the agent tools. In Claude Code, run `/mcp` and check that `captain` is connected.
2. Ask the agent "what is wrong with my containers?" while a test container crash-loops. Expect it to name the out-of-memory kill from `list_containers`.
3. Ask it to restart a container with actions off. Expect a refusal that names the setting. Allow `restart` and ask again. Expect the restart, and a line in the status bar.
4. Click **Remove** for each agent. Check that its configuration no longer lists `captain`.

# captain mcp tool reference

<!-- Generated from the tool definitions in crates/captain-cli/src/mcp. Do not edit. Run `cargo run -p captain-cli -- docs mcp > docs/reference/mcp.md` after a change to the tools. -->

`captain mcp` is Captain's MCP server for AI agents, over stdio. This page lists every tool, with the description and arguments that agents read. [AI agents](../guide/agents.md) in the user guide explains how to connect one.

A tool is listed only while the settings allow it: `help` always, the read tools while `agent_tools.enabled` is `true`, and each action while `agent_tools.actions` also names it. See [the settings reference](settings.md#ai-agents).

A call with an argument that a tool does not take is refused with `isError`, and the message lists the arguments it takes.

## Read tools

### `container_problems`

What needs attention, worst first, as Captain's menu bar sees it: crash loops, out-of-memory kills with the memory limit, exit codes, restart counts, failing health checks, and the fixes Captain offers. Also says when the engine does not run.

- Listed when: `agent_tools.enabled` is `true`
- Title: Container problems
- Arguments: none

### `disk_usage`

What the engine stores, as Captain's Storage page shows it: the categories, the largest items and who uses them, and what each cleanup group would free. It only reads; agents cannot clean up.

- Listed when: `agent_tools.enabled` is `true`
- Title: Disk usage
- Arguments: none

### `engine_status`

Which engine Captain uses and its state: running or why not, Docker version, CPUs, memory, disk, Kubernetes on or off, and how many containers run.

- Listed when: `agent_tools.enabled` is `true`
- Title: Engine status
- Arguments: none

### `help`

How Captain names things, what each tool does, and how container output is marked. Read it first.

- Listed when: always
- Title: Help
- Arguments: none

### `inspect`

One container's summary: command, ports, mounts, networks, limits, restart policy, last exit, and environment with secret-looking values masked. Never the raw inspect JSON.

- Listed when: `agent_tools.enabled` is `true`
- Title: Inspect a container
- Arguments:
  - `container` (string, required): The container: its name, its pod/container name, or an ID prefix.

### `list_containers`

Every container as a short row: name, state, health, needs_attention, ID, project, service, image, published ports as URLs, and uptime. Pass status_only for the shortest form, or project for one Compose project.

- Listed when: `agent_tools.enabled` is `true`
- Title: List containers
- Arguments:
  - `project` (string, optional): Only the containers of this Compose project.
  - `status_only` (boolean, optional): Only name, state, health, and needs_attention: the shortest answer.

### `list_projects`

One row per Compose project, Kubernetes namespace, and the loose containers: containers running of total, services, health, published ports as URLs, the Compose folder, and the worst problem.

- Listed when: `agent_tools.enabled` is `true`
- Title: List projects
- Arguments: none

### `logs`

The recent output of one container, or of a whole Compose project merged in time order. Defaults to the last 100 lines. Filters: service (with project), since, errors_only, grep. At most 500 lines or 32 KB come back, newest kept, with truncated and a hint. The lines are between UNTRUSTED CONTAINER OUTPUT delimiters and secret-looking values are masked.

- Listed when: `agent_tools.enabled` is `true`
- Title: Logs
- Arguments:
  - `container` (string, optional): One container: its name, its pod/container name, or an ID prefix.
  - `project` (string, optional): A whole Compose project, its containers' lines merged in time order.
  - `service` (string, optional): With project: only this Compose service's lines.
  - `tail` (integer, optional): Past lines per container, before the filters. Default 100, or 1000 with errors_only or grep; at most 5000.
  - `since` (string, optional): Only lines from this time on: an age such as 10m, 2h, or 1d, a Unix time, or an RFC 3339 time. Pass newest_time from an earlier answer to read on.
  - `errors_only` (boolean, optional): Only lines that look like errors (error, fatal, panic, exception).
  - `grep` (string, optional): Only lines that contain this text, ignoring case.

### `wait_for_healthy`

Waits until a container, or every container of a Compose project, runs and passes its health check. Stops early when one exits with an error. Default timeout 60 seconds, at most 600. Use it instead of polling.

- Listed when: `agent_tools.enabled` is `true`
- Title: Wait until healthy
- Arguments:
  - `container` (string, optional): One container: its name, its pod/container name, or an ID prefix.
  - `project` (string, optional): Every container of a Compose project.
  - `timeout_seconds` (integer, optional): How long to wait, in seconds. Default 60, at most 600; 0 looks once.

## Actions

### `raise_memory`

Raises a container's memory limit the way Captain does after an out-of-memory kill: twice the old limit, at least 512 MB. The container keeps running. A container without a limit is refused.

- Listed when: `agent_tools.enabled` is `true` and `agent_tools.actions` includes `"raise_memory"`
- Title: Raise memory
- Arguments:
  - `container` (string, required): The container: its name, its pod/container name, or an ID prefix.

### `restart`

Restarts a container, or every service of a Compose project. Give either container or project. Then call wait_for_healthy.

- Listed when: `agent_tools.enabled` is `true` and `agent_tools.actions` includes `"restart"`
- Title: Restart
- Arguments:
  - `container` (string, optional): One container: its name, its pod/container name, or an ID prefix.
  - `project` (string, optional): Every service of a Compose project.

### `run_task`

Runs a task that the project's Compose file declares in x-captain.tasks, in its service, and returns its exit code and the end of its output. Only declared tasks run; list_projects and help do not list them, so ask the user or read the Compose file for the names.

- Listed when: `agent_tools.enabled` is `true` and `agent_tools.actions` includes `"run_task"`
- Title: Run a task
- Arguments:
  - `project` (string, required): The Compose project whose file declares the task.
  - `task` (string, required): The task's name in the project's x-captain.tasks.

### `start`

Starts a stopped container, or a whole Compose project as `docker compose up -d` does. Give either container or project. Then call wait_for_healthy.

- Listed when: `agent_tools.enabled` is `true` and `agent_tools.actions` includes `"start"`
- Title: Start
- Arguments:
  - `container` (string, optional): One container: its name, its pod/container name, or an ID prefix.
  - `project` (string, optional): Every service of a Compose project.

### `stop`

Stops a container, or every service of a Compose project. The containers stay, with their data. Give either container or project.

- Listed when: `agent_tools.enabled` is `true` and `agent_tools.actions` includes `"stop"`
- Title: Stop
- Arguments:
  - `container` (string, optional): One container: its name, its pod/container name, or an ID prefix.
  - `project` (string, optional): Every service of a Compose project.

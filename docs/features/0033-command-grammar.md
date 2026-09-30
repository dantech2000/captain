# Feature 0033: Command grammar in the ⌘K palette

- Milestone: M27
- Status: Implemented. It needs a check by hand in the app.
- Design: the prompt and suggestion list of the `Bridge` screen, and the `V2Palette` screen. See [0027](0027-v3-interface.md) and [0005](0005-command-palette.md).
- Why: keyboard-first tools such as k9s and lazydocker are loved for their density ([lazydocker](https://github.com/jesseduffield/lazydocker), [k9s commands](https://k9scli.io/topics/commands/)). Ports and names matter more than IDs ([docker/roadmap#221](https://github.com/docker/roadmap/issues/221)).

## Goal

A user types a short command with live names in the ⌘K palette, such as `restart api`, `logs worker --since 10m`, `forward svc/web 8080`, or `disk`. The palette completes each word and runs the command on Return.

## In scope

- **Verbs.**
  - `start`, `stop`, `restart`, `pause`, `resume` with a container, a Compose service, or a project. On a project, `start` is `docker compose up`, `stop` is `docker compose stop`, and `restart` is `docker compose restart`; `pause` and `resume` go to each container.
  - `up` and `down` with a project. Down opens the existing confirmation dialog.
  - `logs <container|service> [--since 10m] [--errors]` opens the Project page with the inspector at Logs. `--since` loads only lines from that time on (`LogOptions.since`), and a chip "Since 10 minutes" clears it. `--errors` sets the level filter to Error. `logs <project>` opens the Project page and its log; the project log has no filters.
  - `shell <container|service>` opens the inspector at Terminal.
  - `open <container|service|port>` opens `http://localhost:PORT`, or copies the address of a database port, as the Open row does.
  - `forward svc/<name> [local port]` forwards a Kubernetes service through the Port Forwarding page's model, and shows that page. It works only while k3s runs.
  - `float <container|service>` opens the floating log window.
  - `disk` opens Storage. `go <page>` and a page name alone open a page.
- **Times** are a whole number and one unit: `30s`, `10m`, `1h`, `2d`. `--since=10m` works too.
- **Names resolve against live data**: container names; Compose service names in the shown project first, then in any project, or as `project/service`; project names; and `svc/<name>`, `svc/<namespace>/<name>`, either with `:port`, for Kubernetes services. A service with one container is that container. A name that stands for more than one thing is never guessed: the palette lists each one and the user picks.
- **Completions.** While the first word is typed, the palette suggests verbs and pages that start with it. After the verb, it suggests live names ranked by the fuzzy matcher, then options and times. Each row shows the command with the typed parts in bold, and the target's context in muted text ("shop · running 3 hours"). Tab puts the highlighted row in the field. Return runs a complete row; on a row that needs more words, Return completes it. The key hint shows ↵ or ⇥.
- **Plain search stays.** A line whose first word is not a verb, such as `redis`, gets the plain search results. Once the first word is a verb and more follows, only the grammar's rows show.
- **Errors.** When no row fits, the list shows why: "No container, service, or project named “x”.", "--since needs a time, such as 10m, 1h, or 30s.", "logs has no option --tail. Use --since or --errors.", "Pick a port above 1024."
- **Try.** The empty palette's first row shows up to five commands built from live names. A click puts one in the field.
- **Help.** Every palette row has a help sentence for the status bar. A row that runs uses the sentence of the button that does the same thing, for example Restart project, Down, or a container's Stop.

## Out of scope

- The Bridge screen's pinned blocks and its dashboard.
- `delete` and other commands that remove things.
- Filters on the project log.
- `restart --pull` from the Bridge mockup.

## Notes

- The grammar is pure and lives in `captain_core::grammar`: `parse` turns a line into an `Action` or a `ParseError`, `complete` returns ranked `Suggestion`s, and `examples` builds the Try row. The palette builds a `Catalog` from the workspace store and the shared `ForwardingModel`.
- `--since` maps to the `since` query parameter of `GET /containers/{id}/logs` ([ContainerLogs](https://docs.docker.com/reference/api/engine/version/v1.52/#tag/Container/operation/ContainerLogs)), as `docker logs --since 10m` does ([docker container logs](https://docs.docker.com/reference/cli/docker/container/logs/)). Without `--since`, the tab loads the last 500 lines.
- `svc/<name>` follows kubectl's resource form ([kubectl port-forward](https://kubernetes.io/docs/reference/kubectl/generated/kubectl_port-forward/)). Local ports must be above 1024, as on the Port Forwarding page.
- Project commands map to `docker compose up -d`, `stop`, `restart`, and `down` ([docker compose](https://docs.docker.com/reference/cli/docker/compose/)).
- Tab needs its own binding in the palette's search field, because the text input binds Tab to indent ([gpui-component input](https://docs.rs/gpui-component/0.7.0/gpui_component/input/index.html)). `InputState::set_value` puts the cursor at the end and sends no change event, so the palette resets its highlight itself.
- The Port Forwarding page and the palette share one `ForwardingModel`. The palette reads the services once when it opens while k3s runs.

## Verification

1. Run `cargo test -p captain-core grammar::`.
2. Open Captain with a Compose project running. Press ⌘K. Check the Try row.
3. Type `re`. Check that `restart` and `resume` show first. Press Tab. Check that the field says `restart `.
4. Type part of a service name. Check the bold characters and the context text. Press Return. Check that the service restarts.
5. Type `logs <service> --since 10m --errors` and press Return. Check that the inspector opens at Logs with Error selected and a "Since 10 minutes" chip.
6. Type `shell <service>`. Check that the Terminal tab opens.
7. Type `down <project>`. Check that the confirmation dialog opens.
8. Type `restart nothing`. Check the error text.
9. Type `redis` or another container name. Check that the plain search result still shows.
10. With Kubernetes on, type `forward svc/` and pick a service. Add a port above 1024 and press Return. Check the Port Forwarding page.
11. Hover rows. Check the help sentence in the status bar.

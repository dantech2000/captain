# The command palette

Press ⌘K, or click the search button at the top of the sidebar. The palette finds containers, images, pages, and commands, and it runs short commands such as `restart api`.

## Keys

| Key | What it does |
|-----|--------------|
| ↑ and ↓ | Move between rows. |
| Tab | Put the highlighted row in the field. |
| Return | Run the highlighted row. If the row needs more words, Return completes it instead. |
| Esc | Close the palette. |

The key hint on each row shows ↵ when Return runs it, or ⇥ when Tab completes it.

## Search

Type a word that is not a command, such as `redis`. The palette shows matching containers, images, pages, and actions, ranked by how well they match.

The empty palette starts with a **Try** row. It shows up to five commands built from your own container and project names. Click one to put it in the field.

## Commands

A command is a verb and, for most verbs, a name. The palette completes each word as you type. After the verb, it suggests names that fit, with the typed letters in bold and the context in gray, for example "shop · running 3 hours".

| Command | What it does |
|---------|--------------|
| `start <container\|service\|project>` | Starts a container, the containers of a service, or a project (`docker compose up`). |
| `stop <container\|service\|project>` | Stops a container, a service, or a project (`docker compose stop`). |
| `restart <container\|service\|project>` | Restarts a container, a service, or a project (`docker compose restart`). |
| `pause <container\|service\|project>` | Freezes the processes of each container. |
| `resume <container\|service\|project>` | Resumes the frozen processes. |
| `up <project>` | Creates and starts the services of a project (`docker compose up`). |
| `down <project>` | Stops and removes the containers of a project. Captain asks first. |
| `logs <container\|service\|project> [--since TIME] [--errors]` | Opens the logs. See [logs](#logs). |
| `shell <container\|service>` | Opens the inspector at the **Terminal** tab. |
| `open <container\|service\|port>` | Opens `http://localhost:PORT`, or copies the address of a database port. See [the Open row](projects-and-tasks.md#the-open-row). |
| `forward svc/<name> [local port]` | Forwards a Kubernetes service to a port on this Mac. See [forward](#forward). |
| `float <container\|service>` | Opens the log of a container in a small window that stays on top. |
| `disk` | Opens [Storage](storage.md). |
| `go <page>` | Opens a page. |

The palette has no command that deletes anything.

### Names

A name can be:

- a container name, such as `shop-api-1`,
- a Compose service name, such as `api`. Captain looks in the project on screen first, then in every project. A service with one container stands for that container.
- `project/service`, such as `shop/api`, when two projects have a service with the same name,
- a project name, such as `shop`,
- a published port, for `open`, such as `8080`,
- a Kubernetes service, for `forward`: `svc/<name>` or `svc/<namespace>/<name>`, with an optional `:port`.

When a name fits more than one thing, the palette never guesses. It lists each match, and you pick one.

### logs

`logs api` opens the project page with the inspector at **Logs**.

- `--since TIME` loads only the lines from that time on. A chip such as "Since 10 minutes" shows the filter. Click the chip to clear it. The filter stays when the container restarts, and goes when you select another container.
- `--errors` shows only error lines.

A time is a whole number and one unit: `30s`, `10m`, `1h`, or `2d`. `--since=10m` works too.

`logs shop` opens the project log. The project log has no filters, so `--since` and `--errors` need a container or a service.

### forward

`forward svc/web 8080` forwards the Kubernetes service `web` to `127.0.0.1:8080` and opens the Port Forwarding page. Without a port, Captain picks a free one. The local port must be above 1024. `forward` works only while Kubernetes runs. See [Kubernetes](kubernetes.md).

### go

`go <page>` opens a page. A page name alone works too. The pages are:

`containers`, `images`, `volumes`, `networks`, `extensions`, `snapshots`, `storage`, `forwarding`, `diagnostics`, and `settings`.

## Examples

```text
restart api
stop shop
logs worker --since 10m --errors
logs shop/api --since=1h
shell postgres
open 8080
float worker
forward svc/default/web:80 8081
down shop
disk
go settings
```

## Error messages

When nothing fits, the list says why. For example:

- "No container, service, or project named “x”."
- "--since needs a time, such as 10m, 1h, or 30s."
- "logs has no option --tail. Use --since or --errors."
- "Pick a port above 1024."
- "Kubernetes does not run. Turn it on in Settings to forward a service."

## Help

Hover a row. The status bar shows what the row does. A row that runs a command uses the same sentence as the button that does the same thing.

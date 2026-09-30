# Feature 0034: Project map and staged changes

- Milestone: M28
- Status: Implemented. It needs a check by hand in the app.
- Design: the `Chart table` screen of the v3 canvas. See [0027](0027-v3-interface.md).

## Goal

Show how the services of a project connect: which ports reach them from this Mac, which services talk to each other, which network they share, and which volumes hold their data. Let the user collect changes to limits and restart policies, check them in one list, and apply them together.

## In scope

- **Overview | Map.** A segmented control in the Project page header. Overview is the page of [0030](0030-project-window.md). Map replaces the Open row, the cards, and the log with the map.
- **A laid-out map, not a free canvas.** `captain_core::project_map::layout` places everything in columns, from left to right:
  1. **This Mac:** one pin per published host port, next to the service it reaches.
  2. **Networks:** one dashed lane per network, stacked from top to bottom in name order. A service sits in the lane of its first network by name. In a lane, services that publish ports go in the first column and the others in the second. With no published port, the services fill both columns in name order.
  3. **Volumes:** one node per named volume that a service mounts, next to its users. Bind mounts are not shown.
- **Edges.** Published ports in the action color. "Talks to" in a neutral color. Mounts dashed, in the info color.
- **Talks to.** `captain_core::project_map::talks_to` reads each service's environment. A value names another service when one of its tokens has that service's or container's name as the host:
  - a URL host: `postgres://app:secret@postgres:5432/shop`,
  - `host:port`: `redis:6379`, also in lists such as `kafka:9092,kafka-2:9092`,
  - a bare name, only when the key says it holds a host: its name contains `HOST`, `ADDR`, `SERVER`, `URL`, `URI`, `DSN`, `ENDPOINT`, or `BROKER`. `POSTGRES_USER=postgres` does not count.
  Compose makes each service reachable under its service name on the project network ([Networking in Compose](https://docs.docker.com/compose/how-tos/networking/)).
- **Nodes.** The container glyph in the state color, the service, the image, the state, and a note: the exit reason after a crash (`Exit 137 · out of memory`), else CPU and memory. A click opens the inspector, as on the cards. After an out-of-memory kill, the node shows "Stage 512 MB" (twice the old limit, at least 512 MB) and Logs. An Edit button opens a small editor on the node. A node with staged changes has a "staged" tag.
- **Zoom.** Fit (to the width of the page) and 100%. The map scrolls when it is larger than the page. A legend in the top right.
- **Staged changes.** Changes wait until Apply. They live in the Project page (`captain_core::project_map::StagedChanges`) and are lost when Captain quits.
  - The editor stages the memory limit (64 MB to 16 GB, in doublings), CPUs (in steps of 0.5), and the restart policy (`no`, `always`, `unless-stopped`, `on-failure`).
  - Staging a field again replaces the new value and keeps the old one. Staging the old value again removes the row.
  - A drawer under the map lists each change: the container, the field, and old → new, with a remove button per row, Discard, and "Apply · update N containers".
  - Apply sends one `POST /containers/{id}/update` per container with all its fields ([Engine API: ContainerUpdate](https://docs.docker.com/reference/api/engine/version/v1.47/#tag/Container/operation/ContainerUpdate), [docker update](https://docs.docker.com/reference/cli/docker/container/update/)). Each container gets a toast. Applied rows leave the drawer; failed rows stay.
  - A "N staged changes" pill in the header, in both tabs.
- **Engine API.** `ContainerApi::update_resources(id, ResourceUpdate)` sets any of memory (with `MemorySwap` at twice the limit, as `update_memory` does), `NanoCpus`, and `RestartPolicy`. `ContainerDetail` gets `nano_cpus`, and `Mount` gets `volume` (true for a volume, false for a bind mount).

## Out of scope

- Changes that need a new container: environment, ports, image, mounts, networks. The design's "Apply · recreate N containers" becomes "Apply · update N containers", because `docker update` changes a running container in place.
- Writing changes back to the Compose file. `docker compose up` recreates a container whose configuration changed and sets the file's limits again ([docker compose up](https://docs.docker.com/reference/cli/docker/compose/up/)). The drawer's note says so.
- Dragging nodes, and zoom steps other than Fit and 100%.
- Keeping staged changes across restarts of Captain.
- Network drivers in the lane labels. The lane shows the network's name.

## Notes

- The layout is deterministic: services, networks, ports, and volumes are sorted by name or number, so the map does not move when the list refreshes.
- A service on several networks sits in one lane. Its talks-to edges still reach services in other lanes.
- Pins and volumes stay next to the services they belong to. When two would overlap, the lower one moves down.
- The engine refuses a restart policy on a container started with `--rm`: "Restart policy cannot be updated because AutoRemove is enabled for the container" ([docker update](https://docs.docker.com/reference/cli/docker/container/update/)). The error shows in that container's toast, and the row stays.
- The engine refuses a memory limit below the container's current use. The error shows the same way.
- The map needs `inspect` for each container (networks, mounts, environment). A container without a network, or not inspected yet, sits in a "No network" lane.

## Verification

1. Run `cargo test -p captain-core project_map`. Check that the layout, talks-to, and staging tests pass.
2. Start a Compose project with a web port, an API with `DATABASE_URL=postgres://app@postgres/app`, a Postgres service with a named volume, and a worker. Open the project and click Map.
3. Check the pins on the left, the lane with the network name, the edge from api to postgres, and the dashed edge from postgres to its volume.
4. Click a node. Check that the inspector opens. Click Fit and 100%; check that the map shrinks and grows.
5. Click Edit on the worker. Raise memory and CPUs, choose `always`, and click Stage. Check the three rows in the drawer and "3 staged changes" in the header.
6. Remove one row. Click Apply. Check the toast and `docker inspect` for the new values.
7. Set a low memory limit on the worker and make it run out of memory. Check the red node with "Stage 512 MB". Click it; check the drawer row "Memory limit 64 MB → 512 MB".

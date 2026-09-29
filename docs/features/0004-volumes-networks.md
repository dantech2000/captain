# Feature 0004: Volumes and Networks

- Milestone: M5
- Status: Done
- Design: the "v2" row of the Captain UI canvas, with the container page's cards and rows

## Goal

Replace the Volumes and Networks stubs with real pages. Each page lists its items grouped by Compose project. It can create, remove, and prune them, and it shows a detail panel for the selected item.

## In scope

- Core models:
  - `Volume`: name, driver, mountpoint, scope, creation time, labels, driver options, size in bytes, the number of containers that use it, and the Compose project.
  - `VolumeUser`: a container that mounts a volume, with its state, the destination path, and the read-only flag.
  - `VolumePrune`: the names of the removed volumes and the reclaimed bytes.
  - `Network`: ID, name, driver, scope, subnet, gateway, the internal flag, labels, the number of attached containers, and the Compose project. `bridge`, `host`, and `none` are built-in.
  - `NetworkDetail`: a `Network` with its creation time, every subnet and gateway, the attachable and IPv6 flags, and the attached containers (`NetworkEndpoint`: name, IPv4 and IPv6 addresses, MAC address).
- Engine API:
  - `VolumeApi` lists, creates, and removes volumes. `volume_users(name)` lists the containers that mount a volume. `prune_unused_volumes(all, label)` returns a `VolumePrune`.
  - `NetworkApi` lists, inspects, creates (with the `bridge` driver), and removes networks. `prune_unused_networks(label)` returns the names of the removed networks.
- Docker engine:
  - Volume sizes and container counts come from `/system/df` with `verbose`. If that call fails, the list still shows, and the sizes and counts are unknown.
  - The network list does not include attached containers, so Captain inspects each network for them.
  - `volume_users` lists all containers with the `volume=<name>` filter. The filter also matches a mount by destination path, so Captain keeps only containers that mount the volume by name.
  - Volume prune: on API 1.42 and later, Docker removes only unused anonymous volumes by default. The `all` flag sends the `all=true` filter, which removes unused named volumes too. Below API 1.42, Docker removes unused named volumes even without `all`.
  - Both prunes accept a `label` filter (`key` or `key=value`). The UI does not use it. The live tests use it to prune only their own items.
- Pure logic in `captain-core`, with unit tests:
  - An All / In use / Unused filter. A volume with an unknown count shows only under All.
  - Sorting. Volumes sort by name, with anonymous volumes last. Networks sort with built-in ones first, then by name.
  - Grouping by the `com.docker.compose.project` label. Items without a project go in their own card, last.
  - Which items can be removed. A volume must have no containers. A network must not be built-in and must have no containers.
  - Which items a prune removes (`prunable_volumes`, `prunable_networks`), and label filter matching. The fake engine uses these rules.
  - Name validation with Docker's volume name rule, `[a-zA-Z0-9][a-zA-Z0-9_.-]+`. Networks use the same rule.
  - Which engine events change each list.
  - Prune result text, for example "Removed 3 volumes · reclaimed 120 MB" and "Removed 2 networks".
- Pages:
  - A header with a summary, for example "6 volumes · 1.4 GB" or "4 networks · 2 in use", the prune buttons, and the usage filter.
  - Volumes have two prune buttons. "Prune unused" removes unused anonymous volumes at once. "Prune all" opens a confirmation dialog, then removes unused named volumes too.
  - Networks have one "Prune unused" button. It removes custom networks without containers at once, because networks hold no data.
  - The prune result shows in green text above the list.
  - A name field with a Create button. An invalid name shows a red hint under the field.
  - Project cards with rows styled like container rows. A click on a row selects it. A second click closes the detail panel.
  - A selected row shows Remove. Remove is off for items that are in use or built-in.
  - Failed loads and actions show in red text above the list.
  - An engine switch clears the page, its busy states, and its messages. A create, remove, or prune that was running on the old engine then ends without a message and without changing the page.
- Detail panels, 400 px wide on the right, like the container inspector:
  - Volume: name, driver, scope, mountpoint, creation date, size, the containers that use it (state dot, name, and destination path), labels, and driver options.
  - Network: ID, driver, scope, subnets, gateways, the internal, attachable, and IPv6 flags, creation date, the attached containers (state dot, name, IPv4 address, and MAC address), and labels.
  - A click on a container selects it and opens the Containers page.
- Live updates: each page follows the engine event stream itself and reloads 250 ms after the last matching event. The volume list reloads on volume events and on container create and destroy. The network list reloads on network events. Each reload also reloads the open detail panel.

## Out of scope

- Other drivers, driver options, custom subnets, and labels on create.
- Connecting and disconnecting containers.
- A prune preview that lists the items before they go.

## Verification

1. The Volumes page lists the same volumes as `docker volume ls`, with the sizes that `docker system df -v` shows.
2. The Networks page lists the same networks as `docker network ls`. `bridge`, `host`, and `none` show a built-in badge, and Remove is off for them.
3. `docker volume create captain-test` adds a row within a second. `docker volume rm captain-test` removes it.
4. A name such as `-bad` shows a red hint, and Create does nothing.
5. Create adds and selects a new volume or network. Remove deletes it.
6. `docker run -d --name captain-test --network <network> nginx:alpine` moves that network to In use, and Remove turns off.
7. Select a volume that a container uses. The panel lists the container with its mount path. A click on it opens that container on the Containers page.
8. Select a network with containers. The panel lists each container with the same IPv4 and MAC addresses as `docker network inspect`.
9. Run `docker volume create` without a name. "Prune unused" removes it and shows the reclaimed space. A named unused volume stays until "Prune all" and its confirmation.
10. The ignored live tests pass: `cargo test -p captain-docker --test live_volumes --test live_networks -- --ignored`. They create and remove only items named `captain-agent-...`, and they prune only items with their own `captain-agent-test` label.

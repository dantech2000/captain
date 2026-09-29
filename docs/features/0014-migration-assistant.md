# Feature 0014: Migration Assistant

- Milestone: M13
- Status: In progress
- Decision: [ADR 0009](../adr/0009-migration.md)

## Goal

Copy a user's volumes, images, networks, Compose projects, and containers from another engine (Rancher Desktop, Docker Desktop, Colima, OrbStack, a remote host) into the engine Captain is connected to. The copy goes through the Docker API only. The old engine keeps everything.

## Safety rules

1. Captain never writes to, removes from, or changes the source engine. The code enforces this. The source is reachable only through `SourceEngine` (`crates/captain-docker/src/transfer/source.rs`), which can list, inspect, and export, and nothing else.
2. There are three exceptions. Each one is temporary.
   - Helper containers. They are named `captain-migrate-*`, carry the `io.captain.migrate` label, mount the volume read-only, and have no network. Captain removes them after each copy, when the run ends, and when the dialog closes. `SourceEngine` removes only the helpers it created, plus labeled `captain-migrate-*` leftovers from an earlier crash.
   - The helper image `busybox:latest`. Captain pulls it only if the source does not have it. If Captain pulled it, Captain removes it at the end. If the image was already there, Captain never pulls, so an existing tag never moves.
   - An opt-in snapshot. It is off by default and offered only for containers with changes in their own filesystem. Captain commits the container, without pausing it, to `captain-migrate/<name>:snapshot`. Captain copies that image and then removes it from the source, whether the copy worked or not. If the removal fails, the item's note says so.
3. Captain never overwrites the target. A volume, network, container, or image that already exists there is skipped, and the row says why.
4. A volume copy that fails or is stopped removes its half-filled target volume.
5. Captain refuses a source and a target that are the same engine. It compares the engine IDs, so a symlinked socket also counts as the same engine.

## In scope

- **Step 1, Source.** The list shows engines that discovery finds (`DOCKER_HOST`, the current context, known sockets), except the connected one. Rows with a known socket path name the product. A field takes any other endpoint URL. The target is the connected engine. If that engine is not Captain Engine, a note says so, and the copy still works.
- **Step 2, Plan.** The plan groups items as networks, volumes, images, Compose projects, and containers, each with a checkbox and a size. Everything starts selected. A switch picks all images or only images in use.
  - The plan shows the total size and a time estimate at 80 MB/s.
  - It checks free space in the target with `df` in a helper. The check is green when the target has twice the copy size free, orange when the copy fits but not twice, and red when the copy does not fit. Red blocks the start.
  - A warning lists containers whose own filesystem has changes (`SizeRw > 0`). A copy does not keep those changes unless the user turns on Snapshot for the container.
- **Step 3, Copy.** Items run one at a time, in this order: networks, volumes, images, Compose projects, containers. Each row has a status icon, bytes copied, and a note.
  - A failed row has Retry.
  - Stop cancels the current item. Captain removes its helpers and any partial volume, and the item goes back in the queue. Resume continues with the items that are not done.
  - A new run of the plan skips items that are already done.
- **Step 4, Summary.** The summary counts copied, skipped, failed, and not-run items and lists the items that did not copy. It ends with "Your old engine was not changed."
- **Command palette.** "Bring data from another engine…" dispatches `OpenMigrationAssistant`.

## How each item copies

- **Networks.** Captain creates the network with the same name, driver, options, labels, and IPAM. If the subnet overlaps one in the target ("Pool overlaps"), Captain creates the network again without the subnet and adds a note.
- **Volumes.** Captain creates the target volume with the same name, driver, options, and labels. A helper in the source streams `GET /containers/{id}/archive?path=/v`. The stream goes straight into `PUT /containers/{id}/archive?path=/` on a helper in the target, so the data lands in `/v`. The daemon keeps owners from the tar headers, because `copyUIDGID` is unset. After the copy, Captain compares the entry count and file bytes on both sides. A volume bound to a host folder (a `device` option) or owned by another driver gets created without a data copy, and a note explains why.
- **Images.** `GET /images/get` streams into `POST /images/load`. Captain saves by tag, because a save by ID carries no tags. An untagged image goes by ID.
- **Compose projects.** If the working folder and every Compose file exist on this computer, Captain runs `docker compose up -d` against the target (ADR 0005). Otherwise, or without the CLI, Captain recreates the project's containers one by one. The Compose labels stay, so the project still groups.
- **Containers.** Captain creates the container from its inspect data: image, command, entrypoint, environment, ports, binds and mounts, labels, restart policy, and networks with their aliases. A volume that the image declared keeps its copied volume. A container that was stopped is created and not started.

## Known limits

- The size estimate uses the size the engine lists. With Docker 29 and the containerd store, an export can be larger, because it can hold every platform (moby/moby#51779).
- Captain cannot save one platform of an image. The Engine API has a `platform` parameter on `/images/get` since API 1.48, but bollard 0.21 does not send it. If the source is missing blobs for other platforms, the export fails ("content digest ... not found", docker/cli#5476). The row then suggests that the user pull the image again for one platform.
- `POST /images/load` is not atomic (moby/moby#48591). This is one reason Captain skips images that the target already has under every tag.
- Transfers use their own bollard clients with a one-day request timeout. Bollard has one timeout for a whole client (fussybeaver/bollard#165), and large loads timed out at the default (fussybeaver/bollard#503). Stop cancels a copy. The timeout does not.
- The first-launch setup screen and Settings entry (ADR 0009) belong to the Captain Engine work (M12). They only need to dispatch `OpenMigrationAssistant`.

## Verification

1. Run `cargo test -p captain-docker --test live_transfer -- --ignored`. It uses one engine as both source and target, and only resources named `captain-agent-*`. It copies a volume, keeps owners, links, and modes, then checks the copy. It stops a 64 MB copy and confirms that the half copy and the helpers are gone. It reloads an image tag. It recreates a container from a snapshot. It confirms that items already in the target are skipped.
2. Then confirm that `docker ps -a`, `docker volume ls`, `docker images`, and `docker network ls` show no `captain-agent-*` or `captain-migrate-*` entries.
3. Connect Captain to a second engine. Open the palette and choose "Bring data from another engine…". Pick the old engine and copy a volume with data. Then run `docker volume ls` on the old engine and confirm that nothing changed.

## References

- Engine API: https://github.com/moby/moby/blob/master/api/swagger.yaml and https://github.com/moby/moby/blob/master/api/docs/CHANGELOG.md
- Archive ownership: https://github.com/moby/moby/blob/master/daemon/archive_tarcopyoptions.go
- Commit does not include volumes: https://docs.docker.com/reference/cli/docker/container/commit/
- OrbStack migration and its reported data loss (orbstack/orbstack#1530, #1585): https://docs.orbstack.dev/install#docker-migration

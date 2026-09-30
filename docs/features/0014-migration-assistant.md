# Feature 0014: Migration Assistant

- Milestone: M13
- Status: In progress
- Decision: [ADR 0009](../adr/0009-migration.md)

## Goal

Copy a user's volumes, images, networks, Compose projects, and containers from another engine (Rancher Desktop, Docker Desktop, Colima, OrbStack, a remote host) into the engine Captain is connected to. The copy goes through the Docker API only. The old engine keeps everything.

## Safety rules

1. Captain never writes to, removes from, or changes the source engine. The code enforces this. The source is reachable only through `SourceEngine` (`crates/captain-docker/src/transfer/source.rs`), which can list, inspect, and export, and nothing else.
2. There are four exceptions. The first three are temporary.
   - Helper containers. They are named `captain-migrate-*`, carry the `io.captain.migrate` label, mount the volume read-only, and have no network. Captain removes them after each copy, when the run ends, and when the dialog closes. `SourceEngine` removes only the helpers it created, plus labeled `captain-migrate-*` leftovers from an earlier crash.
   - The helper image `busybox:latest`. Captain pulls it only if the source does not have it. If Captain pulled it, Captain removes it at the end. If the image was already there, Captain never pulls, so an existing tag never moves.
   - An opt-in snapshot. It is off by default and offered only for containers with changes in their own filesystem. Captain commits the container, without pausing it, to `captain-migrate/<name>:snapshot`. Captain copies that image and then removes it from the source, whether the copy worked or not. If the removal fails, the item's note says so.
   - Switch-over (opt-in, see below). Captain stops the containers the user confirmed in a dialog. It never removes them. Roll back starts them again. `SourceEngine` has a stop and a start for this, and no remove.
3. Captain never overwrites the target. The one exception is a switch-over: it empties the item's volumes in the target and copies them again. It does this only for volumes that Captain copied there, and it refuses while an active container (running, paused, or restarting) in the target uses the volume. A volume, network, container, or image that already exists there is skipped, and the row says why.
   - Each volume and container that Captain creates in the target carries the label `dev.captain.migrated-from=<source engine ID>`. A switch-over replaces or starts only what carries this label for its source. For Compose projects, Captain adds the label with an override file on stdin (`docker compose --project-directory <folder> -f <files> -f - up -d`), after it lists the services with `docker compose --profile "*" config --services`. The override also sets `dev.captain.compose-labels-override`. When Captain reads the config files label, it drops the last `-` only on containers with that label. A `-` from the user's own `-f -` stays, so the project counts as having no files here. Closing the assistant during the copy kills the `docker compose` command, and `up` does not start after a stop during the listing.
4. A volume copy that fails or is stopped removes its half-filled target volume.
5. Captain refuses a source and a target that are the same engine. It compares the engine IDs, so a symlinked socket also counts as the same engine.

## In scope

- **Step 1, Source.** The list shows engines that discovery finds (`DOCKER_HOST`, the current context, known sockets), except the connected one. Rows with a known socket path name the product. A field takes any other endpoint URL. The target is the connected engine. If that engine is not Captain Engine, a note says so, and the copy still works.
- **Step 2, Plan.** The plan groups items as networks, volumes, images, Compose projects, and containers, each with a checkbox and a size. Everything starts selected. A switch picks all images or only images in use.
  - The plan shows the total size and a time estimate at 80 MB/s.
  - It checks free space in the target with `df` in a helper. The check is green when the target has twice the copy size free, orange when the copy fits but not twice, and red when the copy does not fit. Red blocks the start.
  - A running Compose project or container has a **Switch over** toggle, off by default: "Stops it in the old engine (not deleted), copies its data again, and starts it here. Downtime is usually a few seconds."
  - If any selected item switches over, Copy first opens a dialog. It lists each container that Captain will stop in the old engine. It also says when Captain refuses a switch-over (see "Switch-over", step 0).
  - A warning lists containers whose own filesystem has changes (`SizeRw > 0`). A copy does not keep those changes unless the user turns on Snapshot for the container.
- **Step 3, Copy.** Items run one at a time, in this order: networks, volumes, images, Compose projects, containers. Each row has a status icon, bytes copied, and a note.
  - A failed row has Retry.
  - Stop cancels the current item. Captain removes its helpers and any partial volume, and the item goes back in the queue. Resume continues with the items that are not done.
  - A new run of the plan skips items that are already done.
  - A switch-over row shows its steps (Stop in the old engine, Copy data again, Start here, Check) and the measured downtime. Stop is off while a switch-over runs, so an item is never left stopped in both engines.
- **Step 4, Summary.** The summary counts copied, skipped, failed, and not-run items and lists the items that did not copy. It ends with "Your old engine was not changed." After a switch-over it says how many items Captain stopped there, and that it deleted nothing. A "Switched over" section lists each switched item with Roll back, and restates that the old containers are stopped, not deleted.
- **Command palette.** "Bring data from another engine…" dispatches `OpenMigrationAssistant`.

## How each item copies

- **Networks.** Captain creates the network with the same name, driver, options, labels, and IPAM. If the subnet overlaps one in the target ("Pool overlaps"), Captain creates the network again without the subnet and adds a note.
- **Volumes.** Captain creates the target volume with the same name, driver, options, and labels. A helper in the source streams `GET /containers/{id}/archive?path=/v`. The stream goes straight into `PUT /containers/{id}/archive?path=/` on a helper in the target, so the data lands in `/v`. The daemon keeps owners from the tar headers, because `copyUIDGID` is unset. After the copy, Captain compares the entry count and file bytes on both sides. A volume bound to a host folder (a `device` option) or owned by another driver gets created without a data copy, and a note explains why.
- **Images.** `GET /images/get` streams into `POST /images/load`. Captain saves by tag, because a save by ID carries no tags. An untagged image goes by ID.
- **Compose projects.** If the working folder and every Compose file exist on this computer, Captain runs `docker compose up -d` against the target (ADR 0005). If the target already has containers of a project with that name that Captain did not copy from this source, or that Compose ran from another folder, Captain skips the project and does not touch them. Otherwise, or without the CLI, Captain recreates the project's containers one by one. The Compose labels stay, so the project still groups.
- **Containers.** Captain creates the container from its inspect data: image, command, entrypoint, environment, ports, binds and mounts, labels, restart policy, and networks with their aliases. A volume that the image declared keeps its copied volume. A container that was stopped is created and not started.

## Switch-over

For a clean copy of live data (a database), and to free the published ports in the old engine. See ADR 0009, "Switch-over mode". The switch-over replaces the item's normal copy step.

0. **Check first.** Before it stops anything, Captain refuses the switch-over, and the row says why, when:
   - A container to stop was started with `--rm` (`HostConfig.AutoRemove`). The engine deletes such a container when it stops, so a roll back could not start it again.
   - Another active container (running, paused, or restarting) in the old engine mounts one of the item's volumes for writing. It would write while Captain copies. Stop it first, or switch over the item it belongs to.
   - The target has a volume or container with the same name that does not carry Captain's label for this source. Captain will not empty or start it.
   - A project starts with `docker compose up`, and the target has containers of a project with that name that do not carry Captain's label for this source, or that Compose ran from another folder.
1. **Stop in the old engine.** `POST /containers/{id}/stop` with `t` set to the container's `StopTimeout`, or 30 s. The engine's default of 10 s is short for a database. A container that already stopped is left as it is.
2. **Copy data again.** For each volume the item mounts, Captain empties the target volume (`find /v -mindepth 1 -delete` in a helper) and streams the tar again. It checks the entry count and file bytes, as a normal copy does. A volume that is not in the target yet is created.
3. **Start here.** A project with its files on this computer runs `docker compose -p <name> -f <files> up -d <services>`, with only the services that ran in the source. Compose starts a named service even when it has a profile, and it starts the services it depends on. Services that were not named, and profile services, stay off. The copied network and volumes keep their Compose labels, so Compose adopts them. Without files or the CLI, Captain recreates the containers one by one and starts the ones that ran. A loose container (in no project) is recreated and started.
4. **Check.** A container with a health check must report `healthy` within 2 minutes. One without must keep running, without a restart, for 10 s. When the target runs on this computer, each published TCP port must accept a connection on `127.0.0.1` within 30 s.
5. **Done.** The downtime is the time from the stop until the check passed.

Once the source is stopped, the switch-over runs to the end, even when the event stream closes. Closing the assistant does not end it: the session and its helpers stay until every switch-over ends. If a step fails, the row has Retry and Roll back. Roll back stops the item in the target and starts the stopped originals in the source. A copy that is missing or already stopped counts as stopped, so a switch-over that failed before the start rolls back too. Roll back tries to stop every copy. If Captain cannot confirm that each copy is missing or stopped, for example because the target does not answer, it leaves the originals stopped and says so. It reports every error at the end.

Captain does not use checkpoint and restore to avoid the stop. It is experimental and fails with volume mounts and some network setups (moby/moby#32227, #48207, #50750).

## Known limits

- The size estimate uses the size the engine lists. With Docker 29 and the containerd store, an export can be larger, because it can hold every platform (moby/moby#51779).
- Captain cannot save one platform of an image. The Engine API has a `platform` parameter on `/images/get` since API 1.48, but bollard 0.21 does not send it. If the source is missing blobs for other platforms, the export fails ("content digest ... not found", docker/cli#5476). The row then suggests that the user pull the image again for one platform.
- `POST /images/load` is not atomic (moby/moby#48591). This is one reason Captain skips images that the target already has under every tag.
- Transfers use their own bollard clients with a one-day request timeout. Bollard has one timeout for a whole client (fussybeaver/bollard#165), and large loads timed out at the default (fussybeaver/bollard#503). Stop cancels a copy. The timeout does not.
- When the assistant closes, it waits for each switch-over to end, and for a stopped copy up to 60 s. Then it removes the helpers. If a copy still runs after 60 s, the helpers stay, and the next session removes them as leftovers.
- A volume or container that an older Captain copied into the target has no `dev.captain.migrated-from` label, so a switch-over refuses it. Remove or rename it in the target, then switch over again.
- The port check connects on `127.0.0.1`. It is skipped for a `tcp://` target, and it cannot tell Captain's forward from another program that holds the same port.
- The first-launch setup screen and Settings entry (ADR 0009) belong to the Captain Engine work (M12). They only need to dispatch `OpenMigrationAssistant`.

## Verification

1. Run `cargo test -p captain-docker --test live_transfer -- --ignored`. It uses one engine as both source and target, and only resources named `captain-agent-*`. It copies a volume, keeps owners, links, and modes, then checks the copy. It stops a 64 MB copy and confirms that the half copy and the helpers are gone. It reloads an image tag. It recreates a container from a snapshot. It confirms that items already in the target are skipped. It confirms that a Compose project started with the user's own `-f -` file does not count as replayable.
2. Run `cargo test -p captain-docker --test live_switchover -- --ignored`. It uses one engine as both sides and only `captain-agent-sw*` resources. It switches a busybox httpd container over into a new name with a replaced volume copy, checks the port, and rolls back. It confirms that a switch-over refuses to empty a target volume that Captain did not copy, and that nothing stops then. It rolls back a switch-over whose copy was never created. It confirms that a switch-over refuses while a paused container writes to the item's volume. It confirms that a roll back leaves the original stopped when the target stops answering. It confirms that a Compose project run with `--project-directory`, whose target containers came from a plain `docker compose up`, is refused by a switch-over and skipped by a plain copy, without a new label. After `up_labeled` recreates it with Captain's label, a new scan can replay its files, and it switches the project over and confirms that its profile service does not start.
3. Then confirm that `docker ps -a`, `docker volume ls`, `docker images`, and `docker network ls` show no `captain-agent-*` or `captain-migrate-*` entries.
4. Connect Captain to a second engine. Open the palette and choose "Bring data from another engine…". Pick the old engine and copy a volume with data. Then run `docker volume ls` on the old engine and confirm that nothing changed.

## References

- Engine API: https://github.com/moby/moby/blob/master/api/swagger.yaml and https://github.com/moby/moby/blob/master/api/docs/CHANGELOG.md
- Archive ownership: https://github.com/moby/moby/blob/master/daemon/archive_tarcopyoptions.go
- Commit does not include volumes: https://docs.docker.com/reference/cli/docker/container/commit/
- Stop, start, and wait: https://docs.docker.com/reference/api/engine/ (ContainerStop `t`, 304 when already stopped)
- `docker stop` and stop timeouts: https://docs.docker.com/reference/cli/docker/container/stop/
- `docker compose up [SERVICE...]`: https://docs.docker.com/reference/cli/docker/compose/up/
- Profiles, and services named on the command line: https://docs.docker.com/compose/how-tos/profiles/
- `--project-directory` (default: the folder of the first Compose file): https://docs.docker.com/reference/cli/docker/compose/
- `docker ps --filter status=` values (running, paused, restarting): https://docs.docker.com/reference/cli/docker/container/ls/#status
- OrbStack migration and its reported data loss (orbstack/orbstack#1530, #1585): https://docs.orbstack.dev/install#docker-migration

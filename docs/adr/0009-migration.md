# ADR 0009: Migrating data into Captain Engine through the Docker API

- Status: Accepted
- Date: 2026-09-29

## Context

When a user switches to Captain Engine (ADR 0008), their images, containers, volumes, and networks stay inside the old engine's VM disk. Volumes hold the data people care about most: databases, uploads, caches that are slow to rebuild.

We compared three approaches:

| Approach | Risk | Verdict |
|----------|------|---------|
| Fresh start: the new engine starts empty, the old one stays reachable | None, but users lose easy access to volume data and must re-pull images | Always available, but not enough alone |
| Copy through the Docker API, with both engines running | Low. Works with any source engine. The source is never changed | Chosen |
| Read the old engine's VM disk directly (the OrbStack approach) | High and engine-specific. OrbStack has a reported case where a failed migration deleted the user's original data | Rejected |

## Decision

Captain gets a **Migration Assistant** that copies data from any engine it can connect to (Rancher Desktop, Docker Desktop, Colima, OrbStack, a remote host) into Captain Engine, using only the Docker API.

### Rules

1. **Copy, never move.** Nothing in the source engine is changed or deleted. The assistant never offers to clean up the source. The one exception is the opt-in switch-over (below), which stops confirmed containers and never deletes them.
2. **Plan before copying.** The assistant lists what it found, with sizes, and estimates the total size and time. It checks free disk space first, because a copy briefly needs about twice the space.
3. **Per-item progress, check, and retry.** Each image, volume, and project is its own step. A failed step can be retried alone. The run can be stopped and resumed.
4. **Say what is lost.** Changes made inside a container's own filesystem (not in a volume) are not kept when the container is recreated. The assistant says so before it starts, and offers an opt-in snapshot (`docker commit`) for those containers.

### What it copies (all selected by default)

- **Volumes:** a helper container in the source engine packs each volume into a tar stream. A helper container in Captain Engine unpacks it into a new volume with the same name, driver options, and labels. The check compares the file count and total size.
- **Images:** all images, or only images that containers use. They stream from the source's `GET /images/get` into Captain Engine's `POST /images/load`, tags included.
- **Compose projects:** after their images and volumes arrive, each project is recreated with `docker compose up -d` from its files (the working folder and file labels, ADR 0005), pointed at Captain Engine. Projects whose files no longer exist on disk fall back to the standalone path.
- **Standalone containers:** recreated from their inspect data: image, command, environment, ports, mounts, labels, restart policy, and networks. User-defined networks are created first.

### Switch-over mode

Amended 2026-09-29. A copy of a running database is only crash-consistent, because the program writes while the tar stream reads. The old container also holds its published ports, so the copy cannot take them. Switch-over mode solves both for one project or container at a time.

It adds one write to the source: Captain **stops** the containers that the user explicitly confirmed. It never removes them, and nothing in the source is ever deleted. Rule 1 still holds for everything else.

- **Opt-in per item.** Only running Compose projects and running standalone containers offer it. It is off by default.
- **Confirmation.** Before the run, a dialog lists every container that Captain will stop in the old engine.
- **Checks before the stop.** Captain refuses the item, and stops nothing, when a container to stop was started with `--rm` (stopping deletes it), when another running container writes to the item's volumes, or when the target already has a volume or container of that name that Captain did not copy. Captain labels each volume and container it creates in the target with `dev.captain.migrated-from=<source engine ID>`, and a switch-over replaces only what carries that label. This holds for Compose projects too: every existing target container of the project must carry the label, and Compose must have run it from the same folder. When Captain runs `docker compose up`, it passes a small override file on stdin (`-f -`) that adds the label to every service, so the project folder stays untouched. Compose records that file as `-` in `com.docker.compose.project.config_files`, and Captain ignores `-` when it reads the label. A later `docker compose up` without the override recreates the containers without the label; after that, the user owns them and a switch-over refuses them.
- **Steps.** Stop in the source (with the container's own stop timeout, or 30 s) → copy the item's volumes again into the emptied target volumes, and check them → start the item in the target → check it (health, or a steady run, and the published ports) → done. A Compose project starts with `docker compose up -d <services>`, with only the services that ran in the source. Services behind a profile, or ones that were off, stay off.
- **Roll back.** Stop the item in the target, then start the originals in the source. A missing or stopped copy counts as stopped. Roll back tries every step and reports every error. Nothing is removed on either side.
- **Session end.** A switch-over keeps its session alive until it ends, even when the assistant closes. Helpers are removed only when no copy runs.
- **No live migration.** Checkpoint and restore (CRIU) would avoid the stop, but it is experimental in Docker and has open issues with volume mounts and networking (moby/moby#32227, #48207, #50750). A short stop is the reliable way to get a consistent copy.

### Where it appears

- The first-launch setup screen (ADR 0008), when Captain detects other engines with data.
- Settings, as "Bring data from another engine…", any time later.

## Consequences

- Migration works with every engine that speaks the Docker API, including ones Captain has never seen.
- It is slower than a raw disk copy for large image sets, because images are re-exported. Selecting only images in use keeps it short.
- It needs both engines running at once, so it needs enough memory for two VMs for the duration.
- The source stays intact, so a failed or partial migration never costs the user data. They can keep using the old engine from Settings.
- With switch-over mode, a confirmed item is stopped in the source, not deleted. The user can roll back at any time and start it there again.

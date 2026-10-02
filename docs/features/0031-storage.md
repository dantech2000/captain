# Feature 0031: Storage and cleanup

- Milestone: M25
- Status: Implemented (M25). The page was checked by hand; a real cleanup with the snapshot first is left. The sidebar Disk card was removed later; the status bar's Disk segment opens the page.
- Design: the `Storage` screen of the v3 canvas. See [0027](0027-v3-interface.md).
- Why: disk use is opaque in Docker Desktop ([docker/for-mac#371](https://github.com/docker/for-mac/issues/371), [docker/roadmap#13](https://github.com/docker/roadmap/issues/13)), and one confirm per delete slows cleanup ([orbstack#1869](https://github.com/orbstack/orbstack/issues/1869)).

## Goal

A user sees what fills the engine's disk, who uses each large item, and frees space in one step with a preview. Nothing a container uses is removed.

## In scope

- **A Storage page**, with an entry in the sidebar above Diagnostics (now a button in the icon rail) and a `disk` command in the ⌘K palette.
- **The header.** "Captain Engine disk", the bytes in use, and "of 64 GB", the VM disk size from the settings. Another engine shows the bytes in use only. With Captain Engine, the free space on this computer and a link, "Change the disk size in Settings".
- **The bar and its legend.** Images, Volumes, Build cache, Snapshots (Captain Engine only), and Containers (writable layers). With Captain Engine each part is a share of the disk size; otherwise a share of the bytes in use.
- **Largest first.** The 12 largest items with an icon, a name, who uses them, and a size:
  - An image or a volume lists the containers, running or stopped, that use it: "Used by api, worker and 2 more".
  - Unused tagged images say "No container uses it". Dangling images are one row: "<none> × 11 dangling".
  - Build cache is one row for records older than 14 days ("Not used since Sep 12") and one for the rest.
  - Containers show the engine's status, for example "Exited (0) 5 days ago". Snapshots say "Snapshot".
  - Rows the cleanup can remove are tinted with the warning color. An unused volume's note is red.
- **Free up space.** A checkbox per group, with its size and a note:
  1. Build cache older than 14 days (checked).
  2. Dangling images (checked).
  3. Images no container uses (checked).
  4. Stopped containers older than 3 days (checked).
  5. Volumes no container uses (unchecked, with a red note: they may hold data).
- The total updates as the user checks groups. "Review N items, free X" opens a dialog that lists every item by group. "Remove N items and free X" runs the cleanup; a toast reports the bytes freed and any item the engine refused.
- **Take a snapshot first** (Captain Engine only, on by default). It stops the engine, saves a snapshot named "Before cleanup <date>", starts the engine, waits up to 3 minutes for the workspace to connect again, then removes the items. If the snapshot fails or the engine does not come back, nothing is removed.
- **Weekly build-cache cleanup.** A checkbox sets `weekly_build_cache_cleanup` (off by default). The app checks each hour; when the engine is connected and 7 days have passed since `build_cache_cleaned_at`, it prunes build cache older than 14 days and saves the time. The run is logged, not shown.
- **A Disk segment in the status bar**: "Disk 18.2 GB of 64 GB", in the warning text color when a cleanup can free something. A click opens Storage. It shows once the storage model has read the disk use. Its help sentence has what can be freed: "Disk: 18.2 GB of 64.0 GB. 6.1 GB can be freed. Click to review.", or "Disk: 18.2 GB of 64.0 GB. Click to open Storage." when nothing can be freed. The data is read on connect, when the Storage page opens, after a cleanup, 3 seconds after the last engine event that adds or removes data (an image pull, tag, or removal; a volume or container created or removed; a prune), and every 10 minutes.

## Out of scope

- Growing or shrinking the VM disk from this page. The link opens Settings.
- A per-layer view of images, and the space that images share.
- A cleanup of networks. They take no disk.
- A schedule other than weekly, and a weekly cleanup of anything but build cache.

## Notes

- **The engine API.** `ContainerApi::disk_usage` reads `GET /system/df?verbose=true` ([SystemDataUsage](https://docs.docker.com/reference/api/engine/version/v1.52/#tag/System/operation/SystemDataUsage), bollard [`Docker::df`](https://docs.rs/bollard/0.21.1/bollard/struct.Docker.html#method.df)). API 1.52 groups the answer as `ImageUsage`, `ContainerUsage`, `VolumeUsage`, and `BuildCacheUsage`, each with `TotalSize` and its items. The totals count shared layers once. An engine older than API 1.52 sends `LayersSize`, `Images`, `Containers`, `Volumes`, and `BuildCache`, which bollard 0.21 does not read; then Captain builds the items from the image, container (`size=true`), and volume lists, without build cache.
- The images-to-containers link comes from the containers' `ImageID`, because an image's `Containers` field is only a count and is often -1. Volume users come from the containers' `Mounts` names.
- **Prunes.** `ImageApi::prune_build_cache` calls `POST /build/prune` with `filters={"until":["336h"]}` and without `all`, so internal and frontend records stay ([BuildPrune](https://docs.docker.com/reference/api/engine/version/v1.52/#tag/Image/operation/BuildPrune), bollard [`prune_build`](https://docs.rs/bollard/0.21.1/bollard/struct.Docker.html#method.prune_build)). The `until` filter keeps records used in that time ([docker buildx prune](https://docs.docker.com/reference/cli/docker/buildx/prune/)). The weekly cleanup uses this prune.
- **Build cache goes by record.** The engine checks `until` when the prune runs, so a daemon-wide prune could remove a record that turned 14 days old after the preview. The cleanup calls `ImageApi::prune_build_record` once per previewed record, with `filters={"until":["336h"],"id":["^<id>$"]}`. The engine takes one value per filter and matches `id` as a regular expression (`id~=`), so Captain anchors it and accepts only letters and digits ([builder.go](https://github.com/moby/moby/blob/v28.5.0/builder/builder-next/builder.go#L644-L711), [containerd filter syntax](https://github.com/containerd/containerd/blob/v1.7.27/filters/scanner.go#L290-L297)). A record a build used since the preview stays, and the report lists it as not removed.
- Old stopped containers are removed one by one by ID, with the normal (not forced) remove, so only the containers in the preview go. The engine refuses one that started again since the preview.
- **Images go by immutable ID, volumes by name.** A tag can move to another image after the preview, so Captain never removes by tag. Right before each image, Captain lists the images again and skips one that is gone or that a container uses now. `DELETE /images/<id>` without force then refuses an image that a container uses, or that tags in more than one repository point at; it never only untags it ([image_delete.go](https://github.com/moby/moby/blob/v28.5.0/daemon/images/image_delete.go#L99-L159)). Each refusal is a line in the report. Nothing is forced, so the engine also refuses a volume that a container started to use after the preview.
- **The cleanup is pinned to one daemon.** The Storage model reads `ContainerApi::info` with the disk use and keeps the daemon's `ID` and endpoint with the plan ([SystemInfo](https://docs.docker.com/reference/api/engine/version/v1.52/#tag/System/operation/SystemInfo)). The daemon ID stays the same across a restart. Before it removes anything, `run_cleanup` reads the info again. If the ID or the endpoint differs, for example because the user switched engines during the snapshot, it removes nothing and reports why.
- The plan (`captain_core::storage::ReclaimPlan`) never selects an image that a container uses by ID or by count, a volume that a container mounts, a volume with an unknown container count, build cache in use, or internal and frontend records.
- The build cache size in the preview is the sum of the records; shared records may free less.
- The snapshot step reuses `HostModel::begin_snapshot` and `end_snapshot` from the Snapshots page ([0023](0023-snapshots.md)).
- The free space on this computer comes from the snapshot list, which the host reads with `df -Pk` (`captain-host/src/probe/disk.rs`).

## Verification

1. Run `cargo test -p captain-core storage::`.
2. Run `DOCKER_HOST=unix://$HOME/.captain/lima/captain/sock/docker.sock cargo test -p captain-docker --test live_disk -- --ignored`.
3. Open Captain with Captain Engine. Open Storage from the sidebar. Check the header, the bar, and the legend against `docker system df`.
4. Check that each large image lists the containers that use it, and that a used volume says "Used by".
5. Run `docker pull busybox` and `docker run --name captain-agent-old busybox true`. Reload the page. Check that busybox is not in "Images no container uses". Remove the container.
6. Check and uncheck groups. Check that the total and the button text change.
7. Click Review. Check that the dialog lists every item. Cancel.
8. Turn on the weekly option. Check that `weekly_build_cache_cleanup` is `true` in the settings file.
9. Hover the Disk segment in the status bar. Check the sentence and the warning color. Click it; check that Storage opens.
10. Press ⌘K, type `disk`, and press Return. Check that Storage opens.
11. Switch to another engine. Check that the header shows only the bytes in use and that the snapshot option is gone.

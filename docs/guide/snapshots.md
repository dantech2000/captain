# Snapshots

A snapshot is a saved copy of Captain Engine. It holds the engine disk, with all images, containers, and volumes. It also holds the engine settings for CPUs, memory, disk, the Docker daemon, and Kubernetes. Restore a snapshot to go back to that state.

Snapshots need Captain Engine, so they work only on macOS. They live in `~/.captain/snapshots`.

## Create a snapshot

1. Click the Snapshots button in the icon rail.
2. Click **Create snapshot…**.
3. Enter a **Name**. The default is the date and time. Add a **Description** if you want.
4. Click **Create**.

Captain Engine stops while Captain saves the snapshot, then starts again. Your containers stop for that time.

A new snapshot takes little space at first, because it shares blocks with the engine disk. It grows as the engine changes files. When your Mac has less free space than the engine disk uses, the dialog warns you. You can still create the snapshot.

## Restore a snapshot

1. Click **Restore…** on the snapshot's row.
2. Keep **Save the current state as a snapshot first** on, so you can go back.
3. Click **Restore**.

Containers, images, and volumes that you made after the snapshot are lost. Captain Engine restarts. The engine settings in the snapshot replace the current ones.

## Rename or delete

- Click **Edit…** to change the name or description, then click **Save**. The engine keeps running.
- Click **Delete…** and confirm to delete a snapshot. It cannot be undone. The engine keeps running.

## From the terminal

```sh
captain snapshot create before-upgrade --description "Postgres 16 data"
captain snapshot list
captain snapshot restore before-upgrade
captain snapshot delete before-upgrade
```

`captain snapshot restore` needs Captain to be closed. See the [command reference](../reference/cli.md#captain-snapshot).

## Snapshots and cleanup

The [Storage](storage.md#take-a-snapshot-first) cleanup can save a snapshot named "Before cleanup" first. Delete it when you no longer need it, because it keeps the removed items on disk.

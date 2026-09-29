# Feature 0017: Files and processes

- Milestone: M3 (container detail)
- Status: Done; needs a manual check in the app
- Builds on: [0002](0002-v2-interface.md), which added the inspector tabs with a placeholder Files tab
- Parity: the Files tab of Docker Desktop ([docs](https://docs.docker.com/desktop/use-desktop/container/)) and the process list of `docker top`

## Goal

Finish M3. The Files tab browses a running container's filesystem, previews small text files, and saves a file or folder to `~/Downloads`. The Stats tab lists the container's processes.

## In scope

- Engine: four `ContainerApi` methods, in the Docker engine and the fake engine.
  - `list_files(id, path)`: the entries of one folder, with name, kind, size, mode, and modified time.
  - `read_file(id, path, limit)`: the first `limit` bytes of a file and its full size.
  - `save_path(id, path, dir)`: copies a file into `dir`, or a folder as `name.tar`, and returns the new path. It never overwrites; it picks `name (1).ext` instead.
  - `top(id)`: `docker top`, as column titles and rows.
- Files tab:
  - A path bar with Up and breadcrumbs. A click on a breadcrumb opens that folder.
  - A list of the folder's entries: folders first, then by name. Columns: name, size, mode (`drwxr-xr-x`), and modified time. A symbolic link to a folder shows as a folder.
  - A click selects an entry. A double-click or Enter opens a folder or previews a file. Backspace, the Up button, or Cmd-Up (Alt-Up on Windows and Linux) goes to the parent folder or closes the preview. Escape closes the preview. Up and Down arrows move the selection.
  - Preview: a read-only, monospace view of a text file. Captain reads at most 256 KiB. A larger file shows its first 256 KiB and a note. A binary file (a NUL byte in the first bytes, or not UTF-8) shows "Binary file, no preview".
  - Save to Downloads: saves the selected file, or the folder as a `.tar`, to `~/Downloads`, and shows a toast with the path.
  - Refresh reloads the folder.
  - Clear messages for a stopped container, a folder that does not exist or is not readable, and a container where listing is not possible.
- Processes: the Stats tab shows a table under the charts with the columns the engine returns (`UID`, `PID`, `PPID`, `C`, `STIME`, `TTY`, `TIME`, `CMD`). It refreshes every 2 seconds while the tab shows and the container runs.

## Out of scope

- Editing, deleting, uploading, and drag and drop of files.
- A diff of changed files (`docker diff`).
- Files of a stopped container.
- Choosing a folder other than `~/Downloads`.
- Sorting the process table and killing a process.

## Notes

### How Captain lists a folder

The Engine API has no call that lists one folder. Captain uses two methods:

1. **Exec (first choice).** Captain runs a short `sh` script through the existing exec code. The script changes to the folder and runs `stat -c "%f %s %Y %n"` on every entry (`.[!.]* ..?* *`), then `stat -L -c "%f %n"` to find links that point to folders. `%f` (raw mode in hex), `%s` (size), `%Y` (modified time in seconds), and `%n` (name) work the same in GNU coreutils and BusyBox ([BusyBox stat](https://github.com/brgl/busybox/blob/master/coreutils/stat.c), [GNU stat](https://man7.org/linux/man-pages/man1/stat.1.html)). BusyBox has no `stat --printf` ([NixOS report](https://discourse.nixos.org/t/error-when-logging-in-as-root-stat-unrecognized-option-printf-busybox-v1-30-1-multi-call-binary/3493)), so the script does not use it. The output does not depend on the locale, unlike `ls -l`, whose columns and dates differ between GNU and BusyBox. Checked by hand on `busybox`, `alpine:3.21`, and `debian:bookworm-slim`.
2. **Archive (fallback).** An image with no shell or no `stat`, such as distroless, makes the exec fail with exit code 126 or 127. Captain then reads `GET /containers/{id}/archive?path=…` ([Engine API](https://docs.docker.com/reference/api/engine/version/v1.47/#tag/Container/operation/ContainerArchive)) and keeps only the direct children from the tar headers. The archive holds the folder recursively, like `docker cp` ([docs](https://docs.docker.com/reference/cli/docker/container/cp/)), so Captain stops after 32 MiB and shows "This folder is too large to list, and the container has no shell." In this mode, links to folders show as links and do not open, and hard links show size 0.

A name that contains a line break does not list correctly with exec. Such names are rare in images.

`GET /archive` also reads files for the preview and for Save to Downloads, so both work in images without a shell. `HEAD /archive` returns the stat of only one path, so Captain does not use it to list.

### Stopped containers

Exec needs a running container. The archive endpoint works on a stopped container, but it would read the whole folder for each listing. Docker Desktop's Files tab also shows only running containers. The Files tab shows "Container is not running" for a stopped container.

### Why processes go in the Stats tab

The Stats tab is where a user looks at what uses CPU and memory. The process list answers the next question: which process. The Overview tab shows fixed configuration, and a list that refreshes every 2 seconds would make it jump. The table polls only while the Stats tab shows. `docker top` has no stream, so Captain calls `GET /containers/{id}/top` ([Engine API](https://docs.docker.com/reference/api/engine/version/v1.47/#tag/Container/operation/ContainerTop)) with the default `ps_args` (`-ef`).

## Verification

1. Run `docker run -d --name captain-files nginx:alpine` and select it. Open Files. The list shows `/` with folders first.
2. Double-click `etc`. The path bar shows `/etc`, and each part is a link. Select `nginx` with the arrow keys and press Enter. Press Backspace. The list shows `/etc` again.
3. Double-click `etc/os-release`. A preview shows its text. Double-click `/bin/busybox`. The preview says "Binary file, no preview".
4. Select `os-release` and click Save. A toast says `Saved to ~/Downloads/os-release`. Save it again; the new file is `os-release (1)`.
5. Select the `nginx` folder and click Save. `~/Downloads/nginx.tar` holds the folder.
6. Stop the container. The Files tab says the container is not running.
7. Build an image with no shell: `FROM busybox AS b`, then `FROM scratch` with `COPY --from=b /bin/busybox /sleep`, `/lib`, `/lib64`, and `/etc/group`, and `CMD ["/sleep","300"]`. Run it and open Files. The list shows `/` and `/etc` through the archive. `docker exec … sh` fails with exit code 127 in the same container.
8. Open Stats on a running container. The Processes table shows `nginx: master process` and its workers, and changes within 2 seconds after `docker exec captain-files sleep 30 &`.
9. `cargo test -p captain-docker --test live_files -- --ignored` passes.

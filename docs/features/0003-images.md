# Feature 0003: Images

- Milestone: M4
- Status: In progress
- Design: the v2 look of the Containers page, applied to images

## Goal

Show the local images and let the user inspect, pull, remove, prune, and run them without the Docker CLI.

## In scope

- Engine API: `ImageApi` with `list_images`, `inspect_image`, `image_history`, `remove_image` (never forced), `prune_dangling_images`, `pull_image` as a stream of progress messages, and `run_container`.
- Models: `Image` (ID, tags, size, created time, container count, dangling flag), `PullProgress` (layer, status, current and total bytes), `ImageDetail` (tags, digests, created time, platform, size, and an `ImageConfig` with entrypoint, command, environment, exposed ports, working directory, user, and labels), `ImageLayer` (one history step: command, size, created time), and `RunSpec` (image, name, published ports, environment, auto-remove, restart policy).
- Container count: the engine often reports -1 for it. Then Captain counts containers, running or stopped, by image ID.
- Header: the image count and total size, for example "14 images · 3.2 GB", and an All / In use / Unused / Dangling filter.
- Rows: repository, tag in a monospace font, short ID, size, and age. An "in use" pill shows how many containers use the image. Untagged images get a "dangling" pill.
- Selection: a click on a row selects it and opens the inspector.
- Inspector: a 400 px panel on the right, like the container inspector. The close button clears the selection.
  - Header: repository, tag, short ID, size, and a pill with the usage ("in use by 2", "unused", or "dangling").
  - Actions: Run, Remove (disabled for an image in use), and Copy ID.
  - Details: image ID, tags, digests, created time in UTC, platform (for example `linux/arm64/v8`), and size.
  - Config: entrypoint, command, working directory (`/` if unset), and user (`root` if unset).
  - Exposed ports, environment, and labels. Values of secret-looking variables (`EnvVar::is_secret`) are masked.
  - Layers: the history, newest first. Each step shows its Dockerfile line in a monospace font, without the `/bin/sh -c` wrapper and the BuildKit marker. A long line is truncated, and a tooltip shows all of it. A bar shows the layer size relative to the largest layer. An imported image has no history, and the engine answers `null` for it, so the section says so.
- Run: the Run button opens a dialog, prefilled from the image.
  - Name: optional. An empty name lets the engine pick one. A name must match `[a-zA-Z0-9][a-zA-Z0-9_.-]+`.
  - Host ports: one field for each exposed port. The default is the same number. An empty field does not publish the port. A port must be 1 to 65535, and a host port can be used only once for each protocol.
  - Environment: one `KEY=value` field for each variable of the image. The user can add and remove rows. The key must not be empty or contain spaces. Empty rows are skipped.
  - Restart policy: no, unless-stopped, always, or on-failure.
  - Auto-remove: removes the container when it exits. It works only with the restart policy "no", because the engine refuses the pair.
  - Captain checks the form (`RunForm::to_spec` in `captain-core`) before it calls the engine. The first problem shows in red in the dialog.
  - Captain creates the container, then starts it, like `docker run -d`. If the start fails, the created container stays, as with the CLI, and the dialog shows the error.
  - On success, the dialog closes and the page shows "Started <name>" with a Show link. Show selects the container and opens the Containers page. If the container is not in the list yet (for example, auto-remove already removed it), the page opens without it.
- Remove: removes the selected image. The button is disabled for an image in use, because the engine refuses to remove it without force.
- Prune dangling: removes untagged, unused images. The button shows the size it can free, and the result shows the space reclaimed.
- Pull: a text field and a Pull button. Enter also starts the pull. A missing tag means `latest`. A progress bar and a status line show the pull, with the number of finished layers.
- Live list: the page reloads on image events (`pull`, `tag`, `untag`, `delete`, `import`, `load`, `prune`) and on container `create` and `destroy`, with a 150 ms debounce.
- Errors show inline in red: list errors, remove and prune errors, and pull errors.
- Engine switch: when the workspace switches or loses its engine, the page drops everything from the old one: the list, the selection, errors and notices, the "Started" notice, and any pull or push, which stops. Results that arrive later from the old engine (remove, prune, tag, build, run) do not show. The Build button stays off until the new engine connects.

## Out of scope

- A command, entrypoint, volume, or network override in the Run dialog.
- Removing a variable of the image in the Run dialog. The image still sets it.
- Tag, push, build, import, and export.
- Registry login and private registry credentials.
- Force remove, and removing an image together with its containers.
- A confirmation dialog for Remove and Prune.
- The image count in the sidebar.

## Verification

1. The Images page lists the same images as `docker images`, with the same sizes, rounded.
2. `docker run -d --name captain-img-test busybox sleep 600` marks `busybox` "in use" within a second. Remove is disabled for it.
3. `docker rm -f captain-img-test` clears the pill, and Remove then deletes `busybox`.
4. A pull of `busybox` shows the progress bar move to 100%, and the image appears in the list.
5. A pull of `captain-test/does-not-exist` shows a red error.
6. `docker build` with an untagged result adds a "dangling" row. Prune dangling removes it and shows the reclaimed size.
7. The filters show only matching rows, and an empty filter shows a hint to choose All.
8. A click on `nginx` opens the inspector. Its ports, environment, and labels match `docker image inspect nginx`, and its layers match `docker history nginx`.
9. Run on `nginx` with the name `captain-web` and host port 8080 starts a container. `curl localhost:8080` answers, and the page shows "Started captain-web". Show opens the Containers page with `captain-web` selected.
10. A second Run with the name `captain-web` shows the engine's name conflict in the dialog. A host port of `99999` shows a red hint and does not call the engine.
11. `cargo test -p captain-docker --test live_images -- --ignored` inspects every local image, and runs and removes `captain-agent-run`.

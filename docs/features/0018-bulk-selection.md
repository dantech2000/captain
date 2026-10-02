# Feature 0018: Bulk selection

- Milestone: M3 (container detail)
- Status: Done; checked by hand on macOS (M3).
- Parity: bulk selection on the Containers and Volumes pages of Rancher Desktop ([containers](https://docs.rancherdesktop.io/ui/containers), [volumes](https://docs.rancherdesktop.io/ui/volumes)) and the bulk actions toolbar of Docker Desktop ([docs](https://docs.docker.com/desktop/use-desktop/container/))

## Goal

Act on several containers or volumes at once, without the CLI.

## In scope

- Selection, with the platform's list conventions (Finder on macOS, Explorer on Windows and Linux):
  - A click selects one row, as before.
  - Cmd-click on macOS, or Ctrl-click on Windows and Linux, adds a row to the selection or removes it.
  - Shift-click selects the rows from the last clicked row to this one, in list order. Rows in a folded project card do not count.
  - The selected rows all have the selected style. The inspector or detail panel shows the last row the user clicked.
- Selection bar: when two or more rows are selected, a bar above the list shows "n selected", the actions, and Clear.
  - Containers: Start, Stop, Restart, and Delete. Each action runs on the selected containers it applies to: Start on stopped ones, Stop on running ones, Restart on running ones. A button is disabled when it applies to none.
  - Volumes: Delete.
- Delete asks once. The dialog lists the names. For containers it says how many are running; Captain stops and removes those (`force: true`), like the single Delete.
- Results: the actions reuse the single-item code. After a bulk action, one toast reports the result: "Deleted 3 containers." or "Could not stop 2 of 5 containers" with each name and the engine's message. For volumes, the page's error line lists each volume that failed, for example a volume in use.
- The selection drops rows that go away, for example after a delete.

## Out of scope

- Checkboxes and Select All.
- Pause and Resume for several containers.
- Bulk actions for images and networks.

## Notes

- Rancher Desktop enables a bulk action only when all selected containers share one state. Captain runs each action on the containers it applies to, so a mixed selection still works.
- The selection model is `captain_core::store::MultiSelection`, shared by both pages and tested without GPUI.
- GPUI reports the modifier keys with each click (`ClickEvent::modifiers`), and `Modifiers::secondary` is Cmd on macOS and Ctrl elsewhere.

## Verification

1. Create three containers: `for n in 1 2 3; do docker run -d --name captain-bulk-$n busybox sleep 600; done`.
2. Click `captain-bulk-1`, then Shift-click `captain-bulk-3`. Three rows are selected, and the bar shows "3 selected".
3. Cmd-click (Ctrl-click) `captain-bulk-2`. The bar shows "2 selected".
4. Click Stop. Both containers stop, and one toast reports the result.
5. Shift-click to select all three and click Delete. The dialog lists the three names and says one is running. Confirm. The rows go away, and the toast says "Deleted 3 containers."
6. On Volumes, create `a` and `b` with `docker volume create`, and one in use with `docker run -d --name captain-bulk-v -v c:/c busybox sleep 600`. Select `a`, `b`, and `c`, and click Delete. `a` and `b` go away, and the error line says `c` is in use.

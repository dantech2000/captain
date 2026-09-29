# Feature 0005: command palette

- Milestone: M2 follow-up, built after the v2 interface (feature 0002)
- Status: In progress
- Design: the "v2 · Command palette" artboard of the Captain UI canvas

## Goal

Let the user reach any page, container, or container action from the keyboard. Press ⌘K (Ctrl+K on Linux and Windows), type a few letters, and press Enter.

## In scope

- Open and close:
  - ⌘K or Ctrl+K opens the palette over a dimmed window. The same keys close it.
  - The sidebar has a "Search or run a command ⌘K" button under the Captain brand. A click opens the palette.
  - Escape, a click outside the palette, or a run command closes it.
- Layout: a 640 px card with a 16 px radius, in the Captain palette colors.
  - Top row: a search icon, the query field, and an `esc` key hint.
  - Results: rows in sections. Each row has a 28 px tinted icon tile, the title, and a secondary line. The matched characters of the title are bold. The highlighted row shows a `↵` hint.
  - Footer: the Captain mark and "↑↓ Move · ↵ Run · esc Close".
- Commands, rebuilt from the workspace on each change:
  - Navigate: go to Containers, Images, Volumes, or Networks.
  - Actions: show all, running, or stopped containers (sets the list filter). For each container: Start or Stop, Restart, and Open in browser for each published port.
  - Containers: one row per container. It selects the container and opens the Containers page. If the current filter hides the container, the filter changes to All.
- Search: a case-insensitive fuzzy match on the title. The query letters must appear in order. Prefix matches, word starts, and adjacent letters score higher. Letters skipped between matches score lower. The best 50 matches show. A section sorts by its best row, and the rows of one section stay together.
- Empty query: the Navigate commands and the running containers.
- Keyboard: Up and Down move the highlight and wrap at the ends. The list scrolls to keep the highlighted row visible. Enter runs the highlighted command.
- Mouse: moving the pointer over a row highlights it. A click runs it.

## Out of scope

- Project-level commands, such as "Restart project shop".
- Log search from the palette.
- "⌘↵ Run in background" and "⇥ Actions" from the artboard footer.
- Recently used commands and ranking by use.
- Commands for images, volumes, and networks.

## Design notes

- The matcher lives in `captain-core` (`search::fuzzy_match`). It returns a score and the matched byte ranges, so the UI can bold them. Unit tests cover prefix, word-start, camel-case, and contiguous-run preferences.
- The palette defines its own actions (`SelectPrev`, `SelectNext`, `Confirm`, `Dismiss`) in the `CommandPalette` key context. The search field binds Up, Down, Enter, and Escape in its own `Input` context. The deeper context wins, so `palette::init` also binds the keys for `CommandPalette > Input`.

## Verification

1. Press ⌘K. The palette opens with the cursor in the search field. It lists Navigate commands and running containers.
2. Click the sidebar search button. The palette opens.
3. Type `rest`. The Restart rows come first, and "Rest" is bold in each title.
4. Press Down past the last row. The highlight wraps to the first row.
5. Press Enter on "Restart api". The container restarts, and the palette closes.
6. Run `docker run -d --name captain-test -p 8089:80 nginx:alpine`. Type `open cap`. Enter opens `http://localhost:8089` in the browser.
7. Type `stopped`, and run "Show stopped containers". The Containers page shows only stopped containers.
8. Press Escape, or click the dimmed area. The palette closes, and the window keeps keyboard focus.
9. Check the palette in dark and light mode against the artboard.

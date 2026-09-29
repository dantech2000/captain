# Feature 0012: Icon theme

- Milestone: M11
- Status: Planned

## Goal

Replace the stock Lucide glyphs with a Captain icon set, so every icon in the app looks like it belongs to the Glass helm app icon.

## Why

The app icon is now custom (the Glass helm, in `assets/icon/`). Inside the app, every glyph is still a stock Lucide icon. They are clean but generic, and a few do not fit Captain's subjects: containers, images, volumes, networks, and Compose projects.

## In scope

1. Design the set on the design canvas first, in the Glass helm style:
   - A 24 px grid with 2 px padding, a 1.75 px stroke, round caps and joins, and a 2 px corner radius.
   - A duotone option: the outline plus a soft accent fill at about 20% for selected and active states.
   - Light and dark mode checks at 13, 16, and 20 px, the sizes the app uses.
2. Cover every glyph the app shows today:
   - Navigation: Containers, Images, Volumes, Networks, Settings, Projects.
   - Container actions: Start, Stop, Pause, Resume, Restart, Delete, Open in browser, Terminal, Logs, Copy.
   - Resource actions: Pull, Prune, Create, Run, Up, Down.
   - States: running, paused, exited, healthy, unhealthy, starting, engine connected, engine stopped.
   - Inspector tabs: Overview, Logs, Terminal, Files, Stats.
   - Empty and error states: larger illustrations (64 to 96 px) for "No containers", "No images", and "Engine isn't running".
   - The menu bar icon: the template glyph for the tray, drawn from the same wheel.
3. Build it in:
   - SVG sources in `assets/icons/`, one file per glyph.
   - A `CaptainIcon` enum in `captain-ui` that maps each name to its SVG through GPUI's asset source, used the same way as `IconName` today.
   - Replace the Lucide uses view by view. Keep Lucide as the fallback for anything not yet drawn.

## Out of scope

- Custom fonts.
- Animated icons (a later polish pass could add a spinning wheel for "working").

## Verification

1. Every icon in the running app comes from the Captain set. A search for `IconName::` in `crates/captain-ui` finds only the documented fallbacks.
2. Each icon reads clearly at 13 px in light and dark mode.
3. The tray icon and the in-app brand mark use the same wheel as the app icon.

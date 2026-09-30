# Feature 0012: Icon theme

- Milestone: M11
- Status: Done

## Goal

Give Captain's resources their own icons, so a container, an image, or a Compose project looks like Captain and not like a stock icon set. Give the menu bar a glyph that tells the engine state by shape.

## Why

Every glyph in the app was a stock Lucide icon. Lucide has no good glyph for Captain's subjects: containers, images, volumes, Compose projects, and the engine. The v3 design ([0027](0027-v3-interface.md), rule 4) makes resource icons custom and duotone, and keeps Lucide for actions.

## In scope

1. Thirteen resource glyphs on a 24 px grid, from the design canvas (`CapIcon`): container, image, volume, network, stack (Compose project), pod, cluster (Kubernetes), snapshot, extension, engine, forward (port forward), exec (shell), and reclaim (free up space).
   - The line drawing has a 1.5 px stroke with round caps and joins.
   - Each glyph has a main body fill, and some have a second, lighter fill.
2. The duotone look: the body fill in the icon color at 22% opacity, the second fill at 10%, and the line drawing in the full color on top.
3. The files, in `assets/icons/`, three per glyph at most:
   - `<glyph>-line.svg`: the stroked line drawing.
   - `<glyph>-fill.svg`: the main body fill.
   - `<glyph>-fill2.svg`: the second fill, for image, volume, pod, cluster, and engine.
4. The code, in `captain-ui`:
   - A `CaptainIcon` enum with one variant per glyph.
   - `cap_icon(icon, size, color)`, which stacks the fill and line layers. At 12 px and below it draws the line only, because the fills blur at that size.
   - `CaptainAssets`, an asset source that serves the Captain files and passes every other path to GPUI Kit's Lucide set.
5. The places that show a resource use the Captain glyph:
   - The sidebar pages: Containers, Images, Volumes, Networks, Extensions, Snapshots, and Port Forwarding.
   - Project badges in the sidebar and on project cards (the stack glyph in the project color).
   - Container rows (the container glyph with a state dot) and the inspector headers for containers, images, volumes, and networks.
   - The empty states for those pages, the shell's "not running" state, and the "engine unreachable" state.
   - Palette results for pages and containers.
   - The engine card in the sidebar.
6. The menu bar glyph: a rounded-square bezel with a porthole, drawn in code as a template image, one shape per state:
   - Stopped: a dashed, empty porthole.
   - Starting: a half-filled porthole.
   - Running: a filled porthole.
   - Needs attention: a filled porthole and a notch dot at the top right. Captain shows it when Captain Engine failed to start, or when a container is restarting or unhealthy.

## Out of scope

- Custom glyphs for actions. Play, stop, restart, delete, search, copy, and chevrons stay Lucide.
- Diagnostics and Settings in the sidebar stay Lucide; they are not resources.
- Theme colors. Callers pass the color; the theme work is milestone M22.
- The "glass tile" and "extruded" looks from the canvas.
- Custom fonts and animated icons.

## Notes

- GPUI draws an SVG as a one-color alpha mask in the element's text color. It ignores the colors in the file (`Window::paint_svg` calls `render_alpha_mask`; see [gpui `window.rs`](https://github.com/zed-industries/zed/blob/main/crates/gpui/src/window.rs) and [`gpui::svg`](https://docs.rs/gpui/latest/gpui/fn.svg.html)). So one file cannot hold two tones. Each tone is its own file, and `cap_icon` stacks them in one box with `color.opacity(0.22)`, `color.opacity(0.10)`, and `color`.
- The files use `fill="black"` and `stroke="black"`. Only their alpha counts.
- GPUI Kit's `AllAssets` returns an error for a path it does not have ([`gpui_kit_assets::AllAssets`](https://docs.rs/gpui-kit-assets/0.7.0/gpui_kit_assets/struct.AllAssets.html)). `CaptainAssets` answers `captain/icons/…` itself and sends every other path to `AllAssets`. The files are compiled in with `include_bytes!`, so there is no new dependency and nothing to ship next to the binary.
- The menu bar icon is still a 36 by 36 RGBA buffer drawn in code with 4 by 4 samples per pixel for smooth edges. On macOS it is a template image, so only its alpha counts ([`tray_icon::TrayIcon::set_icon_as_template`](https://docs.rs/tray-icon/latest/tray_icon/struct.TrayIcon.html#method.set_icon_as_template), [Apple HIG: The menu bar](https://developer.apple.com/design/human-interface-guidelines/the-menu-bar)). The state is a shape, not a color, because a template image has no color.
- The tray snapshot keeps each container's health, so the icon can change when a container turns unhealthy. The menu still shows a failed Captain Engine as not running.

## Verification

1. `cargo test -p captain-ui`: every `CaptainIcon` file loads through `CaptainAssets`, and a Lucide path still loads.
2. `cargo test -p captain-app`: the four menu bar states draw different pixels.
3. In the app, look at the sidebar, a container row, the container inspector, the image inspector, the empty Volumes and Networks pages, and the ⌘K palette. Each resource shows its Captain glyph with a soft fill. Buttons such as Stop and Delete keep their Lucide glyphs.
4. Switch between light and dark mode. The glyphs keep their contrast at 16 px in the sidebar and at 22 px in the inspector header.
5. Stop Captain Engine and start it again. The menu bar glyph goes from dashed to half-filled to filled. Run `docker run -d --name captain-agent-restart --restart always busybox false`. The glyph shows the notch dot until you remove the container.

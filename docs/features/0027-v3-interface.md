# Feature 0027: The v3 interface

- Milestones: M22–M28, and M11
- Status: Built in M22–M28. See each feature spec and [ROADMAP.md](../../ROADMAP.md) for the state of each part. Later changes: the menu bar popover became the native menu ([0009](0009-menu-bar.md)), and the sidebar lost its engine header and cards ([0030](0030-project-window.md)).

## Goal

Rebuild the interface around what people complain about in Docker Desktop, Rancher Desktop, OrbStack, and Podman Desktop. The main window becomes project-first. The menu bar does the daily work. Every control explains itself in a status bar. Captain gets its own colors and icons.

The designs are on the Captain UI canvas (the "v3" and "Palette options" sections).

## Why

Research from GitHub issues sorted by reactions, Hacker News threads, and the products' docs:

1. Many people open Docker Desktop only to dismiss a prompt. The daily questions are "what is running" and "which port" ([docker/roadmap#221](https://github.com/docker/roadmap/issues/221)).
2. Disk use is opaque ([docker/for-mac#371](https://github.com/docker/for-mac/issues/371), 1.1k reactions; [docker/roadmap#13](https://github.com/docker/roadmap/issues/13)).
3. One confirm dialog per delete slows cleanup ([orbstack#1869](https://github.com/orbstack/orbstack/issues/1869)).
4. Errors do not say what to do: "socket not reachable", a hang at "starting" ([podman-desktop#1633](https://github.com/podman-desktop/podman-desktop/issues/1633)).
5. Kubernetes must be opt-in and invisible when off ([rancher-desktop#985](https://github.com/rancher-sandbox/rancher-desktop/issues/985), [podman-desktop#7982](https://github.com/podman-desktop/podman-desktop/issues/7982)).
6. Developers love k9s and lazydocker for keyboard-first density and logs that survive restarts ([lazydocker](https://github.com/jesseduffield/lazydocker)).
7. Ports and URLs matter more than IDs ([OrbStack domains](https://docs.orbstack.dev/docker/domains)).

## Milestones

| # | Name | Spec |
|---|------|------|
| M22 | Themes: Dusk (default), Periwinkle, Harbor, each light and dark | 0028 |
| M11 | Resource icons: duotone, in `assets/icons/` | 0012 (updated) |
| M23 | Status bar with hover help | 0029 |
| M24 | Project-first main window | 0030 |
| M25 | Storage and cleanup | 0031 |
| M26 | Menu bar popover, floating log, Dock badge | 0032 |
| M27 | Command grammar in the ⌘K palette | 0033 |
| M28 | Project map and staged changes | 0034 |

Order: M22, M11, and M23 first (they change shared building blocks), then M24–M26 in parallel, then M27 and M28.

## Design rules

1. **Blue-violet means you can act.** The theme's action color is for buttons, links, focus, and selection. It never marks a state.
2. **State colors mean state.** Green, amber, red, and cyan mark what a thing is doing. A state always has a word or a shape as well as a color.
3. **Label colors only tell projects apart.**
4. **Resource icons are custom and duotone.** Actions (play, stop, search) stay Lucide.
5. **Every control has a help sentence.** The status bar shows it on hover, and the ⌘K palette uses the same sentence.
6. **No nags.** No sign-in, no upsell, no forced windows, no update pop-ups.

## Theme tokens

The palette tables from the canvas. `dusk` is Harbor's neutrals with Periwinkle's action color and Celadon for running; Burning Flame is Dusk's warning color.

| Token | Dusk dark | Dusk light | Periwinkle dark | Periwinkle light | Harbor dark | Harbor light |
|---|---|---|---|---|---|---|
| Window | #1B2632 | #F4F0E8 | #0C0C16 | #FFFFFF | #1B2632 | #F4F0E8 |
| Sidebar | #16202A | #EAE4D8 | #11111D | #F5F5FF | #16202A | #EAE4D8 |
| Rail | #111921 | #E1DACB | #0A0A12 | #ECECF9 | #111921 | #E1DACB |
| Card | #202D3B | #FBF8F2 | #141424 | #FFFFFF | #202D3B | #FBF8F2 |
| Field | #223040 | #EEE9DF | #171729 | #F1F1FB | #223040 | #EEE9DF |
| Button | #2C3B4D | #FBF8F2 | #1C1C31 | #EFEFFA | #2C3B4D | #FBF8F2 |
| Border | #2E3D4F | #DAD2C3 | #25253C | #E1E1F2 | #2E3D4F | #DAD2C3 |
| Border strong | #3C4E64 | #C9C1B1 | #33334F | #CFCFE6 | #3C4E64 | #C9C1B1 |
| Text | #EEE9DF | #1B2632 | #EDEEFF | #0C0C16 | #EEE9DF | #1B2632 |
| Text 2 | #C9C1B1 | #3E4B5B | #A9ABD6 | #474868 | #C9C1B1 | #3E4B5B |
| Text 3 | #9A9486 | #56606B | #7E80AE | #5D5F85 | #9A9486 | #56606B |
| Action | #5F5FF0 | #5C5CF7 | #6060FF | #5C5CF7 | #FFB162 | #FFB162 |
| On action | #FFFFFF | #FFFFFF | #FFFFFF | #FFFFFF | #1B2632 | #1B2632 |
| Link | #B8BAFF | #4747DE | #B8BAFF | #4747DE | #FFB162 | #A35139 |
| Running | #B9F0D7 | #2F7A4D | #B9F0D7 | #1E8A5C | #8CCB9B | #2F7A4D |
| Warning | #FFB162 | #D98A0B | #FFC46B | #D98A0B | #F2D06B | #C8960F |
| Warning text | #FFC48A | #8F5A00 | #FFD08A | #8F5A00 | #F4D987 | #7A5A00 |
| Failing | #E8735A | #B8452C | #FF7A85 | #D23A4B | #E8735A | #B8452C |
| Info | #8FB4D9 | #2F5E8C | #C9E8FF | #2C7FB8 | #8FB4D9 | #2F5E8C |
| Log panel | #141D27 | #FBF8F2 | #08080F | #F8F8FE | #141D27 | #FBF8F2 |
| Labels | #C9785C #8FB4D9 #D4A5C9 #E6B980 #B9F0D7 | #A35139 #2F5E8C #8A4F7D #B06A1E #3F7A55 | #9D8CFF #7FDDC0 #FF8FC7 #FFB36B #C9E8FF | #7B61FF #1F9E83 #C23F8A #C26A12 #2C7FB8 | #C9785C #8FB4D9 #D4A5C9 #E6B980 #C9C1B1 | #A35139 #2F5E8C #8A4F7D #B06A1E #3F7A55 |

Text on the action color, and text colors on their window, card, and rail colors, meet 4.5:1. The rail is the icon column at the far left of the window (spec 0030), one step darker than the sidebar.

## Components

Captain draws its own look on top of GPUI Kit 0.7 parts. A kit part is used where it adds behavior Captain would otherwise lack; Captain's own widget stays where the kit has no fit (stat tiles, sparklines, grouped container cards, the stepper with units, the ⌘K palette).

- **Button** ([docs](https://gpui-kit.com/component/button)) is under `text_button`, `primary_button`, `small_primary_button`, `icon_button`, `action_button`, and the terminal tab strip's icons (`widgets/kit_button.rs`). It gives every button Tab focus, Enter and Space, a focus ring in the action color, and a button role with a name for VoiceOver. An icon-only button's name is its help sentence. The helpers set Captain's height, radius, and colors, so the call sites stay as they were. A div with the same id wraps each button and carries `.help()`, which the kit button cannot take. The div's `CaptainButton` key context hides a dialog's Enter binding while the button has focus, so Enter presses the focused button in a dialog too (`widgets/button_keys.rs`). A dialog's Cancel button has the same context.
- **Scrollable** ([docs](https://gpui-kit.com/component/scrollable)): `overflow_y_scrollbar()` on pages, inspectors, sheets, and lists, and `list_scrollbar` (a `vertical_scrollbar` over a `uniform_list`) on the logs, files, project log, build log, and scan results. The thumb uses Border strong, and Text 3 on hover.
- **Empty** ([docs](https://gpui-kit.com/component/empty)) is under `empty_note`, with Captain's duotone icon, sizes, and text colors.
- **Skeleton** ([docs](https://gpui-kit.com/component/skeleton)): `skeleton_rows` while the Images, Volumes, and Networks lists load, and `skeleton_lines` for inspector sections, processes, and files. The bars use the Border color.
- Already in use: Input, Select, Checkbox, Switch, Dialog, Notification, Spinner, Progress, Tag, Kbd, Tooltip, List, and Menu.

`theme/kit_theme.rs` maps the tokens onto the kit theme and sets the kit radius to 7 px, the radius of Captain's buttons.

## Out of scope

- A Windows or Linux engine host (M12).
- Pinned command blocks (the "Bridge" direction's dashboard). The ⌘K palette gets the grammar only.
- Domain names per container (OrbStack style). It needs a DNS resolver in the engine; a later milestone.

## Verification

Each milestone spec has its own steps. At the end, every screen of the canvas's v3 section exists in the app, in all six theme combinations.

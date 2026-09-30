# Feature 0028: themes

- Milestone: M22 (themes)
- Status: Implemented; checked with unit tests
- Builds on: [0027](0027-v3-interface.md), which has the design rules and the token table, and [0008](0008-settings.md), which added the accent color that this replaces
- Decision: [ADR 0004](../adr/0004-settings-file.md) (the settings file)

## Goal

The user picks one of three themes in Settings > Appearance: Dusk (the default), Periwinkle, or Harbor. Each theme has a light and a dark version. The Appearance control (System, Light, Dark) picks the version. Captain's own views and gpui-kit's components use the same colors.

## In scope

- Settings: `ThemeFamily` in `captain-core` (`dusk`, `periwinkle`, `harbor`), saved as `theme`. It replaces `accent`. Reading stays lenient: an unknown theme gives Dusk, and the old `accent` key is ignored, so an old file loads with its other settings.
- Tokens: `theme::Tokens` in `captain-ui` holds the table from 0027 as hex values, one set for each theme and mode.
- Palette: `Palette::new(family, dark)` builds the view colors from the tokens. The old field names stay, so views need few edits:
  - `accent` is the action color; `on_accent` is the text on it; `link` is the action color for text.
  - `green`, `orange`, and `red` are running, warning, and failing. `warn_text` is the warning color for text. `on_red` is the text on a failing badge.
  - `info` is the info color; `teal` is the same color. `indigo` is the first label color. `gray` is the tertiary text color, for quiet states.
  - `labels` has the five label colors. `project_color` picks one from the project name.
  - `group` and `card` are the card color. `hover` is a faint text-colored overlay for rows under the mouse. `nav_selected` is a tint of the action color.
  - `readable(color)` gives the text version of a fill color: the link color for the action color, and the warning text for the warning color. Pills and text buttons use it.
- State colors: one method each for `ContainerState`, `Health`, `HostStatus`, and `CheckState` in `theme/state_colors.rs`. The container list, inspector, sidebar engine card, Settings, the host summary, Diagnostics, and the ⌘K palette use them.
- Text on the action color uses `on_accent`, not white. Harbor's action color needs dark text.
- gpui-kit: `install_kit_themes` registers a light and a dark `ThemeConfig` built from the tokens as gpui-kit's `light_theme` and `dark_theme`. `apply_appearance` calls it before `Theme::change`, which loads the config for the mode. The system appearance observer, the Appearance control, and the theme cards all go through `apply_appearance`. Inputs, switches, selects, buttons, notifications, and scroll bars then use the theme's colors. Colors the config leaves out get gpui-kit's own fallbacks ([gpui-kit themes](https://gpui-kit.com/component/theme)).
- Appearance card: three theme cards in place of the accent swatches. Each card shows the theme's name on its window color, and a strip of its sidebar, action, running, warning, and failing colors for the current mode. The selected card has a ring in the action color.

## Out of scope

- The terminal's ANSI color table. It stays as it is.
- Theme files that the user writes, and more than three themes.
- The duotone resource icons (M11) and the status bar (M23).

## Notes

- The canvas has `#6666FF` for Periwinkle's dark action color. White text on it is 4.28:1, below the 4.5:1 that 0027 asks for. Captain uses `#6060FF` (4.55:1), and 0027's table now says so.
- Contrast uses the WCAG 2 formula ([contrast ratio](https://www.w3.org/TR/WCAG21/#dfn-contrast-ratio), [relative luminance](https://www.w3.org/TR/WCAG21/#dfn-relative-luminance)).
- `Theme::change` loads the registered config again even when the mode stays the same. A theme change therefore needs no second write to the colors.
- `sync_system_appearance` reads the window's appearance when it has a window, because the app-wide read crashed on Linux ([gpui-kit#104](https://github.com/longbridge/gpui-kit/issues/104)).

## Verification

1. Run `cargo test -p captain-core settings`. The tests cover the `theme` field, an unknown theme, and an old file with `accent`.
2. Run `cargo test -p captain-ui theme`. The test checks that text on the window and card colors, and text on the action color, meet 4.5:1 in all six combinations.
3. Start Captain and open Settings > Appearance. Three theme cards show. Dusk has the ring.
4. Click each card in Light and in Dark. The window, sidebar, buttons, pills, and links change at once. Harbor's primary buttons have dark text.
5. Open a dialog with a text field, a switch, and a select, for example Settings > Captain Engine resources or the Run dialog. The field border, the focus ring, the switch, and the dropdown use the theme's colors.
6. Set Appearance to System and switch the macOS appearance. Captain and the gpui-kit components follow.
7. Quit and start Captain. The theme stays. `settings.json` has `"theme"` and no `"accent"`.

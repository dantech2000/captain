use captain_core::store::SelectMode;
use gpui_kit::ClickEvent;

/// How a click on a list row changes the selection: Shift selects a range, and
/// Cmd (macOS) or Ctrl (Windows, Linux) adds or removes the row.
pub fn select_mode(event: &ClickEvent) -> SelectMode {
    let keys = event.modifiers();
    SelectMode::from_keys(keys.shift, keys.secondary())
}

use gpui_kit::*;

/// The sheet's key context. Its bindings apply only while the sheet has focus.
pub const CONTEXT: &str = "NewSheet";

gpui_kit::actions!(
    new_sheet,
    [SelectPrev, SelectNext, Confirm, Pick1, Pick2, Pick3, Pick4]
);

gpui_kit::actions!(captain, [NewProject]);

/// Binds ⌘N (Ctrl N elsewhere) for the New sheet, and the sheet's keys: up and
/// down move the highlight, enter opens it, and ⌘1 to ⌘4 open a card.
pub fn init(cx: &mut App) {
    let context = Some(CONTEXT);
    let mut bindings = vec![
        KeyBinding::new("up", SelectPrev, context),
        KeyBinding::new("down", SelectNext, context),
        KeyBinding::new("enter", Confirm, context),
    ];
    for modifier in ["cmd", "ctrl"] {
        bindings.extend([
            KeyBinding::new(&format!("{modifier}-n"), NewProject, None),
            KeyBinding::new(&format!("{modifier}-1"), Pick1, context),
            KeyBinding::new(&format!("{modifier}-2"), Pick2, context),
            KeyBinding::new(&format!("{modifier}-3"), Pick3, context),
            KeyBinding::new(&format!("{modifier}-4"), Pick4, context),
        ]);
    }
    cx.bind_keys(bindings);
}

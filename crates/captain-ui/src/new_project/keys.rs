use gpui_kit::*;

/// The sheet's key context. Its bindings apply only while the sheet has focus.
pub const CONTEXT: &str = "NewSheet";
/// The key context of a step's form: Tab moves between fields, and ⌘Return makes
/// the project.
pub const FORM_CONTEXT: &str = "NewSheetForm";

gpui_kit::actions!(
    new_sheet,
    [
        SelectPrev, SelectNext, Confirm, Pick1, Pick2, Pick3, Pick4, FocusNext, FocusPrev, Submit
    ]
);

gpui_kit::actions!(captain, [NewProject]);

/// Binds ⌘N (Ctrl N elsewhere) for the New sheet, and the sheet's keys: up and
/// down move the highlight, enter opens it, and ⌘1 to ⌘4 open a card. In a form,
/// Tab and Shift Tab move between fields and ⌘Return submits.
pub fn init(cx: &mut App) {
    let context = Some(CONTEXT);
    let form = Some(FORM_CONTEXT);
    let mut bindings = vec![
        KeyBinding::new("up", SelectPrev, context),
        KeyBinding::new("down", SelectNext, context),
        KeyBinding::new("enter", Confirm, context),
        KeyBinding::new("tab", FocusNext, form),
        KeyBinding::new("shift-tab", FocusPrev, form),
    ];
    for modifier in ["cmd", "ctrl"] {
        bindings.extend([
            KeyBinding::new(&format!("{modifier}-n"), NewProject, None),
            KeyBinding::new(&format!("{modifier}-1"), Pick1, context),
            KeyBinding::new(&format!("{modifier}-2"), Pick2, context),
            KeyBinding::new(&format!("{modifier}-3"), Pick3, context),
            KeyBinding::new(&format!("{modifier}-4"), Pick4, context),
            KeyBinding::new(&format!("{modifier}-enter"), Submit, form),
        ]);
    }
    cx.bind_keys(bindings);
}

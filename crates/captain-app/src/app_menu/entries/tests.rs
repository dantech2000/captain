use gpui_kit::{Action, KeyBinding, NoAction};

use super::{Entry, menu_bindings, menu_specs};
use crate::actions::app_bindings;

/// Every key binding Captain adds, in the order the app registers them.
fn all_bindings() -> Vec<KeyBinding> {
    let mut bindings = captain_ui::new_project_bindings();
    bindings.extend(app_bindings());
    bindings.extend(menu_bindings());
    bindings
}

/// The binding GPUI shows next to `action`: the first one without a key context,
/// else the first one (see `create_menu_item` in GPUI's macOS platform).
fn shown<'a>(action: &dyn Action, bindings: &'a [KeyBinding]) -> Option<&'a KeyBinding> {
    let mut matching = bindings.iter().filter(|b| b.action().partial_eq(action));
    let first = matching.clone().next();
    matching.find(|b| b.predicate().is_none()).or(first)
}

#[test]
fn every_shortcut_in_the_menus_matches_a_key_binding() {
    let bindings = all_bindings();
    for spec in menu_specs() {
        for entry in spec.entries {
            let Entry::Item {
                name, action, keys, ..
            } = entry
            else {
                continue;
            };
            let Some(keys) = keys else { continue };
            let expected = KeyBinding::new(keys, NoAction, None);
            let binding = shown(action.as_ref(), &bindings)
                .unwrap_or_else(|| panic!("{} > {name} has no key binding", spec.name));
            assert_eq!(
                binding.keystrokes(),
                expected.keystrokes(),
                "{} > {name} would show other keys",
                spec.name
            );
        }
    }
}

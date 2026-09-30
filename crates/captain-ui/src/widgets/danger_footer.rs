use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::dialog::{DialogAction, DialogClose, DialogFooter};
use gpui_kit::*;

use crate::help::{HelpExt, Hint};

/// The footer of an alert dialog that asks before removing something: Cancel and a
/// red confirm button, each with a status bar sentence. It replaces gpui-kit's
/// default footer, whose buttons take no help. The dialog's `on_ok` still runs on
/// confirm.
pub fn danger_footer(label: impl Into<SharedString>, help: impl Into<Hint>) -> DialogFooter {
    footer(Button::new("ok").label(label.into()).danger(), help)
}

/// Like [`danger_footer`], with the accent color, for a confirm that is not a
/// removal, such as applying saved files.
pub fn confirm_footer(label: impl Into<SharedString>, help: impl Into<Hint>) -> DialogFooter {
    footer(Button::new("ok").label(label.into()).primary(), help)
}

fn footer(confirm: Button, help: impl Into<Hint>) -> DialogFooter {
    DialogFooter::new()
        .child(
            div()
                .id("dialog-cancel")
                .child(DialogClose::new().trigger(|button| button.label("Cancel")))
                .help("Close this dialog. Nothing changes."),
        )
        .child(
            div()
                .id("dialog-confirm")
                .child(DialogAction::new().child(confirm))
                .help(help),
        )
}

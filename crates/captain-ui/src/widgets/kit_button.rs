//! Captain's buttons sit on gpui-kit's Button, which gives them keyboard focus (Tab,
//! then Enter or Space, also inside a dialog), a focus ring, and a button role and name for screen readers.
//! The helpers keep Captain's size, radius, and colors. See
//! https://gpui-kit.com/component/button.

use gpui_kit::component::Disableable;
use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_kit::*;

use super::BUTTON_CONTEXT;

/// The colors of an enabled button: its fill, its text, and its fill under the mouse
/// and while pressed.
#[derive(Debug, Clone, Copy)]
pub struct Look {
    pub bg: Hsla,
    pub fg: Hsla,
    pub hover: Hsla,
}

/// A kit Button that takes its [`Look`] when it renders, because the kit's custom
/// colors start from the theme, which only render can read.
#[derive(IntoElement)]
struct KitButton {
    button: Button,
    look: Look,
}

impl RenderOnce for KitButton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let Look { bg, fg, hover } = self.look;
        let colors = ButtonCustomVariant::new(cx)
            .color(bg)
            .foreground(fg)
            .hover(hover)
            .active(hover);
        self.button.custom(colors)
    }
}

/// A kit button inside a div with the same id. The caller sizes the div, and the
/// button fills it; the div carries `.help()`, which needs a stateful element, and
/// keeps its hover when the button is disabled, and its key context lets Enter and
/// Space reach the button inside a dialog. `name` is what a screen reader says.
/// `style` sets the button's shape, colors, and content.
pub fn kit_button(
    id: impl Into<ElementId>,
    name: impl Into<SharedString>,
    look: Look,
    enabled: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    style: impl FnOnce(Button) -> Button,
) -> Stateful<Div> {
    let id = id.into();
    let button = Button::new(id.clone())
        .accessibility_label(name)
        .disabled(!enabled)
        .on_click(on_click);
    div()
        .id(id)
        .key_context(BUTTON_CONTEXT)
        .flex()
        .child(KitButton {
            button: style(button),
            look,
        })
}

use gpui_kit::component::Icon;
use gpui_kit::*;

use super::kit_button::{Look, kit_button};
use crate::help::{HelpExt, Hint};
use crate::theme::Palette;

/// A square button that shows only an icon. `help` is its status bar sentence, the
/// only text that says what it does, so a screen reader says it too.
pub fn icon_button(
    id: impl Into<ElementId>,
    icon: impl Into<Icon>,
    help: impl Into<Hint>,
    palette: &Palette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let help = help.into();
    let look = Look {
        bg: palette.button,
        fg: palette.text2,
        hover: palette.nav_selected,
    };
    // The border is on the outer div: the kit paints a custom button's border in
    // its hover color.
    let border = palette.sep;
    let icon = Icon::new(icon).size(px(13.));
    kit_button(id, help.text.clone(), look, true, on_click, |button| {
        button
            .size_full()
            .p_0()
            .rounded(px(6.))
            .cursor_pointer()
            .child(icon)
    })
    .size(px(26.))
    .rounded(px(7.))
    .border_1()
    .border_color(border)
    .flex_shrink_0()
    .help(help)
}

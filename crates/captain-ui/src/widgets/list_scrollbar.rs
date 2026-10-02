use gpui_kit::component::scroll::{ScrollableElement, ScrollbarHandle};
use gpui_kit::*;

/// A virtual list (a `uniform_list` that tracks `scroll`) with gpui-kit's vertical
/// scrollbar over its right edge. The box takes the list's place in a column: it
/// grows, and it may shrink below its content. See
/// https://gpui-kit.com/component/scrollable.
pub fn list_scrollbar<H: ScrollbarHandle + Clone>(list: impl IntoElement, scroll: &H) -> Div {
    div()
        .relative()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .child(list)
        .vertical_scrollbar(scroll)
}

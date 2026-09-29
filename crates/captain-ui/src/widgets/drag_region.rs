use gpui_kit::*;

/// A strip that moves the window, for use with a transparent title bar.
/// A double click zooms the window, like a native title bar.
pub fn drag_region(id: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .window_control_area(WindowControlArea::Drag)
        .on_mouse_down(MouseButton::Left, |event, window, _| {
            if event.click_count == 2 {
                window.titlebar_double_click();
            } else {
                window.start_window_move();
            }
        })
}

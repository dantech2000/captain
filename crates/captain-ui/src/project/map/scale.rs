use captain_core::project_map::Rect;
use gpui_kit::*;

/// Map units to pixels at the current zoom.
#[derive(Debug, Clone, Copy)]
pub struct Scale(pub f32);

impl Scale {
    pub fn px(self, units: f32) -> Pixels {
        px(units * self.0)
    }

    /// An absolutely placed box at `rect`.
    pub fn place(self, rect: Rect) -> Div {
        div()
            .absolute()
            .left(self.px(rect.x))
            .top(self.px(rect.y))
            .w(self.px(rect.w))
            .h(self.px(rect.h))
    }
}

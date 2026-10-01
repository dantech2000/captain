use gpui_kit::px;

use super::{MIN_HEIGHT, height_for};

#[test]
fn height_follows_the_mouse_within_bounds() {
    assert_eq!(height_for(px(900.), px(600.), 900.), 300.);
    assert_eq!(height_for(px(900.), px(880.), 900.), MIN_HEIGHT);
    // The page above keeps 240 px.
    assert_eq!(height_for(px(900.), px(0.), 900.), 660.);
    // A short window still gets the smallest panel.
    assert_eq!(height_for(px(300.), px(0.), 300.), MIN_HEIGHT);
}

use gpui_kit::px;

use super::{MAX_WIDTH, MIN_WIDTH, width_for};

#[test]
fn the_width_follows_the_mouse_within_its_limits() {
    let wide = 2000.;
    assert_eq!(width_for(px(1400.), px(900.), wide), 500.);
    assert_eq!(width_for(px(1400.), px(1300.), wide), MIN_WIDTH);
    assert_eq!(width_for(px(1400.), px(100.), wide), MAX_WIDTH);
    // 1128 px beside the rail and the list keeps 700 for the page: the panel stops at 428.
    assert_eq!(width_for(px(1440.), px(100.), 1128.), 428.);
}

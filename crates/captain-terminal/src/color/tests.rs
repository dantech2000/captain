use super::{Color, Rgb};

#[test]
fn the_cube_and_the_gray_ramp_follow_xterm() {
    assert_eq!(Color::xterm_rgb(7), None);
    assert_eq!(Color::xterm_rgb(16), Some(Rgb::new(0, 0, 0)));
    assert_eq!(Color::xterm_rgb(196), Some(Rgb::new(255, 0, 0)));
    assert_eq!(Color::xterm_rgb(231), Some(Rgb::new(255, 255, 255)));
    assert_eq!(Color::xterm_rgb(232), Some(Rgb::new(8, 8, 8)));
    assert_eq!(Color::xterm_rgb(255), Some(Rgb::new(238, 238, 238)));
}

use super::{Light, rgba};
use crate::tray::icon::SIZE;

#[test]
fn a_dot_is_solid_in_the_middle_and_clear_at_the_corner() {
    let dot = rgba(Light::Green);
    let pixel = |x: u32, y: u32| {
        let at = ((y * SIZE + x) * 4) as usize;
        dot[at..at + 4].to_vec()
    };
    let middle = SIZE / 2;
    assert_eq!(pixel(middle, middle), [0x34, 0xC7, 0x59, 255]);
    assert_eq!(pixel(0, 0)[3], 0);
}

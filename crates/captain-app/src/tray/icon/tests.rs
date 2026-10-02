use super::{SIZE, TURN_FRAMES, rgba};
use crate::tray::dot::Light;
use crate::tray::look::{Dot, IconLook, Wheel};

fn look(wheel: Wheel, dot: Option<Dot>) -> IconLook {
    IconLook { wheel, dot }
}

#[test]
fn each_look_draws_a_different_icon() {
    let looks = [
        look(Wheel::Dim, None),
        look(Wheel::Turning, Some(Dot::Colored(Light::Amber))),
        look(Wheel::Full, Some(Dot::Colored(Light::Green))),
        look(Wheel::Full, Some(Dot::Colored(Light::Red))),
        look(Wheel::Full, Some(Dot::Plain)),
        look(Wheel::Full, None),
    ];
    let icons: Vec<Vec<u8>> = looks.iter().map(|&l| rgba(l, 1, [0; 3], true)).collect();
    for (i, icon) in icons.iter().enumerate() {
        assert_eq!(icon.len(), (SIZE * SIZE * 4) as usize);
        for other in &icons[i + 1..] {
            assert_ne!(icon, other, "{:?}", looks[i]);
        }
    }
    // Unpainted, a colored dot leaves only its place clear, for macOS to draw it.
    let red = look(Wheel::Full, Some(Dot::Colored(Light::Red)));
    let clear = rgba(red, 0, [0; 3], false);
    assert!(clear.chunks(4).all(|px| px[..3] == [0; 3]));
    assert_ne!(clear, rgba(look(Wheel::Full, None), 0, [0; 3], false));
}

#[test]
fn the_wheel_turns_and_loops() {
    let turning = look(Wheel::Turning, Some(Dot::Colored(Light::Amber)));
    let frame = |n| rgba(turning, n, [0; 3], true);
    assert_ne!(frame(0), frame(1));
    assert_eq!(frame(0), frame(TURN_FRAMES));
    let full = look(Wheel::Full, None);
    assert_eq!(rgba(full, 0, [0; 3], true), rgba(full, 3, [0; 3], true));
}

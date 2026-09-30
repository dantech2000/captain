use super::{Rect, Screen, popover_rect};

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}

/// A 1512 by 982 Retina screen with a 25 point menu bar, and a 1920 by 1080 screen
/// at scale 1 to its right.
fn screens() -> [Screen<u32>; 2] {
    [
        Screen {
            id: 1,
            frame: rect(0., 0., 1512., 982.),
            visible: rect(0., 25., 1512., 957.),
            scale: 2.,
        },
        Screen {
            id: 2,
            frame: rect(1512., 0., 1920., 1080.),
            visible: rect(1512., 25., 1920., 1055.),
            scale: 1.,
        },
    ]
}

#[test]
fn the_popover_opens_under_the_icon_inside_its_screen() {
    // An icon near the right edge, in physical pixels: it clamps to the edge.
    let icon = rect(2880., 0., 44., 48.);
    let (id, placed) = popover_rect(icon, &screens(), 380., 600., true).unwrap();
    assert_eq!(id, 1);
    assert_eq!(placed, rect(1124., 28., 380., 600.));

    // On the second screen, listed first because the mouse is there, the rectangle
    // is relative to that screen, and a tall popover is cut to the screen's height.
    let [first, second] = screens();
    let icon = rect(2000., 0., 22., 24.);
    let (id, placed) = popover_rect(icon, &[second, first], 380., 2000., true).unwrap();
    assert_eq!(id, 2);
    assert_eq!(placed, rect(2011. - 190. - 1512., 28., 380., 1039.));

    // Off every screen: no popover.
    assert_eq!(
        popover_rect(rect(-500., -500., 10., 10.), &screens(), 380., 600., true),
        None
    );
}

#[test]
fn a_bottom_taskbar_puts_the_popover_above_the_icon() {
    // Windows at 150%: the taskbar icon sits at the bottom, and rectangles are global.
    let screen = Screen {
        id: 7,
        frame: rect(0., 0., 1280., 720.),
        visible: rect(0., 0., 1280., 688.),
        scale: 1.5,
    };
    let icon = rect(1500., 1035., 48., 45.);
    let (_, placed) = popover_rect(icon, &[screen], 380., 400., false).unwrap();
    assert_eq!(placed, rect(1016. - 190., 690. - 4. - 400., 380., 400.));
}

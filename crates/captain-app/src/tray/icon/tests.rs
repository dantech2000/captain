use super::{SIZE, rgba};
use crate::tray::snapshot::EngineStatus;

fn alpha(pixels: &[u8], x: u32, y: u32) -> u8 {
    pixels[((y * SIZE + x) * 4 + 3) as usize]
}

#[test]
fn the_icon_is_size_by_size_rgba_in_one_color() {
    let pixels = rgba(EngineStatus::Running, [10, 20, 30]);
    assert_eq!(pixels.len(), (SIZE * SIZE * 4) as usize);
    assert!(pixels.chunks(4).all(|p| p[..3] == [10, 20, 30]));
}

#[test]
fn the_wheel_stays_inside_the_canvas() {
    let pixels = rgba(EngineStatus::Running, [0, 0, 0]);
    for i in 0..SIZE {
        for (x, y) in [(i, 0), (i, SIZE - 1), (0, i), (SIZE - 1, i)] {
            assert_eq!(alpha(&pixels, x, y), 0, "edge pixel ({x}, {y})");
        }
    }
}

#[test]
fn the_rim_and_hub_are_solid_and_the_center_is_open() {
    let pixels = rgba(EngineStatus::Running, [0, 0, 0]);
    // The rim, between two spokes.
    let (x, y) = (18 + 4, 18 + 10);
    assert_eq!(alpha(&pixels, x, y), 255);
    // The hub ring, and the hole in its middle.
    assert_eq!(alpha(&pixels, 18 + 2, 18 + 2), 255);
    assert!(alpha(&pixels, 17, 17) < 255);
}

#[test]
fn edges_are_anti_aliased() {
    let pixels = rgba(EngineStatus::Running, [0, 0, 0]);
    let partial = pixels.chunks(4).filter(|p| p[3] > 0 && p[3] < 255).count();
    assert!(partial > 20, "only {partial} partial pixels");
}

#[test]
fn the_running_wheel_is_mirror_symmetric() {
    let pixels = rgba(EngineStatus::Running, [0, 0, 0]);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let mirrored = alpha(&pixels, SIZE - 1 - x, y);
            assert!(alpha(&pixels, x, y).abs_diff(mirrored) <= 1, "({x}, {y})");
        }
    }
}

#[test]
fn starting_is_dim_and_stopped_adds_a_solid_slash() {
    let running = rgba(EngineStatus::Running, [0, 0, 0]);
    let starting = rgba(EngineStatus::Starting, [0, 0, 0]);
    let stopped = rgba(EngineStatus::Stopped, [0, 0, 0]);
    let rim = (18 + 4, 18 + 10);
    assert_eq!(alpha(&starting, rim.0, rim.1), 128);
    assert!(starting.chunks(4).all(|p| p[3] <= 128));

    // The slash crosses the middle of the icon, where the running wheel is open.
    assert_eq!(alpha(&running, 17, 18), 0);
    assert_eq!(alpha(&stopped, 17, 18), 255);
    assert_ne!(running, stopped);
}

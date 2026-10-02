//! The menu bar icon: a ship's wheel, drawn in code so there is no file to ship.
//! [`IconLook`] says how: full, turning, or dimmed, and with a dot or not.

use std::f32::consts::FRAC_PI_4;

use super::look::{Dot, IconLook, Wheel};

/// 18 points at 2x, the height of a macOS menu bar icon.
pub const SIZE: u32 = 36;

/// The frames of one turn step: the wheel looks the same after an eighth of a turn,
/// so this many frames loop smoothly.
pub const TURN_FRAMES: u32 = 8;

/// Samples per pixel along each axis, for anti-aliased edges.
const SAMPLES: u32 = 4;

const CENTER: f32 = SIZE as f32 / 2.;
const RIM: (f32, f32) = (10., 12.5);
const HUB: (f32, f32) = (1.6, 4.);
const SPOKE_HALF_WIDTH: f32 = 1.1;
const SPOKE_LENGTH: f32 = 14.;
/// The handles outside the rim, from and to this radius.
const HANDLE: (f32, f32) = (13., 15.4);
const HANDLE_HALF_WIDTH: f32 = 1.5;
/// The status dot's center and radius, over the upper right handle, with a clear
/// gap around it so it reads apart from the wheel.
pub const DOT: (f32, f32, f32) = (29.5, 6.5, 5.);
const DOT_GAP: f32 = 2.;
/// How strong a stopped engine's wheel shows, like a macOS menu bar icon that is
/// off.
const DIM: f32 = 0.4;

/// The icon as RGBA rows, `SIZE` by `SIZE`, in `color`. `frame` turns the wheel by
/// a share of an eighth of a turn; it only matters while it turns. A plain dot has
/// `color`, a colored dot its light's color. With `paint_dot` false the dot's place
/// stays clear, for macOS, which draws the colored dot itself. On macOS the color
/// of a template image does not matter: only its alpha counts.
pub fn rgba(look: IconLook, frame: u32, color: [u8; 3], paint_dot: bool) -> Vec<u8> {
    let turn = match look.wheel {
        Wheel::Turning => (frame % TURN_FRAMES) as f32 * FRAC_PI_4 / TURN_FRAMES as f32,
        _ => 0.,
    };
    let dim = if look.wheel == Wheel::Dim { DIM } else { 1. };
    let dot_color = match look.dot {
        Some(Dot::Colored(light)) => light.rgb(),
        _ => color,
    };
    let has_dot = look.dot.is_some();
    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            // The gap keeps the wheel and the dot apart, so a pixel shows one of them.
            let wheel = coverage(x, y, |px, py| {
                in_wheel(px, py, turn) && !(has_dot && dot_distance(px, py) <= DOT.2 + DOT_GAP)
            });
            let dot = coverage(x, y, |px, py| has_dot && dot_distance(px, py) <= DOT.2);
            let (rgb, alpha) = if paint_dot && dot > 0. {
                (dot_color, dot)
            } else {
                (color, wheel * dim)
            };
            pixels.extend_from_slice(&rgb);
            pixels.push((alpha * 255.).round() as u8);
        }
    }
    pixels
}

/// The distance from the dot's center.
fn dot_distance(x: f32, y: f32) -> f32 {
    (x - DOT.0).hypot(y - DOT.1)
}

/// The share of a grid of points inside pixel `(x, y)` for which `shape` is true.
pub fn coverage(x: u32, y: u32, shape: impl Fn(f32, f32) -> bool) -> f32 {
    let step = 1. / SAMPLES as f32;
    let mut total = 0;
    for sy in 0..SAMPLES {
        for sx in 0..SAMPLES {
            let px = x as f32 + (sx as f32 + 0.5) * step;
            let py = y as f32 + (sy as f32 + 0.5) * step;
            total += u32::from(shape(px, py));
        }
    }
    total as f32 / (SAMPLES * SAMPLES) as f32
}

/// The rim, hub, spokes, and handles, turned by `turn`.
fn in_wheel(x: f32, y: f32, turn: f32) -> bool {
    let (dx, dy) = (x - CENTER, y - CENTER);
    let radius = dx.hypot(dy);
    if (RIM.0..=RIM.1).contains(&radius) {
        return true;
    }
    if (HUB.0..=HUB.1).contains(&radius) {
        return true;
    }
    (0..8).any(|spoke| {
        let angle = spoke as f32 * FRAC_PI_4 + turn;
        let (sin, cos) = angle.sin_cos();
        let at = |r: f32| (CENTER + r * cos, CENTER + r * sin);
        let on_spoke = radius >= HUB.0
            && segment_distance((x, y), at(0.), at(SPOKE_LENGTH)) <= SPOKE_HALF_WIDTH;
        on_spoke || segment_distance((x, y), at(HANDLE.0), at(HANDLE.1)) <= HANDLE_HALF_WIDTH
    })
}

/// The distance from `p` to the segment from `a` to `b`.
fn segment_distance(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (abx, aby) = (b.0 - a.0, b.1 - a.1);
    let (apx, apy) = (p.0 - a.0, p.1 - a.1);
    let t = ((apx * abx + apy * aby) / (abx * abx + aby * aby)).clamp(0., 1.);
    (apx - t * abx).hypot(apy - t * aby)
}

#[cfg(test)]
mod tests;

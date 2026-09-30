//! The menu bar icon: a ship's wheel, drawn in code so there is no file to ship.
//! It tells the engine state: full while running, turning while starting, dimmed
//! while stopped, and with a notch dot when something needs you.

use std::f32::consts::FRAC_PI_4;

use super::snapshot::EngineStatus;

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
/// The notch dot for an engine that needs attention, over the upper right handle,
/// with a clear gap around it so it reads apart from the wheel.
const NOTCH: (f32, f32) = (29., 7.);
const NOTCH_RADIUS: f32 = 4.;
const NOTCH_GAP: f32 = 2.;
/// How strong a stopped engine's wheel shows, like a macOS menu bar icon that is
/// off.
const DIM: f32 = 0.4;

/// The icon for `status` as RGBA rows, `SIZE` by `SIZE`, in `color`. `frame` turns
/// the wheel by a share of an eighth of a turn; it only matters while starting. On
/// macOS the color does not matter: the image is a template, and only its alpha
/// counts.
pub fn rgba(status: EngineStatus, frame: u32, color: [u8; 3]) -> Vec<u8> {
    let turn = (frame % TURN_FRAMES) as f32 * FRAC_PI_4 / TURN_FRAMES as f32;
    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let mut alpha = coverage(x, y, |px, py| inside(status, turn, px, py));
            if status == EngineStatus::Stopped {
                alpha *= DIM;
            }
            pixels.extend_from_slice(&color);
            pixels.push((alpha * 255.).round() as u8);
        }
    }
    pixels
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

/// Whether the point is part of the icon for `status`.
fn inside(status: EngineStatus, turn: f32, x: f32, y: f32) -> bool {
    if status == EngineStatus::NeedsAttention {
        let notch = (x - NOTCH.0).hypot(y - NOTCH.1);
        if notch <= NOTCH_RADIUS + NOTCH_GAP {
            return notch <= NOTCH_RADIUS;
        }
    }
    let turn = if status == EngineStatus::Starting {
        turn
    } else {
        0.
    };
    in_wheel(x, y, turn)
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

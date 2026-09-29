//! The menu bar icon: a ship's wheel, drawn in code so there is no file to ship.

use std::f32::consts::FRAC_PI_4;

use super::snapshot::EngineStatus;

/// 18 points at 2x, the height of a macOS menu bar icon.
pub const SIZE: u32 = 36;

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
/// A dim wheel for an engine that is starting or stopped.
const DIM: f32 = 0.5;
/// A stopped engine gets a slash from the lower left to the upper right, with a
/// clear gap on each side so it reads at 18 points.
const SLASH: ((f32, f32), (f32, f32)) = ((6., 30.), (30., 6.));
const SLASH_HALF_WIDTH: f32 = 1.4;
const SLASH_GAP: f32 = 3.;

/// The icon for `status` as RGBA rows, `SIZE` by `SIZE`, in `color`. On macOS the
/// color does not matter: the image is a template, and only its alpha counts.
pub fn rgba(status: EngineStatus, color: [u8; 3]) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let alpha = coverage(x, y, |px, py| alpha_at(status, px, py));
            pixels.extend_from_slice(&color);
            pixels.push((alpha * 255.).round() as u8);
        }
    }
    pixels
}

/// The mean of `shape` over a grid of points inside pixel `(x, y)`.
fn coverage(x: u32, y: u32, shape: impl Fn(f32, f32) -> f32) -> f32 {
    let step = 1. / SAMPLES as f32;
    let mut total = 0.;
    for sy in 0..SAMPLES {
        for sx in 0..SAMPLES {
            let px = x as f32 + (sx as f32 + 0.5) * step;
            let py = y as f32 + (sy as f32 + 0.5) * step;
            total += shape(px, py);
        }
    }
    total / (SAMPLES * SAMPLES) as f32
}

/// The opacity at one point: 0 outside, 1 inside, or dimmed.
fn alpha_at(status: EngineStatus, x: f32, y: f32) -> f32 {
    let wheel = if in_wheel(x, y) { 1. } else { 0. };
    match status {
        EngineStatus::Running => wheel,
        EngineStatus::Starting => wheel * DIM,
        EngineStatus::Stopped => {
            let distance = segment_distance((x, y), SLASH.0, SLASH.1);
            if distance <= SLASH_HALF_WIDTH {
                1.
            } else if distance <= SLASH_GAP {
                0.
            } else {
                wheel * DIM
            }
        }
    }
}

fn in_wheel(x: f32, y: f32) -> bool {
    let (dx, dy) = (x - CENTER, y - CENTER);
    let radius = dx.hypot(dy);
    if (RIM.0..=RIM.1).contains(&radius) {
        return true;
    }
    if radius < HUB.0 {
        return false;
    }
    if radius <= HUB.1 {
        return true;
    }
    (0..8).any(|spoke| {
        let angle = spoke as f32 * FRAC_PI_4;
        let (sin, cos) = angle.sin_cos();
        let at = |r: f32| (CENTER + r * cos, CENTER + r * sin);
        segment_distance((x, y), at(0.), at(SPOKE_LENGTH)) <= SPOKE_HALF_WIDTH
            || segment_distance((x, y), at(HANDLE.0), at(HANDLE.1)) <= HANDLE_HALF_WIDTH
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

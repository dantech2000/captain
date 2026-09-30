//! The menu bar icon: a porthole in a rounded bezel, drawn in code so there is no
//! file to ship. The porthole's fill tells the engine state by shape.

use std::f32::consts::TAU;

use super::snapshot::EngineStatus;

/// 18 points at 2x, the height of a macOS menu bar icon.
pub const SIZE: u32 = 36;

/// Samples per pixel along each axis, for anti-aliased edges.
const SAMPLES: u32 = 4;

/// The design is on a 24 unit grid; the icon is 36 pixels.
const SCALE: f32 = SIZE as f32 / 24.;
const CENTER: f32 = SIZE as f32 / 2.;
/// Half the 1.8 unit stroke.
const HALF_STROKE: f32 = 0.9 * SCALE;
/// The bezel: a square from 4 to 20 units with 3 unit corners.
const BEZEL_HALF: f32 = 8. * SCALE;
const BEZEL_RADIUS: f32 = 3. * SCALE;
/// The porthole: a 4.5 unit circle in the middle.
const PORTHOLE: f32 = 4.5 * SCALE;
/// A stopped engine's porthole is dashed: this many dashes, each this share of its
/// period.
const DASHES: f32 = 6.;
const DASH_ON: f32 = 0.45;
/// The notch dot for an engine that needs attention, with a clear gap around it so
/// it reads apart from the bezel.
const NOTCH: (f32, f32) = (19. * SCALE, 5.5 * SCALE);
const NOTCH_RADIUS: f32 = 2.5 * SCALE;
const NOTCH_GAP: f32 = 1.5;

/// The icon for `status` as RGBA rows, `SIZE` by `SIZE`, in `color`. On macOS the
/// color does not matter: the image is a template, and only its alpha counts.
pub fn rgba(status: EngineStatus, color: [u8; 3]) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let alpha = coverage(x, y, |px, py| inside(status, px, py));
            pixels.extend_from_slice(&color);
            pixels.push((alpha * 255.).round() as u8);
        }
    }
    pixels
}

/// The share of a grid of points inside pixel `(x, y)` for which `shape` is true.
fn coverage(x: u32, y: u32, shape: impl Fn(f32, f32) -> bool) -> f32 {
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
fn inside(status: EngineStatus, x: f32, y: f32) -> bool {
    let (dx, dy) = (x - CENTER, y - CENTER);
    let radius = dx.hypot(dy);
    let ring = (radius - PORTHOLE).abs() <= HALF_STROKE;
    let full = radius <= PORTHOLE + HALF_STROKE;
    let notch = (x - NOTCH.0).hypot(y - NOTCH.1);
    if status == EngineStatus::NeedsAttention && notch <= NOTCH_RADIUS + NOTCH_GAP {
        return notch <= NOTCH_RADIUS;
    }
    let porthole = match status {
        EngineStatus::Stopped => dashed(dx, dy),
        EngineStatus::Starting => ring || (full && dx <= 0.),
        EngineStatus::Running | EngineStatus::NeedsAttention => full,
    };
    porthole || on_bezel(dx, dy)
}

/// Whether the point, relative to the center, is on the bezel's stroke.
fn on_bezel(dx: f32, dy: f32) -> bool {
    let inner = BEZEL_HALF - BEZEL_RADIUS;
    let (qx, qy) = (dx.abs() - inner, dy.abs() - inner);
    let outside = qx.max(0.).hypot(qy.max(0.));
    let distance = outside + qx.max(qy).min(0.) - BEZEL_RADIUS;
    distance.abs() <= HALF_STROKE
}

/// Whether the point is on one of the porthole's dashes, which have round ends.
fn dashed(dx: f32, dy: f32) -> bool {
    let period = TAU / DASHES;
    let half_dash = period * DASH_ON / 2.;
    // The dashes start at the top and are centered on multiples of `period`.
    let angle = dx.atan2(-dy);
    let from_center = angle - (angle / period).round() * period;
    if from_center.abs() <= half_dash {
        return (dx.hypot(dy) - PORTHOLE).abs() <= HALF_STROKE;
    }
    // Past the end of a dash: the round cap around its end point.
    let end = (angle / period).round() * period + half_dash * from_center.signum();
    let (ex, ey) = (PORTHOLE * end.sin(), -PORTHOLE * end.cos());
    (dx - ex).hypot(dy - ey) <= HALF_STROKE
}

#[cfg(test)]
mod tests;

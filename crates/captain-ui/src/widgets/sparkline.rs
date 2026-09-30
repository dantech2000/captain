use std::collections::HashMap;
use std::time::{Duration, Instant};

use captain_core::store::HISTORY_LEN;
use gpui_kit::*;

/// How a sparkline maps values to height.
#[derive(Debug, Clone, Copy)]
pub enum Scale {
    /// From zero up to the largest value, but never less than `floor`. Keeps near-zero
    /// noise, such as 0.02% CPU, as a flat line instead of full-height spikes.
    FromZero { floor: f64 },
    /// From the smallest to the largest value. Shows small changes in a large value,
    /// such as memory.
    Range,
}

/// How much each new sample moves the drawn line, from 0 (never) to 1 (all the
/// way). Stats come once a second and CPU use comes in bursts, so the raw series
/// zigzags; easing it draws the trend.
const EASE: f64 = 0.4;

/// The engine sends stats about once a second. A live chart scrolls one step over
/// this time, so it moves at a steady speed between samples.
const SAMPLE_EVERY: Duration = Duration::from_secs(1);
/// A moving chart redraws each time its line has moved this far. Anti-aliasing
/// makes steps this small look continuous, and a small chart that scrolls two
/// pixels a second then needs only four frames a second. A redraw repaints the
/// whole window, and every display frame cost a quarter of a CPU core.
const PIXELS_PER_FRAME: f32 = 0.5;
/// The shortest and longest times between frames of a moving chart.
const FASTEST_FRAME: Duration = Duration::from_millis(33);
const SLOWEST_FRAME: Duration = Duration::from_millis(250);

/// With no sample for this long, the stream has paused: the chart stops moving and
/// stops asking for frames.
const STILL_AFTER: Duration = Duration::from_millis(2500);

/// A small line chart of `values`, eased and drawn as a smooth curve. With `fill`,
/// the area under the line is filled too. The caller sets the size.
///
/// `last_at` is when the newest value arrived. With it, the chart is live: the
/// points keep the spacing of a full history from the right edge, and the line
/// slides left on every frame until the next value, so it scrolls instead of
/// jumping once a second. It stops when samples stop or the system asks for
/// reduced motion.
pub fn sparkline(
    values: Vec<f64>,
    scale_by: Scale,
    line: Hsla,
    fill: Option<Hsla>,
    last_at: Option<Instant>,
) -> Canvas<()> {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, cx| {
            let elapsed = last_at.map(|at| at.elapsed());
            let moving = values.len() >= 2
                && !cx.reduce_motion()
                && elapsed.is_some_and(|elapsed| elapsed < STILL_AFTER);
            let phase = match elapsed {
                Some(elapsed) if moving => {
                    (elapsed.as_secs_f32() / SAMPLE_EVERY.as_secs_f32()).min(1.)
                }
                _ => 0.,
            };
            let slots = last_at.map(|_| HISTORY_LEN.max(values.len()));
            let points = shift(
                scale(&ease(&values), scale_by, bounds, slots),
                phase,
                bounds,
            );
            if moving && window.is_window_active() && points.len() >= 2 {
                let step = (points[1].x - points[0].x).as_f32().max(0.01);
                let frame = SAMPLE_EVERY.mul_f32(PIXELS_PER_FRAME / step);
                redraw_after(frame.clamp(FASTEST_FRAME, SLOWEST_FRAME), window, cx);
            }
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                let mut stroke = PathBuilder::stroke(px(1.5));
                stroke.move_to(points[0]);
                curve_through(&mut stroke, &points);
                if let Some(fill) = fill {
                    let mut area = PathBuilder::fill();
                    area.move_to(point(points[0].x, bounds.bottom()));
                    area.line_to(points[0]);
                    curve_through(&mut area, &points);
                    area.line_to(point(points[points.len() - 1].x, bounds.bottom()));
                    area.close();
                    if let Ok(path) = area.build() {
                        window.paint_path(path, fill);
                    }
                }
                if let Ok(path) = stroke.build() {
                    window.paint_path(path, line);
                }
            });
        },
    )
}

/// Slides live points left by `phase` of a step and holds the newest value out
/// to the right edge, so the chart stays full while it waits for the next sample.
fn shift(points: Vec<Point<Pixels>>, phase: f32, bounds: Bounds<Pixels>) -> Vec<Point<Pixels>> {
    if phase <= 0. || points.len() < 2 {
        return points;
    }
    let step = points[1].x - points[0].x;
    let mut shifted: Vec<_> = points
        .iter()
        .map(|p| point(p.x - step * phase, p.y))
        .collect();
    let last = shifted[shifted.len() - 1];
    shifted.push(point(bounds.right(), last.y));
    shifted
}

/// An exponential moving average of `values`, so a one-second burst shows as a
/// small rise and fall instead of a full-height spike.
fn ease(values: &[f64]) -> Vec<f64> {
    let mut eased = Vec::with_capacity(values.len());
    let mut last = None;
    for &value in values {
        let next = last.map_or(value, |last: f64| last + (value - last) * EASE);
        eased.push(next);
        last = Some(next);
    }
    eased
}

/// Continues `path` from the first point through the rest with quadratic curves
/// between the midpoints, which rounds the corners without overshooting.
fn curve_through(path: &mut PathBuilder, points: &[Point<Pixels>]) {
    for pair in points.windows(2).skip(1) {
        let mid = point((pair[0].x + pair[1].x) / 2., (pair[0].y + pair[1].y) / 2.);
        path.curve_to(mid, pair[0]);
    }
    if let Some(last) = points.last() {
        path.line_to(*last);
    }
}

/// Maps values onto the bounds. Fewer than two values draw a flat line at the bottom.
fn scale(
    values: &[f64],
    scale_by: Scale,
    bounds: Bounds<Pixels>,
    slots: Option<usize>,
) -> Vec<Point<Pixels>> {
    let inset = px(1.5);
    let top = bounds.top() + inset;
    let bottom = bounds.bottom() - inset;
    if values.len() < 2 {
        return vec![point(bounds.left(), bottom), point(bounds.right(), bottom)];
    }
    let max = values.iter().copied().fold(f64::MIN, f64::max);
    let min = values.iter().copied().fold(f64::MAX, f64::min);
    let (low, high) = match scale_by {
        Scale::FromZero { floor } => (0.0, (max * 1.15).max(floor)),
        // Pad the range so the line never touches the edges. A flat series sits in the middle.
        Scale::Range if (max - min).abs() < f64::EPSILON => (min - 1.0, max + 1.0),
        Scale::Range => {
            let pad = (max - min) * 0.25;
            (min - pad, max + pad)
        }
    };
    // A live chart spaces its points for a full history and ends at the right
    // edge; a still one stretches its points over the width.
    let slots = slots.unwrap_or(values.len());
    let step = bounds.size.width / (slots - 1) as f32;
    let first = bounds.right() - step * (values.len() - 1) as f32;
    values
        .iter()
        .enumerate()
        .map(|(i, value)| {
            let x = first + step * i as f32;
            let y = bottom - (bottom - top) * ((value - low) / (high - low)) as f32;
            point(x, y)
        })
        .collect()
}

/// When each window's next chart redraw is due, so the charts in one window share
/// frames.
#[derive(Default)]
struct PendingFrames(HashMap<WindowId, Instant>);

impl Global for PendingFrames {}

/// Redraws the window after `delay`, unless a redraw is already due by then.
fn redraw_after(delay: Duration, window: &mut Window, cx: &mut App) {
    let id = window.window_handle().window_id();
    let due = Instant::now() + delay;
    let pending = &mut cx.default_global::<PendingFrames>().0;
    if pending.get(&id).is_some_and(|at| *at <= due) {
        return;
    }
    pending.insert(id, due);
    window
        .spawn(cx, async move |cx| {
            cx.background_executor().timer(delay).await;
            cx.update(|window, cx| {
                let pending = &mut cx.default_global::<PendingFrames>().0;
                if pending.get(&id) == Some(&due) {
                    pending.remove(&id);
                }
                window.refresh();
            })
            .ok();
        })
        .detach();
}

#[cfg(test)]
mod tests;

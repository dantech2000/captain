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

/// A small line chart of `values`, eased and drawn as a smooth curve. With `fill`,
/// the area under the line is filled too. The caller sets the size.
pub fn sparkline(values: Vec<f64>, scale_by: Scale, line: Hsla, fill: Option<Hsla>) -> Canvas<()> {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let points = scale(&ease(&values), scale_by, bounds);
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
        },
    )
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
fn scale(values: &[f64], scale_by: Scale, bounds: Bounds<Pixels>) -> Vec<Point<Pixels>> {
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
    let step = bounds.size.width / (values.len() - 1) as f32;
    values
        .iter()
        .enumerate()
        .map(|(i, value)| {
            let x = bounds.left() + step * i as f32;
            let y = bottom - (bottom - top) * ((value - low) / (high - low)) as f32;
            point(x, y)
        })
        .collect()
}

#[cfg(test)]
mod tests;

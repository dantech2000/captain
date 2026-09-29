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

/// A small line chart of `values`. With `fill`, the area under the line is filled too.
/// The caller sets the size.
pub fn sparkline(values: Vec<f64>, scale_by: Scale, line: Hsla, fill: Option<Hsla>) -> Canvas<()> {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let points = scale(&values, scale_by, bounds);
            let mut stroke = PathBuilder::stroke(px(1.5));
            stroke.move_to(points[0]);
            for point in &points[1..] {
                stroke.line_to(*point);
            }
            if let Some(fill) = fill {
                let mut area = PathBuilder::fill();
                area.move_to(point(points[0].x, bounds.bottom()));
                for point in &points {
                    area.line_to(*point);
                }
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

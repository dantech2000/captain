use captain_core::project_map::{Edge, EdgeKind};
use gpui_kit::*;

use super::scale::Scale;
use crate::theme::Palette;

/// The distance between the dots of the background grid, in map units.
const GRID: f32 = 24.;

/// The dotted grid and every edge, under the nodes. The canvas fills its parent.
pub fn edges(edges: Vec<Edge>, z: Scale, palette: &Palette) -> Canvas<()> {
    let dot = palette.border_strong;
    let colors = [
        (EdgeKind::Port, palette.accent.alpha(0.7)),
        (EdgeKind::TalksTo, palette.text2.alpha(0.55)),
        (EdgeKind::Mount, palette.info.alpha(0.7)),
    ];
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let origin = bounds.origin;
            let at = |(x, y): (f32, f32)| origin + point(z.px(x), z.px(y));
            let mut dots = PathBuilder::fill();
            let step = z.px(GRID);
            let size = px(1.5);
            let mut y = step;
            while y < bounds.size.height {
                let mut x = step;
                while x < bounds.size.width {
                    let corner = origin + point(x, y);
                    dots.add_polygon(
                        &[
                            corner,
                            corner + point(size, px(0.)),
                            corner + point(size, size),
                            corner + point(px(0.), size),
                        ],
                        true,
                    );
                    x += step;
                }
                y += step;
            }
            if let Ok(path) = dots.build() {
                window.paint_path(path, dot);
            }
            for (kind, color) in colors {
                for edge in edges.iter().filter(|e| e.kind == kind) {
                    let mut line = PathBuilder::stroke(px(1.5));
                    if kind == EdgeKind::Mount {
                        line = line.dash_array(&[z.px(5.), z.px(5.)]);
                    }
                    line.move_to(at(edge.from));
                    line.cubic_bezier_to(at(edge.to), at(edge.c1), at(edge.c2));
                    if let Ok(path) = line.build() {
                        window.paint_path(path, color);
                    }
                }
            }
        },
    )
}

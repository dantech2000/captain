use captain_terminal::{GridPoint, Side};
use gpui_kit::{Bounds, point, px, size};

use super::GridMetrics;

fn metrics() -> GridMetrics {
    let bounds = Bounds::new(point(px(10.), px(20.)), size(px(75.), px(50.)));
    GridMetrics::fit(bounds, size(px(7.), px(16.)))
}

#[test]
fn fits_whole_cells() {
    let metrics = metrics();
    assert_eq!((metrics.cols, metrics.rows), (10, 3));
    let tiny = GridMetrics::fit(Bounds::default(), size(px(7.), px(16.)));
    assert_eq!((tiny.cols, tiny.rows), (2, 1));
}

#[test]
fn finds_the_cell_and_side_under_the_pointer() {
    let metrics = metrics();
    assert_eq!(
        metrics.point_at(point(px(10.), px(20.))),
        GridPoint::new(0, 0, Side::Left)
    );
    assert_eq!(
        metrics.point_at(point(px(10. + 7. * 3. + 5.), px(20. + 17.))),
        GridPoint::new(1, 3, Side::Right)
    );
    // Outside the grid clamps to the nearest cell.
    assert_eq!(
        metrics.point_at(point(px(500.), px(500.))),
        GridPoint::new(2, 9, Side::Right)
    );
    assert_eq!(
        metrics.point_at(point(px(0.), px(0.))),
        GridPoint::new(0, 0, Side::Left)
    );
}

#[test]
fn spans_cover_whole_cells() {
    let span = metrics().span(1, 2, 3);
    assert_eq!(span.origin, point(px(24.), px(36.)));
    assert_eq!(span.size, size(px(21.), px(16.)));
}

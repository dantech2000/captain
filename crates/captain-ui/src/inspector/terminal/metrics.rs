use captain_terminal::{GridPoint, Side};
use gpui_kit::*;

/// Where the grid sits in the window and how big its cells are. The grid element
/// measures it on each frame; mouse handlers use it to find the cell under the pointer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridMetrics {
    pub origin: Point<Pixels>,
    pub cell: Size<Pixels>,
    pub cols: u16,
    pub rows: u16,
}

impl GridMetrics {
    /// As many whole cells as fit in `bounds`, at least 2 columns by 1 row.
    pub fn fit(bounds: Bounds<Pixels>, cell: Size<Pixels>) -> Self {
        let count = |total: Pixels, one: Pixels, min: u16| {
            let fits = (f32::from(total) / f32::from(one).max(1.)).floor();
            (fits.clamp(0., f32::from(u16::MAX)) as u16).max(min)
        };
        Self {
            origin: bounds.origin,
            cell,
            cols: count(bounds.size.width, cell.width, 2),
            rows: count(bounds.size.height, cell.height, 1),
        }
    }

    /// The cell under `position`, clamped to the grid.
    pub fn point_at(&self, position: Point<Pixels>) -> GridPoint {
        let x = f32::from(position.x - self.origin.x).max(0.);
        let y = f32::from(position.y - self.origin.y).max(0.);
        let width = f32::from(self.cell.width).max(1.);
        let height = f32::from(self.cell.height).max(1.);
        let col = ((x / width) as usize).min(usize::from(self.cols.saturating_sub(1)));
        let row = ((y / height) as usize).min(usize::from(self.rows.saturating_sub(1)));
        let side = if x - col as f32 * width < width / 2. {
            Side::Left
        } else {
            Side::Right
        };
        GridPoint::new(row, col, side)
    }

    /// The top-left corner of a cell.
    pub fn cell_origin(&self, row: usize, col: usize) -> Point<Pixels> {
        point(
            self.origin.x + self.cell.width * col as f32,
            self.origin.y + self.cell.height * row as f32,
        )
    }

    /// The bounds of `len` cells starting at a cell.
    pub fn span(&self, row: usize, col: usize, len: usize) -> Bounds<Pixels> {
        Bounds::new(
            self.cell_origin(row, col),
            size(self.cell.width * len as f32, self.cell.height),
        )
    }
}

#[cfg(test)]
mod tests;

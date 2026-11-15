//! Page geometry of a contact sheet (ADR-024): a header, a grid of cells, a footer. All
//! lengths are PDF points (1/72 inch); the origin is the page's lower-left corner.

/// Paper size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PageSize {
    A4,
    Letter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Orientation {
    Portrait,
    Landscape,
}

/// What the user configures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Layout {
    pub page_size: PageSize,
    pub orientation: Orientation,
    pub columns: u8,
    pub rows: u8,
    pub show_filename: bool,
    pub show_timestamp: bool,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            page_size: PageSize::A4,
            orientation: Orientation::Portrait,
            columns: 4,
            rows: 6,
            show_filename: true,
            show_timestamp: true,
        }
    }
}

/// Bounds of the grid, per side.
pub const MIN_CELLS: u8 = 1;
pub const MAX_CELLS: u8 = 12;

impl Layout {
    /// The same layout with the grid clamped to `[MIN_CELLS, MAX_CELLS]`.
    pub fn clamped(self) -> Self {
        Self {
            columns: self.columns.clamp(MIN_CELLS, MAX_CELLS),
            rows: self.rows.clamp(MIN_CELLS, MAX_CELLS),
            ..self
        }
    }

    pub fn per_page(&self) -> usize {
        let l = self.clamped();
        usize::from(l.columns) * usize::from(l.rows)
    }

    /// Pages needed for `count` photographs (one page even when there are none, so an
    /// empty sheet still says so).
    pub fn pages(&self, count: usize) -> usize {
        count.div_ceil(self.per_page()).max(1)
    }

    fn caption_lines(&self) -> usize {
        usize::from(self.show_filename) + usize::from(self.show_timestamp)
    }
}

/// An axis-aligned rectangle in points.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub const MARGIN: f32 = 28.0;
pub const GUTTER: f32 = 6.0;
pub const TITLE_SIZE: f32 = 12.0;
pub const SMALL_SIZE: f32 = 8.0;
const HEADER_H: f32 = 32.0;
const FOOTER_H: f32 = 16.0;

/// The fixed geometry of every page of a sheet.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct Geometry {
    pub page_w: f32,
    pub page_h: f32,
    pub columns: usize,
    pub rows: usize,
    pub cell_w: f32,
    pub cell_h: f32,
    /// The image box's height in a cell (its width is `cell_w`).
    pub image_h: f32,
    pub caption_size: f32,
    pub caption_lines: usize,
    /// Baselines of the title and the source line, and of the footer.
    pub title_y: f32,
    pub source_y: f32,
    pub footer_y: f32,
}

impl Geometry {
    pub fn new(layout: &Layout) -> Self {
        let l = layout.clamped();
        let (short, long) = match l.page_size {
            PageSize::A4 => (595.28, 841.89),
            PageSize::Letter => (612.0, 792.0),
        };
        let (page_w, page_h) = match l.orientation {
            Orientation::Portrait => (short, long),
            Orientation::Landscape => (long, short),
        };
        let (columns, rows) = (usize::from(l.columns), usize::from(l.rows));
        let grid_w = page_w - 2.0 * MARGIN;
        let grid_h = page_h - 2.0 * MARGIN - HEADER_H - FOOTER_H;
        let cell_w = (grid_w - GUTTER * (columns as f32 - 1.0)) / columns as f32;
        let cell_h = (grid_h - GUTTER * (rows as f32 - 1.0)) / rows as f32;
        let caption_size = (cell_w / 22.0).clamp(5.0, 8.0);
        let caption_lines = l.caption_lines();
        let captions_h =
            if caption_lines == 0 { 0.0 } else { caption_lines as f32 * caption_size * 1.25 + 2.0 };
        Self {
            page_w,
            page_h,
            columns,
            rows,
            cell_w,
            cell_h,
            image_h: (cell_h - captions_h).max(1.0),
            caption_size,
            caption_lines,
            title_y: page_h - MARGIN - TITLE_SIZE,
            source_y: page_h - MARGIN - TITLE_SIZE - 4.0 - SMALL_SIZE * 1.2,
            footer_y: MARGIN,
        }
    }

    /// The cell at `index` on a page (row-major, from the top left).
    pub fn cell(&self, index: usize) -> Rect {
        let (col, row) = (index % self.columns, index / self.columns);
        let top = self.page_h - MARGIN - HEADER_H;
        Rect {
            x: MARGIN + col as f32 * (self.cell_w + GUTTER),
            y: top - (row as f32 + 1.0) * self.cell_h - row as f32 * GUTTER,
            w: self.cell_w,
            h: self.cell_h,
        }
    }

    /// The image box of a cell: its upper part, above the captions.
    pub fn image_box(&self, cell: Rect) -> Rect {
        Rect { x: cell.x, y: cell.y + cell.h - self.image_h, w: cell.w, h: self.image_h }
    }

    /// Baseline of caption line `line` (0 = the first under the image).
    pub fn caption_baseline(&self, cell: Rect, line: usize) -> f32 {
        cell.y + cell.h - self.image_h - 1.0 - (line as f32 + 1.0) * self.caption_size * 1.25
            + self.caption_size * 0.25
    }

    /// The cached thumbnail size (ADR-020) that is sharp enough for the image box at about
    /// 144 dpi.
    pub fn thumbnail_size(&self) -> u32 {
        let needed = self.cell_w.max(self.image_h) * 2.0;
        if needed <= 256.0 { 256 } else { 1024 }
    }
}

/// `w × h` pixels fitted inside `b`, centred, aspect ratio kept.
pub fn fit(w: u32, h: u32, b: Rect) -> Rect {
    if w == 0 || h == 0 {
        return b;
    }
    let scale = (b.w / w as f32).min(b.h / h as f32);
    let (fw, fh) = (w as f32 * scale, h as f32 * scale);
    Rect { x: b.x + (b.w - fw) / 2.0, y: b.y + (b.h - fh) / 2.0, w: fw, h: fh }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_fits_inside_the_margins_without_overlap() {
        for (cols, rows) in [(1, 1), (4, 6), (12, 12), (3, 2)] {
            for orientation in [Orientation::Portrait, Orientation::Landscape] {
                let l = Layout { columns: cols, rows, orientation, ..Layout::default() };
                let g = Geometry::new(&l);
                let n = l.per_page();
                for i in 0..n {
                    let c = g.cell(i);
                    assert!(c.x >= MARGIN - 0.01 && c.x + c.w <= g.page_w - MARGIN + 0.01);
                    assert!(c.y >= MARGIN + FOOTER_H - 0.01, "{cols}x{rows} cell {i}: {c:?}");
                    assert!(c.y + c.h <= g.page_h - MARGIN - HEADER_H + 0.01);
                    if i + 1 < n && (i + 1) % g.columns != 0 {
                        assert!(g.cell(i + 1).x >= c.x + c.w);
                    }
                    if i + g.columns < n {
                        assert!(g.cell(i + g.columns).y + c.h <= c.y);
                    }
                }
            }
        }
    }

    #[test]
    fn grid_is_clamped_and_pages_round_up() {
        let l = Layout { columns: 0, rows: 40, ..Layout::default() };
        assert_eq!(l.per_page(), 12);
        let l = Layout { columns: 4, rows: 6, ..Layout::default() };
        assert_eq!(l.pages(0), 1);
        assert_eq!(l.pages(24), 1);
        assert_eq!(l.pages(25), 2);
        assert_eq!(l.pages(100_000), 4167);
    }

    #[test]
    fn captions_take_room_from_the_image() {
        let both = Geometry::new(&Layout::default());
        let none = Geometry::new(&Layout {
            show_filename: false,
            show_timestamp: false,
            ..Layout::default()
        });
        assert_eq!(none.image_h, none.cell_h);
        assert!(both.image_h < both.cell_h);
        let cell = both.cell(0);
        let b = both.image_box(cell);
        assert!(both.caption_baseline(cell, 1) < both.caption_baseline(cell, 0));
        assert!(both.caption_baseline(cell, 0) < b.y);
        assert!(both.caption_baseline(cell, 1) >= cell.y);
    }

    #[test]
    fn fit_keeps_the_aspect_ratio_and_centres() {
        let b = Rect { x: 10.0, y: 20.0, w: 100.0, h: 50.0 };
        let r = fit(400, 400, b);
        assert_eq!((r.w, r.h), (50.0, 50.0));
        assert_eq!((r.x, r.y), (35.0, 20.0));
        let r = fit(1000, 100, b);
        assert_eq!((r.w, r.h), (100.0, 10.0));
        assert_eq!(fit(0, 5, b), b);
    }

    #[test]
    fn small_cells_use_the_small_thumbnail() {
        let small = Geometry::new(&Layout { columns: 12, rows: 12, ..Layout::default() });
        let big = Geometry::new(&Layout { columns: 2, rows: 2, ..Layout::default() });
        assert_eq!(small.thumbnail_size(), 256);
        assert_eq!(big.thumbnail_size(), 1024);
    }
}

//! A contact sheet written as a PDF one object at a time (ADR-024): each object goes to the
//! output as soon as it is complete and only its offset is kept, so memory stays bounded
//! by one page. No creation date or random id: the same input gives the same bytes.

use std::io::{self, Write};

use pdf_writer::{Chunk, Content, Filter, Finish, Name, Rect as PdfRect, Ref, Str};

use super::layout::{Geometry, Layout, Rect, SMALL_SIZE, TITLE_SIZE, fit};
use super::text;

/// Texts on every page, already translated by the caller.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct Labels {
    pub title: String,
    /// What the sheet shows (e.g. "Flagged photographs · 1 204 photographs").
    pub source: String,
    /// The page footer, with `{page}` and `{pages}` placeholders.
    pub page: String,
    /// The footer's other note (e.g. "Times in UTC · Timelapse Studio").
    pub note: String,
    /// Shown in the cell of a photograph that could not be read.
    pub unreadable: String,
}

/// One cell's content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub filename: String,
    pub timestamp: Option<String>,
    /// The thumbnail's JPEG bytes, `None` when the photograph could not be read.
    pub jpeg: Option<Vec<u8>>,
}

/// Dimensions and colour components of a baseline or progressive JPEG.
pub fn jpeg_info(bytes: &[u8]) -> Option<(u32, u32, u8)> {
    if bytes.get(..2)? != [0xff, 0xd8] {
        return None;
    }
    let mut i = 2;
    while i + 4 <= bytes.len() {
        if bytes[i] != 0xff {
            return None;
        }
        let marker = bytes[i + 1];
        if marker == 0xff {
            i += 1;
            continue;
        }
        let len = usize::from(u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]));
        // SOF0–SOF15 except DHT (C4), JPG (C8) and DAC (CC). Only 8-bit baseline,
        // extended and progressive Huffman frames (C0–C2) are what PDF readers decode.
        if (0xc0..=0xcf).contains(&marker) && !matches!(marker, 0xc4 | 0xc8 | 0xcc) {
            let s = bytes.get(i + 4..i + 10)?;
            if marker > 0xc2 || s[0] != 8 {
                return None;
            }
            let h = u32::from(u16::from_be_bytes([s[1], s[2]]));
            let w = u32::from(u16::from_be_bytes([s[3], s[4]]));
            return (w > 0 && h > 0).then_some((w, h, s[5]));
        }
        i += 2 + len;
    }
    None
}

/// The frame header of a JPEG this writer embeds (greyscale or RGB); anything else gets
/// a placeholder cell.
pub fn usable(jpeg: &[u8]) -> Option<(u32, u32, u8)> {
    jpeg_info(jpeg).filter(|i| matches!(i.2, 1 | 3))
}

const FONT: Name<'static> = Name(b"F1");
const BOLD: Name<'static> = Name(b"F2");

/// Writes a sheet page by page.
pub struct SheetWriter<W: Write> {
    out: W,
    pos: u64,
    /// Offsets of objects 1, 2, … (index = id - 1); `None` until written.
    offsets: Vec<Option<u64>>,
    geometry: Geometry,
    layout: Layout,
    labels: Labels,
    pages_id: Ref,
    font: Ref,
    bold: Ref,
    page_ids: Vec<Ref>,
    total_pages: usize,
}

impl<W: Write> SheetWriter<W> {
    pub fn new(
        mut out: W,
        layout: &Layout,
        labels: &Labels,
        total_pages: usize,
    ) -> io::Result<Self> {
        let header = b"%PDF-1.7\n%\x80\x80\x80\x80\n\n";
        out.write_all(header)?;
        let mut w = Self {
            out,
            pos: header.len() as u64,
            offsets: Vec::new(),
            geometry: Geometry::new(layout),
            layout: layout.clamped(),
            labels: labels.clone(),
            pages_id: Ref::new(1),
            font: Ref::new(1),
            bold: Ref::new(1),
            page_ids: Vec::with_capacity(total_pages),
            total_pages,
        };
        w.pages_id = w.alloc();
        w.font = w.alloc();
        w.bold = w.alloc();
        for (id, base) in [(w.font, b"Helvetica".as_slice()), (w.bold, b"Helvetica-Bold")] {
            let mut c = Chunk::new();
            c.type1_font(id).base_font(Name(base)).encoding_predefined(Name(b"WinAnsiEncoding"));
            w.emit(id, &c)?;
        }
        Ok(w)
    }

    fn alloc(&mut self) -> Ref {
        self.offsets.push(None);
        Ref::new(self.offsets.len() as i32)
    }

    /// Writes a chunk that holds exactly the object `id`.
    fn emit(&mut self, id: Ref, chunk: &Chunk) -> io::Result<()> {
        self.offsets[id.get() as usize - 1] = Some(self.pos);
        self.out.write_all(chunk.as_bytes())?;
        self.pos += chunk.as_bytes().len() as u64;
        Ok(())
    }

    /// Writes one page with `cells` (at most one page's worth).
    pub fn page(&mut self, cells: &[Cell]) -> io::Result<()> {
        let g = self.geometry;
        let page_no = self.page_ids.len() + 1;
        let mut content = Content::new();
        let image_names: Vec<Vec<u8>> =
            (0..cells.len()).map(|i| format!("Im{i}").into_bytes()).collect();
        let mut images: Vec<(Name<'_>, Ref)> = Vec::new();

        self.header_and_footer(&mut content, page_no);

        for (i, cell) in cells.iter().take(self.layout.per_page()).enumerate() {
            let rect = g.cell(i);
            let b = g.image_box(rect);
            let info = cell.jpeg.as_deref().and_then(usable);
            match (cell.jpeg.as_deref(), info) {
                (Some(jpeg), Some((w, h, comps))) => {
                    let id = self.alloc();
                    let mut c = Chunk::new();
                    let mut img = c.image_xobject(id, jpeg);
                    img.filter(Filter::DctDecode);
                    img.width(w as i32).height(h as i32).bits_per_component(8);
                    img.color_space_name(Name(if comps == 1 {
                        b"DeviceGray"
                    } else {
                        b"DeviceRGB"
                    }));
                    img.finish();
                    self.emit(id, &c)?;
                    let name = image_names[i].as_slice();
                    images.push((Name(name), id));
                    let r = fit(w, h, b);
                    content.save_state();
                    content.transform([r.w, 0.0, 0.0, r.h, r.x, r.y]);
                    content.x_object(Name(name));
                    content.restore_state();
                }
                _ => self.placeholder(&mut content, b),
            }
            self.captions(&mut content, rect, cell);
        }

        let content_id = self.alloc();
        let mut c = Chunk::new();
        c.stream(content_id, &content.finish().into_vec());
        self.emit(content_id, &c)?;

        let page_id = self.alloc();
        let mut c = Chunk::new();
        let mut page = c.page(page_id);
        page.parent(self.pages_id)
            .media_box(PdfRect::new(0.0, 0.0, g.page_w, g.page_h))
            .contents(content_id);
        let mut res = page.resources();
        res.fonts().pair(FONT, self.font).pair(BOLD, self.bold);
        let mut xo = res.x_objects();
        for (name, id) in &images {
            xo.pair(*name, *id);
        }
        xo.finish();
        res.finish();
        page.finish();
        self.emit(page_id, &c)?;
        self.page_ids.push(page_id);
        Ok(())
    }

    fn header_and_footer(&self, content: &mut Content, page_no: usize) {
        let g = self.geometry;
        let max = g.page_w - 2.0 * super::layout::MARGIN;
        let line = |content: &mut Content, font, size, x: f32, y: f32, text: &[u8]| {
            content.begin_text().set_font(font, size).next_line(x, y).show(Str(text)).end_text();
        };
        content.set_fill_gray(0.0);
        let title = text::fit(&text::encode(&self.labels.title), max, TITLE_SIZE, false);
        line(content, BOLD, TITLE_SIZE, super::layout::MARGIN, g.title_y, &title);
        content.set_fill_gray(0.3);
        let source = text::fit(&text::encode(&self.labels.source), max, SMALL_SIZE, false);
        line(content, FONT, SMALL_SIZE, super::layout::MARGIN, g.source_y, &source);

        let page = self
            .labels
            .page
            .replace("{page}", &page_no.to_string())
            .replace("{pages}", &self.total_pages.to_string());
        let page = text::fit(&text::encode(&page), max / 2.0, SMALL_SIZE, false);
        let page_w = text::width(&page, SMALL_SIZE);
        let note =
            text::fit(&text::encode(&self.labels.note), max - page_w - 12.0, SMALL_SIZE, false);
        line(content, FONT, SMALL_SIZE, super::layout::MARGIN, g.footer_y, &note);
        let x = g.page_w - super::layout::MARGIN - page_w;
        line(content, FONT, SMALL_SIZE, x, g.footer_y, &page);
    }

    fn placeholder(&self, content: &mut Content, b: Rect) {
        let size = self.geometry.caption_size;
        content.save_state();
        content.set_fill_gray(0.9).rect(b.x, b.y, b.w, b.h).fill_nonzero();
        let label = text::fit(&text::encode(&self.labels.unreadable), b.w - 4.0, size, false);
        let w = text::width(&label, size);
        content.set_fill_gray(0.4);
        content
            .begin_text()
            .set_font(FONT, size)
            .next_line(b.x + (b.w - w) / 2.0, b.y + (b.h - size) / 2.0)
            .show(Str(&label))
            .end_text();
        content.restore_state();
    }

    fn captions(&self, content: &mut Content, cell: Rect, c: &Cell) {
        let g = self.geometry;
        let mut lines: Vec<(Vec<u8>, bool)> = Vec::new();
        if self.layout.show_filename {
            lines.push((text::encode(&c.filename), true));
        }
        if self.layout.show_timestamp {
            lines.push((text::encode(c.timestamp.as_deref().unwrap_or("—")), false));
        }
        if lines.is_empty() {
            return;
        }
        content.save_state();
        // Clip to the cell, so nothing can run into a neighbour.
        content.rect(cell.x, cell.y, cell.w, cell.h).clip_nonzero().end_path();
        content.set_fill_gray(0.15);
        for (i, (line, keep_end)) in lines.iter().enumerate() {
            let t = text::fit(line, cell.w, g.caption_size, *keep_end);
            content
                .begin_text()
                .set_font(FONT, g.caption_size)
                .next_line(cell.x, g.caption_baseline(cell, i))
                .show(Str(&t))
                .end_text();
        }
        content.restore_state();
    }

    pub fn pages_written(&self) -> usize {
        self.page_ids.len()
    }

    /// Writes the page tree, catalog, xref and trailer; returns the output.
    pub fn finish(mut self) -> io::Result<W> {
        let mut c = Chunk::new();
        c.pages(self.pages_id)
            .kids(self.page_ids.iter().copied())
            .count(self.page_ids.len() as i32);
        let pages_id = self.pages_id;
        self.emit(pages_id, &c)?;
        let catalog = self.alloc();
        let mut c = Chunk::new();
        c.indirect(catalog).start::<pdf_writer::writers::Catalog>().pages(self.pages_id);
        self.emit(catalog, &c)?;

        let xref = self.pos;
        let mut tail = format!("xref\n0 {}\n0000000000 65535 f \n", self.offsets.len() + 1);
        for o in &self.offsets {
            let o = o.ok_or_else(|| io::Error::other("unwritten PDF object"))?;
            tail.push_str(&format!("{o:010} 00000 n \n"));
        }
        tail.push_str(&format!(
            "trailer\n<< /Size {} /Root {} 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            self.offsets.len() + 1,
            catalog.get()
        ));
        self.out.write_all(tail.as_bytes())?;
        self.out.flush()?;
        Ok(self.out)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Every xref entry points at `N 0 obj`, the trailer names a catalog, and the page
    /// count matches. Returns the number of pages.
    pub(crate) fn check_structure(pdf: &[u8]) -> usize {
        // Offsets are byte offsets: never index a lossy UTF-8 copy with them.
        let s = String::from_utf8_lossy(pdf);
        assert!(pdf.starts_with(b"%PDF-1.7\n"));
        assert!(pdf.ends_with(b"%%EOF\n"));
        let sx = pdf.windows(10).rposition(|w| w == b"startxref\n").unwrap();
        let tail = std::str::from_utf8(&pdf[sx + 10..]).unwrap();
        let xref: usize = tail.lines().next().unwrap().parse().unwrap();
        assert_eq!(&pdf[xref..xref + 5], b"xref\n");
        let table = std::str::from_utf8(&pdf[xref + 5..]).unwrap();
        let mut lines = table.lines();
        let n: usize = lines.next().unwrap().split(' ').nth(1).unwrap().parse().unwrap();
        lines.next();
        for id in 1..n {
            let off: usize = lines.next().unwrap()[..10].parse().unwrap();
            let head = format!("{id} 0 obj");
            assert_eq!(&pdf[off..off + head.len()], head.as_bytes(), "object {id}");
        }
        assert!(s.contains("/Type /Catalog"));
        s.matches("/Type /Page").count() - s.matches("/Type /Pages").count()
    }

    pub(crate) fn tiny_jpeg(w: u32, h: u32) -> Vec<u8> {
        let img = image::RgbImage::from_pixel(w, h, image::Rgb([120, 80, 40]));
        let mut out = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(std::io::Cursor::new(&mut out), 80)
            .encode_image(&img)
            .unwrap();
        out
    }

    fn labels() -> Labels {
        Labels {
            title: "Planche contact — Été".into(),
            source: "Toutes les photographies".into(),
            page: "Page {page} de {pages}".into(),
            note: "Heures en UTC".into(),
            unreadable: "Illisible".into(),
        }
    }

    fn sheet(cells: &[Cell], layout: &Layout) -> Vec<u8> {
        let pages = layout.pages(cells.len());
        let mut w = SheetWriter::new(Vec::new(), layout, &labels(), pages).unwrap();
        for chunk in cells.chunks(layout.per_page()) {
            w.page(chunk).unwrap();
        }
        if cells.is_empty() {
            w.page(&[]).unwrap();
        }
        w.finish().unwrap()
    }

    fn cells(n: usize) -> Vec<Cell> {
        (0..n)
            .map(|i| Cell {
                filename: format!("dir/IMG_{i:04}.JPG"),
                timestamp: Some("2024-05-01 08:31:05 UTC".into()),
                jpeg: (i % 5 != 4).then(|| tiny_jpeg(16 + i as u32, 9)),
            })
            .collect()
    }

    #[test]
    fn jpeg_info_reads_the_frame_header() {
        assert_eq!(jpeg_info(&tiny_jpeg(31, 17)), Some((31, 17, 3)));
        assert_eq!(jpeg_info(b"not a jpeg"), None);
        assert_eq!(jpeg_info(&[0xff, 0xd8, 0xff]), None);
    }

    #[test]
    fn a_sheet_is_a_valid_pdf_with_one_page_per_grid() {
        let layout = Layout { columns: 3, rows: 2, ..Layout::default() };
        let pdf = sheet(&cells(13), &layout);
        assert_eq!(check_structure(&pdf), 3);
        assert_eq!(String::from_utf8_lossy(&pdf).matches("/Subtype /Image").count(), 11);
        // Labels in WinAnsi, the page placeholders filled.
        assert!(pdf.windows(9).any(|w| w == b"Page 3 de"));
        assert!(has_text(&pdf, "Planche contact — Été"));
        assert!(!has_text(&pdf, "Planche contact — Ete"));
    }

    /// `s` appears as shown text: a literal string when it encodes to ASCII, otherwise the
    /// hex string `pdf-writer` writes for it.
    pub(crate) fn has_text(pdf: &[u8], s: &str) -> bool {
        let bytes = text::encode(s);
        let needle: Vec<u8> = if bytes.is_ascii() {
            [b"(", bytes.as_slice(), b")"].concat()
        } else {
            let hex: String = bytes.iter().map(|b| format!("{b:02X}")).collect();
            format!("<{hex}>").into_bytes()
        };
        let lower = needle.to_ascii_lowercase();
        pdf.windows(needle.len()).any(|w| w == needle.as_slice() || w == lower.as_slice())
    }

    #[test]
    fn an_empty_sheet_has_one_page() {
        let pdf = sheet(&[], &Layout::default());
        assert_eq!(check_structure(&pdf), 1);
    }

    #[test]
    fn the_same_input_gives_the_same_bytes() {
        let layout = Layout::default();
        assert_eq!(sheet(&cells(30), &layout), sheet(&cells(30), &layout));
    }

    #[test]
    fn captions_follow_the_layout() {
        let both = sheet(&cells(2), &Layout::default());
        let none = sheet(
            &cells(2),
            &Layout { show_filename: false, show_timestamp: false, ..Layout::default() },
        );
        let has = |pdf: &[u8], s: &[u8]| pdf.windows(s.len()).any(|w| w == s);
        assert!(has(&both, b"IMG_0001.JPG") && has(&both, b"08:31:05"));
        assert!(!has(&none, b"IMG_0001.JPG") && !has(&none, b"08:31:05"));
    }
}

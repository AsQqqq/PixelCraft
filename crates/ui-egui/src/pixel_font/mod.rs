//! PixelCraft's pixel font: the hand-drawn glyphs in [`glyphs`] built into a TrueType font at
//! start-up, so egui can lay out and render it like any other font.
//!
//! One font pixel is [`UNIT`] font units; the em is 10 pixels (8 above the baseline, 2 below), so
//! at [`SIZE`] points one font pixel is exactly [`crate::theme::size::UI_PX`] points and text sits
//! on the same pixel grid as the rest of the interface. Each run of ink in a row becomes one
//! rectangle contour. Characters the font lacks fall back to egui's default fonts.

mod glyphs;

pub use glyphs::GLYPHS;

/// Font units per font pixel.
pub const UNIT: i16 = 100;
/// Font units per em: ten pixels.
pub const UNITS_PER_EM: u16 = 1000;
/// Ascent and descent in font pixels.
const ASCENT_PX: i16 = 8;
const DESCENT_PX: i16 = 2;
/// Rows on the baseline (the rest are the descender).
const BASELINE_ROW: i16 = 7;
/// The point size at which one font pixel is one UI pixel (two points).
pub const SIZE: f32 = 20.0;
pub const FAMILY: &str = "PixelCraft Pixel";

/// A glyph as rectangle contours in font units, plus its advance.
struct Outline {
    rects: Vec<(i16, i16, i16, i16)>,
    advance: u16,
}

fn outline(rows: &[&str]) -> Outline {
    let width = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0) as i16;
    let mut rects = Vec::new();
    for (r, row) in rows.iter().enumerate() {
        // Row r spans from the baseline-relative bottom (BASELINE_ROW - 1 - r) up one pixel.
        let y0 = (BASELINE_ROW - 1 - r as i16) * UNIT;
        let mut run: Option<i16> = None;
        let cells: Vec<char> = row.chars().chain(std::iter::once('.')).collect();
        for (x, c) in cells.iter().enumerate() {
            match (c, run) {
                ('#', None) => run = Some(x as i16),
                ('#', Some(_)) => {}
                (_, Some(start)) => {
                    rects.push((start * UNIT, y0, x as i16 * UNIT, y0 + UNIT));
                    run = None;
                }
                (_, None) => {}
            }
        }
    }
    Outline { rects, advance: ((width + 1) * UNIT) as u16 }
}

// --- binary writing -----------------------------------------------------------------------------

fn u16be(v: &mut Vec<u8>, x: u16) {
    v.extend_from_slice(&x.to_be_bytes());
}
fn i16be(v: &mut Vec<u8>, x: i16) {
    v.extend_from_slice(&x.to_be_bytes());
}
fn u32be(v: &mut Vec<u8>, x: u32) {
    v.extend_from_slice(&x.to_be_bytes());
}

fn checksum(data: &[u8]) -> u32 {
    data.chunks(4).fold(0u32, |sum, c| {
        let mut b = [0u8; 4];
        b[..c.len()].copy_from_slice(c);
        sum.wrapping_add(u32::from_be_bytes(b))
    })
}

/// The glyph table entry for one outline (empty for blank glyphs).
fn glyf_entry(o: &Outline) -> Vec<u8> {
    let mut v = Vec::new();
    if o.rects.is_empty() {
        return v;
    }
    let (x_min, y_min) = (o.rects.iter().map(|r| r.0).min().unwrap_or(0), o.rects.iter().map(|r| r.1).min().unwrap_or(0));
    let (x_max, y_max) = (o.rects.iter().map(|r| r.2).max().unwrap_or(0), o.rects.iter().map(|r| r.3).max().unwrap_or(0));
    i16be(&mut v, o.rects.len() as i16);
    for b in [x_min, y_min, x_max, y_max] {
        i16be(&mut v, b);
    }
    for i in 0..o.rects.len() {
        u16be(&mut v, (i * 4 + 3) as u16);
    }
    u16be(&mut v, 0); // no instructions
    let n = o.rects.len() * 4;
    v.extend(std::iter::repeat_n(0x01u8, n)); // all points on the curve, 16-bit deltas
    // Clockwise in y-up font space: top-left, top-right, bottom-right, bottom-left.
    let points: Vec<(i16, i16)> = o.rects.iter().flat_map(|&(x0, y0, x1, y1)| [(x0, y1), (x1, y1), (x1, y0), (x0, y0)]).collect();
    let mut prev = 0i16;
    for &(x, _) in &points {
        i16be(&mut v, x - prev);
        prev = x;
    }
    prev = 0;
    for &(_, y) in &points {
        i16be(&mut v, y - prev);
        prev = y;
    }
    while v.len() % 4 != 0 {
        v.push(0);
    }
    v
}

/// The whole font as TrueType bytes.
pub fn ttf() -> Vec<u8> {
    // Glyph 0 is .notdef: a hollow box.
    let notdef = Outline { rects: vec![(0, 0, 500, UNIT), (0, 600, 500, 700), (0, UNIT, UNIT, 600), (400, UNIT, 500, 600)], advance: 600 };
    let mut outlines = vec![notdef];
    let mut chars: Vec<(char, u16)> = Vec::new();
    for (i, (c, rows)) in GLYPHS.iter().enumerate() {
        outlines.push(outline(rows));
        chars.push((*c, (i + 1) as u16));
    }
    chars.sort_by_key(|(c, _)| *c as u32);
    let num_glyphs = outlines.len() as u16;

    // glyf + loca (long offsets).
    let mut glyf = Vec::new();
    let mut loca = Vec::new();
    for o in &outlines {
        u32be(&mut loca, glyf.len() as u32);
        glyf.extend(glyf_entry(o));
    }
    u32be(&mut loca, glyf.len() as u32);

    let x_max = outlines.iter().flat_map(|o| o.rects.iter().map(|r| r.2)).max().unwrap_or(0);
    let y_min = -DESCENT_PX * UNIT;
    let y_max = ASCENT_PX * UNIT;
    let adv_max = outlines.iter().map(|o| o.advance).max().unwrap_or(0);

    let mut head = Vec::new();
    u32be(&mut head, 0x0001_0000); // version
    u32be(&mut head, 0x0001_0000); // fontRevision
    u32be(&mut head, 0); // checkSumAdjustment (not needed by the renderers we use)
    u32be(&mut head, 0x5F0F_3CF5);
    u16be(&mut head, 0x000B); // flags: baseline at 0, integer ppem, lsb at 0
    u16be(&mut head, UNITS_PER_EM);
    head.extend([0u8; 16]); // created, modified
    for b in [0, y_min, x_max, y_max] {
        i16be(&mut head, b);
    }
    u16be(&mut head, 0); // macStyle
    u16be(&mut head, 8); // lowestRecPPEM
    i16be(&mut head, 2); // fontDirectionHint
    i16be(&mut head, 1); // indexToLocFormat: long
    i16be(&mut head, 0);

    let mut hhea = Vec::new();
    u32be(&mut hhea, 0x0001_0000);
    i16be(&mut hhea, y_max);
    i16be(&mut hhea, y_min);
    i16be(&mut hhea, 0); // lineGap
    u16be(&mut hhea, adv_max);
    i16be(&mut hhea, 0); // minLeftSideBearing
    i16be(&mut hhea, 0); // minRightSideBearing
    i16be(&mut hhea, x_max);
    i16be(&mut hhea, 1); // caretSlopeRise
    i16be(&mut hhea, 0);
    i16be(&mut hhea, 0);
    hhea.extend([0u8; 8]);
    i16be(&mut hhea, 0); // metricDataFormat
    u16be(&mut hhea, num_glyphs);

    let max_contours = outlines.iter().map(|o| o.rects.len()).max().unwrap_or(0) as u16;
    let mut maxp = Vec::new();
    u32be(&mut maxp, 0x0001_0000);
    u16be(&mut maxp, num_glyphs);
    u16be(&mut maxp, max_contours * 4); // maxPoints
    u16be(&mut maxp, max_contours);
    maxp.extend([0u8; 4]); // composite points/contours
    u16be(&mut maxp, 1); // maxZones
    maxp.extend([0u8; 16]);

    let mut hmtx = Vec::new();
    for o in &outlines {
        u16be(&mut hmtx, o.advance);
        i16be(&mut hmtx, o.rects.iter().map(|r| r.0).min().unwrap_or(0));
    }

    // cmap format 4, one segment per character.
    let seg_count = chars.len() as u16 + 1;
    let mut sub = Vec::new();
    u16be(&mut sub, 4);
    u16be(&mut sub, 16 + 8 * seg_count); // length
    u16be(&mut sub, 0); // language
    u16be(&mut sub, seg_count * 2);
    let mut power = 1u16;
    let mut log = 0u16;
    while power * 2 <= seg_count {
        power *= 2;
        log += 1;
    }
    u16be(&mut sub, power * 2);
    u16be(&mut sub, log);
    u16be(&mut sub, seg_count * 2 - power * 2);
    for (c, _) in &chars {
        u16be(&mut sub, *c as u32 as u16);
    }
    u16be(&mut sub, 0xFFFF);
    u16be(&mut sub, 0); // reservedPad
    for (c, _) in &chars {
        u16be(&mut sub, *c as u32 as u16);
    }
    u16be(&mut sub, 0xFFFF);
    for (c, g) in &chars {
        u16be(&mut sub, g.wrapping_sub(*c as u32 as u16));
    }
    u16be(&mut sub, 1);
    for _ in 0..seg_count {
        u16be(&mut sub, 0);
    }
    let mut cmap = Vec::new();
    u16be(&mut cmap, 0);
    u16be(&mut cmap, 1);
    u16be(&mut cmap, 3); // Windows
    u16be(&mut cmap, 1); // Unicode BMP
    u32be(&mut cmap, 12);
    cmap.extend(sub);

    let mut os2 = Vec::new();
    u16be(&mut os2, 4); // version
    i16be(&mut os2, 500); // xAvgCharWidth
    u16be(&mut os2, 400); // weight
    u16be(&mut os2, 5); // width: medium
    u16be(&mut os2, 0); // fsType: installable
    for _ in 0..10 {
        i16be(&mut os2, 0); // sub/superscript and strikeout metrics
    }
    i16be(&mut os2, 0); // sFamilyClass
    os2.extend([0u8; 10]); // panose
    os2.extend([0u8; 16]); // unicode ranges
    os2.extend(b"ARTC"); // vendor
    u16be(&mut os2, 0x00C0); // fsSelection: regular, use typo metrics
    u16be(&mut os2, chars.first().map_or(32, |c| c.0 as u32 as u16));
    u16be(&mut os2, chars.last().map_or(126, |c| (c.0 as u32).min(0xFFFF) as u16));
    i16be(&mut os2, y_max); // typo ascender
    i16be(&mut os2, y_min); // typo descender
    i16be(&mut os2, 0); // typo line gap
    u16be(&mut os2, y_max as u16); // win ascent
    u16be(&mut os2, (-y_min) as u16); // win descent
    u32be(&mut os2, 1); // code page ranges
    u32be(&mut os2, 0);
    i16be(&mut os2, 5 * UNIT); // x-height
    i16be(&mut os2, 7 * UNIT); // cap height
    u16be(&mut os2, 0); // default char
    u16be(&mut os2, 32); // break char
    u16be(&mut os2, 1); // max context

    // name: family, subfamily, full name, PostScript name (Windows, UTF-16BE).
    let names = [(1u16, FAMILY), (2, "Regular"), (4, FAMILY), (6, "PixelCraftPixel-Regular")];
    let mut strings = Vec::new();
    let mut name = Vec::new();
    u16be(&mut name, 0);
    u16be(&mut name, names.len() as u16);
    u16be(&mut name, 6 + 12 * names.len() as u16);
    for (id, text) in names {
        let utf16: Vec<u8> = text.encode_utf16().flat_map(u16::to_be_bytes).collect();
        u16be(&mut name, 3);
        u16be(&mut name, 1);
        u16be(&mut name, 0x0409);
        u16be(&mut name, id);
        u16be(&mut name, utf16.len() as u16);
        u16be(&mut name, strings.len() as u16);
        strings.extend(utf16);
    }
    name.extend(strings);

    let mut post = Vec::new();
    u32be(&mut post, 0x0003_0000);
    post.extend([0u8; 28]);

    // Table directory, tags in order.
    let mut tables: Vec<(&[u8; 4], Vec<u8>)> = vec![
        (b"OS/2", os2),
        (b"cmap", cmap),
        (b"glyf", glyf),
        (b"head", head),
        (b"hhea", hhea),
        (b"hmtx", hmtx),
        (b"loca", loca),
        (b"maxp", maxp),
        (b"name", name),
        (b"post", post),
    ];
    tables.sort_by(|a, b| a.0.cmp(b.0));
    let n = tables.len() as u16;
    let mut out = Vec::new();
    u32be(&mut out, 0x0001_0000);
    u16be(&mut out, n);
    let mut power = 1u16;
    let mut log = 0u16;
    while power * 2 <= n {
        power *= 2;
        log += 1;
    }
    u16be(&mut out, power * 16);
    u16be(&mut out, log);
    u16be(&mut out, n * 16 - power * 16);
    let mut offset = 12 + 16 * u32::from(n);
    let mut body = Vec::new();
    for (tag, data) in &tables {
        out.extend_from_slice(*tag);
        u32be(&mut out, checksum(data));
        u32be(&mut out, offset);
        u32be(&mut out, data.len() as u32);
        body.extend_from_slice(data);
        while body.len() % 4 != 0 {
            body.push(0);
        }
        offset = 12 + 16 * u32::from(n) + body.len() as u32;
    }
    out.extend(body);
    out
}

/// Installs the pixel font as the first choice for proportional and monospace text, keeping
/// egui's default fonts behind it for characters it doesn't have.
pub fn install(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(FAMILY.to_owned(), std::sync::Arc::new(egui::FontData::from_owned(ttf())));
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts.families.entry(family).or_default().insert(0, FAMILY.to_owned());
    }
    ctx.set_fonts(fonts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_is_well_formed() {
        let mut seen = std::collections::HashSet::new();
        for (c, rows) in GLYPHS {
            assert!(seen.insert(*c), "duplicate glyph {c:?}");
            assert!(!rows.is_empty() && rows.len() <= 9, "{c:?} has {} rows", rows.len());
            let w = rows.first().map_or(0, |r| r.chars().count());
            assert!(rows.iter().all(|r| r.chars().count() == w && r.chars().all(|ch| ch == '#' || ch == '.')), "{c:?} rows differ");
        }
    }

    #[test]
    fn egui_lays_the_font_out_on_the_pixel_grid() {
        let ctx = egui::Context::default();
        install(&ctx);
        let mut out = ctx.run_ui(Default::default(), |_| {});
        out.textures_delta.clear();
        // "Hi": H is 5 px + 1, i is 1 px + 1 → 8 font pixels; at SIZE one font pixel is 2 points.
        let galley = ctx.fonts_mut(|f| f.layout_no_wrap("Hi".into(), egui::FontId::proportional(SIZE), egui::Color32::WHITE));
        assert_eq!(galley.size().x, 16.0);
        // Characters the font lacks still lay out (fallback).
        let galley = ctx.fonts_mut(|f| f.layout_no_wrap("Ж".into(), egui::FontId::proportional(SIZE), egui::Color32::WHITE));
        assert!(galley.size().x > 0.0);
    }
}

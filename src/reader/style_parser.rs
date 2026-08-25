use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::format::{BorderStyle, Format, Pattern, Underline};
use crate::xml::xml_reader::get_attr;

// ── Parsed style component structs ──────────────────────────────────────────

/// Font record parsed from `<font>` elements in styles XML.
#[derive(Debug, Clone)]
pub struct ParsedFont {
    pub bold: bool,
    pub italic: bool,
    pub underline: Underline,
    pub strikethrough: bool,
    pub size: f64,
    pub name: String,
    pub color: Option<[u8; 3]>,
}

impl Default for ParsedFont {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: Underline::None,
            strikethrough: false,
            size: 11.0,
            name: "Calibri".into(),
            color: None,
        }
    }
}

/// Fill record parsed from `<fill>` elements in styles XML.
#[derive(Debug, Clone)]
pub struct ParsedFill {
    pub pattern: Pattern,
    pub fg_color: Option<[u8; 3]>,
    pub bg_color: Option<[u8; 3]>,
}

impl Default for ParsedFill {
    fn default() -> Self {
        Self {
            pattern: Pattern::None,
            fg_color: None,
            bg_color: None,
        }
    }
}

/// Border side parsed from individual `<left>`, `<right>`, `<top>`, `<bottom>` elements.
#[derive(Debug, Clone, Default)]
pub struct ParsedBorderSide {
    pub style: BorderStyle,
    pub color: Option<[u8; 3]>,
}

/// Border record parsed from `<border>` elements in styles XML.
#[derive(Debug, Clone, Default)]
pub struct ParsedBorder {
    pub left: ParsedBorderSide,
    pub right: ParsedBorderSide,
    pub top: ParsedBorderSide,
    pub bottom: ParsedBorderSide,
    pub diagonal: ParsedBorderSide,
}

/// Alignment record parsed from `<alignment>` elements inside `<xf>`.
#[derive(Debug, Clone, Default)]
pub struct ParsedAlignment {
    pub horizontal: u8,
    pub vertical: u8,
    pub wrap_text: bool,
    pub shrink_to_fit: bool,
    pub indent: u8,
    pub rotation: i16,
}

/// Full xf record: references into fonts, fills, borders plus alignment and number format.
#[derive(Debug, Clone, Default)]
pub struct XfRecord {
    pub font_id: usize,
    pub fill_id: usize,
    pub border_id: usize,
    pub num_fmt_id: u16,
    pub alignment: ParsedAlignment,
}

/// Differential formatting record parsed from `<dxf>` elements (used by conditional formatting).
#[derive(Debug, Clone, Default)]
pub struct DxfRecord {
    pub font: Option<ParsedFont>,
    pub fill: Option<ParsedFill>,
    pub border: Option<ParsedBorder>,
    pub num_fmt: Option<(u16, String)>,
}

// ── ParsedStyles ────────────────────────────────────────────────────────────

pub struct ParsedStyles {
    pub num_formats: Vec<(u16, String)>,
    pub xf_num_fmt_ids: Vec<u16>,
    pub fonts: Vec<ParsedFont>,
    pub fills: Vec<ParsedFill>,
    pub borders: Vec<ParsedBorder>,
    pub xf_records: Vec<XfRecord>,
    pub dxf_records: Vec<DxfRecord>,
}

impl Default for ParsedStyles {
    fn default() -> Self {
        Self {
            num_formats: Vec::new(),
            xf_num_fmt_ids: vec![0],
            fonts: Vec::new(),
            fills: Vec::new(),
            borders: Vec::new(),
            xf_records: Vec::new(),
            dxf_records: Vec::new(),
        }
    }
}

impl ParsedStyles {
    /// Resolve an xf index into a fully populated `Format` struct.
    ///
    /// Returns `None` if `xf_index` is out of bounds.
    pub fn resolve_format(&self, xf_index: usize) -> Option<Format> {
        let xf = self.xf_records.get(xf_index)?;

        let mut fmt = Format::new();

        // ── Font ──
        if let Some(font) = self.fonts.get(xf.font_id) {
            fmt.bold = font.bold;
            fmt.italic = font.italic;
            fmt.underline = font.underline;
            fmt.strikethrough = font.strikethrough;
            fmt.font_size = font.size;
            fmt.font_name = font.name.clone();
            fmt.font_color = font.color;
        }

        // ── Fill ──
        if let Some(fill) = self.fills.get(xf.fill_id) {
            fmt.fg_color = fill.fg_color;
            fmt.bg_color = fill.bg_color;
            fmt.pattern = fill.pattern;
        }

        // ── Border ──
        if let Some(border) = self.borders.get(xf.border_id) {
            fmt.border_left = border.left.style;
            fmt.border_right = border.right.style;
            fmt.border_top = border.top.style;
            fmt.border_bottom = border.bottom.style;
            fmt.border_left_color = border.left.color;
            fmt.border_right_color = border.right.color;
            fmt.border_top_color = border.top.color;
            fmt.border_bottom_color = border.bottom.color;
            fmt.diagonal_border = border.diagonal.style;
        }

        // ── Alignment ──
        fmt.h_align = xf.alignment.horizontal;
        fmt.v_align = xf.alignment.vertical;
        fmt.wrap_text = xf.alignment.wrap_text;
        fmt.shrink = xf.alignment.shrink_to_fit;
        fmt.indent = xf.alignment.indent;
        fmt.rotation = xf.alignment.rotation;

        // ── Number format ──
        if xf.num_fmt_id != 0 {
            // Look up custom number formats first
            if let Some((_, code)) = self.num_formats.iter().find(|(id, _)| *id == xf.num_fmt_id) {
                fmt.num_format = code.clone();
            } else {
                // Fall back to built-in number format codes
                if let Some(builtin) = builtin_num_format(xf.num_fmt_id) {
                    fmt.num_format = builtin.to_string();
                }
            }
        }

        Some(fmt)
    }
}

/// Map built-in number format IDs to their format code strings.
/// See ECMA-376 Part 1, §18.8.30 (numFmt) for the full list.
fn builtin_num_format(id: u16) -> Option<&'static str> {
    match id {
        0 => Some("General"),
        1 => Some("0"),
        2 => Some("0.00"),
        3 => Some("#,##0"),
        4 => Some("#,##0.00"),
        9 => Some("0%"),
        10 => Some("0.00%"),
        11 => Some("0.00E+00"),
        12 => Some("# ?/?"),
        13 => Some("# ??/??"),
        14 => Some("mm-dd-yy"),
        15 => Some("d-mmm-yy"),
        16 => Some("d-mmm"),
        17 => Some("mmm-yy"),
        18 => Some("h:mm AM/PM"),
        19 => Some("h:mm:ss AM/PM"),
        20 => Some("h:mm"),
        21 => Some("h:mm:ss"),
        22 => Some("m/d/yy h:mm"),
        37 => Some("#,##0 ;(#,##0)"),
        38 => Some("#,##0 ;[Red](#,##0)"),
        39 => Some("#,##0.00;(#,##0.00)"),
        40 => Some("#,##0.00;[Red](#,##0.00)"),
        45 => Some("mm:ss"),
        46 => Some("[h]:mm:ss"),
        47 => Some("mmss.0"),
        48 => Some("##0.0E+0"),
        49 => Some("@"),
        _ => None,
    }
}

// ── Helper functions ─────────────────────────────────────────────────────────

/// Parse an RGB color from the `rgb` attribute value (e.g. "FF000000" → [0,0,0]).
/// Skips the first 2 alpha hex chars and parses the remaining 6 as RGB.
fn parse_rgb_color(val: &[u8]) -> Option<[u8; 3]> {
    let s = std::str::from_utf8(val).ok()?;
    // Expect at least 6 hex chars; if 8, skip first 2 (alpha)
    let hex = if s.len() >= 8 {
        &s[2..8]
    } else if s.len() >= 6 {
        &s[0..6]
    } else {
        return None;
    };
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r, g, b])
}

/// Map a pattern type string to the `Pattern` enum.
fn parse_pattern_type(s: &str) -> Pattern {
    match s {
        "solid" => Pattern::Solid,
        "mediumGray" => Pattern::MediumGray,
        "darkGray" => Pattern::DarkGray,
        "lightGray" => Pattern::LightGray,
        "darkHorizontal" => Pattern::DarkHorizontal,
        "darkVertical" => Pattern::DarkVertical,
        "darkDown" => Pattern::DarkDown,
        "darkUp" => Pattern::DarkUp,
        "darkGrid" => Pattern::DarkGrid,
        "darkTrellis" => Pattern::DarkTrellis,
        "lightHorizontal" => Pattern::LightHorizontal,
        "lightVertical" => Pattern::LightVertical,
        "lightDown" => Pattern::LightDown,
        "lightUp" => Pattern::LightUp,
        "lightGrid" => Pattern::LightGrid,
        "lightTrellis" => Pattern::LightTrellis,
        "gray125" => Pattern::Gray125,
        _ => Pattern::None,
    }
}

/// Map a border style string to the `BorderStyle` enum.
fn parse_border_style(s: &str) -> BorderStyle {
    match s {
        "thin" => BorderStyle::Thin,
        "medium" => BorderStyle::Medium,
        "thick" => BorderStyle::Thick,
        "dashed" => BorderStyle::Dashed,
        "dotted" => BorderStyle::Dotted,
        "double" => BorderStyle::Double,
        _ => BorderStyle::None,
    }
}

/// Map horizontal alignment string to numeric value.
fn parse_h_align(s: &str) -> u8 {
    match s {
        "left" => 1,
        "center" => 2,
        "right" => 3,
        "fill" => 4,
        "justify" => 5,
        "centerContinuous" => 6,
        "distributed" => 7,
        _ => 0,
    }
}

/// Map vertical alignment string to numeric value.
fn parse_v_align(s: &str) -> u8 {
    match s {
        "top" => 1,
        "center" => 2,
        "bottom" => 3,
        "justify" => 4,
        "distributed" => 5,
        _ => 0,
    }
}

/// Helper: read a `<color>` element's rgb attribute from the current event.
/// Also handles theme color references by resolving them to RGB.
fn parse_color_from_event(event: &quick_xml::events::BytesStart<'_>) -> Option<[u8; 3]> {
    // First check for rgb attribute
    if let Some(rgb) = get_attr(event.attributes(), b"rgb").and_then(parse_rgb_color) {
        return Some(rgb);
    }
    // Then check for theme color reference
    if let Some(theme_idx) = get_attr(event.attributes(), b"theme")
        .and_then(|v| std::str::from_utf8(v).ok())
        .and_then(|v| v.parse::<u8>().ok())
        && let Some(idx) = crate::format::ThemeColorIndex::from_index(theme_idx)
    {
        let base_rgb = idx.default_rgb();
        let tint = get_attr(event.attributes(), b"tint")
            .and_then(|v| std::str::from_utf8(v).ok())
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);
        return Some(crate::format::apply_tint(base_rgb, tint));
    }
    None
}

/// Helper: extract a usize attribute value.
fn get_attr_usize(event: &quick_xml::events::BytesStart<'_>, name: &[u8]) -> usize {
    get_attr(event.attributes(), name)
        .and_then(|v| std::str::from_utf8(v).ok())
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0)
}

/// Helper: extract a u16 attribute value.
fn get_attr_u16(event: &quick_xml::events::BytesStart<'_>, name: &[u8]) -> u16 {
    get_attr(event.attributes(), name)
        .and_then(|v| std::str::from_utf8(v).ok())
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(0)
}

/// Helper: extract a string attribute value.
fn get_attr_string(event: &quick_xml::events::BytesStart<'_>, name: &[u8]) -> String {
    get_attr(event.attributes(), name)
        .and_then(|v| std::str::from_utf8(v).ok())
        .unwrap_or("")
        .to_string()
}

// ── Section parsers ─────────────────────────────────────────────────────────

/// Parse a single `<font>` element (from current Start tag to its End tag).
fn parse_single_font(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> crate::Result<ParsedFont> {
    let mut font = ParsedFont::default();
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(e) | Event::Empty(e) => match e.local_name().as_ref() {
                b"b" => font.bold = true,
                b"i" => font.italic = true,
                b"u" => {
                    let val = get_attr(e.attributes(), b"val")
                        .and_then(|v| std::str::from_utf8(v).ok())
                        .unwrap_or("single");
                    font.underline = match val {
                        "double" => Underline::Double,
                        "none" => Underline::None,
                        _ => Underline::Single,
                    };
                }
                b"strike" => font.strikethrough = true,
                b"sz" => {
                    if let Some(v) = get_attr(e.attributes(), b"val")
                        .and_then(|v| std::str::from_utf8(v).ok())
                        .and_then(|v| v.parse::<f64>().ok())
                    {
                        font.size = v;
                    }
                }
                b"name" => {
                    if let Some(v) =
                        get_attr(e.attributes(), b"val").and_then(|v| std::str::from_utf8(v).ok())
                    {
                        font.name = v.to_string();
                    }
                }
                b"color" => {
                    font.color = parse_color_from_event(&e);
                }
                _ => {}
            },
            Event::End(e) if e.local_name().as_ref() == b"font" => break,
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(font)
}

/// Parse a single `<fill>` element.
fn parse_single_fill(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> crate::Result<ParsedFill> {
    let mut fill = ParsedFill::default();
    let mut in_pattern_fill = false;
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(e) | Event::Empty(e) => match e.local_name().as_ref() {
                b"patternFill" => {
                    in_pattern_fill = true;
                    if let Some(pt) = get_attr(e.attributes(), b"patternType")
                        .and_then(|v| std::str::from_utf8(v).ok())
                    {
                        fill.pattern = parse_pattern_type(pt);
                    }
                }
                b"fgColor" if in_pattern_fill => {
                    fill.fg_color = parse_color_from_event(&e);
                }
                b"bgColor" if in_pattern_fill => {
                    fill.bg_color = parse_color_from_event(&e);
                }
                _ => {}
            },
            Event::End(e) => match e.local_name().as_ref() {
                b"patternFill" => in_pattern_fill = false,
                b"fill" => break,
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(fill)
}

/// Parse a single border side element (e.g. `<left style="thin"><color rgb="..."/></left>`).
/// Called when we encounter a Start event for left/right/top/bottom/diagonal.
fn parse_border_side(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    end_tag: &[u8],
    style_attr: Option<BorderStyle>,
) -> crate::Result<ParsedBorderSide> {
    let mut side = ParsedBorderSide {
        style: style_attr.unwrap_or(BorderStyle::None),
        color: None,
    };
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == b"color" => {
                side.color = parse_color_from_event(&e);
            }
            Event::End(e) if e.local_name().as_ref() == end_tag => break,
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(side)
}

/// Parse a single `<border>` element.
fn parse_single_border(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
) -> crate::Result<ParsedBorder> {
    let mut border = ParsedBorder::default();
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                let tag = e.local_name();
                let style_val = get_attr(e.attributes(), b"style")
                    .and_then(|v| std::str::from_utf8(v).ok())
                    .map(parse_border_style);
                match tag.as_ref() {
                    b"left" => border.left = parse_border_side(reader, buf, b"left", style_val)?,
                    b"right" => border.right = parse_border_side(reader, buf, b"right", style_val)?,
                    b"top" => border.top = parse_border_side(reader, buf, b"top", style_val)?,
                    b"bottom" => {
                        border.bottom = parse_border_side(reader, buf, b"bottom", style_val)?
                    }
                    b"diagonal" => {
                        border.diagonal = parse_border_side(reader, buf, b"diagonal", style_val)?
                    }
                    _ => {}
                }
            }
            Event::Empty(e) => {
                // Empty border side elements (e.g. `<left/>` with no children)
                let tag = e.local_name();
                let style_val = get_attr(e.attributes(), b"style")
                    .and_then(|v| std::str::from_utf8(v).ok())
                    .map(parse_border_style);
                let side = ParsedBorderSide {
                    style: style_val.unwrap_or(BorderStyle::None),
                    color: None,
                };
                match tag.as_ref() {
                    b"left" => border.left = side,
                    b"right" => border.right = side,
                    b"top" => border.top = side,
                    b"bottom" => border.bottom = side,
                    b"diagonal" => border.diagonal = side,
                    _ => {}
                }
            }
            Event::End(e) if e.local_name().as_ref() == b"border" => break,
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(border)
}

/// Parse an `<alignment>` element's attributes into a `ParsedAlignment`.
fn parse_alignment_attrs(event: &quick_xml::events::BytesStart<'_>) -> ParsedAlignment {
    let mut align = ParsedAlignment::default();
    if let Some(h) =
        get_attr(event.attributes(), b"horizontal").and_then(|v| std::str::from_utf8(v).ok())
    {
        align.horizontal = parse_h_align(h);
    }
    if let Some(v) =
        get_attr(event.attributes(), b"vertical").and_then(|v| std::str::from_utf8(v).ok())
    {
        align.vertical = parse_v_align(v);
    }
    if let Some(v) =
        get_attr(event.attributes(), b"wrapText").and_then(|v| std::str::from_utf8(v).ok())
    {
        align.wrap_text = v == "1" || v == "true";
    }
    if let Some(v) =
        get_attr(event.attributes(), b"shrinkToFit").and_then(|v| std::str::from_utf8(v).ok())
    {
        align.shrink_to_fit = v == "1" || v == "true";
    }
    if let Some(v) = get_attr(event.attributes(), b"indent")
        .and_then(|v| std::str::from_utf8(v).ok())
        .and_then(|v| v.parse::<u8>().ok())
    {
        align.indent = v;
    }
    if let Some(v) = get_attr(event.attributes(), b"textRotation")
        .and_then(|v| std::str::from_utf8(v).ok())
        .and_then(|v| v.parse::<i16>().ok())
    {
        align.rotation = v;
    }
    align
}

/// Parse a single `<dxf>` element containing optional font/fill/border/numFmt.
fn parse_single_dxf(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> crate::Result<DxfRecord> {
    let mut dxf = DxfRecord::default();
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(e) => match e.local_name().as_ref() {
                b"font" => dxf.font = Some(parse_single_font(reader, buf)?),
                b"fill" => dxf.fill = Some(parse_single_fill(reader, buf)?),
                b"border" => dxf.border = Some(parse_single_border(reader, buf)?),
                b"numFmt" => {
                    let id = get_attr_u16(&e, b"numFmtId");
                    let code = get_attr_string(&e, b"formatCode");
                    dxf.num_fmt = Some((id, code));
                }
                _ => {}
            },
            Event::Empty(e) if e.local_name().as_ref() == b"numFmt" => {
                let id = get_attr_u16(&e, b"numFmtId");
                let code = get_attr_string(&e, b"formatCode");
                dxf.num_fmt = Some((id, code));
            }
            Event::End(e) if e.local_name().as_ref() == b"dxf" => break,
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(dxf)
}

// ── Main parse function ─────────────────────────────────────────────────────

pub fn parse_styles(data: &[u8]) -> crate::Result<ParsedStyles> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = false;
    let mut buf = Vec::with_capacity(512);

    let mut num_formats = Vec::new();
    let mut xf_num_fmt_ids = Vec::new();
    let mut fonts = Vec::new();
    let mut fills = Vec::new();
    let mut borders = Vec::new();
    let mut xf_records = Vec::new();
    let mut dxf_records = Vec::new();

    // Track which section we're in
    let mut in_fonts = false;
    let mut in_fills = false;
    let mut in_borders = false;
    let mut in_cell_xfs = false;
    let mut in_dxfs = false;

    // For xf records: we need to track the current xf and whether we're inside one
    let mut current_xf: Option<XfRecord> = None;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                let local = e.local_name();
                match local.as_ref() {
                    // ── Section containers ──
                    b"fonts" => in_fonts = true,
                    b"fills" => in_fills = true,
                    b"borders" => in_borders = true,
                    b"cellXfs" => in_cell_xfs = true,
                    b"dxfs" => in_dxfs = true,

                    // ── Font parsing ──
                    b"font" if in_fonts && !in_dxfs => {
                        fonts.push(parse_single_font(&mut reader, &mut buf)?);
                    }

                    // ── Fill parsing ──
                    b"fill" if in_fills && !in_dxfs => {
                        fills.push(parse_single_fill(&mut reader, &mut buf)?);
                    }

                    // ── Border parsing ──
                    b"border" if in_borders && !in_dxfs => {
                        borders.push(parse_single_border(&mut reader, &mut buf)?);
                    }

                    // ── cellXfs / xf parsing ──
                    b"xf" if in_cell_xfs => {
                        let num_fmt_id = get_attr_u16(&e, b"numFmtId");
                        let font_id = get_attr_usize(&e, b"fontId");
                        let fill_id = get_attr_usize(&e, b"fillId");
                        let border_id = get_attr_usize(&e, b"borderId");
                        xf_num_fmt_ids.push(num_fmt_id);
                        current_xf = Some(XfRecord {
                            font_id,
                            fill_id,
                            border_id,
                            num_fmt_id,
                            alignment: ParsedAlignment::default(),
                        });
                    }

                    // ── Alignment inside xf ──
                    b"alignment" if current_xf.is_some() => {
                        if let Some(ref mut xf) = current_xf {
                            xf.alignment = parse_alignment_attrs(&e);
                        }
                    }

                    // ── DXF parsing ──
                    b"dxf" if in_dxfs => {
                        dxf_records.push(parse_single_dxf(&mut reader, &mut buf)?);
                    }

                    // ── numFmt (can appear outside sections) ──
                    b"numFmt" => {
                        let id = get_attr_u16(&e, b"numFmtId");
                        let code = get_attr_string(&e, b"formatCode");
                        num_formats.push((id, code));
                    }

                    _ => {}
                }
            }
            Event::Empty(e) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"numFmt" => {
                        let id = get_attr_u16(&e, b"numFmtId");
                        let code = get_attr_string(&e, b"formatCode");
                        num_formats.push((id, code));
                    }
                    b"xf" if in_cell_xfs => {
                        let num_fmt_id = get_attr_u16(&e, b"numFmtId");
                        let font_id = get_attr_usize(&e, b"fontId");
                        let fill_id = get_attr_usize(&e, b"fillId");
                        let border_id = get_attr_usize(&e, b"borderId");
                        xf_num_fmt_ids.push(num_fmt_id);
                        xf_records.push(XfRecord {
                            font_id,
                            fill_id,
                            border_id,
                            num_fmt_id,
                            alignment: ParsedAlignment::default(),
                        });
                    }
                    b"alignment" if current_xf.is_some() => {
                        if let Some(ref mut xf) = current_xf {
                            xf.alignment = parse_alignment_attrs(&e);
                        }
                    }
                    // Empty font/fill/border elements (rare but possible)
                    b"font" if in_fonts && !in_dxfs => {
                        fonts.push(ParsedFont::default());
                    }
                    b"fill" if in_fills && !in_dxfs => {
                        fills.push(ParsedFill::default());
                    }
                    b"border" if in_borders && !in_dxfs => {
                        borders.push(ParsedBorder::default());
                    }
                    b"dxf" if in_dxfs => {
                        dxf_records.push(DxfRecord::default());
                    }
                    _ => {}
                }
            }
            Event::End(e) => match e.local_name().as_ref() {
                b"fonts" => in_fonts = false,
                b"fills" => in_fills = false,
                b"borders" => in_borders = false,
                b"cellXfs" => in_cell_xfs = false,
                b"dxfs" => in_dxfs = false,
                b"xf" if in_cell_xfs => {
                    if let Some(xf) = current_xf.take() {
                        xf_records.push(xf);
                    }
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }

    if xf_num_fmt_ids.is_empty() {
        xf_num_fmt_ids.push(0);
    }

    Ok(ParsedStyles {
        num_formats,
        xf_num_fmt_ids,
        fonts,
        fills,
        borders,
        xf_records,
        dxf_records,
    })
}

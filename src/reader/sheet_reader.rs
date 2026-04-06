use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::cell::CellValue;
use crate::datetime::ExcelDateTime;
use crate::model::shared_strings::SharedStringTable;
use crate::model::style_registry::StyleRegistry;
use crate::reader::style_parser::ParsedStyles;
use crate::utility::{parse_cell_attr, ColNum, RowNum};
use crate::xml::xml_reader::get_attr;

pub struct RawCell {
    pub row: RowNum,
    pub col: ColNum,
    pub value: CellValue,
    pub xf_index: u32,
}

/// Metadata parsed from sheet XML (merges, widths, heights, freeze panes).
#[derive(Default)]
pub struct SheetMeta {
    pub merge_ranges: Vec<(RowNum, ColNum, RowNum, ColNum)>,
    pub col_widths: Vec<(ColNum, f64)>,
    pub row_heights: Vec<(RowNum, f64)>,
    pub freeze_row: RowNum,
    pub freeze_col: ColNum,
    pub drawing_rid: Option<String>,
    pub legacy_drawing_rid: Option<String>,
}

/// Read cells AND metadata from sheet XML in one pass.
pub fn read_sheet_full(
    data: &[u8],
    sst: &SharedStringTable,
    styles: &ParsedStyles,
) -> crate::Result<(Vec<RawCell>, SheetMeta)> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = true;
    let mut buf = Vec::with_capacity(1024);
    let mut cells = Vec::new();
    let mut meta = SheetMeta::default();

    // Pre-sheetData scan: cols, pane, merges
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) | Event::Empty(e) => {
                match e.local_name().as_ref() {
                    b"col" => {
                        let min = get_attr(e.attributes(), b"min").and_then(|v| atoi_simd::parse::<u16>(v).ok()).unwrap_or(1);
                        let width = get_attr(e.attributes(), b"width").and_then(|v| std::str::from_utf8(v).ok()).and_then(|s| s.parse::<f64>().ok());
                        if let Some(w) = width { meta.col_widths.push((min - 1, w)); }
                    }
                    b"pane" => {
                        meta.freeze_row = get_attr(e.attributes(), b"ySplit").and_then(|v| atoi_simd::parse::<u32>(v).ok()).unwrap_or(0);
                        meta.freeze_col = get_attr(e.attributes(), b"xSplit").and_then(|v| atoi_simd::parse::<u16>(v).ok()).unwrap_or(0);
                    }
                    b"mergeCell" => {
                        if let Some(r) = get_attr(e.attributes(), b"ref").and_then(|v| std::str::from_utf8(v).ok()) {
                            if let Some(m) = crate::utility::parse_range(r) { meta.merge_ranges.push(m); }
                        }
                    }
                    b"sheetData" => break,
                    _ => {}
                }
            }
            Event::Eof => return Ok((cells, meta)),
            _ => {}
        }
    }

    let mut cell_pos: Option<(RowNum, ColNum)> = None;
    let mut cell_type: Option<Vec<u8>> = None;
    let mut cell_style: usize = 0;
    let mut in_v = false;
    let mut in_f = false;
    let mut v_text = String::new();
    let mut f_text = String::new();
    let mut cell_buf = Vec::with_capacity(256);

    loop {
        cell_buf.clear();
        match reader.read_event_into(&mut cell_buf)? {
            Event::Start(e) => {
                match e.local_name().as_ref() {
                    b"row" => {
                        if let Some(ht) = get_attr(e.attributes(), b"ht").and_then(|v| std::str::from_utf8(v).ok()).and_then(|s| s.parse::<f64>().ok()) {
                            if let Some(r) = get_attr(e.attributes(), b"r").and_then(|v| atoi_simd::parse::<u32>(v).ok()) {
                                meta.row_heights.push((r - 1, ht));
                            }
                        }
                    }
                    b"c" => {
                        cell_pos = get_attr(e.attributes(), b"r").and_then(parse_cell_attr);
                        cell_type = get_attr(e.attributes(), b"t").map(|v| v.to_vec());
                        cell_style = get_attr(e.attributes(), b"s")
                            .and_then(|v| atoi_simd::parse::<usize>(v).ok())
                            .unwrap_or(0);
                        v_text.clear();
                        f_text.clear();
                    }
                    b"v" => { in_v = true; v_text.clear(); }
                    b"f" => { in_f = true; f_text.clear(); }
                    _ => {}
                }
            }
            Event::Text(e) => {
                if in_v {
                    if let Ok(t) = e.unescape() { v_text.push_str(&t); }
                } else if in_f {
                    if let Ok(t) = e.unescape() { f_text.push_str(&t); }
                }
            }
            Event::End(e) => {
                match e.local_name().as_ref() {
                    b"v" => { in_v = false; }
                    b"f" => { in_f = false; }
                    b"c" => {
                        if let Some((row, col)) = cell_pos {
                            let value = resolve_cell_value(
                                cell_type.as_deref(), &v_text, &f_text,
                                cell_style, sst, styles,
                            );
                            if !value.is_empty() || !f_text.is_empty() {
                                cells.push(RawCell { row, col, value, xf_index: cell_style as u32 });
                            }
                        }
                        cell_pos = None;
                        cell_type = None;
                    }
                    b"sheetData" => break,
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    // Post-sheetData: scan for mergeCells, drawing, legacyDrawing
    loop {
        cell_buf.clear();
        match reader.read_event_into(&mut cell_buf)? {
            Event::Start(e) | Event::Empty(e) => {
                match e.local_name().as_ref() {
                    b"mergeCell" => {
                        if let Some(r) = get_attr(e.attributes(), b"ref").and_then(|v| std::str::from_utf8(v).ok()) {
                            if let Some(m) = crate::utility::parse_range(r) { meta.merge_ranges.push(m); }
                        }
                    }
                    b"drawing" => {
                        meta.drawing_rid = get_attr(e.attributes(), b"r:id").and_then(|v| std::str::from_utf8(v).ok()).map(|s| s.to_string());
                    }
                    b"legacyDrawing" => {
                        meta.legacy_drawing_rid = get_attr(e.attributes(), b"r:id").and_then(|v| std::str::from_utf8(v).ok()).map(|s| s.to_string());
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok((cells, meta))
}

/// Backward-compatible wrapper: read cells only.
pub fn _read_sheet_cells(
    data: &[u8],
    sst: &SharedStringTable,
    styles: &ParsedStyles,
) -> crate::Result<Vec<RawCell>> {
    read_sheet_full(data, sst, styles).map(|(cells, _)| cells)
}

fn resolve_cell_value(
    cell_type: Option<&[u8]>, v: &str, f: &str,
    style_idx: usize, sst: &SharedStringTable, styles: &ParsedStyles,
) -> CellValue {
    let base = match cell_type {
        Some(b"s") => {
            let idx: u32 = atoi_simd::parse(v.as_bytes()).unwrap_or(0);
            sst.get(idx).map(|s| CellValue::String(s.to_string())).unwrap_or(CellValue::Empty)
        }
        Some(b"b") => CellValue::Bool(v == "1"),
        Some(b"e") => CellValue::Error(v.to_string()),
        Some(b"str") | Some(b"inlineStr") => CellValue::String(v.to_string()),
        _ => {
            if v.is_empty() {
                CellValue::Empty
            } else if let Ok(n) = fast_float2::parse::<f64, _>(v) {
                let num_fmt_id = styles.xf_num_fmt_ids.get(style_idx).copied().unwrap_or(0);
                if StyleRegistry::is_date_format(num_fmt_id, &styles.num_formats) {
                    CellValue::DateTime(ExcelDateTime::new(n, false))
                } else {
                    CellValue::Number(n)
                }
            } else {
                CellValue::String(v.to_string())
            }
        }
    };

    if !f.is_empty() {
        CellValue::Formula { formula: f.to_string(), cached_value: Box::new(base) }
    } else {
        base
    }
}

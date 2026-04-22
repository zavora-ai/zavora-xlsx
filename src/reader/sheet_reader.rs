use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::cell::CellValue;
use crate::datetime::ExcelDateTime;
use crate::model::shared_strings::SharedStringTable;
use crate::model::style_registry::StyleRegistry;
use crate::reader::style_parser::ParsedStyles;
use crate::utility::{ColNum, RowNum, parse_cell_attr};
use crate::xml::xml_reader::{get_attr, get_attr_str};

pub struct RawCell {
    pub row: RowNum,
    pub col: ColNum,
    pub value: CellValue,
    pub xf_index: u32,
}

/// A hyperlink parsed from the sheet XML (before relationship resolution).
#[derive(Debug, Clone)]
pub struct ParsedHyperlink {
    pub cell_ref: String,
    pub rid: Option<String>,
    pub location: Option<String>,
    #[allow(dead_code)]
    pub display: Option<String>,
    pub tooltip: Option<String>,
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
    pub hyperlinks: Vec<ParsedHyperlink>,
    pub print_settings: Option<crate::worksheet::PrintSettings>,
    pub protection: Option<crate::worksheet::SheetProtection>,
    pub row_outline_levels: std::collections::BTreeMap<RowNum, u8>,
    pub col_outline_levels: std::collections::BTreeMap<ColNum, u8>,
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
    let mut fit_to_page = false;

    // Pre-sheetData scan: cols, pane, merges, pageSetUpPr
    loop {
        buf.clear();
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) | Event::Empty(e) => {
                match e.local_name().as_ref() {
                    b"col" => {
                        let min = get_attr(e.attributes(), b"min")
                            .and_then(|v| atoi_simd::parse::<u16>(v).ok())
                            .unwrap_or(1);
                        let max = get_attr(e.attributes(), b"max")
                            .and_then(|v| atoi_simd::parse::<u16>(v).ok())
                            .unwrap_or(min);
                        let width = get_attr(e.attributes(), b"width")
                            .and_then(|v| std::str::from_utf8(v).ok())
                            .and_then(|s| s.parse::<f64>().ok());
                        let outline_level = get_attr(e.attributes(), b"outlineLevel")
                            .and_then(|v| atoi_simd::parse::<u8>(v).ok());
                        for col_1based in min..=max {
                            if let Some(w) = width {
                                meta.col_widths.push((col_1based - 1, w));
                            }
                            if let Some(level) = outline_level
                                && level > 0
                            {
                                meta.col_outline_levels.insert(col_1based - 1, level);
                            }
                        }
                    }
                    b"pane" => {
                        meta.freeze_row = get_attr(e.attributes(), b"ySplit")
                            .and_then(|v| atoi_simd::parse::<u32>(v).ok())
                            .unwrap_or(0);
                        meta.freeze_col = get_attr(e.attributes(), b"xSplit")
                            .and_then(|v| atoi_simd::parse::<u16>(v).ok())
                            .unwrap_or(0);
                    }
                    b"mergeCell" => {
                        if let Some(r) = get_attr(e.attributes(), b"ref")
                            .and_then(|v| std::str::from_utf8(v).ok())
                            && let Some(m) = crate::utility::parse_range(r)
                        {
                            meta.merge_ranges.push(m);
                        }
                    }
                    b"pageSetUpPr" => {
                        fit_to_page = get_attr_str(e.attributes(), b"fitToPage")
                            .is_some_and(|v| v == "1" || v == "true");
                    }
                    b"sheetProtection" => {
                        #[allow(clippy::field_reassign_with_default)]
                        {
                            let mut prot = crate::worksheet::SheetProtection::default();
                            // The "sheet" attribute indicates protection is enabled
                            prot.sheet = get_attr_str(e.attributes(), b"sheet")
                                .is_some_and(|v| v == "1" || v == "true");
                            prot.objects = get_attr_str(e.attributes(), b"objects")
                                .is_some_and(|v| v == "1" || v == "true");
                            prot.scenarios = get_attr_str(e.attributes(), b"scenarios")
                                .is_some_and(|v| v == "1" || v == "true");
                            prot.format_cells = get_attr_str(e.attributes(), b"formatCells")
                                .is_none_or(|v| v == "1" || v == "true");
                            prot.format_columns = get_attr_str(e.attributes(), b"formatColumns")
                                .is_none_or(|v| v == "1" || v == "true");
                            prot.format_rows = get_attr_str(e.attributes(), b"formatRows")
                                .is_none_or(|v| v == "1" || v == "true");
                            prot.insert_columns = get_attr_str(e.attributes(), b"insertColumns")
                                .is_none_or(|v| v == "1" || v == "true");
                            prot.insert_rows = get_attr_str(e.attributes(), b"insertRows")
                                .is_none_or(|v| v == "1" || v == "true");
                            prot.insert_hyperlinks =
                                get_attr_str(e.attributes(), b"insertHyperlinks")
                                    .is_none_or(|v| v == "1" || v == "true");
                            prot.delete_columns = get_attr_str(e.attributes(), b"deleteColumns")
                                .is_none_or(|v| v == "1" || v == "true");
                            prot.delete_rows = get_attr_str(e.attributes(), b"deleteRows")
                                .is_none_or(|v| v == "1" || v == "true");
                            prot.select_locked_cells =
                                get_attr_str(e.attributes(), b"selectLockedCells")
                                    .is_some_and(|v| v == "1" || v == "true");
                            prot.sort = get_attr_str(e.attributes(), b"sort")
                                .is_none_or(|v| v == "1" || v == "true");
                            prot.auto_filter = get_attr_str(e.attributes(), b"autoFilter")
                                .is_none_or(|v| v == "1" || v == "true");
                            prot.pivot_tables = get_attr_str(e.attributes(), b"pivotTables")
                                .is_none_or(|v| v == "1" || v == "true");
                            prot.select_unlocked_cells =
                                get_attr_str(e.attributes(), b"selectUnlockedCells")
                                    .is_some_and(|v| v == "1" || v == "true");
                            // Legacy password hash
                            prot.password_hash =
                                get_attr_str(e.attributes(), b"password").map(|s| s.to_string());
                            meta.protection = Some(prot);
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
            Event::Start(e) => match e.local_name().as_ref() {
                b"row" => {
                    let row_num = get_attr(e.attributes(), b"r")
                        .and_then(|v| atoi_simd::parse::<u32>(v).ok());
                    if let Some(ht) = get_attr(e.attributes(), b"ht")
                        .and_then(|v| std::str::from_utf8(v).ok())
                        .and_then(|s| s.parse::<f64>().ok())
                        && let Some(r) = row_num
                    {
                        meta.row_heights.push((r - 1, ht));
                    }
                    if let Some(level) = get_attr(e.attributes(), b"outlineLevel")
                        .and_then(|v| atoi_simd::parse::<u8>(v).ok())
                        && level > 0
                        && let Some(r) = row_num
                    {
                        meta.row_outline_levels.insert(r - 1, level);
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
                b"v" => {
                    in_v = true;
                    v_text.clear();
                }
                b"f" => {
                    in_f = true;
                    f_text.clear();
                }
                _ => {}
            },
            Event::Text(e) => {
                if in_v {
                    if let Ok(t) = e.unescape() {
                        v_text.push_str(&t);
                    }
                } else if in_f && let Ok(t) = e.unescape() {
                    f_text.push_str(&t);
                }
            }
            Event::End(e) => match e.local_name().as_ref() {
                b"v" => {
                    in_v = false;
                }
                b"f" => {
                    in_f = false;
                }
                b"c" => {
                    if let Some((row, col)) = cell_pos {
                        let value = resolve_cell_value(
                            cell_type.as_deref(),
                            &v_text,
                            &f_text,
                            cell_style,
                            sst,
                            styles,
                        );
                        if !value.is_empty() || !f_text.is_empty() {
                            cells.push(RawCell {
                                row,
                                col,
                                value,
                                xf_index: cell_style as u32,
                            });
                        }
                    }
                    cell_pos = None;
                    cell_type = None;
                }
                b"sheetData" => break,
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }

    // Post-sheetData: scan for mergeCells, drawing, legacyDrawing, hyperlinks, print settings
    let mut ps = crate::worksheet::PrintSettings::default();
    let mut has_print_settings = false;
    let mut in_header_footer = false;
    let mut in_odd_header = false;
    let mut in_odd_footer = false;
    let mut odd_header_text = String::new();
    let mut odd_footer_text = String::new();
    let mut in_row_breaks = false;
    let mut in_col_breaks = false;

    loop {
        cell_buf.clear();
        match reader.read_event_into(&mut cell_buf)? {
            Event::Start(e) | Event::Empty(e) => match e.local_name().as_ref() {
                b"mergeCell" => {
                    if let Some(r) =
                        get_attr(e.attributes(), b"ref").and_then(|v| std::str::from_utf8(v).ok())
                        && let Some(m) = crate::utility::parse_range(r)
                    {
                        meta.merge_ranges.push(m);
                    }
                }
                b"drawing" => {
                    meta.drawing_rid = get_attr(e.attributes(), b"r:id")
                        .and_then(|v| std::str::from_utf8(v).ok())
                        .map(|s| s.to_string());
                }
                b"legacyDrawing" => {
                    meta.legacy_drawing_rid = get_attr(e.attributes(), b"r:id")
                        .and_then(|v| std::str::from_utf8(v).ok())
                        .map(|s| s.to_string());
                }
                b"sheetProtection" => {
                    #[allow(clippy::field_reassign_with_default)]
                    {
                        let mut prot = crate::worksheet::SheetProtection::default();
                        prot.sheet = get_attr_str(e.attributes(), b"sheet")
                            .is_some_and(|v| v == "1" || v == "true");
                        prot.objects = get_attr_str(e.attributes(), b"objects")
                            .is_some_and(|v| v == "1" || v == "true");
                        prot.scenarios = get_attr_str(e.attributes(), b"scenarios")
                            .is_some_and(|v| v == "1" || v == "true");
                        prot.format_cells = get_attr_str(e.attributes(), b"formatCells")
                            .is_none_or(|v| v == "1" || v == "true");
                        prot.format_columns = get_attr_str(e.attributes(), b"formatColumns")
                            .is_none_or(|v| v == "1" || v == "true");
                        prot.format_rows = get_attr_str(e.attributes(), b"formatRows")
                            .is_none_or(|v| v == "1" || v == "true");
                        prot.insert_columns = get_attr_str(e.attributes(), b"insertColumns")
                            .is_none_or(|v| v == "1" || v == "true");
                        prot.insert_rows = get_attr_str(e.attributes(), b"insertRows")
                            .is_none_or(|v| v == "1" || v == "true");
                        prot.insert_hyperlinks = get_attr_str(e.attributes(), b"insertHyperlinks")
                            .is_none_or(|v| v == "1" || v == "true");
                        prot.delete_columns = get_attr_str(e.attributes(), b"deleteColumns")
                            .is_none_or(|v| v == "1" || v == "true");
                        prot.delete_rows = get_attr_str(e.attributes(), b"deleteRows")
                            .is_none_or(|v| v == "1" || v == "true");
                        prot.select_locked_cells =
                            get_attr_str(e.attributes(), b"selectLockedCells")
                                .is_some_and(|v| v == "1" || v == "true");
                        prot.sort = get_attr_str(e.attributes(), b"sort")
                            .is_none_or(|v| v == "1" || v == "true");
                        prot.auto_filter = get_attr_str(e.attributes(), b"autoFilter")
                            .is_none_or(|v| v == "1" || v == "true");
                        prot.pivot_tables = get_attr_str(e.attributes(), b"pivotTables")
                            .is_none_or(|v| v == "1" || v == "true");
                        prot.select_unlocked_cells =
                            get_attr_str(e.attributes(), b"selectUnlockedCells")
                                .is_some_and(|v| v == "1" || v == "true");
                        prot.password_hash =
                            get_attr_str(e.attributes(), b"password").map(|s| s.to_string());
                        meta.protection = Some(prot);
                    }
                }
                b"hyperlink" => {
                    let cell_ref = get_attr_str(e.attributes(), b"ref")
                        .unwrap_or("")
                        .to_string();
                    let rid = get_attr_str(e.attributes(), b"r:id").map(|s| s.to_string());
                    let location = get_attr_str(e.attributes(), b"location").map(|s| s.to_string());
                    let display = get_attr_str(e.attributes(), b"display").map(|s| s.to_string());
                    let tooltip = get_attr_str(e.attributes(), b"tooltip").map(|s| s.to_string());
                    if !cell_ref.is_empty() {
                        meta.hyperlinks.push(ParsedHyperlink {
                            cell_ref,
                            rid,
                            location,
                            display,
                            tooltip,
                        });
                    }
                }
                b"pageMargins" => {
                    has_print_settings = true;
                    ps.margin_top =
                        get_attr_str(e.attributes(), b"top").and_then(|s| s.parse().ok());
                    ps.margin_bottom =
                        get_attr_str(e.attributes(), b"bottom").and_then(|s| s.parse().ok());
                    ps.margin_left =
                        get_attr_str(e.attributes(), b"left").and_then(|s| s.parse().ok());
                    ps.margin_right =
                        get_attr_str(e.attributes(), b"right").and_then(|s| s.parse().ok());
                    ps.margin_header =
                        get_attr_str(e.attributes(), b"header").and_then(|s| s.parse().ok());
                    ps.margin_footer =
                        get_attr_str(e.attributes(), b"footer").and_then(|s| s.parse().ok());
                }
                b"pageSetup" => {
                    has_print_settings = true;
                    ps.paper_size = get_attr(e.attributes(), b"paperSize")
                        .and_then(|v| atoi_simd::parse::<u8>(v).ok());
                    ps.scale = get_attr(e.attributes(), b"scale")
                        .and_then(|v| atoi_simd::parse::<u16>(v).ok());
                    if let Some(orient) = get_attr_str(e.attributes(), b"orientation") {
                        ps.orientation = match orient {
                            "landscape" => Some(crate::worksheet::Orientation::Landscape),
                            "portrait" => Some(crate::worksheet::Orientation::Portrait),
                            _ => None,
                        };
                    }
                    ps.fit_to_width = get_attr(e.attributes(), b"fitToWidth")
                        .and_then(|v| atoi_simd::parse::<u16>(v).ok());
                    ps.fit_to_height = get_attr(e.attributes(), b"fitToHeight")
                        .and_then(|v| atoi_simd::parse::<u16>(v).ok());
                    ps.black_and_white = get_attr_str(e.attributes(), b"blackAndWhite")
                        .is_some_and(|v| v == "1" || v == "true");
                    ps.first_page_number = get_attr(e.attributes(), b"firstPageNumber")
                        .and_then(|v| atoi_simd::parse::<u16>(v).ok());
                }
                b"headerFooter" => {
                    in_header_footer = true;
                    has_print_settings = true;
                }
                b"oddHeader" if in_header_footer => {
                    in_odd_header = true;
                    odd_header_text.clear();
                }
                b"oddFooter" if in_header_footer => {
                    in_odd_footer = true;
                    odd_footer_text.clear();
                }
                b"rowBreaks" => {
                    in_row_breaks = true;
                    has_print_settings = true;
                }
                b"colBreaks" => {
                    in_col_breaks = true;
                    has_print_settings = true;
                }
                b"brk" => {
                    let id = get_attr(e.attributes(), b"id")
                        .and_then(|v| std::str::from_utf8(v).ok())
                        .and_then(|s| s.parse::<u32>().ok());
                    if let Some(id_val) = id {
                        if in_row_breaks {
                            ps.row_breaks.push(id_val - 1);
                        } else if in_col_breaks {
                            ps.col_breaks.push(id_val.saturating_sub(1) as ColNum);
                        }
                    }
                }
                _ => {}
            },
            Event::Text(e) => {
                if in_odd_header {
                    if let Ok(t) = e.unescape() {
                        odd_header_text.push_str(&t);
                    }
                } else if in_odd_footer && let Ok(t) = e.unescape() {
                    odd_footer_text.push_str(&t);
                }
            }
            Event::End(e) => match e.local_name().as_ref() {
                b"oddHeader" => {
                    in_odd_header = false;
                    if !odd_header_text.is_empty() {
                        ps.header = Some(odd_header_text.clone());
                    }
                }
                b"oddFooter" => {
                    in_odd_footer = false;
                    if !odd_footer_text.is_empty() {
                        ps.footer = Some(odd_footer_text.clone());
                    }
                }
                b"headerFooter" => {
                    in_header_footer = false;
                }
                b"rowBreaks" => {
                    in_row_breaks = false;
                }
                b"colBreaks" => {
                    in_col_breaks = false;
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }

    if has_print_settings || fit_to_page {
        ps.fit_to_page = fit_to_page;
        meta.print_settings = Some(ps);
    }

    Ok((cells, meta))
}

/// Backward-compatible wrapper: read cells only.
#[allow(dead_code)]
pub fn read_sheet_cells(
    data: &[u8],
    sst: &SharedStringTable,
    styles: &ParsedStyles,
) -> crate::Result<Vec<RawCell>> {
    read_sheet_full(data, sst, styles).map(|(cells, _)| cells)
}

fn resolve_cell_value(
    cell_type: Option<&[u8]>,
    v: &str,
    f: &str,
    style_idx: usize,
    sst: &SharedStringTable,
    styles: &ParsedStyles,
) -> CellValue {
    let base = match cell_type {
        Some(b"s") => {
            let idx: u32 = atoi_simd::parse(v.as_bytes()).unwrap_or(0);
            sst.get(idx)
                .map(|s| CellValue::String(s.to_string()))
                .unwrap_or(CellValue::Empty)
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
        CellValue::Formula {
            formula: f.to_string(),
            cached_value: Box::new(base),
        }
    } else {
        base
    }
}

//! Sheet writer — split into submodules.

mod cells;
mod validation;

use std::collections::BTreeMap;
use crate::cell::CellType;
use crate::utility::{col_to_letter, ColNum, RowNum};
use crate::xml::xml_writer::XmlWriter;
use crate::features::conditional::StoredCf;
use crate::features::sparkline::Sparkline;
use crate::features::validation::DataValidation;
use crate::worksheet::{Hyperlink, Orientation, PrintSettings, SheetProtection};

use cells::write_cell;
use validation::{write_data_validation, compute_dimension};

pub(crate) struct SheetCells<'a> {
    pub cells: &'a BTreeMap<RowNum, BTreeMap<ColNum, (CellType, u32)>>,
    pub merge_ranges: &'a [(RowNum, ColNum, RowNum, ColNum)],
    pub col_widths: &'a BTreeMap<ColNum, f64>,
    pub row_heights: &'a BTreeMap<RowNum, f64>,
    pub freeze_row: RowNum,
    pub freeze_col: ColNum,
    pub drawing_rid: Option<String>,
    pub table_parts: Vec<String>,
    pub conditional_formats: &'a [StoredCf],
    pub validations: &'a [DataValidation],
    pub sparklines: &'a [Sparkline],
    pub protection: Option<&'a SheetProtection>,
    pub print_settings: Option<&'a PrintSettings>,
    pub hidden_rows: &'a std::collections::BTreeSet<RowNum>,
    pub hidden_cols: &'a std::collections::BTreeSet<ColNum>,
    pub autofilter: Option<(RowNum, ColNum, RowNum, ColNum)>,
    pub hyperlinks: &'a [Hyperlink],
    #[allow(dead_code)]
    pub hyperlink_rels: &'a [(String, String)],
    pub row_outline_levels: &'a BTreeMap<RowNum, u8>,
    pub col_outline_levels: &'a BTreeMap<ColNum, u8>,
    pub legacy_drawing_rid: Option<String>,
    pub zoom: Option<u16>,
    pub show_gridlines: bool,
    pub show_headings: bool,
    pub right_to_left: bool,
    pub tab_color: Option<[u8; 3]>,
    pub is_active: bool,
    pub selection: Option<(RowNum, ColNum)>,
    pub top_left_cell: Option<(RowNum, ColNum)>,
    pub default_row_height: Option<f64>,
    pub col_formats: &'a BTreeMap<ColNum, u32>,
    pub row_formats: &'a BTreeMap<RowNum, u32>,
    pub ignored_errors: &'a [(String, String)],
    pub autofilter_columns: &'a [(ColNum, Vec<String>)],
}

// write_sheet is the main orchestrator — kept in mod.rs as it coordinates all submodules.
// At 350 lines it's the largest single function but each section is clearly commented.

pub(crate) fn write_sheet(data: &SheetCells<'_>) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag("worksheet", &[
        ("xmlns", "http://schemas.openxmlformats.org/spreadsheetml/2006/main"),
        ("xmlns:r", "http://schemas.openxmlformats.org/officeDocument/2006/relationships"),
    ]);

    // 0. sheetPr (tab color, fit-to-page)
    let need_sheet_pr = data.tab_color.is_some()
        || data.print_settings.map_or(false, |ps| ps.fit_to_page);
    if need_sheet_pr {
        w.start_tag("sheetPr", &[]);
        if let Some(rgb) = data.tab_color {
            let hex = format!("FF{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
            w.empty_tag("tabColor", &[("rgb", &hex)]);
        }
        if data.print_settings.map_or(false, |ps| ps.fit_to_page) {
            w.empty_tag("pageSetUpPr", &[("fitToPage", "1")]);
        }
        w.end_tag("sheetPr");
    }

    // 1. dimension
    let dim = compute_dimension(data);
    w.empty_tag("dimension", &[("ref", &dim)]);

    // 2. sheetViews (always present)
    w.start_tag("sheetViews", &[]);
    let mut sv_attrs: Vec<(&str, String)> = vec![("workbookViewId", "0".into())];
    if data.is_active { sv_attrs.push(("tabSelected", "1".into())); }
    if !data.show_gridlines { sv_attrs.push(("showGridLines", "0".into())); }
    if !data.show_headings { sv_attrs.push(("showRowColHeaders", "0".into())); }
    if data.right_to_left { sv_attrs.push(("rightToLeft", "1".into())); }
    if let Some(z) = data.zoom { sv_attrs.push(("zoomScale", z.to_string())); sv_attrs.push(("zoomScaleNormal", z.to_string())); }
    if let Some((r, c)) = data.top_left_cell {
        sv_attrs.push(("topLeftCell", format!("{}{}", col_to_letter(c), r + 1)));
    }
    let sv_refs: Vec<(&str, &str)> = sv_attrs.iter().map(|(k, v)| (*k, v.as_str())).collect();
    w.start_tag("sheetView", &sv_refs);
    if data.freeze_row > 0 || data.freeze_col > 0 {
        let top_left = format!("{}{}", col_to_letter(data.freeze_col), data.freeze_row + 1);
        let mut pane_attrs: Vec<(&str, String)> = Vec::new();
        if data.freeze_col > 0 { pane_attrs.push(("xSplit", data.freeze_col.to_string())); }
        if data.freeze_row > 0 { pane_attrs.push(("ySplit", data.freeze_row.to_string())); }
        pane_attrs.push(("topLeftCell", top_left));
        pane_attrs.push(("state", "frozen".into()));
        let refs: Vec<(&str, &str)> = pane_attrs.iter().map(|(k, v)| (*k, v.as_str())).collect();
        w.empty_tag("pane", &refs);
    }
    if let Some((r, c)) = data.selection {
        let cell = format!("{}{}", col_to_letter(c), r + 1);
        w.empty_tag("selection", &[("activeCell", &cell), ("sqref", &cell)]);
    }
    w.end_tag("sheetView");
    w.end_tag("sheetViews");

    // 3. sheetFormatPr
    let drh = data.default_row_height.map(|h| format!("{h}")).unwrap_or_else(|| "15".into());
    let mut sfp_attrs: Vec<(&str, &str)> = vec![("defaultRowHeight", &drh)];
    if data.default_row_height.is_some() { sfp_attrs.push(("customHeight", "1")); }
    w.empty_tag("sheetFormatPr", &sfp_attrs);

    // 4. cols (widths + hidden + outline + col formats)
    let has_cols = !data.col_widths.is_empty() || !data.hidden_cols.is_empty() || !data.col_outline_levels.is_empty() || !data.col_formats.is_empty();
    if has_cols {
        let mut all_cols: std::collections::BTreeSet<ColNum> = std::collections::BTreeSet::new();
        for &c in data.col_widths.keys() { all_cols.insert(c); }
        for &c in data.hidden_cols { all_cols.insert(c); }
        for &c in data.col_outline_levels.keys() { all_cols.insert(c); }
        for &c in data.col_formats.keys() { all_cols.insert(c); }
        w.start_tag("cols", &[]);
        for &col in &all_cols {
            let c = (col + 1).to_string();
            let width = data.col_widths.get(&col).copied().unwrap_or(8.43);
            let ws = format!("{width:.2}");
            let mut attrs: Vec<(&str, &str)> = vec![("min", &c), ("max", &c), ("width", &ws)];
            if data.col_widths.contains_key(&col) { attrs.push(("customWidth", "1")); }
            let ol;
            if let Some(&level) = data.col_outline_levels.get(&col) {
                ol = level.to_string();
                attrs.push(("outlineLevel", &ol));
            }
            if data.hidden_cols.contains(&col) { attrs.push(("hidden", "1")); }
            let sf;
            if let Some(&xf) = data.col_formats.get(&col) {
                sf = xf.to_string();
                attrs.push(("style", &sf));
            }
            w.empty_tag("col", &attrs);
        }
        w.end_tag("cols");
    }

    // 5. sheetData
    w.start_tag("sheetData", &[]);
    for (&row, cols) in data.cells {
        let r = (row + 1).to_string();
        let mut row_attrs: Vec<(&str, &str)> = vec![("r", &r)];
        let hs;
        if let Some(&height) = data.row_heights.get(&row) {
            hs = format!("{height:.2}");
            row_attrs.push(("ht", &hs));
            row_attrs.push(("customHeight", "1"));
        }
        let ol;
        if let Some(&level) = data.row_outline_levels.get(&row) {
            ol = level.to_string();
            row_attrs.push(("outlineLevel", &ol));
        }
        if data.hidden_rows.contains(&row) { row_attrs.push(("hidden", "1")); }
        let rf;
        if let Some(&xf) = data.row_formats.get(&row) {
            rf = xf.to_string();
            row_attrs.push(("s", &rf));
            row_attrs.push(("customFormat", "1"));
        }
        w.start_tag("row", &row_attrs);
        for (&col, (cell, xf_idx)) in cols {
            write_cell(&mut w, row, col, cell, *xf_idx);
        }
        w.end_tag("row");
    }
    w.end_tag("sheetData");

    // sheetProtection
    if let Some(prot) = data.protection {
        let mut attrs: Vec<(&str, &str)> = vec![("sheet", "1")];
        let pw;
        if let Some(ref hash) = prot.password_hash {
            pw = hash.clone();
            attrs.push(("password", &pw));
        }
        if prot.objects { attrs.push(("objects", "1")); }
        if prot.scenarios { attrs.push(("scenarios", "1")); }
        if !prot.format_cells { attrs.push(("formatCells", "0")); }
        if !prot.format_columns { attrs.push(("formatColumns", "0")); }
        if !prot.format_rows { attrs.push(("formatRows", "0")); }
        if !prot.insert_columns { attrs.push(("insertColumns", "0")); }
        if !prot.insert_rows { attrs.push(("insertRows", "0")); }
        if !prot.insert_hyperlinks { attrs.push(("insertHyperlinks", "0")); }
        if !prot.delete_columns { attrs.push(("deleteColumns", "0")); }
        if !prot.delete_rows { attrs.push(("deleteRows", "0")); }
        if prot.select_locked_cells { attrs.push(("selectLockedCells", "1")); }
        if !prot.sort { attrs.push(("sort", "0")); }
        if !prot.auto_filter { attrs.push(("autoFilter", "0")); }
        if !prot.pivot_tables { attrs.push(("pivotTables", "0")); }
        if prot.select_unlocked_cells { attrs.push(("selectUnlockedCells", "1")); }
        w.empty_tag("sheetProtection", &attrs);
    }

    // protectedRanges
    if let Some(ps) = data.print_settings {
        if !ps.protected_ranges.is_empty() {
            w.start_tag("protectedRanges", &[]);
            for (name, sqref, pw_hash) in &ps.protected_ranges {
                let mut attrs: Vec<(&str, &str)> = vec![("sqref", sqref), ("name", name)];
                if let Some(h) = pw_hash { attrs.push(("password", h)); }
                w.empty_tag("protectedRange", &attrs);
            }
            w.end_tag("protectedRanges");
        }
    }

    // autoFilter
    if let Some((r1, c1, r2, c2)) = data.autofilter {
        let ref_str = format!("{}{}:{}{}", col_to_letter(c1), r1 + 1, col_to_letter(c2), r2 + 1);
        if data.autofilter_columns.is_empty() {
            w.empty_tag("autoFilter", &[("ref", &ref_str)]);
        } else {
            w.start_tag("autoFilter", &[("ref", &ref_str)]);
            for (col, values) in data.autofilter_columns {
                let col_id = (col - c1).to_string();
                w.start_tag("filterColumn", &[("colId", &col_id)]);
                w.start_tag("filters", &[]);
                for v in values {
                    w.empty_tag("filter", &[("val", v)]);
                }
                w.end_tag("filters");
                w.end_tag("filterColumn");
            }
            w.end_tag("autoFilter");
        }
    }

    // mergeCells
    if !data.merge_ranges.is_empty() {
        let count = data.merge_ranges.len().to_string();
        w.start_tag("mergeCells", &[("count", &count)]);
        for &(r1, c1, r2, c2) in data.merge_ranges {
            let ref_str = format!("{}{}:{}{}", col_to_letter(c1), r1 + 1, col_to_letter(c2), r2 + 1);
            w.empty_tag("mergeCell", &[("ref", &ref_str)]);
        }
        w.end_tag("mergeCells");
    }

    // conditionalFormatting
    for (i, cf) in data.conditional_formats.iter().enumerate() {
        let (r1, c1, r2, c2) = cf.range;
        let sqref = format!("{}{}:{}{}", col_to_letter(c1), r1 + 1, col_to_letter(c2), r2 + 1);
        w.start_tag("conditionalFormatting", &[("sqref", &sqref)]);
        cf.rule.write_rule(&mut w, (i + 1) as u32, cf.dxf_id);
        w.end_tag("conditionalFormatting");
    }

    // dataValidations
    if !data.validations.is_empty() {
        let count = data.validations.len().to_string();
        w.start_tag("dataValidations", &[("count", &count)]);
        for dv in data.validations {
            write_data_validation(&mut w, dv);
        }
        w.end_tag("dataValidations");
    }

    // hyperlinks
    if !data.hyperlinks.is_empty() {
        w.start_tag("hyperlinks", &[]);
        for (i, hl) in data.hyperlinks.iter().enumerate() {
            let cell_ref = format!("{}{}", col_to_letter(hl.col), hl.row + 1);
            if let Some(ref loc) = hl.location {
                w.empty_tag("hyperlink", &[("ref", &cell_ref), ("location", loc)]);
            } else if !hl.url.is_empty() {
                let rid = format!("rId_hl{}", i + 1);
                w.empty_tag("hyperlink", &[("ref", &cell_ref), ("r:id", &rid)]);
            }
        }
        w.end_tag("hyperlinks");
    }

    // print settings
    if let Some(ps) = data.print_settings {
        // printOptions
        let need_print_opts = ps.print_gridlines || ps.print_headings || ps.center_horizontally || ps.center_vertically;
        if need_print_opts {
            let mut po_attrs: Vec<(&str, &str)> = Vec::new();
            if ps.print_gridlines { po_attrs.push(("gridLines", "1")); }
            if ps.print_headings { po_attrs.push(("headings", "1")); }
            if ps.center_horizontally { po_attrs.push(("horizontalCentered", "1")); }
            if ps.center_vertically { po_attrs.push(("verticalCentered", "1")); }
            w.empty_tag("printOptions", &po_attrs);
        }

        // pageMargins
        let top = ps.margin_top.unwrap_or(0.75);
        let bot = ps.margin_bottom.unwrap_or(0.75);
        let left = ps.margin_left.unwrap_or(0.7);
        let right = ps.margin_right.unwrap_or(0.7);
        let hdr = ps.margin_header.unwrap_or(0.3);
        let ftr = ps.margin_footer.unwrap_or(0.3);
        let ts = format!("{top}"); let bs = format!("{bot}");
        let ls = format!("{left}"); let rs = format!("{right}");
        let hs = format!("{hdr}"); let fs = format!("{ftr}");
        w.empty_tag("pageMargins", &[("top", &ts), ("bottom", &bs), ("left", &ls), ("right", &rs), ("header", &hs), ("footer", &fs)]);

        // pageSetup
        let mut setup_attrs: Vec<(&str, String)> = Vec::new();
        if let Some(sz) = ps.paper_size { setup_attrs.push(("paperSize", sz.to_string())); }
        if let Some(Orientation::Landscape) = ps.orientation { setup_attrs.push(("orientation", "landscape".into())); }
        else if ps.orientation.is_some() { setup_attrs.push(("orientation", "portrait".into())); }
        if let Some(s) = ps.scale { setup_attrs.push(("scale", s.to_string())); }
        if ps.fit_to_page {
            if let Some(fw) = ps.fit_to_width { setup_attrs.push(("fitToWidth", fw.to_string())); }
            if let Some(fh) = ps.fit_to_height { setup_attrs.push(("fitToHeight", fh.to_string())); }
        }
        if ps.black_and_white { setup_attrs.push(("blackAndWhite", "1".into())); }
        if let Some(fpn) = ps.first_page_number {
            setup_attrs.push(("firstPageNumber", fpn.to_string()));
            setup_attrs.push(("useFirstPageNumber", "1".into()));
        }
        if !setup_attrs.is_empty() {
            let refs: Vec<(&str, &str)> = setup_attrs.iter().map(|(k, v)| (*k, v.as_str())).collect();
            w.empty_tag("pageSetup", &refs);
        }

        // headerFooter
        if ps.header.is_some() || ps.footer.is_some() {
            w.start_tag("headerFooter", &[]);
            if let Some(ref h) = ps.header { w.text_element("oddHeader", &[], h); }
            if let Some(ref f) = ps.footer { w.text_element("oddFooter", &[], f); }
            w.end_tag("headerFooter");
        }

        // rowBreaks
        if !ps.row_breaks.is_empty() {
            let count = ps.row_breaks.len().to_string();
            w.start_tag("rowBreaks", &[("count", &count), ("manualBreakCount", &count)]);
            for &rb in &ps.row_breaks {
                let id = (rb + 1).to_string();
                w.empty_tag("brk", &[("id", &id), ("max", "16383"), ("man", "1")]);
            }
            w.end_tag("rowBreaks");
        }
        if !ps.col_breaks.is_empty() {
            let count = ps.col_breaks.len().to_string();
            w.start_tag("colBreaks", &[("count", &count), ("manualBreakCount", &count)]);
            for &cb in &ps.col_breaks {
                let id = (cb + 1).to_string();
                w.empty_tag("brk", &[("id", &id), ("max", "1048575"), ("man", "1")]);
            }
            w.end_tag("colBreaks");
        }
    }

    // drawing reference
    if let Some(ref rid) = data.drawing_rid {
        // pageMargins is required before drawing in many Excel implementations
        if data.print_settings.is_none() {
            w.empty_tag("pageMargins", &[("top", "0.75"), ("bottom", "0.75"), ("left", "0.7"), ("right", "0.7"), ("header", "0.3"), ("footer", "0.3")]);
        }
        w.empty_tag("drawing", &[("r:id", rid)]);
    }

    // legacyDrawing (for comments VML)
    if let Some(ref rid) = data.legacy_drawing_rid {
        w.empty_tag("legacyDrawing", &[("r:id", rid)]);
    }

    // tableParts
    if !data.table_parts.is_empty() {
        let count = data.table_parts.len().to_string();
        w.start_tag("tableParts", &[("count", &count)]);
        for rid in &data.table_parts {
            w.empty_tag("tablePart", &[("r:id", rid)]);
        }
        w.end_tag("tableParts");
    }

    // sparklines (as extLst)
    if !data.sparklines.is_empty() {
        w.start_tag("extLst", &[]);
        w.start_tag("ext", &[("xmlns:x14", "http://schemas.microsoft.com/office/spreadsheetml/2009/9/main"), ("uri", "{05C60535-1F16-4fd2-B633-F4F36F0B64E0}")]);
        w.start_tag("x14:sparklineGroups", &[("xmlns:xm", "http://schemas.microsoft.com/office/excel/2006/main")]);
        for sp in data.sparklines {
            let sp_type = sp.sparkline_type.xml_str();
            w.start_tag("x14:sparklineGroup", &[("type", sp_type)]);
            if let Some(rgb) = sp.color {
                let hex = format!("FF{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
                w.start_tag("x14:colorSeries", &[]);
                w.empty_tag("x14:rgbColor", &[("rgb", &hex)]);
                w.end_tag("x14:colorSeries");
            }
            w.start_tag("x14:sparklines", &[]);
            w.start_tag("x14:sparkline", &[]);
            w.text_element("xm:f", &[], &sp.data_range);
            let loc = format!("{}{}", col_to_letter(sp.col), sp.row + 1);
            w.text_element("xm:sqref", &[], &loc);
            w.end_tag("x14:sparkline");
            w.end_tag("x14:sparklines");
            w.end_tag("x14:sparklineGroup");
        }
        w.end_tag("x14:sparklineGroups");
        w.end_tag("ext");
        w.end_tag("extLst");
    }

    // ignoredErrors
    if !data.ignored_errors.is_empty() {
        w.start_tag("ignoredErrors", &[]);
        for (err_type, range) in data.ignored_errors {
            w.empty_tag("ignoredError", &[("sqref", range), (err_type, "1")]);
        }
        w.end_tag("ignoredErrors");
    }

    w.end_tag("worksheet");
    w.into_bytes()
}


use std::collections::{BTreeMap, HashMap};

use crate::cell::{CellType, CellValue, IntoExcelData, RichText};
use crate::datetime::ExcelDateTime;
use crate::features::chart::Chart;
use crate::format::IntoColor;
use crate::features::conditional::{ConditionalFormat, StoredCf};
use crate::features::image::Image;
use crate::features::sparkline::Sparkline;
use crate::features::table::Table;
use crate::features::validation::DataValidation;
use crate::format::Format;
use crate::formula::adjust_formula;
use crate::model::shared_strings::SharedStringTable;
use crate::model::style_registry::StyleRegistry;
use crate::reader::sheet_reader::RawCell;
use crate::utility::{ColNum, RowNum};

/// A worksheet within a workbook. Contains cells, formatting, charts, images,
/// tables, conditional formatting, data validation, and sparklines.
/// All coordinates are 0-based: row 0 = Excel row 1, col 0 = column A.
pub struct Worksheet {
    pub(crate) name: String,
    pub(crate) cells: BTreeMap<RowNum, BTreeMap<ColNum, (CellType, u32)>>,
    pub(crate) pending_formats: HashMap<(RowNum, ColNum), Format>,
    pub(crate) merge_ranges: Vec<(RowNum, ColNum, RowNum, ColNum)>,
    pub(crate) col_widths: BTreeMap<ColNum, f64>,
    pub(crate) row_heights: BTreeMap<RowNum, f64>,
    pub(crate) freeze_row: RowNum,
    pub(crate) freeze_col: ColNum,
    pub(crate) read_cells: Option<Vec<RawCell>>,
    pub(crate) raw_xml: Option<Vec<u8>>,
    pub(crate) dirty: bool,
    // Phase 3 features
    pub(crate) charts: Vec<Chart>,
    pub(crate) images: Vec<Image>,
    pub(crate) tables: Vec<Table>,
    pub(crate) conditional_formats: Vec<StoredCf>,
    pub(crate) validations: Vec<DataValidation>,
    pub(crate) sparklines: Vec<Sparkline>,
    // Phase 4 features
    pub(crate) protection: Option<SheetProtection>,
    pub(crate) print_settings: Option<PrintSettings>,
    // Phase 5 features
    pub(crate) hidden_rows: std::collections::BTreeSet<RowNum>,
    pub(crate) hidden_cols: std::collections::BTreeSet<ColNum>,
    pub(crate) autofilter: Option<(RowNum, ColNum, RowNum, ColNum)>,
    pub(crate) hyperlinks: Vec<Hyperlink>,
    pub(crate) comments: Vec<Comment>,
    pub(crate) row_outline_levels: BTreeMap<RowNum, u8>,
    pub(crate) col_outline_levels: BTreeMap<ColNum, u8>,
    // Phase 6 features
    pub(crate) zoom: Option<u16>,
    pub(crate) show_gridlines: bool,
    pub(crate) show_headings: bool,
    pub(crate) right_to_left: bool,
    pub(crate) tab_color: Option<[u8; 3]>,
}

impl Worksheet {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            cells: BTreeMap::new(),
            pending_formats: HashMap::new(),
            merge_ranges: Vec::new(),
            col_widths: BTreeMap::new(),
            row_heights: BTreeMap::new(),
            freeze_row: 0, freeze_col: 0,
            read_cells: None, raw_xml: None, dirty: false,
            charts: Vec::new(), images: Vec::new(), tables: Vec::new(),
            conditional_formats: Vec::new(), validations: Vec::new(), sparklines: Vec::new(),
            protection: None, print_settings: None,
            hidden_rows: std::collections::BTreeSet::new(),
            hidden_cols: std::collections::BTreeSet::new(),
            autofilter: None, hyperlinks: Vec::new(), comments: Vec::new(),
            row_outline_levels: BTreeMap::new(), col_outline_levels: BTreeMap::new(),
            zoom: None, show_gridlines: true, show_headings: true,
            right_to_left: false, tab_color: None,
        }
    }

    pub fn name(&self) -> &str { &self.name }

    pub fn set_name(&mut self, name: &str) -> crate::Result<&mut Self> {
        self.name = name.to_string();
        Ok(self)
    }

    /// Ensure cells are deserialized from read_cells into the BTreeMap for editing.
    pub(crate) fn ensure_deserialized(&mut self) {
        if let Some(raw_cells) = self.read_cells.take() {
            for rc in raw_cells {
                let cell_type = cell_value_to_type(&rc.value);
                self.cells.entry(rc.row).or_default().insert(rc.col, (cell_type, rc.xf_index));
            }
        }
    }

    // ── Writing ──

    pub fn write(&mut self, row: RowNum, col: ColNum, data: impl IntoExcelData) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        data.write_cell(self, row, col)?;
        Ok(self)
    }

    pub fn write_with_format(&mut self, row: RowNum, col: ColNum, data: impl IntoExcelData, fmt: &Format) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        data.write_cell_with_format(self, row, col, fmt)?;
        Ok(self)
    }

    pub fn write_row(&mut self, row: RowNum, start_col: ColNum, data: impl IntoIterator<Item = impl IntoExcelData>) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        for (i, val) in data.into_iter().enumerate() {
            val.write_cell(self, row, start_col + i as ColNum)?;
        }
        Ok(self)
    }

    pub fn write_column(&mut self, start_row: RowNum, col: ColNum, data: impl IntoIterator<Item = impl IntoExcelData>) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        for (i, val) in data.into_iter().enumerate() {
            val.write_cell(self, start_row + i as RowNum, col)?;
        }
        Ok(self)
    }

    pub fn write_formula(&mut self, row: RowNum, col: ColNum, formula: &str) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        self.cells.entry(row).or_default().insert(col, (CellType::Formula { text: formula.to_string(), cached_number: None }, 0));
        Ok(self)
    }

    pub fn write_rich_text(&mut self, row: RowNum, col: ColNum, rich_text: &RichText) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        self.cells.entry(row).or_default().insert(col, (CellType::RichText(rich_text.clone()), 0));
        Ok(self)
    }

    // Internal write methods called by IntoExcelData impls
    pub(crate) fn write_number_internal(&mut self, row: RowNum, col: ColNum, n: f64, fmt: Option<&Format>) -> crate::Result<()> {
        self.cells.entry(row).or_default().insert(col, (CellType::Number(n), 0));
        if let Some(f) = fmt { self.pending_formats.insert((row, col), f.clone()); }
        Ok(())
    }

    pub(crate) fn write_string_internal(&mut self, row: RowNum, col: ColNum, s: &str, fmt: Option<&Format>) -> crate::Result<()> {
        let cell = if let Some(formula) = s.strip_prefix('=') {
            CellType::Formula { text: formula.to_string(), cached_number: None }
        } else {
            CellType::InlineString(s.to_string())
        };
        self.cells.entry(row).or_default().insert(col, (cell, 0));
        if let Some(f) = fmt { self.pending_formats.insert((row, col), f.clone()); }
        Ok(())
    }

    pub(crate) fn write_bool_internal(&mut self, row: RowNum, col: ColNum, b: bool, fmt: Option<&Format>) -> crate::Result<()> {
        self.cells.entry(row).or_default().insert(col, (CellType::Bool(b), 0));
        if let Some(f) = fmt { self.pending_formats.insert((row, col), f.clone()); }
        Ok(())
    }

    pub(crate) fn write_datetime_internal(&mut self, row: RowNum, col: ColNum, dt: ExcelDateTime, fmt: Option<&Format>) -> crate::Result<()> {
        self.cells.entry(row).or_default().insert(col, (CellType::DateTime(dt.serial()), 0));
        if let Some(f) = fmt { self.pending_formats.insert((row, col), f.clone()); }
        Ok(())
    }

    // ── Reading ──

    pub fn read_cell(&self, row: RowNum, col: ColNum) -> CellValue {
        if let Some(cols) = self.cells.get(&row) {
            if let Some((cell, _)) = cols.get(&col) {
                return cell_type_to_value(cell);
            }
        }
        if let Some(ref raw) = self.read_cells {
            for rc in raw {
                if rc.row == row && rc.col == col { return rc.value.clone(); }
            }
        }
        CellValue::Empty
    }

    pub fn used_range(&self) -> Option<(RowNum, ColNum, RowNum, ColNum)> {
        let mut min_r = u32::MAX; let mut max_r = 0u32;
        let mut min_c = u16::MAX; let mut max_c = 0u16;
        let mut found = false;
        for (&r, cols) in &self.cells {
            for &c in cols.keys() {
                min_r = min_r.min(r); max_r = max_r.max(r);
                min_c = min_c.min(c); max_c = max_c.max(c);
                found = true;
            }
        }
        if let Some(ref raw) = self.read_cells {
            for rc in raw {
                min_r = min_r.min(rc.row); max_r = max_r.max(rc.row);
                min_c = min_c.min(rc.col); max_c = max_c.max(rc.col);
                found = true;
            }
        }
        if found { Some((min_r, min_c, max_r, max_c)) } else { None }
    }

    // ── Formatting ──

    pub fn set_cell_format(&mut self, row: RowNum, col: ColNum, format: &Format) -> crate::Result<&mut Self> {
        self.pending_formats.insert((row, col), format.clone());
        self.dirty = true;
        Ok(self)
    }

    pub fn set_range_format(&mut self, r1: RowNum, c1: ColNum, r2: RowNum, c2: ColNum, format: &Format) -> crate::Result<&mut Self> {
        for r in r1..=r2 { for c in c1..=c2 { self.pending_formats.insert((r, c), format.clone()); } }
        self.dirty = true;
        Ok(self)
    }

    pub fn merge_range(&mut self, r1: RowNum, c1: ColNum, r2: RowNum, c2: ColNum, text: &str, format: &Format) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        self.merge_ranges.push((r1, c1, r2, c2));
        self.write_with_format(r1, c1, text, format)?;
        Ok(self)
    }

    // ── Layout ──

    pub fn set_column_width(&mut self, col: ColNum, width: f64) -> crate::Result<&mut Self> { self.col_widths.insert(col, width); self.dirty = true; Ok(self) }
    pub fn set_row_height(&mut self, row: RowNum, height: f64) -> crate::Result<&mut Self> { self.row_heights.insert(row, height); self.dirty = true; Ok(self) }
    pub fn set_freeze_panes(&mut self, row: RowNum, col: ColNum) -> crate::Result<&mut Self> { self.freeze_row = row; self.freeze_col = col; self.dirty = true; Ok(self) }

    /// Automatically set column widths based on cell content.
    /// Estimates character widths using a simple heuristic (1 char ≈ 1.1 units, min 8, max 64).
    pub fn autofit(&mut self) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        let mut max_widths: BTreeMap<ColNum, f64> = BTreeMap::new();
        for cols in self.cells.values() {
            for (&col, (cell, _)) in cols {
                let len = match cell {
                    CellType::Number(n) => format!("{n}").len(),
                    CellType::InlineString(s) => s.len(),
                    CellType::SharedString(_) => 8,
                    CellType::Bool(_) => 5,
                    CellType::Formula { text, .. } => text.len().min(20),
                    CellType::DateTime(_) => 10,
                    CellType::Error(e) => e.len(),
                    CellType::RichText(rt) => rt.plain_text().len(),
                    CellType::Empty => 0,
                };
                let width = (len as f64 * 1.1 + 2.0).max(8.0).min(64.0);
                let entry = max_widths.entry(col).or_insert(8.0);
                if width > *entry { *entry = width; }
            }
        }
        for (col, w) in max_widths {
            self.col_widths.insert(col, w);
        }
        self.dirty = true;
        Ok(self)
    }

    // ── Charts ──

    pub fn insert_chart(&mut self, row: RowNum, col: ColNum, chart: &Chart) -> crate::Result<&mut Self> {
        let mut c = chart.clone();
        c.row = row;
        c.col = col;
        self.charts.push(c);
        self.dirty = true;
        Ok(self)
    }

    // ── Images ──

    pub fn insert_image(&mut self, row: RowNum, col: ColNum, image: &Image) -> crate::Result<&mut Self> {
        let mut img = image.clone();
        img.row = row;
        img.col = col;
        self.images.push(img);
        self.dirty = true;
        Ok(self)
    }

    // ── Tables ──

    pub fn add_table(&mut self, first_row: RowNum, first_col: ColNum, last_row: RowNum, last_col: ColNum, table: &Table) -> crate::Result<&mut Self> {
        let mut t = table.clone();
        t.first_row = first_row;
        t.first_col = first_col;
        t.last_row = last_row;
        t.last_col = last_col;
        self.tables.push(t);
        self.dirty = true;
        Ok(self)
    }

    // ── Conditional Formatting ──

    pub fn add_conditional_format(&mut self, r1: RowNum, c1: ColNum, r2: RowNum, c2: ColNum, cf: impl ConditionalFormat + 'static) -> crate::Result<&mut Self> {
        self.conditional_formats.push(StoredCf { range: (r1, c1, r2, c2), rule: Box::new(cf) });
        self.dirty = true;
        Ok(self)
    }

    // ── Data Validation ──

    pub fn add_data_validation(&mut self, r1: RowNum, c1: ColNum, r2: RowNum, c2: ColNum, dv: &DataValidation) -> crate::Result<&mut Self> {
        let mut v = dv.clone();
        v.first_row = r1; v.first_col = c1; v.last_row = r2; v.last_col = c2;
        self.validations.push(v);
        self.dirty = true;
        Ok(self)
    }

    // ── Sparklines ──

    pub fn add_sparkline(&mut self, row: RowNum, col: ColNum, sparkline: &Sparkline) -> crate::Result<&mut Self> {
        let mut s = sparkline.clone();
        s.row = row;
        s.col = col;
        self.sparklines.push(s);
        self.dirty = true;
        Ok(self)
    }

    // ── Protection ──

    pub fn protect(&mut self) -> &mut Self {
        self.protection = Some(SheetProtection::default());
        self.dirty = true;
        self
    }

    pub fn protect_with_password(&mut self, password: &str) -> &mut Self {
        let mut prot = SheetProtection::default();
        prot.password_hash = Some(hash_password(password));
        self.protection = Some(prot);
        self.dirty = true;
        self
    }

    // ── Print Settings ──

    pub fn set_print_settings(&mut self, settings: &PrintSettings) -> &mut Self {
        self.print_settings = Some(settings.clone());
        self.dirty = true;
        self
    }

    pub fn set_page_breaks(&mut self, row_breaks: &[RowNum], col_breaks: &[ColNum]) -> &mut Self {
        let ps = self.print_settings.get_or_insert_with(PrintSettings::default);
        ps.row_breaks = row_breaks.to_vec();
        ps.col_breaks = col_breaks.to_vec();
        self.dirty = true;
        self
    }

    // ── Hidden Rows/Columns ──

    pub fn set_row_hidden(&mut self, row: RowNum, hidden: bool) -> &mut Self {
        if hidden { self.hidden_rows.insert(row); } else { self.hidden_rows.remove(&row); }
        self.dirty = true;
        self
    }

    pub fn set_column_hidden(&mut self, col: ColNum, hidden: bool) -> &mut Self {
        if hidden { self.hidden_cols.insert(col); } else { self.hidden_cols.remove(&col); }
        self.dirty = true;
        self
    }

    // ── Auto-filter ──

    pub fn set_autofilter(&mut self, r1: RowNum, c1: ColNum, r2: RowNum, c2: ColNum) -> &mut Self {
        self.autofilter = Some((r1, c1, r2, c2));
        self.dirty = true;
        self
    }

    // ── Hyperlinks ──

    pub fn write_url(&mut self, row: RowNum, col: ColNum, url: &str, text: &str) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        self.write_string_internal(row, col, if text.is_empty() { url } else { text }, None)?;
        self.hyperlinks.push(Hyperlink { row, col, url: url.to_string(), location: None, tooltip: None });
        Ok(self)
    }

    pub fn write_internal_link(&mut self, row: RowNum, col: ColNum, location: &str, text: &str) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        self.write_string_internal(row, col, text, None)?;
        self.hyperlinks.push(Hyperlink { row, col, url: String::new(), location: Some(location.to_string()), tooltip: None });
        Ok(self)
    }

    // ── Comments ──

    pub fn add_comment(&mut self, row: RowNum, col: ColNum, text: &str) -> &mut Self {
        self.comments.push(Comment { row, col, text: text.to_string(), author: "Author".to_string() });
        self.dirty = true;
        self
    }

    pub fn add_comment_with_author(&mut self, row: RowNum, col: ColNum, text: &str, author: &str) -> &mut Self {
        self.comments.push(Comment { row, col, text: text.to_string(), author: author.to_string() });
        self.dirty = true;
        self
    }

    // ── Row/Column Grouping ──

    pub fn group_rows(&mut self, start: RowNum, end: RowNum, level: u8) -> &mut Self {
        for r in start..=end { self.row_outline_levels.insert(r, level); }
        self.dirty = true;
        self
    }

    pub fn group_columns(&mut self, start: ColNum, end: ColNum, level: u8) -> &mut Self {
        for c in start..=end { self.col_outline_levels.insert(c, level); }
        self.dirty = true;
        self
    }

    // ── Print Area / Repeat Rows ──

    pub fn set_print_area(&mut self, r1: RowNum, c1: ColNum, r2: RowNum, c2: ColNum) -> &mut Self {
        let ps = self.print_settings.get_or_insert_with(PrintSettings::default);
        ps.print_area = Some((r1, c1, r2, c2));
        self.dirty = true;
        self
    }

    pub fn set_repeat_rows(&mut self, first: RowNum, last: RowNum) -> &mut Self {
        let ps = self.print_settings.get_or_insert_with(PrintSettings::default);
        ps.repeat_rows = Some((first, last));
        self.dirty = true;
        self
    }

    // ── Phase 6: View / Print ──

    pub fn set_zoom(&mut self, percent: u16) -> &mut Self { self.zoom = Some(percent.clamp(10, 400)); self }
    pub fn hide_gridlines(&mut self) -> &mut Self { self.show_gridlines = false; self }
    pub fn hide_headings(&mut self) -> &mut Self { self.show_headings = false; self }
    pub fn set_right_to_left(&mut self) -> &mut Self { self.right_to_left = true; self }
    pub fn set_tab_color(&mut self, c: impl IntoColor) -> &mut Self { self.tab_color = Some(c.into_color().to_rgb()); self }

    pub fn set_repeat_columns(&mut self, first: ColNum, last: ColNum) -> &mut Self {
        let ps = self.print_settings.get_or_insert_with(PrintSettings::default);
        ps.repeat_cols = Some((first, last));
        self.dirty = true;
        self
    }

    pub fn set_print_scale(&mut self, percent: u16) -> &mut Self {
        let ps = self.print_settings.get_or_insert_with(PrintSettings::default);
        ps.scale = Some(percent.clamp(10, 400));
        self.dirty = true;
        self
    }

    // ── Row/Column operations ──

    /// Insert `count` rows at the given 0-based row index. Shifts existing rows down.
    pub fn insert_rows(&mut self, at_row: RowNum, count: u32) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;

        // Shift cells: iterate in reverse to avoid overwriting
        let mut new_cells = BTreeMap::new();
        for (&r, cols) in &self.cells {
            let new_r = if r >= at_row { r + count } else { r };
            new_cells.insert(new_r, cols.clone());
        }
        self.cells = new_cells;

        // Shift merge ranges
        for m in &mut self.merge_ranges {
            if m.0 >= at_row { m.0 += count; }
            if m.2 >= at_row { m.2 += count; }
        }

        // Shift row heights
        let mut new_heights = BTreeMap::new();
        for (&r, &h) in &self.row_heights {
            let new_r = if r >= at_row { r + count } else { r };
            new_heights.insert(new_r, h);
        }
        self.row_heights = new_heights;

        // Adjust formulas
        self.adjust_formulas_for_row_insert(at_row, count as i64);
        Ok(self)
    }

    /// Remove `count` rows starting at the given 0-based row index.
    pub fn remove_rows(&mut self, at_row: RowNum, count: u32) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;

        let end_row = at_row + count;
        let mut new_cells = BTreeMap::new();
        for (&r, cols) in &self.cells {
            if r >= at_row && r < end_row { continue; } // deleted
            let new_r = if r >= end_row { r - count } else { r };
            new_cells.insert(new_r, cols.clone());
        }
        self.cells = new_cells;

        // Shift merge ranges (remove those fully in deleted range)
        self.merge_ranges.retain(|m| !(m.0 >= at_row && m.2 < end_row));
        for m in &mut self.merge_ranges {
            if m.0 >= end_row { m.0 -= count; }
            if m.2 >= end_row { m.2 -= count; }
        }

        let mut new_heights = BTreeMap::new();
        for (&r, &h) in &self.row_heights {
            if r >= at_row && r < end_row { continue; }
            let new_r = if r >= end_row { r - count } else { r };
            new_heights.insert(new_r, h);
        }
        self.row_heights = new_heights;

        self.adjust_formulas_for_row_insert(at_row, -(count as i64));
        Ok(self)
    }

    /// Insert `count` columns at the given 0-based column index.
    pub fn insert_columns(&mut self, at_col: ColNum, count: u16) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;

        for cols in self.cells.values_mut() {
            let mut new_cols = BTreeMap::new();
            for (&c, v) in cols.iter() {
                let new_c = if c >= at_col { c + count } else { c };
                new_cols.insert(new_c, v.clone());
            }
            *cols = new_cols;
        }

        for m in &mut self.merge_ranges {
            if m.1 >= at_col { m.1 += count; }
            if m.3 >= at_col { m.3 += count; }
        }

        let mut new_widths = BTreeMap::new();
        for (&c, &w) in &self.col_widths {
            let new_c = if c >= at_col { c + count } else { c };
            new_widths.insert(new_c, w);
        }
        self.col_widths = new_widths;

        self.adjust_formulas_for_col_insert(at_col, count as i64);
        Ok(self)
    }

    /// Remove `count` columns starting at the given 0-based column index.
    pub fn remove_columns(&mut self, at_col: ColNum, count: u16) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        let end_col = at_col + count;

        for cols in self.cells.values_mut() {
            let mut new_cols = BTreeMap::new();
            for (&c, v) in cols.iter() {
                if c >= at_col && c < end_col { continue; }
                let new_c = if c >= end_col { c - count } else { c };
                new_cols.insert(new_c, v.clone());
            }
            *cols = new_cols;
        }

        self.merge_ranges.retain(|m| !(m.1 >= at_col && m.3 < end_col));
        for m in &mut self.merge_ranges {
            if m.1 >= end_col { m.1 -= count; }
            if m.3 >= end_col { m.3 -= count; }
        }

        let mut new_widths = BTreeMap::new();
        for (&c, &w) in &self.col_widths {
            if c >= at_col && c < end_col { continue; }
            let new_c = if c >= end_col { c - count } else { c };
            new_widths.insert(new_c, w);
        }
        self.col_widths = new_widths;

        self.adjust_formulas_for_col_insert(at_col, -(count as i64));
        Ok(self)
    }

    fn adjust_formulas_for_row_insert(&mut self, at_row: RowNum, delta: i64) {
        for cols in self.cells.values_mut() {
            for (cell, _) in cols.values_mut() {
                if let CellType::Formula { text, .. } = cell {
                    *text = adjust_formula(text, Some(at_row), delta, None, 0);
                }
            }
        }
    }

    fn adjust_formulas_for_col_insert(&mut self, at_col: ColNum, delta: i64) {
        for cols in self.cells.values_mut() {
            for (cell, _) in cols.values_mut() {
                if let CellType::Formula { text, .. } = cell {
                    *text = adjust_formula(text, None, 0, Some(at_col), delta);
                }
            }
        }
    }

    // ── Finalization ──

    pub(crate) fn finalize(&mut self, sst: &mut SharedStringTable, styles: &mut StyleRegistry) {
        // If we have read_cells that were never deserialized into cells, do it now
        self.ensure_deserialized();

        let pending = std::mem::take(&mut self.pending_formats);
        for ((row, col), fmt) in pending {
            let xf = styles.register_format(&fmt);
            self.cells.entry(row).or_default().entry(col)
                .and_modify(|e| e.1 = xf)
                .or_insert((CellType::Empty, xf));
        }
        for cols in self.cells.values_mut() {
            for (cell, _) in cols.values_mut() {
                if let CellType::InlineString(s) = cell {
                    let idx = sst.intern(s);
                    *cell = CellType::SharedString(idx);
                }
            }
        }
        for cols in self.cells.values_mut() {
            for (cell, xf) in cols.values_mut() {
                if matches!(cell, CellType::DateTime(_)) && *xf == 0 {
                    *xf = styles.register_format(&Format::new().num_format("yyyy-mm-dd"));
                }
            }
        }
    }
}

fn cell_type_to_value(cell: &CellType) -> CellValue {
    match cell {
        CellType::Empty => CellValue::Empty,
        CellType::Number(n) => CellValue::Number(*n),
        CellType::SharedString(_) => CellValue::Empty,
        CellType::InlineString(s) => CellValue::String(s.clone()),
        CellType::Bool(b) => CellValue::Bool(*b),
        CellType::Formula { text, cached_number } => CellValue::Formula {
            formula: text.clone(),
            cached_value: Box::new(cached_number.map(CellValue::Number).unwrap_or(CellValue::Empty)),
        },
        CellType::DateTime(s) => CellValue::DateTime(ExcelDateTime::new(*s, false)),
        CellType::Error(e) => CellValue::Error(e.clone()),
        CellType::RichText(rt) => CellValue::RichText(rt.clone()),
    }
}

fn cell_value_to_type(value: &CellValue) -> CellType {
    match value {
        CellValue::Empty => CellType::Empty,
        CellValue::String(s) => CellType::InlineString(s.clone()),
        CellValue::Number(n) => CellType::Number(*n),
        CellValue::Bool(b) => CellType::Bool(*b),
        CellValue::DateTime(dt) => CellType::DateTime(dt.serial()),
        CellValue::Error(e) => CellType::Error(e.clone()),
        CellValue::Formula { formula, cached_value } => CellType::Formula {
            text: formula.clone(),
            cached_number: match cached_value.as_ref() {
                CellValue::Number(n) => Some(*n),
                _ => None,
            },
        },
        CellValue::RichText(rt) => CellType::RichText(rt.clone()),
    }
}

/// Sheet protection settings.
#[derive(Debug, Clone)]
pub struct SheetProtection {
    pub(crate) password_hash: Option<String>,
    pub sheet: bool,
    pub objects: bool,
    pub scenarios: bool,
    pub format_cells: bool,
    pub format_columns: bool,
    pub format_rows: bool,
    pub insert_columns: bool,
    pub insert_rows: bool,
    pub insert_hyperlinks: bool,
    pub delete_columns: bool,
    pub delete_rows: bool,
    pub select_locked_cells: bool,
    pub sort: bool,
    pub auto_filter: bool,
    pub pivot_tables: bool,
    pub select_unlocked_cells: bool,
}

impl Default for SheetProtection {
    fn default() -> Self {
        Self {
            password_hash: None,
            sheet: true, objects: true, scenarios: true,
            format_cells: true, format_columns: true, format_rows: true,
            insert_columns: true, insert_rows: true, insert_hyperlinks: true,
            delete_columns: true, delete_rows: true,
            select_locked_cells: false, sort: true, auto_filter: true,
            pivot_tables: true, select_unlocked_cells: false,
        }
    }
}

/// Print settings for a worksheet.
#[derive(Debug, Clone, Default)]
pub struct PrintSettings {
    pub paper_size: Option<u8>,
    pub orientation: Option<Orientation>,
    pub fit_to_page: bool,
    pub fit_to_width: Option<u16>,
    pub fit_to_height: Option<u16>,
    pub scale: Option<u16>,
    pub margin_top: Option<f64>,
    pub margin_bottom: Option<f64>,
    pub margin_left: Option<f64>,
    pub margin_right: Option<f64>,
    pub margin_header: Option<f64>,
    pub margin_footer: Option<f64>,
    pub header: Option<String>,
    pub footer: Option<String>,
    pub row_breaks: Vec<RowNum>,
    pub col_breaks: Vec<ColNum>,
    pub print_area: Option<(RowNum, ColNum, RowNum, ColNum)>,
    pub repeat_rows: Option<(RowNum, RowNum)>,
    pub repeat_cols: Option<(ColNum, ColNum)>,
}

impl PrintSettings {
    pub fn new() -> Self { Self::default() }
    pub fn paper_size(mut self, size: u8) -> Self { self.paper_size = Some(size); self }
    pub fn orientation(mut self, o: Orientation) -> Self { self.orientation = Some(o); self }
    pub fn fit_to_page(mut self, width: u16, height: u16) -> Self {
        self.fit_to_page = true;
        self.fit_to_width = Some(width);
        self.fit_to_height = Some(height);
        self
    }
    pub fn margins(mut self, top: f64, bottom: f64, left: f64, right: f64) -> Self {
        self.margin_top = Some(top); self.margin_bottom = Some(bottom);
        self.margin_left = Some(left); self.margin_right = Some(right);
        self
    }
    pub fn header(mut self, h: &str) -> Self { self.header = Some(h.to_string()); self }
    pub fn footer(mut self, f: &str) -> Self { self.footer = Some(f.to_string()); self }
}

#[derive(Debug, Clone, Copy)]
pub enum Orientation { Portrait, Landscape }

/// Excel legacy password hash (XOR-based, 16-bit).
fn hash_password(password: &str) -> String {
    let bytes = password.as_bytes();
    let mut hash: u16 = 0;
    for (i, &b) in bytes.iter().rev().enumerate() {
        let mut val = b as u16;
        val ^= (i + 1) as u16;
        // Rotate left by 1 within 15 bits
        val = ((val >> 14) & 1) | ((val << 1) & 0x7FFF);
        hash ^= val;
    }
    hash ^= bytes.len() as u16;
    hash ^= 0xCE4B;
    format!("{hash:04X}")
}

pub(crate) fn hash_password_public(password: &str) -> String {
    hash_password(password)
}

/// A hyperlink on a cell.
#[derive(Debug, Clone)]
pub struct Hyperlink {
    pub row: RowNum,
    pub col: ColNum,
    pub url: String,
    pub location: Option<String>, // internal link like "Sheet2!A1"
    pub tooltip: Option<String>,
}

/// A comment/note on a cell.
#[derive(Debug, Clone)]
pub struct Comment {
    pub row: RowNum,
    pub col: ColNum,
    pub text: String,
    pub author: String,
}

use std::collections::{BTreeMap, HashMap};

use crate::cell::{CellType, CellValue, IntoExcelData};
use crate::datetime::ExcelDateTime;
use crate::features::chart::Chart;
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
    }
}

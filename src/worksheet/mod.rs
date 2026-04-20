//! Worksheet module — split into logical submodules.

mod write;
mod features;
mod layout;
mod ops;
pub mod types;

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use crate::cell::{CellType, CellValue};
use crate::datetime::ExcelDateTime;
use crate::features::chart::Chart;
use crate::features::conditional::StoredCf;
use crate::features::image::Image;
use crate::features::sparkline::Sparkline;
use crate::features::table::Table;
use crate::features::validation::DataValidation;
use crate::format::Format;
use crate::model::shared_strings::SharedStringTable;
use crate::model::style_registry::StyleRegistry;
use crate::reader::sheet_reader::RawCell;
use crate::reader::style_parser::ParsedStyles;
use crate::utility::{ColNum, RowNum};

pub use types::{Comment, Hyperlink, Orientation, PrintSettings, SheetProtection, SheetVisibility};
pub(crate) use types::{hash_password_public, validate_sheet_name};

/// A worksheet within a workbook. Contains cells, formatting, charts, images,
/// tables, conditional formatting, data validation, and sparklines.
/// All coordinates are 0-based: row 0 = Excel row 1, col 0 = column A.
pub struct Worksheet {
    pub(crate) name: String,
    pub(crate) cells: BTreeMap<RowNum, BTreeMap<ColNum, (CellType, u32)>>,
    pub(crate) pending_formats: HashMap<(RowNum, ColNum), Format>,
    pub(crate) range_formats: Vec<(RowNum, ColNum, RowNum, ColNum, Format)>,
    pub(crate) merge_ranges: Vec<(RowNum, ColNum, RowNum, ColNum)>,
    pub(crate) col_widths: BTreeMap<ColNum, f64>,
    pub(crate) row_heights: BTreeMap<RowNum, f64>,
    pub(crate) freeze_row: RowNum,
    pub(crate) freeze_col: ColNum,
    pub(crate) read_cells: Option<Vec<RawCell>>,
    pub(crate) read_cells_map: Option<BTreeMap<(RowNum, ColNum), CellValue>>,
    pub(crate) raw_xml: Option<Vec<u8>>,
    pub(crate) original_rels: Option<Vec<u8>>,
    pub(crate) original_drawing_rid: Option<String>,
    pub(crate) original_legacy_drawing_rid: Option<String>,
    pub(crate) dirty: bool,
    pub(crate) charts: Vec<Chart>,
    pub(crate) images: Vec<Image>,
    pub(crate) tables: Vec<Table>,
    pub(crate) conditional_formats: Vec<StoredCf>,
    pub(crate) validations: Vec<DataValidation>,
    pub(crate) sparklines: Vec<Sparkline>,
    pub(crate) pivot_tables: Vec<crate::features::pivot::PivotTable>,
    pub(crate) treemap_charts: Vec<crate::features::treemap::TreemapChart>,
    pub(crate) chartex_charts: Vec<crate::features::chartex::ChartExChart>,
    pub(crate) protection: Option<SheetProtection>,
    pub(crate) print_settings: Option<PrintSettings>,
    pub(crate) hidden_rows: std::collections::BTreeSet<RowNum>,
    pub(crate) hidden_cols: std::collections::BTreeSet<ColNum>,
    pub(crate) autofilter: Option<(RowNum, ColNum, RowNum, ColNum)>,
    pub(crate) hyperlinks: Vec<Hyperlink>,
    pub(crate) comments: Vec<Comment>,
    pub(crate) row_outline_levels: BTreeMap<RowNum, u8>,
    pub(crate) col_outline_levels: BTreeMap<ColNum, u8>,
    pub(crate) zoom: Option<u16>,
    pub(crate) show_gridlines: bool,
    pub(crate) show_headings: bool,
    pub(crate) right_to_left: bool,
    pub(crate) tab_color: Option<[u8; 3]>,
    pub(crate) visibility: SheetVisibility,
    pub(crate) selection: Option<(RowNum, ColNum)>,
    pub(crate) top_left_cell: Option<(RowNum, ColNum)>,
    pub(crate) default_row_height: Option<f64>,
    pub(crate) col_formats: BTreeMap<ColNum, Format>,
    pub(crate) row_formats: BTreeMap<RowNum, Format>,
    pub(crate) ignored_errors: Vec<(String, String)>,
    pub(crate) autofilter_columns: Vec<(ColNum, Vec<String>)>,
    pub(crate) parsed_styles: Option<Arc<ParsedStyles>>,
}

impl Worksheet {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            cells: BTreeMap::new(),
            pending_formats: HashMap::new(),
            range_formats: Vec::new(),
            merge_ranges: Vec::new(),
            col_widths: BTreeMap::new(),
            row_heights: BTreeMap::new(),
            freeze_row: 0, freeze_col: 0,
            read_cells: None, read_cells_map: None, raw_xml: None, original_rels: None,
            original_drawing_rid: None, original_legacy_drawing_rid: None, dirty: false,
            charts: Vec::new(), images: Vec::new(), tables: Vec::new(),
            conditional_formats: Vec::new(), validations: Vec::new(), sparklines: Vec::new(),
            pivot_tables: Vec::new(),
            treemap_charts: Vec::new(),
            chartex_charts: Vec::new(),
            protection: None, print_settings: None,
            hidden_rows: std::collections::BTreeSet::new(),
            hidden_cols: std::collections::BTreeSet::new(),
            autofilter: None, hyperlinks: Vec::new(), comments: Vec::new(),
            row_outline_levels: BTreeMap::new(), col_outline_levels: BTreeMap::new(),
            zoom: None, show_gridlines: true, show_headings: true,
            right_to_left: false, tab_color: None,
            visibility: SheetVisibility::Visible,
            selection: None, top_left_cell: None,
            default_row_height: None,
            col_formats: BTreeMap::new(), row_formats: BTreeMap::new(),
            ignored_errors: Vec::new(),
            autofilter_columns: Vec::new(),
            parsed_styles: None,
        }
    }

    // ── Read accessors ──

    pub fn name(&self) -> &str { &self.name }
    pub fn visibility(&self) -> SheetVisibility { self.visibility }
    pub fn is_hidden(&self) -> bool { self.visibility == SheetVisibility::Hidden }
    pub fn is_very_hidden(&self) -> bool { self.visibility == SheetVisibility::VeryHidden }
    pub fn merge_ranges(&self) -> &[(RowNum, ColNum, RowNum, ColNum)] { &self.merge_ranges }
    pub fn column_width(&self, col: ColNum) -> Option<f64> { self.col_widths.get(&col).copied() }
    pub fn row_height(&self, row: RowNum) -> Option<f64> { self.row_heights.get(&row).copied() }
    pub fn charts(&self) -> &[Chart] { &self.charts }
    pub fn tables(&self) -> &[Table] { &self.tables }
    pub fn treemap_charts(&self) -> &[crate::features::treemap::TreemapChart] { &self.treemap_charts }
    pub fn comments(&self) -> &[Comment] { &self.comments }
    pub fn conditional_formats(&self) -> &[StoredCf] { &self.conditional_formats }
    pub fn validations(&self) -> &[DataValidation] { &self.validations }
    pub fn sparklines(&self) -> &[Sparkline] { &self.sparklines }
    pub fn hyperlinks(&self) -> &[Hyperlink] { &self.hyperlinks }
    pub fn print_settings(&self) -> Option<&PrintSettings> { self.print_settings.as_ref() }
    pub fn protection(&self) -> Option<&SheetProtection> { self.protection.as_ref() }
    pub fn row_outline_levels(&self) -> &BTreeMap<RowNum, u8> { &self.row_outline_levels }
    pub fn col_outline_levels(&self) -> &BTreeMap<ColNum, u8> { &self.col_outline_levels }
    pub fn get_comment(&self, row: RowNum, col: ColNum) -> Option<(&str, &str)> {
        self.comments.iter().find(|c| c.row == row && c.col == col).map(|c| (c.author.as_str(), c.text.as_str()))
    }

    pub fn set_name(&mut self, name: &str) -> crate::Result<&mut Self> {
        validate_sheet_name(name)?;
        self.name = name.to_string();
        Ok(self)
    }

    pub(crate) fn ensure_deserialized(&mut self) {
        if let Some(raw_cells) = self.read_cells.take() {
            for rc in raw_cells {
                let cell_type = cell_value_to_type(&rc.value);
                self.cells.entry(rc.row).or_default().insert(rc.col, (cell_type, rc.xf_index));
            }
        }
    }

    pub fn read_cell(&self, row: RowNum, col: ColNum) -> CellValue {
        if let Some(cols) = self.cells.get(&row) {
            if let Some((cell, _)) = cols.get(&col) {
                return cell_type_to_value(cell);
            }
        }
        if let Some(ref map) = self.read_cells_map {
            if let Some(rc) = map.get(&(row, col)) {
                return rc.clone();
            }
        }
        CellValue::Empty
    }

    /// Return the resolved `Format` for the cell at `(row, col)`, if one exists.
    ///
    /// This looks up the cell's xf index (style index from the `s` attribute in
    /// the sheet XML) and resolves it through the parsed styles to produce a
    /// fully-populated `Format`.  Returns `None` when:
    /// - the cell does not exist,
    /// - the cell has the default style (xf index 0), or
    /// - the workbook was not opened from a file (no parsed styles available).
    pub fn cell_format(&self, row: RowNum, col: ColNum) -> Option<Format> {
        let styles = self.parsed_styles.as_ref()?;

        // First check deserialized cells (BTreeMap)
        if let Some(cols) = self.cells.get(&row) {
            if let Some((_, xf_index)) = cols.get(&col) {
                let idx = *xf_index as usize;
                if idx == 0 {
                    return None;
                }
                return styles.resolve_format(idx);
            }
        }

        // Fall back to raw cells that haven't been deserialized yet
        if let Some(ref raw_cells) = self.read_cells {
            for rc in raw_cells {
                if rc.row == row && rc.col == col {
                    let idx = rc.xf_index as usize;
                    if idx == 0 {
                        return None;
                    }
                    return styles.resolve_format(idx);
                }
            }
        }

        None
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
        if let Some(ref map) = self.read_cells_map {
            for &(r, c) in map.keys() {
                min_r = min_r.min(r); max_r = max_r.max(r);
                min_c = min_c.min(c); max_c = max_c.max(c);
                found = true;
            }
        }
        if found { Some((min_r, min_c, max_r, max_c)) } else { None }
    }

    // ── Formatting ──

    pub fn set_cell_format(&mut self, row: RowNum, col: ColNum, format: &Format) -> crate::Result<&mut Self> {
        self.pending_formats.insert((row, col), format.clone());
        self.dirty = true; Ok(self)
    }

    pub fn set_range_format(&mut self, r1: RowNum, c1: ColNum, r2: RowNum, c2: ColNum, format: &Format) -> crate::Result<&mut Self> {
        self.range_formats.push((r1, c1, r2, c2, format.clone()));
        self.dirty = true; Ok(self)
    }

    pub fn merge_range(&mut self, r1: RowNum, c1: ColNum, r2: RowNum, c2: ColNum, text: &str, format: &Format) -> crate::Result<&mut Self> {
        self.ensure_deserialized();
        self.dirty = true;
        self.merge_ranges.push((r1, c1, r2, c2));
        self.write_with_format(r1, c1, text, format)?;
        Ok(self)
    }

    // ── Finalization ──

    pub(crate) fn finalize(&mut self, sst: &mut SharedStringTable, styles: &mut StyleRegistry) {
        self.ensure_deserialized();

        let pending = std::mem::take(&mut self.pending_formats);
        for ((row, col), fmt) in pending {
            let xf = styles.register_format(&fmt);
            self.cells.entry(row).or_default().entry(col)
                .and_modify(|e| e.1 = xf)
                .or_insert((CellType::Empty, xf));
        }
        let ranges = std::mem::take(&mut self.range_formats);
        for (r1, c1, r2, c2, fmt) in ranges {
            let xf = styles.register_format(&fmt);
            for r in r1..=r2 {
                if let Some(cols) = self.cells.get_mut(&r) {
                    for c in c1..=c2 {
                        if let Some(cell) = cols.get_mut(&c) { cell.1 = xf; }
                    }
                }
            }
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
        CellType::ArrayFormula { text, .. } | CellType::DynamicFormula { text, .. } => CellValue::Formula {
            formula: text.clone(), cached_value: Box::new(CellValue::Empty),
        },
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

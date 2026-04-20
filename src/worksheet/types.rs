// Types: SheetProtection, PrintSettings, SheetVisibility, Orientation, Hyperlink, Comment, helpers

use crate::utility::{ColNum, RowNum};

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

impl SheetProtection {
    /// Returns the legacy password hash, if set.
    pub fn password_hash(&self) -> Option<&str> {
        self.password_hash.as_deref()
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
    pub print_gridlines: bool,
    pub print_headings: bool,
    pub center_horizontally: bool,
    pub center_vertically: bool,
    pub black_and_white: bool,
    pub first_page_number: Option<u16>,
    pub(crate) protected_ranges: Vec<(String, String, Option<String>)>,
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
    pub fn print_gridlines(mut self, v: bool) -> Self { self.print_gridlines = v; self }
    pub fn print_headings(mut self, v: bool) -> Self { self.print_headings = v; self }
    pub fn center_horizontally(mut self, v: bool) -> Self { self.center_horizontally = v; self }
    pub fn center_vertically(mut self, v: bool) -> Self { self.center_vertically = v; self }
    pub fn black_and_white(mut self, v: bool) -> Self { self.black_and_white = v; self }
    pub fn first_page_number(mut self, n: u16) -> Self { self.first_page_number = Some(n); self }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SheetVisibility { #[default] Visible, Hidden, VeryHidden }

#[derive(Debug, Clone, Copy)]
pub enum Orientation { Portrait, Landscape }

/// A hyperlink on a cell.
#[derive(Debug, Clone)]
pub struct Hyperlink {
    pub row: RowNum,
    pub col: ColNum,
    pub url: String,
    pub location: Option<String>,
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

/// Excel legacy password hash (XOR-based, 16-bit).
pub(crate) fn hash_password(password: &str) -> String {
    let bytes = password.as_bytes();
    let mut hash: u16 = 0;
    for (i, &b) in bytes.iter().rev().enumerate() {
        let mut val = b as u16;
        val ^= (i + 1) as u16;
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

/// Validate an Excel sheet name.
pub(crate) fn validate_sheet_name(name: &str) -> crate::Result<()> {
    if name.is_empty() {
        return Err(crate::Error::InvalidData("Sheet name cannot be empty".into()));
    }
    if name.len() > 31 {
        return Err(crate::Error::InvalidData(format!("Sheet name '{}' exceeds 31 characters", name)));
    }
    for c in name.chars() {
        if matches!(c, '/' | '\\' | '*' | '?' | '[' | ']' | ':') {
            return Err(crate::Error::InvalidData(format!("Sheet name '{}' contains invalid character '{}'", name, c)));
        }
    }
    Ok(())
}

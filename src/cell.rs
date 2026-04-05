use crate::datetime::ExcelDateTime;
use crate::format::Format;
use crate::utility::{ColNum, RowNum};
use crate::worksheet::Worksheet;

/// What you get back when reading a cell.
#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    Empty,
    String(String),
    Number(f64),
    Bool(bool),
    DateTime(ExcelDateTime),
    Error(String),
    Formula { formula: String, cached_value: Box<CellValue> },
    RichText(RichText),
}

impl CellValue {
    pub fn is_empty(&self) -> bool { matches!(self, CellValue::Empty) }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            CellValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            CellValue::Number(n) => Some(*n),
            CellValue::DateTime(dt) => Some(dt.serial()),
            _ => None,
        }
    }
}

/// Internal cell type stored in the worksheet.
#[derive(Debug, Clone)]
pub(crate) enum CellType {
    Empty,
    Number(f64),
    SharedString(u32),   // index into SST
    InlineString(String),
    Bool(bool),
    Formula { text: String, cached_number: Option<f64> },
    ArrayFormula { text: String, range: String },
    DynamicFormula { text: String, range: String },
    DateTime(f64),       // serial date
    Error(String),
    RichText(RichText),
}

/// Trait for types that can be written to a cell.
pub trait IntoExcelData {
    fn write_cell(self, ws: &mut Worksheet, row: RowNum, col: ColNum) -> crate::Result<()>;
    fn write_cell_with_format(self, ws: &mut Worksheet, row: RowNum, col: ColNum, fmt: &Format) -> crate::Result<()>;
}

macro_rules! impl_into_excel_number {
    ($($t:ty),*) => {
        $(
            impl IntoExcelData for $t {
                fn write_cell(self, ws: &mut Worksheet, row: RowNum, col: ColNum) -> crate::Result<()> {
                    ws.write_number_internal(row, col, self as f64, None)
                }
                fn write_cell_with_format(self, ws: &mut Worksheet, row: RowNum, col: ColNum, fmt: &Format) -> crate::Result<()> {
                    ws.write_number_internal(row, col, self as f64, Some(fmt))
                }
            }
        )*
    };
}

impl_into_excel_number!(f64, f32, i8, i16, i32, i64, u8, u16, u32, u64);

impl IntoExcelData for &str {
    fn write_cell(self, ws: &mut Worksheet, row: RowNum, col: ColNum) -> crate::Result<()> {
        ws.write_string_internal(row, col, self, None)
    }
    fn write_cell_with_format(self, ws: &mut Worksheet, row: RowNum, col: ColNum, fmt: &Format) -> crate::Result<()> {
        ws.write_string_internal(row, col, self, Some(fmt))
    }
}

impl IntoExcelData for String {
    fn write_cell(self, ws: &mut Worksheet, row: RowNum, col: ColNum) -> crate::Result<()> {
        ws.write_string_internal(row, col, &self, None)
    }
    fn write_cell_with_format(self, ws: &mut Worksheet, row: RowNum, col: ColNum, fmt: &Format) -> crate::Result<()> {
        ws.write_string_internal(row, col, &self, Some(fmt))
    }
}

impl IntoExcelData for bool {
    fn write_cell(self, ws: &mut Worksheet, row: RowNum, col: ColNum) -> crate::Result<()> {
        ws.write_bool_internal(row, col, self, None)
    }
    fn write_cell_with_format(self, ws: &mut Worksheet, row: RowNum, col: ColNum, fmt: &Format) -> crate::Result<()> {
        ws.write_bool_internal(row, col, self, Some(fmt))
    }
}

impl IntoExcelData for ExcelDateTime {
    fn write_cell(self, ws: &mut Worksheet, row: RowNum, col: ColNum) -> crate::Result<()> {
        ws.write_datetime_internal(row, col, self, None)
    }
    fn write_cell_with_format(self, ws: &mut Worksheet, row: RowNum, col: ColNum, fmt: &Format) -> crate::Result<()> {
        ws.write_datetime_internal(row, col, self, Some(fmt))
    }
}

/// Rich text: multiple styled runs in a single cell.
#[derive(Debug, Clone, PartialEq)]
pub struct RichText {
    pub runs: Vec<RichTextRun>,
}

impl RichText {
    pub fn new() -> Self { Self { runs: Vec::new() } }

    pub fn add_run(mut self, text: &str) -> Self {
        self.runs.push(RichTextRun { text: text.to_string(), bold: false, italic: false, font_size: None, font_name: None, color: None, superscript: false, subscript: false });
        self
    }

    pub fn add_bold(mut self, text: &str) -> Self {
        self.runs.push(RichTextRun { text: text.to_string(), bold: true, italic: false, font_size: None, font_name: None, color: None, superscript: false, subscript: false });
        self
    }

    pub fn add_italic(mut self, text: &str) -> Self {
        self.runs.push(RichTextRun { text: text.to_string(), bold: false, italic: true, font_size: None, font_name: None, color: None, superscript: false, subscript: false });
        self
    }

    pub fn add_styled(mut self, text: &str, run: RichTextRun) -> Self {
        let mut r = run;
        r.text = text.to_string();
        self.runs.push(r);
        self
    }

    /// Get the plain text content (all runs concatenated).
    pub fn plain_text(&self) -> String {
        self.runs.iter().map(|r| r.text.as_str()).collect()
    }
}

impl Default for RichText {
    fn default() -> Self { Self::new() }
}

/// A single styled run within rich text.
#[derive(Debug, Clone, PartialEq)]
pub struct RichTextRun {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub font_size: Option<f64>,
    pub font_name: Option<String>,
    pub color: Option<String>,
    pub superscript: bool,
    pub subscript: bool,
}

impl RichTextRun {
    pub fn new() -> Self {
        Self { text: String::new(), bold: false, italic: false, font_size: None, font_name: None, color: None, superscript: false, subscript: false }
    }
    pub fn bold(mut self) -> Self { self.bold = true; self }
    pub fn italic(mut self) -> Self { self.italic = true; self }
    pub fn font_size(mut self, size: f64) -> Self { self.font_size = Some(size); self }
    pub fn font_name(mut self, name: &str) -> Self { self.font_name = Some(name.to_string()); self }
    pub fn color(mut self, hex: &str) -> Self { self.color = Some(hex.to_string()); self }
    pub fn superscript(mut self) -> Self { self.superscript = true; self }
    pub fn subscript(mut self) -> Self { self.subscript = true; self }
}

impl Default for RichTextRun {
    fn default() -> Self { Self::new() }
}

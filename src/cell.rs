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
    DateTime(f64),       // serial date
    Error(String),
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

//! Traits for the `#[derive(ExcelRow)]` proc macro.

use crate::utility::RowNum;
use crate::worksheet::Worksheet;

/// Write struct fields as Excel row data.
pub trait ExcelRowWriter {
    /// Write column headers to the specified row.
    fn write_header(&self, ws: &mut Worksheet, row: RowNum) -> crate::Result<()>;
    /// Write field values to the specified row.
    fn write_row(&self, ws: &mut Worksheet, row: RowNum) -> crate::Result<()>;
}

/// Read an Excel row into a struct instance.
pub trait ExcelRowReader: Sized {
    /// Read a row from the worksheet and construct a struct instance.
    /// `headers` maps column positions to header names.
    fn read_row(ws: &Worksheet, row: RowNum, headers: &[String]) -> crate::Result<Self>;
}

//! # zavora-xlsx
//!
//! High-performance Rust crate for reading, writing, and editing Excel `.xlsx` files.
//!
//! ## Quick Start
//! ```no_run
//! use zavora_xlsx::{Workbook, Format};
//!
//! let mut wb = Workbook::new();
//! let ws = wb.worksheet(0).unwrap();
//! ws.write(0, 0, "Hello").unwrap();
//! ws.write(0, 1, 42.5).unwrap();
//! wb.save("output.xlsx").unwrap();
//! ```

pub mod cell;
pub mod datetime;
pub mod error;
pub mod features;
pub mod format;
pub mod formula;
pub mod properties;
pub mod streaming;
pub mod utility;
pub mod workbook;
pub mod worksheet;

mod model;
mod reader;
mod writer;
pub(crate) mod xml;
mod zip;

pub use cell::{CellValue, IntoExcelData, RichText, RichTextRun};
pub use datetime::ExcelDateTime;
pub use error::{Error, Result};
pub use features::chart::{Chart, ChartSeries, ChartType, LegendPosition};
pub use features::conditional::{
    CfOperator, ConditionalFormat, ConditionalFormat2ColorScale, ConditionalFormat3ColorScale,
    ConditionalFormatCell, ConditionalFormatDataBar, ConditionalFormatIconSet, IconSetType,
};
pub use features::image::Image;
pub use features::sparkline::{Sparkline, SparklineType};
pub use features::table::{Table, TableColumn, TableStyle};
pub use features::validation::{DataValidation, ErrorStyle, ValidationRule};
pub use format::{Align, BorderStyle, Color, Format, IntoColor, NamedColor, Pattern, Underline};
pub use properties::DocProperties;
pub use utility::{ColNum, RowNum};
pub use workbook::Workbook;
pub use worksheet::{Orientation, PrintSettings, SheetProtection, Worksheet};
pub use streaming::StreamingWorkbook;

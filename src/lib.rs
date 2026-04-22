//! # zavora-xlsx
//!
//! High-performance Rust crate for reading, writing, and editing Excel `.xlsx` files.
//!
//! ## Features
//!
//! - **Unified**: Read, write, and edit in one crate — no separate reader/writer dependencies
//! - **Fast**: Minimal dependencies, constant-memory streaming mode for large files
//! - **Excel-compatible**: Validated against Microsoft Excel with zero repair errors
//! - **Ergonomic**: Builder patterns, `impl IntoExcelData`, chainable methods
//!
//! ## Quick Start
//!
//! ```no_run
//! use zavora_xlsx::{Workbook, Format};
//!
//! let mut wb = Workbook::new();
//! let ws = wb.worksheet(0).unwrap();
//! ws.write(0, 0, "Hello").unwrap();
//! ws.write(0, 1, 42.5).unwrap();
//! wb.save("output.xlsx").unwrap();
//! ```
//!
//! ## Feature Flags
//!
//! | Flag | Description |
//! |------|-------------|
//! | `mmap` | Memory-mapped file I/O via `memmap2` |
//! | `serde-support` | Serialize/deserialize rows via `serde` |
//! | `async-tokio` | Async file I/O via `tokio` |
//! | `wasm` | WASM target support |
//! | `cffi` | C FFI bindings |
//! | `xlsb` | XLSB format reading (experimental) |
//! | `xls` | Legacy XLS format reading (experimental) |
//! | `ods` | ODS format support (experimental) |
//! | `crypto` | Encrypted workbook support (experimental) |

// ── Core modules ──────────────────────────────────────────────────────────────

pub mod cell;
pub mod datetime;
pub mod derive_traits;
pub mod error;
pub mod format;
pub mod formula;
pub mod formula_engine;
pub mod properties;
pub mod streaming;
pub mod utility;

// ── Workbook & worksheet ──────────────────────────────────────────────────────

pub mod workbook;
pub mod worksheet;

// ── Features (charts, tables, conditional formatting, etc.) ───────────────────

pub mod features;

// ── I/O ───────────────────────────────────────────────────────────────────────

pub mod crypto;
pub mod formats;
mod model;
pub mod reader;
pub mod writer;
pub(crate) mod xml;
mod zip;

// ── Optional modules (behind feature flags) ───────────────────────────────────

#[cfg(feature = "serde-support")]
pub mod serde_support;

#[cfg(feature = "async-tokio")]
pub mod async_io;

#[cfg(feature = "cffi")]
pub mod cffi;

/// WASM target support.
///
/// When targeting `wasm32-unknown-unknown`, file-system operations are not
/// available. Use [`Workbook::save_to_buffer()`] / [`Workbook::open_from_buffer()`] instead.
/// The `wasm` feature flag enables `wasm-bindgen` exports for the buffer API.
pub mod wasm_support {
    /// Check whether we are running on a WASM target.
    pub const fn is_wasm() -> bool {
        cfg!(target_arch = "wasm32")
    }
}

// ── Public re-exports ─────────────────────────────────────────────────────────

// Cell types
pub use cell::{CellValue, IntoExcelData, RichText, RichTextRun};
pub use datetime::ExcelDateTime;
pub use derive_traits::{ExcelRowReader, ExcelRowWriter};
pub use error::{Error, Result};
pub use utility::{ColNum, RowNum};

// Formatting
pub use format::{
    Align, BorderStyle, Color, DiagonalType, Format, GradientFill, GradientStop, IntoColor,
    NamedColor, Pattern, ThemeColor, ThemeColorIndex, Underline,
};

// Workbook & worksheet
pub use properties::DocProperties;
pub use properties::{CustomProperty, CustomPropertyValue};
pub use workbook::{
    CalcMode, ChartSheet, DefinedName, DefinedNameScope, Workbook, WorkbookContentType,
    WorkbookProtection,
};
pub use worksheet::{
    AdvancedFilterColumn, Comment, FilterRule, FormControl, Hyperlink, Orientation, PhoneticRun,
    PrintSettings, SheetProtection, SheetVisibility, SortCondition, SortDirection, SortState,
    ThreadedComment, ThreadedCommentReply, Worksheet,
};

// Charts
pub use features::chart::{
    AxisFormat, Chart, ChartSeries, ChartType, DashStyle, ErrorBar, ErrorBarType,
    ErrorBarValueType, GridlineStyle, LegendPosition, MapLevel, MarkerType, PivotChartSeriesData,
    PivotChartSource, PlotAreaFormat, TickMark, TrendlineType, View3D,
};
pub use features::chartex::{
    BoxWhiskerChart, ChartExChart, FunnelChart, HistogramChart, MapChart, SunburstChart,
    SunburstLevel, WaterfallChart, WaterfallPointType,
};
pub use features::treemap::TreemapChart;

// Conditional formatting
pub use features::conditional::{
    AverageType, CfOperator, ConditionalFormat, ConditionalFormat2ColorScale,
    ConditionalFormat3ColorScale, ConditionalFormatAverage, ConditionalFormatCell,
    ConditionalFormatDataBar, ConditionalFormatDate, ConditionalFormatDuplicate,
    ConditionalFormatFormula, ConditionalFormatIconSet, ConditionalFormatText,
    ConditionalFormatTopBottom, ConditionalFormatUnique, DateOccurring, IconSetType, StoredCf,
    TextOperator, TopBottomType,
};

// Tables, validation, images, shapes, sparklines, slicers, timelines, pivots
pub use features::image::{Image, ImageType};
pub use features::pivot::{
    CalculatedItem, DateGroupLevel, FieldGrouping, PivotAggregation, PivotLayout, PivotStyle,
    PivotTable, PivotValueField,
};
pub use features::shape::{Shape, ShapeType};
pub use features::slicer::Slicer;
pub use features::sparkline::{Sparkline, SparklineType};
pub use features::table::{
    CustomTableStyle, Table, TableColumn, TableStyle, TableStyleElementFormat,
};
pub use features::timeline::{Timeline, TimelineLevel};
pub use features::validation::{DataValidation, ErrorStyle, ValidationRule};

// Streaming & reading
pub use reader::streaming_reader::{
    SheetRowIter, SheetRows, StreamingCell, StreamingReader, StreamingRow, StreamingSheetInfo,
};
pub use streaming::StreamingWorkbook;

// Formula engine
pub use formula_engine::tokenize;

// Format support
pub use formats::csv_export::CsvOptions;
pub use formats::strict_ooxml;

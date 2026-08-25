# Changelog

All notable changes to zavora-xlsx will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] — 2026-08-25

### Fixed

- Preserve edits to cells that already contained a parsed value when saving an
  opened workbook.
- Preserve workbook formatting and chart placement across read-edit-save
  round trips.
- Upgrade the XML parser and Python bindings to patched releases, including
  correct handling of entity-reference events introduced by `quick-xml` 0.41.

### Added

- Expose parsed chart metadata and frozen-pane locations to callers.

## [0.1.1] — 2025-04-22

### Added

- `Workbook::recalculate()` — evaluate all formula cells across all worksheets using the built-in formula engine. Builds a dependency graph, detects circular references, evaluates in topological order, and re-evaluates volatile functions (`RAND`, `NOW`, `TODAY`, `INDIRECT`) and their dependents. Returns the number of cells evaluated.
- `examples/formula_recalculate.rs` — demonstrates building a financial model with formulas and recalculating programmatically.

## [0.1.0] — 2025-04-22

Initial public release.

### Core library

#### Writing
- Create workbooks with `Workbook::new()`, save to file or buffer
- Cell types: string, number, boolean, formula (regular, array, dynamic), date/time, rich text
- Batch writing: `write_row`, `write_column`
- Formatted empty cells (`write_blank`), clear cells
- Merge ranges with text and formatting

#### Reading
- Open files: `open`, `open_readonly`, `open_from_buffer`, `open_readonly_from_buffer`
- Read cell values via `CellValue` enum (String, Number, Bool, DateTime, Formula, Error, RichText, Empty)
- Read metadata: sheet names, visibility, merge ranges, column widths, row heights, freeze panes
- Read formatting: `cell_format()` returns parsed `Format` with font, borders, alignment, number format
- Read charts, tables, comments, sparklines, conditional formats, validations, hyperlinks, slicers, timelines
- Streaming reader (`StreamingReader`) for large files — SAX-style row-by-row iteration
- Extract embedded images via `Workbook::pictures()`

#### Formatting
- Font: bold, italic, underline (Single/Double), strikethrough, size, name, color
- Background/foreground colors, 18 pattern fills, gradient fills with multi-stop support
- Borders: all sides, individual sides, individual side colors, diagonal borders (up/down/both)
- Alignment: 5 horizontal + 3 vertical modes, text wrap, shrink to fit, indent, rotation
- Number formats (Excel format strings), cell lock/unlock, formula hidden, quote prefix
- Theme colors with tint adjustment, named cell styles
- Text effects: shadow, outline, emboss, engrave

#### Charts
- 10 standard types: Column, Bar, Line, Pie, Scatter, Area, Doughnut, Radar, Stock, Bubble
- 7 3D types: Column3D, Bar3D, Line3D, Pie3D, Area3D, Surface, WireframeSurface
- 7 ChartEx types (Excel 2016+): Waterfall, Funnel, Sunburst, Histogram, BoxWhisker, Map, Treemap
- Series: values, categories, name, color, point colors, data labels, markers, line width, dash style, gradient, error bars
- Trendlines: Linear, Exponential, Polynomial, Power, Logarithmic, MovingAverage
- Combo charts with chart type override per series and secondary axis
- Chart configuration: title, axis names, legend position, size, data table, axis min/max/log/reverse
- 3D view settings, chart styles (1–48), axis formatting, plot area formatting
- Pivot charts, chart sheets, drop lines, high-low lines
- Accessibility: alt text for charts, images, tables

#### Tables
- Table with columns, 60+ built-in styles (Light/Medium/Dark), custom table styles
- Total row with aggregation functions (sum, count, average, min, max, etc.)
- Autofilter with column criteria, advanced filters (Top10, DateFilter, CustomFilter)
- Sort state with ascending/descending direction
- Table name, alt text

#### Conditional formatting
- 11 rule types: cell value, 2-color scale, 3-color scale, data bars (solid + gradient), icon sets, formula, top/bottom N, text rules, duplicates, uniques, above/below average, date occurring
- All rules support DXF formatting

#### Data validation
- 7 rule types: list, list range, whole number, decimal, date range, text length, custom formula
- Input messages, error messages with Stop/Warning/Information styles

#### Images
- PNG and JPEG with auto dimension detection
- Resize by pixel or scale factor
- Alt text for accessibility

#### Pivot tables
- Row, column, value, and filter fields
- 11 aggregation functions
- Calculated fields and calculated items
- Date grouping (years, quarters, months, days) and numeric range grouping
- Styles, layout modes (Compact, Outline, Tabular), subtotal control, grand totals

#### Sparklines
- Line, Column, Win/Loss types with optional color

#### Other features
- Streaming write mode (`StreamingWorkbook`) for 100K+ rows with constant memory
- Edit mode: open → modify → save with VBA/macro passthrough
- Insert/remove rows and columns with automatic formula reference shifting
- Row/column grouping (outline levels), hidden rows/columns
- Print settings: orientation, paper size, margins, headers/footers, page breaks, repeat rows/columns, print area, scale, fit-to-page
- Sheet and workbook protection with password hashing, unprotect ranges, per-feature flags
- Defined names (workbook and sheet scoped) with CRUD operations
- Document properties (core + custom), custom XML parts
- Comments (legacy + threaded), hyperlinks (external + internal)
- Form controls: checkbox, dropdown, button, spinner
- Drawing shapes: rectangle, rounded rectangle, ellipse, triangle, diamond, arrow, callout, text box
- Slicers and timelines for tables and pivot tables
- CSV/TSV export with configurable delimiter, quote, line ending, date format
- Formula engine: tokenizer, parser, evaluator with 60+ functions (math, text, date, lookup, logical, statistical)
- Freeze panes, zoom, hide gridlines/headings, right-to-left, tab colors, active sheet, selection

#### Feature flags
- `serde-support` — serialize/deserialize structs to rows via serde
- `async-tokio` — async file I/O via tokio
- `mmap` — memory-mapped reading via memmap2
- `wasm` — WASM target support
- `cffi` — C FFI bindings

### Ecosystem crates

#### zavora-xlsx-derive v0.1.0
- `#[derive(ExcelRow)]` proc macro generating `ExcelRowWriter` and `ExcelRowReader` implementations
- Attributes: `#[excel(header = "...")]`, `#[excel(format = "...")]`, `#[excel(skip)]`
- Supports String, numeric types, bool, `Option<T>`
- Column-order-independent reading via header name matching
- Compile-time validation with descriptive error messages

#### zavora-xlsx-cli v0.1.0
- `inspect` — display sheet names, dimensions, document properties
- `export` — export worksheet to CSV/TSV with configurable delimiter, sheet selection, date format
- `convert` — convert between xlsx, xlsm, and CSV formats with automatic type detection

#### zavora-xlsx-node v0.1.0
- Node.js native addon via napi-rs
- Classes: Workbook, Worksheet, Format, Chart, Table
- Cell read/write with proper CellValue → JS type mapping
- Chainable Format builder, 8 chart types, table configuration
- CSV export, document properties, layout methods

#### zavora-xlsx-python v0.1.0
- Python extension module via PyO3 + maturin
- Classes: Workbook, Worksheet, Format, Chart, Table
- Idiomatic Python API: snake_case, dict-based properties/options, bytes I/O
- Error mapping: Io→OSError, SheetNotFound→IndexError, InvalidData→ValueError
- Chainable Format builder via PyRefMut pattern

[0.1.1]: https://github.com/zavora-ai/zavora-xlsx/releases/tag/v0.1.1
[0.1.2]: https://github.com/zavora-ai/zavora-xlsx/compare/v0.1.1...v0.1.2
[0.1.0]: https://github.com/zavora-ai/zavora-xlsx/releases/tag/v0.1.0

# Implementation Plan: node-bindings

## Overview

Build `zavora-xlsx-node`, a Node.js native addon wrapping the `zavora-xlsx` Rust library via napi-rs v2. The implementation proceeds incrementally: scaffold the crate, implement error conversion and value-type wrappers (Format, Chart, Table), then the shared-state wrappers (Workbook, Worksheet), followed by feature methods (layout, CSV export), and finally integration tests. Each task builds on the previous, ensuring no orphaned code.

## Tasks

- [x] 1. Scaffold the zavora-xlsx-node crate
  - [x] 1.1 Create crate directory and Cargo.toml
    - Create `zavora-xlsx-node/Cargo.toml` with `crate-type = ["cdylib"]`, dependencies on `napi` (v2, feature `napi4`), `napi-derive` (v2), and `zavora-xlsx` (path `..`)
    - Add `napi-build` (v2) as a build dependency
    - _Requirements: 1.1, 1.2, 1.3, 1.4_
  - [x] 1.2 Create build.rs and package.json
    - Create `zavora-xlsx-node/build.rs` calling `napi_build::setup()`
    - Create `zavora-xlsx-node/package.json` with napi configuration for the addon
    - _Requirements: 1.3, 1.5_
  - [x] 1.3 Create lib.rs with module declarations
    - Create `zavora-xlsx-node/src/lib.rs` declaring modules: `error`, `workbook`, `worksheet`, `format`, `chart`, `table`
    - _Requirements: 1.1_
  - [x] 1.4 Add zavora-xlsx-node to workspace members
    - Update root `Cargo.toml` workspace members to include `zavora-xlsx-node`
    - _Requirements: 1.1_

- [x] 2. Implement error conversion module
  - [x] 2.1 Create error.rs with to_napi_error and IntoNapi trait
    - Implement `to_napi_error(e: zavora_xlsx::Error) -> napi::Error` using `Status::GenericFailure` and `format!("{e}")`
    - Implement `IntoNapi<T>` trait for `Result<T, zavora_xlsx::Error>` with `into_napi()` method
    - _Requirements: 11.1, 11.2_
  - [ ]* 2.2 Write property test for error message non-emptiness
    - **Property 10: Rust errors produce non-empty JS error messages**
    - **Validates: Requirements 11.1, 11.2**

- [x] 3. Implement Format class
  - [x] 3.1 Create format.rs with Format struct and constructor
    - Define `Format` napi class wrapping `RefCell<zavora_xlsx::Format>`
    - Implement `#[napi(constructor)] fn new()` returning default format
    - _Requirements: 6.1_
  - [x] 3.2 Implement chainable builder methods on Format
    - Implement all builder methods: `bold`, `italic`, `underline(style)`, `strikethrough`, `fontSize(size)`, `fontName(name)`, `fontColor(hex)`, `backgroundColor(hex)`, `numFormat(format)`, `border(style)`, `align(alignment)`, `textWrap`, `shrinkToFit`, `indent(level)`, `rotation(angle)`
    - Each method borrows `RefCell` mutably, applies the change, returns `&Self` for JS chaining
    - Implement string-to-enum mappings for border styles (`"none"`, `"thin"`, `"medium"`, `"thick"`, `"double"`, `"dashed"`, `"dotted"`), underline styles (`"single"`, `"double"`, `"singleAccounting"`, `"doubleAccounting"`), and alignment (`"left"`, `"center"`, `"right"`, `"fill"`, `"justify"`, `"top"`, `"middle"`, `"bottom"`)
    - _Requirements: 6.2, 6.3_
  - [ ]* 3.3 Write property test for Format builder chainability
    - **Property 7: Format builder methods are chainable**
    - **Validates: Requirements 6.2, 6.3**

- [x] 4. Implement Chart class
  - [x] 4.1 Create chart.rs with Chart struct and constructor
    - Define `Chart` napi class wrapping `RefCell<zavora_xlsx::Chart>`
    - Implement `#[napi(constructor)] fn new(chart_type: String)` with string-to-`ChartType` mapping for 8 types: `"bar"`, `"column"`, `"line"`, `"pie"`, `"scatter"`, `"area"`, `"doughnut"`, `"radar"`
    - Return `napi::Error` with `InvalidArg` status for unrecognized strings
    - _Requirements: 7.1, 7.2, 7.5_
  - [x] 4.2 Implement Chart configuration methods
    - Implement `addSeries(values, categories, name)`, `setTitle(title)`, `setXAxisName(name)`, `setYAxisName(name)`, `setSize(width, height)`, `setLegendPosition(position)`
    - Implement string-to-`LegendPosition` mapping: `"bottom"`, `"top"`, `"left"`, `"right"`, `"none"`
    - Each method returns `&Self` for chaining
    - _Requirements: 7.3_
  - [ ]* 4.3 Write property test for invalid chart type rejection
    - **Property 8: Invalid chart type string throws**
    - **Validates: Requirements 7.5**

- [x] 5. Implement Table class
  - [x] 5.1 Create table.rs with Table struct and constructor
    - Define `Table` napi class wrapping `RefCell<zavora_xlsx::Table>`
    - Implement `#[napi(constructor)] fn new()` returning default table (autofilter enabled)
    - _Requirements: 8.1_
  - [x] 5.2 Implement Table configuration methods
    - Implement `setColumns(columns)` accepting `Vec<TableColumnJs>` with `name`, `totalLabel`, `totalFunction` fields
    - Implement `setStyle(style)` with string-to-`TableStyle` parsing (`"TableStyleLight1"` → `TableStyle::Light(1)`, etc.)
    - Implement `setTotalRow(enabled)`, `setAutofilter(enabled)`, `setName(name)`
    - Each method returns `&Self` for chaining
    - _Requirements: 8.2, 8.4_

- [x] 6. Checkpoint - Verify value types compile
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Implement Workbook class
  - [x] 7.1 Create workbook.rs with Workbook struct and constructors
    - Define `Workbook` napi class wrapping `Arc<Mutex<zavora_xlsx::Workbook>>`
    - Implement `#[napi(constructor)] fn new()` creating empty workbook
    - Implement `#[napi(factory)] fn open(path: String)` loading from file path
    - Implement `#[napi(factory)] fn open_from_buffer(buffer: Buffer)` loading from buffer
    - Use `IntoNapi` trait for error conversion on all fallible operations
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  - [x] 7.2 Implement Workbook save methods
    - Implement `save(path: String)` writing workbook to file
    - Implement `saveToBuffer()` returning `Buffer` with xlsx data
    - _Requirements: 2.6, 2.7, 2.8_
  - [x] 7.3 Implement Workbook worksheet access methods
    - Implement `worksheet(index: u32)` returning `Worksheet` (Arc clone + index)
    - Implement `addWorksheet()` and `addWorksheetWithName(name: String)` returning `Worksheet`
    - Implement `sheetNames()` returning `Vec<String>` and `sheetCount()` returning `u32`
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_
  - [x] 7.4 Implement Workbook document properties methods
    - Define `DocPropertiesJs` napi object with optional fields: `title`, `author`, `subject`, `description`, `keywords`, `category`, `company`
    - Implement `setProperties(props: DocPropertiesJs)` and `properties()` methods
    - _Requirements: 10.1, 10.2_
  - [ ]* 7.5 Write property tests for Workbook
    - **Property 2: Buffer save/load round-trip**
    - **Property 3: addWorksheet increments sheet count**
    - **Property 9: Document properties round-trip**
    - **Validates: Requirements 2.3, 2.7, 3.3, 3.6, 10.1, 10.2**

- [x] 8. Implement Worksheet class - cell operations
  - [x] 8.1 Create worksheet.rs with Worksheet struct
    - Define `Worksheet` napi class holding `Arc<Mutex<zavora_xlsx::Workbook>>` and `usize` index
    - Implement helper method to lock the mutex and access the worksheet by index, returning `napi::Error` on lock failure or out-of-bounds
    - _Requirements: 3.1_
  - [x] 8.2 Implement cell write methods
    - Implement `writeString(row, col, value, format?)`, `writeNumber(row, col, value, format?)`, `writeBoolean(row, col, value, format?)`, `writeFormula(row, col, formula, format?)`, `writeBlank(row, col, format)`
    - Accept optional `&Format` parameter, extract inner `RefCell` borrow to pass to core API
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_
  - [x] 8.3 Implement cell read methods
    - Implement `readCell(row, col)` returning `JsUnknown` mapped from `CellValue`: String→string, Number→number, Bool→boolean, Empty→null, Formula→`{formula, cachedValue}` object
    - Implement `usedRange()` returning `Option<UsedRangeResult>` with `firstRow`, `firstCol`, `lastRow`, `lastCol`
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7_
  - [x] 8.4 Implement worksheet name methods
    - Implement `name()` returning current sheet name as string
    - Implement `setName(name: String)` renaming the worksheet
    - _Requirements: 13.1, 13.2, 13.3_
  - [ ]* 8.5 Write property tests for cell round-trip and worksheet operations
    - **Property 1: Cell write/read round-trip**
    - **Property 4: Worksheet name preservation on creation**
    - **Property 5: Worksheet rename round-trip**
    - **Property 6: usedRange encompasses all written cells**
    - **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 5.1, 5.2, 5.3, 5.5, 5.6, 3.4, 3.5, 13.1, 13.2**

- [x] 9. Checkpoint - Verify core Workbook/Worksheet functionality
  - Ensure all tests pass, ask the user if questions arise.

- [x] 10. Implement Worksheet layout and feature methods
  - [x] 10.1 Implement layout methods
    - Implement `setColumnWidth(col, width)`, `setRowHeight(row, height)`, `setFreezePanes(row, col)`, `mergeRange(r1, c1, r2, c2, text, format?)`, `autofit()`, `setZoom(percent)`
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6_
  - [x] 10.2 Implement chart and table insertion methods
    - Implement `insertChart(row, col, chart)` extracting `RefCell<Chart>` borrow and passing to core API
    - Implement `addTable(firstRow, firstCol, lastRow, lastCol, table)` extracting `RefCell<Table>` borrow
    - _Requirements: 7.4, 8.3_

- [x] 11. Implement CSV export
  - [x] 11.1 Implement toCsvString method on Worksheet
    - Define `CsvOptionsJs` napi object with optional fields: `delimiter`, `quote`, `lineEnding`, `dateFormat`
    - Implement `toCsvString(options?)` converting JS options to `CsvOptions`, calling core `to_csv_string()`
    - Convert `delimiter` and `quote` strings to `u8` by taking first byte, error on empty strings
    - _Requirements: 12.1, 12.2, 12.3_
  - [ ]* 11.2 Write property test for CSV export completeness
    - **Property 11: CSV export contains all written cell values**
    - **Validates: Requirements 12.1**

- [x] 12. Checkpoint - Verify all Rust code compiles and property tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 13. Write JavaScript integration tests
  - [ ]* 13.1 Create test infrastructure and workbook lifecycle tests
    - Create `zavora-xlsx-node/__test__/index.spec.mjs` with test runner setup
    - Write tests for: `new Workbook()`, `Workbook.open()`, `workbook.save()`, `workbook.saveToBuffer()`, `Workbook.openFromBuffer()`
    - Test error cases: invalid file path, invalid buffer
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8_
  - [ ]* 13.2 Write worksheet access and cell operation tests
    - Test `worksheet(0)`, `addWorksheet()`, `addWorksheetWithName()`, `sheetNames()`, `sheetCount()`
    - Test write/read round-trip for each cell type (string, number, boolean, formula, blank)
    - Test `usedRange()` on populated and empty worksheets
    - Test worksheet `name()` and `setName()`
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 13.1, 13.2, 13.3_
  - [ ]* 13.3 Write format, chart, table, layout, and CSV tests
    - Test Format chaining: `new Format().bold().italic().fontSize(14)`
    - Test Chart creation for each type, `addSeries`, `setTitle`, `insertChart`
    - Test invalid chart type throws error
    - Test Table creation, `setColumns`, `setStyle`, `addTable`
    - Test layout methods: `setColumnWidth`, `setRowHeight`, `setFreezePanes`, `mergeRange`, `autofit`, `setZoom`
    - Test CSV export: default options, custom delimiter, empty worksheet
    - Test document properties: `setProperties`, `properties`
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 7.1, 7.2, 7.3, 7.4, 7.5, 8.1, 8.2, 8.3, 8.4, 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 10.1, 10.2, 12.1, 12.2, 12.3_

- [x] 14. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests (Rust, using `proptest`) validate universal correctness properties from the design
- JavaScript integration tests validate end-to-end behavior through the napi boundary
- The `Arc<Mutex<>>` ownership model is critical — Workbook and Worksheet share state, while Format/Chart/Table are independent value types

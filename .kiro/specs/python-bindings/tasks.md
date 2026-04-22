# Implementation Plan: python-bindings

## Overview

Build `zavora-xlsx-python`, a Python extension module wrapping the `zavora-xlsx` Rust library via PyO3 v0.22 and maturin. The implementation proceeds incrementally: scaffold the crate, implement error conversion and value-type wrappers (Format, Chart, Table), then the shared-state wrappers (Workbook, Worksheet), followed by feature methods (layout, CSV export), and finally integration tests. Each task builds on the previous, ensuring no orphaned code.

## Tasks

- [x] 1. Scaffold the zavora-xlsx-python crate
  - [x] 1.1 Create crate directory and Cargo.toml
    - Create `zavora-xlsx-python/Cargo.toml` with `crate-type = ["cdylib"]`, dependencies on `pyo3` (v0.22, feature `extension-module`), and `zavora-xlsx` (path `..`)
    - _Requirements: 1.1, 1.2, 1.3_
  - [x] 1.2 Create pyproject.toml
    - Create `zavora-xlsx-python/pyproject.toml` with maturin build backend and `requires = ["maturin>=1.0"]`
    - _Requirements: 1.4, 1.5_
  - [x] 1.3 Create lib.rs with module declarations
    - Create `zavora-xlsx-python/src/lib.rs` with `#[pymodule] fn zavora_xlsx` registering classes, and `mod` declarations for: `error`, `workbook`, `worksheet`, `format`, `chart`, `table`
    - Create stub files for each module so the crate compiles
    - _Requirements: 1.1, 1.5_
  - [x] 1.4 Add zavora-xlsx-python to workspace members
    - Update root `Cargo.toml` workspace members to include `zavora-xlsx-python`
    - _Requirements: 1.1_

- [x] 2. Implement error conversion module
  - [x] 2.1 Create error.rs with to_py_err and IntoPyResult trait
    - Implement `to_py_err(e: zavora_xlsx::Error) -> PyErr` mapping `Error::Io` → `PyOSError`, `Error::SheetNotFound` → `PyIndexError`, `Error::InvalidData` → `PyValueError`, all others → `PyRuntimeError`
    - Implement `IntoPyResult<T>` trait for `Result<T, zavora_xlsx::Error>` with `into_pyresult()` method
    - _Requirements: 11.1, 11.2, 11.4, 11.5_
  - [ ]* 2.2 Write property test for error message non-emptiness
    - **Property 9: Rust errors produce non-empty Python exception messages**
    - **Validates: Requirements 2.4, 2.5, 11.1, 11.2**

- [x] 3. Implement Format class
  - [x] 3.1 Create format.rs with Format pyclass and constructor
    - Define `#[pyclass] Format` wrapping `zavora_xlsx::Format` (owned directly)
    - Implement `#[new] fn new()` returning default format
    - _Requirements: 6.1_
  - [x] 3.2 Implement chainable builder methods on Format
    - Implement all builder methods using `PyRefMut<'_, Self>` pattern: `bold`, `italic`, `underline(style)`, `strikethrough`, `font_size(size)`, `font_name(name)`, `font_color(hex)`, `background_color(hex)`, `num_format(format_str)`, `border(style)`, `align(alignment)`, `text_wrap`, `shrink_to_fit`, `indent(level)`, `rotation(angle)`
    - Each method clones inner `Format`, applies the builder call, stores back, returns `PyRefMut<'_, Self>` for Python chaining
    - Implement string-to-enum mappings for border styles (`"none"`, `"thin"`, `"medium"`, `"thick"`, `"double"`, `"dashed"`, `"dotted"`), underline styles (`"single"`, `"double"`), and alignment (`"left"`, `"center"`, `"right"`, `"fill"`, `"justify"`, `"top"`, `"middle"`, `"bottom"`)
    - _Requirements: 6.2, 6.3_
  - [ ]* 3.3 Write property test for Format builder chainability
    - **Property 6: Format builder methods are chainable**
    - **Validates: Requirements 6.2, 6.3**

- [x] 4. Implement Chart class
  - [x] 4.1 Create chart.rs with Chart pyclass and constructor
    - Define `#[pyclass] Chart` wrapping `zavora_xlsx::Chart` (owned directly)
    - Implement `#[new] fn new(chart_type: &str) -> PyResult<Self>` with string-to-`ChartType` mapping for 8 types: `"bar"`, `"column"`, `"line"`, `"pie"`, `"scatter"`, `"area"`, `"doughnut"`, `"radar"`
    - Return `PyValueError` for unrecognized strings
    - _Requirements: 7.1, 7.2, 7.5_
  - [x] 4.2 Implement Chart configuration methods
    - Implement `add_series(values, categories=None, name=None)`, `set_title(title)`, `set_x_axis_name(name)`, `set_y_axis_name(name)`, `set_size(width, height)`, `set_legend_position(position)` using `PyRefMut<'_, Self>` for chaining
    - Implement string-to-`LegendPosition` mapping: `"bottom"`, `"top"`, `"left"`, `"right"`, `"none"`
    - _Requirements: 7.3_
  - [ ]* 4.3 Write property test for invalid chart type rejection
    - **Property 7: Invalid chart type string raises ValueError**
    - **Validates: Requirements 7.5**

- [x] 5. Implement Table class
  - [x] 5.1 Create table.rs with Table pyclass and constructor
    - Define `#[pyclass] Table` wrapping `zavora_xlsx::Table` (owned directly)
    - Implement `#[new] fn new()` returning default table (autofilter enabled)
    - _Requirements: 8.1_
  - [x] 5.2 Implement Table configuration methods
    - Implement `set_columns(columns: &Bound<'_, PyList>)` accepting a Python list of dicts with `name`, `total_label`, `total_function` keys, converting to `zavora_xlsx::TableColumn`
    - Implement `set_style(style: &str)` with string-to-`TableStyle` parsing (`"TableStyleLight1"` → `TableStyle::Light(1)`, etc.)
    - Implement `set_total_row(enabled)`, `set_autofilter(enabled)`, `set_name(name)` using `PyRefMut<'_, Self>` for chaining
    - _Requirements: 8.2, 8.4_

- [x] 6. Checkpoint - Verify value types compile
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Implement Workbook class
  - [x] 7.1 Create workbook.rs with Workbook pyclass and constructors
    - Define `#[pyclass] Workbook` wrapping `Arc<Mutex<zavora_xlsx::Workbook>>`
    - Implement `#[new] fn new()` creating empty workbook
    - Implement `#[staticmethod] fn open(path: &str) -> PyResult<Self>` loading from file path
    - Implement `#[staticmethod] fn open_from_bytes(data: &[u8]) -> PyResult<Self>` loading from bytes
    - Use `IntoPyResult` trait for error conversion on all fallible operations
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  - [x] 7.2 Implement Workbook save methods
    - Implement `save(&self, path: &str) -> PyResult<()>` writing workbook to file
    - Implement `save_to_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>>` returning bytes with xlsx data
    - _Requirements: 2.6, 2.7, 2.8_
  - [x] 7.3 Implement Workbook worksheet access methods
    - Implement `worksheet(&self, index: usize) -> PyResult<Worksheet>` returning `Worksheet` (Arc clone + index)
    - Implement `add_worksheet(&self) -> PyResult<Worksheet>` and `add_worksheet_with_name(&self, name: &str) -> PyResult<Worksheet>` returning `Worksheet`
    - Implement `sheet_names(&self) -> PyResult<Vec<String>>` and `sheet_count(&self) -> PyResult<usize>`
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_
  - [x] 7.4 Implement Workbook document properties methods
    - Implement `set_properties(&self, props: &Bound<'_, PyDict>) -> PyResult<()>` extracting optional `title`, `author`, `subject`, `description`, `keywords`, `category`, `company` keys
    - Implement `properties<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>>` returning current properties as a dict
    - _Requirements: 10.1, 10.2_
  - [ ]* 7.5 Write property tests for Workbook
    - **Property 2: Buffer save/load round-trip**
    - **Property 3: add_worksheet increments sheet count**
    - **Property 8: Document properties round-trip**
    - **Validates: Requirements 2.3, 2.7, 3.3, 3.6, 10.1, 10.2**

- [x] 8. Implement Worksheet class - cell operations
  - [x] 8.1 Create worksheet.rs with Worksheet pyclass
    - Define `#[pyclass] Worksheet` holding `Arc<Mutex<zavora_xlsx::Workbook>>` and `usize` index
    - Implement helper method to lock the mutex and access the worksheet by index, returning `PyRuntimeError` on lock failure or `PyIndexError` on out-of-bounds
    - _Requirements: 3.1_
  - [x] 8.2 Implement cell write methods
    - Implement `write_string(row, col, value, format=None)`, `write_number(row, col, value, format=None)`, `write_boolean(row, col, value, format=None)`, `write_formula(row, col, formula, format=None)`, `write_blank(row, col, format)` using `#[pyo3(signature = (...))]` for optional format parameter
    - Accept optional `&Format` parameter, access `inner` field to pass to core API
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_
  - [x] 8.3 Implement cell read methods
    - Implement `read_cell(&self, py: Python<'_>, row: u32, col: u16) -> PyResult<PyObject>` with `cell_value_to_py` conversion: String→str, Number→float, Bool→bool, Empty→None, Formula→dict with `formula` and `cached_value` keys, RichText→str, DateTime→float, Error→str
    - Implement `used_range<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyDict>>>` returning dict with `first_row`, `first_col`, `last_row`, `last_col` or `None`
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7_
  - [x] 8.4 Implement worksheet name methods
    - Implement `name(&self) -> PyResult<String>` returning current sheet name
    - Implement `set_name(&self, name: &str) -> PyResult<()>` renaming the worksheet
    - _Requirements: 13.1, 13.2, 13.3_
  - [ ]* 8.5 Write property tests for cell round-trip and worksheet operations
    - **Property 1: Cell write/read round-trip**
    - **Property 4: Worksheet name round-trip**
    - **Property 5: used_range encompasses all written cells**
    - **Property 11: Invalid sheet name raises ValueError**
    - **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 5.1, 5.2, 5.3, 5.5, 5.6, 3.4, 3.5, 13.1, 13.2, 13.3**

- [x] 9. Checkpoint - Verify core Workbook/Worksheet functionality
  - Ensure all tests pass, ask the user if questions arise.

- [x] 10. Implement Worksheet layout and feature methods
  - [x] 10.1 Implement layout methods
    - Implement `set_column_width(col, width)`, `set_row_height(row, height)`, `set_freeze_panes(row, col)`, `merge_range(r1, c1, r2, c2, text, format=None)`, `autofit()`, `set_zoom(percent)`
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6_
  - [x] 10.2 Implement chart and table insertion methods
    - Implement `insert_chart(&self, row: u32, col: u16, chart: &Chart) -> PyResult<()>` accessing `chart.inner` and passing to core API
    - Implement `add_table(&self, first_row: u32, first_col: u16, last_row: u32, last_col: u16, table: &Table) -> PyResult<()>` accessing `table.inner`
    - _Requirements: 7.4, 8.3_

- [x] 11. Implement CSV export
  - [x] 11.1 Implement to_csv_string method on Worksheet
    - Implement `to_csv_string(&self, options: Option<&Bound<'_, PyDict>>) -> PyResult<String>` extracting optional `delimiter`, `quote`, `line_ending`, `date_format` keys from dict
    - Convert `delimiter` and `quote` strings to `u8` by taking first byte, raise `PyValueError` on empty strings
    - Call core `to_csv_string()` with constructed `CsvOptions`
    - _Requirements: 12.1, 12.2, 12.3_
  - [ ]* 11.2 Write property test for CSV export completeness
    - **Property 10: CSV export contains all written cell values**
    - **Validates: Requirements 12.1**

- [x] 12. Checkpoint - Verify all Rust code compiles and property tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 13. Write Python integration tests
  - [ ]* 13.1 Create test infrastructure and workbook lifecycle tests
    - Create `zavora-xlsx-python/tests/test_basic.py` with pytest setup
    - Write tests for: `Workbook()`, `Workbook.open()`, `workbook.save()`, `workbook.save_to_bytes()`, `Workbook.open_from_bytes()`
    - Test error cases: invalid file path raises `OSError`, invalid buffer raises exception
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8_
  - [ ]* 13.2 Write worksheet access and cell operation tests
    - Test `worksheet(0)`, `add_worksheet()`, `add_worksheet_with_name()`, `sheet_names()`, `sheet_count()`
    - Test write/read round-trip for each cell type (string, number, boolean, formula, blank)
    - Test `used_range()` on populated and empty worksheets
    - Test worksheet `name()` and `set_name()`
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 13.1, 13.2, 13.3_
  - [ ]* 13.3 Write format, chart, table, layout, and CSV tests
    - Test Format chaining: `Format().bold().italic().font_size(14)`
    - Test Chart creation for each type, `add_series`, `set_title`, `insert_chart`
    - Test invalid chart type raises `ValueError`
    - Test Table creation, `set_columns` with list of dicts, `set_style`, `add_table`
    - Test layout methods: `set_column_width`, `set_row_height`, `set_freeze_panes`, `merge_range`, `autofit`, `set_zoom`
    - Test CSV export: default options, custom delimiter via dict, empty worksheet
    - Test document properties: `set_properties` and `properties` with dicts
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 7.1, 7.2, 7.3, 7.4, 7.5, 8.1, 8.2, 8.3, 8.4, 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 10.1, 10.2, 12.1, 12.2, 12.3_

- [x] 14. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests (Rust, using `proptest`) validate universal correctness properties from the design
- Python integration tests (pytest) validate end-to-end behavior through the PyO3 boundary
- The `Arc<Mutex<>>` ownership model is critical — Workbook and Worksheet share state, while Format/Chart/Table are independent value types with direct ownership
- Format builder methods use the `PyRefMut<'_, Self>` pattern (clone inner, apply builder, store back) for Python-side chaining
- Dict-based APIs are used for properties, CSV options, and table columns (idiomatic Python)

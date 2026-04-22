# Requirements Document

## Introduction

Python bindings for the zavora-xlsx Rust library, exposing high-performance Excel .xlsx reading, writing, and editing capabilities to the Python ecosystem. The bindings use PyO3 to create a native Python extension module distributed as a workspace member crate (`zavora-xlsx-python`), built and packaged with maturin. The goal is to provide an idiomatic Python API (snake_case methods, Python exceptions, `bytes` objects) that covers the core functionality of the Rust library: workbook lifecycle, cell writing/reading, formatting, charts, and tables.

## Glossary

- **Extension_Module**: The compiled native Python extension module produced by PyO3 from the `zavora-xlsx-python` Rust crate, importable as `zavora_xlsx`
- **Workbook_Class**: The Python class wrapping the Rust `Workbook` struct, providing workbook lifecycle methods
- **Worksheet_Class**: The Python class wrapping the Rust `Worksheet` struct, providing cell read/write and layout methods
- **Format_Class**: The Python class wrapping the Rust `Format` struct, providing cell formatting via builder methods
- **Chart_Class**: The Python class wrapping the Rust `Chart` struct, providing chart creation and configuration
- **Table_Class**: The Python class wrapping the Rust `Table` struct, providing table creation and configuration
- **PyO3**: The Rust framework for building native Python extension modules using the Python C API
- **maturin**: The build tool for compiling PyO3 crates into Python wheels distributable via pip
- **CellValue**: A Python value representing a cell's content — one of `str`, `float`, `bool`, `None`, or a `dict` with a `formula` key

## Requirements

### Requirement 1: Crate and Build Setup

**User Story:** As a developer, I want the Python bindings to be a separate workspace member crate with proper PyO3 and maturin build configuration, so that the extension module can be compiled and distributed as a Python wheel.

#### Acceptance Criteria

1. THE Extension_Module SHALL be defined as a workspace member crate named `zavora-xlsx-python` in the `zavora-xlsx-python/` directory
2. THE Extension_Module SHALL declare `pyo3` (version 0.22, feature `extension-module`) and `zavora-xlsx` (path dependency `..`) as dependencies in its `Cargo.toml`
3. THE Extension_Module SHALL set `crate-type` to `["cdylib"]` in its `Cargo.toml`
4. THE Extension_Module SHALL include a `pyproject.toml` with `maturin` as the build backend and `requires = ["maturin>=1.0"]`
5. WHEN `maturin develop` is executed in the `zavora-xlsx-python/` directory, THE Extension_Module SHALL produce an importable Python module named `zavora_xlsx`

### Requirement 2: Workbook Lifecycle

**User Story:** As a Python developer, I want to create, open, save, and export workbooks, so that I can generate and manipulate Excel files from Python.

#### Acceptance Criteria

1. WHEN `Workbook()` is called, THE Workbook_Class SHALL create an empty workbook with one default worksheet
2. WHEN `Workbook.open(path)` is called with a valid file path string, THE Workbook_Class SHALL load the xlsx file and return a Workbook_Class instance
3. WHEN `Workbook.open_from_bytes(data)` is called with a valid `bytes` object, THE Workbook_Class SHALL load the xlsx data from the bytes and return a Workbook_Class instance
4. IF `Workbook.open(path)` is called with a non-existent or invalid file, THEN THE Workbook_Class SHALL raise a Python exception with a descriptive message
5. IF `Workbook.open_from_bytes(data)` is called with invalid data, THEN THE Workbook_Class SHALL raise a Python exception with a descriptive message
6. WHEN `workbook.save(path)` is called, THE Workbook_Class SHALL write the workbook to the specified file path
7. WHEN `workbook.save_to_bytes()` is called, THE Workbook_Class SHALL return a Python `bytes` object containing the xlsx file data
8. IF `workbook.save(path)` fails due to I/O errors, THEN THE Workbook_Class SHALL raise a Python `OSError` with a descriptive message

### Requirement 3: Worksheet Access

**User Story:** As a Python developer, I want to access and manage worksheets within a workbook, so that I can work with multiple sheets.

#### Acceptance Criteria

1. WHEN `workbook.worksheet(index)` is called with a valid zero-based index, THE Workbook_Class SHALL return the corresponding Worksheet_Class instance
2. IF `workbook.worksheet(index)` is called with an out-of-bounds index, THEN THE Workbook_Class SHALL raise an `IndexError`
3. WHEN `workbook.add_worksheet()` is called, THE Workbook_Class SHALL add a new worksheet and return the Worksheet_Class instance
4. WHEN `workbook.add_worksheet_with_name(name)` is called, THE Workbook_Class SHALL add a new worksheet with the specified name and return the Worksheet_Class instance
5. WHEN `workbook.sheet_names()` is called, THE Workbook_Class SHALL return a list of strings containing all worksheet names
6. WHEN `workbook.sheet_count()` is called, THE Workbook_Class SHALL return the number of worksheets as an integer

### Requirement 4: Cell Writing

**User Story:** As a Python developer, I want to write strings, numbers, booleans, and formulas to cells, so that I can populate spreadsheet data.

#### Acceptance Criteria

1. WHEN `worksheet.write_string(row, col, value)` is called, THE Worksheet_Class SHALL write a string value to the specified cell
2. WHEN `worksheet.write_number(row, col, value)` is called, THE Worksheet_Class SHALL write a numeric value to the specified cell
3. WHEN `worksheet.write_boolean(row, col, value)` is called, THE Worksheet_Class SHALL write a boolean value to the specified cell
4. WHEN `worksheet.write_formula(row, col, formula)` is called, THE Worksheet_Class SHALL write a formula string to the specified cell
5. WHEN any write method is called with an optional `format` keyword argument of type Format_Class, THE Worksheet_Class SHALL apply the format to the written cell
6. WHEN `worksheet.write_blank(row, col, format)` is called, THE Worksheet_Class SHALL write an empty cell with the specified format
7. IF a write method is called with a negative row or column index, THEN THE Worksheet_Class SHALL raise a `ValueError`

### Requirement 5: Cell Reading

**User Story:** As a Python developer, I want to read cell values from worksheets, so that I can extract data from existing Excel files.

#### Acceptance Criteria

1. WHEN `worksheet.read_cell(row, col)` is called on a cell containing a string, THE Worksheet_Class SHALL return the string value
2. WHEN `worksheet.read_cell(row, col)` is called on a cell containing a number, THE Worksheet_Class SHALL return the numeric value as a `float`
3. WHEN `worksheet.read_cell(row, col)` is called on a cell containing a boolean, THE Worksheet_Class SHALL return the boolean value
4. WHEN `worksheet.read_cell(row, col)` is called on an empty cell, THE Worksheet_Class SHALL return `None`
5. WHEN `worksheet.read_cell(row, col)` is called on a cell containing a formula, THE Worksheet_Class SHALL return a `dict` with `formula` and `cached_value` keys
6. WHEN `worksheet.used_range()` is called on a worksheet with data, THE Worksheet_Class SHALL return a `dict` with `first_row`, `first_col`, `last_row`, and `last_col` keys
7. WHEN `worksheet.used_range()` is called on an empty worksheet, THE Worksheet_Class SHALL return `None`

### Requirement 6: Cell Formatting

**User Story:** As a Python developer, I want to create and apply cell formats, so that I can style spreadsheet cells.

#### Acceptance Criteria

1. WHEN `Format()` is called, THE Format_Class SHALL create a default format object
2. THE Format_Class SHALL expose chainable builder methods: `bold()`, `italic()`, `underline(style)`, `strikethrough()`, `font_size(size)`, `font_name(name)`, `font_color(hex)`, `background_color(hex)`, `num_format(format_str)`, `border(style)`, `align(alignment)`, `text_wrap()`, `shrink_to_fit()`, `indent(level)`, `rotation(angle)`
3. WHEN a builder method is called on a Format_Class instance, THE Format_Class SHALL return the same instance to enable method chaining
4. WHEN a Format_Class instance is passed to a cell write method, THE Worksheet_Class SHALL apply the format properties to the target cell

### Requirement 7: Chart Support

**User Story:** As a Python developer, I want to create and insert charts into worksheets, so that I can visualize data.

#### Acceptance Criteria

1. WHEN `Chart(chart_type)` is called with a valid chart type string, THE Chart_Class SHALL create a chart of the specified type
2. THE Chart_Class SHALL support chart type strings: `"bar"`, `"column"`, `"line"`, `"pie"`, `"scatter"`, `"area"`, `"doughnut"`, `"radar"`
3. THE Chart_Class SHALL expose configuration methods: `add_series(values, categories, name)`, `set_title(title)`, `set_x_axis_name(name)`, `set_y_axis_name(name)`, `set_size(width, height)`, `set_legend_position(position)`
4. WHEN `worksheet.insert_chart(row, col, chart)` is called, THE Worksheet_Class SHALL embed the chart at the specified cell position
5. IF `Chart(chart_type)` is called with an unrecognized chart type string, THEN THE Chart_Class SHALL raise a `ValueError`

### Requirement 8: Table Support

**User Story:** As a Python developer, I want to create and insert tables into worksheets, so that I can structure data with autofilter and styling.

#### Acceptance Criteria

1. WHEN `Table()` is called, THE Table_Class SHALL create a default table object with autofilter enabled
2. THE Table_Class SHALL expose configuration methods: `set_columns(columns)`, `set_style(style)`, `set_total_row(enabled)`, `set_autofilter(enabled)`, `set_name(name)`
3. WHEN `worksheet.add_table(first_row, first_col, last_row, last_col, table)` is called, THE Worksheet_Class SHALL insert the table spanning the specified cell range
4. THE Table_Class `set_columns` method SHALL accept a list of dicts with `name`, `total_label`, and `total_function` keys

### Requirement 9: Worksheet Layout

**User Story:** As a Python developer, I want to configure worksheet layout properties like column widths, row heights, and freeze panes, so that I can control the visual presentation.

#### Acceptance Criteria

1. WHEN `worksheet.set_column_width(col, width)` is called, THE Worksheet_Class SHALL set the width of the specified column
2. WHEN `worksheet.set_row_height(row, height)` is called, THE Worksheet_Class SHALL set the height of the specified row
3. WHEN `worksheet.set_freeze_panes(row, col)` is called, THE Worksheet_Class SHALL freeze rows above and columns to the left of the specified position
4. WHEN `worksheet.merge_range(r1, c1, r2, c2, text, format)` is called, THE Worksheet_Class SHALL merge the specified cell range and write the text with the given format
5. WHEN `worksheet.autofit()` is called, THE Worksheet_Class SHALL automatically adjust column widths to fit cell contents
6. WHEN `worksheet.set_zoom(percent)` is called, THE Worksheet_Class SHALL set the worksheet zoom level to the specified percentage

### Requirement 10: Document Properties

**User Story:** As a Python developer, I want to set document metadata like title and author, so that the generated files have proper document properties.

#### Acceptance Criteria

1. WHEN `workbook.set_properties(props)` is called with a dict containing optional `title`, `author`, `subject`, `description`, `keywords`, `category`, and `company` keys, THE Workbook_Class SHALL set the corresponding document properties
2. WHEN `workbook.properties()` is called, THE Workbook_Class SHALL return a dict containing the current document properties

### Requirement 11: Error Handling

**User Story:** As a Python developer, I want Rust errors to be surfaced as Python exceptions with clear messages, so that I can handle errors idiomatically.

#### Acceptance Criteria

1. WHEN a Rust operation returns an `Err` result, THE Extension_Module SHALL convert the error into a Python exception and raise the exception
2. THE Extension_Module SHALL include the original Rust error message in the Python exception's message
3. IF a type mismatch occurs in a method argument, THEN THE Extension_Module SHALL raise a `TypeError` with a descriptive message
4. WHEN a Rust `Error::Io` variant is encountered, THE Extension_Module SHALL raise a Python `OSError`
5. WHEN a Rust `Error::SheetNotFound` variant is encountered, THE Extension_Module SHALL raise a Python `IndexError`

### Requirement 12: CSV Export

**User Story:** As a Python developer, I want to export worksheet data to CSV format, so that I can interoperate with other tools.

#### Acceptance Criteria

1. WHEN `worksheet.to_csv_string(options)` is called, THE Worksheet_Class SHALL return the worksheet data as a CSV-formatted string
2. THE `options` parameter SHALL accept a dict with optional `delimiter`, `quote`, `line_ending`, and `date_format` keys
3. WHEN `worksheet.to_csv_string()` is called without options, THE Worksheet_Class SHALL use default CSV options (comma delimiter, double-quote, CRLF line ending)

### Requirement 13: Worksheet Name

**User Story:** As a Python developer, I want to read and change worksheet names, so that I can organize sheets.

#### Acceptance Criteria

1. WHEN `worksheet.name()` is called, THE Worksheet_Class SHALL return the current worksheet name as a string
2. WHEN `worksheet.set_name(name)` is called, THE Worksheet_Class SHALL rename the worksheet to the specified name
3. IF `worksheet.set_name(name)` is called with an invalid name, THEN THE Worksheet_Class SHALL raise a `ValueError`

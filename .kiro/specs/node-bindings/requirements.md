# Requirements Document

## Introduction

Node.js bindings for the zavora-xlsx Rust library, exposing high-performance Excel .xlsx reading, writing, and editing capabilities to the Node.js ecosystem. The bindings use napi-rs to create a native addon distributed as a workspace member crate (`zavora-xlsx-node`). The goal is to provide an idiomatic JavaScript API that covers the core functionality of the Rust library: workbook lifecycle, cell writing/reading, formatting, charts, and tables.

## Glossary

- **Addon**: The compiled native Node.js module produced by napi-rs from the `zavora-xlsx-node` Rust crate
- **Workbook_Class**: The JavaScript class wrapping the Rust `Workbook` struct, providing workbook lifecycle methods
- **Worksheet_Class**: The JavaScript class wrapping the Rust `Worksheet` struct, providing cell read/write and layout methods
- **Format_Class**: The JavaScript class wrapping the Rust `Format` struct, providing cell formatting via builder methods
- **Chart_Class**: The JavaScript class wrapping the Rust `Chart` struct, providing chart creation and configuration
- **Table_Class**: The JavaScript class wrapping the Rust `Table` struct, providing table creation and configuration
- **napi-rs**: The Rust framework for building precompiled Node.js native addons using N-API
- **Buffer**: A Node.js `Buffer` object representing raw binary data
- **CellValue**: A JavaScript value representing a cell's content — one of `string`, `number`, `boolean`, `null`, or an object with a `formula` field

## Requirements

### Requirement 1: Crate and Build Setup

**User Story:** As a developer, I want the Node.js bindings to be a separate workspace member crate with proper napi-rs build configuration, so that the addon can be compiled and distributed independently.

#### Acceptance Criteria

1. THE Addon SHALL be defined as a workspace member crate named `zavora-xlsx-node` in the `zavora-xlsx-node/` directory
2. THE Addon SHALL declare `napi` (version 2, feature `napi4`), `napi-derive` (version 2), and `zavora-xlsx` (path dependency `..`) as dependencies in its `Cargo.toml`
3. THE Addon SHALL declare `napi-build` (version 2) as a build dependency and use it in a `build.rs` file
4. THE Addon SHALL set `crate-type` to `["cdylib"]` in its `Cargo.toml`
5. WHEN `napi build` is executed in the `zavora-xlsx-node/` directory, THE Addon SHALL produce a loadable `.node` native module

### Requirement 2: Workbook Lifecycle

**User Story:** As a Node.js developer, I want to create, open, save, and export workbooks, so that I can generate and manipulate Excel files from JavaScript.

#### Acceptance Criteria

1. WHEN `new Workbook()` is called, THE Workbook_Class SHALL create an empty workbook with one default worksheet
2. WHEN `Workbook.open(path)` is called with a valid file path, THE Workbook_Class SHALL load the xlsx file and return a Workbook_Class instance
3. WHEN `Workbook.openFromBuffer(buffer)` is called with a valid Buffer, THE Workbook_Class SHALL load the xlsx data from the Buffer and return a Workbook_Class instance
4. IF `Workbook.open(path)` is called with a non-existent or invalid file, THEN THE Workbook_Class SHALL throw a JavaScript Error with a descriptive message
5. IF `Workbook.openFromBuffer(buffer)` is called with invalid data, THEN THE Workbook_Class SHALL throw a JavaScript Error with a descriptive message
6. WHEN `workbook.save(path)` is called, THE Workbook_Class SHALL write the workbook to the specified file path
7. WHEN `workbook.saveToBuffer()` is called, THE Workbook_Class SHALL return a Node.js Buffer containing the xlsx file data
8. IF `workbook.save(path)` fails due to I/O errors, THEN THE Workbook_Class SHALL throw a JavaScript Error with a descriptive message

### Requirement 3: Worksheet Access

**User Story:** As a Node.js developer, I want to access and manage worksheets within a workbook, so that I can work with multiple sheets.

#### Acceptance Criteria

1. WHEN `workbook.worksheet(index)` is called with a valid zero-based index, THE Workbook_Class SHALL return the corresponding Worksheet_Class instance
2. IF `workbook.worksheet(index)` is called with an out-of-bounds index, THEN THE Workbook_Class SHALL throw a JavaScript Error
3. WHEN `workbook.addWorksheet()` is called, THE Workbook_Class SHALL add a new worksheet and return the Worksheet_Class instance
4. WHEN `workbook.addWorksheetWithName(name)` is called, THE Workbook_Class SHALL add a new worksheet with the specified name and return the Worksheet_Class instance
5. WHEN `workbook.sheetNames()` is called, THE Workbook_Class SHALL return an array of strings containing all worksheet names
6. WHEN `workbook.sheetCount()` is called, THE Workbook_Class SHALL return the number of worksheets as a number

### Requirement 4: Cell Writing

**User Story:** As a Node.js developer, I want to write strings, numbers, booleans, and formulas to cells, so that I can populate spreadsheet data.

#### Acceptance Criteria

1. WHEN `worksheet.writeString(row, col, value)` is called, THE Worksheet_Class SHALL write a string value to the specified cell
2. WHEN `worksheet.writeNumber(row, col, value)` is called, THE Worksheet_Class SHALL write a numeric value to the specified cell
3. WHEN `worksheet.writeBoolean(row, col, value)` is called, THE Worksheet_Class SHALL write a boolean value to the specified cell
4. WHEN `worksheet.writeFormula(row, col, formula)` is called, THE Worksheet_Class SHALL write a formula string to the specified cell
5. WHEN any write method is called with an optional Format_Class argument as the last parameter, THE Worksheet_Class SHALL apply the format to the written cell
6. WHEN `worksheet.writeBlank(row, col, format)` is called, THE Worksheet_Class SHALL write an empty cell with the specified format
7. IF a write method is called with a negative row or column index, THEN THE Worksheet_Class SHALL throw a JavaScript Error

### Requirement 5: Cell Reading

**User Story:** As a Node.js developer, I want to read cell values from worksheets, so that I can extract data from existing Excel files.

#### Acceptance Criteria

1. WHEN `worksheet.readCell(row, col)` is called on a cell containing a string, THE Worksheet_Class SHALL return the string value
2. WHEN `worksheet.readCell(row, col)` is called on a cell containing a number, THE Worksheet_Class SHALL return the numeric value
3. WHEN `worksheet.readCell(row, col)` is called on a cell containing a boolean, THE Worksheet_Class SHALL return the boolean value
4. WHEN `worksheet.readCell(row, col)` is called on an empty cell, THE Worksheet_Class SHALL return `null`
5. WHEN `worksheet.readCell(row, col)` is called on a cell containing a formula, THE Worksheet_Class SHALL return an object with `formula` and `cachedValue` properties
6. WHEN `worksheet.usedRange()` is called on a worksheet with data, THE Worksheet_Class SHALL return an object with `firstRow`, `firstCol`, `lastRow`, and `lastCol` properties
7. WHEN `worksheet.usedRange()` is called on an empty worksheet, THE Worksheet_Class SHALL return `null`

### Requirement 6: Cell Formatting

**User Story:** As a Node.js developer, I want to create and apply cell formats, so that I can style spreadsheet cells.

#### Acceptance Criteria

1. WHEN `new Format()` is called, THE Format_Class SHALL create a default format object
2. THE Format_Class SHALL expose chainable builder methods: `bold()`, `italic()`, `underline(style)`, `strikethrough()`, `fontSize(size)`, `fontName(name)`, `fontColor(hex)`, `backgroundColor(hex)`, `numFormat(format)`, `border(style)`, `align(alignment)`, `textWrap()`, `shrinkToFit()`, `indent(level)`, `rotation(angle)`
3. WHEN a builder method is called on a Format_Class instance, THE Format_Class SHALL return the same instance to enable method chaining
4. WHEN a Format_Class instance is passed to a cell write method, THE Worksheet_Class SHALL apply the format properties to the target cell

### Requirement 7: Chart Support

**User Story:** As a Node.js developer, I want to create and insert charts into worksheets, so that I can visualize data.

#### Acceptance Criteria

1. WHEN `new Chart(type)` is called with a valid chart type string, THE Chart_Class SHALL create a chart of the specified type
2. THE Chart_Class SHALL support chart type strings: `"bar"`, `"column"`, `"line"`, `"pie"`, `"scatter"`, `"area"`, `"doughnut"`, `"radar"`
3. THE Chart_Class SHALL expose configuration methods: `addSeries(values, categories, name)`, `setTitle(title)`, `setXAxisName(name)`, `setYAxisName(name)`, `setSize(width, height)`, `setLegendPosition(position)`
4. WHEN `worksheet.insertChart(row, col, chart)` is called, THE Worksheet_Class SHALL embed the chart at the specified cell position
5. IF `new Chart(type)` is called with an unrecognized chart type string, THEN THE Chart_Class SHALL throw a JavaScript Error

### Requirement 8: Table Support

**User Story:** As a Node.js developer, I want to create and insert tables into worksheets, so that I can structure data with autofilter and styling.

#### Acceptance Criteria

1. WHEN `new Table()` is called, THE Table_Class SHALL create a default table object with autofilter enabled
2. THE Table_Class SHALL expose configuration methods: `setColumns(columns)`, `setStyle(style)`, `setTotalRow(enabled)`, `setAutofilter(enabled)`, `setName(name)`
3. WHEN `worksheet.addTable(firstRow, firstCol, lastRow, lastCol, table)` is called, THE Worksheet_Class SHALL insert the table spanning the specified cell range
4. THE Table_Class `setColumns` method SHALL accept an array of objects with `name`, `totalLabel`, and `totalFunction` properties

### Requirement 9: Worksheet Layout

**User Story:** As a Node.js developer, I want to configure worksheet layout properties like column widths, row heights, and freeze panes, so that I can control the visual presentation.

#### Acceptance Criteria

1. WHEN `worksheet.setColumnWidth(col, width)` is called, THE Worksheet_Class SHALL set the width of the specified column
2. WHEN `worksheet.setRowHeight(row, height)` is called, THE Worksheet_Class SHALL set the height of the specified row
3. WHEN `worksheet.setFreezePanes(row, col)` is called, THE Worksheet_Class SHALL freeze rows above and columns to the left of the specified position
4. WHEN `worksheet.mergeRange(r1, c1, r2, c2, text, format)` is called, THE Worksheet_Class SHALL merge the specified cell range and write the text with the given format
5. WHEN `worksheet.autofit()` is called, THE Worksheet_Class SHALL automatically adjust column widths to fit cell contents
6. WHEN `worksheet.setZoom(percent)` is called, THE Worksheet_Class SHALL set the worksheet zoom level to the specified percentage

### Requirement 10: Document Properties

**User Story:** As a Node.js developer, I want to set document metadata like title and author, so that the generated files have proper document properties.

#### Acceptance Criteria

1. WHEN `workbook.setProperties(props)` is called with an object containing optional `title`, `author`, `subject`, `description`, `keywords`, `category`, and `company` fields, THE Workbook_Class SHALL set the corresponding document properties
2. WHEN `workbook.properties()` is called, THE Workbook_Class SHALL return an object containing the current document properties

### Requirement 11: Error Handling

**User Story:** As a Node.js developer, I want Rust errors to be surfaced as JavaScript exceptions with clear messages, so that I can handle errors idiomatically.

#### Acceptance Criteria

1. WHEN a Rust operation returns an `Err` result, THE Addon SHALL convert the error into a JavaScript Error and throw the exception
2. THE Addon SHALL include the original Rust error message in the JavaScript Error's `message` property
3. IF a type mismatch occurs in a method argument, THEN THE Addon SHALL throw a JavaScript TypeError with a descriptive message

### Requirement 12: CSV Export

**User Story:** As a Node.js developer, I want to export worksheet data to CSV format, so that I can interoperate with other tools.

#### Acceptance Criteria

1. WHEN `worksheet.toCsvString(options)` is called, THE Worksheet_Class SHALL return the worksheet data as a CSV-formatted string
2. THE `options` parameter SHALL accept an object with optional `delimiter`, `quote`, `lineEnding`, and `dateFormat` fields
3. WHEN `worksheet.toCsvString()` is called without options, THE Worksheet_Class SHALL use default CSV options (comma delimiter, double-quote, CRLF line ending)

### Requirement 13: Worksheet Name

**User Story:** As a Node.js developer, I want to read and change worksheet names, so that I can organize sheets.

#### Acceptance Criteria

1. WHEN `worksheet.name()` is called, THE Worksheet_Class SHALL return the current worksheet name as a string
2. WHEN `worksheet.setName(name)` is called, THE Worksheet_Class SHALL rename the worksheet to the specified name
3. IF `worksheet.setName(name)` is called with an invalid name, THEN THE Worksheet_Class SHALL throw a JavaScript Error

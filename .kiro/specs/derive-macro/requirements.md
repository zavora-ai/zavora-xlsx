# Requirements Document

## Introduction

The `zavora-xlsx-derive` crate provides a `#[derive(ExcelRow)]` proc macro that generates trait implementations for writing and reading Rust structs to and from Excel worksheet rows. This is a standalone alternative to the existing serde-based approach — it does not depend on serde and instead generates code that directly calls the zavora-xlsx cell-level API (`ws.write()`, `ws.read_cell()`). The crate lives in a separate workspace member directory (`zavora-xlsx-derive/`) because proc-macro crates require `proc-macro = true`.

## Glossary

- **Derive_Macro**: The `#[derive(ExcelRow)]` procedural macro that parses struct definitions and generates `ExcelRowWriter` and `ExcelRowReader` trait implementations
- **ExcelRowWriter**: A trait with methods `write_header(&self, ws, row) -> Result<()>` and `write_row(&self, ws, row) -> Result<()>` for writing struct data to worksheet rows
- **ExcelRowReader**: A trait with method `read_row(ws, row, headers) -> Result<Self>` for reading a worksheet row into a struct instance
- **Header**: The column label written to the header row; defaults to the Rust field name, overridden by `#[excel(header = "...")]`
- **Field_Attribute**: An `#[excel(...)]` attribute applied to a struct field to customize column header, number format, or skip behavior
- **Format**: The `zavora_xlsx::Format` type used to apply number formatting to cells via `Format::new().num_format(...)`
- **CellValue**: The `zavora_xlsx::CellValue` enum returned by `ws.read_cell(row, col)`, with variants `String`, `Number`, `Bool`, `Empty`, `DateTime`, `Error`, `Formula`, `RichText`
- **Worksheet**: The `zavora_xlsx::Worksheet` type providing `write()`, `write_with_format()`, and `read_cell()` methods
- **RowNum**: Type alias `u32` representing a 0-based row index
- **ColNum**: Type alias `u16` representing a 0-based column index
- **IntoExcelData**: The trait implemented by types that can be written to a cell (`&str`, `String`, `f64`, `bool`, numeric types, `ExcelDateTime`)

## Requirements

### Requirement 1: Crate Structure

**User Story:** As a library maintainer, I want the derive macro to be a separate workspace member crate, so that it compiles as a proc-macro library independently from the core crate.

#### Acceptance Criteria

1. THE Derive_Macro crate SHALL reside in the `zavora-xlsx-derive/` directory at the workspace root
2. THE Derive_Macro crate SHALL declare `proc-macro = true` in its `Cargo.toml` `[lib]` section
3. THE Derive_Macro crate SHALL depend on `syn` version 2 with the `full` feature, `quote` version 1, and `proc-macro2` version 1
4. THE Derive_Macro crate SHALL be listed as a workspace member in the root `Cargo.toml`
5. THE Derive_Macro crate SHALL NOT depend on `serde` or any serde-related crate

### Requirement 2: ExcelRow Derive Macro Entry Point

**User Story:** As a developer, I want to annotate my struct with `#[derive(ExcelRow)]`, so that the macro generates both writing and reading trait implementations automatically.

#### Acceptance Criteria

1. WHEN a named struct is annotated with `#[derive(ExcelRow)]`, THE Derive_Macro SHALL generate an `ExcelRowWriter` implementation and an `ExcelRowReader` implementation for that struct
2. WHEN a tuple struct, unit struct, or enum is annotated with `#[derive(ExcelRow)]`, THE Derive_Macro SHALL produce a compile-time error with a descriptive message indicating that only named structs are supported
3. THE Derive_Macro SHALL process all non-skipped fields in declaration order, assigning each field a sequential 0-based column index

### Requirement 3: ExcelRowWriter Trait — Header Writing

**User Story:** As a developer, I want to write column headers to a worksheet row from my struct definition, so that the spreadsheet has labeled columns matching my struct fields.

#### Acceptance Criteria

1. WHEN `write_header` is called on a struct implementing ExcelRowWriter, THE ExcelRowWriter SHALL write one header string per non-skipped field to consecutive columns starting at column 0 in the specified row
2. WHEN a field has an `#[excel(header = "...")]` attribute, THE ExcelRowWriter SHALL use the attribute value as the header string for that field's column
3. WHEN a field does not have an `#[excel(header = "...")]` attribute, THE ExcelRowWriter SHALL use the Rust field name as the header string
4. THE ExcelRowWriter SHALL call `ws.write(row, col, header_str)` for each header cell

### Requirement 4: ExcelRowWriter Trait — Row Writing

**User Story:** As a developer, I want to write struct field values to a worksheet row, so that each field maps to the correct column cell.

#### Acceptance Criteria

1. WHEN `write_row` is called on a struct implementing ExcelRowWriter, THE ExcelRowWriter SHALL write each non-skipped field value to the corresponding column in the specified row
2. WHEN a field has type `String` or `&str`, THE ExcelRowWriter SHALL write the value using `ws.write(row, col, value)`
3. WHEN a field has a numeric type (`f64`, `f32`, `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`), THE ExcelRowWriter SHALL write the value using `ws.write(row, col, value)`
4. WHEN a field has type `bool`, THE ExcelRowWriter SHALL write the value using `ws.write(row, col, value)`
5. WHEN a field has type `Option<T>` and the value is `Some(v)`, THE ExcelRowWriter SHALL write `v` to the cell
6. WHEN a field has type `Option<T>` and the value is `None`, THE ExcelRowWriter SHALL leave the cell empty by not writing to that column
7. WHEN a field has an `#[excel(format = "...")]` attribute, THE ExcelRowWriter SHALL create a `Format::new().num_format(format_str)` and write the value using `ws.write_with_format(row, col, value, &format)`

### Requirement 5: ExcelRowReader Trait — Row Reading

**User Story:** As a developer, I want to read a worksheet row into a struct instance, so that I can work with typed data from Excel files.

#### Acceptance Criteria

1. WHEN `read_row` is called with a worksheet, row number, and header-to-column mapping, THE ExcelRowReader SHALL construct a struct instance by reading cell values from the specified row
2. WHEN a field has type `String`, THE ExcelRowReader SHALL convert the CellValue to a String (CellValue::String directly, CellValue::Number via `to_string()`, CellValue::Bool via `to_string()`)
3. WHEN a field has a numeric type, THE ExcelRowReader SHALL extract the value from CellValue::Number and cast it to the target type
4. WHEN a field has type `bool`, THE ExcelRowReader SHALL extract the value from CellValue::Bool
5. WHEN a field has type `Option<T>` and the cell is CellValue::Empty, THE ExcelRowReader SHALL set the field to `None`
6. WHEN a field has type `Option<T>` and the cell contains a value, THE ExcelRowReader SHALL set the field to `Some(converted_value)`
7. IF a cell value cannot be converted to the expected field type, THEN THE ExcelRowReader SHALL return an `Err` with a descriptive error message including the field name, expected type, and actual CellValue variant
8. THE ExcelRowReader SHALL use the header-to-column mapping (a `&[String]` slice of header names) to find the column index for each field by matching the field's header name

### Requirement 6: Field Skip Attribute

**User Story:** As a developer, I want to skip certain struct fields during Excel serialization and deserialization, so that I can have fields that exist only in Rust and not in the spreadsheet.

#### Acceptance Criteria

1. WHEN a field is annotated with `#[excel(skip)]`, THE Derive_Macro SHALL exclude that field from header writing, row writing, and row reading
2. WHEN a field is annotated with `#[excel(skip)]` and the struct is being read, THE ExcelRowReader SHALL use `Default::default()` to populate the skipped field
3. WHEN a field is annotated with `#[excel(skip)]`, THE Derive_Macro SHALL require that the field's type implements the `Default` trait

### Requirement 7: Attribute Parsing and Validation

**User Story:** As a developer, I want clear compile-time errors when I use invalid attributes, so that I can fix mistakes before running my code.

#### Acceptance Criteria

1. IF an `#[excel(...)]` attribute contains an unrecognized key (not `header`, `format`, or `skip`), THEN THE Derive_Macro SHALL produce a compile-time error identifying the unrecognized key
2. IF an `#[excel(header = ...)]` attribute has a non-string-literal value, THEN THE Derive_Macro SHALL produce a compile-time error stating that the header value must be a string literal
3. IF an `#[excel(format = ...)]` attribute has a non-string-literal value, THEN THE Derive_Macro SHALL produce a compile-time error stating that the format value must be a string literal
4. THE Derive_Macro SHALL support combining `header` and `format` in a single attribute (e.g., `#[excel(header = "Amount", format = "#,##0.00")]`)
5. IF `skip` is combined with `header` or `format` in the same attribute, THEN THE Derive_Macro SHALL produce a compile-time error stating that `skip` cannot be combined with other options

### Requirement 8: Trait Definitions in Core Crate

**User Story:** As a library maintainer, I want the `ExcelRowWriter` and `ExcelRowReader` traits defined in the core `zavora-xlsx` crate, so that user code depends on stable trait definitions without needing the derive crate at runtime.

#### Acceptance Criteria

1. THE `zavora_xlsx` crate SHALL define the `ExcelRowWriter` trait with methods `fn write_header(&self, ws: &mut Worksheet, row: RowNum) -> Result<()>` and `fn write_row(&self, ws: &mut Worksheet, row: RowNum) -> Result<()>`
2. THE `zavora_xlsx` crate SHALL define the `ExcelRowReader` trait with method `fn read_row(ws: &Worksheet, row: RowNum, headers: &[String]) -> Result<Self> where Self: Sized`
3. THE `zavora_xlsx` crate SHALL export both traits from the crate root
4. THE Derive_Macro SHALL generate implementations that reference `zavora_xlsx::ExcelRowWriter` and `zavora_xlsx::ExcelRowReader`

### Requirement 9: Error Handling

**User Story:** As a developer, I want reading errors to provide enough context to diagnose problems, so that I can fix data issues in my spreadsheets.

#### Acceptance Criteria

1. IF a required field's column header is not found in the headers slice during `read_row`, THEN THE ExcelRowReader SHALL return an error containing the missing header name
2. IF a cell value type does not match the expected Rust field type during `read_row`, THEN THE ExcelRowReader SHALL return an error containing the field name, expected type description, and actual CellValue variant
3. THE ExcelRowReader SHALL return errors as `zavora_xlsx::Error::InvalidData(String)`

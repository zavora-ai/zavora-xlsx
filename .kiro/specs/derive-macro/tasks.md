# Implementation Plan: zavora-xlsx-derive

## Overview

Implement the `#[derive(ExcelRow)]` proc macro across two crates: add `ExcelRowWriter` and `ExcelRowReader` traits to the core `zavora-xlsx` crate, then create the `zavora-xlsx-derive` proc-macro crate that generates implementations of those traits. Tasks are ordered so each step builds on the previous, with testing woven in close to the code it validates.

## Tasks

- [x] 1. Define ExcelRowWriter and ExcelRowReader traits in the core crate
  - [x] 1.1 Add trait definitions to a new `src/derive_traits.rs` module
    - Define `ExcelRowWriter` with `fn write_header(&self, ws: &mut Worksheet, row: RowNum) -> Result<()>` and `fn write_row(&self, ws: &mut Worksheet, row: RowNum) -> Result<()>`
    - Define `ExcelRowReader` with `fn read_row(ws: &Worksheet, row: RowNum, headers: &[String]) -> Result<Self> where Self: Sized`
    - _Requirements: 8.1, 8.2_
  - [x] 1.2 Export traits from `src/lib.rs`
    - Add `pub mod derive_traits;` to `src/lib.rs`
    - Add `pub use derive_traits::{ExcelRowWriter, ExcelRowReader};` to the re-exports section
    - _Requirements: 8.3_

- [x] 2. Scaffold the zavora-xlsx-derive proc-macro crate
  - [x] 2.1 Create `zavora-xlsx-derive/Cargo.toml`
    - Set `proc-macro = true` in `[lib]`
    - Add dependencies: `syn = { version = "2", features = ["full"] }`, `quote = "1"`, `proc-macro2 = "1"`
    - Add dev-dependencies: `zavora-xlsx = { path = ".." }`, `trybuild = "1"`, `proptest = "1"`
    - Do NOT add serde as a dependency
    - _Requirements: 1.1, 1.2, 1.3, 1.5_
  - [x] 2.2 Add `zavora-xlsx-derive` to workspace members in root `Cargo.toml`
    - _Requirements: 1.4_
  - [x] 2.3 Create `zavora-xlsx-derive/src/lib.rs` with the derive entry point stub
    - Add `#[proc_macro_derive(ExcelRow, attributes(excel))]` function that returns an empty `TokenStream` for now
    - _Requirements: 2.1_

- [x] 3. Implement attribute parser and field analyzer
  - [x] 3.1 Implement `FieldConfig` struct and `parse_field_attrs` function
    - Parse `#[excel(header = "...", format = "...", skip)]` using `syn::meta::ParseNestedMeta`
    - Validate: reject unknown keys, require string literals for header/format, reject skip combined with header or format
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_
  - [x] 3.2 Implement `FieldInfo` struct and field analysis logic
    - Extract field ident, type, and parsed config for each named field
    - Detect `Option<T>` by matching the type path last segment against `"Option"` and extracting the generic argument
    - Assign sequential 0-based column indices to non-skipped fields
    - Reject non-named-struct inputs (enums, tuple structs, unit structs) with descriptive compile errors
    - _Requirements: 2.2, 2.3_

- [x] 4. Checkpoint — Verify crate structure compiles
  - Ensure `cargo check --workspace` passes with the new trait definitions and proc-macro crate stub
  - Ensure all existing tests pass, ask the user if questions arise

- [x] 5. Implement writer code generation
  - [x] 5.1 Implement `write_header` code generation
    - For each non-skipped field, emit `ws.write(row, COL, "header_text")?;`
    - Use `#[excel(header = "...")]` value if present, otherwise use the field name
    - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - [x] 5.2 Implement `write_row` code generation
    - For `String` fields: emit `ws.write(row, col, &self.field)?;`
    - For numeric and bool fields: emit `ws.write(row, col, self.field)?;`
    - For `Option<T>` fields: emit match on Some/None, writing value or skipping
    - When `#[excel(format = "...")]` is present: create `Format::new().num_format(...)` and use `ws.write_with_format`
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7_
  - [x] 5.3 Wire `write_header` and `write_row` into the derive entry point
    - Combine attribute parsing, field analysis, and writer codegen into the `impl_excel_row` function
    - Generate `impl zavora_xlsx::ExcelRowWriter for StructName { ... }`
    - _Requirements: 8.4_

- [x] 6. Implement reader code generation
  - [x] 6.1 Implement `read_row` code generation
    - For each non-skipped field, search `headers` slice for the field's header name to find column index
    - Read cell with `ws.read_cell(row, col_index)` and convert `CellValue` to target type
    - For `String` fields: convert from String/Number/Bool/Empty variants
    - For numeric fields: extract from `CellValue::Number`, cast to target type
    - For `bool` fields: extract from `CellValue::Bool`
    - For `Option<T>` fields: return `None` on `CellValue::Empty` or missing header, `Some(converted)` otherwise
    - For skipped fields: use `Default::default()`
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.8, 6.1, 6.2, 6.3_
  - [x] 6.2 Implement error handling in reader codegen
    - Missing required header → `Error::InvalidData("missing column header '{header}' for field '{field}'")`
    - Type mismatch → `Error::InvalidData("field '{field}': expected {type_desc}, got {variant}")`
    - _Requirements: 9.1, 9.2, 9.3, 5.7_
  - [x] 6.3 Wire `read_row` into the derive entry point
    - Add reader codegen to `impl_excel_row`, generating `impl zavora_xlsx::ExcelRowReader for StructName { ... }`
    - _Requirements: 8.4_

- [x] 7. Checkpoint — Verify full derive macro compiles and generates code
  - Ensure `cargo check --workspace` passes
  - Ensure all existing tests pass, ask the user if questions arise

- [x] 8. Add compile-fail tests with trybuild
  - [x] 8.1 Create trybuild test harness and compile-fail test cases
    - Create `zavora-xlsx-derive/tests/compile_tests.rs` with `trybuild::TestCases`
    - Add test cases in `zavora-xlsx-derive/tests/compile-fail/`:
      - `tuple_struct.rs` — derive on tuple struct → error (Req 2.2)
      - `unit_struct.rs` — derive on unit struct → error (Req 2.2)
      - `enum_derive.rs` — derive on enum → error (Req 2.2)
      - `unknown_attr.rs` — unknown attribute key → error (Req 7.1)
      - `non_string_header.rs` — non-string header value → error (Req 7.2)
      - `non_string_format.rs` — non-string format value → error (Req 7.3)
      - `skip_with_header.rs` — skip + header → error (Req 7.5)
      - `skip_with_format.rs` — skip + format → error (Req 7.5)
    - _Requirements: 2.2, 7.1, 7.2, 7.3, 7.5_

- [x] 9. Add integration tests for write and read round-trip
  - [x] 9.1 Create integration test file with basic round-trip tests
    - Create `zavora-xlsx-derive/tests/integration_tests.rs`
    - Define test structs with `#[derive(ExcelRow)]` covering: String, numeric types, bool, Option<T>, custom headers, format attributes, skip fields
    - Test `write_header` + `write_row` then `read_row` produces equal struct values
    - Test format attribute applies `num_format` correctly (Req 4.7)
    - Test combined header + format attribute works (Req 7.4)
    - _Requirements: 2.1, 3.1, 3.2, 3.3, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.8, 6.1, 6.2, 7.4, 8.1, 8.2, 8.3, 8.4_
  - [ ]* 9.2 Write property test: round-trip preserves field values (Property 1)
    - **Property 1: Write-read round trip preserves field values**
    - Use `proptest` to generate random field values for an `AllTypes` test struct
    - Write with `write_header` + `write_row`, read back with `read_row`, assert equality
    - **Validates: Requirements 2.1, 2.3, 3.1, 3.2, 3.3, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6**
  - [ ]* 9.3 Write property test: skipped fields excluded and default on read (Property 2)
    - **Property 2: Skipped fields are excluded from Excel and default on read**
    - Verify header count equals non-skipped field count, skipped fields have Default values after read
    - **Validates: Requirements 6.1, 6.2**
  - [ ]* 9.4 Write property test: missing header produces error (Property 3)
    - **Property 3: Missing header produces error with header name**
    - Generate random subsets of headers with at least one required header removed
    - Verify `read_row` returns `Err` whose message contains the missing header name
    - **Validates: Requirements 9.1**
  - [ ]* 9.5 Write property test: type mismatch produces descriptive error (Property 4)
    - **Property 4: Type mismatch produces descriptive error**
    - Write incompatible CellValue variants for typed fields, verify error contains field name, expected type, and actual variant
    - **Validates: Requirements 5.7, 9.2, 9.3**
  - [ ]* 9.6 Write property test: header column order independence (Property 5)
    - **Property 5: Header column order independence**
    - Generate random permutations of headers, verify `read_row` produces the same struct values regardless of order
    - **Validates: Requirements 5.8**

- [x] 10. Final checkpoint — Ensure all tests pass
  - Run `cargo test --workspace` and verify everything passes
  - Ensure all existing tests still pass, ask the user if questions arise

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties from the design document
- The derive macro crate has no runtime dependency on zavora-xlsx — generated code references `zavora_xlsx::` paths resolved at user compile time
- trybuild tests require `.stderr` files matching expected compiler output; these are auto-generated on first run with `TRYBUILD=overwrite`

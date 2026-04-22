# Implementation Plan: zavora-xlsx-cli

## Overview

Build a CLI binary crate (`zavora-xlsx-cli`) that wraps the `zavora-xlsx` library to provide `inspect`, `export`, and `convert` subcommands. The crate uses `clap` derive macros for argument parsing, prints errors to stderr, and uses exit codes for scripting compatibility. Implementation proceeds incrementally: crate scaffolding → argument parsing → inspect → export → convert → error handling polish.

## Tasks

- [x] 1. Scaffold the binary crate and workspace integration
  - [x] 1.1 Create `zavora-xlsx-cli/Cargo.toml` with `clap` (version 4, `derive` feature) and `zavora-xlsx` (path dependency) as dependencies, binary target named `zavora-xlsx`, and dev-dependencies (`proptest`, `tempfile`, `assert_cmd`, `predicates`)
    - _Requirements: 1.1, 1.2_
  - [x] 1.2 Add `"zavora-xlsx-cli"` to the `[workspace]` members list in the root `Cargo.toml`
    - _Requirements: 1.3_
  - [x] 1.3 Create `zavora-xlsx-cli/src/main.rs` with the `Cli` struct, `Commands` enum (Inspect, Export, Convert variants with all clap args), and a `main()` that parses args and dispatches to stub handler functions returning `Result<(), Box<dyn std::error::Error>>`
    - Define `Cli` with `#[command(name = "zavora-xlsx", version, about)]`
    - Define `Commands::Inspect { file }`, `Commands::Export { file, sheet, sheet_index, output, delimiter, tsv, date_format }`, `Commands::Convert { file, format, sheet, output, delimiter }`
    - Wire `main()` to match on the subcommand and call `cmd_inspect`, `cmd_export`, `cmd_convert` stubs
    - Print errors to stderr with `eprintln!` and exit with code 1 on `Err`
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 6.1, 6.2, 6.3, 6.4_

- [x] 2. Implement the inspect subcommand
  - [x] 2.1 Implement `cmd_inspect(file: &str)` that opens the file with `Workbook::open_readonly`, iterates `sheet_names()`, calls `worksheet_ref(i)` to get `used_range()`, and prints each sheet name with row/column counts, total sheet count, and document properties (title, author, subject, keywords, category, company) when present
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  - [ ]* 2.2 Write property test for inspect output completeness
    - **Property 1: Inspect output completeness**
    - Create workbooks with generated sheet names, dimensions, and random subsets of document properties; run `cmd_inspect` logic and verify output contains all sheet names with correct dimensions, total count, and exactly the set properties
    - **Validates: Requirements 3.2, 3.3, 3.4**

- [x] 3. Implement the export subcommand
  - [x] 3.1 Implement `cmd_export` that opens the file with `Workbook::open_readonly`, resolves the target sheet (by `--sheet` name, `--sheet-index`, or default index 0), builds `CsvOptions` from flags (`--tsv`, `--delimiter`, `--date-format`), and writes output via `to_csv_file` or `to_csv_string` to stdout
    - Use `worksheet_ref()` for read-only access
    - Return descriptive errors for missing sheet name/index
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 4.9, 4.10_
  - [ ]* 3.2 Write property test for export sheet selection correctness
    - **Property 2: Export sheet selection correctness**
    - Create multi-sheet workbooks with distinct data per sheet; export by name and by index; verify output matches only the selected sheet's data
    - **Validates: Requirements 4.2, 4.3**
  - [ ]* 3.3 Write property test for export delimiter passthrough
    - **Property 3: Export delimiter passthrough**
    - Generate multi-column workbooks and arbitrary single-byte delimiters; export with `--delimiter`; verify fields are separated by exactly that delimiter
    - **Validates: Requirements 4.6**

- [x] 4. Checkpoint - Verify inspect and export
  - Ensure all tests pass, ask the user if questions arise.

- [x] 5. Implement the convert subcommand (xlsx/xlsm → csv)
  - [x] 5.1 Implement the xlsx-to-csv conversion path in `cmd_convert`: open with `open_readonly`, export all sheets (or single `--sheet`) to CSV files named `<sheet_name>.csv` in the output directory, deriving output directory from input filename when `--output` is not provided
    - Use `to_csv_file` with `CsvOptions` built from `--delimiter`
    - _Requirements: 5.1, 5.2, 5.5, 5.6, 5.8_
  - [ ]* 5.2 Write property test for convert xlsx-to-csv file naming
    - **Property 4: Convert xlsx-to-csv file naming**
    - Create workbooks with N uniquely-named sheets; convert to CSV without `--sheet`; verify exactly N files produced, each named `<sheet_name>.csv`
    - **Validates: Requirements 5.1**

- [x] 6. Implement the convert subcommand (csv → xlsx)
  - [x] 6.1 Implement CSV parsing helpers: `parse_csv_line(line, delimiter)` for splitting fields (handling quoted fields with escaped quotes) and `detect_cell_value(field)` for type detection (f64 → Number, TRUE/FALSE case-insensitive → Bool, else → String)
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_
  - [x] 6.2 Implement the csv-to-xlsx conversion path in `cmd_convert`: read CSV line-by-line, parse fields with the configured delimiter, detect types, write cells into a new `Workbook`, and save as `.xlsx`
    - Derive output filename by replacing `.csv` extension with `.xlsx` when `--output` is not provided
    - _Requirements: 5.3, 5.5, 5.6, 7.1, 7.6_
  - [ ]* 6.3 Write property test for CSV-to-xlsx data round-trip
    - **Property 5: CSV-to-xlsx data round-trip**
    - Generate CSV data with mixed numeric, boolean, and string values; convert to xlsx; read back and verify all cell values and types are preserved
    - **Validates: Requirements 5.3, 7.2, 7.3, 7.4**
  - [ ]* 6.4 Write property test for output filename derivation
    - **Property 6: Output filename derivation**
    - Generate input filenames with recognized extensions (.xlsx, .xlsm, .csv) and compatible target formats; verify derived output filename equals input with extension replaced
    - **Validates: Requirements 5.6**
  - [ ]* 6.5 Write property test for CSV custom delimiter parsing
    - **Property 7: CSV custom delimiter parsing**
    - Generate CSV data using non-comma delimiters; convert to xlsx with matching `--delimiter`; verify column count and cell values match original data
    - **Validates: Requirements 7.5**

- [x] 7. Implement the convert subcommand (xlsx → xlsm)
  - [x] 7.1 Implement the xlsx-to-xlsm conversion path in `cmd_convert`: open with `Workbook::open(file)` (edit mode), call `save_as_xlsm(path, &vba_project)`. Since a plain xlsx has no VBA project, return a descriptive error explaining that xlsx→xlsm conversion requires VBA data
    - Derive output filename by replacing `.xlsx` with `.xlsm` when `--output` is not provided
    - Handle incompatible format errors (e.g., csv→xlsm)
    - _Requirements: 5.4, 5.5, 5.6, 5.7_

- [x] 8. Checkpoint - Verify all convert paths
  - Ensure all tests pass, ask the user if questions arise.

- [x] 9. Polish error handling and integration tests
  - [x] 9.1 Add integration tests using `assert_cmd` that run the compiled binary as a subprocess: verify `--help` exits 0, `--version` exits 0, no-args exits non-zero, invalid subcommand exits 2, inspect on a valid fixture, export with various flags, convert paths, and error cases (bad file, bad sheet) exit 1 with stderr output
    - Create a small xlsx test fixture in the test setup
    - _Requirements: 2.2, 2.3, 2.4, 3.5, 4.9, 4.10, 5.7, 5.8, 6.1, 6.2, 6.3, 6.4, 7.6_

- [x] 10. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests use `proptest` crate and validate universal correctness properties from the design
- Unit tests validate specific examples and edge cases
- The CLI uses Rust throughout, matching the existing `zavora-xlsx` codebase
- All subcommand handlers return `Result<(), Box<dyn std::error::Error>>` for consistent error propagation

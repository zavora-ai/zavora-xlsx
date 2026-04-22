# Requirements Document

## Introduction

The `zavora-xlsx-cli` tool is a command-line interface for inspecting, exporting, and converting Excel `.xlsx` files. It is a separate binary crate (workspace member) that wraps the core `zavora-xlsx` library. The tool provides three main subcommands — `inspect`, `export`, and `convert` — and is installable via `cargo install`. It uses `clap` derive macros for argument parsing, prints errors to stderr, and exits with a non-zero status code on failure.

## Glossary

- **CLI**: The `zavora-xlsx-cli` binary application
- **Workbook**: An Excel `.xlsx` file opened via the `zavora-xlsx` core library
- **Sheet**: A single worksheet within a Workbook
- **Inspect_Command**: The `inspect` subcommand that displays Workbook metadata and sheet information
- **Export_Command**: The `export` subcommand that exports a Sheet to CSV or TSV format
- **Convert_Command**: The `convert` subcommand that converts files between xlsx, xlsm, and csv formats
- **CsvOptions**: The CSV export configuration from the core library, controlling delimiter, quote character, line ending, and date format
- **Used_Range**: The bounding rectangle of non-empty cells in a Sheet, expressed as (min_row, min_col, max_row, max_col)
- **Exit_Code**: The process exit status; 0 indicates success, non-zero indicates failure

## Requirements

### Requirement 1: Binary Crate Structure

**User Story:** As a developer, I want `zavora-xlsx-cli` to be a separate workspace member binary crate, so that I can install it independently via `cargo install`.

#### Acceptance Criteria

1. THE CLI SHALL be defined as a binary crate in a `zavora-xlsx-cli/` directory at the workspace root
2. THE CLI SHALL declare `clap` with the `derive` feature and `zavora-xlsx` as a path dependency in its `Cargo.toml`
3. THE CLI SHALL be registered as a workspace member in the root `Cargo.toml`
4. WHEN a user runs `cargo install --path zavora-xlsx-cli`, THE CLI SHALL produce a binary named `zavora-xlsx`

### Requirement 2: Top-Level Argument Parsing

**User Story:** As a user, I want the CLI to use clap derive macros for argument parsing with clear help text, so that I can discover available subcommands and options.

#### Acceptance Criteria

1. THE CLI SHALL parse arguments using clap derive macros with a top-level struct containing a subcommand enum
2. WHEN the user provides no arguments, THE CLI SHALL display a help message listing available subcommands and exit with a non-zero Exit_Code
3. WHEN the user provides the `--help` flag, THE CLI SHALL display a help message describing the tool and its subcommands and exit with Exit_Code 0
4. WHEN the user provides the `--version` flag, THE CLI SHALL display the crate version and exit with Exit_Code 0

### Requirement 3: Inspect Subcommand

**User Story:** As a user, I want to inspect an Excel file to see sheet names, dimensions, and document metadata, so that I can understand the file contents before processing.

#### Acceptance Criteria

1. WHEN the user runs `inspect <file>`, THE Inspect_Command SHALL open the file in read-only mode using `Workbook::open_readonly`
2. WHEN the file is opened successfully, THE Inspect_Command SHALL print each Sheet name along with its row count and column count derived from the Used_Range
3. WHEN the file is opened successfully, THE Inspect_Command SHALL print document properties including title, author, subject, keywords, category, and company when those properties are present
4. WHEN the file is opened successfully, THE Inspect_Command SHALL print the total number of sheets in the Workbook
5. IF the file cannot be opened, THEN THE Inspect_Command SHALL print a descriptive error message to stderr and exit with a non-zero Exit_Code

### Requirement 4: Export Subcommand

**User Story:** As a user, I want to export a worksheet to CSV or TSV format with configurable options, so that I can use spreadsheet data in other tools.

#### Acceptance Criteria

1. WHEN the user runs `export <file>`, THE Export_Command SHALL open the file in read-only mode and export the first Sheet to CSV by default
2. WHEN the user provides a `--sheet <name>` option, THE Export_Command SHALL export the specified Sheet by name
3. WHEN the user provides a `--sheet-index <n>` option, THE Export_Command SHALL export the Sheet at the specified zero-based index
4. WHEN the user provides an `--output <path>` option, THE Export_Command SHALL write the CSV output to the specified file path using `to_csv_file`
5. WHEN the user does not provide an `--output` option, THE Export_Command SHALL write the CSV output to stdout
6. WHEN the user provides a `--delimiter <char>` option, THE Export_Command SHALL use the specified single-byte character as the field delimiter
7. WHEN the user provides a `--tsv` flag, THE Export_Command SHALL use a tab character as the field delimiter
8. WHEN the user provides a `--date-format <fmt>` option, THE Export_Command SHALL use the specified date format string in CsvOptions
9. IF the specified Sheet name or index does not exist, THEN THE Export_Command SHALL print a descriptive error message to stderr and exit with a non-zero Exit_Code
10. IF the file cannot be opened, THEN THE Export_Command SHALL print a descriptive error message to stderr and exit with a non-zero Exit_Code

### Requirement 5: Convert Subcommand

**User Story:** As a user, I want to convert files between xlsx, csv, and xlsm formats, so that I can transform spreadsheet data for different workflows.

#### Acceptance Criteria

1. WHEN the user runs `convert <file> --format csv`, THE Convert_Command SHALL open the xlsx file and export all sheets to CSV files in the output directory, naming each file `<sheet_name>.csv`
2. WHEN the user runs `convert <file> --format csv` with a `--sheet <name>` option, THE Convert_Command SHALL export only the specified Sheet to a single CSV file
3. WHEN the user runs `convert <file> --format xlsx` with a CSV input file, THE Convert_Command SHALL create a new Workbook with one Sheet containing the parsed CSV data
4. WHEN the user runs `convert <file> --format xlsm` with an xlsx input file, THE Convert_Command SHALL convert the file to xlsm format using `save_as_xlsm`
5. WHEN the user provides an `--output <path>` option, THE Convert_Command SHALL write the converted file to the specified path
6. WHEN the user does not provide an `--output` option, THE Convert_Command SHALL derive the output filename from the input filename by changing the extension to match the target format
7. IF the input file format is incompatible with the target format, THEN THE Convert_Command SHALL print a descriptive error message to stderr and exit with a non-zero Exit_Code
8. IF the input file cannot be opened, THEN THE Convert_Command SHALL print a descriptive error message to stderr and exit with a non-zero Exit_Code

### Requirement 6: Error Handling and Exit Codes

**User Story:** As a user, I want clear error messages on stderr and meaningful exit codes, so that I can use the CLI in scripts and automation pipelines.

#### Acceptance Criteria

1. WHEN any operation fails, THE CLI SHALL print the error message to stderr using the `eprintln!` macro
2. WHEN any operation fails, THE CLI SHALL exit with Exit_Code 1 using `std::process::exit`
3. WHEN all operations succeed, THE CLI SHALL exit with Exit_Code 0
4. WHEN the user provides an invalid subcommand or missing required arguments, THE CLI SHALL display a clap-generated error message to stderr and exit with Exit_Code 2

### Requirement 7: CSV Parsing for Conversion

**User Story:** As a user, I want to convert CSV files to xlsx format, so that I can import tabular data into Excel-compatible workflows.

#### Acceptance Criteria

1. WHEN parsing a CSV input file for conversion, THE Convert_Command SHALL read the file using a line-by-line reader that splits fields by comma delimiter
2. WHEN parsing a CSV input file, THE Convert_Command SHALL detect numeric values and write them as numbers in the output Workbook
3. WHEN parsing a CSV input file, THE Convert_Command SHALL detect boolean values (`TRUE`, `FALSE`, case-insensitive) and write them as booleans in the output Workbook
4. WHEN parsing a CSV input file, THE Convert_Command SHALL write all other field values as strings in the output Workbook
5. WHEN the user provides a `--delimiter <char>` option with csv-to-xlsx conversion, THE Convert_Command SHALL use the specified delimiter for parsing the CSV input
6. IF the CSV input file cannot be read, THEN THE Convert_Command SHALL print a descriptive error message to stderr and exit with a non-zero Exit_Code

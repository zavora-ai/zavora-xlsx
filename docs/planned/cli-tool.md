# zavora-xlsx-cli

> **Status: Planned** — This crate will be a separate workspace member.

Command-line tool for inspecting, exporting, and converting Excel files.

## Planned Features

- `inspect` subcommand: sheet names, row/col counts, metadata
- `export` subcommand: sheet to CSV with options
- `convert` subcommand: xlsx ↔ xlsm ↔ csv
- Descriptive errors to stderr, non-zero exit on failure

## Example Usage (planned)

```bash
# Inspect a workbook
zavora-xlsx inspect report.xlsx
# Output:
#   Sheets: Sheet1 (1000 rows × 10 cols), Summary (50 rows × 5 cols)
#   Author: Jane Doe
#   Created: 2024-01-15

# Export a sheet to CSV
zavora-xlsx export report.xlsx --sheet Sheet1 --output data.csv

# Convert xlsx to CSV
zavora-xlsx convert report.xlsx --format csv --output report.csv

# Convert CSV to xlsx
zavora-xlsx convert data.csv --format xlsx --output data.xlsx
```

## Architecture

This crate uses `clap` for argument parsing and the core `zavora-xlsx`
crate for file operations. It will be a separate binary crate in the
workspace.

## Dependencies

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
zavora-xlsx = { path = ".." }
```

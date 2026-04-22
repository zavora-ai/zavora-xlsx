# zavora-xlsx-cli

> **Status: ✅ Implemented** — Available as the `zavora-xlsx-cli` workspace member.

Command-line tool for inspecting, exporting, and converting Excel files.

## Installation

```bash
# From the workspace root
cargo install --path zavora-xlsx-cli

# Or run directly
cargo run -p zavora-xlsx-cli -- <subcommand>
```

## Subcommands

### `inspect` — Show workbook metadata

Displays sheet names, dimensions, and document properties.

```bash
zavora-xlsx inspect report.xlsx
```

Output:
```
  Sheet1: 1000 rows x 10 cols
  Summary: 50 rows x 5 cols
Sheets: 2
Title: Quarterly Report
Author: Jane Doe
```

### `export` — Export a sheet to CSV/TSV

Export a worksheet to delimited text (CSV by default, TSV with `--tsv`).

```bash
# Export first sheet to stdout
zavora-xlsx export report.xlsx

# Export specific sheet to a file
zavora-xlsx export report.xlsx --sheet Sheet1 --output data.csv

# Export as TSV
zavora-xlsx export report.xlsx --tsv

# Export by sheet index
zavora-xlsx export report.xlsx --sheet-index 2

# Custom delimiter and date format
zavora-xlsx export report.xlsx --delimiter '|' --date-format "yyyy-mm-dd"
```

**Flags:**

| Flag | Description |
|------|-------------|
| `--sheet <name>` | Sheet name to export (default: first sheet) |
| `--sheet-index <n>` | Zero-based sheet index to export |
| `-o, --output <path>` | Output file path (default: stdout) |
| `--delimiter <char>` | Field delimiter character (default: `,`) |
| `--tsv` | Use tab as delimiter |
| `--date-format <fmt>` | Date format string |

### `convert` — Convert between formats

Convert between xlsx, xlsm, and CSV formats.

```bash
# xlsx → CSV (all sheets, one CSV per sheet)
zavora-xlsx convert report.xlsx --format csv --output output_dir/

# xlsx → CSV (single sheet)
zavora-xlsx convert report.xlsx --format csv --sheet Sheet1 --output output_dir/

# CSV → xlsx
zavora-xlsx convert data.csv --format xlsx --output data.xlsx

# xlsx → xlsm (requires VBA project in source)
zavora-xlsx convert report.xlsx --format xlsm --output report.xlsm
```

**Flags:**

| Flag | Description |
|------|-------------|
| `--format <fmt>` | Target format: `csv`, `xlsx`, `xlsm` |
| `--sheet <name>` | Sheet name (for single-sheet CSV export) |
| `-o, --output <path>` | Output file or directory path |
| `--delimiter <char>` | Field delimiter for CSV operations |

## CSV → xlsx Type Detection

When converting CSV to xlsx, the CLI automatically detects cell value types:

- **Numbers**: Parsed as `f64` (e.g., `42`, `3.14`, `-7.5`)
- **Booleans**: Case-insensitive `TRUE`/`FALSE`
- **Strings**: Everything else

## Architecture

The CLI is built with `clap` for argument parsing and uses the core `zavora-xlsx` crate for all file operations. It is a separate binary crate in the workspace.

## Dependencies

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
zavora-xlsx = { path = ".." }
```

## Demo

A demo shell script is available at `zavora-xlsx-cli/examples/cli_demo.sh`:

```bash
bash zavora-xlsx-cli/examples/cli_demo.sh
```

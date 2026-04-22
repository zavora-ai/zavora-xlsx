# zavora-xlsx-cli

Command-line tool for inspecting, exporting, and converting Excel `.xlsx` files.

Part of the [zavora-xlsx](https://github.com/zavora-ai/zavora-xlsx) workspace.

## Install

```bash
cargo install zavora-xlsx-cli
```

The binary is named `zavora-xlsx`.

## Subcommands

### inspect

Display sheet names, dimensions, and document metadata.

```bash
$ zavora-xlsx inspect report.xlsx
  Sheet1: 1000 rows x 10 cols
  Summary: 50 rows x 5 cols
Sheets: 2
Title: Quarterly Report
Author: Jane Doe
```

### export

Export a single worksheet to CSV or TSV. Writes to stdout by default.

```bash
# Default: first sheet to stdout as CSV
zavora-xlsx export report.xlsx

# Specific sheet, to a file
zavora-xlsx export report.xlsx --sheet Sales --output sales.csv

# By zero-based index
zavora-xlsx export report.xlsx --sheet-index 2 --output sheet3.csv

# TSV mode
zavora-xlsx export report.xlsx --tsv

# Custom delimiter and date format
zavora-xlsx export report.xlsx --delimiter '|' --date-format "dd/mm/yyyy"
```

| Flag | Description | Default |
|------|-------------|---------|
| `--sheet <name>` | Sheet name to export | First sheet |
| `--sheet-index <n>` | Zero-based sheet index | 0 |
| `-o, --output <path>` | Output file path | stdout |
| `--delimiter <char>` | Field delimiter | `,` |
| `--tsv` | Use tab delimiter | |
| `--date-format <fmt>` | Date format string | `yyyy-mm-dd` |

### convert

Convert between xlsx, xlsm, and CSV formats.

```bash
# xlsx → CSV (one file per sheet)
zavora-xlsx convert report.xlsx --format csv --output csv_dir/
# Creates csv_dir/Sheet1.csv, csv_dir/Summary.csv, etc.

# xlsx → CSV (single sheet)
zavora-xlsx convert report.xlsx --format csv --sheet Sales

# CSV → xlsx (auto-detects numbers, booleans, strings)
zavora-xlsx convert data.csv --format xlsx
# Creates data.xlsx

# CSV → xlsx with custom delimiter
zavora-xlsx convert data.tsv --format xlsx --delimiter '\t'

# xlsx → xlsm (requires VBA project in source file)
zavora-xlsx convert macros.xlsx --format xlsm
```

**CSV → xlsx type detection:**

| Input | Detected as |
|-------|-------------|
| `42`, `3.14`, `-7.5` | Number |
| `TRUE`, `false`, `True` | Boolean |
| Everything else | String |

## Error handling

Errors print to stderr and exit with code 1. Argument errors exit with code 2 (clap default). Success exits with code 0.

```bash
$ zavora-xlsx inspect nonexistent.xlsx
Error: IO error: No such file or directory (os error 2)
$ echo $?
1
```

## License

Apache-2.0

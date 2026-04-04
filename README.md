# zavora-xlsx

High-performance Rust crate for reading, writing, and editing Excel `.xlsx` files.

## Features

- **Read** — Stream cells from existing xlsx files with minimal memory
- **Write** — Create new xlsx files with full formatting, charts, images, tables
- **Edit** — Open existing files, modify cells/sheets, save with unmodified sheets passed through as raw bytes
- **Streaming write** — Constant-memory mode for million-row files
- **Parallel assembly** — Multi-threaded sheet XML generation on save

### Cell Types
- Strings, numbers, booleans, dates, formulas, errors, rich text

### Formatting
- Font (bold, italic, size, color, name, underline)
- Background color and patterns
- Borders (all sides, styles, colors)
- Number formats
- Alignment (horizontal, vertical, wrap, indent)
- Cell-level format dedup via xf index (4 bytes per cell, not 200)

### Rich Features
- **Charts** — Bar, Column, Line, Pie, Scatter, Area, Doughnut, Radar
- **Tables** — Columns, styles (Light/Medium/Dark), autofilter, totals
- **Images** — PNG/JPEG with auto dimension detection
- **Conditional formatting** — Cell value, 2/3-color scale, data bar, icon set
- **Data validation** — List, range, whole number, decimal, date, text length, custom
- **Sparklines** — Line, Column, Win/Loss

### Sheet Operations
- Add, remove, rename, reorder sheets
- Insert/remove rows and columns with formula adjustment
- Merge cells, freeze panes, column widths, row heights
- Autofit column widths
- Sheet and workbook protection with password
- Print settings (margins, page setup, headers/footers, page breaks)
- Defined names

### File Format Support
- `.xlsx` (standard)
- `.xlsm` (macro-enabled, VBA passthrough on edit)
- Password-protected file detection

## Quick Start

```rust
use zavora_xlsx::{Workbook, Format, Color};

let mut wb = Workbook::new();
let ws = wb.worksheet_mut(0).unwrap();

// Write data
ws.write(0, 0, "Hello")?;
ws.write(0, 1, 42.5)?;
ws.write(0, 2, true)?;

// With formatting
let bold = Format::new().bold();
ws.write_with_format(1, 0, "Bold text", &bold)?;

// Save
wb.save("output.xlsx")?;
```

## Edit Mode

```rust
use zavora_xlsx::Workbook;

let mut wb = Workbook::open("existing.xlsx")?;
let ws = wb.worksheet_mut(0).unwrap();
ws.write(0, 0, "Updated")?;
wb.save("modified.xlsx")?;
```

## Streaming Mode (Constant Memory)

```rust
use zavora_xlsx::StreamingWorkbook;

let mut wb = StreamingWorkbook::new();
for row in 0..1_000_000u32 {
    wb.write_string(row, 0, &format!("Row {row}"))?;
    wb.write_number(row, 1, row as f64)?;
}
wb.save("big.xlsx")?;
```

## Dependencies

Only 4 runtime dependencies:
- `quick-xml` — Streaming XML parsing/writing
- `zip` — ZIP archive handling
- `atoi_simd` — SIMD-accelerated integer parsing
- `fast-float2` — SIMD-accelerated float parsing

No serde, no chrono, no tokio.

## Coordinates

All coordinates are 0-based `(row: u32, col: u16)`:
- Row 0 = Excel row 1
- Col 0 = Column A

A1 notation helpers are provided via `utility::parse_cell_reference()` and `utility::col_to_letter()`.

## License

MIT OR Apache-2.0

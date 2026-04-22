# Getting Started with zavora-xlsx

A quick start guide for reading, writing, and editing Excel `.xlsx` files in Rust.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
zavora-xlsx = "0.1"
```

## Creating Your First Workbook

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    ws.write(0, 0, "Hello, Excel!")?;
    ws.write(0, 1, 42.5)?;
    ws.write(0, 2, true)?;

    wb.save("hello.xlsx")?;
    Ok(())
}
```

All coordinates are **0-based**: row 0 = Excel row 1, col 0 = column A.

## Writing Different Cell Types

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // String
    ws.write(0, 0, "Name")?;

    // Number (f64, f32, i8–i64, u8–u64 all work)
    ws.write(1, 0, 99.95)?;

    // Boolean
    ws.write(2, 0, true)?;

    // Formula
    ws.write_formula(3, 0, "SUM(A1:A3)")?;

    // Formula with cached result (opens faster in Excel)
    ws.write_formula_with_result(4, 0, "A2*2", 199.9)?;

    // Date (ExcelDateTime)
    let date = ExcelDateTime::from_ymd(2024, 6, 15).unwrap();
    ws.write(5, 0, date)?;

    // Date with time
    let datetime = ExcelDateTime::from_ymd_hms(2024, 6, 15, 14, 30, 0).unwrap();
    ws.write(6, 0, datetime)?;

    // Parse from ISO string
    let parsed = ExcelDateTime::parse("2024-06-15T14:30:00").unwrap();
    ws.write(7, 0, parsed)?;

    wb.save("cell_types.xlsx")?;
    Ok(())
}
```

## Basic Formatting

`Format` uses a builder pattern — all methods consume and return `Self`:

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Bold header
    let header = Format::new().bold().font_size(14.0).font_color("#FFFFFF").background_color("#4472C4");
    ws.write_with_format(0, 0, "Revenue", &header)?;

    // Currency format
    let currency = Format::new().num_format("$#,##0.00");
    ws.write_with_format(1, 0, 1234.56, &currency)?;

    // Date format
    let date_fmt = Format::new().num_format("yyyy-mm-dd");
    let dt = ExcelDateTime::from_ymd(2024, 6, 15).unwrap();
    ws.write_with_format(2, 0, dt, &date_fmt)?;

    // Borders and alignment
    let bordered = Format::new()
        .border(BorderStyle::Thin)
        .align(Align::Center)
        .align(Align::VerticalCenter);
    ws.write_with_format(3, 0, "Centered", &bordered)?;

    wb.save("formatted.xlsx")?;
    Ok(())
}
```

## Batch Writing

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Write a row of values starting at (0, 0)
    ws.write_row(0, 0, ["Q1", "Q2", "Q3", "Q4"])?;

    // Write a column of values starting at (1, 0)
    ws.write_column(1, 0, [100, 200, 300, 400])?;

    wb.save("batch.xlsx")?;
    Ok(())
}
```

## Saving to File and Buffer

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.write(0, 0, "data")?;

    // Save to file
    wb.save("output.xlsx")?;

    // Save to in-memory buffer (useful for web servers, WASM, etc.)
    let bytes: Vec<u8> = wb.save_to_buffer()?;

    Ok(())
}
```

## Opening Existing Files

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    // Read-only (no edit, no save)
    let mut wb = Workbook::open_readonly("data.xlsx")?;
    let ws = wb.worksheet(0)?;
    let val = ws.read_cell(0, 0);
    println!("{:?}", val);

    // Edit mode (open → modify → save)
    let mut wb = Workbook::open("data.xlsx")?;
    let ws = wb.worksheet(0)?;
    ws.write(0, 0, "Updated")?;
    wb.save("modified.xlsx")?;

    // From in-memory buffer
    let bytes = std::fs::read("data.xlsx")?;
    let mut wb = Workbook::open_readonly_from_buffer(&bytes)?;
    let mut wb = Workbook::open_from_buffer(&bytes)?;

    Ok(())
}
```

## Reading Cell Values

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::open_readonly("data.xlsx")?;
    let ws = wb.worksheet(0)?;

    let val = ws.read_cell(0, 0);
    match val {
        CellValue::String(s) => println!("String: {s}"),
        CellValue::Number(n) => println!("Number: {n}"),
        CellValue::Bool(b) => println!("Bool: {b}"),
        CellValue::DateTime(dt) => println!("Date: {}", dt.to_iso_string()),
        CellValue::Formula { formula, cached_value } => {
            println!("Formula: {formula}, cached: {cached_value:?}");
        }
        CellValue::Error(e) => println!("Error: {e}"),
        CellValue::RichText(rt) => println!("Rich text: {}", rt.plain_text()),
        CellValue::Empty => println!("Empty"),
    }

    // Sheet metadata
    println!("Sheets: {:?}", wb.sheet_names());
    println!("Count: {}", wb.sheet_count());

    if let Some((r1, c1, r2, c2)) = ws.used_range() {
        println!("Used range: ({r1},{c1}) to ({r2},{c2})");
    }

    Ok(())
}
```

## Feature Flags

| Feature | Description | Dependency |
|---------|-------------|------------|
| `mmap` | Memory-mapped file I/O | `memmap2` |
| `serde-support` | Serialize/deserialize rows via serde | `serde` |
| `async-tokio` | Async file I/O | `tokio` |
| `wasm` | WASM target support | — |
| `cffi` | C FFI bindings | — |
| `xlsb` | XLSB format reading (experimental) | — |
| `xls` | Legacy XLS format reading (experimental) | — |
| `ods` | ODS format support (experimental) | — |
| `crypto` | Encrypted workbook support (experimental) | — |

Enable features in `Cargo.toml`:

```toml
[dependencies]
zavora-xlsx = { version = "0.1", features = ["serde-support", "async-tokio"] }
```

## Core Dependencies

| Crate | Purpose |
|-------|---------|
| `quick-xml` | XML parsing and writing |
| `zip` | ZIP archive read/write (deflate) |
| `atoi_simd` | SIMD-accelerated integer parsing |
| `fast-float2` | Fast float parsing |

No heavy frameworks — just 4 runtime dependencies.

## Next Steps

- [Formatting Guide](formatting.md) — fonts, colors, borders, alignment, number formats
- [Charts Guide](charts.md) — all chart types with series configuration
- [Tables & Data](tables-and-data.md) — tables, validation, conditional formatting
- [Reading Guide](reading.md) — streaming reader, cell formats, metadata
- [Advanced Features](advanced.md) — streaming write, pivot tables, images, protection
- [Serde Integration](serde.md) — serialize/deserialize structs to rows

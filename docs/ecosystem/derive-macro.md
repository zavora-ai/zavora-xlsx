# zavora-xlsx-derive

> **Status: ✅ Implemented** — Available as the `zavora-xlsx-derive` workspace member.

Proc-macro crate providing `#[derive(ExcelRow)]` for typed row mapping with zavora-xlsx.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
zavora-xlsx = { version = "0.1" }
zavora-xlsx-derive = { version = "0.1" }
```

## Quick Start

```rust
use zavora_xlsx::*;
use zavora_xlsx_derive::ExcelRow;

#[derive(ExcelRow)]
struct Employee {
    #[excel(header = "Employee Name")]
    name: String,
    #[excel(header = "Salary", format = "#,##0.00")]
    salary: f64,
    #[excel(header = "Active")]
    active: bool,
    #[excel(header = "Notes")]
    notes: Option<String>,
    #[excel(skip)]
    internal_id: String,
}
```

## Attributes

### `#[excel(header = "...")]`

Sets a custom column header name. If omitted, the field name is used as the header.

```rust
#[excel(header = "Full Name")]
name: String,  // Header: "Full Name"

department: String,  // Header: "department"
```

### `#[excel(format = "...")]`

Applies an Excel number format to the cell when writing. Uses standard Excel format strings.

```rust
#[excel(header = "Amount", format = "#,##0.00")]
amount: f64,

#[excel(header = "Due Date", format = "yyyy-mm-dd")]
due_date: String,
```

### `#[excel(skip)]`

Excludes the field from Excel I/O. Skipped fields use `Default::default()` when reading.

```rust
#[excel(skip)]
internal_id: String,  // Not written to Excel; reads back as ""
```

**Note:** `skip` cannot be combined with `header` or `format`.

## Supported Field Types

| Rust Type | Excel Cell Type | Read Behavior |
|-----------|----------------|---------------|
| `String` | String | Reads string, number→string, bool→string |
| `f64` | Number | Reads numeric value |
| `f32` | Number | Reads numeric value (cast from f64) |
| `u32` | Number | Reads numeric value (cast from f64) |
| `i32` | Number | Reads numeric value (cast from f64) |
| `u64` | Number | Reads numeric value (cast from f64) |
| `i64` | Number | Reads numeric value (cast from f64) |
| `bool` | Boolean | Reads boolean value |
| `Option<T>` | T or Empty | `None` → empty cell; empty cell → `None` |

## Generated Traits

The derive macro generates implementations for two traits:

### `ExcelRowWriter`

```rust
pub trait ExcelRowWriter {
    fn write_header(&self, ws: &mut Worksheet, row: RowNum) -> Result<()>;
    fn write_row(&self, ws: &mut Worksheet, row: RowNum) -> Result<()>;
}
```

- `write_header` writes column headers to the specified row
- `write_row` writes field values to the specified row

### `ExcelRowReader`

```rust
pub trait ExcelRowReader: Sized {
    fn read_row(ws: &Worksheet, row: RowNum, headers: &[String]) -> Result<Self>;
}
```

- `read_row` reads a row from the worksheet using header names to locate columns
- Missing required columns produce an error
- Missing `Option<T>` columns return `None`

## Full Example

```rust
use zavora_xlsx::*;
use zavora_xlsx_derive::ExcelRow;

#[derive(ExcelRow, Debug, PartialEq)]
struct Employee {
    #[excel(header = "Employee Name")]
    name: String,
    #[excel(header = "Department")]
    department: String,
    #[excel(header = "Salary", format = "#,##0.00")]
    salary: f64,
    #[excel(header = "Years")]
    years: u32,
    #[excel(header = "Active")]
    active: bool,
    #[excel(header = "Notes")]
    notes: Option<String>,
    #[excel(skip)]
    internal_id: String,
}

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let emp = Employee {
        name: "Alice Johnson".into(),
        department: "Engineering".into(),
        salary: 95000.0,
        years: 5,
        active: true,
        notes: Some("Team lead".into()),
        internal_id: "EMP-001".into(),
    };

    // Write header and data
    emp.write_header(ws, 0)?;
    emp.write_row(ws, 1)?;

    // Read back
    let headers = vec![
        "Employee Name".into(), "Department".into(), "Salary".into(),
        "Years".into(), "Active".into(), "Notes".into(),
    ];
    let read_back = Employee::read_row(ws, 1, &headers)?;
    assert_eq!(read_back.name, "Alice Johnson");
    assert_eq!(read_back.internal_id, ""); // skipped field

    wb.save("output.xlsx")?;
    Ok(())
}
```

Run the full demo example:

```bash
cargo run --example derive_macro_demo
```

## Architecture

The crate uses `syn`, `quote`, and `proc-macro2` to parse struct definitions and generate trait implementations at compile time. It is a separate workspace member because proc-macro crates must be compiled as `proc-macro = true` library crates.

## Dependencies

```toml
[dependencies]
syn = { version = "2", features = ["full"] }
quote = "1"
proc-macro2 = "1"
```

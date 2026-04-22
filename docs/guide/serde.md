# Serde Integration

Serialize Rust structs to worksheet rows and deserialize rows back into structs.

## Feature Flag

Enable the `serde-support` feature:

```toml
[dependencies]
zavora-xlsx = { version = "0.1", features = ["serde-support"] }
serde = { version = "1", features = ["derive"] }
```

## Writing Rows (`write_rows`)

Serialize a `Vec<T: Serialize>` to a worksheet. Row 0 becomes the header (field names), and each struct becomes a data row.

```rust
use zavora_xlsx::*;
use serde::Serialize;

#[derive(Serialize)]
struct Employee {
    name: String,
    department: String,
    salary: f64,
    active: bool,
}

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    let data = vec![
        Employee { name: "Alice".into(), department: "Engineering".into(), salary: 95000.0, active: true },
        Employee { name: "Bob".into(), department: "Marketing".into(), salary: 82000.0, active: true },
        Employee { name: "Carol".into(), department: "Engineering".into(), salary: 105000.0, active: false },
    ];

    // Write to sheet 0
    wb.write_rows(0, &data)?;

    wb.save("employees.xlsx")?;
    Ok(())
}
```

This produces:

| name | department | salary | active |
|------|-----------|--------|--------|
| Alice | Engineering | 95000 | TRUE |
| Bob | Marketing | 82000 | TRUE |
| Carol | Engineering | 105000 | FALSE |

## Reading Rows (`read_rows`)

Deserialize worksheet rows into a `Vec<T: DeserializeOwned>`. Row 0 is treated as the header row, and field names must match struct field names.

```rust
use zavora_xlsx::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Employee {
    name: String,
    department: String,
    salary: f64,
    active: bool,
}

fn main() -> Result<()> {
    let mut wb = Workbook::open("employees.xlsx")?;

    let employees: Vec<Employee> = wb.read_rows(0)?;

    for emp in &employees {
        println!("{}: {} (${:.0}) active={}", emp.name, emp.department, emp.salary, emp.active);
    }

    Ok(())
}
```

## Supported Types

| Rust Type | Excel Cell Type |
|-----------|----------------|
| `String`, `&str` | String |
| `f64`, `f32`, `i8`–`i64`, `u8`–`u64` | Number |
| `bool` | Boolean |
| `Option<T>` | Empty cell for `None`, value for `Some` |

### Option Fields

`Option<T>` fields are supported. `None` values produce empty cells when writing, and empty cells deserialize to `None` when reading.

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Record {
    name: String,
    value: f64,
    notes: Option<String>,  // Empty cells become None
}
```

## Header Row Behavior

- **Writing:** Struct field names become column headers in row 0. The first struct in the slice determines the headers.
- **Reading:** Row 0 is read as headers. Each header is matched to a struct field by name. Unknown headers are ignored.

## Limitations

- Nested structs are not supported as cell values
- Sequences (Vec, arrays) are not supported as cell values
- Maps are not supported as cell values
- Enum variants serialize as their string name (unit variants only for cell values)

## The Derive Macro Alternative

For a zero-dependency alternative to serde, see the `#[derive(ExcelRow)]` proc macro in [derive-macro.md](../ecosystem/derive-macro.md). It provides `ExcelRowWriter` and `ExcelRowReader` traits without requiring serde.

```rust
use zavora_xlsx::*;
use zavora_xlsx_derive::ExcelRow;

#[derive(ExcelRow)]
struct Record {
    name: String,
    value: f64,
    active: bool,
}
```

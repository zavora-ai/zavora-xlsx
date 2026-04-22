# zavora-xlsx-derive

Proc macro providing `#[derive(ExcelRow)]` for [zavora-xlsx](https://github.com/zavora-ai/zavora-xlsx).

Generates `ExcelRowWriter` and `ExcelRowReader` implementations that map struct fields directly to Excel columns — no serde dependency required.

## Install

```toml
[dependencies]
zavora-xlsx = "0.1"
zavora-xlsx-derive = "0.1"
```

## Quick start

```rust
use zavora_xlsx::*;
use zavora_xlsx_derive::ExcelRow;

#[derive(ExcelRow, Debug, PartialEq)]
struct Invoice {
    #[excel(header = "Invoice #")]
    id: u32,
    #[excel(header = "Client")]
    client: String,
    #[excel(header = "Amount", format = "#,##0.00")]
    amount: f64,
    #[excel(header = "Paid")]
    paid: bool,
    #[excel(header = "Notes")]
    notes: Option<String>,
    #[excel(skip)]
    internal_ref: String,
}

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let inv = Invoice {
        id: 1042,
        client: "Acme Corp".into(),
        amount: 15750.00,
        paid: true,
        notes: Some("Net 30".into()),
        internal_ref: "REF-001".into(),
    };

    // Write header row, then data row
    inv.write_header(ws, 0)?;
    inv.write_row(ws, 1)?;

    // Read it back
    let headers = vec![
        "Invoice #".into(), "Client".into(), "Amount".into(),
        "Paid".into(), "Notes".into(),
    ];
    let read_back = Invoice::read_row(ws, 1, &headers)?;

    assert_eq!(read_back.client, "Acme Corp");
    assert_eq!(read_back.amount, 15750.00);
    assert_eq!(read_back.internal_ref, ""); // skipped → Default

    wb.save("invoices.xlsx")?;
    Ok(())
}
```

## Attributes

### `#[excel(header = "...")]`

Custom column header. Defaults to the Rust field name if omitted.

```rust
#[excel(header = "Full Name")]
name: String,       // Column header: "Full Name"

department: String,  // Column header: "department"
```

### `#[excel(format = "...")]`

Apply an Excel number format when writing. Uses standard Excel format strings.

```rust
#[excel(format = "#,##0.00")]
revenue: f64,

#[excel(format = "0.0%")]
growth: f64,

#[excel(header = "Date", format = "yyyy-mm-dd")]
date: String,
```

### `#[excel(skip)]`

Exclude the field from Excel entirely. On read, the field is populated with `Default::default()`. Cannot be combined with `header` or `format`.

```rust
#[excel(skip)]
cache: Vec<u8>,  // Not written, reads back as vec![]
```

### Combining attributes

`header` and `format` can be used together:

```rust
#[excel(header = "Revenue ($)", format = "$#,##0")]
revenue: f64,
```

## Supported types

| Rust type | Write behavior | Read behavior |
|-----------|---------------|---------------|
| `String` | String cell | From String, Number→string, Bool→string |
| `f64`, `f32` | Number cell | From Number |
| `u8`–`u64`, `i8`–`i64` | Number cell | From Number (cast from f64) |
| `bool` | Boolean cell | From Bool |
| `Option<T>` | `None` → skip, `Some(v)` → write v | Empty → `None`, value → `Some` |

## Generated traits

The macro generates implementations for two traits defined in `zavora-xlsx`:

```rust
// Write struct fields as a header row and data rows
pub trait ExcelRowWriter {
    fn write_header(&self, ws: &mut Worksheet, row: u32) -> Result<()>;
    fn write_row(&self, ws: &mut Worksheet, row: u32) -> Result<()>;
}

// Read a worksheet row into a struct
pub trait ExcelRowReader: Sized {
    fn read_row(ws: &Worksheet, row: u32, headers: &[String]) -> Result<Self>;
}
```

The reader uses the `headers` slice to map column names to positions, so column order doesn't matter.

## Error handling

- Missing required column header → `Error::InvalidData("missing column header 'X' for field 'y'")`
- Type mismatch → `Error::InvalidData("field 'y': expected numeric value, got String(...)")`
- Missing `Option<T>` column → field set to `None` (not an error)

## License

Apache-2.0

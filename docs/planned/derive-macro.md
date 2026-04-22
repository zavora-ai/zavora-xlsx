# zavora-xlsx-derive

> **Status: Planned** — This crate will be a separate workspace member.

Proc-macro crate providing `#[derive(ExcelRow)]` for zavora-xlsx.

## Planned Features

- `#[derive(ExcelRow)]` generates `Serialize` and `Deserialize` implementations
- `#[excel(header = "...")]` attribute for custom column names
- `#[excel(format = "...")]` attribute for number formats
- `Option<T>` fields map to empty cells when `None`

## Example (planned)

```rust
use zavora_xlsx_derive::ExcelRow;

#[derive(ExcelRow)]
struct Invoice {
    #[excel(header = "Invoice #")]
    id: u32,
    #[excel(header = "Amount", format = "#,##0.00")]
    amount: f64,
    #[excel(header = "Due Date", format = "yyyy-mm-dd")]
    due_date: Option<String>,
}
```

## Architecture

This crate will use `syn`, `quote`, and `proc-macro2` to parse struct definitions
and generate trait implementations. It will be a separate workspace member because
proc-macro crates must be compiled as `proc-macro = true` library crates.

## Dependencies

```toml
[dependencies]
syn = { version = "2", features = ["full"] }
quote = "1"
proc-macro2 = "1"
```

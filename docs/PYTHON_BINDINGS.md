# zavora-xlsx-python

> **Status: 📋 Planned** — Requirements spec complete, implementation pending.

Python bindings for zavora-xlsx using PyO3 and maturin.

## Planned Features

- `Workbook`, `Worksheet`, `Format`, `Chart`, `Table` Python classes
- Cell writing methods: `write_string`, `write_number`, `write_formula`
- Format application methods
- Save to file and save to bytes
- `pip install` packaging via maturin

## Planned API

```python
from zavora_xlsx import Workbook, Format

wb = Workbook()
ws = wb.worksheet(0)

bold = Format().bold()
ws.write_string(0, 0, "Name", bold)
ws.write_string(0, 1, "Score", bold)
ws.write_string(1, 0, "Alice")
ws.write_number(1, 1, 95.5)

wb.save("output.xlsx")

# Or save to bytes
data = wb.save_to_buffer()
```

## Planned Classes

### Workbook

```python
wb = Workbook()                          # Create new
wb = Workbook.open("report.xlsx")        # Open existing
wb = Workbook.open_from_buffer(data)     # Open from bytes

ws = wb.worksheet(0)                     # Access by index
ws = wb.add_worksheet()                  # Add new sheet
ws = wb.add_worksheet("Sales")           # Add named sheet

wb.sheet_names()                         # List sheet names
wb.sheet_count()                         # Number of sheets

wb.set_properties(title="Report", author="Python Demo")
wb.save("output.xlsx")
data = wb.save_to_buffer()
```

### Worksheet

```python
ws.write_string(row, col, "text")
ws.write_string(row, col, "text", format)
ws.write_number(row, col, 42.5)
ws.write_number(row, col, 42.5, format)
ws.write_boolean(row, col, True)
ws.write_formula(row, col, "SUM(A1:A10)")

value = ws.read_cell(row, col)
used = ws.used_range()

ws.set_column_width(col, 20.0)
ws.set_row_height(row, 30.0)
ws.set_freeze_panes(1, 0)
ws.merge_range(r1, c1, r2, c2, "Title", format)
ws.autofit()

csv_str = ws.to_csv_string()
```

### Format

```python
fmt = (Format()
    .bold()
    .italic()
    .font_size(14)
    .font_name("Arial")
    .font_color("#FF0000")
    .background_color("#FFFF00")
    .num_format("#,##0.00")
    .border("thin")
    .align("center"))
```

## Architecture

This crate will wrap the core `zavora-xlsx` API using PyO3's `#[pyclass]` and `#[pymethods]` attributes. It will be built with maturin for distribution as a Python wheel.

## Dependencies

```toml
[dependencies]
pyo3 = { version = "0.22", features = ["extension-module"] }
zavora-xlsx = { path = ".." }

[build-system]
requires = ["maturin>=1.0"]
build-backend = "maturin"
```

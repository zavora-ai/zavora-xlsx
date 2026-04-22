# zavora-xlsx-python

> **Status: Planned** — This crate will be a separate workspace member.

Python bindings for zavora-xlsx using PyO3 and maturin.

## Planned Features

- `Workbook`, `Worksheet`, `Format`, `Chart`, `Table` Python classes
- Cell writing methods: `write_string`, `write_number`, `write_formula`
- Format application methods
- Save to file and save to bytes
- `pip install` packaging via maturin

## Example (planned)

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
```

## Architecture

This crate wraps the core `zavora-xlsx` API using PyO3's `#[pyclass]` and
`#[pymethods]` attributes. It will be built with maturin for distribution
as a Python wheel.

## Dependencies

```toml
[dependencies]
pyo3 = { version = "0.22", features = ["extension-module"] }
zavora-xlsx = { path = ".." }

[build-system]
requires = ["maturin>=1.0"]
build-backend = "maturin"
```

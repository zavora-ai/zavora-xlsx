# zavora-xlsx-python

Python bindings for [zavora-xlsx](https://github.com/zavora-ai/zavora-xlsx), built with [PyO3](https://pyo3.rs) and [maturin](https://www.maturin.rs).

Read, write, and edit Excel `.xlsx` files from Python with native Rust performance.

## Build

```bash
cd zavora-xlsx-python
pip install maturin
maturin develop
```

## API overview

### Workbook

```python
from zavora_xlsx import Workbook

# Create
wb = Workbook()

# Open from file or bytes
wb = Workbook.open("report.xlsx")
wb = Workbook.open_from_bytes(open("report.xlsx", "rb").read())

# Sheet management
ws = wb.worksheet(0)
ws = wb.add_worksheet()
ws = wb.add_worksheet_with_name("Sales")
wb.sheet_names()   # ['Sheet1', 'Sales']
wb.sheet_count()   # 2

# Document properties (dict-based)
wb.set_properties({"title": "Report", "author": "Team", "company": "Acme"})
props = wb.properties()  # {"title": "Report", ...}

# Save
wb.save("output.xlsx")
data = wb.save_to_bytes()  # returns bytes
```

### Worksheet — writing cells

```python
from zavora_xlsx import Format

bold = Format().bold().font_size(12).font_color("#FFFFFF").background_color("#2B579A")
currency = Format().num_format("$#,##0.00")

ws.write_string(0, 0, "Product", bold)
ws.write_number(1, 0, 1234.56, currency)
ws.write_boolean(2, 0, True)
ws.write_formula(3, 0, "SUM(A2:A3)")
ws.write_blank(4, 0, bold)
```

The `format` parameter is always optional (keyword argument):

```python
ws.write_string(0, 0, "plain")
ws.write_string(0, 0, "styled", format=bold)
```

### Worksheet — reading cells

```python
val = ws.read_cell(0, 0)
# Returns: str | float | bool | None | dict

# Formula cells return a dict:
# {"formula": "SUM(A1:A10)", "cached_value": 55.0}

used = ws.used_range()
# Returns: {"first_row": 0, "first_col": 0, "last_row": 99, "last_col": 5} or None
```

### Worksheet — layout

```python
ws.set_column_width(0, 20.0)
ws.set_row_height(0, 30.0)
ws.set_freeze_panes(1, 0)
ws.merge_range(0, 0, 0, 3, "Title")
ws.autofit()
ws.set_zoom(125)
ws.name()
ws.set_name("Data")
```

### Format

All methods are chainable:

```python
fmt = (Format()
    .bold()
    .italic()
    .strikethrough()
    .font_size(14)
    .font_name("Arial")
    .font_color("#FF0000")
    .background_color("#FFFF00")
    .num_format("#,##0.00")
    .underline("single")       # "single" | "double"
    .border("thin")            # "none" | "thin" | "medium" | "thick" | "double" | "dashed" | "dotted"
    .align("center")           # "left" | "center" | "right" | "fill" | "justify" | "top" | "middle" | "bottom"
    .text_wrap()
    .shrink_to_fit()
    .indent(2)
    .rotation(45))
```

### Chart

```python
from zavora_xlsx import Chart

# Types: "bar", "column", "line", "pie", "scatter", "area", "doughnut", "radar"
chart = Chart("column")
chart.add_series("Sheet1!$B$2:$B$5", categories="Sheet1!$A$2:$A$5", name="Q1 Sales")
chart.set_title("Quarterly Sales")
chart.set_x_axis_name("Product")
chart.set_y_axis_name("Revenue")
chart.set_size(800, 400)
chart.set_legend_position("bottom")  # "top" | "bottom" | "left" | "right" | "none"

ws.insert_chart(6, 0, chart)
```

### Table

```python
from zavora_xlsx import Table

table = Table()
table.set_name("SalesData")
table.set_columns([
    {"name": "Product"},
    {"name": "Revenue", "total_function": "sum"},
    {"name": "Count", "total_label": "Total"},
])
table.set_style("TableStyleMedium9")
table.set_total_row(True)
table.set_autofilter(True)

ws.add_table(0, 0, 10, 2, table)
```

### CSV export

```python
csv = ws.to_csv_string()
tsv = ws.to_csv_string({"delimiter": "\t"})
custom = ws.to_csv_string({
    "delimiter": ";",
    "quote": '"',
    "line_ending": "\n",
    "date_format": "dd/mm/yyyy",
})
```

## Error handling

Rust errors map to idiomatic Python exceptions:

| Rust error | Python exception |
|------------|-----------------|
| `Error::Io` | `OSError` |
| `Error::SheetNotFound` | `IndexError` |
| `Error::InvalidData` | `ValueError` |
| Other | `RuntimeError` |

```python
try:
    wb = Workbook.open("missing.xlsx")
except OSError as e:
    print(f"File error: {e}")

try:
    ws = wb.worksheet(99)
except IndexError as e:
    print(f"No such sheet: {e}")
```

## Cell value types

| Excel cell | Python return type |
|------------|-------------------|
| Empty | `None` |
| String | `str` |
| Number | `float` |
| Boolean | `bool` |
| DateTime | `float` (Excel serial) |
| Error | `str` |
| Rich text | `str` (plain text) |
| Formula | `dict` with `formula` and `cached_value` keys |

## License

Apache-2.0

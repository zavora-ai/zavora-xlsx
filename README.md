# zavora-xlsx

High-performance Rust crate for reading, writing, and editing Excel `.xlsx` files.

- **Unified**: Read, write, and edit in one crate — no separate reader/writer dependencies
- **Fast**: 4 dependencies, ~6.6K lines of Rust, constant-memory streaming mode for large files
- **Excel-compatible**: Validated against Microsoft Excel with zero repair errors
- **Ergonomic**: Builder patterns, `impl IntoExcelData`, chainable methods

## Quick Start

```rust
use zavora_xlsx::{Workbook, Format};

let mut wb = Workbook::new();
let ws = wb.worksheet(0)?;

// Write data
ws.write(0, 0, "Hello")?;
ws.write(0, 1, 42.5)?;
ws.write(0, 2, true)?;

// With formatting
let bold = Format::new().bold().font_size(14.0);
ws.write_with_format(1, 0, "Formatted", &bold)?;

wb.save("output.xlsx")?;
```

## Installation

```toml
[dependencies]
zavora-xlsx = "0.1"
```

### Optional features

```toml
[dependencies]
zavora-xlsx = { version = "0.1", features = ["serde-support", "async-tokio"] }
```

| Feature | Description |
|---------|-------------|
| `mmap` | Memory-mapped file I/O via `memmap2` |
| `serde-support` | Serialize/deserialize rows via `serde` |
| `async-tokio` | Async file I/O via `tokio` |
| `wasm` | WASM target support |
| `cffi` | C FFI bindings |

Dependencies: `quick-xml`, `zip`, `atoi_simd`, `fast-float2` — no heavy frameworks.

---

## Feature Matrix

| Category | Features |
|----------|----------|
| **Core** | Read, write, edit xlsx; multi-sheet; formulas (regular, array, dynamic); defined names; doc properties; open from file or buffer |
| **Formatting** | Bold, italic, underline, strikethrough, font size/name/color, background color, foreground color, number formats, borders (all sides + diagonal), alignment (9 types), text wrap, shrink to fit, indent, rotation, cell lock/unlock, quote prefix, 18 pattern fills |
| **Charts** | 9 types (Column, Bar, Line, Pie, Scatter, Area, Doughnut, Radar, Stock), combo charts, secondary axis, data labels, trendlines (6 types), data table, titles, legends, axis names, pixel offset positioning |
| **Tables** | Headers, 20+ styles, autofilter with column criteria, total row |
| **Conditional Formatting** | Cell value, 2/3-color scales, data bars, icon sets, formula, top/bottom N, text contains/begins/ends, duplicates/uniques, above/below average, date occurring — all with DXF support |
| **Data Validation** | Dropdown list, whole number, decimal, date, time, text length, custom formula — with input/error messages |
| **Images** | PNG and JPEG with auto dimension detection, scale width/height |
| **Rich Text** | Multiple fonts/colors/styles in one cell, superscript/subscript |
| **Print** | Page setup, margins, headers/footers, page breaks, repeat rows/columns, print area, scale/fit-to-page, print gridlines/headings, centering, black & white, first page number |
| **View** | Freeze panes, zoom, hide gridlines/headings, right-to-left, tab colors, active sheet, selection, top-left cell, default row height |
| **Structure** | Insert/remove rows & columns (with formula shift), hidden rows/columns, row/column grouping, autofit column widths, merge cells, column/row format, write blank, clear cell, ignore error indicators |
| **Protection** | Sheet protection, workbook protection, password hashing, cell lock/unlock, unprotect ranges |
| **Read** | Cell values, formulas, sheet visibility, merge ranges, column widths, row heights, freeze panes, extract images |
| **Edit Mode** | Open → modify → save with VBA/macro passthrough; preserves drawings, charts, comments, merges, widths, heights on dirty sheets |

---

## API Reference

### Workbook

```rust
use zavora_xlsx::{Workbook, CalcMode, DocProperties};

let mut wb = Workbook::new();                          // New workbook
let mut wb = Workbook::open("file.xlsx")?;             // Edit existing
let mut wb = Workbook::open_readonly("file.xlsx")?;    // Read-only
let mut wb = Workbook::open_from_buffer(&bytes)?;      // Edit from memory
let mut wb = Workbook::open_readonly_from_buffer(&bytes)?; // Read from memory

// Sheets
let ws = wb.add_worksheet();                           // Add sheet
let ws = wb.add_worksheet_with_name("Sales")?;         // Named sheet
let ws = wb.worksheet(0)?;                             // By index
let ws = wb.worksheet_by_name("Sales")?;               // By name
wb.remove_worksheet(1)?;                               // Remove
wb.rename_worksheet(0, "NewName")?;                    // Rename
wb.move_worksheet(0, 2)?;                              // Reorder
let names = wb.sheet_names();                          // List names
let n = wb.sheet_count();                              // Count

// Settings
wb.define_name("TaxRate", "'Config'!$A$1");
wb.set_properties(DocProperties::new().title("Report").author("Team"));
wb.set_calc_mode(CalcMode::Manual);
wb.set_active_sheet(0);                                // Which sheet opens first
wb.protect();                                          // Structure protection
wb.protect_with_password("secret");

// Save
wb.save("output.xlsx")?;
let bytes = wb.save_to_buffer()?;
```

### Worksheet — Writing

All coordinates are 0-based: `(row: u32, col: u16)`.

```rust
let ws = wb.worksheet(0)?;

// Write cells
ws.write(0, 0, "text")?;                              // String
ws.write(0, 1, 42.5)?;                                // Number
ws.write(0, 2, true)?;                                // Boolean
ws.write_formula(0, 3, "SUM(A1:C1)")?;                // Formula
ws.write_formula_with_result(0, 4, "A1*2", 42.0)?;    // With cached result
ws.write_array_formula(0, 5, 1, 6, "MMULT(A1:B2,C1:D2)")?; // CSE array
ws.write_dynamic_formula(0, 7, "_xlfn.UNIQUE(A:A)")?; // Excel 365 spill
ws.write_blank(0, 8, &fmt)?;                          // Formatted empty cell
ws.clear_cell(0, 9);                                  // Remove cell value
ws.write_with_format(0, 0, "bold", &fmt)?;             // With format

// Batch write
ws.write_row(0, 0, ["Q1", "Q2", "Q3", "Q4"])?;       // Row
ws.write_column(0, 0, [100, 200, 300])?;               // Column

// Rich text
use zavora_xlsx::{RichText, RichTextRun};
let rt = RichText::new()
    .add_run(RichTextRun::new("Bold ").bold())
    .add_run(RichTextRun::new("Red").color("#FF0000"));
ws.write_rich_text(0, 0, &rt)?;

// Merge
ws.merge_range(0, 0, 0, 3, "Header", &fmt)?;
```

### Worksheet — Reading

```rust
use zavora_xlsx::CellValue;

let val = ws.read_cell(0, 0);                          // CellValue enum
match val {
    CellValue::String(s) => println!("{s}"),
    CellValue::Number(n) => println!("{n}"),
    CellValue::Bool(b) => println!("{b}"),
    CellValue::Empty => println!("empty"),
    _ => {}
}

let range = ws.used_range();                           // Option<(r1,c1,r2,c2)>
```

### Worksheet — Layout & View

```rust
ws.set_column_width(0, 20.0)?;                        // Column A = 20 chars
ws.set_row_height(0, 30.0)?;                          // Row 1 = 30pt
ws.set_freeze_panes(1, 0)?;                           // Freeze top row
ws.autofit()?;                                         // Auto-size columns

ws.set_zoom(85);                                       // 10-400%
ws.hide_gridlines();
ws.hide_headings();
ws.set_right_to_left();
ws.set_tab_color("#4472C4");
ws.set_selection(4, 1);                                // Cursor position
ws.set_top_left_cell(0, 0);                            // Scroll position
ws.set_default_row_height(20.0);                       // All rows 20pt
ws.set_column_format(1, &currency_fmt);                // Format entire column
ws.set_row_format(0, &header_fmt);                     // Format entire row
ws.set_hidden();                                       // Hide sheet
ws.set_very_hidden();                                  // VBA-only access
ws.ignore_error("numberStoredAsText", "A1:A100");      // No green triangles

ws.set_row_hidden(5, true);
ws.set_column_hidden(3, true);
ws.group_rows(2, 5, 1);                               // Outline level 1
ws.group_columns(1, 3, 1);
```

### Worksheet — Insert/Remove Rows & Columns

```rust
ws.insert_rows(5, 3)?;                                // Insert 3 rows at row 5
ws.remove_rows(5, 2)?;                                // Remove 2 rows at row 5
ws.insert_columns(2, 1)?;                             // Insert 1 column at C
ws.remove_columns(2, 1)?;                             // Remove column C
// Formulas automatically shift references
```

### Format

Builder pattern — all methods consume and return `Self`:

```rust
use zavora_xlsx::{Format, Align, BorderStyle, Underline};

let fmt = Format::new()
    .bold()
    .italic()
    .underline(Underline::Single)
    .strikethrough()
    .font_size(12.0)
    .font_name("Calibri")
    .font_color("#FFFFFF")
    .background_color("#4472C4")
    .num_format("#,##0.00")
    .align(Align::Center)
    .align(Align::VerticalCenter)                      // Chain both
    .border(BorderStyle::Thin)
    .border_color("#000000")
    .text_wrap()
    .shrink_to_fit()
    .indent(2)
    .rotation(45)
    .unlocked()                                        // For protected sheets
    .diagonal_border(BorderStyle::Thin, DiagonalType::Up) // Diagonal border
    .pattern_fill(Pattern::DarkDown)                   // Pattern fill
    .foreground_color("#FF0000")                       // Pattern foreground
    .quote_prefix();                                   // Force text display

// Apply to cell or range
ws.set_cell_format(0, 0, &fmt)?;
ws.set_range_format(0, 0, 10, 5, &fmt)?;
```

### Charts

```rust
use zavora_xlsx::{Chart, ChartType, TrendlineType, LegendPosition};

let mut chart = Chart::new(ChartType::Column);
chart.set_title("Revenue by Quarter");
chart.set_x_axis_name("Quarter");
chart.set_y_axis_name("Revenue ($)");
chart.set_legend_position(LegendPosition::Bottom);
chart.set_width(800);
chart.set_height(500);

// Add series
chart.add_series()
    .set_values("Sheet1!$B$2:$E$2")
    .set_categories("Sheet1!$B$1:$E$1")
    .set_name("North");

// Data labels
chart.add_series()
    .set_values("Sheet1!$B$3:$E$3")
    .set_name("South")
    .set_data_labels(true);

// Trendline
chart.add_series()
    .set_values("Sheet1!$B$4:$E$4")
    .set_name("East")
    .set_trendline(TrendlineType::Linear);

ws.insert_chart(7, 0, &chart)?;
```

**Combo charts** — override chart type per series:

```rust
let mut chart = Chart::new(ChartType::Column);
chart.add_series()
    .set_values("Sheet1!$B$2:$E$2")
    .set_name("Revenue");                              // Column (default)

chart.add_series()
    .set_values("Sheet1!$B$3:$E$3")
    .set_name("Margin %")
    .set_chart_type(ChartType::Line)                   // Override to line
    .set_secondary_axis(true);                         // Right Y-axis

chart.set_y2_axis_name("Margin %");
ws.insert_chart(7, 0, &chart)?;
```

**Trendline types**: `Linear`, `Exponential`, `Polynomial`, `Power`, `Logarithmic`, `MovingAverage`

**Chart types**: `Column`, `Bar`, `Line`, `Pie`, `Scatter`, `Area`, `Doughnut`, `Radar`

### Tables

```rust
use zavora_xlsx::{Table, TableColumn, TableStyle};

let table = Table::new()
    .set_columns(&[
        TableColumn::new("Name"),
        TableColumn::new("Revenue"),
        TableColumn::new("Growth"),
    ])
    .set_style(TableStyle::Medium2)
    .set_total_row(true);

ws.add_table(0, 0, 10, 2, &table)?;
```

### Conditional Formatting

```rust
use zavora_xlsx::*;

// Cell value rule
ws.add_conditional_format(1, 1, 10, 1,
    ConditionalFormatCell::new(CfOperator::GreaterThan, 100.0))?;

// Color scales
ws.add_conditional_format(1, 2, 10, 2,
    ConditionalFormat2ColorScale::new("#FFFFFF", "#FF0000"))?;

ws.add_conditional_format(1, 3, 10, 3,
    ConditionalFormat3ColorScale::new("#F8696B", "#FFEB84", "#63BE7B"))?;

// Data bars
ws.add_conditional_format(1, 4, 10, 4,
    ConditionalFormatDataBar::new("#4472C4"))?;

// Icon sets
ws.add_conditional_format(1, 5, 10, 5,
    ConditionalFormatIconSet::new(IconSetType::ThreeArrows))?;

// Formula-based
ws.add_conditional_format(1, 0, 10, 5,
    ConditionalFormatFormula::new("MOD(ROW(),2)=0"))?;

// Top/Bottom
ws.add_conditional_format(1, 1, 10, 1,
    ConditionalFormatTopBottom::new(TopBottomType::Top, 5))?;

// Text rules
ws.add_conditional_format(1, 0, 10, 0,
    ConditionalFormatText::new(TextOperator::Contains, "overdue"))?;

// Duplicates / Uniques
ws.add_conditional_format(1, 0, 10, 0, ConditionalFormatDuplicate::new())?;
ws.add_conditional_format(1, 0, 10, 0, ConditionalFormatUnique::new())?;

// Above/Below Average
ws.add_conditional_format(1, 1, 10, 1,
    ConditionalFormatAverage::new(AverageType::Above))?;

// Date Occurring
ws.add_conditional_format(1, 2, 10, 2,
    ConditionalFormatDate::new(DateOccurring::ThisWeek))?;
```

### Data Validation

```rust
use zavora_xlsx::{DataValidation, ValidationRule, ErrorStyle};

// Dropdown list
let dv = DataValidation::new()
    .list(&["Yes", "No", "Maybe"])
    .input_message("Select", "Choose an option")
    .error_message(ErrorStyle::Stop, "Invalid", "Pick from the list");
ws.add_data_validation(1, 0, 10, 0, &dv)?;

// Number range
let dv = DataValidation::new()
    .rule(ValidationRule::WholeNumber { min: 1, max: 100 });
ws.add_data_validation(1, 1, 10, 1, &dv)?;

// Custom formula
let dv = DataValidation::new()
    .custom("AND(A1>0,A1<1000)");
ws.add_data_validation(1, 2, 10, 2, &dv)?;
```

### Images

```rust
use zavora_xlsx::Image;

let img = Image::from_path("logo.png")?;               // Auto-detects dimensions
ws.insert_image(0, 0, &img)?;
```

### Sparklines

```rust
use zavora_xlsx::{Sparkline, SparklineType};

let sp = Sparkline::new("Sheet1!A1:F1", SparklineType::Line);
ws.add_sparkline(0, 6, &sp)?;
```

### Comments

```rust
ws.add_comment(0, 0, "Review this value");
ws.add_comment_with_author(1, 0, "Looks correct", "James");
```

### Hyperlinks

```rust
ws.write_url(0, 0, "https://example.com", "Click here")?;
ws.write_internal_link(1, 0, "'Sheet2'!A1", "Go to Sheet2")?;
```

### Print Settings

```rust
use zavora_xlsx::{PrintSettings, Orientation};

let ps = PrintSettings::new()
    .orientation(Orientation::Landscape)
    .paper_size(1)                                     // Letter
    .margins(0.75, 0.75, 0.7, 0.7)
    .header("&C&\"Arial,Bold\"Monthly Report")
    .footer("&CPage &P of &N");

ws.set_print_settings(&ps);
ws.set_page_breaks(&[20, 40], &[]);                   // Row breaks
ws.set_print_area(0, 0, 50, 10);
ws.set_repeat_rows(0, 0);                             // Repeat header row
ws.set_repeat_columns(0, 1);                          // Repeat columns A:B
ws.set_print_scale(85);                               // 85%
```

### Protection

```rust
// Sheet protection
ws.protect();                                          // Default (no password)
ws.protect_with_password("secret");                    // With password

// Workbook structure protection
wb.protect();
wb.protect_with_password("secret");
```

### Autofilter

```rust
ws.set_autofilter(0, 0, 100, 5);                      // Header row through data
```

### Streaming Mode

For large files (100K+ rows) with constant memory usage:

```rust
use zavora_xlsx::{StreamingWorkbook, Format, DocProperties};

let mut sw = StreamingWorkbook::new();
sw.add_worksheet("Data");
sw.set_current_sheet(0);
sw.set_column_width(0, 20.0);
sw.merge_range(0, 0, 0, 3);

for row in 0..100_000 {
    sw.write_number(row, 0, row as f64)?;
    sw.write_string(row, 1, "data")?;
}

sw.save("large_file.xlsx")?;
```

### Edit Mode

Open existing files, modify, and save — preserving VBA macros and unknown parts:

```rust
let mut wb = Workbook::open("existing.xlsx")?;
let ws = wb.worksheet(0)?;
ws.write(0, 0, "Updated")?;
wb.save("modified.xlsx")?;
```

### Formula Recalculation

Evaluate all formula cells programmatically:

```rust
let mut wb = Workbook::new();
let ws = wb.worksheet(0)?;
ws.write(0, 0, 10.0)?;
ws.write(0, 1, 20.0)?;
ws.write_formula(0, 2, "A1+B1")?;
ws.write_formula(0, 3, "SUM(A1:C1)")?;

let count = wb.recalculate()?;                         // Evaluates in dependency order
println!("Evaluated {} cells", count);                 // 2
```

Handles dependency graphs, circular reference detection, and volatile functions (`RAND`, `NOW`, `TODAY`).

---

## Colors

```rust
use zavora_xlsx::Color;

// Named colors
Format::new().font_color(Color::Red);

// Hex string
Format::new().font_color("#4472C4");

// RGB array
Format::new().font_color([0x44, 0x72, 0xC4]);
```

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `quick-xml` | XML parsing and writing |
| `zip` | ZIP archive read/write (deflate) |
| `atoi_simd` | SIMD-accelerated integer parsing |
| `fast-float2` | Fast float parsing |

Dev dependencies: `calamine` (read verification), `tempfile` (test isolation).

---

## Tests

- **61 library tests** covering all features
- **38 integration tests** (demo suite) validating end-to-end file generation
- All output files validated in Microsoft Excel with zero repair errors

```bash
cargo test                    # Run all tests
cargo test --lib              # Library tests only
```

## License

Apache-2.0

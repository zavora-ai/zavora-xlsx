# Reading Excel Files

Complete guide to reading existing `.xlsx` files with zavora-xlsx.

## Opening Files

```rust
use zavora_xlsx::*;

// Read-only mode (fastest, no edit capability)
let mut wb = Workbook::open_readonly("data.xlsx")?;

// Edit mode (open → modify → save)
let mut wb = Workbook::open("data.xlsx")?;

// From in-memory buffer (read-only)
let bytes = std::fs::read("data.xlsx")?;
let mut wb = Workbook::open_readonly_from_buffer(&bytes)?;

// From in-memory buffer (edit mode)
let mut wb = Workbook::open_from_buffer(&bytes)?;
```

## Reading Cells

### CellValue Enum

Every cell read returns a `CellValue`:

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::open_readonly("data.xlsx")?;
    let ws = wb.worksheet(0)?;

    let val = ws.read_cell(0, 0);  // row 0, col 0

    match val {
        CellValue::Empty => println!("Empty cell"),
        CellValue::String(s) => println!("String: {s}"),
        CellValue::Number(n) => println!("Number: {n}"),
        CellValue::Bool(b) => println!("Boolean: {b}"),
        CellValue::DateTime(dt) => {
            println!("Date: {}", dt.to_iso_string());
            let (y, m, d, h, mi, s) = dt.to_ymd_hms();
            println!("Components: {y}-{m:02}-{d:02} {h:02}:{mi:02}:{s:02}");
        }
        CellValue::Error(e) => println!("Error: {e}"),
        CellValue::Formula { formula, cached_value } => {
            println!("Formula: ={formula}");
            println!("Cached value: {cached_value:?}");
        }
        CellValue::RichText(rt) => {
            println!("Rich text: {}", rt.plain_text());
            for run in &rt.runs {
                println!("  Run: '{}' bold={} italic={}", run.text, run.bold, run.italic);
            }
        }
    }

    // Convenience methods
    if let Some(s) = val.as_str() { println!("{s}"); }
    if let Some(n) = val.as_f64() { println!("{n}"); }
    println!("Is empty: {}", val.is_empty());

    Ok(())
}
```

## Reading Metadata

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::open_readonly("data.xlsx")?;

    // Sheet names and count
    let names = wb.sheet_names();  // Vec<&str>
    let count = wb.sheet_count();
    println!("Sheets: {:?} ({})", names, count);

    // Document properties
    let props = wb.properties();
    println!("Title: {:?}", props.title);
    println!("Author: {:?}", props.author);
    println!("Subject: {:?}", props.subject);
    println!("Description: {:?}", props.description);
    println!("Keywords: {:?}", props.keywords);
    println!("Category: {:?}", props.category);
    println!("Company: {:?}", props.company);

    // Defined names
    for (name, formula) in wb.defined_names() {
        println!("Name: {name} = {formula}");
    }
    for dn in wb.defined_names_with_scope() {
        println!("Name: {} = {} (scope: {:?})", dn.name, dn.formula, dn.scope);
    }

    Ok(())
}
```

## Reading Worksheet Structure

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::open_readonly("data.xlsx")?;
    let ws = wb.worksheet(0)?;

    // Used range
    if let Some((r1, c1, r2, c2)) = ws.used_range() {
        println!("Data from ({r1},{c1}) to ({r2},{c2})");
    }

    // Merge ranges
    for &(r1, c1, r2, c2) in ws.merge_ranges() {
        println!("Merged: ({r1},{c1}):({r2},{c2})");
    }

    // Column widths and row heights
    if let Some(w) = ws.column_width(0) {
        println!("Column A width: {w}");
    }
    if let Some(h) = ws.row_height(0) {
        println!("Row 1 height: {h}");
    }

    // Sheet visibility
    println!("Visibility: {:?}", ws.visibility());
    println!("Hidden: {}", ws.is_hidden());
    println!("Very hidden: {}", ws.is_very_hidden());

    // Outline levels
    for (row, level) in ws.row_outline_levels() {
        println!("Row {row}: outline level {level}");
    }
    for (col, level) in ws.col_outline_levels() {
        println!("Col {col}: outline level {level}");
    }

    Ok(())
}
```

## Reading Cell Formatting

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::open_readonly("data.xlsx")?;
    let ws = wb.worksheet(0)?;

    // Returns Option<Format> — None for default style (xf index 0)
    if let Some(fmt) = ws.cell_format(0, 0) {
        println!("Bold: {}", fmt.is_bold());
        println!("Italic: {}", fmt.is_italic());
        println!("Underline: {:?}", fmt.get_underline());
        println!("Strikethrough: {}", fmt.is_strikethrough());
        println!("Font size: {}", fmt.get_font_size());
        println!("Font name: {}", fmt.get_font_name());
        println!("Num format: {}", fmt.get_num_format());
        println!("Wrap text: {}", fmt.is_wrap_text());
        println!("Shrink: {}", fmt.is_shrink());
        println!("Indent: {}", fmt.get_indent());
        println!("Rotation: {}", fmt.get_rotation());
        println!("H align: {}", fmt.get_h_align());
        println!("V align: {}", fmt.get_v_align());
        println!("Pattern: {:?}", fmt.get_pattern());

        if let Some(c) = fmt.get_font_color() {
            println!("Font color: #{:02X}{:02X}{:02X}", c[0], c[1], c[2]);
        }
        if let Some(c) = fmt.get_bg_color() {
            println!("Background: #{:02X}{:02X}{:02X}", c[0], c[1], c[2]);
        }
        if let Some(c) = fmt.get_fg_color() {
            println!("Foreground: #{:02X}{:02X}{:02X}", c[0], c[1], c[2]);
        }

        println!("Border left: {:?}", fmt.get_border_left());
        println!("Border right: {:?}", fmt.get_border_right());
        println!("Border top: {:?}", fmt.get_border_top());
        println!("Border bottom: {:?}", fmt.get_border_bottom());

        if let Some(tc) = fmt.get_theme_color() {
            println!("Theme: {:?} tint={}", tc.index, tc.tint);
        }
        if let Some(style) = fmt.get_cell_style() {
            println!("Cell style: {style}");
        }
    }

    Ok(())
}
```

## Reading Features

### Charts

```rust
use zavora_xlsx::*;

let ws = wb.worksheet(0)?;
for chart in ws.charts() {
    println!("Type: {:?}", chart.chart_type());
    println!("Title: {:?}", chart.title());
    println!("Style: {:?}", chart.style());
    for s in chart.series() {
        println!("  Values: {}", s.values());
        println!("  Categories: {:?}", s.categories());
        println!("  Name: {:?}", s.name());
    }
}
```

### Tables

```rust
use zavora_xlsx::*;

for table in ws.tables() {
    println!("Name: {:?}", table.table_name());
    println!("Style: {:?}", table.style());
    println!("Total row: {}", table.total_row());
    println!("Autofilter: {}", table.autofilter());
    for col in table.columns() {
        println!("  {}: total_fn={:?}", col.name(), col.total_function());
    }
}
```

### Comments

```rust
use zavora_xlsx::*;

for comment in ws.comments() {
    println!("({},{}) by {}: {}", comment.row, comment.col, comment.author, comment.text);
}

// Or look up a specific cell
if let Some((author, text)) = ws.get_comment(0, 0) {
    println!("Comment by {author}: {text}");
}
```

### Sparklines

```rust
use zavora_xlsx::*;

for sp in ws.sparklines() {
    println!("Range: {}", sp.data_range());
    println!("Type: {:?}", sp.sparkline_type());
    println!("Position: ({}, {})", sp.row(), sp.col());
    if let Some(color) = sp.color() {
        println!("Color: #{:02X}{:02X}{:02X}", color[0], color[1], color[2]);
    }
}
```

### Conditional Formats, Validations, Hyperlinks

```rust
use zavora_xlsx::*;

// Conditional formats
println!("CF rules: {}", ws.conditional_formats().len());

// Validations
for dv in ws.validations() {
    println!("Validation: {:?}", dv.rule());
}

// Hyperlinks
for link in ws.hyperlinks() {
    println!("({},{}) -> {} {:?}", link.row, link.col, link.url, link.location);
}
```

### Slicers and Timelines

```rust
use zavora_xlsx::*;

for slicer in ws.slicers() {
    println!("Slicer: {} (source: {})", slicer.name, slicer.source_name);
}

for timeline in ws.timelines() {
    println!("Timeline: {} (source: {})", timeline.name, timeline.source_name);
}
```

### Print Settings and Protection

```rust
use zavora_xlsx::*;

if let Some(ps) = ws.print_settings() {
    println!("Paper size: {:?}", ps.paper_size);
    println!("Orientation: {:?}", ps.orientation);
    println!("Scale: {:?}", ps.scale);
    println!("Print area: {:?}", ps.print_area);
    println!("Repeat rows: {:?}", ps.repeat_rows);
}

if let Some(prot) = ws.protection() {
    println!("Sheet protected: {}", prot.sheet);
    println!("Sort allowed: {}", !prot.sort);
    println!("Password hash: {:?}", prot.password_hash());
}
```

---

## Streaming Reader

For large files, use `StreamingReader` to iterate rows without loading the entire sheet into memory:

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let reader = StreamingReader::open("large_file.xlsx")?;

    println!("Sheets: {:?}", reader.sheet_names());
    println!("Count: {}", reader.sheet_count());

    // Iterate rows for sheet 0
    let mut rows = reader.sheet_rows(0)?;
    println!("Total rows: {}", rows.row_count());

    while let Some(row) = rows.next_row() {
        print!("Row {}: ", row.row_index);
        for cell in &row.cells {
            print!("[col {}] {:?}  ", cell.col, cell.value);
        }
        println!();
    }

    Ok(())
}
```

`StreamingReader` loads the shared string table and styles upfront (they're small), then parses sheet data row by row.

### StreamingReader from Buffer

```rust
use zavora_xlsx::*;

let data = std::fs::read("large_file.xlsx")?;
let reader = StreamingReader::from_buffer(data)?;
```

### Using as Iterator

`SheetRows` implements `Iterator`:

```rust
use zavora_xlsx::*;

let reader = StreamingReader::open("data.xlsx")?;
let rows = reader.sheet_rows(0)?;

for row in rows {
    println!("Row {}: {} cells", row.row_index, row.cells.len());
}
```

### Consuming All Rows

```rust
use zavora_xlsx::*;

let reader = StreamingReader::open("data.xlsx")?;
let rows = reader.sheet_rows(0)?;
let all_rows: Vec<StreamingRow> = rows.into_rows();
```

---

## Edit Mode

Open an existing file, modify it, and save — preserving VBA macros and unknown parts:

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::open("existing.xlsx")?;

    // Read existing data
    let ws = wb.worksheet(0)?;
    let old_val = ws.read_cell(0, 0);
    println!("Old value: {:?}", old_val);

    // Modify
    ws.write(0, 0, "Updated value")?;

    // Add a new sheet
    let ws2 = wb.add_worksheet_with_name("Summary")?;
    ws2.write(0, 0, "New sheet")?;

    // Save (preserves VBA, drawings, etc.)
    wb.save("modified.xlsx")?;
    Ok(())
}
```

### VBA Passthrough

When opening `.xlsm` files, VBA projects are preserved automatically:

```rust
use zavora_xlsx::*;

let mut wb = Workbook::open("macro_workbook.xlsm")?;
let ws = wb.worksheet(0)?;
ws.write(0, 0, "Updated")?;
wb.save("macro_workbook.xlsm")?;  // VBA macros preserved
```

### Passthrough Entries

Access preserved ZIP entries from the original file:

```rust
use zavora_xlsx::*;

let mut wb = Workbook::open("data.xlsx")?;
let entries = wb.passthrough_entries();
for (name, data) in entries {
    println!("{name}: {} bytes", data.len());
}
```

### Iterating All Cells

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::open_readonly("data.xlsx")?;
    let ws = wb.worksheet(0)?;

    if let Some((r1, c1, r2, c2)) = ws.used_range() {
        for row in r1..=r2 {
            for col in c1..=c2 {
                let val = ws.read_cell(row, col);
                if !val.is_empty() {
                    println!("({row},{col}): {val:?}");
                }
            }
        }
    }

    Ok(())
}
```

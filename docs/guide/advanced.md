# Advanced Features

## Streaming Write Mode

For large files (100K+ rows) with constant memory usage. Rows must be written in ascending order per sheet.

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut sw = StreamingWorkbook::new();

    // Add sheets
    sw.add_worksheet("Data");
    sw.set_current_sheet(0);

    // Column widths and merges
    sw.set_column_width(0, 20.0);
    sw.merge_range(0, 0, 0, 3);

    // Write rows (must be in ascending order)
    for row in 0..100_000u32 {
        sw.write_number(row, 0, row as f64)?;
        sw.write_string(row, 1, "data")?;
    }

    // With formatting
    let bold = Format::new().bold();
    sw.write_string_with_format(0, 0, "Header", &bold)?;
    sw.write_number_with_format(1, 0, 42.0, &Format::new().num_format("#,##0"))?;

    // Booleans and formulas
    sw.write_boolean(1, 2, true)?;
    sw.write_formula(1, 3, "SUM(A1:A100000)")?;

    // Charts and images work too
    let mut chart = Chart::new(ChartType::Line);
    chart.add_series().set_values("Data!$A$1:$A$100");
    sw.insert_chart(0, 5, chart);

    // Document properties
    sw.set_properties(DocProperties::new().title("Large Dataset").author("Team"));

    // Save
    sw.save("large_file.xlsx")?;
    // Or: let bytes = sw.save_to_buffer()?;

    Ok(())
}
```

---

## Pivot Tables

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    // Write source data on Sheet1
    let ws = wb.worksheet(0)?;
    ws.write_row(0, 0, ["Region", "Product", "Date", "Amount"])?;
    ws.write_row(1, 0, ["North", "Widget", "2024-01-15", "500"])?;
    ws.write_row(2, 0, ["South", "Gadget", "2024-02-20", "300"])?;
    // ... more data rows

    // Create pivot table on a new sheet
    let pivot_ws = wb.add_worksheet_with_name("Pivot")?;

    let pivot = PivotTable::new("SalesPivot", "Sheet1!$A$1:$D$100")
        .add_row_field("Region")
        .add_column_field("Product")
        .add_value_field("Amount", PivotAggregation::Sum)
        .add_filter_field("Date");

    pivot_ws.add_pivot_table(0, 0, &pivot)?;
    wb.save("pivot.xlsx")?;
    Ok(())
}
```

### Value Fields with Custom Names and Formats

```rust
use zavora_xlsx::*;

let pivot = PivotTable::new("PT", "Sheet1!$A$1:$D$100")
    .add_row_field("Region")
    .add_value_field("Amount", PivotAggregation::Sum)
    .add_value_field_named("Amount", PivotAggregation::Average, "Avg Amount")
    .set_value_format("Amount", "$#,##0.00");
```

### Aggregation Functions

| Function | Description |
|----------|-------------|
| `PivotAggregation::Sum` | Sum |
| `PivotAggregation::Count` | Count |
| `PivotAggregation::Average` | Average |
| `PivotAggregation::Max` | Maximum |
| `PivotAggregation::Min` | Minimum |
| `PivotAggregation::Product` | Product |
| `PivotAggregation::CountNums` | Count numbers |
| `PivotAggregation::StdDev` | Standard deviation |
| `PivotAggregation::StdDevP` | Population std dev |
| `PivotAggregation::Var` | Variance |
| `PivotAggregation::VarP` | Population variance |

### Calculated Fields

```rust
use zavora_xlsx::*;

let pivot = PivotTable::new("PT", "Sheet1!$A$1:$D$100")
    .add_row_field("Region")
    .add_value_field("Revenue", PivotAggregation::Sum)
    .add_value_field("Cost", PivotAggregation::Sum)
    .add_calculated_field("Profit", "Revenue-Cost");
```

### Calculated Items

```rust
use zavora_xlsx::*;

let pivot = PivotTable::new("PT", "Sheet1!$A$1:$D$100")
    .add_row_field("Region")
    .add_value_field("Amount", PivotAggregation::Sum)
    .add_calculated_item("Combined", "North+South");
```

### Date Grouping

```rust
use zavora_xlsx::*;

let pivot = PivotTable::new("PT", "Sheet1!$A$1:$D$100")
    .add_row_field("Date")
    .add_value_field("Amount", PivotAggregation::Sum)
    .group_by_date("Date", &[DateGroupLevel::Years, DateGroupLevel::Months]);
```

### Numeric Range Grouping

```rust
use zavora_xlsx::*;

let pivot = PivotTable::new("PT", "Sheet1!$A$1:$D$100")
    .add_row_field("Amount")
    .add_value_field("Amount", PivotAggregation::Count)
    .group_by_range("Amount", 0.0, 1000.0, 100.0);
```

### Styles and Layout

```rust
use zavora_xlsx::*;

let pivot = PivotTable::new("PT", "Sheet1!$A$1:$D$100")
    .add_row_field("Region")
    .add_value_field("Amount", PivotAggregation::Sum)
    .set_style_name("PivotStyleMedium9")
    .set_layout(PivotLayout::Tabular)
    .show_row_headers(true)
    .show_column_headers(true)
    .show_row_stripes(true)
    .show_grand_totals(true, true)
    .show_subtotals("Region", false);
```

| Layout | Description |
|--------|-------------|
| `PivotLayout::Compact` | Compact form (default) |
| `PivotLayout::Outline` | Outline form |
| `PivotLayout::Tabular` | Tabular form |

---

## Images

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // From file (auto-detects PNG/JPEG dimensions)
    let img = Image::from_path("logo.png")?;
    ws.insert_image(0, 0, &img)?;

    // From buffer
    let data = std::fs::read("photo.jpeg")?;
    let img = Image::from_buffer(&data)?;
    ws.insert_image(5, 0, &img)?;

    // Resize
    let mut img = Image::from_path("logo.png")?;
    img.set_width(200);   // Set width in pixels (scales proportionally)
    img.set_height(100);  // Set height in pixels

    // Or scale
    img.set_scale_width(0.5);   // 50% width
    img.set_scale_height(0.5);  // 50% height

    // Alt text (accessibility)
    img.set_alt_text("Company Logo", "The Acme Corp logo in blue and white");

    ws.insert_image(10, 0, &img)?;
    wb.save("images.xlsx")?;
    Ok(())
}
```

Supported formats: PNG and JPEG only.

---

## Sparklines

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Write data in row 0, cols 0–5
    ws.write_row(0, 0, [10, 20, 15, 30, 25, 35])?;

    // Line sparkline in cell G1
    let sp = Sparkline::new("Sheet1!A1:F1", SparklineType::Line);
    ws.add_sparkline(0, 6, &sp)?;

    // Column sparkline with color
    let mut sp = Sparkline::new("Sheet1!A1:F1", SparklineType::Column);
    sp.set_color("#4472C4");
    ws.add_sparkline(1, 6, &sp)?;

    // Win/Loss sparkline
    let sp = Sparkline::new("Sheet1!A1:F1", SparklineType::WinLoss);
    ws.add_sparkline(2, 6, &sp)?;

    wb.save("sparklines.xlsx")?;
    Ok(())
}
```

| Type | Description |
|------|-------------|
| `SparklineType::Line` | Line sparkline |
| `SparklineType::Column` | Column sparkline |
| `SparklineType::WinLoss` | Win/loss sparkline |

---

## Shapes

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let shape = Shape::new(ShapeType::Rectangle, 200, 100)
        .text("Hello")
        .fill_color([0x44, 0x72, 0xC4])
        .outline_color([0x00, 0x00, 0x00])
        .outline_width(1.5)
        .font_size(14.0)
        .bold();

    ws.add_shape(0, 0, &shape)?;
    wb.save("shapes.xlsx")?;
    Ok(())
}
```

### Shape Types

| Type | Description |
|------|-------------|
| `ShapeType::Rectangle` | Rectangle |
| `ShapeType::RoundedRectangle` | Rounded rectangle |
| `ShapeType::Ellipse` | Ellipse/circle |
| `ShapeType::Triangle` | Triangle |
| `ShapeType::Diamond` | Diamond |
| `ShapeType::Arrow` | Right arrow |
| `ShapeType::Callout` | Callout box |
| `ShapeType::TextBox` | Text box |

---

## Slicers

Visual filter controls for tables and pivot tables:

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Table slicer
    let slicer = Slicer::new("Region Filter", "Region")
        .set_caption("Select Region")
        .set_width(200)
        .set_height(300)
        .set_style("SlicerStyleLight1");
    ws.add_slicer(0, 5, &slicer)?;

    // Pivot table slicer
    let pivot_slicer = Slicer::new("Category Filter", "Category")
        .link_to_pivot_cache("PivotCache1");
    ws.add_slicer(0, 8, &pivot_slicer)?;

    wb.save("slicers.xlsx")?;
    Ok(())
}
```

---

## Timelines

Date-based filter controls for pivot tables:

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let timeline = Timeline::new("Date Filter", "OrderDate")
        .set_caption("Filter by Date")
        .set_level(TimelineLevel::Months);

    ws.add_timeline(0, 5, &timeline)?;
    wb.save("timelines.xlsx")?;
    Ok(())
}
```

| Level | Description |
|-------|-------------|
| `TimelineLevel::Years` | Year granularity |
| `TimelineLevel::Quarters` | Quarter granularity |
| `TimelineLevel::Months` | Month granularity |
| `TimelineLevel::Days` | Day granularity |

---

## Comments

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Simple comment (default author: "Author")
    ws.add_comment(0, 0, "Review this value");

    // Comment with author
    ws.add_comment_with_author(1, 0, "Looks correct", "James");

    // Threaded comment (modern Excel comments)
    let mut tc = ThreadedComment::new("Alice", "What about this number?")
        .timestamp("2024-06-15T10:30:00.000");
    tc.add_reply("Bob", "It's the Q2 total");
    tc.add_reply_with_timestamp("Alice", "Thanks!", "2024-06-15T11:00:00.000");
    ws.add_threaded_comment(2, 0, tc);

    wb.save("comments.xlsx")?;
    Ok(())
}
```

---

## Hyperlinks

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // External URL
    ws.write_url(0, 0, "https://example.com", "Click here")?;

    // Internal link to another sheet
    ws.write_internal_link(1, 0, "'Sheet2'!A1", "Go to Sheet2")?;

    wb.save("links.xlsx")?;
    Ok(())
}
```

---

## Form Controls

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Checkbox
    ws.add_form_control(0, 0, FormControl::checkbox("Enable feature"));
    ws.add_form_control(1, 0, FormControl::checkbox_with_link("Active", "$C$1"));

    // Dropdown
    ws.add_form_control(2, 0, FormControl::dropdown(
        vec!["Option A".into(), "Option B".into(), "Option C".into()]
    ));

    // Button
    ws.add_form_control(3, 0, FormControl::button("Click Me"));

    // Spinner
    ws.add_form_control(4, 0, FormControl::spinner(0, 100, 50));
    ws.add_form_control(5, 0, FormControl::spinner_with_link(1, 10, 5, "$C$5"));

    wb.save("controls.xlsx")?;
    Ok(())
}
```

---

## Insert/Remove Rows and Columns

Formulas automatically shift references:

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    ws.write(0, 0, "A")?;
    ws.write_formula(5, 0, "SUM(A1:A5)")?;

    // Insert 3 rows at row 2 (shifts everything below)
    ws.insert_rows(2, 3)?;

    // Remove 2 rows starting at row 5
    ws.remove_rows(5, 2)?;

    // Insert 1 column at column C (index 2)
    ws.insert_columns(2, 1)?;

    // Remove column C
    ws.remove_columns(2, 1)?;

    wb.save("ops.xlsx")?;
    Ok(())
}
```

---

## Row/Column Grouping (Outline Levels)

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Group rows 2–5 at outline level 1
    ws.group_rows(2, 5, 1);

    // Nested group: rows 3–4 at level 2
    ws.group_rows(3, 4, 2);

    // Group columns B–D at level 1
    ws.group_columns(1, 3, 1);

    wb.save("grouped.xlsx")?;
    Ok(())
}
```

---

## Print Settings

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Using PrintSettings builder
    let ps = PrintSettings::new()
        .orientation(Orientation::Landscape)
        .paper_size(1)  // 1=Letter, 9=A4
        .margins(0.75, 0.75, 0.7, 0.7)
        .header("&C&\"Arial,Bold\"Monthly Report")
        .footer("&CPage &P of &N")
        .print_gridlines(true)
        .print_headings(false)
        .center_horizontally(true)
        .black_and_white(false)
        .first_page_number(1)
        .fit_to_page(1, 0);  // Fit to 1 page wide, auto height

    ws.set_print_settings(&ps);

    // Or use convenience methods directly
    ws.set_landscape();
    ws.set_paper_size(9);  // A4
    ws.set_margins(1.0, 1.0, 0.75, 0.75);
    ws.set_header_center("Report Title");
    ws.set_footer_center("Page &P");
    ws.set_fit_to_page(1, 1);

    // Page breaks
    ws.set_page_breaks(&[20, 40], &[]);  // Row breaks at rows 20 and 40

    // Print area
    ws.set_print_area(0, 0, 50, 10);

    // Repeat rows/columns on every printed page
    ws.set_repeat_rows(0, 0);      // Repeat row 1
    ws.set_repeat_columns(0, 1);   // Repeat columns A:B

    // Print scale
    ws.set_print_scale(85);  // 85% (range: 10–400)

    wb.save("print.xlsx")?;
    Ok(())
}
```

---

## Protection

### Sheet Protection

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Basic protection (no password)
    ws.protect();

    // With password
    ws.protect_with_password("secret");

    // With custom options
    let mut prot = SheetProtection::default();
    prot.set_sort(false);           // Allow sorting
    prot.set_auto_filter(false);    // Allow autofilter
    prot.set_format_cells(false);   // Allow cell formatting
    prot.set_insert_rows(false);    // Allow inserting rows
    ws.protect_with_options(prot);

    // Unprotect specific ranges
    ws.unprotect_range("EditableArea", "A1:B10");
    ws.unprotect_range_with_password("SecureEdit", "C1:D10", "edit_pass");

    // Mark cells as unlocked (editable on protected sheets)
    let unlocked = Format::new().unlocked();
    ws.write_with_format(0, 0, "Editable", &unlocked)?;

    wb.save("protected.xlsx")?;
    Ok(())
}
```

### Workbook Protection

```rust
use zavora_xlsx::*;

let mut wb = Workbook::new();
wb.protect();                          // Structure protection
wb.protect_with_password("secret");    // With password
```

---

## Defined Names

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    // Workbook-scoped name
    wb.define_name("TaxRate", "'Config'!$A$1");

    // Sheet-scoped name
    wb.define_name_scoped("LocalName", "Sheet1!$B$1:$B$10", 0);

    // CRUD operations
    wb.add_named_range("Revenue", "Sheet1!$C$1:$C$100", DefinedNameScope::Workbook)?;
    wb.update_named_range("Revenue", "Sheet1!$C$1:$C$200")?;
    wb.remove_named_range("Revenue", &DefinedNameScope::Workbook)?;

    // Read defined names
    for (name, formula) in wb.defined_names() {
        println!("{name} = {formula}");
    }
    for dn in wb.defined_names_with_scope() {
        println!("{} = {} ({:?})", dn.name, dn.formula, dn.scope);
    }

    wb.save("names.xlsx")?;
    Ok(())
}
```

---

## Document Properties

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();

    // Core properties
    wb.set_properties(
        DocProperties::new()
            .title("Annual Report")
            .author("Finance Team")
            .subject("FY2024 Results")
            .description("Comprehensive financial report")
            .keywords("finance, annual, report")
            .category("Reports")
            .company("Acme Corp")
    );

    // Custom properties
    wb.set_custom_property("Department", CustomPropertyValue::Text("Finance".into()));
    wb.set_custom_property("Version", CustomPropertyValue::Number(2.1));
    wb.set_custom_property("Revision", CustomPropertyValue::Integer(5));
    wb.set_custom_property("Approved", CustomPropertyValue::Bool(true));

    // Read custom properties
    for prop in wb.custom_properties() {
        println!("{}: {:?}", prop.name, prop.value);
    }

    wb.save("properties.xlsx")?;
    Ok(())
}
```

---

## CSV Export

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.write_row(0, 0, ["Name", "Value"])?;
    ws.write_row(1, 0, ["Alice", "100"])?;

    // Export to CSV string
    let csv = ws.to_csv_string(&CsvOptions::new());
    println!("{csv}");

    // Export to CSV file
    ws.to_csv_file("output.csv", &CsvOptions::new())?;

    // TSV export
    ws.to_csv_file("output.tsv", &CsvOptions::tsv())?;

    // Custom options
    let opts = CsvOptions::new()
        .delimiter(b';')
        .quote(b'"')
        .line_ending("\n")
        .date_format("dd/mm/yyyy");
    ws.to_csv_file("custom.csv", &opts)?;

    Ok(())
}
```

---

## Formula Engine

### Tokenizer

```rust
use zavora_xlsx::*;

let tokens = tokenize("SUM(A1:B10)*2+IF(C1>0,C1,0)");
for token in &tokens {
    println!("{:?}", token);
}
```

### Formula Evaluation

```rust
use zavora_xlsx::formula_engine::*;

// Parse formula into AST
let ast = parse("1+2*3").unwrap();

// Evaluate with a simple context
let ctx = SimpleContext::new();
let result = evaluate(&ast, &ctx);
println!("{:?}", result);  // Value::Number(7.0)
```

---

## Edit Mode Details

### Save as XLSM (Macro-Enabled)

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.write(0, 0, "Data")?;

    // Provide VBA project binary data
    let vba_data = std::fs::read("vbaProject.bin")?;
    wb.save_as_xlsm("output.xlsm", &vba_data)?;

    Ok(())
}
```

### Save as Template

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.write(0, 0, "Template")?;

    wb.save_as_template("template.xltx")?;

    Ok(())
}
```

### Calc Mode

```rust
use zavora_xlsx::*;

let mut wb = Workbook::new();
wb.set_calc_mode(CalcMode::Auto);       // Automatic recalculation
wb.set_calc_mode(CalcMode::Manual);      // Manual recalculation
wb.set_calc_mode(CalcMode::AutoNoTable); // Auto except data tables
```

### Active Sheet

```rust
use zavora_xlsx::*;

let mut wb = Workbook::new();
wb.add_worksheet_with_name("Summary")?;
wb.set_active_sheet(1);  // Open on "Summary" sheet
```

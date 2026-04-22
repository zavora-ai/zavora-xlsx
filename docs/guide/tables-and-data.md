# Tables, Data Validation, and Conditional Formatting

## Tables

### Basic Table

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Write header and data
    ws.write_row(0, 0, ["Name", "Revenue", "Growth"])?;
    ws.write_row(1, 0, ["Alice", "50000", "0.12"])?;
    ws.write_row(2, 0, ["Bob", "62000", "0.08"])?;
    ws.write_row(3, 0, ["Carol", "45000", "0.15"])?;

    let mut table = Table::new();
    table.set_columns(&[
        TableColumn::new("Name"),
        TableColumn::new("Revenue"),
        TableColumn::new("Growth"),
    ]);
    table.set_style(TableStyle::Medium2);

    ws.add_table(0, 0, 3, 2, &table)?;
    wb.save("table.xlsx")?;
    Ok(())
}
```

### Table Styles

```rust
use zavora_xlsx::*;

// Light styles (1–21)
table.set_style(TableStyle::Light(1));

// Medium styles (1–28)
table.set_style(TableStyle::Medium(9));

// Dark styles (1–11)
table.set_style(TableStyle::Dark(1));
```

### Total Row

```rust
use zavora_xlsx::*;

let mut table = Table::new();
let mut rev_col = TableColumn::new("Revenue");
rev_col.set_total_function("sum");

let mut name_col = TableColumn::new("Name");
name_col.set_total_label("Total");

table.set_columns(&[name_col, rev_col]);
table.set_total_row(true);
```

Available total functions: `"sum"`, `"count"`, `"average"`, `"min"`, `"max"`, `"countNums"`, `"stdDev"`, `"var"`.

### Table Name and Autofilter

```rust
use zavora_xlsx::*;

let mut table = Table::new();
table.set_name("SalesData");
table.set_autofilter(true);  // Enabled by default
```

### Table Alt Text (Accessibility)

```rust
use zavora_xlsx::*;

let mut table = Table::new();
table.set_alt_text("Sales Data Table", "Quarterly sales data by region for FY2024");
```

### Custom Table Styles

```rust
use zavora_xlsx::*;

let custom = CustomTableStyle::new("MyStyle")
    .header_row(
        TableStyleElementFormat::new()
            .bg_color([0x44, 0x72, 0xC4])
            .font_color([0xFF, 0xFF, 0xFF])
            .bold()
    )
    .first_row_stripe(
        TableStyleElementFormat::new()
            .bg_color([0xD6, 0xE4, 0xF0])
    )
    .second_row_stripe(
        TableStyleElementFormat::new()
            .bg_color([0xFF, 0xFF, 0xFF])
    )
    .first_row_stripe_size(1)
    .second_row_stripe_size(1);

let mut table = Table::new();
table.set_custom_style(custom);
```

### Reading Tables

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::open_readonly("data.xlsx")?;
    let ws = wb.worksheet(0)?;

    for table in ws.tables() {
        println!("Name: {:?}", table.table_name());
        println!("Range: ({},{}) to ({},{})",
            table.first_row(), table.first_col(),
            table.last_row(), table.last_col());
        println!("Total row: {}", table.total_row());
        for col in table.columns() {
            println!("  Column: {}", col.name());
        }
    }

    Ok(())
}
```

---

## Data Validation

Create validation rules with `DataValidation::new(rule)`:

### Dropdown List

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let dv = DataValidation::new(ValidationRule::List(
        vec!["Yes".into(), "No".into(), "Maybe".into()]
    ));
    ws.add_data_validation(1, 0, 10, 0, &dv)?;

    wb.save("validation.xlsx")?;
    Ok(())
}
```

### List from Cell Range

```rust
use zavora_xlsx::*;

let dv = DataValidation::new(ValidationRule::ListRange("Sheet2!$A$1:$A$10".into()));
```

### Whole Number Range

```rust
use zavora_xlsx::*;

let dv = DataValidation::new(ValidationRule::WholeNumber {
    min: Some(1),
    max: Some(100),
});
```

### Decimal Range

```rust
use zavora_xlsx::*;

let dv = DataValidation::new(ValidationRule::Decimal {
    min: Some(0.0),
    max: Some(1.0),
});
```

### Date Range

```rust
use zavora_xlsx::*;

let dv = DataValidation::new(ValidationRule::DateRange {
    min: Some("2024-01-01".into()),
    max: Some("2024-12-31".into()),
});
```

### Text Length

```rust
use zavora_xlsx::*;

let dv = DataValidation::new(ValidationRule::TextLength {
    min: Some(1),
    max: Some(50),
});
```

### Custom Formula

```rust
use zavora_xlsx::*;

let dv = DataValidation::new(ValidationRule::Custom("AND(A1>0,A1<1000)".into()));
```

### Input and Error Messages

```rust
use zavora_xlsx::*;

let mut dv = DataValidation::new(ValidationRule::WholeNumber {
    min: Some(1),
    max: Some(100),
});
dv.set_input_message("Enter a number", "Please enter a value between 1 and 100");
dv.set_error_message(ErrorStyle::Stop, "Invalid", "Value must be between 1 and 100");
```

### Error Styles

| Style | Description |
|-------|-------------|
| `ErrorStyle::Stop` | Prevents invalid entry |
| `ErrorStyle::Warning` | Warns but allows entry |
| `ErrorStyle::Information` | Informational message |

### Reading Validations

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::open_readonly("data.xlsx")?;
    let ws = wb.worksheet(0)?;

    for dv in ws.validations() {
        println!("Rule: {:?}", dv.rule());
        println!("Range: ({},{}) to ({},{})",
            dv.first_row(), dv.first_col(),
            dv.last_row(), dv.last_col());
        println!("Input: {:?}", dv.input_message());
        println!("Error style: {:?}", dv.error_style());
    }

    Ok(())
}
```

---

## Conditional Formatting

All conditional format types implement the `ConditionalFormat` trait and are added via `add_conditional_format(r1, c1, r2, c2, rule)`.

### Cell Value Rule

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Greater than 100
    ws.add_conditional_format(1, 0, 10, 0,
        ConditionalFormatCell::new(CfOperator::GreaterThan, 100.0))?;

    // Between 50 and 100
    let mut cf = ConditionalFormatCell::new(CfOperator::Between, 50.0);
    cf.set_value2(100.0);
    ws.add_conditional_format(1, 1, 10, 1, cf)?;

    // With custom format
    let mut cf = ConditionalFormatCell::new(CfOperator::LessThan, 0.0);
    cf.set_format(&Format::new().font_color("#FF0000").bold());
    ws.add_conditional_format(1, 2, 10, 2, cf)?;

    wb.save("cf.xlsx")?;
    Ok(())
}
```

#### Operators

| Operator | Description |
|----------|-------------|
| `CfOperator::GreaterThan` | Greater than value |
| `CfOperator::LessThan` | Less than value |
| `CfOperator::Between` | Between value and value2 |
| `CfOperator::EqualTo` | Equal to value |
| `CfOperator::NotEqualTo` | Not equal to value |
| `CfOperator::GreaterThanOrEqual` | Greater than or equal |
| `CfOperator::LessThanOrEqual` | Less than or equal |

### 2-Color Scale

```rust
use zavora_xlsx::*;

ws.add_conditional_format(1, 0, 10, 0,
    ConditionalFormat2ColorScale::new("#FFFFFF", "#FF0000"))?;
```

### 3-Color Scale

```rust
use zavora_xlsx::*;

ws.add_conditional_format(1, 0, 10, 0,
    ConditionalFormat3ColorScale::new("#F8696B", "#FFEB84", "#63BE7B"))?;
```

### Data Bars

```rust
use zavora_xlsx::*;

// Solid data bars
ws.add_conditional_format(1, 0, 10, 0,
    ConditionalFormatDataBar::new("#4472C4"))?;

// Gradient data bars
let mut db = ConditionalFormatDataBar::new("#4472C4");
db.set_gradient(true);
ws.add_conditional_format(1, 1, 10, 1, db)?;
```

### Icon Sets

```rust
use zavora_xlsx::*;

ws.add_conditional_format(1, 0, 10, 0,
    ConditionalFormatIconSet::new(IconSetType::ThreeArrows))?;
```

| Icon Set Type | Icons |
|---------------|-------|
| `IconSetType::ThreeArrows` | 3 arrows (up/side/down) |
| `IconSetType::ThreeTrafficLights` | 3 traffic lights |
| `IconSetType::ThreeSymbols` | 3 symbols |
| `IconSetType::FourArrows` | 4 arrows |
| `IconSetType::FiveArrows` | 5 arrows |

### Formula-Based

```rust
use zavora_xlsx::*;

// Highlight every other row
let mut cf = ConditionalFormatFormula::new("MOD(ROW(),2)=0");
cf.set_format(&Format::new().background_color("#F2F2F2"));
ws.add_conditional_format(1, 0, 100, 5, cf)?;
```

### Top/Bottom N

```rust
use zavora_xlsx::*;

// Top 5 values
ws.add_conditional_format(1, 0, 100, 0,
    ConditionalFormatTopBottom::new(TopBottomType::Top, 5))?;

// Bottom 10%
ws.add_conditional_format(1, 1, 100, 1,
    ConditionalFormatTopBottom::new(TopBottomType::BottomPercent, 10))?;
```

| Type | Description |
|------|-------------|
| `TopBottomType::Top` | Top N values |
| `TopBottomType::Bottom` | Bottom N values |
| `TopBottomType::TopPercent` | Top N percent |
| `TopBottomType::BottomPercent` | Bottom N percent |

### Text Rules

```rust
use zavora_xlsx::*;

ws.add_conditional_format(1, 0, 10, 0,
    ConditionalFormatText::new(TextOperator::Contains, "overdue"))?;

ws.add_conditional_format(1, 1, 10, 1,
    ConditionalFormatText::new(TextOperator::BeginsWith, "URGENT"))?;

ws.add_conditional_format(1, 2, 10, 2,
    ConditionalFormatText::new(TextOperator::EndsWith, ".com"))?;

ws.add_conditional_format(1, 3, 10, 3,
    ConditionalFormatText::new(TextOperator::NotContains, "draft"))?;
```

### Duplicate and Unique Values

```rust
use zavora_xlsx::*;

// Highlight duplicates
let mut cf = ConditionalFormatDuplicate::new();
cf.set_format(&Format::new().background_color("#FFC7CE"));
ws.add_conditional_format(1, 0, 100, 0, cf)?;

// Highlight unique values
let mut cf = ConditionalFormatUnique::new();
cf.set_format(&Format::new().background_color("#C6EFCE"));
ws.add_conditional_format(1, 1, 100, 1, cf)?;
```

### Above/Below Average

```rust
use zavora_xlsx::*;

ws.add_conditional_format(1, 0, 100, 0,
    ConditionalFormatAverage::new(AverageType::Above))?;

ws.add_conditional_format(1, 1, 100, 1,
    ConditionalFormatAverage::new(AverageType::Below))?;

ws.add_conditional_format(1, 2, 100, 2,
    ConditionalFormatAverage::new(AverageType::AboveOrEqual))?;

ws.add_conditional_format(1, 3, 100, 3,
    ConditionalFormatAverage::new(AverageType::BelowOrEqual))?;
```

### Date Occurring

```rust
use zavora_xlsx::*;

ws.add_conditional_format(1, 0, 100, 0,
    ConditionalFormatDate::new(DateOccurring::Today))?;

ws.add_conditional_format(1, 1, 100, 1,
    ConditionalFormatDate::new(DateOccurring::ThisWeek))?;

ws.add_conditional_format(1, 2, 100, 2,
    ConditionalFormatDate::new(DateOccurring::LastMonth))?;
```

| Period | Description |
|--------|-------------|
| `DateOccurring::Yesterday` | Yesterday |
| `DateOccurring::Today` | Today |
| `DateOccurring::Tomorrow` | Tomorrow |
| `DateOccurring::Last7Days` | Last 7 days |
| `DateOccurring::ThisWeek` | This week |
| `DateOccurring::LastWeek` | Last week |
| `DateOccurring::NextWeek` | Next week |
| `DateOccurring::ThisMonth` | This month |
| `DateOccurring::LastMonth` | Last month |
| `DateOccurring::NextMonth` | Next month |

---

## Autofilter

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Enable autofilter on range (header row through data)
    ws.set_autofilter(0, 0, 100, 5);

    // Filter a specific column to show only certain values
    ws.filter_column(1, &["Active", "Pending"]);

    // Advanced filter: Top 10
    ws.filter_column_advanced(2, FilterRule::Top10 {
        top: true,
        percent: false,
        val: 10.0,
    });

    // Advanced filter: Date filter
    ws.filter_column_advanced(3, FilterRule::DateFilter {
        year: 2024,
        month: Some(6),
        day: None,
    });

    // Advanced filter: Custom filter
    ws.filter_column_advanced(4, FilterRule::CustomFilter {
        and: true,
        conditions: vec![
            ("greaterThan".into(), "100".into()),
            ("lessThan".into(), "500".into()),
        ],
    });

    wb.save("filter.xlsx")?;
    Ok(())
}
```

## Sort State

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    ws.set_autofilter(0, 0, 100, 5);
    ws.set_sort(1, SortDirection::Ascending);
    ws.set_sort(2, SortDirection::Descending);  // Secondary sort

    wb.save("sorted.xlsx")?;
    Ok(())
}
```

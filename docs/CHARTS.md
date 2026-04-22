# Charts Guide

Complete reference for creating charts in zavora-xlsx.

## Standard Chart Types

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Column);
```

| Type | Description |
|------|-------------|
| `ChartType::Column` | Vertical bar chart |
| `ChartType::Bar` | Horizontal bar chart |
| `ChartType::Line` | Line chart |
| `ChartType::Pie` | Pie chart |
| `ChartType::Scatter` | XY scatter plot |
| `ChartType::Area` | Area chart |
| `ChartType::Doughnut` | Doughnut chart |
| `ChartType::Radar` | Radar/spider chart |
| `ChartType::Stock` | Stock chart (OHLC) |
| `ChartType::Bubble` | Bubble chart |

## 3D Chart Types

| Type | Description |
|------|-------------|
| `ChartType::Column3D` | 3D column chart |
| `ChartType::Bar3D` | 3D bar chart |
| `ChartType::Line3D` | 3D line chart |
| `ChartType::Pie3D` | 3D pie chart |
| `ChartType::Area3D` | 3D area chart |
| `ChartType::Surface` | 3D surface chart |
| `ChartType::WireframeSurface` | Wireframe surface chart |
| `ChartType::Map` | Map chart (Bing Maps) |

## Basic Chart Example

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Write data
    ws.write_row(0, 0, ["Quarter", "North", "South"])?;
    ws.write_row(1, 0, ["Q1", "100", "80"])?;
    ws.write_row(2, 0, ["Q2", "120", "90"])?;
    ws.write_row(3, 0, ["Q3", "140", "110"])?;
    ws.write_row(4, 0, ["Q4", "160", "130"])?;

    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Revenue by Quarter");
    chart.set_x_axis_name("Quarter");
    chart.set_y_axis_name("Revenue ($)");
    chart.set_legend_position(LegendPosition::Bottom);
    chart.set_width(600);
    chart.set_height(400);

    chart.add_series()
        .set_values("Sheet1!$B$2:$B$5")
        .set_categories("Sheet1!$A$2:$A$5")
        .set_name("North");

    chart.add_series()
        .set_values("Sheet1!$C$2:$C$5")
        .set_categories("Sheet1!$A$2:$A$5")
        .set_name("South");

    ws.insert_chart(6, 0, &chart)?;
    wb.save("chart.xlsx")?;
    Ok(())
}
```

## Series Configuration

### Values, Categories, and Name

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Line);
chart.add_series()
    .set_values("Sheet1!$B$2:$B$10")       // Required: data values
    .set_categories("Sheet1!$A$2:$A$10")    // Optional: category labels
    .set_name("Sales");                      // Optional: legend name
```

### Series Color

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Column);
chart.add_series()
    .set_values("Sheet1!$B$2:$B$5")
    .set_color("#FF6600");
```

### Point Colors (Individual Data Points)

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Pie);
chart.add_series()
    .set_values("Sheet1!$B$2:$B$5")
    .set_categories("Sheet1!$A$2:$A$5")
    .set_point_color(0, "#4472C4")
    .set_point_color(1, "#ED7D31")
    .set_point_color(2, "#A5A5A5")
    .set_point_color(3, "#FFC000");
```

### Data Labels

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Column);
chart.add_series()
    .set_values("Sheet1!$B$2:$B$5")
    .set_data_labels(true);
```

### Trendlines

Six trendline types:

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Scatter);

chart.add_series()
    .set_values("Sheet1!$B$2:$B$20")
    .set_trendline(TrendlineType::Linear);

chart.add_series()
    .set_values("Sheet1!$C$2:$C$20")
    .set_trendline(TrendlineType::Exponential);

// Polynomial with degree
chart.add_series()
    .set_values("Sheet1!$D$2:$D$20")
    .set_trendline(TrendlineType::Polynomial(3));

// Moving average with period
chart.add_series()
    .set_values("Sheet1!$E$2:$E$20")
    .set_trendline(TrendlineType::MovingAverage(4));

// Display R² value and equation
chart.add_series()
    .set_values("Sheet1!$F$2:$F$20")
    .set_trendline(TrendlineType::Linear)
    .set_trendline_display_rsquared(true)
    .set_trendline_display_equation(true);
```

| Trendline Type | Description |
|----------------|-------------|
| `TrendlineType::Linear` | Linear regression |
| `TrendlineType::Exponential` | Exponential curve |
| `TrendlineType::Polynomial(n)` | Polynomial of degree n |
| `TrendlineType::Power` | Power curve |
| `TrendlineType::Logarithmic` | Logarithmic curve |
| `TrendlineType::MovingAverage(n)` | Moving average with period n |

### Markers

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Line);
chart.add_series()
    .set_values("Sheet1!$B$2:$B$10")
    .set_marker(MarkerType::Circle)
    .set_marker_size(8);
```

| Marker Type | Description |
|-------------|-------------|
| `MarkerType::None` | No marker |
| `MarkerType::Circle` | Circle |
| `MarkerType::Diamond` | Diamond |
| `MarkerType::Square` | Square |
| `MarkerType::Triangle` | Triangle |
| `MarkerType::Star` | Star |
| `MarkerType::Plus` | Plus sign |
| `MarkerType::X` | X mark |

### Line Width and Dash Style

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Line);
chart.add_series()
    .set_values("Sheet1!$B$2:$B$10")
    .set_line_width(2.5)
    .set_dash_style(DashStyle::DashDot);
```

| Dash Style | Description |
|------------|-------------|
| `DashStyle::Solid` | Solid line |
| `DashStyle::Dash` | Dashed |
| `DashStyle::Dot` | Dotted |
| `DashStyle::DashDot` | Dash-dot |
| `DashStyle::LongDash` | Long dash |
| `DashStyle::LongDashDot` | Long dash-dot |

### Series Gradient Fill

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Column);
chart.add_series()
    .set_values("Sheet1!$B$2:$B$5")
    .set_gradient(vec![
        ([0x44, 0x72, 0xC4], 0.0),
        ([0x5B, 0x9B, 0xD5], 1.0),
    ]);
```

### Error Bars

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Column);
chart.add_series()
    .set_values("Sheet1!$B$2:$B$5")
    .set_error_bars(ErrorBar::new(
        ErrorBarType::Both,
        ErrorBarValueType::Percentage,
        10.0,
    ));
```

| Error Bar Type | Description |
|----------------|-------------|
| `ErrorBarType::Both` | Both directions |
| `ErrorBarType::Plus` | Positive only |
| `ErrorBarType::Minus` | Negative only |

| Value Type | Description |
|------------|-------------|
| `ErrorBarValueType::FixedValue` | Fixed value |
| `ErrorBarValueType::Percentage` | Percentage of value |
| `ErrorBarValueType::StandardDeviation` | Standard deviation |
| `ErrorBarValueType::StandardError` | Standard error |

### Bubble Charts

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Bubble);
chart.add_series()
    .set_values("Sheet1!$B$2:$B$5")          // Y values
    .set_categories("Sheet1!$A$2:$A$5")      // X values
    .set_bubble_sizes("Sheet1!$C$2:$C$5")    // Bubble sizes
    .set_name("Products");
```

## Chart Configuration

### Title and Axis Names

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Column);
chart.set_title("Sales Report");
chart.set_x_axis_name("Month");
chart.set_y_axis_name("Revenue ($)");
chart.set_y2_axis_name("Margin %");  // Secondary Y axis name
```

### Legend Position

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Column);
chart.set_legend_position(LegendPosition::Bottom);
```

| Position | Description |
|----------|-------------|
| `LegendPosition::Top` | Above chart |
| `LegendPosition::Bottom` | Below chart |
| `LegendPosition::Left` | Left of chart |
| `LegendPosition::Right` | Right of chart |
| `LegendPosition::None` | No legend |

### Size

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Column);
chart.set_width(800);   // Width in pixels
chart.set_height(500);  // Height in pixels
```

### Data Table

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Column);
chart.show_data_table(true);
```

### Axis Min/Max/Log and Reverse

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Line);
chart.set_y_axis_min(0.0);
chart.set_y_axis_max(1000.0);
chart.set_y_axis_log_base(10.0);  // Logarithmic scale
chart.set_x_axis_reverse();        // Reverse X axis
chart.set_y_axis_reverse();        // Reverse Y axis
```

### Drop Lines and High-Low Lines

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Line);
chart.set_drop_lines(true);       // Vertical lines from points to X axis
chart.set_high_low_lines(true);   // Lines connecting high/low values
```

## 3D View Settings

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Column3D);
chart.set_view3d(View3D {
    rot_x: 15,              // X rotation (-90 to 90)
    rot_y: 20,              // Y rotation (0 to 360)
    perspective: 30,         // Perspective (0 to 200)
    right_angle_axes: true,  // Use right-angle axes
});
```

## Chart Styles (1–48)

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Column);
chart.set_style(2);  // Excel chart style number (1-48)
```

## Axis Formatting

```rust
use zavora_xlsx::*;

let mut x_fmt = AxisFormat::new();
x_fmt.set_num_format("mmm yyyy");
x_fmt.set_font_name("Arial");
x_fmt.set_font_size(10.0);
x_fmt.set_font_color([0x44, 0x54, 0x6A]);
x_fmt.set_font_bold(true);
x_fmt.set_major_tick_mark(TickMark::Outside);
x_fmt.set_minor_tick_mark(TickMark::None);

let mut y_fmt = AxisFormat::new();
y_fmt.set_num_format("$#,##0");
y_fmt.set_major_gridlines(true);
y_fmt.set_minor_gridlines(false);
y_fmt.set_gridline_color([0xD9, 0xD9, 0xD9]);
y_fmt.set_gridline_width(0.5);

let mut chart = Chart::new(ChartType::Column);
chart.set_x_axis_format(x_fmt);
chart.set_y_axis_format(y_fmt);
```

### Tick Mark Types

| Type | Description |
|------|-------------|
| `TickMark::None` | No tick marks |
| `TickMark::Inside` | Inside the axis |
| `TickMark::Outside` | Outside the axis |
| `TickMark::Cross` | Both inside and outside |

## Plot Area Formatting

```rust
use zavora_xlsx::*;

let mut plot_fmt = PlotAreaFormat::new();
plot_fmt.set_fill([0xF2, 0xF2, 0xF2]);
plot_fmt.set_border([0xD9, 0xD9, 0xD9]);

// Or with gradient
plot_fmt.set_gradient(vec![
    ([0xFF, 0xFF, 0xFF], 0.0),
    ([0xF2, 0xF2, 0xF2], 1.0),
]);

let mut chart = Chart::new(ChartType::Column);
chart.set_plot_area_format(plot_fmt);
```

## Combo Charts

Override the chart type per series and use a secondary axis:

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Write data...

    let mut chart = Chart::new(ChartType::Column);
    chart.set_title("Revenue vs Margin");

    // Column series on primary axis
    chart.add_series()
        .set_values("Sheet1!$B$2:$B$5")
        .set_name("Revenue");

    // Line series on secondary axis
    chart.add_series()
        .set_values("Sheet1!$C$2:$C$5")
        .set_name("Margin %")
        .set_chart_type(ChartType::Line)
        .set_secondary_axis(true);

    chart.set_y2_axis_name("Margin %");
    ws.insert_chart(6, 0, &chart)?;

    wb.save("combo.xlsx")?;
    Ok(())
}
```

## Pivot Charts

Link a chart to a pivot table:

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Column);
chart.set_title("Sales by Region");
chart.set_pivot_source("SalesPivot", "PivotSheet");

// Check if chart is a pivot chart
assert!(chart.is_pivot_chart());
```

## Chart Sheets

A chart sheet is a dedicated sheet containing only a chart (no cells):

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    // Write data to Sheet1...

    let mut chart = Chart::new(ChartType::Column);
    chart.add_series().set_values("Sheet1!$B$2:$B$5");
    chart.set_title("Full-Page Chart");

    wb.add_chart_sheet("Chart1", chart)?;
    wb.save("chart_sheet.xlsx")?;
    Ok(())
}
```

## Accessibility

```rust
use zavora_xlsx::*;

let mut chart = Chart::new(ChartType::Column);
chart.set_alt_text("Revenue Chart", "Bar chart showing quarterly revenue for 2024");
```

## Chart Positioning with Offset

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let chart = Chart::new(ChartType::Column);
    // Insert at row 6, col 0 with 10px X offset and 5px Y offset
    ws.insert_chart_with_offset(6, 0, &chart, 10, 5)?;

    wb.save("offset.xlsx")?;
    Ok(())
}
```

## ChartEx Types (Excel 2016+)

These use the `cx:` namespace and have dedicated insert methods on the worksheet.

### Waterfall Chart

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let mut wf = WaterfallChart::new();
    wf.set_title("Cash Flow");
    wf.add_point("Revenue", 500.0, WaterfallPointType::Increase);
    wf.add_point("COGS", -200.0, WaterfallPointType::Decrease);
    wf.add_point("Gross Profit", 300.0, WaterfallPointType::Total);
    wf.add_point("OpEx", -150.0, WaterfallPointType::Decrease);
    wf.add_point("Net Income", 150.0, WaterfallPointType::Total);

    ws.insert_waterfall(0, 0, &wf)?;
    wb.save("waterfall.xlsx")?;
    Ok(())
}
```

### Funnel Chart

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let mut funnel = FunnelChart::new();
    funnel.set_title("Sales Pipeline");
    funnel.add_point("Leads", 1000.0);
    funnel.add_point("Qualified", 600.0);
    funnel.add_point("Proposals", 300.0);
    funnel.add_point("Closed", 100.0);

    ws.insert_funnel(0, 0, &funnel)?;
    wb.save("funnel.xlsx")?;
    Ok(())
}
```

### Sunburst Chart

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let mut sb = SunburstChart::new();
    sb.set_title("Organization");
    sb.add_level(&["Engineering", "Engineering", "Sales", "Sales"]);
    sb.add_level(&["Frontend", "Backend", "US", "EU"]);
    sb.set_values(&[30.0, 40.0, 50.0, 20.0]);

    ws.insert_sunburst(0, 0, &sb)?;
    wb.save("sunburst.xlsx")?;
    Ok(())
}
```

### Histogram Chart

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let mut hist = HistogramChart::new();
    hist.set_title("Score Distribution");
    hist.set_values(&[65.0, 72.0, 78.0, 81.0, 85.0, 88.0, 92.0, 95.0, 98.0]);
    hist.set_bin_count(5);

    ws.insert_histogram(0, 0, &hist)?;

    // Pareto chart (histogram + cumulative line)
    let mut pareto = HistogramChart::pareto();
    pareto.set_title("Pareto Analysis");
    pareto.set_values(&[50.0, 30.0, 15.0, 5.0]);

    ws.insert_histogram(0, 8, &pareto)?;
    wb.save("histogram.xlsx")?;
    Ok(())
}
```

### Box & Whisker Chart

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let mut bw = BoxWhiskerChart::new();
    bw.set_title("Test Scores");
    bw.add_data_set("Class A", &[65.0, 72.0, 78.0, 81.0, 85.0, 92.0]);
    bw.add_data_set("Class B", &[55.0, 68.0, 74.0, 79.0, 88.0, 95.0]);
    bw.set_show_outliers(true);
    bw.set_show_mean_markers(true);
    bw.set_show_inner_points(false);

    ws.insert_box_whisker(0, 0, &bw)?;
    wb.save("box_whisker.xlsx")?;
    Ok(())
}
```

### Map Chart

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let mut map = MapChart::new();
    map.set_title("Sales by Country");
    map.add_point("United States", 500.0);
    map.add_point("Germany", 300.0);
    map.add_point("Japan", 200.0);
    map.set_map_level(MapLevel::Country);

    ws.insert_map(0, 0, &map)?;
    wb.save("map.xlsx")?;
    Ok(())
}
```

### Treemap Chart

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let mut tm = TreemapChart::new();
    tm.set_title("Market Share");
    tm.add_point("Product A", 45.0);
    tm.add_point("Product B", 30.0);
    tm.add_point("Product C", 15.0);
    tm.add_point_with_color("Product D", 10.0, "#FF6600");

    ws.insert_treemap(0, 0, &tm)?;
    wb.save("treemap.xlsx")?;
    Ok(())
}
```

## Reading Charts

When opening existing files, charts are available via the worksheet:

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::open_readonly("data.xlsx")?;
    let ws = wb.worksheet(0)?;

    for chart in ws.charts() {
        println!("Type: {:?}", chart.chart_type());
        println!("Title: {:?}", chart.title());
        println!("Series count: {}", chart.series().len());
        for s in chart.series() {
            println!("  Values: {}", s.values());
            println!("  Name: {:?}", s.name());
        }
    }

    Ok(())
}
```

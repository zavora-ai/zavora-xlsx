# Formatting Guide

Complete reference for cell formatting in zavora-xlsx. All `Format` methods consume `self` and return `Self`, enabling builder-style chaining.

```rust
use zavora_xlsx::*;

let fmt = Format::new().bold().italic().font_size(14.0).font_color("#FF0000");
```

## Font

```rust
use zavora_xlsx::*;

// Bold, italic, strikethrough
let fmt = Format::new().bold().italic().strikethrough();

// Underline (Single or Double)
let fmt = Format::new().underline(Underline::Single);
let fmt = Format::new().underline(Underline::Double);

// Font size (points)
let fmt = Format::new().font_size(16.0);

// Font name
let fmt = Format::new().font_name("Arial");

// Font color
let fmt = Format::new().font_color("#FF0000");
let fmt = Format::new().font_color((255, 0, 0));
let fmt = Format::new().font_color(Color::Named(NamedColor::Red));
```

## Text Effects

```rust
use zavora_xlsx::*;

let fmt = Format::new().shadow();    // Shadow effect
let fmt = Format::new().outline();   // Outline effect
let fmt = Format::new().emboss();    // Emboss effect
let fmt = Format::new().engrave();   // Engrave (imprint) effect
```

## Background and Fill Colors

```rust
use zavora_xlsx::*;

// Solid background color
let fmt = Format::new().background_color("#4472C4");

// Pattern fill with foreground color
let fmt = Format::new()
    .pattern_fill(Pattern::DarkDown)
    .foreground_color("#FF0000")
    .background_color("#FFFFFF");
```

### Pattern Fill Types

All 18 pattern types:

| Pattern | Description |
|---------|-------------|
| `Pattern::None` | No fill |
| `Pattern::Solid` | Solid fill |
| `Pattern::MediumGray` | 50% gray |
| `Pattern::DarkGray` | 75% gray |
| `Pattern::LightGray` | 25% gray |
| `Pattern::DarkHorizontal` | Dark horizontal lines |
| `Pattern::DarkVertical` | Dark vertical lines |
| `Pattern::DarkDown` | Dark diagonal down |
| `Pattern::DarkUp` | Dark diagonal up |
| `Pattern::DarkGrid` | Dark grid |
| `Pattern::DarkTrellis` | Dark trellis |
| `Pattern::LightHorizontal` | Light horizontal lines |
| `Pattern::LightVertical` | Light vertical lines |
| `Pattern::LightDown` | Light diagonal down |
| `Pattern::LightUp` | Light diagonal up |
| `Pattern::LightGrid` | Light grid |
| `Pattern::LightTrellis` | Light trellis |
| `Pattern::Gray125` | 12.5% gray |

## Gradient Fills

```rust
use zavora_xlsx::*;

let fmt = Format::new().gradient_fill(90.0, vec![
    GradientStop { position: 0.0, color: [0x44, 0x72, 0xC4] },
    GradientStop { position: 1.0, color: [0xFF, 0xFF, 0xFF] },
]);
```

The angle is in degrees (0 = left-to-right, 90 = top-to-bottom). You can add multiple stops for complex gradients.

## Borders

### All Sides

```rust
use zavora_xlsx::*;

// Same style on all four sides
let fmt = Format::new().border(BorderStyle::Thin);

// With color
let fmt = Format::new().border(BorderStyle::Thin).border_color("#000000");
```

### Individual Sides

```rust
use zavora_xlsx::*;

let fmt = Format::new()
    .border_top(BorderStyle::Thick)
    .border_bottom(BorderStyle::Thin)
    .border_left(BorderStyle::Medium)
    .border_right(BorderStyle::Dashed);
```

### Individual Side Colors

```rust
use zavora_xlsx::*;

let fmt = Format::new()
    .border(BorderStyle::Thin)
    .border_top_color("#FF0000")
    .border_bottom_color("#00FF00")
    .border_left_color("#0000FF")
    .border_right_color("#000000");
```

### Diagonal Borders

```rust
use zavora_xlsx::*;

let fmt = Format::new().diagonal_border(BorderStyle::Thin, DiagonalType::Up);
let fmt = Format::new().diagonal_border(BorderStyle::Thin, DiagonalType::Down);
let fmt = Format::new().diagonal_border(BorderStyle::Thin, DiagonalType::Both);
```

### Border Styles

| Style | Description |
|-------|-------------|
| `BorderStyle::None` | No border |
| `BorderStyle::Thin` | Thin line |
| `BorderStyle::Medium` | Medium line |
| `BorderStyle::Thick` | Thick line |
| `BorderStyle::Dashed` | Dashed line |
| `BorderStyle::Dotted` | Dotted line |
| `BorderStyle::Double` | Double line |

## Alignment

### Horizontal Alignment

```rust
use zavora_xlsx::*;

let fmt = Format::new().align(Align::Left);
let fmt = Format::new().align(Align::Center);
let fmt = Format::new().align(Align::Right);
let fmt = Format::new().align(Align::Fill);
let fmt = Format::new().align(Align::Justify);
```

### Vertical Alignment

```rust
use zavora_xlsx::*;

let fmt = Format::new().align(Align::Top);
let fmt = Format::new().align(Align::VerticalCenter);
let fmt = Format::new().align(Align::Bottom);
```

### Combined Alignment

Chain both horizontal and vertical:

```rust
use zavora_xlsx::*;

let fmt = Format::new()
    .align(Align::Center)
    .align(Align::VerticalCenter);
```

### Text Wrap, Shrink, Indent, Rotation

```rust
use zavora_xlsx::*;

let fmt = Format::new().text_wrap();           // Wrap text in cell
let fmt = Format::new().shrink_to_fit();       // Shrink text to fit cell width
let fmt = Format::new().indent(2);             // Indent level (0–15)
let fmt = Format::new().rotation(45);          // Rotation angle (-90 to 90)
```

## Number Formats

Use Excel format strings with `num_format`:

```rust
use zavora_xlsx::*;

// General number formats
let fmt = Format::new().num_format("0");              // Integer: 1234
let fmt = Format::new().num_format("0.00");           // 2 decimals: 1234.56
let fmt = Format::new().num_format("#,##0");          // Thousands separator: 1,234
let fmt = Format::new().num_format("#,##0.00");       // Both: 1,234.56

// Currency
let fmt = Format::new().num_format("$#,##0.00");      // $1,234.56
let fmt = Format::new().num_format("€#,##0.00");      // €1,234.56

// Percentage
let fmt = Format::new().num_format("0%");              // 50%
let fmt = Format::new().num_format("0.00%");           // 50.00%

// Date formats
let fmt = Format::new().num_format("yyyy-mm-dd");      // 2024-06-15
let fmt = Format::new().num_format("mm/dd/yyyy");      // 06/15/2024
let fmt = Format::new().num_format("d-mmm-yyyy");      // 15-Jun-2024
let fmt = Format::new().num_format("dddd, mmmm d, yyyy"); // Saturday, June 15, 2024

// Time formats
let fmt = Format::new().num_format("hh:mm:ss");        // 14:30:00
let fmt = Format::new().num_format("h:mm AM/PM");      // 2:30 PM

// Date + time
let fmt = Format::new().num_format("yyyy-mm-dd hh:mm:ss");

// Scientific notation
let fmt = Format::new().num_format("0.00E+00");        // 1.23E+03

// Fractions
let fmt = Format::new().num_format("# ?/?");           // 1 1/2

// Text
let fmt = Format::new().num_format("@");               // Force text display

// Accounting (with alignment)
let fmt = Format::new().num_format("_(\"$\"* #,##0.00_)");
```

> **Note:** Dates written via `ExcelDateTime` automatically get a `yyyy-mm-dd` format if no explicit format is set.

## Protection

```rust
use zavora_xlsx::*;

// Lock cell (default for protected sheets)
let fmt = Format::new().locked();

// Unlock cell (allows editing on protected sheets)
let fmt = Format::new().unlocked();

// Hide formula in formula bar
let fmt = Format::new().formula_hidden();
```

## Theme Colors

Use theme color indices with optional tint adjustment:

```rust
use zavora_xlsx::*;

// Accent1 color (default blue: #4472C4)
let fmt = Format::new().theme_color(ThemeColorIndex::Accent1, 0.0);

// Accent1 lightened by 40%
let fmt = Format::new().theme_color(ThemeColorIndex::Accent1, 0.4);

// Accent1 darkened by 25%
let fmt = Format::new().theme_color(ThemeColorIndex::Accent1, -0.25);
```

### Theme Color Indices

| Index | Name | Default RGB |
|-------|------|-------------|
| `ThemeColorIndex::Dark1` | Dark 1 | `#000000` |
| `ThemeColorIndex::Light1` | Light 1 | `#FFFFFF` |
| `ThemeColorIndex::Dark2` | Dark 2 | `#44546A` |
| `ThemeColorIndex::Light2` | Light 2 | `#E7E6E6` |
| `ThemeColorIndex::Accent1` | Accent 1 | `#4472C4` |
| `ThemeColorIndex::Accent2` | Accent 2 | `#ED7D31` |
| `ThemeColorIndex::Accent3` | Accent 3 | `#A5A5A5` |
| `ThemeColorIndex::Accent4` | Accent 4 | `#FFC000` |
| `ThemeColorIndex::Accent5` | Accent 5 | `#5B9BD5` |
| `ThemeColorIndex::Accent6` | Accent 6 | `#70AD47` |

Tint ranges from `-1.0` (fully dark) to `1.0` (fully light). `0.0` means no tint.

## Cell Styles

Apply named Excel cell styles:

```rust
use zavora_xlsx::*;

let fmt = Format::new().cell_style("Normal");
let fmt = Format::new().cell_style("Heading 1");
let fmt = Format::new().cell_style("Currency");
let fmt = Format::new().cell_style("Percent");
```

## Color Types

Three ways to specify colors anywhere a color is accepted:

```rust
use zavora_xlsx::*;

// Hex string (with or without #)
Format::new().font_color("#FF0000");
Format::new().font_color("4472C4");

// RGB tuple
Format::new().font_color((255, 0, 0));

// Named color enum
Format::new().font_color(Color::Named(NamedColor::Red));
```

### Named Colors

| Color | RGB |
|-------|-----|
| `NamedColor::Black` | `(0, 0, 0)` |
| `NamedColor::White` | `(255, 255, 255)` |
| `NamedColor::Red` | `(255, 0, 0)` |
| `NamedColor::Green` | `(0, 128, 0)` |
| `NamedColor::Blue` | `(0, 0, 255)` |
| `NamedColor::Yellow` | `(255, 255, 0)` |
| `NamedColor::Cyan` | `(0, 255, 255)` |
| `NamedColor::Magenta` | `(255, 0, 255)` |
| `NamedColor::Orange` | `(255, 165, 0)` |
| `NamedColor::Purple` | `(128, 0, 128)` |
| `NamedColor::Gray` | `(128, 128, 128)` |

Any type implementing `IntoColor` works: `&str`, `(u8, u8, u8)`, `Color`, `NamedColor`.

## Applying Formats

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    let fmt = Format::new().bold().background_color("#FFFF00");

    // Write with format
    ws.write_with_format(0, 0, "Bold Yellow", &fmt)?;

    // Apply format to existing cell
    ws.set_cell_format(1, 0, &fmt)?;

    // Apply format to a range
    ws.set_range_format(0, 0, 10, 5, &fmt)?;

    // Format entire column
    let col_fmt = Format::new().num_format("$#,##0.00");
    ws.set_column_format(1, &col_fmt);

    // Format entire row
    let row_fmt = Format::new().bold();
    ws.set_row_format(0, &row_fmt);

    // Write blank cell with format (no value, just formatting)
    ws.write_blank(2, 0, &fmt)?;

    // Quote prefix (forces text display, leading apostrophe)
    let text_fmt = Format::new().quote_prefix();
    ws.write_with_format(3, 0, "001234", &text_fmt)?;

    wb.save("formats.xlsx")?;
    Ok(())
}
```

## Reading Formats

When opening existing files, you can read back cell formatting:

```rust
use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::open_readonly("data.xlsx")?;
    let ws = wb.worksheet(0)?;

    if let Some(fmt) = ws.cell_format(0, 0) {
        println!("Bold: {}", fmt.is_bold());
        println!("Italic: {}", fmt.is_italic());
        println!("Font size: {}", fmt.get_font_size());
        println!("Font name: {}", fmt.get_font_name());
        println!("Num format: {}", fmt.get_num_format());
        println!("Wrap text: {}", fmt.is_wrap_text());

        if let Some(color) = fmt.get_font_color() {
            println!("Font color: #{:02X}{:02X}{:02X}", color[0], color[1], color[2]);
        }
        if let Some(bg) = fmt.get_bg_color() {
            println!("Background: #{:02X}{:02X}{:02X}", bg[0], bg[1], bg[2]);
        }
    }

    Ok(())
}
```

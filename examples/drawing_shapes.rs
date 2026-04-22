/// Example: Drawing Shapes (Task 72)
///
/// Demonstrates adding various shapes to a worksheet with fill colors,
/// outlines, and text bodies.
use zavora_xlsx::{Format, Shape, ShapeType, Workbook};

fn main() -> zavora_xlsx::Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;

    // Title
    let bold = Format::new().bold();
    ws.write_with_format(0, 0, "Drawing Shapes Example", &bold)?;
    ws.write(1, 0, "Various shapes are placed on this sheet.")?;

    // Rectangle with blue fill and white text
    let rect = Shape::new(ShapeType::Rectangle, 200, 80)
        .text("Rectangle")
        .fill_color([0x44, 0x72, 0xC4])
        .outline_color([0x2F, 0x52, 0x8F])
        .outline_width(1.5)
        .font_size(12.0)
        .bold();
    ws.add_shape(3, 1, &rect)?;

    // Rounded rectangle with green fill
    let rounded = Shape::new(ShapeType::RoundedRectangle, 200, 80)
        .text("Rounded Rect")
        .fill_color([0x70, 0xAD, 0x47])
        .outline_color([0x50, 0x8D, 0x27])
        .font_size(12.0);
    ws.add_shape(3, 5, &rounded)?;

    // Ellipse with orange fill
    let ellipse = Shape::new(ShapeType::Ellipse, 160, 120)
        .text("Ellipse")
        .fill_color([0xED, 0x7D, 0x31])
        .font_size(11.0);
    ws.add_shape(8, 1, &ellipse)?;

    // Diamond with purple fill
    let diamond = Shape::new(ShapeType::Diamond, 140, 140)
        .text("Diamond")
        .fill_color([0x7B, 0x2D, 0x8E])
        .font_size(11.0);
    ws.add_shape(8, 5, &diamond)?;

    // Arrow shape
    let arrow = Shape::new(ShapeType::Arrow, 200, 60)
        .text("Arrow")
        .fill_color([0xFF, 0xC0, 0x00])
        .outline_color([0xBF, 0x90, 0x00])
        .font_size(11.0);
    ws.add_shape(15, 1, &arrow)?;

    // Text box (no fill)
    let textbox = Shape::new(ShapeType::TextBox, 250, 60)
        .text("This is a text box with no fill color")
        .outline_color([0x00, 0x00, 0x00])
        .outline_width(0.5)
        .font_size(10.0);
    ws.add_shape(15, 5, &textbox)?;

    // Callout with red fill
    let callout = Shape::new(ShapeType::Callout, 200, 100)
        .text("Callout!")
        .fill_color([0xFF, 0x00, 0x00])
        .font_size(14.0)
        .bold();
    ws.add_shape(20, 1, &callout)?;

    // Triangle
    let triangle = Shape::new(ShapeType::Triangle, 120, 120)
        .text("Tri")
        .fill_color([0x00, 0xB0, 0xF0])
        .font_size(11.0);
    ws.add_shape(20, 5, &triangle)?;

    wb.save("output/drawing_shapes.xlsx")?;
    println!("Saved output/drawing_shapes.xlsx with 8 shapes.");
    Ok(())
}

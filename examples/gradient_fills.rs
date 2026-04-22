use zavora_xlsx::{Format, GradientStop, Workbook};

fn main() {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0).unwrap();

    // Simple two-color gradient (red to blue, 90 degrees)
    let fmt1 = Format::new().gradient_fill(
        90.0,
        vec![
            GradientStop {
                position: 0.0,
                color: [255, 0, 0],
            },
            GradientStop {
                position: 1.0,
                color: [0, 0, 255],
            },
        ],
    );
    ws.write_with_format(0, 0, "Red to Blue (90°)", &fmt1)
        .unwrap();

    // Diagonal gradient (green to yellow, 45 degrees)
    let fmt2 = Format::new().gradient_fill(
        45.0,
        vec![
            GradientStop {
                position: 0.0,
                color: [0, 128, 0],
            },
            GradientStop {
                position: 1.0,
                color: [255, 255, 0],
            },
        ],
    );
    ws.write_with_format(1, 0, "Green to Yellow (45°)", &fmt2)
        .unwrap();

    // Three-color gradient (red, white, blue)
    let fmt3 = Format::new().gradient_fill(
        0.0,
        vec![
            GradientStop {
                position: 0.0,
                color: [255, 0, 0],
            },
            GradientStop {
                position: 0.5,
                color: [255, 255, 255],
            },
            GradientStop {
                position: 1.0,
                color: [0, 0, 255],
            },
        ],
    );
    ws.write_with_format(2, 0, "Red-White-Blue (0°)", &fmt3)
        .unwrap();

    // Sunset gradient (orange to purple, 180 degrees)
    let fmt4 = Format::new().gradient_fill(
        180.0,
        vec![
            GradientStop {
                position: 0.0,
                color: [255, 165, 0],
            },
            GradientStop {
                position: 0.5,
                color: [255, 69, 0],
            },
            GradientStop {
                position: 1.0,
                color: [128, 0, 128],
            },
        ],
    );
    ws.write_with_format(3, 0, "Sunset (180°)", &fmt4).unwrap();

    // Gradient with bold text
    let fmt5 = Format::new().bold().font_size(14.0).gradient_fill(
        270.0,
        vec![
            GradientStop {
                position: 0.0,
                color: [0, 0, 0],
            },
            GradientStop {
                position: 1.0,
                color: [200, 200, 200],
            },
        ],
    );
    ws.write_with_format(4, 0, "Dark gradient + bold", &fmt5)
        .unwrap();

    // Set column width for readability
    let _ = ws.set_column_width(0, 25.0);

    wb.save("output/gradient_fills_example.xlsx").unwrap();
    println!("Created output/gradient_fills_example.xlsx");
}

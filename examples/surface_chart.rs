//! Example: Create surface and wireframe surface charts (Task 34).

use zavora_xlsx::*;

fn main() -> Result<()> {
    let path = std::path::PathBuf::from("output/surface_chart_example.xlsx");

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("Surface Data")?;

    // Create a grid of data for the surface
    let hdr = Format::new().bold();
    ws.write_with_format(0, 0, "", &hdr)?;
    for col in 1..=5u16 {
        ws.write_with_format(0, col, format!("X{}", col), &hdr)?;
    }
    for row in 1..=5u32 {
        ws.write(row, 0, format!("Y{}", row))?;
        for col in 1..=5u16 {
            let val = (row as f64 * col as f64).sin() * 10.0 + 20.0;
            ws.write(row, col, val)?;
        }
    }

    // Solid surface chart
    let mut surface = Chart::new(ChartType::Surface);
    surface.set_title("3D Surface");
    for row in 1..=5u32 {
        surface
            .add_series()
            .set_name(&format!("Y{}", row))
            .set_values(&format!("'Surface Data'!$B${}:$F${}", row + 1, row + 1));
    }
    ws.insert_chart(8, 0, &surface)?;

    // Wireframe surface chart
    let mut wireframe = Chart::new(ChartType::WireframeSurface);
    wireframe.set_title("Wireframe Surface");
    for row in 1..=5u32 {
        wireframe
            .add_series()
            .set_name(&format!("Y{}", row))
            .set_values(&format!("'Surface Data'!$B${}:$F${}", row + 1, row + 1));
    }
    ws.insert_chart(8, 6, &wireframe)?;

    wb.save(&path)?;
    println!("✅ Surface charts saved to {}", path.display());
    Ok(())
}

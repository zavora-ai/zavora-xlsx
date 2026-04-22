//! Example: Create 3D chart variants with custom View3D settings (Task 33).

use zavora_xlsx::*;

fn main() -> Result<()> {
    let path = std::path::PathBuf::from("output/chart_3d_example.xlsx");

    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    ws.set_name("3D Charts")?;

    let hdr = Format::new().bold();
    ws.write_with_format(0, 0, "Quarter", &hdr)?;
    ws.write_with_format(0, 1, "Sales", &hdr)?;
    ws.write_with_format(0, 2, "Costs", &hdr)?;

    let quarters = ["Q1", "Q2", "Q3", "Q4"];
    let sales = [120.0, 180.0, 150.0, 210.0];
    let costs = [80.0, 110.0, 95.0, 130.0];
    for (i, (q, (s, c))) in quarters
        .iter()
        .zip(sales.iter().zip(costs.iter()))
        .enumerate()
    {
        let r = (i + 1) as u32;
        ws.write(r, 0, *q)?;
        ws.write(r, 1, *s)?;
        ws.write(r, 2, *c)?;
    }

    // 3D Column chart with custom perspective
    let mut col3d = Chart::new(ChartType::Column3D);
    col3d.set_title("3D Column — Sales vs Costs");
    col3d.set_view3d(View3D {
        rot_x: 20,
        rot_y: 30,
        perspective: 40,
        right_angle_axes: false,
    });
    col3d
        .add_series()
        .set_values("'3D Charts'!$B$2:$B$5")
        .set_categories("'3D Charts'!$A$2:$A$5")
        .set_name("Sales");
    col3d
        .add_series()
        .set_values("'3D Charts'!$C$2:$C$5")
        .set_categories("'3D Charts'!$A$2:$A$5")
        .set_name("Costs");
    ws.insert_chart(7, 0, &col3d)?;

    // 3D Pie chart
    let mut pie3d = Chart::new(ChartType::Pie3D);
    pie3d.set_title("3D Pie — Sales Distribution");
    pie3d
        .add_series()
        .set_values("'3D Charts'!$B$2:$B$5")
        .set_categories("'3D Charts'!$A$2:$A$5");
    ws.insert_chart(7, 6, &pie3d)?;

    wb.save(&path)?;
    println!("✅ 3D charts saved to {}", path.display());
    Ok(())
}

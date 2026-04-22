use zavora_xlsx::*;
fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let navy = "#1B2A4A";
    let green = "#0D7C3D";
    let _red = "#C00000";
    let blue = "#2B579A";
    let border = "#D6DCE4";
    let light = "#F5F7FA";
    let ws = wb.worksheet(0)?;
    ws.set_name("Inventory")?;
    ws.hide_gridlines();
    let cw = [2.0, 12.0, 22.0, 14.0, 12.0, 10.0, 10.0, 12.0, 14.0, 18.0];
    for (c, w) in cw.iter().enumerate() {
        ws.set_column_width(c as u16, *w)?;
    }
    let hdr = Format::new()
        .bold()
        .font_size(9.0)
        .font_color("#FFFFFF")
        .background_color(navy)
        .align(Align::Center)
        .border(BorderStyle::Thin);
    let hl = Format::new()
        .bold()
        .font_size(9.0)
        .font_color("#FFFFFF")
        .background_color(navy)
        .align(Align::Left)
        .border(BorderStyle::Thin);
    let tf = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Left)
        .border(BorderStyle::Thin)
        .border_color(border);
    let nf = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .border(BorderStyle::Thin)
        .border_color(border);
    let mf = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Right)
        .num_format("$#,##0")
        .border(BorderStyle::Thin)
        .border_color(border);
    let title = Format::new()
        .bold()
        .font_size(20.0)
        .font_color(navy)
        .align(Align::Left)
        .align(Align::Bottom);
    let mut r = 0u32;
    ws.set_row_height(r, 6.0)?;
    r += 1;
    ws.write_with_format(r, 1, "📦 Inventory Tracker", &title)?;
    ws.set_row_height(r, 32.0)?;
    r += 1;
    ws.write_with_format(
        r,
        1,
        "Stock levels & reorder alerts  |  Formulas auto-calculate",
        &Format::new().font_size(10.0).font_color("#667085").italic(),
    )?;
    r += 1;
    for c in 1..=9u16 {
        ws.write_with_format(r, c, "", &Format::new().background_color(blue))?;
    }
    ws.set_row_height(r, 3.0)?;
    r += 2;
    let h = [
        "SKU",
        "Product",
        "Category",
        "Unit Cost",
        "Qty",
        "Reorder Lvl",
        "Stock Value",
        "Status",
        "Supplier",
    ];
    ws.write_with_format(r, 1, h[0], &hdr)?;
    ws.write_with_format(r, 2, h[1], &hl)?;
    for (c, hh) in h[2..].iter().enumerate() {
        ws.write_with_format(r, (c + 3) as u16, *hh, &hdr)?;
    }
    ws.set_row_height(r, 22.0)?;
    r += 1;
    let items: Vec<(&str, &str, &str, f64, f64, f64, &str)> = vec![
        (
            "SKU-001",
            "Wireless Mouse",
            "Electronics",
            25.0,
            150.0,
            50.0,
            "TechSupply Co",
        ),
        (
            "SKU-002",
            "USB-C Cable",
            "Electronics",
            8.0,
            320.0,
            100.0,
            "CablePro Ltd",
        ),
        (
            "SKU-003",
            "Office Chair",
            "Furniture",
            180.0,
            25.0,
            10.0,
            "FurniCraft",
        ),
        (
            "SKU-004",
            "Standing Desk",
            "Furniture",
            450.0,
            12.0,
            5.0,
            "FurniCraft",
        ),
        (
            "SKU-005",
            "Monitor 27\"",
            "Electronics",
            350.0,
            45.0,
            20.0,
            "TechSupply Co",
        ),
        (
            "SKU-006",
            "Keyboard",
            "Electronics",
            35.0,
            200.0,
            75.0,
            "TechSupply Co",
        ),
        (
            "SKU-007",
            "Printer Paper",
            "Supplies",
            12.0,
            80.0,
            100.0,
            "OfficeMart",
        ),
        (
            "SKU-008",
            "Ink Cartridge",
            "Supplies",
            45.0,
            30.0,
            25.0,
            "OfficeMart",
        ),
        (
            "SKU-009",
            "Webcam HD",
            "Electronics",
            65.0,
            60.0,
            30.0,
            "TechSupply Co",
        ),
        (
            "SKU-010",
            "Headset",
            "Electronics",
            55.0,
            90.0,
            40.0,
            "AudioTech",
        ),
        (
            "SKU-011",
            "Whiteboard",
            "Office",
            85.0,
            15.0,
            8.0,
            "OfficeMart",
        ),
        (
            "SKU-012",
            "Desk Lamp",
            "Office",
            40.0,
            42.0,
            20.0,
            "LightWorks",
        ),
        (
            "SKU-013",
            "Filing Cabinet",
            "Furniture",
            220.0,
            8.0,
            5.0,
            "FurniCraft",
        ),
        (
            "SKU-014",
            "Notebook Pack",
            "Supplies",
            6.0,
            500.0,
            200.0,
            "OfficeMart",
        ),
        (
            "SKU-015",
            "Projector",
            "Electronics",
            680.0,
            5.0,
            3.0,
            "TechSupply Co",
        ),
        (
            "SKU-016",
            "Extension Cord",
            "Electronics",
            15.0,
            120.0,
            50.0,
            "PowerPlus",
        ),
        (
            "SKU-017",
            "Hand Sanitizer",
            "Supplies",
            4.0,
            200.0,
            100.0,
            "CleanCo",
        ),
        (
            "SKU-018",
            "First Aid Kit",
            "Safety",
            35.0,
            10.0,
            5.0,
            "SafetyFirst",
        ),
        (
            "SKU-019",
            "Fire Extinguisher",
            "Safety",
            75.0,
            6.0,
            4.0,
            "SafetyFirst",
        ),
        (
            "SKU-020",
            "Surge Protector",
            "Electronics",
            28.0,
            55.0,
            25.0,
            "PowerPlus",
        ),
    ];
    let ds = r;
    for (i, (sku, name, cat, cost, qty, reorder, supplier)) in items.iter().enumerate() {
        let alt = i % 2 == 1;
        let t = if alt {
            &Format::new()
                .font_size(10.0)
                .font_color(navy)
                .align(Align::Left)
                .background_color(light)
                .border(BorderStyle::Thin)
                .border_color(border)
        } else {
            &tf
        };
        let n = if alt {
            &Format::new()
                .font_size(10.0)
                .font_color(navy)
                .align(Align::Center)
                .background_color(light)
                .border(BorderStyle::Thin)
                .border_color(border)
        } else {
            &nf
        };
        ws.write_with_format(r, 1, *sku, n)?;
        ws.write_with_format(r, 2, *name, t)?;
        ws.write_with_format(r, 3, *cat, t)?;
        ws.write_with_format(r, 4, *cost, &mf)?;
        ws.write_with_format(r, 5, *qty, n)?;
        ws.write_with_format(r, 6, *reorder, n)?;
        ws.write_formula(r, 7, &format!("E{}*F{}", r + 1, r + 1))?;
        ws.set_cell_format(r, 7, &mf)?;
        ws.write_formula(
            r,
            8,
            &format!(
                "IF(F{r1}<=G{r1},\"⚠ Reorder\",IF(F{r1}<=G{r1}*1.5,\"Low\",\"✅ OK\"))",
                r1 = r + 1
            ),
        )?;
        ws.set_cell_format(r, 8, &nf)?;
        ws.write_with_format(r, 9, *supplier, t)?;
        r += 1;
    }
    let de = r - 1;
    r += 1;
    ws.write_with_format(
        r,
        6,
        "Total Value:",
        &Format::new()
            .bold()
            .font_size(10.0)
            .font_color(navy)
            .align(Align::Right),
    )?;
    ws.write_formula(r, 7, &format!("SUM(H{}:H{})", ds + 1, de + 1))?;
    ws.set_cell_format(
        r,
        7,
        &Format::new()
            .bold()
            .font_size(12.0)
            .font_color(green)
            .align(Align::Center)
            .num_format("$#,##0"),
    )?;
    ws.add_conditional_format(ds, 5, de, 5, ConditionalFormatDataBar::new(blue))?;
    ws.set_autofilter(5, 1, de, 9);
    ws.set_freeze_panes(6, 0)?;
    ws.set_landscape();
    ws.set_fit_to_page(1, 0);
    ws.set_header("&CInventory Tracker");
    ws.set_footer("&C&D  |  Page &P");
    wb.save(home("inventory_tracker.xlsx"))?;
    println!("✅ Inventory Tracker saved");
    Ok(())
}
fn home(n: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("output").join(n)
}

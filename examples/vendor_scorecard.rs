use zavora_xlsx::*;
fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let navy = "#1B2A4A";
    let green = "#0D7C3D";
    let _blue = "#2B579A";
    let border = "#D6DCE4";
    let light = "#F5F7FA";
    let ws = wb.worksheet(0)?;
    ws.set_name("Vendor Scorecard")?;
    ws.hide_gridlines();
    let cw = [2.0, 20.0, 14.0, 12.0, 12.0, 12.0, 12.0, 12.0, 14.0];
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
    let sf = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .num_format("0.0")
        .border(BorderStyle::Thin)
        .border_color(border);
    let cf = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
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
    ws.write_with_format(r, 1, "🏆 Vendor Scorecard", &title)?;
    ws.set_row_height(r, 32.0)?;
    r += 1;
    ws.write_with_format(
        r,
        1,
        "Supplier Performance — Q1 2025",
        &Format::new().font_size(10.0).font_color("#667085").italic(),
    )?;
    r += 1;
    for c in 1..=8u16 {
        ws.write_with_format(r, c, "", &Format::new().background_color(green))?;
    }
    ws.set_row_height(r, 3.0)?;
    r += 2;
    let h = [
        "Vendor",
        "Category",
        "Quality",
        "Delivery",
        "Price",
        "Communication",
        "Overall",
        "Rating",
    ];
    ws.write_with_format(r, 1, h[0], &hl)?;
    ws.write_with_format(r, 2, h[1], &hl)?;
    for (c, hh) in h[2..].iter().enumerate() {
        ws.write_with_format(r, (c + 3) as u16, *hh, &hdr)?;
    }
    ws.set_row_height(r, 22.0)?;
    r += 1;
    let vendors: Vec<(&str, &str, f64, f64, f64, f64)> = vec![
        ("TechSupply Co", "Electronics", 4.8, 4.5, 4.0, 4.6),
        ("FurniCraft", "Furniture", 4.2, 3.8, 4.5, 4.0),
        ("OfficeMart", "Supplies", 4.0, 4.2, 4.8, 3.5),
        ("AudioTech", "Electronics", 4.5, 4.0, 3.8, 4.2),
        ("LightWorks", "Office", 3.5, 4.5, 4.2, 3.8),
        ("PowerPlus", "Electronics", 4.3, 3.5, 4.0, 4.1),
        ("CleanCo", "Supplies", 3.8, 4.0, 4.5, 3.2),
        ("SafetyFirst", "Safety", 4.6, 4.8, 3.5, 4.5),
        ("CablePro Ltd", "Electronics", 4.1, 3.2, 4.8, 3.0),
        ("EcoSupply", "Supplies", 3.0, 3.5, 4.0, 3.8),
    ];
    let ds = r;
    for (i, (name, cat, q, d, p, c_score)) in vendors.iter().enumerate() {
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
        ws.write_with_format(r, 1, *name, t)?;
        ws.write_with_format(r, 2, *cat, t)?;
        ws.write_with_format(r, 3, *q, &sf)?;
        ws.write_with_format(r, 4, *d, &sf)?;
        ws.write_with_format(r, 5, *p, &sf)?;
        ws.write_with_format(r, 6, *c_score, &sf)?;
        ws.write_formula(r, 7, &format!("AVERAGE(D{}:G{})", r + 1, r + 1))?;
        ws.set_cell_format(
            r,
            7,
            &Format::new()
                .bold()
                .font_size(10.0)
                .font_color(navy)
                .align(Align::Center)
                .num_format("0.0")
                .border(BorderStyle::Thin)
                .border_color(border),
        )?;
        ws.write_formula(r,8,&format!("IF(H{r1}>=4.5,\"⭐ Preferred\",IF(H{r1}>=4,\"✅ Approved\",IF(H{r1}>=3.5,\"⚠ Probation\",\"❌ Review\")))",r1=r+1))?;
        ws.set_cell_format(r, 8, &cf)?;
        r += 1;
    }
    let de = r - 1;
    ws.add_conditional_format(ds, 7, de, 7, ConditionalFormatDataBar::new(green))?;
    ws.set_autofilter(5, 1, de, 8);
    ws.set_freeze_panes(6, 0)?;
    ws.set_landscape();
    ws.set_fit_to_page(1, 1);
    ws.set_header("&CVendor Scorecard");
    wb.save(home("vendor_scorecard.xlsx"))?;
    println!("✅ Vendor Scorecard saved");
    Ok(())
}
fn home(n: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("output").join(n)
}

use zavora_xlsx::*;

fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let navy = "#1B2A4A"; let blue = "#2B579A"; let green = "#0D7C3D";
    let red = "#C00000"; let border = "#D6DCE4"; let light = "#F5F7FA";

    let ws = wb.worksheet(0)?;
    ws.set_name("Balance Sheet")?;
    ws.hide_gridlines();

    let cw = [2.0, 32.0, 16.0, 16.0, 16.0, 12.0];
    for (c, w) in cw.iter().enumerate() { ws.set_column_width(c as u16, *w)?; }

    let title = Format::new().bold().font_size(20.0).font_color(navy).align(Align::Left).align(Align::Bottom);
    let sub = Format::new().font_size(10.0).font_color("#667085").italic();
    let divider = Format::new().background_color(blue);
    let hdr = Format::new().bold().font_size(10.0).font_color("#FFFFFF").background_color(navy).align(Align::Center).border(BorderStyle::Thin);
    let hdr_l = Format::new().bold().font_size(10.0).font_color("#FFFFFF").background_color(navy).align(Align::Left).border(BorderStyle::Thin);
    let item = Format::new().font_size(10.0).font_color(navy).align(Align::Left).border(BorderStyle::Thin).border_color(border);
    let val = Format::new().font_size(10.0).font_color(navy).align(Align::Right).num_format("#,##0").border(BorderStyle::Thin).border_color(border);
    let val_a = Format::new().font_size(10.0).font_color(navy).align(Align::Right).num_format("#,##0").background_color(light).border(BorderStyle::Thin).border_color(border);
    let chg_pos = Format::new().font_size(10.0).font_color(green).bold().align(Align::Right).num_format("#,##0").border(BorderStyle::Thin).border_color(border);
    let chg_neg = Format::new().font_size(10.0).font_color(red).bold().align(Align::Right).num_format("(#,##0)").border(BorderStyle::Thin).border_color(border);
    let pct_fmt = Format::new().font_size(10.0).font_color(navy).align(Align::Right).num_format("0.0%").border(BorderStyle::Thin).border_color(border);
    let stl = Format::new().bold().font_size(10.0).font_color(navy).align(Align::Left).background_color("#E8E8E8").border(BorderStyle::Thin).border_color(border);
    let stv = Format::new().bold().font_size(10.0).font_color(navy).align(Align::Right).num_format("#,##0").background_color("#E8E8E8").border(BorderStyle::Thin).border_color(border);

    macro_rules! wl {
        ($r:ident, $name:expr, $cy:expr, $py:expr, $alt:expr) => {{
            let vf = if $alt { &val_a } else { &val };
            ws.write_with_format($r, 1, $name, &item).unwrap();
            ws.write_with_format($r, 2, $cy, vf).unwrap();
            ws.write_with_format($r, 3, $py, vf).unwrap();
            ws.write_formula($r, 4, &format!("C{}-D{}", $r+1, $r+1)).unwrap();
            ws.set_cell_format($r, 4, if $cy >= $py { &chg_pos } else { &chg_neg }).unwrap();
            ws.write_formula($r, 5, &format!("IFERROR(E{}/D{},0)", $r+1, $r+1)).unwrap();
            ws.set_cell_format($r, 5, &pct_fmt).unwrap();
            $r += 1;
        }};
    }
    macro_rules! sec {
        ($r:ident, $name:expr, $color:expr) => {{
            let sf = Format::new().bold().font_size(11.0).font_color("#FFFFFF").background_color($color).align(Align::Left).border(BorderStyle::Thin);
            let sv = Format::new().bold().background_color($color).border(BorderStyle::Thin);
            ws.write_with_format($r, 1, $name, &sf).unwrap();
            for c in 2..=5u16 { ws.write_with_format($r, c, "", &sv).unwrap(); }
            ws.set_row_height($r, 24.0).unwrap(); $r += 1;
        }};
    }
    macro_rules! subtotal {
        ($r:ident, $name:expr, $start:expr, $end:expr) => {{
            ws.write_with_format($r, 1, $name, &stl).unwrap();
            for c in 2..=3u16 { ws.write_formula($r, c, &format!("SUM({}{}:{}{})", col(c), $start+1, col(c), $end+1)).unwrap(); ws.set_cell_format($r, c, &stv).unwrap(); }
            ws.write_formula($r, 4, &format!("C{}-D{}", $r+1, $r+1)).unwrap(); ws.set_cell_format($r, 4, &stv).unwrap();
            ws.write_formula($r, 5, &format!("IFERROR(E{}/D{},0)", $r+1, $r+1)).unwrap(); ws.set_cell_format($r, 5, &pct_fmt).unwrap();
        }};
    }
    macro_rules! grand {
        ($r:ident, $name:expr, $color:expr, $r1:expr, $r2:expr) => {{
            let gl = Format::new().bold().font_size(11.0).font_color("#FFFFFF").background_color($color).align(Align::Left).border(BorderStyle::Medium);
            let gv = Format::new().bold().font_size(11.0).font_color("#FFFFFF").num_format("#,##0").background_color($color).align(Align::Right).border(BorderStyle::Medium);
            ws.write_with_format($r, 1, $name, &gl).unwrap();
            for c in 2..=3u16 { ws.write_formula($r, c, &format!("{}{}+{}{}", col(c), $r1+1, col(c), $r2+1)).unwrap(); ws.set_cell_format($r, c, &gv).unwrap(); }
            ws.write_formula($r, 4, &format!("C{}-D{}", $r+1, $r+1)).unwrap(); ws.set_cell_format($r, 4, &gv).unwrap();
            ws.write_formula($r, 5, &format!("IFERROR(E{}/D{},0)", $r+1, $r+1)).unwrap(); ws.set_cell_format($r, 5, &pct_fmt).unwrap();
            ws.set_row_height($r, 28.0).unwrap();
        }};
    }

    let mut r = 0u32;
    ws.set_row_height(r, 6.0)?; r += 1;
    ws.write_with_format(r, 1, "Balance Sheet", &title)?; ws.set_row_height(r, 32.0)?; r += 1;
    ws.write_with_format(r, 1, "As of December 31, 2025  |  Comparative with Prior Year", &sub)?; r += 1;
    for c in 1..=5u16 { ws.write_with_format(r, c, "", &divider)?; } ws.set_row_height(r, 3.0)?; r += 2;

    ws.write_with_format(r, 1, "Account", &hdr_l)?;
    ws.write_with_format(r, 2, "FY 2025", &hdr)?; ws.write_with_format(r, 3, "FY 2024", &hdr)?;
    ws.write_with_format(r, 4, "Change", &hdr)?; ws.write_with_format(r, 5, "% Change", &hdr)?;
    ws.set_row_height(r, 22.0)?; r += 1;

    // ASSETS
    sec!(r, "ASSETS", blue);
    let ca_s = r;
    wl!(r, "  Cash & Equivalents", 485000.0, 320000.0, false);
    wl!(r, "  Accounts Receivable", 215000.0, 185000.0, true);
    wl!(r, "  Inventory", 142000.0, 128000.0, false);
    wl!(r, "  Prepaid Expenses", 28000.0, 22000.0, true);
    wl!(r, "  Short-term Investments", 150000.0, 100000.0, false);
    let ca_e = r - 1; subtotal!(r, "Total Current Assets", ca_s, ca_e); let tca = r; r += 1;

    let nca_s = r;
    wl!(r, "  Property & Equipment", 890000.0, 820000.0, false);
    wl!(r, "  Less: Accum. Depreciation", -180000.0, -150000.0, true);
    wl!(r, "  Intangible Assets", 120000.0, 95000.0, false);
    wl!(r, "  Long-term Investments", 340000.0, 280000.0, true);
    let nca_e = r - 1; subtotal!(r, "Total Non-Current Assets", nca_s, nca_e); let tnca = r; r += 1;

    grand!(r, "TOTAL ASSETS", blue, tca, tnca); let ta = r; r += 2;

    // LIABILITIES
    sec!(r, "LIABILITIES", red);
    let cl_s = r;
    wl!(r, "  Accounts Payable", 95000.0, 82000.0, false);
    wl!(r, "  Accrued Expenses", 45000.0, 38000.0, true);
    wl!(r, "  Short-term Debt", 50000.0, 75000.0, false);
    wl!(r, "  Current Portion LT Debt", 30000.0, 30000.0, true);
    let cl_e = r - 1; subtotal!(r, "Total Current Liabilities", cl_s, cl_e); let tcl = r; r += 1;

    let ncl_s = r;
    wl!(r, "  Long-term Debt", 280000.0, 310000.0, false);
    wl!(r, "  Deferred Tax Liabilities", 35000.0, 30000.0, true);
    let ncl_e = r - 1; subtotal!(r, "Total Non-Current Liabilities", ncl_s, ncl_e); let tncl = r; r += 1;

    grand!(r, "TOTAL LIABILITIES", red, tcl, tncl); let tl = r; r += 2;

    // EQUITY
    sec!(r, "EQUITY", green);
    let eq_s = r;
    wl!(r, "  Common Stock", 500000.0, 500000.0, false);
    wl!(r, "  Retained Earnings", 1035000.0, 838000.0, true);
    wl!(r, "  Other Comprehensive Income", 15000.0, 12000.0, false);
    let eq_e = r - 1;
    let te = r;
    let gl = Format::new().bold().font_size(11.0).font_color("#FFFFFF").background_color(green).align(Align::Left).border(BorderStyle::Medium);
    let gv = Format::new().bold().font_size(11.0).font_color("#FFFFFF").num_format("#,##0").background_color(green).align(Align::Right).border(BorderStyle::Medium);
    ws.write_with_format(r, 1, "TOTAL EQUITY", &gl)?;
    for c in 2..=3u16 { ws.write_formula(r, c, &format!("SUM({}{}:{}{})", col(c), eq_s+1, col(c), eq_e+1))?; ws.set_cell_format(r, c, &gv)?; }
    ws.write_formula(r, 4, &format!("C{}-D{}", r+1, r+1))?; ws.set_cell_format(r, 4, &gv)?;
    ws.write_formula(r, 5, &format!("IFERROR(E{}/D{},0)", r+1, r+1))?; ws.set_cell_format(r, 5, &pct_fmt)?;
    ws.set_row_height(r, 28.0)?; r += 2;

    // TOTAL L+E
    let tle = r;
    let cl = Format::new().bold().font_size(11.0).font_color("#FFFFFF").background_color(navy).align(Align::Left).border(BorderStyle::Double).border_color(navy);
    let cv = Format::new().bold().font_size(11.0).font_color("#FFFFFF").num_format("#,##0").background_color(navy).align(Align::Right).border(BorderStyle::Double).border_color(navy);
    ws.write_with_format(r, 1, "TOTAL LIABILITIES + EQUITY", &cl)?;
    for c in 2..=3u16 { ws.write_formula(r, c, &format!("{}{}+{}{}", col(c), tl+1, col(c), te+1))?; ws.set_cell_format(r, c, &cv)?; }
    ws.write_formula(r, 4, &format!("C{}-D{}", r+1, r+1))?; ws.set_cell_format(r, 4, &cv)?;
    ws.write_formula(r, 5, &format!("IFERROR(E{}/D{},0)", r+1, r+1))?; ws.set_cell_format(r, 5, &pct_fmt)?;
    ws.set_row_height(r, 28.0)?; r += 1;

    // Balance check
    ws.write_with_format(r, 1, "Balance Check", &Format::new().italic().font_size(10.0).font_color("#667085"))?;
    ws.write_formula(r, 2, &format!("IF(C{}=C{},\"✅ Balanced\",\"❌ Out of Balance\")", ta+1, tle+1))?;
    ws.set_cell_format(r, 2, &Format::new().bold().font_size(11.0).font_color(green).align(Align::Center))?;
    r += 2;

    // Chart
    let src = r + 15;
    ws.write(src, 1, "Assets")?; ws.write_formula(src, 2, &format!("C{}", ta+1))?;
    ws.write(src+1, 1, "Liabilities")?; ws.write_formula(src+1, 2, &format!("C{}", tl+1))?;
    ws.write(src+2, 1, "Equity")?; ws.write_formula(src+2, 2, &format!("C{}", te+1))?;

    let mut chart = Chart::new(ChartType::Bar);
    chart.set_title("Assets vs Liabilities + Equity");
    chart.set_width(480); chart.set_height(260);
    chart.set_legend_position(LegendPosition::None);
    let s = chart.add_series();
    s.set_values(&format!("'Balance Sheet'!$C${}:$C${}", src+1, src+3));
    s.set_categories(&format!("'Balance Sheet'!$B${}:$B${}", src+1, src+3));
    s.set_data_labels(true);
    s.set_point_color(0, blue); s.set_point_color(1, red); s.set_point_color(2, green);
    ws.insert_chart(r, 1, &chart)?;

    ws.set_freeze_panes(6, 0)?;
    ws.set_portrait(); ws.set_fit_to_page(1, 1);
    ws.set_header("&CBalance Sheet — December 31, 2025");
    ws.set_footer("&CConfidential  |  &D");

    let path = std::path::PathBuf::from(std::env::var("HOME").unwrap_or("/tmp".into())).join("Downloads/balance_sheet.xlsx");
    wb.save(&path)?;
    println!("✅ Balance Sheet saved to {}", path.display());
    Ok(())
}

fn col(c: u16) -> String { zavora_xlsx::utility::col_to_letter(c) }

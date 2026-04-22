use zavora_xlsx::*;

fn main() -> Result<()> {
    // ── Generate Invoice ──
    let inv = Invoice {
        number: "INV-2025-0042",
        date: (2025, 4, 6),
        due_date: (2025, 5, 6),
        company: Company {
            name: "Zavora Consulting Ltd.",
            address: "123 Innovation Drive\nNairobi, Kenya 00100",
            email: "billing@zavora.co.ke",
            phone: "+254 700 123 456",
        },
        client: Company {
            name: "Acme Corporation",
            address: "456 Business Park\nMombasa, Kenya 80100",
            email: "accounts@acme.co.ke",
            phone: "+254 711 987 654",
        },
        items: vec![
            LineItem {
                description: "Financial Model Development",
                qty: 40.0,
                unit: "hours",
                rate: 150.00,
            },
            LineItem {
                description: "Excel Dashboard Design",
                qty: 24.0,
                unit: "hours",
                rate: 125.00,
            },
            LineItem {
                description: "Data Migration & Cleanup",
                qty: 16.0,
                unit: "hours",
                rate: 100.00,
            },
            LineItem {
                description: "Training Workshop (2 sessions)",
                qty: 2.0,
                unit: "sessions",
                rate: 500.00,
            },
            LineItem {
                description: "Monthly Support Retainer",
                qty: 1.0,
                unit: "month",
                rate: 2000.00,
            },
        ],
        tax_rate: 0.16,
        notes: "Payment via M-Pesa or bank transfer.\nLate payments subject to 2% monthly interest.",
        terms: "Net 30 days",
    };
    generate_invoice(&inv, "invoice")?;

    // ── Generate Quotation ──
    let quote = Invoice {
        number: "QUO-2025-0018",
        date: (2025, 4, 6),
        due_date: (2025, 4, 20),
        company: inv.company,
        client: Company {
            name: "Global Traders Inc.",
            address: "789 Commerce Avenue\nKisumu, Kenya 40100",
            email: "procurement@globaltraders.co.ke",
            phone: "+254 722 555 888",
        },
        items: vec![
            LineItem {
                description: "ERP System Integration",
                qty: 80.0,
                unit: "hours",
                rate: 175.00,
            },
            LineItem {
                description: "Custom Report Templates (10)",
                qty: 10.0,
                unit: "reports",
                rate: 350.00,
            },
            LineItem {
                description: "API Development & Testing",
                qty: 60.0,
                unit: "hours",
                rate: 200.00,
            },
            LineItem {
                description: "User Acceptance Testing",
                qty: 20.0,
                unit: "hours",
                rate: 125.00,
            },
            LineItem {
                description: "Go-Live Support (1 week)",
                qty: 1.0,
                unit: "week",
                rate: 5000.00,
            },
            LineItem {
                description: "Documentation & Handover",
                qty: 1.0,
                unit: "package",
                rate: 3000.00,
            },
        ],
        tax_rate: 0.16,
        notes: "Quote valid for 14 days.\nPrices in USD. 50% deposit required to commence.",
        terms: "50% upfront, 50% on completion",
    };
    generate_invoice(&quote, "quotation")?;

    Ok(())
}

#[derive(Clone, Copy)]
struct Company {
    name: &'static str,
    address: &'static str,
    email: &'static str,
    phone: &'static str,
}

struct LineItem {
    description: &'static str,
    qty: f64,
    unit: &'static str,
    rate: f64,
}

struct Invoice {
    number: &'static str,
    date: (i32, u32, u32),
    due_date: (i32, u32, u32),
    company: Company,
    client: Company,
    items: Vec<LineItem>,
    tax_rate: f64,
    notes: &'static str,
    terms: &'static str,
}

fn generate_invoice(inv: &Invoice, doc_type: &str) -> Result<()> {
    let mut wb = Workbook::new();
    let ws = wb.worksheet(0)?;
    let label = if doc_type == "invoice" {
        "INVOICE"
    } else {
        "QUOTATION"
    };
    ws.set_name(label)?;

    // ── Colors ──
    let navy = "#1B2A4A";
    let accent = "#2B579A";
    let light_bg = "#F5F7FA";
    let border_clr = "#D0D5DD";
    let green = "#0D7C3D";
    let light_green = "#E8F5E9";

    // ── Column widths ──
    let widths = [2.0, 36.0, 10.0, 10.0, 14.0, 16.0, 2.0]; // margin, desc, qty, unit, rate, amount, margin
    for (c, w) in widths.iter().enumerate() {
        ws.set_column_width(c as u16, *w)?;
    }

    // ── Styles ──
    let _spacer = Format::new().font_size(4.0);
    let doc_title = Format::new()
        .bold()
        .font_size(28.0)
        .font_color(navy)
        .align(Align::Left)
        .align(Align::Bottom);
    let doc_number = Format::new()
        .font_size(12.0)
        .font_color(accent)
        .align(Align::Right)
        .align(Align::Bottom);
    let section_label = Format::new()
        .bold()
        .font_size(9.0)
        .font_color("#667085")
        .align(Align::Left);
    let company_name = Format::new().bold().font_size(11.0).font_color(navy);
    let company_detail = Format::new().font_size(10.0).font_color("#344054");
    let field_label = Format::new()
        .bold()
        .font_size(10.0)
        .font_color("#667085")
        .align(Align::Left);
    let field_value = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Left);
    let date_value = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .num_format("mmmm d, yyyy")
        .align(Align::Left);

    let col_header = Format::new()
        .bold()
        .font_size(10.0)
        .font_color("#FFFFFF")
        .background_color(accent)
        .align(Align::Center)
        .align(Align::VerticalCenter)
        .border(BorderStyle::Thin)
        .border_color(accent);
    let col_header_left = Format::new()
        .bold()
        .font_size(10.0)
        .font_color("#FFFFFF")
        .background_color(accent)
        .align(Align::Left)
        .align(Align::VerticalCenter)
        .border(BorderStyle::Thin)
        .border_color(accent);

    let item_desc = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Left)
        .align(Align::VerticalCenter)
        .border(BorderStyle::Thin)
        .border_color(border_clr);
    let item_num = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .align(Align::VerticalCenter)
        .border(BorderStyle::Thin)
        .border_color(border_clr);
    let item_currency = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Right)
        .align(Align::VerticalCenter)
        .num_format("#,##0.00")
        .border(BorderStyle::Thin)
        .border_color(border_clr);
    let item_alt_desc = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Left)
        .align(Align::VerticalCenter)
        .background_color(light_bg)
        .border(BorderStyle::Thin)
        .border_color(border_clr);
    let item_alt_num = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Center)
        .align(Align::VerticalCenter)
        .background_color(light_bg)
        .border(BorderStyle::Thin)
        .border_color(border_clr);
    let item_alt_currency = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Right)
        .align(Align::VerticalCenter)
        .background_color(light_bg)
        .num_format("#,##0.00")
        .border(BorderStyle::Thin)
        .border_color(border_clr);

    let total_label = Format::new()
        .bold()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Right);
    let total_value = Format::new()
        .font_size(10.0)
        .font_color(navy)
        .align(Align::Right)
        .num_format("#,##0.00")
        .border(BorderStyle::Thin)
        .border_color(border_clr);
    let grand_total_label = Format::new()
        .bold()
        .font_size(13.0)
        .font_color(green)
        .align(Align::Right);
    let grand_total_value = Format::new()
        .bold()
        .font_size(13.0)
        .font_color(green)
        .align(Align::Right)
        .num_format("#,##0.00")
        .background_color(light_green)
        .border(BorderStyle::Medium)
        .border_color(green);

    let notes_label = Format::new().bold().font_size(9.0).font_color("#667085");
    let notes_text = Format::new().font_size(9.0).font_color("#344054");
    let footer_text = Format::new()
        .font_size(8.0)
        .font_color("#98A2B3")
        .italic()
        .align(Align::Center);

    let divider = Format::new().background_color(accent);

    // ── Layout ──
    let mut r = 0u32;

    // Top margin
    ws.set_row_height(r, 8.0)?;
    r += 1;

    // Title row
    ws.write_with_format(r, 1, label, &doc_title)?;
    ws.write_with_format(r, 5, inv.number, &doc_number)?;
    ws.set_row_height(r, 40.0)?;
    r += 1;

    // Divider
    for c in 1..=5u16 {
        ws.write_with_format(r, c, "", &divider)?;
    }
    ws.set_row_height(r, 3.0)?;
    r += 1;

    // Spacer
    ws.set_row_height(r, 12.0)?;
    r += 1;

    // From / To section
    ws.write_with_format(r, 1, "FROM", &section_label)?;
    ws.write_with_format(r, 4, "BILL TO", &section_label)?;
    r += 1;

    ws.write_with_format(r, 1, inv.company.name, &company_name)?;
    ws.write_with_format(r, 4, inv.client.name, &company_name)?;
    r += 1;

    // Address lines
    for (from_line, to_line) in inv
        .company
        .address
        .split('\n')
        .zip(inv.client.address.split('\n'))
    {
        ws.write_with_format(r, 1, from_line, &company_detail)?;
        ws.write_with_format(r, 4, to_line, &company_detail)?;
        r += 1;
    }
    ws.write_with_format(r, 1, inv.company.email, &company_detail)?;
    ws.write_with_format(r, 4, inv.client.email, &company_detail)?;
    r += 1;
    ws.write_with_format(r, 1, inv.company.phone, &company_detail)?;
    ws.write_with_format(r, 4, inv.client.phone, &company_detail)?;
    r += 1;

    // Spacer
    ws.set_row_height(r, 12.0)?;
    r += 1;

    // Date fields
    ws.write_with_format(r, 1, "Date:", &field_label)?;
    ws.write_with_format(
        r,
        2,
        ExcelDateTime::from_ymd(inv.date.0, inv.date.1, inv.date.2).unwrap(),
        &date_value,
    )?;
    ws.write_with_format(
        r,
        4,
        if doc_type == "invoice" {
            "Due Date:"
        } else {
            "Valid Until:"
        },
        &field_label,
    )?;
    ws.write_with_format(
        r,
        5,
        ExcelDateTime::from_ymd(inv.due_date.0, inv.due_date.1, inv.due_date.2).unwrap(),
        &date_value,
    )?;
    r += 1;

    ws.write_with_format(r, 1, "Terms:", &field_label)?;
    ws.write_with_format(r, 2, inv.terms, &field_value)?;
    r += 1;

    // Spacer
    ws.set_row_height(r, 12.0)?;
    r += 1;

    // ── Line Items Table ──
    let headers = ["Description", "Qty", "Unit", "Rate", "Amount"];
    let header_fmts = [
        &col_header_left,
        &col_header,
        &col_header,
        &col_header,
        &col_header,
    ];
    for (c, (h, f)) in headers.iter().zip(header_fmts.iter()).enumerate() {
        ws.write_with_format(r, (c + 1) as u16, *h, f)?;
    }
    ws.set_row_height(r, 26.0)?;
    r += 1;

    let items_start = r;
    for (i, item) in inv.items.iter().enumerate() {
        let alt = i % 2 == 1;
        let (d, n, c) = if alt {
            (&item_alt_desc, &item_alt_num, &item_alt_currency)
        } else {
            (&item_desc, &item_num, &item_currency)
        };
        ws.write_with_format(r, 1, item.description, d)?;
        ws.write_with_format(r, 2, item.qty, n)?;
        ws.write_with_format(r, 3, item.unit, n)?;
        ws.write_with_format(r, 4, item.rate, c)?;
        // Amount formula
        let formula = format!("C{}*E{}", r + 1, r + 1);
        ws.write_formula(r, 5, &formula)?;
        ws.set_cell_format(r, 5, c)?;
        ws.set_row_height(r, 24.0)?;
        r += 1;
    }
    let items_end = r - 1;

    // Spacer
    ws.set_row_height(r, 6.0)?;
    r += 1;

    // ── Totals ──
    let subtotal_formula = format!("SUM(F{}:F{})", items_start + 1, items_end + 1);
    ws.write_with_format(r, 4, "Subtotal", &total_label)?;
    ws.write_formula(r, 5, &subtotal_formula)?;
    ws.set_cell_format(r, 5, &total_value)?;
    r += 1;

    let tax_pct = format!("Tax ({:.0}%)", inv.tax_rate * 100.0);
    let tax_formula = format!("F{}*{}", r, inv.tax_rate);
    ws.write_with_format(r, 4, tax_pct.as_str(), &total_label)?;
    ws.write_formula(r, 5, &tax_formula)?;
    ws.set_cell_format(r, 5, &total_value)?;
    r += 1;

    // Spacer
    ws.set_row_height(r, 4.0)?;
    r += 1;

    // Grand total
    let grand_formula = format!("F{}+F{}", r - 2, r - 1);
    let total_label_text = if doc_type == "invoice" {
        "TOTAL DUE"
    } else {
        "QUOTE TOTAL"
    };
    ws.write_with_format(r, 4, total_label_text, &grand_total_label)?;
    ws.write_formula(r, 5, &grand_formula)?;
    ws.set_cell_format(r, 5, &grand_total_value)?;
    ws.set_row_height(r, 30.0)?;
    r += 2;

    // ── Notes ──
    ws.write_with_format(r, 1, "Notes:", &notes_label)?;
    r += 1;
    for line in inv.notes.split('\n') {
        ws.write_with_format(r, 1, line, &notes_text)?;
        r += 1;
    }

    r += 1;

    // Footer divider
    for c in 1..=5u16 {
        ws.write_with_format(r, c, "", &divider)?;
    }
    ws.set_row_height(r, 2.0)?;
    r += 1;

    // Footer
    ws.merge_range(r, 1, r, 5, "Thank you for your business!", &footer_text)?;
    r += 1;
    let footer_line = format!(
        "{} | {} | {}",
        inv.company.name, inv.company.email, inv.company.phone
    );
    ws.merge_range(r, 1, r, 5, footer_line.as_str(), &footer_text)?;

    // ── Print Setup ──
    ws.set_portrait();
    ws.set_paper_size(1);
    ws.set_margins(0.5, 0.5, 0.4, 0.4);
    ws.set_fit_to_page(1, 1);
    ws.set_print_area(0, 0, r, 6);
    ws.hide_gridlines();

    // ── Save ──
    let filename = format!(
        "{}_{}.xlsx",
        doc_type,
        inv.number.replace("-", "_").to_lowercase()
    );
    let path = std::path::PathBuf::from("output").join(&filename);
    wb.save(&path)?;
    println!("✅ {} saved to {}", label, path.display());

    Ok(())
}

use crate::features::table::CustomTableStyle;
use crate::model::style_registry::StyleRegistry;
use crate::xml::xml_writer::XmlWriter;

pub fn write_styles(reg: &StyleRegistry) -> Vec<u8> {
    write_styles_with_table_styles(reg, &[])
}

pub fn write_styles_with_table_styles(
    reg: &StyleRegistry,
    custom_table_styles: &[CustomTableStyle],
) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag(
        "styleSheet",
        &[(
            "xmlns",
            "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
        )],
    );

    // numFmts
    if !reg.num_formats.is_empty() {
        let count = reg.num_formats.len().to_string();
        w.start_tag("numFmts", &[("count", &count)]);
        for (id, code) in &reg.num_formats {
            let id_s = id.to_string();
            w.empty_tag("numFmt", &[("numFmtId", &id_s), ("formatCode", code)]);
        }
        w.end_tag("numFmts");
    }

    // fonts
    let fc = reg.fonts.len().to_string();
    w.start_tag("fonts", &[("count", &fc)]);
    for f in &reg.fonts {
        w.start_tag("font", &[]);
        if f.bold {
            w.empty_tag("b", &[]);
        }
        if f.italic {
            w.empty_tag("i", &[]);
        }
        if f.strikethrough {
            w.empty_tag("strike", &[]);
        }
        if f.shadow {
            w.empty_tag("shadow", &[]);
        }
        if f.outline {
            w.empty_tag("outline", &[]);
        }
        if f.emboss {
            w.empty_tag("emboss", &[]);
        }
        if f.engrave {
            w.empty_tag("engrave", &[]);
        }
        if f.underline == 1 {
            w.empty_tag("u", &[]);
        } else if f.underline == 2 {
            w.empty_tag("u", &[("val", "double")]);
        }
        let sz = format!("{:.1}", f.size_x100 as f64 / 100.0);
        w.empty_tag("sz", &[("val", &sz)]);
        if let Some((theme_idx, tint_x10000)) = f.theme_color {
            let theme_str = theme_idx.to_string();
            let tint = tint_x10000 as f64 / 10000.0;
            if tint == 0.0 {
                w.empty_tag("color", &[("theme", &theme_str)]);
            } else {
                let tint_str = format!("{:.15}", tint)
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string();
                w.empty_tag("color", &[("theme", &theme_str), ("tint", &tint_str)]);
            }
        } else if let Some(rgb) = f.color_rgb {
            let hex = format!("FF{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
            w.empty_tag("color", &[("rgb", &hex)]);
        }
        w.empty_tag("name", &[("val", &f.name)]);
        w.end_tag("font");
    }
    w.end_tag("fonts");

    // fills
    let fillc = reg.fills.len().to_string();
    w.start_tag("fills", &[("count", &fillc)]);
    for f in &reg.fills {
        w.start_tag("fill", &[]);
        if let Some(ref grad) = f.gradient {
            let angle_str = format!("{}", grad.angle_x100 as f64 / 100.0);
            w.start_tag("gradientFill", &[("degree", &angle_str)]);
            for stop in &grad.stops {
                let pos_str = format!("{}", stop.position_x1000 as f64 / 1000.0);
                w.start_tag("stop", &[("position", &pos_str)]);
                let hex = format!(
                    "FF{:02X}{:02X}{:02X}",
                    stop.color[0], stop.color[1], stop.color[2]
                );
                w.empty_tag("color", &[("rgb", &hex)]);
                w.end_tag("stop");
            }
            w.end_tag("gradientFill");
        } else {
            let pat = pattern_name(f.pattern);
            if f.fg_rgb.is_some() || f.bg_rgb.is_some() {
                w.start_tag("patternFill", &[("patternType", pat)]);
                if let Some(rgb) = f.fg_rgb {
                    let hex = format!("FF{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
                    w.empty_tag("fgColor", &[("rgb", &hex)]);
                }
                if let Some(rgb) = f.bg_rgb {
                    let hex = format!("FF{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
                    w.empty_tag("bgColor", &[("rgb", &hex)]);
                }
                w.end_tag("patternFill");
            } else {
                w.empty_tag("patternFill", &[("patternType", pat)]);
            }
        }
        w.end_tag("fill");
    }
    w.end_tag("fills");

    // borders
    let bc = reg.borders.len().to_string();
    w.start_tag("borders", &[("count", &bc)]);
    for b in &reg.borders {
        let mut border_attrs: Vec<(&str, &str)> = Vec::new();
        if b.diagonal_type == 1 || b.diagonal_type == 3 {
            border_attrs.push(("diagonalUp", "1"));
        }
        if b.diagonal_type == 2 || b.diagonal_type == 3 {
            border_attrs.push(("diagonalDown", "1"));
        }
        w.start_tag("border", &border_attrs);
        write_border_side(&mut w, "left", b.left, b.left_color.or(b.color_rgb));
        write_border_side(&mut w, "right", b.right, b.right_color.or(b.color_rgb));
        write_border_side(&mut w, "top", b.top, b.top_color.or(b.color_rgb));
        write_border_side(&mut w, "bottom", b.bottom, b.bottom_color.or(b.color_rgb));
        write_border_side(&mut w, "diagonal", b.diagonal, b.color_rgb);
        w.end_tag("border");
    }
    w.end_tag("borders");

    // cellStyleXfs (required, at least 1)
    let csxf_count = reg.cell_style_xfs.len().to_string();
    w.start_tag("cellStyleXfs", &[("count", &csxf_count)]);
    for xf in &reg.cell_style_xfs {
        let fid = xf.font_id.to_string();
        let flid = xf.fill_id.to_string();
        let bid = xf.border_id.to_string();
        let nid = xf.num_fmt_id.to_string();
        w.empty_tag(
            "xf",
            &[
                ("numFmtId", &nid),
                ("fontId", &fid),
                ("fillId", &flid),
                ("borderId", &bid),
            ],
        );
    }
    w.end_tag("cellStyleXfs");

    // cellXfs
    let xfc = reg.xf_records.len().to_string();
    w.start_tag("cellXfs", &[("count", &xfc)]);
    for (i, xf) in reg.xf_records.iter().enumerate() {
        let fid = xf.font_id.to_string();
        let flid = xf.fill_id.to_string();
        let bid = xf.border_id.to_string();
        let nid = xf.num_fmt_id.to_string();
        let xf_id_val = reg.xf_style_ids.get(i).copied().unwrap_or(0).to_string();
        let mut attrs: Vec<(&str, &str)> = vec![
            ("numFmtId", &nid),
            ("fontId", &fid),
            ("fillId", &flid),
            ("borderId", &bid),
            ("xfId", &xf_id_val),
        ];
        if xf.font_id > 0 {
            attrs.push(("applyFont", "1"));
        }
        if xf.fill_id > 0 {
            attrs.push(("applyFill", "1"));
        }
        if xf.border_id > 0 {
            attrs.push(("applyBorder", "1"));
        }
        if xf.num_fmt_id > 0 {
            attrs.push(("applyNumberFormat", "1"));
        }

        let has_protection = xf.locked.is_some() || xf.formula_hidden;
        if has_protection {
            attrs.push(("applyProtection", "1"));
        }
        if xf.quote_prefix {
            attrs.push(("quotePrefix", "1"));
        }

        if let Some(ref align) = xf.alignment {
            attrs.push(("applyAlignment", "1"));
            w.start_tag("xf", &attrs);
            let mut aa: Vec<(&str, String)> = Vec::new();
            if align.horizontal != 0 {
                aa.push((
                    "horizontal",
                    match align.horizontal {
                        1 => "left",
                        2 => "center",
                        3 => "right",
                        4 => "fill",
                        5 => "justify",
                        _ => "general",
                    }
                    .into(),
                ));
            }
            if align.vertical != 0 {
                aa.push((
                    "vertical",
                    match align.vertical {
                        1 => "center",
                        2 => "bottom",
                        _ => "top",
                    }
                    .into(),
                ));
            }
            if align.wrap_text {
                aa.push(("wrapText", "1".into()));
            }
            if align.shrink {
                aa.push(("shrinkToFit", "1".into()));
            }
            if align.indent > 0 {
                aa.push(("indent", align.indent.to_string()));
            }
            if align.rotation != 0 {
                aa.push(("textRotation", align.rotation.to_string()));
            }
            let refs: Vec<(&str, &str)> = aa.iter().map(|(k, v)| (*k, v.as_str())).collect();
            w.empty_tag("alignment", &refs);
            if has_protection {
                let mut pa: Vec<(&str, &str)> = Vec::new();
                if xf.locked == Some(false) {
                    pa.push(("locked", "0"));
                }
                if xf.formula_hidden {
                    pa.push(("hidden", "1"));
                }
                w.empty_tag("protection", &pa);
            }
            w.end_tag("xf");
        } else if has_protection {
            w.start_tag("xf", &attrs);
            let mut pa: Vec<(&str, &str)> = Vec::new();
            if xf.locked == Some(false) {
                pa.push(("locked", "0"));
            }
            if xf.formula_hidden {
                pa.push(("hidden", "1"));
            }
            w.empty_tag("protection", &pa);
            w.end_tag("xf");
        } else {
            w.empty_tag("xf", &attrs);
        }
    }
    w.end_tag("cellXfs");

    // cellStyles (required)
    let cs_count = reg.cell_styles.len().to_string();
    w.start_tag("cellStyles", &[("count", &cs_count)]);
    for cs in &reg.cell_styles {
        let xf_id_str = cs.xf_id.to_string();
        let builtin_str = cs.builtin_id.to_string();
        w.empty_tag(
            "cellStyle",
            &[
                ("name", &cs.name),
                ("xfId", &xf_id_str),
                ("builtinId", &builtin_str),
            ],
        );
    }
    w.end_tag("cellStyles");

    // dxfs (differential formatting for conditional formatting) — must come after cellStyles
    let dxf_count = reg.dxf_formats.len().to_string();
    w.start_tag("dxfs", &[("count", &dxf_count)]);
    for dxf in &reg.dxf_formats {
        w.start_tag("dxf", &[]);
        if let Some(ref f) = dxf.font {
            w.start_tag("font", &[]);
            if f.bold {
                w.empty_tag("b", &[]);
            }
            if f.italic {
                w.empty_tag("i", &[]);
            }
            if f.strikethrough {
                w.empty_tag("strike", &[]);
            }
            if f.underline == 1 {
                w.empty_tag("u", &[]);
            } else if f.underline == 2 {
                w.empty_tag("u", &[("val", "double")]);
            }
            if let Some(rgb) = f.color_rgb {
                let hex = format!("FF{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
                w.empty_tag("color", &[("rgb", &hex)]);
            }
            w.end_tag("font");
        }
        if let Some(ref fl) = dxf.fill {
            w.start_tag("fill", &[]);
            let pat = match fl.pattern {
                1 => "solid",
                _ => "none",
            };
            if let Some(rgb) = fl.fg_rgb {
                let hex = format!("FF{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
                w.start_tag("patternFill", &[("patternType", pat)]);
                w.empty_tag("bgColor", &[("rgb", &hex)]);
                w.end_tag("patternFill");
            } else {
                w.empty_tag("patternFill", &[("patternType", pat)]);
            }
            w.end_tag("fill");
        }
        if let Some(ref b) = dxf.border {
            w.start_tag("border", &[]);
            write_border_side(&mut w, "left", b.left, b.color_rgb);
            write_border_side(&mut w, "right", b.right, b.color_rgb);
            write_border_side(&mut w, "top", b.top, b.color_rgb);
            write_border_side(&mut w, "bottom", b.bottom, b.color_rgb);
            w.end_tag("border");
        }
        if let Some(ref nf) = dxf.num_format {
            w.empty_tag("numFmt", &[("numFmtId", "164"), ("formatCode", nf)]);
        }
        w.end_tag("dxf");
    }
    w.end_tag("dxfs");

    // tableStyles (custom table styles)
    if !custom_table_styles.is_empty() {
        let ts_count = custom_table_styles.len().to_string();
        w.start_tag(
            "tableStyles",
            &[
                ("count", &ts_count),
                ("defaultTableStyle", "TableStyleMedium2"),
                ("defaultPivotStyle", "PivotStyleLight16"),
            ],
        );
        for cts in custom_table_styles {
            // Count elements
            let mut element_count = 0u32;
            if cts.header_row.is_some() {
                element_count += 1;
            }
            if cts.total_row.is_some() {
                element_count += 1;
            }
            if cts.first_column.is_some() {
                element_count += 1;
            }
            if cts.last_column.is_some() {
                element_count += 1;
            }
            if cts.first_row_stripe.is_some() {
                element_count += 1;
            }
            if cts.second_row_stripe.is_some() {
                element_count += 1;
            }
            let ec_str = element_count.to_string();
            w.start_tag(
                "tableStyle",
                &[("name", &cts.name), ("pivot", "0"), ("count", &ec_str)],
            );
            // Write elements with dxfId referencing dxf entries (we use placeholder 0 for simplicity)
            if cts.header_row.is_some() {
                let size_str = "1".to_string();
                w.empty_tag(
                    "tableStyleElement",
                    &[("type", "headerRow"), ("size", &size_str)],
                );
            }
            if cts.total_row.is_some() {
                let size_str = "1".to_string();
                w.empty_tag(
                    "tableStyleElement",
                    &[("type", "totalRow"), ("size", &size_str)],
                );
            }
            if cts.first_column.is_some() {
                let size_str = "1".to_string();
                w.empty_tag(
                    "tableStyleElement",
                    &[("type", "firstColumn"), ("size", &size_str)],
                );
            }
            if cts.last_column.is_some() {
                let size_str = "1".to_string();
                w.empty_tag(
                    "tableStyleElement",
                    &[("type", "lastColumn"), ("size", &size_str)],
                );
            }
            if cts.first_row_stripe.is_some() {
                let size_str = cts.first_row_stripe_size.to_string();
                w.empty_tag(
                    "tableStyleElement",
                    &[("type", "firstRowStripe"), ("size", &size_str)],
                );
            }
            if cts.second_row_stripe.is_some() {
                let size_str = cts.second_row_stripe_size.to_string();
                w.empty_tag(
                    "tableStyleElement",
                    &[("type", "secondRowStripe"), ("size", &size_str)],
                );
            }
            w.end_tag("tableStyle");
        }
        w.end_tag("tableStyles");
    }

    w.end_tag("styleSheet");
    w.into_bytes()
}

fn write_border_side(w: &mut XmlWriter, name: &str, style: u8, color: Option<[u8; 3]>) {
    if style == 0 {
        w.empty_tag(name, &[]);
        return;
    }
    let style_name = match style {
        1 => "thin",
        2 => "medium",
        3 => "thick",
        4 => "dashed",
        5 => "dotted",
        6 => "double",
        _ => "thin",
    };
    if let Some(rgb) = color {
        w.start_tag(name, &[("style", style_name)]);
        let hex = format!("FF{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
        w.empty_tag("color", &[("rgb", &hex)]);
        w.end_tag(name);
    } else {
        w.start_tag(name, &[("style", style_name)]);
        w.empty_tag("color", &[("auto", "1")]);
        w.end_tag(name);
    }
}

fn pattern_name(p: u8) -> &'static str {
    match p {
        0 => "none",
        1 => "solid",
        2 => "mediumGray",
        3 => "darkGray",
        4 => "lightGray",
        5 => "darkHorizontal",
        6 => "darkVertical",
        7 => "darkDown",
        8 => "darkUp",
        9 => "darkGrid",
        10 => "darkTrellis",
        11 => "lightHorizontal",
        12 => "lightVertical",
        13 => "lightDown",
        14 => "lightUp",
        15 => "lightGrid",
        16 => "lightTrellis",
        17 => "gray125",
        _ => "none",
    }
}

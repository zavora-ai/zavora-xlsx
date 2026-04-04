use crate::model::style_registry::{StyleRegistry, BorderData};
use crate::xml::xml_writer::XmlWriter;

pub fn write_styles(reg: &StyleRegistry) -> Vec<u8> {
    let mut w = XmlWriter::new();
    w.declaration();
    w.start_tag("styleSheet", &[("xmlns", "http://schemas.openxmlformats.org/spreadsheetml/2006/main")]);

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
        if f.bold { w.empty_tag("b", &[]); }
        if f.italic { w.empty_tag("i", &[]); }
        if f.strikethrough { w.empty_tag("strike", &[]); }
        if f.underline == 1 { w.empty_tag("u", &[]); }
        else if f.underline == 2 { w.empty_tag("u", &[("val", "double")]); }
        let sz = format!("{:.1}", f.size_x100 as f64 / 100.0);
        w.empty_tag("sz", &[("val", &sz)]);
        if let Some(rgb) = f.color_rgb {
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
        let pat = match f.pattern {
            0 => "none", 1 => "solid", 17 => "gray125", _ => "none",
        };
        if let Some(rgb) = f.fg_rgb {
            let hex = format!("FF{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
            w.start_tag("patternFill", &[("patternType", pat)]);
            w.empty_tag("fgColor", &[("rgb", &hex)]);
            w.end_tag("patternFill");
        } else {
            w.empty_tag("patternFill", &[("patternType", pat)]);
        }
        w.end_tag("fill");
    }
    w.end_tag("fills");

    // borders
    let bc = reg.borders.len().to_string();
    w.start_tag("borders", &[("count", &bc)]);
    for b in &reg.borders {
        w.start_tag("border", &[]);
        write_border_side(&mut w, "left", b.left, b.color_rgb);
        write_border_side(&mut w, "right", b.right, b.color_rgb);
        write_border_side(&mut w, "top", b.top, b.color_rgb);
        write_border_side(&mut w, "bottom", b.bottom, b.color_rgb);
        w.empty_tag("diagonal", &[]);
        w.end_tag("border");
    }
    w.end_tag("borders");

    // cellStyleXfs (required, at least 1)
    w.start_tag("cellStyleXfs", &[("count", "1")]);
    w.empty_tag("xf", &[("numFmtId", "0"), ("fontId", "0"), ("fillId", "0"), ("borderId", "0")]);
    w.end_tag("cellStyleXfs");

    // cellXfs
    let xfc = reg.xf_records.len().to_string();
    w.start_tag("cellXfs", &[("count", &xfc)]);
    for xf in &reg.xf_records {
        let fid = xf.font_id.to_string();
        let flid = xf.fill_id.to_string();
        let bid = xf.border_id.to_string();
        let nid = xf.num_fmt_id.to_string();
        let mut attrs: Vec<(&str, &str)> = vec![
            ("numFmtId", &nid), ("fontId", &fid), ("fillId", &flid), ("borderId", &bid),
        ];
        if xf.font_id > 0 { attrs.push(("applyFont", "1")); }
        if xf.fill_id > 0 { attrs.push(("applyFill", "1")); }
        if xf.border_id > 0 { attrs.push(("applyBorder", "1")); }
        if xf.num_fmt_id > 0 { attrs.push(("applyNumberFormat", "1")); }

        if let Some(ref align) = xf.alignment {
            attrs.push(("applyAlignment", "1"));
            w.start_tag("xf", &attrs);
            let mut aa: Vec<(&str, String)> = Vec::new();
            if align.horizontal != 0 {
                aa.push(("horizontal", match align.horizontal {
                    1 => "left", 2 => "center", 3 => "right", 4 => "fill", 5 => "justify", _ => "general",
                }.into()));
            }
            if align.vertical != 0 {
                aa.push(("vertical", match align.vertical {
                    1 => "center", 2 => "bottom", _ => "top",
                }.into()));
            }
            if align.wrap_text { aa.push(("wrapText", "1".into())); }
            if align.shrink { aa.push(("shrinkToFit", "1".into())); }
            let refs: Vec<(&str, &str)> = aa.iter().map(|(k, v)| (*k, v.as_str())).collect();
            w.empty_tag("alignment", &refs);
            w.end_tag("xf");
        } else {
            w.empty_tag("xf", &attrs);
        }
    }
    w.end_tag("cellXfs");

    // cellStyles (required)
    w.start_tag("cellStyles", &[("count", "1")]);
    w.empty_tag("cellStyle", &[("name", "Normal"), ("xfId", "0"), ("builtinId", "0")]);
    w.end_tag("cellStyles");

    w.end_tag("styleSheet");
    w.into_bytes()
}

fn write_border_side(w: &mut XmlWriter, name: &str, style: u8, color: Option<[u8; 3]>) {
    if style == 0 {
        w.empty_tag(name, &[]);
        return;
    }
    let style_name = match style {
        1 => "thin", 2 => "medium", 3 => "thick", 4 => "dashed", 5 => "dotted", 6 => "double", _ => "thin",
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

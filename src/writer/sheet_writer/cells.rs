// Cell and rich text XML writing

use crate::cell::{CellType, RichText};
use crate::utility::{ColNum, RowNum, col_to_letter};
use crate::worksheet::types::PhoneticRun;
use crate::xml::xml_writer::XmlWriter;
use std::fmt::Write;

#[allow(dead_code)]
pub(crate) fn write_cell(w: &mut XmlWriter, row: RowNum, col: ColNum, cell: &CellType, xf: u32) {
    write_cell_with_phonetic(w, row, col, cell, xf, None);
}

pub(crate) fn write_cell_with_phonetic(
    w: &mut XmlWriter,
    row: RowNum,
    col: ColNum,
    cell: &CellType,
    xf: u32,
    phonetic: Option<&[PhoneticRun]>,
) {
    let ref_str = format!("{}{}", col_to_letter(col), row + 1);
    let xf_s = xf.to_string();

    match cell {
        CellType::Number(n) => {
            let mut v = String::new();
            let _ = write!(v, "{n}");
            if xf > 0 {
                w.start_tag("c", &[("r", &ref_str), ("s", &xf_s)]);
            } else {
                w.start_tag("c", &[("r", &ref_str)]);
            }
            w.text_element("v", &[], &v);
            w.end_tag("c");
        }
        CellType::SharedString(idx) => {
            let v = idx.to_string();
            let mut attrs: Vec<(&str, &str)> = vec![("r", &ref_str), ("t", "s")];
            if xf > 0 {
                attrs.push(("s", &xf_s));
            }
            w.start_tag("c", &attrs);
            w.text_element("v", &[], &v);
            w.end_tag("c");
        }
        CellType::InlineString(s) => {
            let mut attrs: Vec<(&str, &str)> = vec![("r", &ref_str), ("t", "inlineStr")];
            if xf > 0 {
                attrs.push(("s", &xf_s));
            }
            w.start_tag("c", &attrs);
            w.start_tag("is", &[]);
            w.text_element("t", &[], s);
            if let Some(runs) = phonetic {
                write_phonetic_runs(w, runs);
            }
            w.end_tag("is");
            w.end_tag("c");
        }
        CellType::Bool(b) => {
            let v = if *b { "1" } else { "0" };
            let mut attrs: Vec<(&str, &str)> = vec![("r", &ref_str), ("t", "b")];
            if xf > 0 {
                attrs.push(("s", &xf_s));
            }
            w.start_tag("c", &attrs);
            w.text_element("v", &[], v);
            w.end_tag("c");
        }
        CellType::Formula {
            text,
            cached_number,
        } => {
            if xf > 0 {
                w.start_tag("c", &[("r", &ref_str), ("s", &xf_s)]);
            } else {
                w.start_tag("c", &[("r", &ref_str)]);
            }
            w.text_element("f", &[], text);
            let n = cached_number.unwrap_or(0.0);
            let mut v = String::new();
            let _ = write!(v, "{n}");
            w.text_element("v", &[], &v);
            w.end_tag("c");
        }
        CellType::ArrayFormula { text, range } | CellType::DynamicFormula { text, range } => {
            if xf > 0 {
                w.start_tag("c", &[("r", &ref_str), ("s", &xf_s)]);
            } else {
                w.start_tag("c", &[("r", &ref_str)]);
            }
            w.text_element("f", &[("t", "array"), ("ref", range)], text);
            w.end_tag("c");
        }
        CellType::DateTime(serial) => {
            let mut v = String::new();
            let _ = write!(v, "{serial}");
            if xf > 0 {
                w.start_tag("c", &[("r", &ref_str), ("s", &xf_s)]);
            } else {
                w.start_tag("c", &[("r", &ref_str)]);
            }
            w.text_element("v", &[], &v);
            w.end_tag("c");
        }
        CellType::Error(e) => {
            let mut attrs: Vec<(&str, &str)> = vec![("r", &ref_str), ("t", "e")];
            if xf > 0 {
                attrs.push(("s", &xf_s));
            }
            w.start_tag("c", &attrs);
            w.text_element("v", &[], e);
            w.end_tag("c");
        }
        CellType::RichText(rt) => {
            let mut attrs: Vec<(&str, &str)> = vec![("r", &ref_str), ("t", "inlineStr")];
            if xf > 0 {
                attrs.push(("s", &xf_s));
            }
            w.start_tag("c", &attrs);
            w.start_tag("is", &[]);
            write_rich_text_runs(w, rt);
            if let Some(runs) = phonetic {
                write_phonetic_runs(w, runs);
            }
            w.end_tag("is");
            w.end_tag("c");
        }
        CellType::Empty => {
            if xf > 0 {
                w.empty_tag("c", &[("r", &ref_str), ("s", &xf_s)]);
            }
        }
    }
}

fn write_phonetic_runs(w: &mut XmlWriter, runs: &[PhoneticRun]) {
    for run in runs {
        let sb = run.start_index.to_string();
        let eb = run.end_index.to_string();
        w.start_tag("rPh", &[("sb", &sb), ("eb", &eb)]);
        w.text_element("t", &[], &run.text);
        w.end_tag("rPh");
    }
    w.empty_tag("phoneticPr", &[("fontId", "0")]);
}

pub(crate) fn write_rich_text_runs(w: &mut XmlWriter, rt: &RichText) {
    for run in &rt.runs {
        w.start_tag("r", &[]);
        let has_props = run.bold
            || run.italic
            || run.font_size.is_some()
            || run.font_name.is_some()
            || run.color.is_some()
            || run.superscript
            || run.subscript;
        if has_props {
            w.start_tag("rPr", &[]);
            if run.bold {
                w.empty_tag("b", &[]);
            }
            if run.italic {
                w.empty_tag("i", &[]);
            }
            if run.superscript {
                w.empty_tag("vertAlign", &[("val", "superscript")]);
            }
            if run.subscript {
                w.empty_tag("vertAlign", &[("val", "subscript")]);
            }
            if let Some(sz) = run.font_size {
                let s = format!("{sz}");
                w.empty_tag("sz", &[("val", &s)]);
            }
            if let Some(ref c) = run.color {
                let argb = if c.len() == 6 {
                    format!("FF{c}")
                } else {
                    c.clone()
                };
                w.empty_tag("color", &[("rgb", &argb)]);
            }
            if let Some(ref name) = run.font_name {
                w.empty_tag("rFont", &[("val", name)]);
            }
            w.end_tag("rPr");
        }
        w.text_element("t", &[("xml:space", "preserve")], &run.text);
        w.end_tag("r");
    }
}

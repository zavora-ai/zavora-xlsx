// Data validation writing and dimension computation

use crate::features::validation::{DataValidation, ValidationRule};
use crate::utility::col_to_letter;
use crate::xml::xml_writer::XmlWriter;
use super::SheetCells;

pub(crate) fn write_data_validation(w: &mut XmlWriter, dv: &DataValidation) {
    let sqref = format!("{}{}:{}{}",
        col_to_letter(dv.first_col), dv.first_row + 1,
        col_to_letter(dv.last_col), dv.last_row + 1);

    let (dv_type, formula1, formula2) = match &dv.rule {
        ValidationRule::List(values) => {
            let joined = format!("\"{}\"", values.join(","));
            ("list".to_string(), Some(joined), None)
        }
        ValidationRule::ListRange(range) => ("list".to_string(), Some(range.clone()), None),
        ValidationRule::WholeNumber { min, max } => {
            ("whole".to_string(), min.map(|v| v.to_string()).or_else(|| max.map(|v| v.to_string())), max.map(|v| v.to_string()))
        }
        ValidationRule::Decimal { min, max } => {
            ("decimal".to_string(), min.map(|v| v.to_string()).or_else(|| max.map(|v| v.to_string())), max.map(|v| v.to_string()))
        }
        ValidationRule::DateRange { min, max } => {
            ("date".to_string(), min.clone().or_else(|| max.clone()), max.clone())
        }
        ValidationRule::TextLength { min, max } => {
            ("textLength".to_string(), min.map(|v| v.to_string()).or_else(|| max.map(|v| v.to_string())), max.map(|v| v.to_string()))
        }
        ValidationRule::Custom(formula) => ("custom".to_string(), Some(formula.clone()), None),
    };

    let mut attrs: Vec<(&str, &str)> = vec![("type", &dv_type), ("sqref", &sqref), ("allowBlank", "1")];
    let op_str;
    if matches!(&dv.rule, ValidationRule::WholeNumber { min: Some(_), max: Some(_) } | ValidationRule::Decimal { min: Some(_), max: Some(_) } | ValidationRule::TextLength { min: Some(_), max: Some(_) }) {
        op_str = "between".to_string();
        attrs.push(("operator", &op_str));
    }
    let err_style = dv.error_style.xml_str().to_string();
    attrs.push(("errorStyle", &err_style));
    if dv.input_title.is_some() { attrs.push(("showInputMessage", "1")); }
    if dv.error_title.is_some() { attrs.push(("showErrorMessage", "1")); }

    let input_title_ref = dv.input_title.as_deref().unwrap_or("");
    let input_msg_ref = dv.input_message.as_deref().unwrap_or("");
    let error_title_ref = dv.error_title.as_deref().unwrap_or("");
    let error_msg_ref = dv.error_message.as_deref().unwrap_or("");
    if !input_title_ref.is_empty() { attrs.push(("promptTitle", input_title_ref)); }
    if !input_msg_ref.is_empty() { attrs.push(("prompt", input_msg_ref)); }
    if !error_title_ref.is_empty() { attrs.push(("errorTitle", error_title_ref)); }
    if !error_msg_ref.is_empty() { attrs.push(("error", error_msg_ref)); }

    w.start_tag("dataValidation", &attrs);
    if let Some(ref f1) = formula1 { w.text_element("formula1", &[], f1); }
    if let Some(ref f2) = formula2 { w.text_element("formula2", &[], f2); }
    w.end_tag("dataValidation");
}

pub(crate) fn compute_dimension(data: &SheetCells<'_>) -> String {
    if data.cells.is_empty() { return "A1".to_string(); }
    let min_r = *data.cells.keys().next().unwrap();
    let max_r = *data.cells.keys().next_back().unwrap();
    let mut min_c = u16::MAX;
    let mut max_c = 0u16;
    for cols in data.cells.values() {
        if let Some(&c) = cols.keys().next() { min_c = min_c.min(c); }
        if let Some(&c) = cols.keys().next_back() { max_c = max_c.max(c); }
    }
    if min_c == u16::MAX { min_c = 0; }
    format!("{}{}:{}{}", col_to_letter(min_c), min_r + 1, col_to_letter(max_c), max_r + 1)
}

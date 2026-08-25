//! Parser for `<dataValidations>` elements from sheet XML.
//!
//! Converts raw sheet XML bytes into [`DataValidation`] structs by parsing
//! `<dataValidation>` blocks and resolving type/operator attributes into
//! [`ValidationRule`] variants.

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::features::validation::{DataValidation, ErrorStyle, ValidationRule};
use crate::utility::{parse_cell_ref, parse_range};
use crate::xml::xml_reader::{BytesTextExt, decode_xml_ref, get_attr};

/// Parsed attributes from a `<dataValidation>` element.
struct DvAttrs {
    dv_type: String,
    operator: String,
    sqref: String,
    error_style: ErrorStyle,
    input_title: Option<String>,
    input_message: Option<String>,
    error_title: Option<String>,
    error_message: Option<String>,
}

impl DvAttrs {
    fn from_event(e: &quick_xml::events::BytesStart<'_>) -> Self {
        let dv_type = get_attr(e.attributes(), b"type")
            .and_then(|v| std::str::from_utf8(v).ok())
            .unwrap_or("")
            .to_string();

        let operator = get_attr(e.attributes(), b"operator")
            .and_then(|v| std::str::from_utf8(v).ok())
            .unwrap_or("between")
            .to_string();

        let sqref = get_attr(e.attributes(), b"sqref")
            .and_then(|v| std::str::from_utf8(v).ok())
            .unwrap_or("")
            .to_string();

        let error_style = match get_attr(e.attributes(), b"errorStyle")
            .and_then(|v| std::str::from_utf8(v).ok())
            .unwrap_or("stop")
        {
            "warning" => ErrorStyle::Warning,
            "information" => ErrorStyle::Information,
            _ => ErrorStyle::Stop,
        };

        let input_title = get_attr(e.attributes(), b"promptTitle")
            .and_then(|v| std::str::from_utf8(v).ok())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let input_message = get_attr(e.attributes(), b"prompt")
            .and_then(|v| std::str::from_utf8(v).ok())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let error_title = get_attr(e.attributes(), b"errorTitle")
            .and_then(|v| std::str::from_utf8(v).ok())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let error_message = get_attr(e.attributes(), b"error")
            .and_then(|v| std::str::from_utf8(v).ok())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        Self {
            dv_type,
            operator,
            sqref,
            error_style,
            input_title,
            input_message,
            error_title,
            error_message,
        }
    }
}

/// Parse all `<dataValidation>` elements from sheet XML into
/// [`DataValidation`] structs.
///
/// `data` is the raw XML bytes of the sheet.
///
/// Returns an empty `Vec` when no data validations are present.
pub fn parse_data_validations(data: &[u8]) -> Vec<DataValidation> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = false;
    let mut buf = Vec::with_capacity(1024);
    let mut results = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"dataValidation" => {
                let attrs = DvAttrs::from_event(e);
                let (formula1, formula2) = parse_dv_children(&mut reader, &mut buf);
                if let Some(dv) = build_dv(attrs, formula1, formula2) {
                    results.push(dv);
                }
            }
            Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"dataValidation" => {
                let attrs = DvAttrs::from_event(e);
                if let Some(dv) = build_dv(attrs, None, None) {
                    results.push(dv);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    results
}

/// Parse child elements (formula1, formula2) of a `<dataValidation>` element.
fn parse_dv_children(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
) -> (Option<String>, Option<String>) {
    let mut formula1: Option<String> = None;
    let mut formula2: Option<String> = None;
    let mut in_formula1 = false;
    let mut in_formula2 = false;
    let mut text = String::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref child)) => match child.local_name().as_ref() {
                b"formula1" => {
                    in_formula1 = true;
                    text.clear();
                }
                b"formula2" => {
                    in_formula2 = true;
                    text.clear();
                }
                _ => {}
            },
            Ok(Event::Text(ref t)) => {
                if (in_formula1 || in_formula2)
                    && let Ok(s) = t.unescape()
                {
                    text.push_str(&s);
                }
            }
            Ok(Event::GeneralRef(ref reference)) => {
                if (in_formula1 || in_formula2)
                    && let Ok(value) = decode_xml_ref(reference)
                {
                    text.push_str(&value);
                }
            }
            Ok(Event::End(ref child)) => match child.local_name().as_ref() {
                b"formula1" => {
                    in_formula1 = false;
                    formula1 = Some(text.clone());
                }
                b"formula2" => {
                    in_formula2 = false;
                    formula2 = Some(text.clone());
                }
                b"dataValidation" => break,
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    (formula1, formula2)
}

/// Build a `DataValidation` from parsed attributes and formula children.
fn build_dv(
    attrs: DvAttrs,
    formula1: Option<String>,
    formula2: Option<String>,
) -> Option<DataValidation> {
    // Parse the sqref — take the first range/cell reference
    let range_str = attrs.sqref.split_whitespace().next().unwrap_or("");
    let (first_row, first_col, last_row, last_col) = if let Some(r) = parse_range(range_str) {
        r
    } else if let Ok((row, col)) = parse_cell_ref(range_str) {
        (row, col, row, col)
    } else {
        return None;
    };

    let rule = build_validation_rule(&attrs.dv_type, &attrs.operator, formula1, formula2)?;

    Some(DataValidation {
        rule,
        input_title: attrs.input_title,
        input_message: attrs.input_message,
        error_style: attrs.error_style,
        error_title: attrs.error_title,
        error_message: attrs.error_message,
        first_row,
        first_col,
        last_row,
        last_col,
    })
}

/// Build a [`ValidationRule`] from the type, operator, and formula values.
fn build_validation_rule(
    dv_type: &str,
    operator: &str,
    formula1: Option<String>,
    formula2: Option<String>,
) -> Option<ValidationRule> {
    match dv_type {
        "list" => {
            let f1 = formula1?;
            // If formula1 is quoted like "Option1,Option2,Option3", it's an inline list
            if f1.starts_with('"') && f1.ends_with('"') {
                let inner = &f1[1..f1.len() - 1];
                let items: Vec<String> = inner.split(',').map(|s| s.to_string()).collect();
                Some(ValidationRule::List(items))
            } else {
                // It's a range reference
                Some(ValidationRule::ListRange(f1))
            }
        }
        "whole" => {
            let (min, max) = parse_numeric_range::<i64>(operator, &formula1, &formula2);
            Some(ValidationRule::WholeNumber { min, max })
        }
        "decimal" => {
            let (min, max) = parse_numeric_range::<f64>(operator, &formula1, &formula2);
            Some(ValidationRule::Decimal { min, max })
        }
        "date" | "time" => {
            let (min, max) = parse_string_range(operator, &formula1, &formula2);
            Some(ValidationRule::DateRange { min, max })
        }
        "textLength" => {
            let (min, max) = parse_numeric_range::<u32>(operator, &formula1, &formula2);
            Some(ValidationRule::TextLength { min, max })
        }
        "custom" => {
            let f1 = formula1?;
            Some(ValidationRule::Custom(f1))
        }
        _ => None,
    }
}

/// Parse min/max from operator and formula values for numeric types.
fn parse_numeric_range<T: std::str::FromStr>(
    operator: &str,
    formula1: &Option<String>,
    formula2: &Option<String>,
) -> (Option<T>, Option<T>) {
    match operator {
        "between" => {
            let min = formula1.as_deref().and_then(|s| s.parse::<T>().ok());
            let max = formula2.as_deref().and_then(|s| s.parse::<T>().ok());
            (min, max)
        }
        "greaterThan" | "greaterThanOrEqual" => {
            let min = formula1.as_deref().and_then(|s| s.parse::<T>().ok());
            (min, None)
        }
        "lessThan" | "lessThanOrEqual" => {
            let max = formula1.as_deref().and_then(|s| s.parse::<T>().ok());
            (None, max)
        }
        "equal" => {
            let val = formula1.as_deref().and_then(|s| s.parse::<T>().ok());
            let val2 = formula1.as_deref().and_then(|s| s.parse::<T>().ok());
            (val, val2)
        }
        _ => {
            // Default to "between" behavior
            let min = formula1.as_deref().and_then(|s| s.parse::<T>().ok());
            let max = formula2.as_deref().and_then(|s| s.parse::<T>().ok());
            (min, max)
        }
    }
}

/// Parse min/max from operator and formula values for string-based types (dates).
fn parse_string_range(
    operator: &str,
    formula1: &Option<String>,
    formula2: &Option<String>,
) -> (Option<String>, Option<String>) {
    match operator {
        "between" => (formula1.clone(), formula2.clone()),
        "greaterThan" | "greaterThanOrEqual" => (formula1.clone(), None),
        "lessThan" | "lessThanOrEqual" => (None, formula1.clone()),
        "equal" => (formula1.clone(), formula1.clone()),
        _ => (formula1.clone(), formula2.clone()),
    }
}

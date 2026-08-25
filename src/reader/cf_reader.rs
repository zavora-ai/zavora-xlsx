//! Parser for `<conditionalFormatting>` elements from sheet XML.
//!
//! Converts raw sheet XML bytes into [`StoredCf`] structs by parsing
//! `<conditionalFormatting>` blocks and resolving DXF indices via the
//! provided [`DxfRecord`] slice.

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::features::conditional::*;
use crate::format::Format;
use crate::reader::style_parser::DxfRecord;
use crate::utility::{ColNum, RowNum, parse_range};
use crate::xml::xml_reader::{BytesTextExt, decode_xml_ref, get_attr};

/// Parse all `<conditionalFormatting>` elements from sheet XML into
/// [`StoredCf`] structs.
///
/// `data` is the raw XML bytes of the sheet.  `dxf_records` is the
/// differential-formatting table parsed from `xl/styles.xml`, used to
/// resolve `dxfId` attributes on `<cfRule>` elements.
///
/// Returns an empty `Vec` when no conditional formatting is present.
pub fn parse_conditional_formats(data: &[u8], dxf_records: &[DxfRecord]) -> Vec<StoredCf> {
    let mut reader = Reader::from_reader(data);
    reader.config_mut().check_end_names = false;
    reader.config_mut().expand_empty_elements = false;
    let mut buf = Vec::with_capacity(1024);
    let mut results = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"conditionalFormatting" => {
                let sqref = get_attr(e.attributes(), b"sqref")
                    .and_then(|v| std::str::from_utf8(v).ok())
                    .unwrap_or("");
                // sqref may contain multiple ranges separated by spaces; take the first
                let range_str = sqref.split_whitespace().next().unwrap_or("");
                let range = if let Some(r) = parse_range(range_str) {
                    r
                } else if let Ok((row, col)) = crate::utility::parse_cell_ref(range_str) {
                    // Single cell reference like "A1"
                    (row, col, row, col)
                } else {
                    continue;
                };

                parse_cf_block(&mut reader, &mut buf, range, dxf_records, &mut results);
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    results
}

/// Parsed attributes from a `<cfRule>` element.
struct CfRuleAttrs {
    rule_type: String,
    dxf_id: Option<u32>,
    operator: String,
    text: String,
    rank: u32,
    bottom: bool,
    percent: bool,
    above_average: bool,
    equal_average: bool,
    time_period: String,
}

impl CfRuleAttrs {
    fn from_event(e: &quick_xml::events::BytesStart<'_>) -> Self {
        Self {
            rule_type: get_attr(e.attributes(), b"type")
                .and_then(|v| std::str::from_utf8(v).ok())
                .unwrap_or("")
                .to_string(),
            dxf_id: get_attr(e.attributes(), b"dxfId")
                .and_then(|v| std::str::from_utf8(v).ok())
                .and_then(|v| v.parse().ok()),
            operator: get_attr(e.attributes(), b"operator")
                .and_then(|v| std::str::from_utf8(v).ok())
                .unwrap_or("")
                .to_string(),
            text: get_attr(e.attributes(), b"text")
                .and_then(|v| std::str::from_utf8(v).ok())
                .unwrap_or("")
                .to_string(),
            rank: get_attr(e.attributes(), b"rank")
                .and_then(|v| std::str::from_utf8(v).ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            bottom: get_attr(e.attributes(), b"bottom")
                .and_then(|v| std::str::from_utf8(v).ok())
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false),
            percent: get_attr(e.attributes(), b"percent")
                .and_then(|v| std::str::from_utf8(v).ok())
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false),
            above_average: get_attr(e.attributes(), b"aboveAverage")
                .and_then(|v| std::str::from_utf8(v).ok())
                .map(|v| v != "0")
                .unwrap_or(true),
            equal_average: get_attr(e.attributes(), b"equalAverage")
                .and_then(|v| std::str::from_utf8(v).ok())
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false),
            time_period: get_attr(e.attributes(), b"timePeriod")
                .and_then(|v| std::str::from_utf8(v).ok())
                .unwrap_or("")
                .to_string(),
        }
    }
}

/// Child data collected from within a `<cfRule>` element.
#[derive(Default)]
struct CfRuleChildren {
    formulas: Vec<String>,
    color_scale_colors: Vec<[u8; 3]>,
    data_bar_color: Option<[u8; 3]>,
    icon_set_type: Option<String>,
}

/// Parse all `<cfRule>` elements within a single `<conditionalFormatting>` block.
fn parse_cf_block(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    range: (RowNum, ColNum, RowNum, ColNum),
    dxf_records: &[DxfRecord],
    results: &mut Vec<StoredCf>,
) {
    loop {
        buf.clear();
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) if e.local_name().as_ref() == b"cfRule" => {
                let attrs = CfRuleAttrs::from_event(e);
                let children = parse_cf_rule_children(reader, buf);
                let fmt = attrs.dxf_id.and_then(|id| resolve_dxf(id, dxf_records));
                if let Some(rule) = build_rule(&attrs, &children, fmt.as_ref()) {
                    results.push(StoredCf {
                        range,
                        rule,
                        dxf_id: attrs.dxf_id,
                    });
                }
            }
            Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"cfRule" => {
                let attrs = CfRuleAttrs::from_event(e);
                let children = CfRuleChildren::default();
                let fmt = attrs.dxf_id.and_then(|id| resolve_dxf(id, dxf_records));
                if let Some(rule) = build_rule(&attrs, &children, fmt.as_ref()) {
                    results.push(StoredCf {
                        range,
                        rule,
                        dxf_id: attrs.dxf_id,
                    });
                }
            }
            Ok(Event::End(ref e)) if e.local_name().as_ref() == b"conditionalFormatting" => break,
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
}

/// Parse child elements of a `<cfRule>`: formulas, colorScale, dataBar, iconSet.
fn parse_cf_rule_children(reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> CfRuleChildren {
    let mut children = CfRuleChildren::default();
    let mut in_color_scale = false;
    let mut in_data_bar = false;
    let mut in_formula = false;
    let mut formula_text = String::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => match e.local_name().as_ref() {
                b"formula" => {
                    in_formula = true;
                    formula_text.clear();
                }
                b"colorScale" => in_color_scale = true,
                b"dataBar" => in_data_bar = true,
                b"iconSet" => {
                    children.icon_set_type = get_attr(e.attributes(), b"iconSet")
                        .and_then(|v| std::str::from_utf8(v).ok())
                        .map(|s| s.to_string());
                }
                _ => {}
            },
            Ok(Event::Empty(ref e)) => match e.local_name().as_ref() {
                b"cfvo" => { /* cfvo type info not needed for reconstruction */ }
                b"color" if in_color_scale => {
                    if let Some(rgb) = parse_rgb_attr(e.attributes()) {
                        children.color_scale_colors.push(rgb);
                    }
                }
                b"color" if in_data_bar => {
                    if let Some(rgb) = parse_rgb_attr(e.attributes()) {
                        children.data_bar_color = Some(rgb);
                    }
                }
                _ => {}
            },
            Ok(Event::Text(ref t)) => {
                if in_formula && let Ok(s) = t.unescape() {
                    formula_text.push_str(&s);
                }
            }
            Ok(Event::GeneralRef(ref reference)) => {
                if in_formula && let Ok(text) = decode_xml_ref(reference) {
                    formula_text.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => match e.local_name().as_ref() {
                b"formula" => {
                    in_formula = false;
                    children.formulas.push(formula_text.clone());
                }
                b"colorScale" => in_color_scale = false,
                b"dataBar" => in_data_bar = false,
                b"iconSet" => {}
                b"cfRule" => break,
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    children
}

// ── Rule construction ───────────────────────────────────────────────────────

/// Build a boxed `ConditionalFormat` rule from parsed attributes and children,
/// applying the resolved DXF format where applicable.
fn build_rule(
    attrs: &CfRuleAttrs,
    children: &CfRuleChildren,
    fmt: Option<&Format>,
) -> Option<Box<dyn ConditionalFormat>> {
    match attrs.rule_type.as_str() {
        "cellIs" => {
            let op = parse_operator(&attrs.operator)?;
            let value: f64 = children.formulas.first()?.parse().ok()?;
            let mut rule = ConditionalFormatCell::new(op, value);
            if let Some(v2_str) = children.formulas.get(1)
                && let Ok(v2) = v2_str.parse::<f64>()
            {
                rule.set_value2(v2);
            }
            if let Some(f) = fmt {
                rule.set_format(f);
            }
            Some(Box::new(rule))
        }
        "colorScale" => match children.color_scale_colors.len() {
            2 => Some(Box::new(ConditionalFormat2ColorScale {
                min_color: children.color_scale_colors[0],
                max_color: children.color_scale_colors[1],
            })),
            3 => Some(Box::new(ConditionalFormat3ColorScale {
                min_color: children.color_scale_colors[0],
                mid_color: children.color_scale_colors[1],
                max_color: children.color_scale_colors[2],
            })),
            _ => None,
        },
        "dataBar" => {
            let c = children.data_bar_color.unwrap_or([99, 142, 198]);
            Some(Box::new(ConditionalFormatDataBar {
                color: c,
                gradient: false,
            }))
        }
        "iconSet" => {
            let ist = match children
                .icon_set_type
                .as_deref()
                .unwrap_or("3TrafficLights")
            {
                "3Arrows" | "3ArrowsGray" => IconSetType::ThreeArrows,
                "3TrafficLights" | "3TrafficLights1" | "3TrafficLights2" => {
                    IconSetType::ThreeTrafficLights
                }
                "3Symbols" | "3Symbols2" => IconSetType::ThreeSymbols,
                "4Arrows" | "4ArrowsGray" => IconSetType::FourArrows,
                "5Arrows" | "5ArrowsGray" => IconSetType::FiveArrows,
                _ => IconSetType::ThreeTrafficLights,
            };
            Some(Box::new(ConditionalFormatIconSet::new(ist)))
        }
        "expression" => {
            let f = children.formulas.first()?;
            if f.is_empty() {
                return None;
            }
            let mut rule = ConditionalFormatFormula::new(f);
            if let Some(fm) = fmt {
                rule.set_format(fm);
            }
            Some(Box::new(rule))
        }
        "top10" => {
            let kind = match (attrs.bottom, attrs.percent) {
                (false, false) => TopBottomType::Top,
                (true, false) => TopBottomType::Bottom,
                (false, true) => TopBottomType::TopPercent,
                (true, true) => TopBottomType::BottomPercent,
            };
            let mut rule = ConditionalFormatTopBottom::new(kind, attrs.rank);
            if let Some(f) = fmt {
                rule.set_format(f);
            }
            Some(Box::new(rule))
        }
        "containsText" | "notContainsText" | "beginsWith" | "endsWith" => {
            let op = match attrs.rule_type.as_str() {
                "containsText" => TextOperator::Contains,
                "notContainsText" => TextOperator::NotContains,
                "beginsWith" => TextOperator::BeginsWith,
                "endsWith" => TextOperator::EndsWith,
                _ => return None,
            };
            let mut rule = ConditionalFormatText::new(op, &attrs.text);
            if let Some(f) = fmt {
                rule.set_format(f);
            }
            Some(Box::new(rule))
        }
        "duplicateValues" => {
            let mut rule = ConditionalFormatDuplicate::new();
            if let Some(f) = fmt {
                rule.set_format(f);
            }
            Some(Box::new(rule))
        }
        "uniqueValues" => {
            let mut rule = ConditionalFormatUnique::new();
            if let Some(f) = fmt {
                rule.set_format(f);
            }
            Some(Box::new(rule))
        }
        "aboveAverage" => {
            let kind = match (attrs.above_average, attrs.equal_average) {
                (true, false) => AverageType::Above,
                (false, false) => AverageType::Below,
                (true, true) => AverageType::AboveOrEqual,
                (false, true) => AverageType::BelowOrEqual,
            };
            let mut rule = ConditionalFormatAverage::new(kind);
            if let Some(f) = fmt {
                rule.set_format(f);
            }
            Some(Box::new(rule))
        }
        "timePeriod" => {
            let period = match attrs.time_period.as_str() {
                "yesterday" => DateOccurring::Yesterday,
                "today" => DateOccurring::Today,
                "tomorrow" => DateOccurring::Tomorrow,
                "last7Days" => DateOccurring::Last7Days,
                "thisWeek" => DateOccurring::ThisWeek,
                "lastWeek" => DateOccurring::LastWeek,
                "nextWeek" => DateOccurring::NextWeek,
                "thisMonth" => DateOccurring::ThisMonth,
                "lastMonth" => DateOccurring::LastMonth,
                "nextMonth" => DateOccurring::NextMonth,
                _ => return None,
            };
            let mut rule = ConditionalFormatDate::new(period);
            if let Some(f) = fmt {
                rule.set_format(f);
            }
            Some(Box::new(rule))
        }
        _ => None, // skip unknown types gracefully
    }
}

// ── DXF resolution (Task 2.3) ──────────────────────────────────────────────

/// Resolve a DXF index to a `Format` struct by looking up the DXF record
/// and converting its optional font/fill/border/num_fmt components.
fn resolve_dxf(dxf_id: u32, dxf_records: &[DxfRecord]) -> Option<Format> {
    let dxf = dxf_records.get(dxf_id as usize)?;
    let mut fmt = Format::new();
    let mut has_any = false;

    if let Some(ref font) = dxf.font {
        fmt.bold = font.bold;
        fmt.italic = font.italic;
        fmt.underline = font.underline;
        fmt.strikethrough = font.strikethrough;
        fmt.font_size = font.size;
        fmt.font_name = font.name.clone();
        fmt.font_color = font.color;
        has_any = true;
    }

    if let Some(ref fill) = dxf.fill {
        fmt.fg_color = fill.fg_color;
        fmt.bg_color = fill.bg_color;
        fmt.pattern = fill.pattern;
        has_any = true;
    }

    if let Some(ref border) = dxf.border {
        fmt.border_left = border.left.style;
        fmt.border_right = border.right.style;
        fmt.border_top = border.top.style;
        fmt.border_bottom = border.bottom.style;
        fmt.border_left_color = border.left.color;
        fmt.border_right_color = border.right.color;
        fmt.border_top_color = border.top.color;
        fmt.border_bottom_color = border.bottom.color;
        fmt.diagonal_border = border.diagonal.style;
        has_any = true;
    }

    if let Some((_, ref code)) = dxf.num_fmt {
        fmt.num_format = code.clone();
        has_any = true;
    }

    if has_any { Some(fmt) } else { None }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Parse an RGB color from a `<color rgb="FFRRGGBB"/>` attribute.
fn parse_rgb_attr(attrs: quick_xml::events::attributes::Attributes<'_>) -> Option<[u8; 3]> {
    let val = get_attr(attrs, b"rgb")?;
    let s = std::str::from_utf8(val).ok()?;
    let hex = if s.len() >= 8 {
        &s[2..8]
    } else if s.len() >= 6 {
        &s[0..6]
    } else {
        return None;
    };
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r, g, b])
}

fn parse_operator(op: &str) -> Option<CfOperator> {
    match op {
        "greaterThan" => Some(CfOperator::GreaterThan),
        "lessThan" => Some(CfOperator::LessThan),
        "between" => Some(CfOperator::Between),
        "equal" => Some(CfOperator::EqualTo),
        "notEqual" => Some(CfOperator::NotEqualTo),
        "greaterThanOrEqual" => Some(CfOperator::GreaterThanOrEqual),
        "lessThanOrEqual" => Some(CfOperator::LessThanOrEqual),
        _ => None,
    }
}

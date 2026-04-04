use crate::format::Format;
use crate::utility::{ColNum, RowNum};

/// Trait for all conditional format types.
pub trait ConditionalFormat {
    fn cf_type(&self) -> &str;
    fn write_rule(&self, w: &mut crate::xml::xml_writer::XmlWriter, priority: u32);
}

#[derive(Debug, Clone, Copy)]
pub enum CfOperator { GreaterThan, LessThan, Between, EqualTo, NotEqualTo, GreaterThanOrEqual, LessThanOrEqual }

impl CfOperator {
    pub fn xml_str(&self) -> &str {
        match self {
            CfOperator::GreaterThan => "greaterThan", CfOperator::LessThan => "lessThan",
            CfOperator::Between => "between", CfOperator::EqualTo => "equal",
            CfOperator::NotEqualTo => "notEqual", CfOperator::GreaterThanOrEqual => "greaterThanOrEqual",
            CfOperator::LessThanOrEqual => "lessThanOrEqual",
        }
    }
}

// ── Cell Value Rule ──

#[derive(Debug, Clone)]
pub struct ConditionalFormatCell {
    pub(crate) operator: CfOperator,
    pub(crate) value: f64,
    pub(crate) value2: Option<f64>,
    pub(crate) format: Option<Format>,
}

impl ConditionalFormatCell {
    pub fn new(op: CfOperator, value: f64) -> Self {
        Self { operator: op, value, value2: None, format: None }
    }
    pub fn set_value2(&mut self, v: f64) -> &mut Self { self.value2 = Some(v); self }
    pub fn set_format(&mut self, f: &Format) -> &mut Self { self.format = Some(f.clone()); self }
}

impl ConditionalFormat for ConditionalFormatCell {
    fn cf_type(&self) -> &str { "cellIs" }
    fn write_rule(&self, w: &mut crate::xml::xml_writer::XmlWriter, priority: u32) {
        let p = priority.to_string();
        w.start_tag("cfRule", &[("type", "cellIs"), ("operator", self.operator.xml_str()), ("priority", &p)]);
        w.text_element("formula", &[], &self.value.to_string());
        if let Some(v2) = self.value2 {
            w.text_element("formula", &[], &v2.to_string());
        }
        w.end_tag("cfRule");
    }
}

// ── 2-Color Scale ──

#[derive(Debug, Clone)]
pub struct ConditionalFormat2ColorScale {
    pub(crate) min_color: [u8; 3],
    pub(crate) max_color: [u8; 3],
}

impl ConditionalFormat2ColorScale {
    pub fn new(min: impl crate::format::IntoColor, max: impl crate::format::IntoColor) -> Self {
        Self { min_color: min.into_color().to_rgb(), max_color: max.into_color().to_rgb() }
    }
}

impl ConditionalFormat for ConditionalFormat2ColorScale {
    fn cf_type(&self) -> &str { "colorScale" }
    fn write_rule(&self, w: &mut crate::xml::xml_writer::XmlWriter, priority: u32) {
        let p = priority.to_string();
        w.start_tag("cfRule", &[("type", "colorScale"), ("priority", &p)]);
        w.start_tag("colorScale", &[]);
        w.empty_tag("cfvo", &[("type", "min")]);
        w.empty_tag("cfvo", &[("type", "max")]);
        let min_hex = format!("FF{:02X}{:02X}{:02X}", self.min_color[0], self.min_color[1], self.min_color[2]);
        let max_hex = format!("FF{:02X}{:02X}{:02X}", self.max_color[0], self.max_color[1], self.max_color[2]);
        w.empty_tag("color", &[("rgb", &min_hex)]);
        w.empty_tag("color", &[("rgb", &max_hex)]);
        w.end_tag("colorScale");
        w.end_tag("cfRule");
    }
}

// ── 3-Color Scale ──

#[derive(Debug, Clone)]
pub struct ConditionalFormat3ColorScale {
    pub(crate) min_color: [u8; 3],
    pub(crate) mid_color: [u8; 3],
    pub(crate) max_color: [u8; 3],
}

impl ConditionalFormat3ColorScale {
    pub fn new(min: impl crate::format::IntoColor, mid: impl crate::format::IntoColor, max: impl crate::format::IntoColor) -> Self {
        Self { min_color: min.into_color().to_rgb(), mid_color: mid.into_color().to_rgb(), max_color: max.into_color().to_rgb() }
    }
}

impl ConditionalFormat for ConditionalFormat3ColorScale {
    fn cf_type(&self) -> &str { "colorScale" }
    fn write_rule(&self, w: &mut crate::xml::xml_writer::XmlWriter, priority: u32) {
        let p = priority.to_string();
        w.start_tag("cfRule", &[("type", "colorScale"), ("priority", &p)]);
        w.start_tag("colorScale", &[]);
        w.empty_tag("cfvo", &[("type", "min")]);
        w.empty_tag("cfvo", &[("type", "percentile"), ("val", "50")]);
        w.empty_tag("cfvo", &[("type", "max")]);
        let c = |rgb: [u8; 3]| format!("FF{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]);
        let min_h = c(self.min_color); let mid_h = c(self.mid_color); let max_h = c(self.max_color);
        w.empty_tag("color", &[("rgb", &min_h)]);
        w.empty_tag("color", &[("rgb", &mid_h)]);
        w.empty_tag("color", &[("rgb", &max_h)]);
        w.end_tag("colorScale");
        w.end_tag("cfRule");
    }
}

// ── Data Bar ──

#[derive(Debug, Clone)]
pub struct ConditionalFormatDataBar {
    pub(crate) color: [u8; 3],
}

impl ConditionalFormatDataBar {
    pub fn new(color: impl crate::format::IntoColor) -> Self {
        Self { color: color.into_color().to_rgb() }
    }
}

impl ConditionalFormat for ConditionalFormatDataBar {
    fn cf_type(&self) -> &str { "dataBar" }
    fn write_rule(&self, w: &mut crate::xml::xml_writer::XmlWriter, priority: u32) {
        let p = priority.to_string();
        w.start_tag("cfRule", &[("type", "dataBar"), ("priority", &p)]);
        w.start_tag("dataBar", &[]);
        w.empty_tag("cfvo", &[("type", "min")]);
        w.empty_tag("cfvo", &[("type", "max")]);
        let hex = format!("FF{:02X}{:02X}{:02X}", self.color[0], self.color[1], self.color[2]);
        w.empty_tag("color", &[("rgb", &hex)]);
        w.end_tag("dataBar");
        w.end_tag("cfRule");
    }
}

// ── Icon Set ──

#[derive(Debug, Clone, Copy)]
pub enum IconSetType { ThreeArrows, ThreeTrafficLights, ThreeSymbols, FourArrows, FiveArrows }

impl IconSetType {
    pub fn xml_str(&self) -> &str {
        match self {
            IconSetType::ThreeArrows => "3Arrows", IconSetType::ThreeTrafficLights => "3TrafficLights",
            IconSetType::ThreeSymbols => "3Symbols", IconSetType::FourArrows => "4Arrows",
            IconSetType::FiveArrows => "5Arrows",
        }
    }
    fn count(&self) -> usize {
        match self { IconSetType::ThreeArrows | IconSetType::ThreeTrafficLights | IconSetType::ThreeSymbols => 3, IconSetType::FourArrows => 4, IconSetType::FiveArrows => 5 }
    }
}

#[derive(Debug, Clone)]
pub struct ConditionalFormatIconSet {
    pub(crate) icon_type: IconSetType,
}

impl ConditionalFormatIconSet {
    pub fn new(icon_type: IconSetType) -> Self { Self { icon_type } }
}

impl ConditionalFormat for ConditionalFormatIconSet {
    fn cf_type(&self) -> &str { "iconSet" }
    fn write_rule(&self, w: &mut crate::xml::xml_writer::XmlWriter, priority: u32) {
        let p = priority.to_string();
        w.start_tag("cfRule", &[("type", "iconSet"), ("priority", &p)]);
        w.start_tag("iconSet", &[("iconSet", self.icon_type.xml_str())]);
        let n = self.icon_type.count();
        for i in 0..n {
            if i == 0 {
                w.empty_tag("cfvo", &[("type", "percent"), ("val", "0")]);
            } else {
                let pct = (100 * i / n).to_string();
                w.empty_tag("cfvo", &[("type", "percent"), ("val", &pct)]);
            }
        }
        w.end_tag("iconSet");
        w.end_tag("cfRule");
    }
}

/// Stored conditional format with range info.
pub(crate) struct StoredCf {
    pub range: (RowNum, ColNum, RowNum, ColNum),
    pub rule: Box<dyn ConditionalFormat>,
}

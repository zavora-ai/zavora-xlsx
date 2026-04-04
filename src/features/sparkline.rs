use crate::utility::{ColNum, RowNum};

#[derive(Debug, Clone, Copy)]
pub enum SparklineType { Line, Column, WinLoss }

impl SparklineType {
    pub fn xml_str(&self) -> &str {
        match self { SparklineType::Line => "line", SparklineType::Column => "column", SparklineType::WinLoss => "stacked" }
    }
}

#[derive(Debug, Clone)]
pub struct Sparkline {
    pub(crate) data_range: String,
    pub(crate) sparkline_type: SparklineType,
    pub(crate) color: Option<[u8; 3]>,
    pub(crate) row: RowNum,
    pub(crate) col: ColNum,
}

impl Sparkline {
    pub fn new(data_range: &str, sparkline_type: SparklineType) -> Self {
        Self { data_range: data_range.into(), sparkline_type, color: None, row: 0, col: 0 }
    }
    pub fn set_color(&mut self, c: impl crate::format::IntoColor) -> &mut Self {
        self.color = Some(c.into_color().to_rgb()); self
    }
}

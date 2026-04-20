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

    /// The data range reference (e.g. `"Sheet1!B1:B10"`).
    pub fn data_range(&self) -> &str { &self.data_range }

    /// The sparkline type (line, column, or win/loss).
    pub fn sparkline_type(&self) -> SparklineType { self.sparkline_type }

    /// The row where this sparkline is rendered (0-based).
    pub fn row(&self) -> RowNum { self.row }

    /// The column where this sparkline is rendered (0-based).
    pub fn col(&self) -> ColNum { self.col }

    /// The optional color of the sparkline series.
    pub fn color(&self) -> Option<[u8; 3]> { self.color }
}

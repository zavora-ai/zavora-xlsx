use crate::format::IntoColor;
use crate::utility::{ColNum, RowNum};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartType { Bar, Column, Line, Pie, Scatter, Area, Doughnut, Radar }

#[derive(Debug, Clone, Copy)]
pub enum LegendPosition { Top, Bottom, Left, Right, None }

#[derive(Debug, Clone)]
pub struct ChartSeries {
    pub(crate) values: String,
    pub(crate) categories: Option<String>,
    pub(crate) name: Option<String>,
}

impl ChartSeries {
    pub fn new() -> Self { Self { values: String::new(), categories: None, name: None } }
    pub fn set_values(&mut self, range: &str) -> &mut Self { self.values = range.into(); self }
    pub fn set_categories(&mut self, range: &str) -> &mut Self { self.categories = Some(range.into()); self }
    pub fn set_name(&mut self, name: &str) -> &mut Self { self.name = Some(name.into()); self }
}

#[derive(Debug, Clone)]
pub struct Chart {
    pub(crate) chart_type: ChartType,
    pub(crate) series: Vec<ChartSeries>,
    pub(crate) title: Option<String>,
    pub(crate) x_axis_name: Option<String>,
    pub(crate) y_axis_name: Option<String>,
    pub(crate) legend_pos: LegendPosition,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) row: RowNum,
    pub(crate) col: ColNum,
}

impl Chart {
    pub fn new(chart_type: ChartType) -> Self {
        Self {
            chart_type, series: Vec::new(), title: None,
            x_axis_name: None, y_axis_name: None,
            legend_pos: LegendPosition::Bottom,
            width: 480, height: 288, row: 0, col: 0,
        }
    }

    pub fn add_series(&mut self) -> &mut ChartSeries {
        self.series.push(ChartSeries::new());
        self.series.last_mut().unwrap()
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self { self.title = Some(title.into()); self }
    pub fn set_x_axis_name(&mut self, name: &str) -> &mut Self { self.x_axis_name = Some(name.into()); self }
    pub fn set_y_axis_name(&mut self, name: &str) -> &mut Self { self.y_axis_name = Some(name.into()); self }
    pub fn set_legend_position(&mut self, pos: LegendPosition) -> &mut Self { self.legend_pos = pos; self }
    pub fn set_width(&mut self, w: u32) -> &mut Self { self.width = w; self }
    pub fn set_height(&mut self, h: u32) -> &mut Self { self.height = h; self }
}

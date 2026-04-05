use crate::utility::{ColNum, RowNum};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartType { Bar, Column, Line, Pie, Scatter, Area, Doughnut, Radar, Stock }

#[derive(Debug, Clone, Copy)]
pub enum LegendPosition { Top, Bottom, Left, Right, None }

#[derive(Debug, Clone, Copy)]
pub enum TrendlineType { Linear, Exponential, Polynomial(u8), Power, Logarithmic, MovingAverage(u8) }

#[derive(Debug, Clone)]
pub struct ChartSeries {
    pub(crate) values: String,
    pub(crate) categories: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) secondary_axis: bool,
    pub(crate) data_labels: bool,
    pub(crate) trendline: Option<TrendlineType>,
    pub(crate) chart_type_override: Option<ChartType>,
}

impl ChartSeries {
    pub fn new() -> Self {
        Self { values: String::new(), categories: None, name: None,
            secondary_axis: false, data_labels: false,
            trendline: None, chart_type_override: None }
    }
    pub fn set_values(&mut self, range: &str) -> &mut Self { self.values = range.into(); self }
    pub fn set_categories(&mut self, range: &str) -> &mut Self { self.categories = Some(range.into()); self }
    pub fn set_name(&mut self, name: &str) -> &mut Self { self.name = Some(name.into()); self }
    pub fn set_secondary_axis(&mut self, v: bool) -> &mut Self { self.secondary_axis = v; self }
    pub fn set_data_labels(&mut self, v: bool) -> &mut Self { self.data_labels = v; self }
    pub fn set_trendline(&mut self, t: TrendlineType) -> &mut Self { self.trendline = Some(t); self }
    pub fn set_chart_type(&mut self, ct: ChartType) -> &mut Self { self.chart_type_override = Some(ct); self }
}

#[derive(Debug, Clone)]
pub struct Chart {
    pub(crate) chart_type: ChartType,
    pub(crate) series: Vec<ChartSeries>,
    pub(crate) title: Option<String>,
    pub(crate) x_axis_name: Option<String>,
    pub(crate) y_axis_name: Option<String>,
    pub(crate) y2_axis_name: Option<String>,
    pub(crate) legend_pos: LegendPosition,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) row: RowNum,
    pub(crate) col: ColNum,
    pub(crate) x_offset: u32,
    pub(crate) y_offset: u32,
    pub(crate) show_data_table: bool,
}

impl Chart {
    pub fn new(chart_type: ChartType) -> Self {
        Self {
            chart_type, series: Vec::new(), title: None,
            x_axis_name: None, y_axis_name: None, y2_axis_name: None,
            legend_pos: LegendPosition::Bottom,
            width: 480, height: 288, row: 0, col: 0,
            x_offset: 0, y_offset: 0, show_data_table: false,
        }
    }

    pub fn add_series(&mut self) -> &mut ChartSeries {
        self.series.push(ChartSeries::new());
        self.series.last_mut().unwrap()
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self { self.title = Some(title.into()); self }
    pub fn set_x_axis_name(&mut self, name: &str) -> &mut Self { self.x_axis_name = Some(name.into()); self }
    pub fn set_y_axis_name(&mut self, name: &str) -> &mut Self { self.y_axis_name = Some(name.into()); self }
    pub fn set_y2_axis_name(&mut self, name: &str) -> &mut Self { self.y2_axis_name = Some(name.into()); self }
    pub fn set_legend_position(&mut self, pos: LegendPosition) -> &mut Self { self.legend_pos = pos; self }
    pub fn set_width(&mut self, w: u32) -> &mut Self { self.width = w; self }
    pub fn set_height(&mut self, h: u32) -> &mut Self { self.height = h; self }
    pub fn show_data_table(&mut self, v: bool) -> &mut Self { self.show_data_table = v; self }
}

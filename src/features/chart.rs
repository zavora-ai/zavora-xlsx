use crate::utility::{ColNum, RowNum};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartType { Bar, Column, Line, Pie, Scatter, Area, Doughnut, Radar, Stock }

#[derive(Debug, Clone, Copy)]
pub enum LegendPosition { Top, Bottom, Left, Right, None }

#[derive(Debug, Clone, Copy)]
pub enum TrendlineType { Linear, Exponential, Polynomial(u8), Power, Logarithmic, MovingAverage(u8) }

#[derive(Debug, Clone, Copy)]
pub enum MarkerType { None, Circle, Diamond, Square, Triangle, Star, Plus, X }

#[derive(Debug, Clone)]
pub struct ChartSeries {
    pub(crate) values: String,
    pub(crate) categories: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) secondary_axis: bool,
    pub(crate) data_labels: bool,
    pub(crate) trendline: Option<TrendlineType>,
    pub(crate) trendline_display_rsquared: bool,
    pub(crate) trendline_display_equation: bool,
    pub(crate) chart_type_override: Option<ChartType>,
    pub(crate) marker: Option<MarkerType>,
    pub(crate) marker_size: Option<u8>,
    pub(crate) color: Option<[u8; 3]>,
    pub(crate) point_colors: Vec<(usize, [u8; 3])>,
}

impl ChartSeries {
    pub fn new() -> Self {
        Self { values: String::new(), categories: None, name: None,
            secondary_axis: false, data_labels: false,
            trendline: None, trendline_display_rsquared: false, trendline_display_equation: false,
            chart_type_override: None, marker: None, marker_size: None,
            color: None, point_colors: Vec::new() }
    }
    pub fn set_values(&mut self, range: &str) -> &mut Self { self.values = range.into(); self }
    pub fn set_categories(&mut self, range: &str) -> &mut Self { self.categories = Some(range.into()); self }
    pub fn set_name(&mut self, name: &str) -> &mut Self { self.name = Some(name.into()); self }
    pub fn set_secondary_axis(&mut self, v: bool) -> &mut Self { self.secondary_axis = v; self }
    pub fn set_data_labels(&mut self, v: bool) -> &mut Self { self.data_labels = v; self }
    pub fn set_trendline(&mut self, t: TrendlineType) -> &mut Self { self.trendline = Some(t); self }
    pub fn set_trendline_display_rsquared(&mut self, v: bool) -> &mut Self { self.trendline_display_rsquared = v; self }
    pub fn set_trendline_display_equation(&mut self, v: bool) -> &mut Self { self.trendline_display_equation = v; self }
    pub fn set_chart_type(&mut self, ct: ChartType) -> &mut Self { self.chart_type_override = Some(ct); self }
    pub fn set_marker(&mut self, m: MarkerType) -> &mut Self { self.marker = Some(m); self }
    pub fn set_marker_size(&mut self, size: u8) -> &mut Self { self.marker_size = Some(size); self }
    /// Set the series fill color (hex RGB).
    pub fn set_color(&mut self, c: impl crate::format::IntoColor) -> &mut Self { self.color = Some(c.into_color().to_rgb()); self }
    /// Set a specific data point's color by index.
    pub fn set_point_color(&mut self, index: usize, c: impl crate::format::IntoColor) -> &mut Self {
        self.point_colors.push((index, c.into_color().to_rgb())); self
    }
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
    // Axis control
    pub(crate) y_axis_min: Option<f64>,
    pub(crate) y_axis_max: Option<f64>,
    pub(crate) y_axis_log_base: Option<f64>,
    pub(crate) x_axis_reverse: bool,
    pub(crate) y_axis_reverse: bool,
    // Pivot chart source
    pub(crate) pivot_source: Option<PivotChartSource>,
    /// Pre-computed pivot chart series (populated at save time from pivot cache)
    pub(crate) pivot_series: Vec<PivotChartSeriesData>,
}

/// Pre-computed series data for a pivot chart, generated from pivot cache at save time.
#[derive(Debug, Clone)]
pub struct PivotChartSeriesData {
    pub(crate) name: String,
    pub(crate) categories: Vec<String>,
    pub(crate) values: Vec<f64>,
}

/// Links a chart to a pivot table, making it a pivot chart.
#[derive(Debug, Clone)]
pub struct PivotChartSource {
    /// Name of the pivot table (must match PivotTable::name)
    pub(crate) pivot_table_name: String,
    /// Sheet where the pivot table lives
    pub(crate) sheet_name: String,
    /// Show filter drop zone on chart
    pub(crate) show_drop_zone_filter: bool,
    /// Show category (axis) drop zone on chart
    pub(crate) show_drop_zone_categories: bool,
    /// Show data (values) drop zone on chart
    pub(crate) show_drop_zone_data: bool,
    /// Show series (legend) drop zone on chart
    pub(crate) show_drop_zone_series: bool,
    /// Show expand/collapse buttons on chart
    pub(crate) show_expand_collapse: bool,
}

impl Chart {
    pub fn new(chart_type: ChartType) -> Self {
        Self {
            chart_type, series: Vec::new(), title: None,
            x_axis_name: None, y_axis_name: None, y2_axis_name: None,
            legend_pos: LegendPosition::Bottom,
            width: 480, height: 288, row: 0, col: 0,
            x_offset: 0, y_offset: 0, show_data_table: false,
            y_axis_min: None, y_axis_max: None, y_axis_log_base: None,
            x_axis_reverse: false, y_axis_reverse: false,
            pivot_source: None,
            pivot_series: Vec::new(),
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
    pub fn set_y_axis_min(&mut self, min: f64) -> &mut Self { self.y_axis_min = Some(min); self }
    pub fn set_y_axis_max(&mut self, max: f64) -> &mut Self { self.y_axis_max = Some(max); self }
    pub fn set_y_axis_log_base(&mut self, base: f64) -> &mut Self { self.y_axis_log_base = Some(base); self }
    pub fn set_x_axis_reverse(&mut self) -> &mut Self { self.x_axis_reverse = true; self }
    pub fn set_y_axis_reverse(&mut self) -> &mut Self { self.y_axis_reverse = true; self }

    /// Link this chart to a pivot table, making it a pivot chart.
    /// The pivot table must exist on the specified sheet.
    /// Drop zones and expand/collapse buttons are enabled by default.
    pub fn set_pivot_source(&mut self, pivot_table_name: &str, sheet_name: &str) -> &mut Self {
        self.pivot_source = Some(PivotChartSource {
            pivot_table_name: pivot_table_name.into(),
            sheet_name: sheet_name.into(),
            show_drop_zone_filter: true,
            show_drop_zone_categories: true,
            show_drop_zone_data: true,
            show_drop_zone_series: true,
            show_expand_collapse: true,
        });
        self
    }

    /// Returns true if this chart is linked to a pivot table.
    pub fn is_pivot_chart(&self) -> bool { self.pivot_source.is_some() }
}

impl PivotChartSource {
    pub fn set_show_drop_zones(&mut self, filter: bool, categories: bool, data: bool, series: bool) -> &mut Self {
        self.show_drop_zone_filter = filter;
        self.show_drop_zone_categories = categories;
        self.show_drop_zone_data = data;
        self.show_drop_zone_series = series;
        self
    }
    pub fn set_show_expand_collapse(&mut self, v: bool) -> &mut Self {
        self.show_expand_collapse = v;
        self
    }
}

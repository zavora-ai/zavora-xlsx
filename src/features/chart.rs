use crate::utility::{ColNum, RowNum};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartType {
    Bar,
    Column,
    Line,
    Pie,
    Scatter,
    Area,
    Doughnut,
    Radar,
    Stock,
    Bubble,
    // 3D variants
    Column3D,
    Bar3D,
    Line3D,
    Pie3D,
    Area3D,
    // Surface
    Surface,
    WireframeSurface,
    // Map
    Map,
}

/// Granularity level for map charts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapLevel {
    /// Region-level (e.g., states/provinces)
    Region,
    /// Country-level
    Country,
}

/// 3D view settings for 3D chart types and surface charts.
#[derive(Debug, Clone, Copy)]
pub struct View3D {
    pub rot_x: i16,
    pub rot_y: i16,
    pub perspective: u8,
    pub right_angle_axes: bool,
}

impl Default for View3D {
    fn default() -> Self {
        Self {
            rot_x: 15,
            rot_y: 20,
            perspective: 30,
            right_angle_axes: true,
        }
    }
}

/// Tick mark style for axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickMark {
    None,
    Inside,
    Outside,
    Cross,
}

/// Gridline style for axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridlineStyle {
    None,
    Solid,
    Dash,
    Dot,
}

/// Axis formatting options.
#[derive(Debug, Clone)]
pub struct AxisFormat {
    pub num_format: Option<String>,
    pub font_name: Option<String>,
    pub font_size: Option<f64>,
    pub font_color: Option<[u8; 3]>,
    pub font_bold: bool,
    pub major_tick_mark: Option<TickMark>,
    pub minor_tick_mark: Option<TickMark>,
    pub major_gridlines: bool,
    pub minor_gridlines: bool,
    pub gridline_color: Option<[u8; 3]>,
    pub gridline_width: Option<f64>,
}

impl Default for AxisFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl AxisFormat {
    pub fn new() -> Self {
        Self {
            num_format: None,
            font_name: None,
            font_size: None,
            font_color: None,
            font_bold: false,
            major_tick_mark: None,
            minor_tick_mark: None,
            major_gridlines: false,
            minor_gridlines: false,
            gridline_color: None,
            gridline_width: None,
        }
    }
    pub fn set_num_format(&mut self, fmt: &str) -> &mut Self {
        self.num_format = Some(fmt.into());
        self
    }
    pub fn set_font_name(&mut self, font: &str) -> &mut Self {
        self.font_name = Some(font.into());
        self
    }
    pub fn set_font_size(&mut self, size: f64) -> &mut Self {
        self.font_size = Some(size);
        self
    }
    pub fn set_font_color(&mut self, rgb: [u8; 3]) -> &mut Self {
        self.font_color = Some(rgb);
        self
    }
    pub fn set_font_bold(&mut self, bold: bool) -> &mut Self {
        self.font_bold = bold;
        self
    }
    pub fn set_major_tick_mark(&mut self, tm: TickMark) -> &mut Self {
        self.major_tick_mark = Some(tm);
        self
    }
    pub fn set_minor_tick_mark(&mut self, tm: TickMark) -> &mut Self {
        self.minor_tick_mark = Some(tm);
        self
    }
    pub fn set_major_gridlines(&mut self, v: bool) -> &mut Self {
        self.major_gridlines = v;
        self
    }
    pub fn set_minor_gridlines(&mut self, v: bool) -> &mut Self {
        self.minor_gridlines = v;
        self
    }
    pub fn set_gridline_color(&mut self, rgb: [u8; 3]) -> &mut Self {
        self.gridline_color = Some(rgb);
        self
    }
    pub fn set_gridline_width(&mut self, width: f64) -> &mut Self {
        self.gridline_width = Some(width);
        self
    }

    // Legacy compatibility aliases
    /// Alias for set_font_name.
    pub fn set_font(&mut self, font: &str) -> &mut Self {
        self.font_name = Some(font.into());
        self
    }
    /// Alias for set_major_tick_mark.
    pub fn set_tick_marks(&mut self, tm: TickMark) -> &mut Self {
        self.major_tick_mark = Some(tm);
        self
    }
    /// Alias for set_major_gridlines with GridlineStyle mapping.
    pub fn set_gridline_style(&mut self, gs: GridlineStyle) -> &mut Self {
        match gs {
            GridlineStyle::None => {
                self.major_gridlines = false;
            }
            _ => {
                self.major_gridlines = true;
            }
        }
        self
    }
}

/// Plot area formatting options.
#[derive(Debug, Clone)]
pub struct PlotAreaFormat {
    pub fill: Option<[u8; 3]>,
    pub border: Option<[u8; 3]>,
    pub gradient: Option<Vec<([u8; 3], f64)>>, // (color, position) pairs
}

impl Default for PlotAreaFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl PlotAreaFormat {
    pub fn new() -> Self {
        Self {
            fill: None,
            border: None,
            gradient: None,
        }
    }
    pub fn set_fill(&mut self, rgb: [u8; 3]) -> &mut Self {
        self.fill = Some(rgb);
        self
    }
    pub fn set_border(&mut self, rgb: [u8; 3]) -> &mut Self {
        self.border = Some(rgb);
        self
    }
    pub fn set_gradient(&mut self, stops: Vec<([u8; 3], f64)>) -> &mut Self {
        self.gradient = Some(stops);
        self
    }
}

/// Dash style for series lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashStyle {
    Solid,
    Dash,
    Dot,
    DashDot,
    LongDash,
    LongDashDot,
}

/// Error bar type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorBarType {
    Both,
    Plus,
    Minus,
}

/// Error bar value type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorBarValueType {
    FixedValue,
    Percentage,
    StandardDeviation,
    StandardError,
}

/// Error bar configuration for a chart series.
#[derive(Debug, Clone)]
pub struct ErrorBar {
    pub bar_type: ErrorBarType,
    pub value_type: ErrorBarValueType,
    pub value: f64,
}

impl ErrorBar {
    pub fn new(bar_type: ErrorBarType, value_type: ErrorBarValueType, value: f64) -> Self {
        Self {
            bar_type,
            value_type,
            value,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LegendPosition {
    Top,
    Bottom,
    Left,
    Right,
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum TrendlineType {
    Linear,
    Exponential,
    Polynomial(u8),
    Power,
    Logarithmic,
    MovingAverage(u8),
}

#[derive(Debug, Clone, Copy)]
pub enum MarkerType {
    None,
    Circle,
    Diamond,
    Square,
    Triangle,
    Star,
    Plus,
    X,
}

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
    pub(crate) bubble_sizes: Option<String>,
    // Series formatting (Task 37)
    pub(crate) line_width: Option<f64>,
    pub(crate) dash_style: Option<DashStyle>,
    pub(crate) gradient: Option<Vec<([u8; 3], f64)>>,
    pub(crate) error_bars: Option<ErrorBar>,
}

impl Default for ChartSeries {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartSeries {
    pub fn new() -> Self {
        Self {
            values: String::new(),
            categories: None,
            name: None,
            secondary_axis: false,
            data_labels: false,
            trendline: None,
            trendline_display_rsquared: false,
            trendline_display_equation: false,
            chart_type_override: None,
            marker: None,
            marker_size: None,
            color: None,
            point_colors: Vec::new(),
            bubble_sizes: None,
            line_width: None,
            dash_style: None,
            gradient: None,
            error_bars: None,
        }
    }
    pub fn set_values(&mut self, range: &str) -> &mut Self {
        self.values = range.into();
        self
    }
    pub fn set_categories(&mut self, range: &str) -> &mut Self {
        self.categories = Some(range.into());
        self
    }
    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.into());
        self
    }
    pub fn set_secondary_axis(&mut self, v: bool) -> &mut Self {
        self.secondary_axis = v;
        self
    }
    pub fn set_data_labels(&mut self, v: bool) -> &mut Self {
        self.data_labels = v;
        self
    }
    pub fn set_trendline(&mut self, t: TrendlineType) -> &mut Self {
        self.trendline = Some(t);
        self
    }
    pub fn set_trendline_display_rsquared(&mut self, v: bool) -> &mut Self {
        self.trendline_display_rsquared = v;
        self
    }
    pub fn set_trendline_display_equation(&mut self, v: bool) -> &mut Self {
        self.trendline_display_equation = v;
        self
    }
    pub fn set_chart_type(&mut self, ct: ChartType) -> &mut Self {
        self.chart_type_override = Some(ct);
        self
    }
    pub fn set_marker(&mut self, m: MarkerType) -> &mut Self {
        self.marker = Some(m);
        self
    }
    pub fn set_marker_size(&mut self, size: u8) -> &mut Self {
        self.marker_size = Some(size);
        self
    }
    /// Set the series fill color (hex RGB).
    pub fn set_color(&mut self, c: impl crate::format::IntoColor) -> &mut Self {
        self.color = Some(c.into_color().to_rgb());
        self
    }
    /// Set a specific data point's color by index.
    pub fn set_point_color(&mut self, index: usize, c: impl crate::format::IntoColor) -> &mut Self {
        self.point_colors.push((index, c.into_color().to_rgb()));
        self
    }
    /// Set the bubble sizes range reference for bubble charts.
    pub fn set_bubble_sizes(&mut self, range: &str) -> &mut Self {
        self.bubble_sizes = Some(range.into());
        self
    }
    /// Set the line width in points.
    pub fn set_line_width(&mut self, width: f64) -> &mut Self {
        self.line_width = Some(width);
        self
    }
    /// Set the dash style for the series line.
    pub fn set_dash_style(&mut self, style: DashStyle) -> &mut Self {
        self.dash_style = Some(style);
        self
    }
    /// Set a gradient fill for the series (list of (color, position) pairs).
    pub fn set_gradient(&mut self, stops: Vec<([u8; 3], f64)>) -> &mut Self {
        self.gradient = Some(stops);
        self
    }
    /// Set error bars on this series.
    pub fn set_error_bars(&mut self, error_bar: ErrorBar) -> &mut Self {
        self.error_bars = Some(error_bar);
        self
    }

    // ── Read accessors ──

    /// Returns the values range reference.
    pub fn values(&self) -> &str {
        &self.values
    }
    /// Returns the categories range reference, if set.
    pub fn categories(&self) -> Option<&str> {
        self.categories.as_deref()
    }
    /// Returns the series name, if set.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
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
    // 3D view settings (Task 33)
    pub(crate) view3d: Option<View3D>,
    // Chart style (Task 35)
    pub(crate) style: Option<u8>,
    // Axis formatting (Task 36)
    pub(crate) x_axis_format: Option<AxisFormat>,
    pub(crate) y_axis_format: Option<AxisFormat>,
    // Plot area formatting (Task 37)
    pub(crate) plot_area_format: Option<PlotAreaFormat>,
    // Map chart settings (Task 32)
    pub(crate) map_level: MapLevel,
    // Chart accessories (Task 38)
    pub(crate) drop_lines: bool,
    pub(crate) high_low_lines: bool,
    // Accessibility metadata (Task 82)
    pub(crate) alt_text: Option<(String, String)>,
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
            chart_type,
            series: Vec::new(),
            title: None,
            x_axis_name: None,
            y_axis_name: None,
            y2_axis_name: None,
            legend_pos: LegendPosition::Bottom,
            width: 480,
            height: 288,
            row: 0,
            col: 0,
            x_offset: 0,
            y_offset: 0,
            show_data_table: false,
            y_axis_min: None,
            y_axis_max: None,
            y_axis_log_base: None,
            x_axis_reverse: false,
            y_axis_reverse: false,
            pivot_source: None,
            pivot_series: Vec::new(),
            view3d: None,
            style: None,
            x_axis_format: None,
            y_axis_format: None,
            plot_area_format: None,
            map_level: MapLevel::Country,
            drop_lines: false,
            high_low_lines: false,
            alt_text: None,
        }
    }

    pub fn add_series(&mut self) -> &mut ChartSeries {
        self.series.push(ChartSeries::new());
        self.series.last_mut().unwrap()
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self {
        self.title = Some(title.into());
        self
    }
    pub fn set_x_axis_name(&mut self, name: &str) -> &mut Self {
        self.x_axis_name = Some(name.into());
        self
    }
    pub fn set_y_axis_name(&mut self, name: &str) -> &mut Self {
        self.y_axis_name = Some(name.into());
        self
    }
    pub fn set_y2_axis_name(&mut self, name: &str) -> &mut Self {
        self.y2_axis_name = Some(name.into());
        self
    }
    pub fn set_legend_position(&mut self, pos: LegendPosition) -> &mut Self {
        self.legend_pos = pos;
        self
    }
    pub fn set_width(&mut self, w: u32) -> &mut Self {
        self.width = w;
        self
    }
    pub fn set_height(&mut self, h: u32) -> &mut Self {
        self.height = h;
        self
    }
    pub fn show_data_table(&mut self, v: bool) -> &mut Self {
        self.show_data_table = v;
        self
    }
    pub fn set_y_axis_min(&mut self, min: f64) -> &mut Self {
        self.y_axis_min = Some(min);
        self
    }
    pub fn set_y_axis_max(&mut self, max: f64) -> &mut Self {
        self.y_axis_max = Some(max);
        self
    }
    pub fn set_y_axis_log_base(&mut self, base: f64) -> &mut Self {
        self.y_axis_log_base = Some(base);
        self
    }
    pub fn set_x_axis_reverse(&mut self) -> &mut Self {
        self.x_axis_reverse = true;
        self
    }
    pub fn set_y_axis_reverse(&mut self) -> &mut Self {
        self.y_axis_reverse = true;
        self
    }

    /// Set 3D view settings for 3D chart types.
    pub fn set_view3d(&mut self, view: View3D) -> &mut Self {
        self.view3d = Some(view);
        self
    }

    /// Set the chart style (1-48).
    pub fn set_style(&mut self, n: u8) -> &mut Self {
        self.style = Some(n);
        self
    }

    /// Set X axis formatting.
    pub fn set_x_axis_format(&mut self, fmt: AxisFormat) -> &mut Self {
        self.x_axis_format = Some(fmt);
        self
    }

    /// Set Y axis formatting.
    pub fn set_y_axis_format(&mut self, fmt: AxisFormat) -> &mut Self {
        self.y_axis_format = Some(fmt);
        self
    }

    /// Set plot area formatting.
    pub fn set_plot_area_format(&mut self, fmt: PlotAreaFormat) -> &mut Self {
        self.plot_area_format = Some(fmt);
        self
    }

    /// Set map chart granularity level.
    pub fn set_map_level(&mut self, level: MapLevel) -> &mut Self {
        self.map_level = level;
        self
    }

    /// Enable drop lines (for line/area charts).
    pub fn set_drop_lines(&mut self, v: bool) -> &mut Self {
        self.drop_lines = v;
        self
    }

    /// Enable high-low lines (for line/stock charts).
    pub fn set_high_low_lines(&mut self, v: bool) -> &mut Self {
        self.high_low_lines = v;
        self
    }

    /// Set accessibility alt text (title and description) for this chart.
    ///
    /// The title and description are serialized as `title` and `descr`
    /// attributes on the `<xdr:cNvPr>` element in the drawing XML.
    pub fn set_alt_text(&mut self, title: &str, description: &str) -> &mut Self {
        self.alt_text = Some((title.to_string(), description.to_string()));
        self
    }

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
    pub fn is_pivot_chart(&self) -> bool {
        self.pivot_source.is_some()
    }

    // ── Read accessors ──

    /// Returns the chart type.
    pub fn chart_type(&self) -> ChartType {
        self.chart_type
    }
    /// Returns the chart series.
    pub fn series(&self) -> &[ChartSeries] {
        &self.series
    }
    /// Returns the chart title, if set.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
    /// Returns the X axis name, if set.
    pub fn x_axis_name(&self) -> Option<&str> {
        self.x_axis_name.as_deref()
    }
    /// Returns the Y axis name, if set.
    pub fn y_axis_name(&self) -> Option<&str> {
        self.y_axis_name.as_deref()
    }
    /// Returns the legend position.
    pub fn legend_position(&self) -> LegendPosition {
        self.legend_pos
    }
    /// Returns the chart style, if set.
    pub fn style(&self) -> Option<u8> {
        self.style
    }

    /// Returns the alt text (title, description) if set.
    pub fn alt_text(&self) -> Option<(&str, &str)> {
        self.alt_text
            .as_ref()
            .map(|(t, d)| (t.as_str(), d.as_str()))
    }
}

impl PivotChartSource {
    pub fn set_show_drop_zones(
        &mut self,
        filter: bool,
        categories: bool,
        data: bool,
        series: bool,
    ) -> &mut Self {
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

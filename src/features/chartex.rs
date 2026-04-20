//! ChartEx chart types (Excel 2016+ cx: namespace).
//!
//! These chart types use the `cx:` namespace and are serialized as chartEx XML parts.
//! Includes: Waterfall, Funnel, Sunburst, Histogram, Pareto, BoxWhisker.

use crate::utility::{ColNum, RowNum};

/// Data point categorization for waterfall charts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterfallPointType {
    /// An increase (positive) value
    Increase,
    /// A decrease (negative) value
    Decrease,
    /// A subtotal or total bar
    Total,
}

/// Waterfall chart (Excel 2016+ ChartEx format).
#[derive(Debug, Clone)]
pub struct WaterfallChart {
    pub(crate) title: Option<String>,
    pub(crate) categories: Vec<String>,
    pub(crate) values: Vec<f64>,
    pub(crate) point_types: Vec<WaterfallPointType>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) row: RowNum,
    pub(crate) col: ColNum,
    pub(crate) series_name: Option<String>,
}

impl WaterfallChart {
    pub fn new() -> Self {
        Self {
            title: None, categories: Vec::new(), values: Vec::new(),
            point_types: Vec::new(), width: 480, height: 320, row: 0, col: 0,
            series_name: None,
        }
    }

    /// Add a data point with category, value, and point type (increase, decrease, or total).
    pub fn add_point(&mut self, category: &str, value: f64, point_type: WaterfallPointType) -> &mut Self {
        self.categories.push(category.to_string());
        self.values.push(value);
        self.point_types.push(point_type);
        self
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self { self.title = Some(title.to_string()); self }
    pub fn set_series_name(&mut self, name: &str) -> &mut Self { self.series_name = Some(name.to_string()); self }
    pub fn set_width(&mut self, w: u32) -> &mut Self { self.width = w; self }
    pub fn set_height(&mut self, h: u32) -> &mut Self { self.height = h; self }
}

/// Funnel chart (Excel 2016+ ChartEx format).
#[derive(Debug, Clone)]
pub struct FunnelChart {
    pub(crate) title: Option<String>,
    pub(crate) categories: Vec<String>,
    pub(crate) values: Vec<f64>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) row: RowNum,
    pub(crate) col: ColNum,
    pub(crate) series_name: Option<String>,
}

impl FunnelChart {
    pub fn new() -> Self {
        Self {
            title: None, categories: Vec::new(), values: Vec::new(),
            width: 480, height: 320, row: 0, col: 0, series_name: None,
        }
    }

    /// Add a data point with category name and value.
    pub fn add_point(&mut self, category: &str, value: f64) -> &mut Self {
        self.categories.push(category.to_string());
        self.values.push(value);
        self
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self { self.title = Some(title.to_string()); self }
    pub fn set_series_name(&mut self, name: &str) -> &mut Self { self.series_name = Some(name.to_string()); self }
    pub fn set_width(&mut self, w: u32) -> &mut Self { self.width = w; self }
    pub fn set_height(&mut self, h: u32) -> &mut Self { self.height = h; self }
}

/// A level of hierarchy data for sunburst charts.
#[derive(Debug, Clone)]
pub struct SunburstLevel {
    pub(crate) labels: Vec<String>,
}

/// Sunburst chart (Excel 2016+ ChartEx format).
/// Supports multi-level hierarchy (at least 3 levels).
#[derive(Debug, Clone)]
pub struct SunburstChart {
    pub(crate) title: Option<String>,
    pub(crate) levels: Vec<SunburstLevel>,
    pub(crate) values: Vec<f64>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) row: RowNum,
    pub(crate) col: ColNum,
    pub(crate) series_name: Option<String>,
}

impl SunburstChart {
    pub fn new() -> Self {
        Self {
            title: None, levels: Vec::new(), values: Vec::new(),
            width: 480, height: 320, row: 0, col: 0, series_name: None,
        }
    }

    /// Add a hierarchy level with labels. Each level corresponds to a ring in the sunburst.
    /// The first level added is the innermost ring.
    pub fn add_level(&mut self, labels: &[&str]) -> &mut Self {
        self.levels.push(SunburstLevel {
            labels: labels.iter().map(|s| s.to_string()).collect(),
        });
        self
    }

    /// Set the values (sizes) for the leaf-level data points.
    pub fn set_values(&mut self, values: &[f64]) -> &mut Self {
        self.values = values.to_vec();
        self
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self { self.title = Some(title.to_string()); self }
    pub fn set_series_name(&mut self, name: &str) -> &mut Self { self.series_name = Some(name.to_string()); self }
    pub fn set_width(&mut self, w: u32) -> &mut Self { self.width = w; self }
    pub fn set_height(&mut self, h: u32) -> &mut Self { self.height = h; self }
}

/// Histogram chart (Excel 2016+ ChartEx format).
#[derive(Debug, Clone)]
pub struct HistogramChart {
    pub(crate) title: Option<String>,
    pub(crate) values: Vec<f64>,
    pub(crate) bin_count: Option<u32>,
    pub(crate) bin_width: Option<f64>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) row: RowNum,
    pub(crate) col: ColNum,
    pub(crate) series_name: Option<String>,
    pub(crate) is_pareto: bool,
}

impl HistogramChart {
    pub fn new() -> Self {
        Self {
            title: None, values: Vec::new(), bin_count: None, bin_width: None,
            width: 480, height: 320, row: 0, col: 0, series_name: None,
            is_pareto: false,
        }
    }

    /// Create a Pareto chart (histogram with cumulative line).
    pub fn pareto() -> Self {
        let mut chart = Self::new();
        chart.is_pareto = true;
        chart
    }

    /// Set the raw data values for the histogram.
    pub fn set_values(&mut self, values: &[f64]) -> &mut Self {
        self.values = values.to_vec();
        self
    }

    /// Set the number of bins.
    pub fn set_bin_count(&mut self, count: u32) -> &mut Self { self.bin_count = Some(count); self }

    /// Set the bin width.
    pub fn set_bin_width(&mut self, width: f64) -> &mut Self { self.bin_width = Some(width); self }

    pub fn set_title(&mut self, title: &str) -> &mut Self { self.title = Some(title.to_string()); self }
    pub fn set_series_name(&mut self, name: &str) -> &mut Self { self.series_name = Some(name.to_string()); self }
    pub fn set_width(&mut self, w: u32) -> &mut Self { self.width = w; self }
    pub fn set_height(&mut self, h: u32) -> &mut Self { self.height = h; self }
}

/// Box and Whisker chart (Excel 2016+ ChartEx format).
#[derive(Debug, Clone)]
pub struct BoxWhiskerChart {
    pub(crate) title: Option<String>,
    pub(crate) categories: Vec<String>,
    pub(crate) data_sets: Vec<Vec<f64>>,
    pub(crate) show_outliers: bool,
    pub(crate) show_mean_markers: bool,
    pub(crate) show_inner_points: bool,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) row: RowNum,
    pub(crate) col: ColNum,
    pub(crate) series_name: Option<String>,
}

impl BoxWhiskerChart {
    pub fn new() -> Self {
        Self {
            title: None, categories: Vec::new(), data_sets: Vec::new(),
            show_outliers: true, show_mean_markers: true, show_inner_points: false,
            width: 480, height: 320, row: 0, col: 0, series_name: None,
        }
    }

    /// Add a data set (one box) with a category label and its data points.
    pub fn add_data_set(&mut self, category: &str, values: &[f64]) -> &mut Self {
        self.categories.push(category.to_string());
        self.data_sets.push(values.to_vec());
        self
    }

    /// Whether to show outlier points.
    pub fn set_show_outliers(&mut self, v: bool) -> &mut Self { self.show_outliers = v; self }

    /// Whether to show mean markers.
    pub fn set_show_mean_markers(&mut self, v: bool) -> &mut Self { self.show_mean_markers = v; self }

    /// Whether to show inner (non-outlier) data points.
    pub fn set_show_inner_points(&mut self, v: bool) -> &mut Self { self.show_inner_points = v; self }

    pub fn set_title(&mut self, title: &str) -> &mut Self { self.title = Some(title.to_string()); self }
    pub fn set_series_name(&mut self, name: &str) -> &mut Self { self.series_name = Some(name.to_string()); self }
    pub fn set_width(&mut self, w: u32) -> &mut Self { self.width = w; self }
    pub fn set_height(&mut self, h: u32) -> &mut Self { self.height = h; self }
}

/// Unified enum for all ChartEx chart types (used for storage in Worksheet).
/// Map chart (Excel 2016+ ChartEx format, uses Bing Maps).
///
/// Map charts display geographic data as a filled map. Excel fetches
/// geography data from Bing Maps when the file is opened.
#[derive(Debug, Clone)]
pub struct MapChart {
    pub(crate) title: Option<String>,
    pub(crate) categories: Vec<String>,
    pub(crate) values: Vec<f64>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) row: RowNum,
    pub(crate) col: ColNum,
    pub(crate) series_name: Option<String>,
}

impl MapChart {
    pub fn new() -> Self {
        Self {
            title: None, categories: Vec::new(), values: Vec::new(),
            width: 480, height: 320, row: 0, col: 0, series_name: None,
        }
    }

    /// Add a data point with a geographic name (country/region) and value.
    pub fn add_point(&mut self, location: &str, value: f64) -> &mut Self {
        self.categories.push(location.to_string());
        self.values.push(value);
        self
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self { self.title = Some(title.to_string()); self }
    pub fn set_series_name(&mut self, name: &str) -> &mut Self { self.series_name = Some(name.to_string()); self }
    pub fn set_width(&mut self, w: u32) -> &mut Self { self.width = w; self }
    pub fn set_height(&mut self, h: u32) -> &mut Self { self.height = h; self }
}

#[derive(Debug, Clone)]
pub enum ChartExChart {
    Waterfall(WaterfallChart),
    Funnel(FunnelChart),
    Sunburst(SunburstChart),
    Histogram(HistogramChart),
    BoxWhisker(BoxWhiskerChart),
    Map(MapChart),
}

impl ChartExChart {
    pub fn row(&self) -> RowNum {
        match self {
            Self::Waterfall(c) => c.row,
            Self::Funnel(c) => c.row,
            Self::Sunburst(c) => c.row,
            Self::Histogram(c) => c.row,
            Self::BoxWhisker(c) => c.row,
            Self::Map(c) => c.row,
        }
    }
    pub fn col(&self) -> ColNum {
        match self {
            Self::Waterfall(c) => c.col,
            Self::Funnel(c) => c.col,
            Self::Sunburst(c) => c.col,
            Self::Histogram(c) => c.col,
            Self::BoxWhisker(c) => c.col,
            Self::Map(c) => c.col,
        }
    }
    pub fn width(&self) -> u32 {
        match self {
            Self::Waterfall(c) => c.width,
            Self::Funnel(c) => c.width,
            Self::Sunburst(c) => c.width,
            Self::Histogram(c) => c.width,
            Self::BoxWhisker(c) => c.width,
            Self::Map(c) => c.width,
        }
    }
    pub fn height(&self) -> u32 {
        match self {
            Self::Waterfall(c) => c.height,
            Self::Funnel(c) => c.height,
            Self::Sunburst(c) => c.height,
            Self::Histogram(c) => c.height,
            Self::BoxWhisker(c) => c.height,
            Self::Map(c) => c.height,
        }
    }
}

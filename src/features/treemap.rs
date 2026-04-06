/// Treemap chart (Excel 2016+ ChartEx format).
///
/// Uses the `cx:` namespace (chartEx) which is different from standard `c:` charts.
#[derive(Debug, Clone)]
pub struct TreemapChart {
    pub(crate) title: Option<String>,
    pub(crate) categories: Vec<String>,
    pub(crate) values: Vec<f64>,
    pub(crate) colors: Vec<Option<[u8; 3]>>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) row: crate::utility::RowNum,
    pub(crate) col: crate::utility::ColNum,
    pub(crate) series_name: Option<String>,
    /// Cell range for categories (e.g. "Sheet1!$A$2:$A$8")
    pub(crate) cat_range: Option<String>,
    /// Cell range for values (e.g. "Sheet1!$B$2:$B$8")
    pub(crate) val_range: Option<String>,
}

impl TreemapChart {
    pub fn new() -> Self {
        Self {
            title: None, categories: Vec::new(), values: Vec::new(),
            colors: Vec::new(), width: 480, height: 320, row: 0, col: 0,
            series_name: None, cat_range: None, val_range: None,
        }
    }

    /// Add a data point with category name and value.
    pub fn add_point(&mut self, category: &str, value: f64) -> &mut Self {
        self.categories.push(category.to_string());
        self.values.push(value);
        self.colors.push(None);
        self
    }

    /// Add a data point with a specific color.
    pub fn add_point_with_color(&mut self, category: &str, value: f64, color: impl crate::format::IntoColor) -> &mut Self {
        self.categories.push(category.to_string());
        self.values.push(value);
        self.colors.push(Some(color.into_color().to_rgb()));
        self
    }

    /// Set cell ranges for categories and values (for formula-linked charts).
    pub fn set_ranges(&mut self, cat_range: &str, val_range: &str) -> &mut Self {
        self.cat_range = Some(cat_range.to_string());
        self.val_range = Some(val_range.to_string());
        self
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self { self.title = Some(title.to_string()); self }
    pub fn set_series_name(&mut self, name: &str) -> &mut Self { self.series_name = Some(name.to_string()); self }
    pub fn set_width(&mut self, w: u32) -> &mut Self { self.width = w; self }
    pub fn set_height(&mut self, h: u32) -> &mut Self { self.height = h; self }
}

// Feature methods: charts, images, tables, CF, DV, sparklines, comments, hyperlinks, protection

use crate::features::chart::Chart;
use crate::features::conditional::{ConditionalFormat, StoredCf};
use crate::features::image::Image;
use crate::features::sparkline::Sparkline;
use crate::features::table::Table;
use crate::features::validation::DataValidation;
use crate::utility::{ColNum, RowNum};
use super::Worksheet;
use super::types::{Comment, Hyperlink, SheetProtection, PrintSettings, hash_password};

impl Worksheet {
    pub fn insert_chart(&mut self, row: RowNum, col: ColNum, chart: &Chart) -> crate::Result<&mut Self> {
        let mut c = chart.clone(); c.row = row; c.col = col;
        self.charts.push(c); self.dirty = true; Ok(self)
    }

    pub fn insert_chart_with_offset(&mut self, row: RowNum, col: ColNum, chart: &Chart, x_px: u32, y_px: u32) -> crate::Result<&mut Self> {
        let mut c = chart.clone(); c.row = row; c.col = col; c.x_offset = x_px; c.y_offset = y_px;
        self.charts.push(c); self.dirty = true; Ok(self)
    }

    pub fn insert_image(&mut self, row: RowNum, col: ColNum, image: &Image) -> crate::Result<&mut Self> {
        let mut img = image.clone(); img.row = row; img.col = col;
        self.images.push(img); self.dirty = true; Ok(self)
    }

    pub fn add_table(&mut self, first_row: RowNum, first_col: ColNum, last_row: RowNum, last_col: ColNum, table: &Table) -> crate::Result<&mut Self> {
        let mut t = table.clone();
        t.first_row = first_row; t.first_col = first_col; t.last_row = last_row; t.last_col = last_col;
        self.tables.push(t); self.dirty = true; Ok(self)
    }

    pub fn add_conditional_format(&mut self, r1: RowNum, c1: ColNum, r2: RowNum, c2: ColNum, cf: impl ConditionalFormat + 'static) -> crate::Result<&mut Self> {
        self.conditional_formats.push(StoredCf { range: (r1, c1, r2, c2), rule: Box::new(cf), dxf_id: None });
        self.dirty = true; Ok(self)
    }

    pub fn add_data_validation(&mut self, r1: RowNum, c1: ColNum, r2: RowNum, c2: ColNum, dv: &DataValidation) -> crate::Result<&mut Self> {
        let mut v = dv.clone();
        v.first_row = r1; v.first_col = c1; v.last_row = r2; v.last_col = c2;
        self.validations.push(v); self.dirty = true; Ok(self)
    }

    pub fn add_sparkline(&mut self, row: RowNum, col: ColNum, sparkline: &Sparkline) -> crate::Result<&mut Self> {
        let mut s = sparkline.clone(); s.row = row; s.col = col;
        self.sparklines.push(s); self.dirty = true; Ok(self)
    }

    /// Add a pivot table at the given position.
    pub fn add_pivot_table(&mut self, row: RowNum, col: ColNum, pivot: &crate::features::pivot::PivotTable) -> crate::Result<&mut Self> {
        let mut pt = pivot.clone(); pt.row = row; pt.col = col;
        self.pivot_tables.push(pt); self.dirty = true; Ok(self)
    }

    /// Insert a treemap chart (Excel 2016+ ChartEx format).
    pub fn insert_treemap(&mut self, row: RowNum, col: ColNum, chart: &crate::features::treemap::TreemapChart) -> crate::Result<&mut Self> {
        let mut tc = chart.clone(); tc.row = row; tc.col = col;
        self.treemap_charts.push(tc); self.dirty = true; Ok(self)
    }

    /// Insert a waterfall chart (Excel 2016+ ChartEx format).
    pub fn insert_waterfall(&mut self, row: RowNum, col: ColNum, chart: &crate::features::chartex::WaterfallChart) -> crate::Result<&mut Self> {
        let mut c = chart.clone(); c.row = row; c.col = col;
        self.chartex_charts.push(crate::features::chartex::ChartExChart::Waterfall(c));
        self.dirty = true; Ok(self)
    }

    /// Insert a funnel chart (Excel 2016+ ChartEx format).
    pub fn insert_funnel(&mut self, row: RowNum, col: ColNum, chart: &crate::features::chartex::FunnelChart) -> crate::Result<&mut Self> {
        let mut c = chart.clone(); c.row = row; c.col = col;
        self.chartex_charts.push(crate::features::chartex::ChartExChart::Funnel(c));
        self.dirty = true; Ok(self)
    }

    /// Insert a sunburst chart (Excel 2016+ ChartEx format).
    pub fn insert_sunburst(&mut self, row: RowNum, col: ColNum, chart: &crate::features::chartex::SunburstChart) -> crate::Result<&mut Self> {
        let mut c = chart.clone(); c.row = row; c.col = col;
        self.chartex_charts.push(crate::features::chartex::ChartExChart::Sunburst(c));
        self.dirty = true; Ok(self)
    }

    /// Insert a histogram chart (Excel 2016+ ChartEx format).
    pub fn insert_histogram(&mut self, row: RowNum, col: ColNum, chart: &crate::features::chartex::HistogramChart) -> crate::Result<&mut Self> {
        let mut c = chart.clone(); c.row = row; c.col = col;
        self.chartex_charts.push(crate::features::chartex::ChartExChart::Histogram(c));
        self.dirty = true; Ok(self)
    }

    /// Insert a box & whisker chart (Excel 2016+ ChartEx format).
    pub fn insert_box_whisker(&mut self, row: RowNum, col: ColNum, chart: &crate::features::chartex::BoxWhiskerChart) -> crate::Result<&mut Self> {
        let mut c = chart.clone(); c.row = row; c.col = col;
        self.chartex_charts.push(crate::features::chartex::ChartExChart::BoxWhisker(c));
        self.dirty = true; Ok(self)
    }

    pub fn protect(&mut self) -> &mut Self {
        self.protection = Some(SheetProtection::default()); self.dirty = true; self
    }

    pub fn protect_with_password(&mut self, password: &str) -> &mut Self {
        let mut prot = SheetProtection::default();
        prot.password_hash = Some(hash_password(password));
        self.protection = Some(prot); self.dirty = true; self
    }

    pub fn unprotect_range(&mut self, name: &str, range: &str) -> &mut Self {
        let ps = self.print_settings.get_or_insert_with(PrintSettings::default);
        ps.protected_ranges.push((name.to_string(), range.to_string(), None));
        self.dirty = true; self
    }

    pub fn unprotect_range_with_password(&mut self, name: &str, range: &str, password: &str) -> &mut Self {
        let ps = self.print_settings.get_or_insert_with(PrintSettings::default);
        ps.protected_ranges.push((name.to_string(), range.to_string(), Some(hash_password(password))));
        self.dirty = true; self
    }

    pub fn write_url(&mut self, row: RowNum, col: ColNum, url: &str, text: &str) -> crate::Result<&mut Self> {
        self.ensure_deserialized(); self.dirty = true;
        self.write_string_internal(row, col, if text.is_empty() { url } else { text }, None)?;
        self.hyperlinks.push(Hyperlink { row, col, url: url.to_string(), location: None, tooltip: None });
        Ok(self)
    }

    pub fn write_internal_link(&mut self, row: RowNum, col: ColNum, location: &str, text: &str) -> crate::Result<&mut Self> {
        self.ensure_deserialized(); self.dirty = true;
        self.write_string_internal(row, col, text, None)?;
        self.hyperlinks.push(Hyperlink { row, col, url: String::new(), location: Some(location.to_string()), tooltip: None });
        Ok(self)
    }

    pub fn add_comment(&mut self, row: RowNum, col: ColNum, text: &str) -> &mut Self {
        self.comments.push(Comment { row, col, text: text.to_string(), author: "Author".to_string() });
        self.dirty = true; self
    }

    pub fn add_comment_with_author(&mut self, row: RowNum, col: ColNum, text: &str, author: &str) -> &mut Self {
        self.comments.push(Comment { row, col, text: text.to_string(), author: author.to_string() });
        self.dirty = true; self
    }
}

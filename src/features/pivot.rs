use crate::utility::{ColNum, RowNum};

/// Aggregation function for pivot table value fields.
#[derive(Debug, Clone, Copy)]
pub enum PivotAggregation {
    Sum, Count, Average, Max, Min, Product, CountNums,
    StdDev, StdDevP, Var, VarP,
}

impl PivotAggregation {
    pub(crate) fn xml_str(&self) -> &str {
        match self {
            Self::Sum => "sum", Self::Count => "count", Self::Average => "average",
            Self::Max => "max", Self::Min => "min", Self::Product => "product",
            Self::CountNums => "countNums", Self::StdDev => "stdDev",
            Self::StdDevP => "stdDevp", Self::Var => "var", Self::VarP => "varp",
        }
    }
}

/// Pivot table layout mode.
#[derive(Debug, Clone, Copy, Default)]
pub enum PivotLayout { #[default] Compact, Outline, Tabular }

/// Pivot table style.
#[derive(Debug, Clone)]
pub enum PivotStyle { Named(String) }

impl Default for PivotStyle {
    fn default() -> Self { Self::Named("PivotStyleLight16".into()) }
}

/// A value field in the pivot table.
#[derive(Debug, Clone)]
pub struct PivotValueField {
    pub(crate) source_field: String,
    pub(crate) name: String,
    pub(crate) aggregation: PivotAggregation,
    pub(crate) num_format: Option<String>,
}

/// A pivot table definition.
#[derive(Debug, Clone)]
pub struct PivotTable {
    pub(crate) name: String,
    pub(crate) source_range: String,
    pub(crate) row_fields: Vec<String>,
    pub(crate) column_fields: Vec<String>,
    pub(crate) value_fields: Vec<PivotValueField>,
    pub(crate) filter_fields: Vec<String>,
    pub(crate) calculated_fields: Vec<(String, String)>,
    pub(crate) style: PivotStyle,
    pub(crate) show_row_headers: bool,
    pub(crate) show_col_headers: bool,
    pub(crate) show_row_stripes: bool,
    pub(crate) show_col_stripes: bool,
    pub(crate) show_row_grand_total: bool,
    pub(crate) show_col_grand_total: bool,
    pub(crate) layout: PivotLayout,
    pub(crate) field_subtotals: Vec<(String, bool)>,
    // Resolved at write time
    pub(crate) row: RowNum,
    pub(crate) col: ColNum,
}

impl PivotTable {
    pub fn new(name: &str, source_range: &str) -> Self {
        Self {
            name: name.into(), source_range: source_range.into(),
            row_fields: Vec::new(), column_fields: Vec::new(),
            value_fields: Vec::new(), filter_fields: Vec::new(),
            calculated_fields: Vec::new(),
            style: PivotStyle::default(),
            show_row_headers: true, show_col_headers: true,
            show_row_stripes: false, show_col_stripes: false,
            show_row_grand_total: true, show_col_grand_total: true,
            layout: PivotLayout::default(),
            field_subtotals: Vec::new(),
            row: 0, col: 0,
        }
    }

    pub fn add_row_field(mut self, field: &str) -> Self { self.row_fields.push(field.into()); self }
    pub fn add_column_field(mut self, field: &str) -> Self { self.column_fields.push(field.into()); self }
    pub fn add_filter_field(mut self, field: &str) -> Self { self.filter_fields.push(field.into()); self }

    pub fn add_value_field(mut self, field: &str, agg: PivotAggregation) -> Self {
        let prefix = match agg {
            PivotAggregation::Sum => "Sum of", PivotAggregation::Count => "Count of",
            PivotAggregation::Average => "Average of", PivotAggregation::Max => "Max of",
            PivotAggregation::Min => "Min of", _ => "Value of",
        };
        self.value_fields.push(PivotValueField {
            source_field: field.into(), name: format!("{prefix} {field}"),
            aggregation: agg, num_format: None,
        });
        self
    }

    pub fn add_value_field_named(mut self, field: &str, agg: PivotAggregation, name: &str) -> Self {
        self.value_fields.push(PivotValueField {
            source_field: field.into(), name: name.into(),
            aggregation: agg, num_format: None,
        });
        self
    }

    pub fn add_calculated_field(mut self, name: &str, formula: &str) -> Self {
        self.calculated_fields.push((name.into(), formula.into())); self
    }

    pub fn set_style_name(mut self, name: &str) -> Self { self.style = PivotStyle::Named(name.into()); self }
    pub fn show_row_headers(mut self, v: bool) -> Self { self.show_row_headers = v; self }
    pub fn show_column_headers(mut self, v: bool) -> Self { self.show_col_headers = v; self }
    pub fn show_row_stripes(mut self, v: bool) -> Self { self.show_row_stripes = v; self }
    pub fn show_grand_totals(mut self, rows: bool, cols: bool) -> Self {
        self.show_row_grand_total = rows; self.show_col_grand_total = cols; self
    }
    pub fn set_layout(mut self, layout: PivotLayout) -> Self { self.layout = layout; self }
    pub fn show_subtotals(mut self, field: &str, show: bool) -> Self {
        self.field_subtotals.push((field.into(), show)); self
    }
    pub fn set_value_format(mut self, field: &str, fmt: &str) -> Self {
        for vf in &mut self.value_fields {
            if vf.source_field == field { vf.num_format = Some(fmt.into()); }
        }
        self
    }
}

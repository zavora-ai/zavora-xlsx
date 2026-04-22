use crate::utility::{ColNum, RowNum};

#[derive(Debug, Clone, Copy)]
pub enum TableStyle {
    Light(u8),
    Medium(u8),
    Dark(u8),
}

impl TableStyle {
    pub fn name(&self) -> String {
        match self {
            TableStyle::Light(n) => format!("TableStyleLight{n}"),
            TableStyle::Medium(n) => format!("TableStyleMedium{n}"),
            TableStyle::Dark(n) => format!("TableStyleDark{n}"),
        }
    }
}

/// Formatting for a single element of a custom table style.
#[derive(Debug, Clone, Default)]
pub struct TableStyleElementFormat {
    /// Background color as RGB.
    pub bg_color: Option<[u8; 3]>,
    /// Font color as RGB.
    pub font_color: Option<[u8; 3]>,
    /// Bold font.
    pub bold: bool,
    /// Italic font.
    pub italic: bool,
}

impl TableStyleElementFormat {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn bg_color(mut self, rgb: [u8; 3]) -> Self {
        self.bg_color = Some(rgb);
        self
    }
    pub fn font_color(mut self, rgb: [u8; 3]) -> Self {
        self.font_color = Some(rgb);
        self
    }
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
}

/// A custom table style with configurable stripe sizes and element formatting.
#[derive(Debug, Clone)]
pub struct CustomTableStyle {
    /// Name of the custom style.
    pub name: String,
    /// Number of rows per first row stripe (default 1).
    pub first_row_stripe_size: u32,
    /// Number of rows per second row stripe (default 1).
    pub second_row_stripe_size: u32,
    /// Formatting for the header row.
    pub header_row: Option<TableStyleElementFormat>,
    /// Formatting for the total row.
    pub total_row: Option<TableStyleElementFormat>,
    /// Formatting for the first column.
    pub first_column: Option<TableStyleElementFormat>,
    /// Formatting for the last column.
    pub last_column: Option<TableStyleElementFormat>,
    /// Formatting for the first row stripe.
    pub first_row_stripe: Option<TableStyleElementFormat>,
    /// Formatting for the second row stripe.
    pub second_row_stripe: Option<TableStyleElementFormat>,
}

impl CustomTableStyle {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            first_row_stripe_size: 1,
            second_row_stripe_size: 1,
            header_row: None,
            total_row: None,
            first_column: None,
            last_column: None,
            first_row_stripe: None,
            second_row_stripe: None,
        }
    }

    pub fn first_row_stripe_size(mut self, size: u32) -> Self {
        self.first_row_stripe_size = size;
        self
    }
    pub fn second_row_stripe_size(mut self, size: u32) -> Self {
        self.second_row_stripe_size = size;
        self
    }
    pub fn header_row(mut self, fmt: TableStyleElementFormat) -> Self {
        self.header_row = Some(fmt);
        self
    }
    pub fn total_row(mut self, fmt: TableStyleElementFormat) -> Self {
        self.total_row = Some(fmt);
        self
    }
    pub fn first_column(mut self, fmt: TableStyleElementFormat) -> Self {
        self.first_column = Some(fmt);
        self
    }
    pub fn last_column(mut self, fmt: TableStyleElementFormat) -> Self {
        self.last_column = Some(fmt);
        self
    }
    pub fn first_row_stripe(mut self, fmt: TableStyleElementFormat) -> Self {
        self.first_row_stripe = Some(fmt);
        self
    }
    pub fn second_row_stripe(mut self, fmt: TableStyleElementFormat) -> Self {
        self.second_row_stripe = Some(fmt);
        self
    }
}

#[derive(Debug, Clone)]
pub struct TableColumn {
    pub(crate) name: String,
    pub(crate) total_label: Option<String>,
    pub(crate) total_function: Option<String>,
}

impl TableColumn {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            total_label: None,
            total_function: None,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn total_label(&self) -> Option<&str> {
        self.total_label.as_deref()
    }
    pub fn total_function(&self) -> Option<&str> {
        self.total_function.as_deref()
    }
    pub fn set_total_label(&mut self, label: &str) -> &mut Self {
        self.total_label = Some(label.into());
        self
    }
    pub fn set_total_function(&mut self, func: &str) -> &mut Self {
        self.total_function = Some(func.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    pub(crate) columns: Vec<TableColumn>,
    pub(crate) style: Option<TableStyle>,
    pub(crate) custom_style: Option<CustomTableStyle>,
    pub(crate) total_row: bool,
    pub(crate) autofilter: bool,
    pub(crate) first_row: RowNum,
    pub(crate) first_col: ColNum,
    pub(crate) last_row: RowNum,
    pub(crate) last_col: ColNum,
    pub(crate) name: Option<String>,
    // Accessibility metadata (Task 82)
    pub(crate) alt_text: Option<(String, String)>,
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Table {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            style: None,
            custom_style: None,
            total_row: false,
            autofilter: true,
            first_row: 0,
            first_col: 0,
            last_row: 0,
            last_col: 0,
            name: None,
            alt_text: None,
        }
    }
    pub fn set_columns(&mut self, cols: &[TableColumn]) -> &mut Self {
        self.columns = cols.to_vec();
        self
    }
    pub fn set_style(&mut self, style: TableStyle) -> &mut Self {
        self.style = Some(style);
        self
    }
    pub fn set_custom_style(&mut self, style: CustomTableStyle) -> &mut Self {
        self.custom_style = Some(style);
        self
    }
    pub fn set_total_row(&mut self, enable: bool) -> &mut Self {
        self.total_row = enable;
        self
    }
    pub fn set_autofilter(&mut self, enable: bool) -> &mut Self {
        self.autofilter = enable;
        self
    }
    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// Set accessibility alt text (title and description) for this table.
    ///
    /// The title and description are serialized as `displayName` title and
    /// `comment` attributes on the `<table>` element in the table XML.
    pub fn set_alt_text(&mut self, title: &str, description: &str) -> &mut Self {
        self.alt_text = Some((title.to_string(), description.to_string()));
        self
    }

    // Read accessors
    pub fn columns(&self) -> &[TableColumn] {
        &self.columns
    }
    pub fn style(&self) -> Option<TableStyle> {
        self.style
    }
    pub fn custom_style(&self) -> Option<&CustomTableStyle> {
        self.custom_style.as_ref()
    }
    pub fn total_row(&self) -> bool {
        self.total_row
    }
    pub fn autofilter(&self) -> bool {
        self.autofilter
    }
    pub fn table_name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    pub fn first_row(&self) -> RowNum {
        self.first_row
    }
    pub fn first_col(&self) -> ColNum {
        self.first_col
    }
    pub fn last_row(&self) -> RowNum {
        self.last_row
    }
    pub fn last_col(&self) -> ColNum {
        self.last_col
    }

    /// Returns the alt text (title, description) if set.
    pub fn alt_text(&self) -> Option<(&str, &str)> {
        self.alt_text
            .as_ref()
            .map(|(t, d)| (t.as_str(), d.as_str()))
    }
}

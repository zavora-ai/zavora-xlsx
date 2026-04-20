use crate::utility::{ColNum, RowNum};

#[derive(Debug, Clone, Copy)]
pub enum TableStyle {
    Light(u8), Medium(u8), Dark(u8),
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

#[derive(Debug, Clone)]
pub struct TableColumn {
    pub(crate) name: String,
    pub(crate) total_label: Option<String>,
    pub(crate) total_function: Option<String>,
}

impl TableColumn {
    pub fn new(name: &str) -> Self { Self { name: name.into(), total_label: None, total_function: None } }
    pub fn name(&self) -> &str { &self.name }
    pub fn total_label(&self) -> Option<&str> { self.total_label.as_deref() }
    pub fn total_function(&self) -> Option<&str> { self.total_function.as_deref() }
    pub fn set_total_label(&mut self, label: &str) -> &mut Self { self.total_label = Some(label.into()); self }
    pub fn set_total_function(&mut self, func: &str) -> &mut Self { self.total_function = Some(func.into()); self }
}

#[derive(Debug, Clone)]
pub struct Table {
    pub(crate) columns: Vec<TableColumn>,
    pub(crate) style: Option<TableStyle>,
    pub(crate) total_row: bool,
    pub(crate) autofilter: bool,
    pub(crate) first_row: RowNum,
    pub(crate) first_col: ColNum,
    pub(crate) last_row: RowNum,
    pub(crate) last_col: ColNum,
    pub(crate) name: Option<String>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(), style: None, total_row: false, autofilter: true,
            first_row: 0, first_col: 0, last_row: 0, last_col: 0, name: None,
        }
    }
    pub fn set_columns(&mut self, cols: &[TableColumn]) -> &mut Self { self.columns = cols.to_vec(); self }
    pub fn set_style(&mut self, style: TableStyle) -> &mut Self { self.style = Some(style); self }
    pub fn set_total_row(&mut self, enable: bool) -> &mut Self { self.total_row = enable; self }
    pub fn set_autofilter(&mut self, enable: bool) -> &mut Self { self.autofilter = enable; self }
    pub fn set_name(&mut self, name: &str) -> &mut Self { self.name = Some(name.into()); self }

    // Read accessors
    pub fn columns(&self) -> &[TableColumn] { &self.columns }
    pub fn style(&self) -> Option<TableStyle> { self.style }
    pub fn total_row(&self) -> bool { self.total_row }
    pub fn autofilter(&self) -> bool { self.autofilter }
    pub fn table_name(&self) -> Option<&str> { self.name.as_deref() }
    pub fn first_row(&self) -> RowNum { self.first_row }
    pub fn first_col(&self) -> ColNum { self.first_col }
    pub fn last_row(&self) -> RowNum { self.last_row }
    pub fn last_col(&self) -> ColNum { self.last_col }
}

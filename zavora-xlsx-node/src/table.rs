use std::cell::RefCell;
use napi_derive::napi;
use zavora_xlsx::{TableColumn, TableStyle};

#[napi(object)]
pub struct TableColumnJs {
    pub name: String,
    pub total_label: Option<String>,
    pub total_function: Option<String>,
}

#[napi]
pub struct Table {
    pub(crate) inner: RefCell<zavora_xlsx::Table>,
}

#[napi]
impl Table {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(zavora_xlsx::Table::new()),
        }
    }

    #[napi]
    pub fn set_columns(&self, columns: Vec<TableColumnJs>) -> &Self {
        let cols: Vec<TableColumn> = columns
            .into_iter()
            .map(|c| {
                let mut tc = TableColumn::new(&c.name);
                if let Some(ref label) = c.total_label {
                    tc.set_total_label(label);
                }
                if let Some(ref func) = c.total_function {
                    tc.set_total_function(func);
                }
                tc
            })
            .collect();
        {
            self.inner.borrow_mut().set_columns(&cols);
        }
        self
    }

    #[napi]
    pub fn set_style(&self, style: String) -> &Self {
        if let Some(ts) = parse_table_style(&style) {
            self.inner.borrow_mut().set_style(ts);
        }
        self
    }

    #[napi]
    pub fn set_total_row(&self, enabled: bool) -> &Self {
        {
            self.inner.borrow_mut().set_total_row(enabled);
        }
        self
    }

    #[napi]
    pub fn set_autofilter(&self, enabled: bool) -> &Self {
        {
            self.inner.borrow_mut().set_autofilter(enabled);
        }
        self
    }

    #[napi]
    pub fn set_name(&self, name: String) -> &Self {
        {
            self.inner.borrow_mut().set_name(&name);
        }
        self
    }
}

fn parse_table_style(s: &str) -> Option<TableStyle> {
    if let Some(rest) = s.strip_prefix("TableStyleLight") {
        rest.parse::<u8>().ok().map(TableStyle::Light)
    } else if let Some(rest) = s.strip_prefix("TableStyleMedium") {
        rest.parse::<u8>().ok().map(TableStyle::Medium)
    } else if let Some(rest) = s.strip_prefix("TableStyleDark") {
        rest.parse::<u8>().ok().map(TableStyle::Dark)
    } else {
        None
    }
}

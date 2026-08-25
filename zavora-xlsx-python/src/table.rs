use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use zavora_xlsx::{TableColumn, TableStyle};

#[pyclass]
pub struct Table {
    pub(crate) inner: zavora_xlsx::Table,
}

#[pymethods]
impl Table {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: zavora_xlsx::Table::new(),
        }
    }

    pub fn set_columns<'a>(
        mut slf: PyRefMut<'a, Self>,
        columns: &'a Bound<'a, PyList>,
    ) -> PyResult<PyRefMut<'a, Self>> {
        let mut cols: Vec<TableColumn> = Vec::new();
        for item in columns.iter() {
            let dict = item.cast::<PyDict>()?;
            let name: String = dict
                .get_item("name")?
                .ok_or_else(|| {
                    pyo3::exceptions::PyValueError::new_err("column dict must have a 'name' key")
                })?
                .extract()?;
            let mut tc = TableColumn::new(&name);
            if let Some(label) = dict.get_item("total_label")? {
                let label_str: String = label.extract()?;
                tc.set_total_label(&label_str);
            }
            if let Some(func) = dict.get_item("total_function")? {
                let func_str: String = func.extract()?;
                tc.set_total_function(&func_str);
            }
            cols.push(tc);
        }
        slf.inner.set_columns(&cols);
        Ok(slf)
    }

    pub fn set_style<'a>(mut slf: PyRefMut<'a, Self>, style: &'a str) -> PyRefMut<'a, Self> {
        if let Some(ts) = parse_table_style(style) {
            slf.inner.set_style(ts);
        }
        slf
    }

    pub fn set_total_row(mut slf: PyRefMut<'_, Self>, enabled: bool) -> PyRefMut<'_, Self> {
        slf.inner.set_total_row(enabled);
        slf
    }

    pub fn set_autofilter(mut slf: PyRefMut<'_, Self>, enabled: bool) -> PyRefMut<'_, Self> {
        slf.inner.set_autofilter(enabled);
        slf
    }

    pub fn set_name<'a>(mut slf: PyRefMut<'a, Self>, name: &'a str) -> PyRefMut<'a, Self> {
        slf.inner.set_name(name);
        slf
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

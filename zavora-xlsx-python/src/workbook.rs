use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use std::sync::{Arc, Mutex};

use crate::error::IntoPyResult;
use crate::worksheet::Worksheet;

#[pyclass]
pub struct Workbook {
    pub(crate) inner: Arc<Mutex<zavora_xlsx::Workbook>>,
}

#[pymethods]
impl Workbook {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(zavora_xlsx::Workbook::new())),
        }
    }

    #[staticmethod]
    pub fn open(path: &str) -> PyResult<Self> {
        let wb = zavora_xlsx::Workbook::open(path).into_pyresult()?;
        Ok(Self {
            inner: Arc::new(Mutex::new(wb)),
        })
    }

    #[staticmethod]
    pub fn open_from_bytes(data: &[u8]) -> PyResult<Self> {
        let wb = zavora_xlsx::Workbook::open_from_buffer(data).into_pyresult()?;
        Ok(Self {
            inner: Arc::new(Mutex::new(wb)),
        })
    }

    pub fn save(&self, path: &str) -> PyResult<()> {
        let mut wb = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("lock poisoned"))?;
        wb.save(path).into_pyresult()
    }

    pub fn save_to_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let mut wb = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("lock poisoned"))?;
        let bytes = wb.save_to_buffer().into_pyresult()?;
        Ok(PyBytes::new_bound(py, &bytes))
    }

    pub fn worksheet(&self, index: usize) -> PyResult<Worksheet> {
        Ok(Worksheet {
            workbook: Arc::clone(&self.inner),
            index,
        })
    }

    pub fn add_worksheet(&self) -> PyResult<Worksheet> {
        let mut wb = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("lock poisoned"))?;
        wb.add_worksheet();
        let idx = wb.sheet_count() - 1;
        drop(wb);
        Ok(Worksheet {
            workbook: Arc::clone(&self.inner),
            index: idx,
        })
    }

    pub fn add_worksheet_with_name(&self, name: &str) -> PyResult<Worksheet> {
        let mut wb = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("lock poisoned"))?;
        wb.add_worksheet_with_name(name).into_pyresult()?;
        let idx = wb.sheet_count() - 1;
        drop(wb);
        Ok(Worksheet {
            workbook: Arc::clone(&self.inner),
            index: idx,
        })
    }

    pub fn sheet_names(&self) -> PyResult<Vec<String>> {
        let wb = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("lock poisoned"))?;
        Ok(wb.sheet_names().iter().map(|s| s.to_string()).collect())
    }

    pub fn sheet_count(&self) -> PyResult<usize> {
        let wb = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("lock poisoned"))?;
        Ok(wb.sheet_count())
    }

    pub fn set_properties(&self, props: &Bound<'_, PyDict>) -> PyResult<()> {
        let mut wb = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("lock poisoned"))?;
        let mut dp = zavora_xlsx::DocProperties::new();
        if let Some(t) = props.get_item("title")? {
            let val: String = t.extract()?;
            dp = dp.title(&val);
        }
        if let Some(a) = props.get_item("author")? {
            let val: String = a.extract()?;
            dp = dp.author(&val);
        }
        if let Some(s) = props.get_item("subject")? {
            let val: String = s.extract()?;
            dp = dp.subject(&val);
        }
        if let Some(d) = props.get_item("description")? {
            let val: String = d.extract()?;
            dp = dp.description(&val);
        }
        if let Some(k) = props.get_item("keywords")? {
            let val: String = k.extract()?;
            dp = dp.keywords(&val);
        }
        if let Some(c) = props.get_item("category")? {
            let val: String = c.extract()?;
            dp = dp.category(&val);
        }
        if let Some(c) = props.get_item("company")? {
            let val: String = c.extract()?;
            dp = dp.company(&val);
        }
        wb.set_properties(dp);
        Ok(())
    }

    pub fn properties<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let wb = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("lock poisoned"))?;
        let p = wb.properties();
        let dict = PyDict::new_bound(py);
        if let Some(ref v) = p.title {
            dict.set_item("title", v)?;
        }
        if let Some(ref v) = p.author {
            dict.set_item("author", v)?;
        }
        if let Some(ref v) = p.subject {
            dict.set_item("subject", v)?;
        }
        if let Some(ref v) = p.description {
            dict.set_item("description", v)?;
        }
        if let Some(ref v) = p.keywords {
            dict.set_item("keywords", v)?;
        }
        if let Some(ref v) = p.category {
            dict.set_item("category", v)?;
        }
        if let Some(ref v) = p.company {
            dict.set_item("company", v)?;
        }
        Ok(dict)
    }
}

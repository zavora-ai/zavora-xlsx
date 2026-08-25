use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::chart::Chart;
use crate::error::IntoPyResult;
use crate::format::Format;
use crate::table::Table;

#[pyclass]
pub struct Worksheet {
    pub(crate) workbook: Arc<Mutex<zavora_xlsx::Workbook>>,
    pub(crate) index: usize,
}

impl Worksheet {
    fn lock(&self) -> PyResult<MutexGuard<'_, zavora_xlsx::Workbook>> {
        self.workbook
            .lock()
            .map_err(|_| PyRuntimeError::new_err("lock poisoned"))
    }
}

#[pymethods]
impl Worksheet {
    // ── Cell write methods ──

    #[pyo3(signature = (row, col, value, format=None))]
    pub fn write_string(
        &self,
        row: u32,
        col: u16,
        value: &str,
        format: Option<&Format>,
    ) -> PyResult<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_pyresult()?;
        match format {
            Some(fmt) => {
                ws.write_with_format(row, col, value, &fmt.inner)
                    .into_pyresult()?;
            }
            None => {
                ws.write(row, col, value).into_pyresult()?;
            }
        }
        Ok(())
    }

    #[pyo3(signature = (row, col, value, format=None))]
    pub fn write_number(
        &self,
        row: u32,
        col: u16,
        value: f64,
        format: Option<&Format>,
    ) -> PyResult<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_pyresult()?;
        match format {
            Some(fmt) => {
                ws.write_with_format(row, col, value, &fmt.inner)
                    .into_pyresult()?;
            }
            None => {
                ws.write(row, col, value).into_pyresult()?;
            }
        }
        Ok(())
    }

    #[pyo3(signature = (row, col, value, format=None))]
    pub fn write_boolean(
        &self,
        row: u32,
        col: u16,
        value: bool,
        format: Option<&Format>,
    ) -> PyResult<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_pyresult()?;
        match format {
            Some(fmt) => {
                ws.write_with_format(row, col, value, &fmt.inner)
                    .into_pyresult()?;
            }
            None => {
                ws.write(row, col, value).into_pyresult()?;
            }
        }
        Ok(())
    }

    #[pyo3(signature = (row, col, formula, format=None))]
    pub fn write_formula(
        &self,
        row: u32,
        col: u16,
        formula: &str,
        format: Option<&Format>,
    ) -> PyResult<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_pyresult()?;
        ws.write_formula(row, col, formula).into_pyresult()?;
        if let Some(fmt) = format {
            ws.set_cell_format(row, col, &fmt.inner).into_pyresult()?;
        }
        Ok(())
    }

    pub fn write_blank(&self, row: u32, col: u16, format: &Format) -> PyResult<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_pyresult()?;
        ws.write_blank(row, col, &format.inner).into_pyresult()?;
        Ok(())
    }

    // ── Cell read methods ──

    pub fn read_cell(&self, py: Python<'_>, row: u32, col: u16) -> PyResult<Py<PyAny>> {
        let wb = self.lock()?;
        let ws = wb.worksheet_ref(self.index).into_pyresult()?;
        let cv = ws.read_cell(row, col);
        Ok(cell_value_to_py(py, &cv))
    }

    pub fn used_range<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyDict>>> {
        let wb = self.lock()?;
        let ws = wb.worksheet_ref(self.index).into_pyresult()?;
        Ok(ws.used_range().map(|(r1, c1, r2, c2)| {
            let dict = PyDict::new(py);
            dict.set_item("first_row", r1).unwrap();
            dict.set_item("first_col", c1).unwrap();
            dict.set_item("last_row", r2).unwrap();
            dict.set_item("last_col", c2).unwrap();
            dict
        }))
    }

    // ── Worksheet name methods ──

    pub fn name(&self) -> PyResult<String> {
        let wb = self.lock()?;
        let ws = wb.worksheet_ref(self.index).into_pyresult()?;
        Ok(ws.name().to_string())
    }

    pub fn set_name(&self, name: &str) -> PyResult<()> {
        let mut wb = self.lock()?;
        wb.rename_worksheet(self.index, name).into_pyresult()
    }

    // ── Layout methods ──

    pub fn set_column_width(&self, col: u16, width: f64) -> PyResult<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_pyresult()?;
        ws.set_column_width(col, width).into_pyresult()?;
        Ok(())
    }

    pub fn set_row_height(&self, row: u32, height: f64) -> PyResult<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_pyresult()?;
        ws.set_row_height(row, height).into_pyresult()?;
        Ok(())
    }

    pub fn set_freeze_panes(&self, row: u32, col: u16) -> PyResult<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_pyresult()?;
        ws.set_freeze_panes(row, col).into_pyresult()?;
        Ok(())
    }

    #[pyo3(signature = (r1, c1, r2, c2, text, format=None))]
    pub fn merge_range(
        &self,
        r1: u32,
        c1: u16,
        r2: u32,
        c2: u16,
        text: &str,
        format: Option<&Format>,
    ) -> PyResult<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_pyresult()?;
        let fmt = match format {
            Some(f) => f.inner.clone(),
            None => zavora_xlsx::Format::new(),
        };
        ws.merge_range(r1, c1, r2, c2, text, &fmt).into_pyresult()?;
        Ok(())
    }

    pub fn autofit(&self) -> PyResult<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_pyresult()?;
        ws.autofit().into_pyresult()?;
        Ok(())
    }

    pub fn set_zoom(&self, percent: u16) -> PyResult<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_pyresult()?;
        ws.set_zoom(percent);
        Ok(())
    }

    // ── Chart and table insertion ──

    pub fn insert_chart(&self, row: u32, col: u16, chart: &Chart) -> PyResult<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_pyresult()?;
        ws.insert_chart(row, col, &chart.inner).into_pyresult()?;
        Ok(())
    }

    pub fn add_table(
        &self,
        first_row: u32,
        first_col: u16,
        last_row: u32,
        last_col: u16,
        table: &Table,
    ) -> PyResult<()> {
        let mut wb = self.lock()?;
        let ws = wb.worksheet(self.index).into_pyresult()?;
        ws.add_table(first_row, first_col, last_row, last_col, &table.inner)
            .into_pyresult()?;
        Ok(())
    }

    // ── CSV export ──

    #[pyo3(signature = (options=None))]
    pub fn to_csv_string(&self, options: Option<&Bound<'_, PyDict>>) -> PyResult<String> {
        let wb = self.lock()?;
        let ws = wb.worksheet_ref(self.index).into_pyresult()?;
        let mut opts = zavora_xlsx::CsvOptions::new();
        if let Some(dict) = options {
            if let Some(d) = dict.get_item("delimiter")? {
                let s: String = d.extract()?;
                if s.is_empty() {
                    return Err(PyValueError::new_err("delimiter must not be empty"));
                }
                opts.delimiter = s.as_bytes()[0];
            }
            if let Some(q) = dict.get_item("quote")? {
                let s: String = q.extract()?;
                if s.is_empty() {
                    return Err(PyValueError::new_err("quote must not be empty"));
                }
                opts.quote = s.as_bytes()[0];
            }
            if let Some(le) = dict.get_item("line_ending")? {
                opts.line_ending = le.extract()?;
            }
            if let Some(df) = dict.get_item("date_format")? {
                opts.date_format = df.extract()?;
            }
        }
        Ok(ws.to_csv_string(&opts))
    }
}

// ── Private helpers ──

fn cell_value_to_py(py: Python<'_>, value: &zavora_xlsx::CellValue) -> Py<PyAny> {
    match value {
        zavora_xlsx::CellValue::Empty => py.None(),
        zavora_xlsx::CellValue::String(s) => s.into_pyobject(py).unwrap().into_any().unbind(),
        zavora_xlsx::CellValue::Number(n) => n.into_pyobject(py).unwrap().into_any().unbind(),
        zavora_xlsx::CellValue::Bool(b) => pyo3::types::PyBool::new(py, *b)
            .to_owned()
            .into_any()
            .unbind(),
        zavora_xlsx::CellValue::DateTime(dt) => {
            dt.serial().into_pyobject(py).unwrap().into_any().unbind()
        }
        zavora_xlsx::CellValue::Error(e) => e.into_pyobject(py).unwrap().into_any().unbind(),
        zavora_xlsx::CellValue::RichText(rt) => rt
            .plain_text()
            .into_pyobject(py)
            .unwrap()
            .into_any()
            .unbind(),
        zavora_xlsx::CellValue::Formula {
            formula,
            cached_value,
        } => {
            let dict = PyDict::new(py);
            dict.set_item("formula", formula).unwrap();
            dict.set_item("cached_value", cell_value_to_py(py, cached_value))
                .unwrap();
            dict.into_any().unbind()
        }
    }
}

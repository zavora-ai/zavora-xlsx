#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::useless_conversion)]

use pyo3::prelude::*;

mod chart;
mod error;
mod format;
mod table;
mod workbook;
mod worksheet;

/// The zavora_xlsx Python module.
#[pymodule]
fn zavora_xlsx(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<workbook::Workbook>()?;
    m.add_class::<worksheet::Worksheet>()?;
    m.add_class::<format::Format>()?;
    m.add_class::<chart::Chart>()?;
    m.add_class::<table::Table>()?;
    Ok(())
}

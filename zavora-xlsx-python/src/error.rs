use pyo3::PyErr;
use pyo3::exceptions::{PyIndexError, PyOSError, PyRuntimeError, PyValueError};

pub fn to_py_err(e: zavora_xlsx::Error) -> PyErr {
    match &e {
        zavora_xlsx::Error::Io(_) => PyOSError::new_err(format!("{e}")),
        zavora_xlsx::Error::SheetNotFound(_) => PyIndexError::new_err(format!("{e}")),
        zavora_xlsx::Error::InvalidData(_) => PyValueError::new_err(format!("{e}")),
        _ => PyRuntimeError::new_err(format!("{e}")),
    }
}

pub trait IntoPyResult<T> {
    fn into_pyresult(self) -> pyo3::PyResult<T>;
}

impl<T> IntoPyResult<T> for Result<T, zavora_xlsx::Error> {
    fn into_pyresult(self) -> pyo3::PyResult<T> {
        self.map_err(to_py_err)
    }
}

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyAny;

/// Convert a Python path-like object (e.g., `str`, `pathlib.Path`) into a Rust `String`.
///
/// Accepts:
/// - Python `str`
/// - Any Python object that provides `__str__`
///
/// Returns a `PyValueError` if conversion is not possible.
pub fn py_path_to_string(arg_name: &str, value: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(path_str) = value.extract::<String>() {
        Ok(path_str)
    } else if let Ok(path_obj) = value.call_method0("__str__") {
        path_obj.extract::<String>()
    } else {
        Err(PyValueError::new_err(format!(
            "{arg_name} must be a string or pathlib.Path object"
        )))
    }
}

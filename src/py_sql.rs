//! Python-facing wrappers for SQL binding and the injection guard.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};

use crate::enums::SqlParamStyle;
use crate::sql_guard;

/// A driver-ready statement: the rewritten SQL plus the values to bind.
///
/// Unpacks straight into a DB-API call:
///
/// ```python
/// query, params = subtask.prepare({"id": 7})
/// cursor.execute(query, params)
/// ```
#[pyclass]
pub struct PreparedQuery {
    /// SQL with driver placeholders (`?`, `$1`, `:name`, ...) in place of the
    /// template placeholders.
    #[pyo3(get)]
    pub query: String,
    /// Parameter names in bind order.
    #[pyo3(get)]
    pub names: Vec<String>,
    /// Placeholder syntax the query was rendered for.
    #[pyo3(get)]
    pub param_style: SqlParamStyle,
    pub(crate) values: Vec<PyObject>,
}

impl PreparedQuery {
    fn params_obj(&self, py: Python) -> PyResult<PyObject> {
        if self.param_style.is_named() {
            let dict = PyDict::new_bound(py);
            for (name, value) in self.names.iter().zip(self.values.iter()) {
                dict.set_item(name, value)?;
            }
            Ok(dict.into())
        } else {
            let list = PyList::empty_bound(py);
            for value in &self.values {
                list.append(value)?;
            }
            Ok(list.into())
        }
    }
}

#[pymethods]
impl PreparedQuery {
    /// Values to bind: a `list` for positional styles, a `dict` for named ones.
    #[getter]
    fn params(&self, py: Python) -> PyResult<PyObject> {
        self.params_obj(py)
    }

    /// `(query, params)`.
    fn as_tuple(&self, py: Python) -> PyResult<Py<PyTuple>> {
        let items: Vec<PyObject> = vec![self.query.clone().into_py(py), self.params_obj(py)?];
        Ok(PyTuple::new_bound(py, items).into())
    }

    /// Two items, so `query, params = prepared` works.
    fn __len__(&self) -> usize {
        2
    }

    fn __iter__(&self, py: Python) -> PyResult<PyObject> {
        let tuple = self.as_tuple(py)?;
        Ok(tuple.bind(py).as_any().iter()?.into_py(py))
    }

    fn __getitem__(&self, py: Python, index: isize) -> PyResult<PyObject> {
        match index {
            0 | -2 => Ok(self.query.clone().into_py(py)),
            1 | -1 => self.params_obj(py),
            _ => Err(pyo3::exceptions::PyIndexError::new_err(
                "PreparedQuery index out of range",
            )),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "PreparedQuery(query={:?}, names={:?}, param_style={})",
            self.query,
            self.names,
            self.param_style.name()
        )
    }
}

/// Baseline SQL-injection checks for values that are **interpolated** into a
/// statement rather than bound.
///
/// Binding (`Subtask.prepare`) is always preferable; this is the safety net for
/// the cases where a value has to become part of the SQL text.
#[pyclass]
pub struct SqlGuard;

#[pymethods]
impl SqlGuard {
    /// True when the value carries none of the patterns the guard rejects.
    #[staticmethod]
    fn is_safe(value: &str) -> bool {
        sql_guard::is_safe(value)
    }

    /// Every reason the value would be rejected, as human-readable strings.
    #[staticmethod]
    fn find_issues(value: &str) -> Vec<String> {
        sql_guard::find_issues(value)
            .iter()
            .map(|i| i.to_string())
            .collect()
    }

    /// Raise `ValueError` if the value is unsafe to interpolate.
    #[staticmethod]
    #[pyo3(signature = (value, name="value"))]
    fn check(value: &str, name: &str) -> PyResult<()> {
        sql_guard::check_value(name, value).map_err(PyValueError::new_err)
    }

    /// Validate a table/schema/column name that must be inlined, since drivers
    /// cannot bind identifiers. Returns it unchanged, or raises `ValueError`.
    #[staticmethod]
    fn check_identifier(value: &str) -> PyResult<String> {
        sql_guard::check_identifier(value).map_err(PyValueError::new_err)
    }

    fn __repr__(&self) -> String {
        "SqlGuard()".to_string()
    }
}

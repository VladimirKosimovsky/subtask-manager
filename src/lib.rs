mod enums;
mod file_classifier;
mod file_loader;
mod file_scanner;
mod models;

use crate::file_classifier::classify;
use crate::file_loader::load;
use crate::file_scanner::scan_files;
use crate::models::Subtask;
use enums::{EtlStage, SystemType};

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::PyObject;


/// SubtaskManager exposed to Python
#[pyclass]
pub struct SubtaskManager {
    #[pyo3(get)]
    pub base_path: String,
    subtasks: Vec<Subtask>,
}

#[pymethods]
impl SubtaskManager {
    #[new]
    fn new(base_path: String) -> PyResult<Self> {
        // Build extension list from enums: hardcode same extension set here
        let extensions = vec![
            "sql", "psql", "tsql", "plpgsql", "sh", "ps1", "py", "gql", "graphql", "json", "jsonl",
            "yaml", "yml",
        ];
        let files = scan_files(&base_path, &extensions)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let mut subtasks = Vec::new();
        for f in files {
            match classify(&base_path, &f) {
                Ok(s) => match load(s) {
                    Ok(loaded) => subtasks.push(loaded),
                    Err(e) => return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
                },
                Err(e) => return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
            }
        }

        Ok(SubtaskManager {
            base_path,
            subtasks,
        })
    }
    #[pyo3(signature = (etl_stage=None, entity=None, system_type=None,task_type=None, is_common=None, include_common=None))]
    fn get_tasks(
        &self,
        py: Python,
        etl_stage: Option<String>,
        entity: Option<String>,
        system_type: Option<String>,
        task_type: Option<String>,
        is_common: Option<bool>,
        include_common: Option<bool>,
    ) -> PyResult<PyObject> {
        let include_common = include_common.unwrap_or(true);
        let mut filtered: Vec<crate::models::Subtask> = Vec::new();

        let input_system_type = system_type
            .as_ref()
            .and_then(|st| SystemType::from_alias(st).ok());

        for subtask in &self.subtasks {
            if let Some(ref es) = etl_stage {
                if subtask.stage.as_ref() != Some(es) {
                    continue;
                }
            }
            if let Some(ref en) = entity {
                if subtask.entity.as_ref() != Some(en) {
                    continue;
                }
            }
            // Filter by system_type using the converted enum
            if let Some(ref input_st) = input_system_type {
                if subtask.system_type.as_ref() != Some(input_st) {
                    continue;
                }
            }
            if let Some(ref tt) = task_type {
                if subtask.task_type.as_ref() != Some(tt) {
                    continue;
                }
            }
            if let Some(ic) = is_common {
                if subtask.is_common != ic {
                    continue;
                }
            }
            filtered.push(subtask.clone());
        }

        if include_common {
            for s in &self.subtasks {
                if s.is_common && !filtered.iter().any(|x| x.path == s.path) {
                    filtered.push(s.clone());
                }
            }
        }

        // Convert to PyObject list
        let mut py_objs: Vec<PyObject> = Vec::with_capacity(filtered.len());
        for s in filtered {
            let py_sub = Py::new(py, s).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            py_objs.push(py_sub.into_py(py));
        }

        let py_list = PyList::empty_bound(py);
        Ok(py_list.into())
    }

    #[pyo3(signature = (name, entity=None))]
    fn get_task(&self, py: Python, name: String, entity: Option<String>) -> PyResult<PyObject> {
        for s in &self.subtasks {
            if s.name == name {
                if let Some(ref e) = entity {
                    if s.entity.as_ref() == Some(e) {
                        let py_sub = Py::new(py, s.clone())
                            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
                        return Ok(py_sub.into_py(py));
                    }
                } else {
                    let py_sub = Py::new(py, s.clone())
                        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
                    return Ok(py_sub.into_py(py));
                }
            }
        }
        Err(PyValueError::new_err(format!(
            "Task with name '{}' not found",
            name
        )))
    }

    #[getter]
    #[pyo3(name = "subtasks")]
    fn subtasks_py(&self, py: Python) -> PyResult<PyObject> {
        let mut py_objs: Vec<PyObject> = Vec::with_capacity(self.subtasks.len());
        for s in &self.subtasks {
            let py_sub =
                Py::new(py, s.clone()).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            py_objs.push(py_sub.into_py(py));
        }
        Ok(PyList::new_bound(py, py_objs).into())
    }
}

#[pymethods]
impl EtlStage {
    pub fn __str__(&self) -> &'static str {
        self.name()
    }

    pub fn __repr__(&self) -> String {
        format!("EtlStage.{}", self.name().to_uppercase())
    }

    #[getter]
    #[pyo3(name = "name")]
    fn stage_name_py(&self) -> &'static str {
        self.name()
    }
    #[getter]
    #[pyo3(name = "aliases")]
    fn aliases_py(&self) -> Vec<&'static str> {
        self.aliases().to_vec()
    }
    
    #[getter]
    #[pyo3(name = "id")]
    fn stage_id_py(&self) -> u8 {
        *self.id()
    }
    
    #[staticmethod]
    #[pyo3(name = "from_alias")]
    fn from_alias_py(alias: String) -> PyResult<EtlStage> {
        EtlStage::from_alias(&alias)
            .map_err(|e| PyValueError::new_err(e))
    }
}

#[pymethods]
impl SystemType {
    pub fn __str__(&self) -> &'static str {
        self.name()
    }

    pub fn __repr__(&self) -> String {
        format!("SystemType.{}", self.name().to_uppercase())
    }

    #[getter]
    #[pyo3(name = "id")]
    fn system_type_id_py(&self) -> u8 {
        *self.id()
    }

    #[getter]
    #[pyo3(name = "name")]
    fn system_type_name_py(&self) -> &'static str {
        self.name()
    }
    #[getter]
    #[pyo3(name = "aliases")]
    fn system_type_aliases_py(&self) -> Vec<&'static str> {
        self.aliases().to_vec()
    }
    
    #[staticmethod]
    #[pyo3(name = "from_alias")]
    fn from_alias_py(alias: String) -> PyResult<SystemType> {
        SystemType::from_alias(&alias)
            .map_err(|e| PyValueError::new_err(e))
    }



    
}

#[pymodule]
fn _core(m: Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SubtaskManager>()?;
    m.add_class::<models::Subtask>()?;
    m.add_class::<EtlStage>()?;
    m.add_class::<SystemType>()?;
    Ok(())
}

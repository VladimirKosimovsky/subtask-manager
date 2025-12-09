mod enums;
mod file_classifier;
mod file_loader;
mod file_scanner;
mod models;

use pyo3::types::{PyAny, PySet};
use std::collections::HashMap;

use crate::enums::ParamType;
use crate::file_classifier::FileClassifier;
use crate::file_loader::load;
use crate::models::Subtask;
use enums::{EtlStage, SystemType, TaskType};
use file_scanner::FileScanner;
use strum::IntoEnumIterator;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::PyObject;

// SubtaskManager with lazy loading
#[pyclass]
pub struct SubtaskManager {
    #[pyo3(get)]
    pub base_path: String,
    file_paths: Vec<String>, // Store file paths instead of loaded subtasks
    subtasks: Option<Vec<Subtask>>, // Loaded lazily
    classifier: FileClassifier, // Classifier instance for lazy loading
}

impl SubtaskManager {
    // Internal Rust method for lazy loading (not exposed to Python)
    fn load_subtasks(&mut self) -> PyResult<()> {
        if self.subtasks.is_some() {
            return Ok(()); // Already loaded
        }

        let mut subtasks = Vec::new();
        for file_path in &self.file_paths {
            match self.classifier.classify_internal(file_path) {
                Ok(s) => match load(s) {
                    Ok(loaded) => subtasks.push(loaded),
                    Err(e) => return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
                },
                Err(e) => return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
            }
        }

        self.subtasks = Some(subtasks);
        Ok(())
    }
}

#[pymethods]
impl SubtaskManager {
    #[new]
    fn new(base_path: &Bound<'_, PyAny>) -> PyResult<Self> {
        // Convert base_path to string, supporting both str and pathlib.Path
        let base_path_str = if let Ok(path_str) = base_path.extract::<String>() {
            // Direct string
            path_str
        } else if let Ok(path_obj) = base_path.call_method0("__str__") {
            // pathlib.Path or other object with __str__ method
            path_obj.extract::<String>()?
        } else {
            return Err(PyValueError::new_err(
                "base_path must be a string or pathlib.Path object",
            ));
        };

        // Build extension list from TaskType variants
        let extensions: Vec<String> = TaskType::iter()
            .flat_map(|task_type| {
                task_type
                    .extensions()
                    .iter()
                    .map(|&s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .collect();

        let file_scanner = FileScanner::new(extensions);
        let file_paths = file_scanner
            .scan_files(&base_path)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        // Create classifier instance for lazy loading
        let classifier = FileClassifier {
            base_path: base_path_str.clone(),
        };

        Ok(SubtaskManager {
            base_path: base_path_str,
            file_paths,
            subtasks: None, // Not loaded yet
            classifier,
        })
    }

    /// Getter for subtasks that loads them if needed
    #[getter]
    fn subtasks(&mut self, py: Python) -> PyResult<Py<PyList>> {
        self.load_subtasks()?;

        let subtasks = self.subtasks.as_ref().unwrap();
        let py_list = PyList::empty_bound(py);

        for subtask in subtasks {
            // Since Subtask is a pyclass, wrap it in Py
            let py_subtask = Py::new(py, subtask.clone())?;
            py_list.append(py_subtask)?;
        }

        Ok(py_list.into())
    }

    // Get file paths as Python list
    #[getter]
    fn file_paths(&self, py: Python) -> PyResult<Py<PyList>> {
        let py_list = PyList::empty_bound(py);

        for file_path in &self.file_paths {
            py_list.append(file_path)?;
        }

        Ok(py_list.into())
    }

    // Get the number of subtasks (without loading them)
    #[getter]
    fn num_files(&self) -> usize {
        self.file_paths.len()
    }

    // Get the classifier instance
    #[getter]
    fn classifier(&self) -> FileClassifier {
        self.classifier.clone()
    }

    // Explicit method to load subtasks
    fn load_all(&mut self) -> PyResult<()> {
        self.load_subtasks()
    }

    #[pyo3(signature = (etl_stage=None, entity=None, system_type=None, task_type=None, is_common=None, include_common=None))]
    fn get_tasks(
        &mut self,
        py: Python,
        etl_stage: Option<EtlStage>,
        entity: Option<String>,
        system_type: Option<SystemType>,
        task_type: Option<TaskType>,
        is_common: Option<bool>,
        include_common: Option<bool>,
    ) -> PyResult<Py<PyList>> {
        // Ensure subtasks are loaded before filtering
        self.load_subtasks()?;

        let include_common = include_common.unwrap_or(true);
        let mut filtered: Vec<crate::models::Subtask> = Vec::new();

        for subtask in self.subtasks.as_ref().unwrap() {
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
            if let Some(ref st) = system_type {
                if subtask.system_type.as_ref() != Some(st) {
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
            for s in self.subtasks.as_ref().unwrap() {
                if s.is_common && !filtered.iter().any(|x| x.path == s.path) {
                    filtered.push(s.clone());
                }
            }
        }

        // Convert filtered results to Python list
        let py_list = PyList::empty_bound(py);
        for subtask in filtered {
            let py_subtask = Py::new(py, subtask)?;
            py_list.append(py_subtask)?;
        }

        Ok(py_list.into())
    }

    #[pyo3(signature = (name, entity=None))]
    fn get_task(&mut self, py: Python, name: String, entity: Option<String>) -> PyResult<PyObject> {
        // Ensure subtasks are loaded before searching
        self.load_subtasks()?;

        for s in self.subtasks.as_ref().unwrap() {
            if s.name == name {
                if let Some(ref e) = entity {
                    if s.entity.as_ref() == Some(e) {
                        let py_sub = Py::new(py, s.clone()).map_err(|e| {
                            pyo3::exceptions::PyRuntimeError::new_err(e.to_string())
                        })?;
                        return Ok(py_sub.into_py(py));
                    }
                } else {
                    let py_sub = Py::new(py, s.clone())
                        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
                    return Ok(py_sub.into_py(py));
                }
            }
        }
        Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Task with name '{}' not found",
            name
        )))
    }
}

#[pymethods]
impl ParamType {
    pub fn __str__(&self) -> &'static str {
        self.name()
    }
    pub fn __repr__(&self) -> String {
        format!("ParamType.{}", self.name().to_uppercase())
    }
    #[getter]
    #[pyo3(name = "name")]
    fn param_type_name_py(&self) -> &'static str {
        self.name()
    }
    #[getter]
    #[pyo3(name = "aliases")]
    fn param_type_aliases_py(&self) -> Vec<&'static str> {
        self.aliases().to_vec()
    }
    #[getter]
    #[pyo3(name = "id")]
    fn param_type_id_py(&self) -> u8 {
        *self.id()
    }
    
    #[staticmethod]
    #[pyo3(name = "from_alias")]
    fn from_alias_py(alias: String) -> PyResult<ParamType> {
        ParamType::from_alias(&alias).map_err(|e| PyValueError::new_err(e))
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
        EtlStage::from_alias(&alias).map_err(|e| PyValueError::new_err(e))
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
        SystemType::from_alias(&alias).map_err(|e| PyValueError::new_err(e))
    }
}

#[pymethods]
impl TaskType {
    pub fn __str__(&self) -> &'static str {
        self.name()
    }

    pub fn __repr__(&self) -> String {
        format!("TaskType.{}", self.name().to_uppercase())
    }

    #[getter]
    #[pyo3(name = "id")]
    fn task_type_id_py(&self) -> u8 {
        *self.id()
    }

    #[getter]
    #[pyo3(name = "name")]
    fn task_type_name_py(&self) -> &'static str {
        self.name()
    }
    #[getter]
    #[pyo3(name = "extensions")]
    fn task_type_extensions_py(&self) -> Vec<&'static str> {
        self.extensions().to_vec()
    }

    #[staticmethod]
    #[pyo3(name = "from_extension")]
    fn from_extension_py(extension: String) -> PyResult<TaskType> {
        TaskType::from_extension(&extension).map_err(|e| PyValueError::new_err(e))
    }
}

#[pymethods]
impl Subtask {
    #[pyo3(name = "get_stored_params")]
    pub fn get_stored_params_py(&self, py: Python) -> PyResult<PyObject> {
        if let Some(stored) = &self.stored_params {
            // convert HashMap<String,String> -> Python dict
            let dict = PyDict::new_bound(py);
            for (k, v) in stored {
                dict.set_item(k, v)?;
            }
            Ok(dict.into())
        } else {
            // return empty dict
            Ok(PyDict::new_bound(py).into())
        }
    }

    #[pyo3(name = "get_params")]
    #[pyo3(signature = (styles=None))]
    pub fn get_params_py(
        &self,
        styles: Option<Vec<ParamType>>, // optional param styles from Python
        py: Python,                  // we need the GIL to build Python objects
    ) -> PyResult<PyObject> {
        // Map style names (strings) → ParamType

        // Call the Rust implementation
        let params = self.get_params(styles.as_deref());

        // Convert HashSet<String> → Python set
        let pyset = PySet::empty_bound(py)?;
        for name in params {
            pyset.add(name)?;
        }

        Ok(pyset.into())
    }

    /// Apply parameters from a Python dict to the subtask.
    /// params: dict-like mapping string->string
    /// styles: optional list of ParamType names, e.g. ["DollarBrace", "Curly"]
    /// ignore_missing: if true, missing placeholders are left unchanged; if false, raises ValueError
    #[pyo3(signature = (params, styles=None, ignore_missing=None))]
    #[pyo3(name = "apply_parameters")]
    pub fn apply_parameters_py(
        &mut self,
        // py: Python,
        params: &Bound<'_, PyDict>,
        styles: Option<Vec<ParamType>>,
        ignore_missing: Option<bool>,
    ) -> PyResult<()> {
        // convert params to HashMap<String,String>
        let mut map = HashMap::new();
        for item in params.items() {
            let (k, v): (Bound<PyAny>, Bound<PyAny>) = item.extract()?;
            let key = k.extract::<String>()?;
            
            // Convert any Python object to string using its __str__ method
            let val = v.str()?.to_string();
            map.insert(key, val);
        }

        self.stored_params = Some(map.clone());

        let ignore_missing = ignore_missing.unwrap_or(false);

        // call the Rust apply_parameters
        match self.apply_parameters(&map, styles.as_deref(), ignore_missing) {
            Ok(()) => Ok(()),
            Err(e) => Err(PyValueError::new_err(e)),
        }
    }
}

#[pymodule]
fn _core(m: Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SubtaskManager>()?;
    // m.add_class::<models::Subtask>()?;
    m.add_class::<Subtask>()?;
    m.add_class::<EtlStage>()?;
    m.add_class::<SystemType>()?;
    m.add_class::<ParamType>()?;
    m.add_class::<TaskType>()?;
    m.add_class::<FileScanner>()?;
    m.add_class::<FileClassifier>()?;
    Ok(())
}

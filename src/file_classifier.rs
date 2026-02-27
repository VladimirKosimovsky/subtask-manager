use crate::enums::{EtlStage, SystemType};
use crate::models::Subtask;
use crate::py_utils::py_path_to_string;
use anyhow::{bail, Result};
use pyo3::prelude::*;
use std::path::Path;
use strum::IntoEnumIterator;

/// FileClassifier classifies file paths into Subtask objects based on folder structure
#[pyclass]
#[derive(Clone)]
pub struct FileClassifier {
    pub(crate) base_path: String,
}

impl FileClassifier {
    /// Internal method for classification logic
    pub(crate) fn classify_internal(&self, file_path: &str) -> Result<Subtask> {
        let base = Path::new(&self.base_path);
        let file = Path::new(file_path);

        let mut sub = Subtask::new(file_path);

        // parts between base and file parent
        let base_components = base.components().count();
        let file_parent = file.parent().unwrap_or(file);
        let parts: Vec<_> = file_parent
            .components()
            .skip(base_components)
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();

        if parts.is_empty() {
            sub.is_common = true;
        }
        if parts.len() > 3 {
            bail!("Incorrect folder structure");
        }

        let mut checked_parts: Vec<String> = Vec::new();

        // simple stage detection: check if part matches known stage aliases
        for part in &parts {
            if sub.stage.is_none() {
                let detected_stage = EtlStage::from_alias(part).unwrap_or(EtlStage::Other);
                if detected_stage != EtlStage::Other {
                    sub.stage = Some(detected_stage);
                    checked_parts.push(part.clone());
                    break;
                }
            } else {
                break;
            }
        }

        // detect system type
        for part in &parts {
            if checked_parts.contains(part) {
                continue;
            }
            let l = part.to_lowercase();
            for system_type in SystemType::iter() {
                if system_type.aliases().contains(&l.as_str()) {
                    checked_parts.push(part.clone());
                    sub.system_type = Some(system_type);
                }
            }
        }

        // remaining candidate is entity
        let candidates: Vec<&String> = parts
            .iter()
            .filter(|p| !checked_parts.contains(p))
            .collect();
        if candidates.len() > 1 {
            bail!("Incorrect folder structure");
        }
        if let Some(ent) = candidates.first() {
            sub.entity = Some((*ent).clone());
        }

        // set task type by extension
        let sub = sub.set_task_type_from_ext();
        if sub.task_type.is_none() {
            bail!("Unknown task type");
        }

        Ok(sub)
    }
}

#[pymethods]
impl FileClassifier {
    #[new]
    fn new(base_path: &Bound<'_, PyAny>) -> PyResult<Self> {
        let base_path_str = py_path_to_string("base_path", base_path)?;
        Ok(FileClassifier {
            base_path: base_path_str,
        })
    }

    /// Classify a file path into a Subtask
    pub fn classify(&self, file_path: &Bound<'_, PyAny>) -> PyResult<Subtask> {
        let file_path_str = py_path_to_string("file_path", file_path)?;
        self.classify_internal(&file_path_str)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    #[getter]
    fn base_path(&self) -> String {
        self.base_path.clone()
    }

    fn __repr__(&self) -> String {
        format!("FileClassifier(base_path='{}')", self.base_path)
    }
}

use crate::enums::{SystemType, TaskType};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subtask {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub task_type: Option<TaskType>,
    #[pyo3(get)]
    pub system_type: Option<SystemType>,
    #[pyo3(get)]
    pub stage: Option<String>,
    #[pyo3(get)]
    pub entity: Option<String>,
    #[pyo3(get)]
    pub is_common: bool,
    #[pyo3(get)]
    pub command: Option<String>,
}

impl Subtask {
    pub fn new(path: &str) -> Self {
        let p = std::path::Path::new(path);
        Subtask {
            name: p
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default(),
            path: path.to_string(),
            task_type: None,
            system_type: None,
            stage: None,
            entity: None,
            is_common: false,
            command: None,
        }
    }

    pub fn set_task_type_from_ext(&mut self) {
        let ext = std::path::Path::new(&self.path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let tt = TaskType::from_extension(ext).unwrap_or(TaskType::Other);
        if tt != TaskType::Other {
            self.task_type = Some(tt);
        }
    }
}

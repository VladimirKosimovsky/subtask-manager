use pyo3::prelude::*;
use std::collections::HashSet;
use walkdir::WalkDir;

use crate::py_utils::py_path_to_string;

// FileScanner struct
#[pyclass]
pub struct FileScanner {
    extensions: HashSet<String>,
}

#[pymethods]
impl FileScanner {
    #[new]
    pub fn new(extensions: Vec<String>) -> Self {
        let mut exts = HashSet::new();
        for e in extensions {
            let normalized = e.trim_start_matches('.').to_lowercase();
            exts.insert(normalized);
        }
        FileScanner { extensions: exts }
    }

    pub fn scan_files(&self, base_dir: &Bound<'_, PyAny>) -> PyResult<Vec<String>> {
        // Convert base_dir to string, supporting both str and pathlib.Path
        let base_dir_str = py_path_to_string("base_dir", base_dir)?;

        let mut found: Vec<String> = Vec::new();

        for entry in WalkDir::new(&base_dir_str)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                    if self.extensions.contains(&ext.to_lowercase()) {
                        found.push(entry.path().to_string_lossy().to_string());
                    }
                }
            }
        }

        Ok(found)
    }

    #[getter]
    fn extensions(&self) -> Vec<String> {
        let mut extensions: Vec<String> = self.extensions.iter().cloned().collect();
        extensions.sort();
        extensions
    }
}

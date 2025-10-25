use pyo3::prelude::*;
use std::collections::HashSet;
use walkdir::WalkDir;

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
        let base_dir_str = if let Ok(path_str) = base_dir.extract::<String>() {
            // Direct string
            path_str
        } else if let Ok(path_obj) = base_dir.call_method0("__str__") {
            // pathlib.Path or other object with __str__ method
            path_obj.extract::<String>()?
        } else {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "base_dir must be a string or pathlib.Path object",
            ));
        };

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
        self.extensions.iter().cloned().collect()
    }
}

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

    pub fn scan_files(&self, base_dir: &str) -> PyResult<Vec<String>> {
        let mut found: Vec<String> = Vec::new();

        for entry in WalkDir::new(base_dir).into_iter().filter_map(|e| e.ok()) {
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

use std::collections::HashSet;
use walkdir::WalkDir;
use anyhow::Result;

pub fn scan_files(base_dir: &str, extensions: &[&str]) -> Result<Vec<String>> {
    let mut exts: HashSet<String> = HashSet::new();
    for e in extensions {
        let normalized = e.trim_start_matches('.').to_lowercase();
        exts.insert(normalized);
    }

    let mut found: Vec<String> = Vec::new();
    for entry in WalkDir::new(base_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                if exts.contains(&ext.to_lowercase()) {
                    found.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(found)
}

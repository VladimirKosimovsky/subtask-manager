use crate::models::Subtask;
use anyhow::{bail, Result};
use std::path::Path;

pub fn classify(base_path: &str, file_path: &str) -> Result<Subtask> {
    let base = Path::new(base_path);
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
        let l = part.to_lowercase();
        match l.as_str() {
            "00_setup" | "setup" | "s" | "00" => {
                sub.stage = Some("SETUP".to_string());
                checked_parts.push(part.clone());
            }
            "01_extract" | "extract" | "e" | "01" => {
                sub.stage = Some("EXTRACT".to_string());
                checked_parts.push(part.clone());
            }
            "02_transform" | "transform" | "t" | "02" => {
                sub.stage = Some("TRANSFORM".to_string());
                checked_parts.push(part.clone());
            }
            "03_load" | "load" | "l" | "03" => {
                sub.stage = Some("LOAD".to_string());
                checked_parts.push(part.clone());
            }
            "04_cleanup" | "cleanup" | "c" | "04" => {
                sub.stage = Some("CLEANUP".to_string());
                checked_parts.push(part.clone());
            }
            "05_post_processing" | "post_processing" | "pp" | "05" => {
                sub.stage = Some("POST_PROCESSING".to_string());
                checked_parts.push(part.clone());
            }
            _ => {}
        }
    }

    // detect system type
    for part in &parts {
        if checked_parts.contains(part) {
            continue;
        }
        let l = part.to_lowercase();
        match l.as_str() {
            "pg" | "postgres" | "pg_dwh" | "postgres_db" | "postgresdb" => {
                sub.system_type = Some("PG".to_string());
                checked_parts.push(part.clone());
            }
            "duck" | "duckdb" => {
                sub.system_type = Some("DUCK".to_string());
                checked_parts.push(part.clone());
            }
            "vertica" => {
                sub.system_type = Some("VERTICA".to_string());
                checked_parts.push(part.clone());
            }
            _ => {}
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
    if let Some(ent) = candidates.get(0) {
        sub.entity = Some((*ent).clone());
    }

    // set task type by extension
    sub.set_task_type_from_ext();
    if sub.task_type.is_none() {
        bail!("Unknown task type");
    }

    Ok(sub)
}

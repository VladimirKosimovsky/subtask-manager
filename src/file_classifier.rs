// use crate::etl_stage::EtlStage;
use crate::enums::{EtlStage, SystemType};
use crate::models::Subtask;
use anyhow::{bail, Result};
use std::path::Path;
use strum::IntoEnumIterator;

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
        if sub.stage.is_none() {
            let detected_stage = EtlStage::from_folder_name(&part);
            if detected_stage != EtlStage::Other {
                sub.stage = Some(detected_stage.as_str().to_string());
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

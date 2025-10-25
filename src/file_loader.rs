use crate::models::Subtask;
use anyhow::Result;
use std::fs;

pub fn load(mut subtask: Subtask) -> Result<Subtask> {
    let content = fs::read_to_string(&subtask.path)?;
    subtask.command = Some(content);
    Ok(subtask)
}

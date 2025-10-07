use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    SQL,
    SHELL,
    POWERSHELL,
    PYTHON,
    GRAPHQL,
    JSON,
    YAML,
    Unknown,
}

impl TaskType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "sql" | "psql" | "tsql" | "plpgsql" => TaskType::SQL,
            "sh" => TaskType::SHELL,
            "ps1" => TaskType::POWERSHELL,
            "py" => TaskType::PYTHON,
            "gql" | "graphql" => TaskType::GRAPHQL,
            "json" | "jsonl" => TaskType::JSON,
            "yaml" | "yml" => TaskType::YAML,
            _ => TaskType::Unknown,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            TaskType::SQL => "SQL",
            TaskType::SHELL => "SHELL",
            TaskType::POWERSHELL => "POWERSHELL",
            TaskType::PYTHON => "PYTHON",
            TaskType::GRAPHQL => "GRAPHQL",
            TaskType::JSON => "JSON",
            TaskType::YAML => "YAML",
            TaskType::Unknown => "UNKNOWN",
        }
    }
}

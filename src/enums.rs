use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use strum_macros::EnumIter;

#[derive(Debug, Clone)]
struct EtlStageData {
    name: &'static str,
    aliases: [&'static str; 4],
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Clone, Hash, Eq, Copy)]
pub enum EtlStage {
    Setup,
    Extract,
    Transform,
    Load,
    Cleanup,
    Postprocessing,
    Other,
}

impl EtlStage {
    fn etl_stage_data() -> &'static HashMap<EtlStage, EtlStageData> {
        static DATA: OnceLock<HashMap<EtlStage, EtlStageData>> = OnceLock::new();
        DATA.get_or_init(|| {
            HashMap::from([
                (
                    EtlStage::Extract,
                    EtlStageData {
                        name: "extract",
                        aliases: ["01_extract", "extract", "e", "01"],
                    },
                ),
                (
                    EtlStage::Transform,
                    EtlStageData {
                        name: "transform",
                        aliases: ["02_transform", "transform", "t", "02"],
                    },
                ),
                (
                    EtlStage::Load,
                    EtlStageData {
                        name: "load",
                        aliases: ["03_load", "load", "l", "03"],
                    },
                ),
                (
                    EtlStage::Setup,
                    EtlStageData {
                        name: "setup",
                        aliases: ["00_setup", "setup", "s", "00"],
                    },
                ),
                (
                    EtlStage::Cleanup,
                    EtlStageData {
                        name: "cleanup",
                        aliases: ["04_cleanup", "cleanup", "c", "04"],
                    },
                ),
                (
                    EtlStage::Postprocessing,
                    EtlStageData {
                        name: "post_processing",
                        aliases: ["05_post_processing", "post_processing", "pp", "05"],
                    },
                ),
                (
                    EtlStage::Other,
                    EtlStageData {
                        name: "other",
                        aliases: ["other", "misc", "unknown", "oth"],
                    },
                ),
            ])
        })
    }

    pub fn from_folder_name(folder_name: &str) -> EtlStage {
        for (stage, stage_info) in Self::etl_stage_data().iter() {
            if stage_info.name == folder_name || stage_info.aliases.contains(&folder_name) {
                return *stage;
            }
        }
        EtlStage::Other
    }

    pub fn as_str(&self) -> &str {
        match self {
            EtlStage::Setup => "SETUP",
            EtlStage::Extract => "EXTRACT",
            EtlStage::Transform => "TRANSFORM",
            EtlStage::Load => "LOAD",
            EtlStage::Cleanup => "CLEANUP",
            EtlStage::Postprocessing => "POSTPROCESSING",
            EtlStage::Other => "OTHER",
        }
    }

    pub fn name(&self) -> &'static str {
        &Self::etl_stage_data()[self].name
    }

    pub fn aliases(&self) -> &[&'static str; 4] {
        &Self::etl_stage_data()[self].aliases
    }
}

#[derive(Debug, Clone)]
struct SystemTypeData {
    id: &'static u8,
    name: &'static str,
    aliases: Vec<&'static str>,
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Clone, Hash, Eq, Copy, EnumIter, Serialize, Deserialize)]
pub enum SystemType {
    Clickhouse,
    Duckdb,
    MySQL,
    OracleDB,
    PostgreSQL,
    SQLite,
    SqlServer,
    Vertica,
    Other,
}

impl SystemType {
    fn system_type_data() -> &'static HashMap<SystemType, SystemTypeData> {
        static DATA: OnceLock<HashMap<SystemType, SystemTypeData>> = OnceLock::new();
        DATA.get_or_init(|| {
            HashMap::from([
                (
                    SystemType::Clickhouse,
                    SystemTypeData {
                        id: &0,
                        name: "clickhouse",
                        aliases: vec!["clickhouse", "click", "ch"],
                    },
                ),
                (
                    SystemType::Duckdb,
                    SystemTypeData {
                        id: &1,
                        name: "duckdb",
                        aliases: vec!["duckdb", "duck", "ddb"],
                    },
                ),
                (
                    SystemType::MySQL,
                    SystemTypeData {
                        id: &2,
                        name: "mysql",
                        aliases: vec!["mysql"],
                    },
                ),
                (
                    SystemType::OracleDB,
                    SystemTypeData {
                        id: &3,
                        name: "oracle",
                        aliases: vec!["oracledb", "oracle", "plsql"],
                    },
                ),
                (
                    SystemType::PostgreSQL,
                    SystemTypeData {
                        id: &4,
                        name: "postgres",
                        aliases: vec!["pg", "postgres", "pg_dwh", "postgres_db", "postgresdb"],
                    },
                ),
                (
                    SystemType::SQLite,
                    SystemTypeData {
                        id: &5,
                        name: "sqlite",
                        aliases: vec!["sqlite"],
                    },
                ),
                (
                    SystemType::SqlServer,
                    SystemTypeData {
                        id: &6,
                        name: "sqlserver",
                        aliases: vec!["sqlserver", "mssql"],
                    },
                ),
                (
                    SystemType::Vertica,
                    SystemTypeData {
                        id: &7,
                        name: "vertica",
                        aliases: vec!["vertica", "vertica"],
                    },
                ),
                (
                    SystemType::Other,
                    SystemTypeData {
                        id: &8,
                        name: "other",
                        aliases: vec!["other", "unknown", "misc"],
                    },
                ),
            ])
        })
    }
    pub fn from_folder_name(folder_name: &str) -> SystemType {
        for (system_type, system_type_info) in Self::system_type_data().iter() {
            if system_type_info.name == folder_name
                || system_type_info.aliases.contains(&folder_name)
            {
                return *system_type;
            }
        }
        SystemType::Other
    }
    pub fn as_str(&self) -> &str {
        match self {
            SystemType::Clickhouse => "CLICKHOUSE",
            SystemType::Duckdb => "DUCKDB",
            SystemType::MySQL => "MYSQL",
            SystemType::OracleDB => "ORACLEDB",
            SystemType::PostgreSQL => "POSTGRESQL",
            SystemType::SQLite => "SQLITE",
            SystemType::SqlServer => "SQLSERVER",
            SystemType::Vertica => "VERTICA",
            SystemType::Other => "OTHER",
        }
    }
    pub fn id(&self) -> &'static u8 {
        &Self::system_type_data()[self].id
    }
    pub fn name(&self) -> &'static str {
        &Self::system_type_data()[self].name
    }

    pub fn aliases(&self) -> &Vec<&'static str> {
        &Self::system_type_data()[self].aliases
    }
}

#[pyclass(eq, eq_int)]
#[derive(Clone, Debug, PartialEq)]
pub enum TaskType {
    Sql,
    Shell,
    Powershell,
    Python,
    Graphql,
    Json,
    Yaml,
    Unknown,
}

impl TaskType {
    pub fn from_extension(ext: &str) -> TaskType {
        match ext.to_lowercase().as_str() {
            "sql" | "psql" | "tsql" | "plpgsql" => TaskType::Sql,
            "sh" => TaskType::Shell,
            "ps1" => TaskType::Powershell,
            "py" => TaskType::Python,
            "graphql" | "gql" => TaskType::Graphql,
            "json" | "jsonl" => TaskType::Json,
            "yaml" | "yml" => TaskType::Yaml,
            _ => TaskType::Unknown,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            TaskType::Sql => "SQL",
            TaskType::Shell => "SHELL",
            TaskType::Powershell => "POWERSHELL",
            TaskType::Python => "PYTHON",
            TaskType::Graphql => "GRAPHQL",
            TaskType::Json => "JSON",
            TaskType::Yaml => "YAML",
            TaskType::Unknown => "UNKNOWN",
        }
    }
}

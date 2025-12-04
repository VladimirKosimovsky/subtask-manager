use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;
use strum_macros::EnumIter;

/* ============================================================================================
 *  ParamType  (Unified with other enums — no regex(), pure metadata)
 * ============================================================================================ */

#[derive(Debug, Clone)]
struct ParamTypeData {
    id: &'static u8,
    name: &'static str,
    aliases: Vec<&'static str>,
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParamType {
    Curly,            // {param}
    Dollar,           // $param
    DollarBrace,      // ${param}
    DoubleUnderscore, // __param__
    Percent,          // %param%
    Angle,            // <param>
    Other,            // fallback / no match
}

impl ParamType {
    pub const ALL: &'static [ParamType] = &[
        ParamType::DollarBrace,
        ParamType::Curly,
        ParamType::Dollar,
        ParamType::DoubleUnderscore,
        ParamType::Percent,
        ParamType::Angle,
    ];
    fn param_type_data() -> &'static HashMap<ParamType, ParamTypeData> {
        static DATA: OnceLock<HashMap<ParamType, ParamTypeData>> = OnceLock::new();
        DATA.get_or_init(|| {
            HashMap::from([
                (
                    ParamType::Curly,
                    ParamTypeData {
                        id: &0,
                        name: "curly",
                        aliases: vec!["curly", "curley", "{name}"],
                    },
                ),
                (
                    ParamType::Dollar,
                    ParamTypeData {
                        id: &1,
                        name: "dollar",
                        aliases: vec!["dollar", "$name"],
                    },
                ),
                (
                    ParamType::DollarBrace,
                    ParamTypeData {
                        id: &2,
                        name: "dollar_brace",
                        aliases: vec!["dollarbrace", "dollar_brace", "${name}"],
                    },
                ),
                (
                    ParamType::DoubleUnderscore,
                    ParamTypeData {
                        id: &3,
                        name: "double_underscore",
                        aliases: vec!["doubleunderscore", "__name__", "__NAME__"],
                    },
                ),
                (
                    ParamType::Percent,
                    ParamTypeData {
                        id: &4,
                        name: "percent",
                        aliases: vec!["percent", "%name%"],
                    },
                ),
                (
                    ParamType::Angle,
                    ParamTypeData {
                        id: &5,
                        name: "angle",
                        aliases: vec!["angle", "<name>"],
                    },
                ),
                (
                    ParamType::Other,
                    ParamTypeData {
                        id: &6,
                        name: "other",
                        aliases: vec!["other"],
                    },
                ),
            ])
        })
    }

    pub fn from_alias(alias: &str) -> Result<ParamType, String> {
        let alias_lower = alias.to_lowercase();
        for (ptype, data) in Self::param_type_data().iter() {
            if data.name == alias_lower || data.aliases.iter().any(|&a| a == alias_lower) {
                return Ok(*ptype);
            }
        }
        Err(format!("Unknown ParamType alias: {}", alias))
    }

    pub fn as_str(&self) -> &'static str {
        Self::param_type_data()[self].name
    }

    pub fn id(&self) -> &'static u8 {
        Self::param_type_data()[self].id
    }

    pub fn name(&self) -> &'static str {
        Self::param_type_data()[self].name
    }

    pub fn aliases(&self) -> &Vec<&'static str> {
        &Self::param_type_data()[self].aliases
    }
}

impl fmt::Display for ParamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/* ============================================================================================
 *  EtlStage
 * ============================================================================================ */

#[derive(Debug, Clone)]
struct EtlStageData {
    id: &'static u8,
    name: &'static str,
    aliases: [&'static str; 4],
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Clone, Hash, Eq, Copy, Deserialize, Serialize)]
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
                    EtlStage::Setup,
                    EtlStageData {
                        id: &0,
                        name: "setup",
                        aliases: ["00_setup", "setup", "s", "00"],
                    },
                ),
                (
                    EtlStage::Extract,
                    EtlStageData {
                        id: &1,
                        name: "extract",
                        aliases: ["01_extract", "extract", "e", "01"],
                    },
                ),
                (
                    EtlStage::Transform,
                    EtlStageData {
                        id: &2,
                        name: "transform",
                        aliases: ["02_transform", "transform", "t", "02"],
                    },
                ),
                (
                    EtlStage::Load,
                    EtlStageData {
                        id: &3,
                        name: "load",
                        aliases: ["03_load", "load", "l", "03"],
                    },
                ),
                (
                    EtlStage::Cleanup,
                    EtlStageData {
                        id: &4,
                        name: "cleanup",
                        aliases: ["04_cleanup", "cleanup", "c", "04"],
                    },
                ),
                (
                    EtlStage::Postprocessing,
                    EtlStageData {
                        id: &5,
                        name: "post_processing",
                        aliases: ["05_post_processing", "post_processing", "pp", "05"],
                    },
                ),
                (
                    EtlStage::Other,
                    EtlStageData {
                        id: &6,
                        name: "other",
                        aliases: ["other", "misc", "unknown", "oth"],
                    },
                ),
            ])
        })
    }

    pub fn from_alias(alias: &str) -> Result<EtlStage, String> {
        let alias_lower = alias.to_lowercase();
        for (stage, data) in Self::etl_stage_data().iter() {
            if data.name == alias_lower || data.aliases.iter().any(|&a| a == alias_lower) {
                return Ok(*stage);
            }
        }
        Err(format!("Unknown ETL stage alias: {}", alias))
    }

    pub fn as_str(&self) -> &'static str {
        Self::etl_stage_data()[self].name
    }

    pub fn id(&self) -> &'static u8 {
        Self::etl_stage_data()[self].id
    }

    pub fn name(&self) -> &'static str {
        Self::etl_stage_data()[self].name
    }

    pub fn aliases(&self) -> &[&'static str; 4] {
        &Self::etl_stage_data()[self].aliases
    }
}

/* ============================================================================================
 *  SystemType
 * ============================================================================================ */

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
                        aliases: vec!["sqlserver", "mssql", "tsql"],
                    },
                ),
                (
                    SystemType::Vertica,
                    SystemTypeData {
                        id: &7,
                        name: "vertica",
                        aliases: vec!["vertica"],
                    },
                ),
                (
                    SystemType::Other,
                    SystemTypeData {
                        id: &8,
                        name: "other",
                        aliases: vec![],
                    },
                ),
            ])
        })
    }

    pub fn from_alias(alias: &str) -> Result<SystemType, String> {
        let alias_lower = alias.to_lowercase();
        for (system_type, data) in Self::system_type_data().iter() {
            if data.name == alias_lower || data.aliases.iter().any(|a| a == &alias_lower) {
                return Ok(*system_type);
            }
        }
        Err(format!("Unknown system type alias: {}", alias))
    }

    pub fn as_str(&self) -> &'static str {
        Self::system_type_data()[self].name
    }

    pub fn id(&self) -> &'static u8 {
        Self::system_type_data()[self].id
    }

    pub fn name(&self) -> &'static str {
        Self::system_type_data()[self].name
    }

    pub fn aliases(&self) -> &Vec<&'static str> {
        &Self::system_type_data()[self].aliases
    }
}

/* ============================================================================================
 *  TaskType
 * ============================================================================================ */

#[derive(Debug, Clone)]
struct TaskTypeData {
    id: &'static u8,
    name: &'static str,
    extensions: Vec<&'static str>,
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Clone, Hash, Eq, Copy, EnumIter, Serialize, Deserialize)]
pub enum TaskType {
    Sql,
    Shell,
    Powershell,
    Python,
    Graphql,
    Json,
    Yaml,
    Other,
}

impl TaskType {
    fn task_type_data() -> &'static HashMap<TaskType, TaskTypeData> {
        static DATA: OnceLock<HashMap<TaskType, TaskTypeData>> = OnceLock::new();
        DATA.get_or_init(|| {
            HashMap::from([
                (
                    TaskType::Sql,
                    TaskTypeData {
                        id: &0,
                        name: "sql",
                        extensions: vec!["sql", "psql", "tsql", "plpgsql"],
                    },
                ),
                (
                    TaskType::Shell,
                    TaskTypeData {
                        id: &1,
                        name: "shell",
                        extensions: vec!["sh"],
                    },
                ),
                (
                    TaskType::Powershell,
                    TaskTypeData {
                        id: &2,
                        name: "powershell",
                        extensions: vec!["ps1"],
                    },
                ),
                (
                    TaskType::Python,
                    TaskTypeData {
                        id: &3,
                        name: "python",
                        extensions: vec!["py"],
                    },
                ),
                (
                    TaskType::Graphql,
                    TaskTypeData {
                        id: &4,
                        name: "graphql",
                        extensions: vec!["graphql", "gql"],
                    },
                ),
                (
                    TaskType::Json,
                    TaskTypeData {
                        id: &5,
                        name: "json",
                        extensions: vec!["json", "jsonl"],
                    },
                ),
                (
                    TaskType::Yaml,
                    TaskTypeData {
                        id: &6,
                        name: "yaml",
                        extensions: vec!["yaml", "yml"],
                    },
                ),
                (
                    TaskType::Other,
                    TaskTypeData {
                        id: &8,
                        name: "other",
                        extensions: vec![],
                    },
                ),
            ])
        })
    }

    pub fn from_extension(ext: &str) -> Result<TaskType, String> {
        let alias_lower = ext.to_lowercase();
        for (task_type, data) in Self::task_type_data().iter() {
            if data.name == alias_lower || data.extensions.iter().any(|&e| e == alias_lower) {
                return Ok(*task_type);
            }
        }
        Err(format!("Unknown task type extension: {}", ext))
    }

    pub fn as_str(&self) -> &'static str {
        Self::task_type_data()[self].name
    }

    pub fn id(&self) -> &'static u8 {
        Self::task_type_data()[self].id
    }

    pub fn name(&self) -> &'static str {
        Self::task_type_data()[self].name
    }

    pub fn extensions(&self) -> &Vec<&'static str> {
        &Self::task_type_data()[self].extensions
    }
}

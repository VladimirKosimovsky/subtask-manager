use once_cell::sync::OnceCell;
use pyo3::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;
use strum_macros::EnumIter;

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Clone, Hash, Eq, Copy, Serialize, Deserialize)]
pub enum ParamType {
    /// {name}
    Curly,
    /// $name
    Dollar,
    /// ${name}
    DollarBrace,
    /// __NAME__ (often all-caps)
    DoubleUnderscore,
    /// %name%
    Percent,
    /// <name>
    Angle,
    /// any custom regex (not exposed to python directly)
    Other,
}

impl ParamType {
    /// Return a compiled Regex for this ParamType. The capture group "name" holds parameter name.
    pub fn regex(&self) -> &'static Regex {
        static RE_CURLEY: OnceCell<Regex> = OnceCell::new();
        static RE_DOLLAR: OnceCell<Regex> = OnceCell::new();
        static RE_DOLLAR_BRACE: OnceCell<Regex> = OnceCell::new();
        static RE_DOUBLE_UNDERSCORE: OnceCell<Regex> = OnceCell::new();
        static RE_PERCENT: OnceCell<Regex> = OnceCell::new();
        static RE_ANGLE: OnceCell<Regex> = OnceCell::new();

        match self {
            ParamType::Curly => RE_CURLEY.get_or_init(|| {
                Regex::new(r"\{(?P<name>[A-Za-z0-9_.:-]+)\}").expect("valid regex")
            }),
            ParamType::Dollar => RE_DOLLAR.get_or_init(|| {
                // $name but not $$ or $1 etc; stop at word boundary
                Regex::new(r"(?P<full>\$(?P<name>[A-Za-z_][A-Za-z0-9_.-]*))\b")
                    .expect("valid regex")
            }),
            ParamType::DollarBrace => RE_DOLLAR_BRACE.get_or_init(|| {
                // ${name}
                Regex::new(r"\$\{(?P<name>[A-Za-z0-9_.:-]+)\}").expect("valid regex")
            }),
            ParamType::DoubleUnderscore => RE_DOUBLE_UNDERSCORE.get_or_init(|| {
                // __NAME__
                Regex::new(r"__(?P<name>[A-Za-z0-9_]+)__").expect("valid regex")
            }),
            ParamType::Percent => RE_PERCENT
                .get_or_init(|| Regex::new(r"%(?P<name>[A-Za-z0-9_.:-]+)%").expect("valid regex")),
            ParamType::Angle => RE_ANGLE
                .get_or_init(|| Regex::new(r"<(?P<name>[A-Za-z0-9_.:-]+)>").expect("valid regex")),
            ParamType::Other => RE_CURLEY.get_or_init(|| {
                Regex::new(r"\{(?P<name>[A-Za-z0-9_.:-]+)\}").expect("valid regex")
            }),
        }
    }

    /// Friendly display name
    pub fn as_str(&self) -> &'static str {
        match self {
            ParamType::Curly => "curly {name}",
            ParamType::Dollar => "dollar $name",
            ParamType::DollarBrace => "dollar_brace ${name}",
            ParamType::DoubleUnderscore => "double_underscore __NAME__",
            ParamType::Percent => "percent %name%",
            ParamType::Angle => "angle <name>",
            ParamType::Other => "other",
        }
    }

    /// Default list of param types to try when user doesn't specify
    pub fn default_order() -> Vec<ParamType> {
        vec![
            ParamType::DollarBrace,
            ParamType::Curly,
            ParamType::Dollar,
            ParamType::DoubleUnderscore,
            ParamType::Percent,
            ParamType::Angle,
        ]
    }
}

impl fmt::Display for ParamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

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
        for (stage, stage_info) in Self::etl_stage_data().iter() {
            if stage_info.name == alias_lower
                || stage_info.aliases.iter().any(|&a| a == alias_lower)
            {
                return Ok(*stage);
            }
        }
        Err(format!("Unknown ETL stage alias: {}", alias))
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
    pub fn id(&self) -> &'static u8 {
        &Self::etl_stage_data()[self].id
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
                        aliases: vec![],
                    },
                ),
            ])
        })
    }
    pub fn from_alias(alias: &str) -> Result<SystemType, String> {
        let alias_lower = alias.to_lowercase();
        for (system_type, system_type_info) in Self::system_type_data().iter() {
            if system_type_info.name == alias_lower
                || system_type_info.aliases.iter().any(|&a| a == alias_lower)
            {
                return Ok(*system_type);
            }
        }
        Err(format!("Unknown system type alias: {}", alias))
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
    pub fn from_extension(alias: &str) -> Result<TaskType, String> {
        let alias_lower = alias.to_lowercase();
        for (task_type, task_type_info) in Self::task_type_data().iter() {
            if task_type_info.name == alias_lower
                || task_type_info.extensions.iter().any(|&a| a == alias_lower)
            {
                return Ok(*task_type);
            }
        }
        Err(format!("Unknown task type alias: {}", alias))
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
            TaskType::Other => "OTHER",
        }
    }
    pub fn id(&self) -> &'static u8 {
        &Self::task_type_data()[self].id
    }
    pub fn name(&self) -> &'static str {
        &Self::task_type_data()[self].name
    }

    pub fn extensions(&self) -> &Vec<&'static str> {
        &Self::task_type_data()[self].extensions
    }
}

//! Dialect-aware analysis of a task's SQL body.
//!
//! Where [`crate::sql_guard`] inspects individual *values*, this module parses
//! the whole statement with [`sqlparser`] and reports what it actually does:
//! which statement kinds it runs, which tables it touches, and whether it does
//! something irreversible.
//!
//! **Analysis always fails open.** Real ETL SQL is full of vendor syntax no
//! parser knows (`CREATE PERSISTENT SECRET`, DuckDB `ATTACH ... (TYPE POSTGRES)`),
//! so an unparseable body yields `parsed = false` and an empty verdict — never a
//! rejection. Treat a negative result as "not analyzed", not as "safe".

use std::collections::HashMap;
use std::fmt;
use std::ops::ControlFlow;
use std::sync::OnceLock;

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use sqlparser::ast::{visit_relations, FromTable, Statement};
use sqlparser::dialect::dialect_from_str;
use sqlparser::parser::Parser;
use strum_macros::EnumIter;

use crate::enums::SystemType;

/* ============================================================================================
 *  StatementKind
 * ============================================================================================ */

#[derive(Debug, Clone)]
struct StatementKindData {
    id: u8,
    name: &'static str,
    aliases: Vec<&'static str>,
    destructive: bool,
}

/// The kind of statement found in a task body.
#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Clone, Hash, Eq, Copy, EnumIter, Serialize, Deserialize)]
pub enum StatementKind {
    Select,
    Insert,
    Update,
    Delete,
    Merge,
    Truncate,
    Drop,
    Create,
    Alter,
    Grant,
    Revoke,
    Copy,
    Call,
    Transaction,
    Other,
}

impl StatementKind {
    fn statement_kind_data() -> &'static HashMap<StatementKind, StatementKindData> {
        static DATA: OnceLock<HashMap<StatementKind, StatementKindData>> = OnceLock::new();
        DATA.get_or_init(|| {
            let entry = |id, name, aliases: Vec<&'static str>, destructive| StatementKindData {
                id,
                name,
                aliases,
                destructive,
            };
            HashMap::from([
                (
                    StatementKind::Select,
                    entry(0, "select", vec!["select", "query", "read"], false),
                ),
                (
                    StatementKind::Insert,
                    entry(1, "insert", vec!["insert", "into"], false),
                ),
                (
                    StatementKind::Update,
                    entry(2, "update", vec!["update"], false),
                ),
                (
                    StatementKind::Delete,
                    entry(3, "delete", vec!["delete", "del"], true),
                ),
                (
                    StatementKind::Merge,
                    entry(4, "merge", vec!["merge", "upsert"], false),
                ),
                (
                    StatementKind::Truncate,
                    entry(5, "truncate", vec!["truncate"], true),
                ),
                (StatementKind::Drop, entry(6, "drop", vec!["drop"], true)),
                (
                    StatementKind::Create,
                    entry(7, "create", vec!["create", "ddl_create"], false),
                ),
                (
                    StatementKind::Alter,
                    entry(8, "alter", vec!["alter"], false),
                ),
                (
                    StatementKind::Grant,
                    entry(9, "grant", vec!["grant"], false),
                ),
                (
                    StatementKind::Revoke,
                    entry(10, "revoke", vec!["revoke"], false),
                ),
                (
                    StatementKind::Copy,
                    entry(11, "copy", vec!["copy", "load"], false),
                ),
                (
                    StatementKind::Call,
                    entry(12, "call", vec!["call", "execute", "exec"], false),
                ),
                (
                    StatementKind::Transaction,
                    entry(
                        13,
                        "transaction",
                        vec!["transaction", "begin", "commit", "rollback"],
                        false,
                    ),
                ),
                (
                    StatementKind::Other,
                    entry(14, "other", vec!["other", "unknown"], false),
                ),
            ])
        })
    }

    pub fn from_alias(alias: &str) -> Result<StatementKind, String> {
        let alias_lower = alias.to_lowercase();
        for (kind, data) in Self::statement_kind_data().iter() {
            if data.name == alias_lower || data.aliases.iter().any(|&a| a == alias_lower) {
                return Ok(*kind);
            }
        }
        Err(format!("Unknown StatementKind alias: {}", alias))
    }

    pub fn id(&self) -> u8 {
        Self::statement_kind_data()[self].id
    }

    pub fn name(&self) -> &'static str {
        Self::statement_kind_data()[self].name
    }

    pub fn aliases(&self) -> &Vec<&'static str> {
        &Self::statement_kind_data()[self].aliases
    }

    /// True for statements that destroy data or schema irreversibly.
    pub fn is_destructive(&self) -> bool {
        Self::statement_kind_data()[self].destructive
    }

    fn of(statement: &Statement) -> StatementKind {
        match statement {
            Statement::Query(_) => StatementKind::Select,
            Statement::Insert(_) => StatementKind::Insert,
            Statement::Update(_) => StatementKind::Update,
            Statement::Delete(_) => StatementKind::Delete,
            Statement::Merge(_) => StatementKind::Merge,
            Statement::Truncate(_) => StatementKind::Truncate,
            Statement::Drop { .. }
            | Statement::DropFunction { .. }
            | Statement::DropProcedure { .. } => StatementKind::Drop,
            Statement::CreateTable(_)
            | Statement::CreateView { .. }
            | Statement::CreateIndex(_)
            | Statement::CreateSchema { .. }
            | Statement::CreateDatabase { .. }
            | Statement::CreateFunction(_) => StatementKind::Create,
            Statement::AlterTable { .. } | Statement::AlterIndex { .. } => StatementKind::Alter,
            Statement::Grant(_) => StatementKind::Grant,
            Statement::Revoke(_) => StatementKind::Revoke,
            Statement::Copy { .. } => StatementKind::Copy,
            Statement::Call(_) | Statement::Execute { .. } => StatementKind::Call,
            Statement::StartTransaction { .. }
            | Statement::Commit { .. }
            | Statement::Rollback { .. } => StatementKind::Transaction,
            _ => StatementKind::Other,
        }
    }
}

impl fmt::Display for StatementKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/* ============================================================================================
 *  SqlAnalysis
 * ============================================================================================ */

/// What a SQL body was found to do.
#[pyclass]
#[derive(Debug, Clone)]
pub struct SqlAnalysis {
    /// False when the dialect could not parse the body — nothing below was
    /// determined, and no conclusion of safety may be drawn from it.
    #[pyo3(get)]
    pub parsed: bool,
    /// Dialect the body was parsed with.
    #[pyo3(get)]
    pub dialect: String,
    /// Statement kinds, in the order they appear.
    #[pyo3(get)]
    pub statements: Vec<StatementKind>,
    /// Every relation referenced, deduplicated, in first-seen order.
    #[pyo3(get)]
    pub tables: Vec<String>,
    /// Human-readable notes about risky constructs.
    #[pyo3(get)]
    pub warnings: Vec<String>,
    /// Parser message when `parsed` is false.
    #[pyo3(get)]
    pub error: Option<String>,
}

impl SqlAnalysis {
    pub(crate) fn unparsed(dialect: &str, error: String) -> Self {
        SqlAnalysis {
            parsed: false,
            dialect: dialect.to_string(),
            statements: Vec::new(),
            tables: Vec::new(),
            warnings: Vec::new(),
            error: Some(error),
        }
    }

    /// Statement kinds present that appear in `forbidden`.
    pub fn forbidden_hits(&self, forbidden: &[StatementKind]) -> Vec<StatementKind> {
        let mut hits: Vec<StatementKind> = self
            .statements
            .iter()
            .filter(|k| forbidden.contains(k))
            .copied()
            .collect();
        hits.dedup();
        hits
    }

    pub fn is_destructive(&self) -> bool {
        self.statements.iter().any(|k| k.is_destructive())
    }
}

/// Dialect name understood by `sqlparser` for a given system.
///
/// Vertica has no dedicated dialect in `sqlparser`; `generic` is the closest fit.
pub fn dialect_name(system_type: Option<SystemType>) -> &'static str {
    match system_type {
        Some(SystemType::Clickhouse) => "clickhouse",
        Some(SystemType::Duckdb) => "duckdb",
        Some(SystemType::MySQL) => "mysql",
        Some(SystemType::OracleDB) => "oracle",
        Some(SystemType::PostgreSQL) => "postgres",
        Some(SystemType::SQLite) => "sqlite",
        Some(SystemType::SqlServer) => "mssql",
        Some(SystemType::Vertica) | Some(SystemType::Other) | None => "generic",
    }
}

fn relations_of(statement: &Statement) -> Vec<String> {
    let mut tables = Vec::new();
    let _ = visit_relations(statement, |name| {
        tables.push(name.to_string());
        ControlFlow::<()>::Continue(())
    });
    // `Statement::Drop` holds its targets in `names`, which the relation
    // visitor does not descend into — collect them by hand.
    if let Statement::Drop { names, .. } = statement {
        for name in names {
            tables.push(name.to_string());
        }
    }
    tables
}

fn warn_statement(statement: &Statement, warnings: &mut Vec<String>) {
    match statement {
        Statement::Delete(delete) if delete.selection.is_none() => {
            let targets = match &delete.from {
                FromTable::WithFromKeyword(t) | FromTable::WithoutKeyword(t) => t
                    .iter()
                    .map(|x| x.relation.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            };
            warnings.push(format!(
                "DELETE on {} has no WHERE clause: it removes every row",
                targets
            ));
        }
        Statement::Update(update) if update.selection.is_none() => {
            warnings.push(format!(
                "UPDATE on {} has no WHERE clause: it rewrites every row",
                update.table.relation
            ));
        }
        Statement::Truncate(truncate) => {
            let targets = truncate
                .table_names
                .iter()
                .map(|t| t.name.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            warnings.push(format!("TRUNCATE on {} removes every row", targets));
        }
        Statement::Drop {
            object_type, names, ..
        } => {
            let targets = names
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            warnings.push(format!(
                "DROP {} on {} is irreversible",
                format!("{:?}", object_type).to_uppercase(),
                targets
            ));
        }
        _ => {}
    }
}

/// Parse `sql` with `dialect` and report what it does. Never fails.
pub fn analyze(sql: &str, dialect: &str) -> SqlAnalysis {
    let Some(parser_dialect) = dialect_from_str(dialect) else {
        return SqlAnalysis::unparsed(dialect, format!("unknown SQL dialect: {}", dialect));
    };

    let statements = match Parser::parse_sql(parser_dialect.as_ref(), sql) {
        Ok(statements) => statements,
        Err(e) => return SqlAnalysis::unparsed(dialect, e.to_string()),
    };

    let mut kinds = Vec::with_capacity(statements.len());
    let mut tables: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    for statement in &statements {
        kinds.push(StatementKind::of(statement));
        for table in relations_of(statement) {
            if !tables.contains(&table) {
                tables.push(table);
            }
        }
        warn_statement(statement, &mut warnings);
    }

    if statements.len() > 1 {
        warnings.push(format!(
            "body contains {} statements; many drivers execute only the first",
            statements.len()
        ));
    }

    SqlAnalysis {
        parsed: true,
        dialect: dialect.to_string(),
        statements: kinds,
        tables,
        warnings,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_destructive_statements() {
        let a = analyze("DROP TABLE users", "postgres");
        assert!(a.parsed);
        assert_eq!(a.statements, vec![StatementKind::Drop]);
        assert_eq!(a.tables, vec!["users"]);
        assert!(a.is_destructive());

        let a = analyze("TRUNCATE TABLE staging.events", "postgres");
        assert_eq!(a.statements, vec![StatementKind::Truncate]);
        assert_eq!(a.tables, vec!["staging.events"]);
        assert!(a.is_destructive());
    }

    #[test]
    fn flags_delete_without_where() {
        let a = analyze("DELETE FROM public.users", "postgres");
        assert_eq!(a.statements, vec![StatementKind::Delete]);
        assert!(a.warnings.iter().any(|w| w.contains("no WHERE clause")));

        let a = analyze("DELETE FROM public.users WHERE id = 1", "postgres");
        assert!(!a.warnings.iter().any(|w| w.contains("no WHERE clause")));
    }

    #[test]
    fn flags_update_without_where() {
        let a = analyze("UPDATE t SET a = 1", "mysql");
        assert_eq!(a.statements, vec![StatementKind::Update]);
        assert!(a.warnings.iter().any(|w| w.contains("rewrites every row")));
    }

    #[test]
    fn select_is_not_destructive() {
        let a = analyze("SELECT * FROM users WHERE id = 1", "clickhouse");
        assert_eq!(a.statements, vec![StatementKind::Select]);
        assert_eq!(a.tables, vec!["users"]);
        assert!(!a.is_destructive());
        assert!(a.warnings.is_empty());
    }

    #[test]
    fn counts_stacked_statements() {
        let a = analyze(
            "SELECT * FROM users WHERE name = 'x'; DROP TABLE users;",
            "postgres",
        );
        assert_eq!(
            a.statements,
            vec![StatementKind::Select, StatementKind::Drop]
        );
        assert!(a.warnings.iter().any(|w| w.contains("2 statements")));
    }

    #[test]
    fn unparseable_sql_fails_open() {
        // Vendor syntax from the repo's own DuckDB fixture: `PORT 5432` in a
        // secret definition is not something any dialect here can read.
        let a = analyze(
            "CREATE PERSISTENT SECRET s (TYPE POSTGRES, HOST 'h', PORT 5432)",
            "duckdb",
        );
        assert!(!a.parsed);
        assert!(a.error.is_some());
        assert!(a.statements.is_empty());
        assert!(!a.is_destructive());
    }

    #[test]
    fn unknown_dialect_fails_open() {
        let a = analyze("SELECT 1", "nonesuch");
        assert!(!a.parsed);
        assert!(a.error.unwrap().contains("unknown SQL dialect"));
    }

    #[test]
    fn dialect_names_map_onto_system_types() {
        assert_eq!(dialect_name(Some(SystemType::PostgreSQL)), "postgres");
        assert_eq!(dialect_name(Some(SystemType::SqlServer)), "mssql");
        assert_eq!(dialect_name(Some(SystemType::Vertica)), "generic");
        assert_eq!(dialect_name(None), "generic");
        // Every mapping must be a dialect sqlparser actually knows.
        for st in [
            SystemType::Clickhouse,
            SystemType::Duckdb,
            SystemType::MySQL,
            SystemType::OracleDB,
            SystemType::PostgreSQL,
            SystemType::SQLite,
            SystemType::SqlServer,
            SystemType::Vertica,
            SystemType::Other,
        ] {
            assert!(
                dialect_from_str(dialect_name(Some(st))).is_some(),
                "no dialect for {:?}",
                st
            );
        }
    }

    #[test]
    fn forbidden_hits() {
        let a = analyze("DROP TABLE a; TRUNCATE TABLE b", "postgres");
        let hits = a.forbidden_hits(&[StatementKind::Drop]);
        assert_eq!(hits, vec![StatementKind::Drop]);
        assert!(a.forbidden_hits(&[StatementKind::Select]).is_empty());
    }
}

use crate::enums::{EtlStage, ParamStyle, SqlParamStyle, SystemType, TaskType};
use crate::sql_analysis::{self, SqlAnalysis, StatementKind};
use crate::sql_binding::{bind_text, BoundQuery};
use crate::sql_guard;
use once_cell::sync::OnceCell;
use pyo3::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use strum::IntoEnumIterator;

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subtask {
    /// Original name template (never mutated)
    pub original_name: String,
    /// Original path template (never mutated)
    pub original_path: String,

    /// Rendered name with parameters applied
    #[pyo3(get)]
    pub name: String,
    /// Rendered path with parameters applied
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub task_type: Option<TaskType>,
    #[pyo3(get)]
    pub system_type: Option<SystemType>,
    #[pyo3(get)]
    pub stage: Option<EtlStage>,
    #[pyo3(get)]
    pub entity: Option<String>,
    #[pyo3(get)]
    pub is_common: bool,
    /// Template command (never mutated)
    #[pyo3(get)]
    pub command: Option<String>,

    /// Rendered command with parameters applied
    #[pyo3(get)]
    pub rendered_command: Option<String>,
    #[pyo3(get)]
    pub params: Option<HashSet<String>>,
    pub stored_params: Option<HashMap<String, String>>,
}

/// Lightweight structure containing only rendered values after parameter application.
/// Use this when you only need the final rendered outputs without carrying metadata.
#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderedSubtask {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub command: Option<String>,
    #[pyo3(get)]
    pub params: HashMap<String, String>,
}

impl Subtask {
    pub fn new(path: &str) -> Self {
        let p = std::path::Path::new(path);
        let name = p
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        Subtask {
            original_name: name.clone(),
            original_path: path.to_string(),
            name,
            path: path.to_string(),
            task_type: None,
            system_type: None,
            stage: None,
            entity: None,
            is_common: false,
            command: None,
            params: None,
            stored_params: None,
            rendered_command: None,
        }
    }
    pub(crate) fn default_param_styles() -> Vec<ParamStyle> {
        ParamStyle::iter()
            .filter(|style| *style != ParamStyle::Other)
            .collect()
    }

    pub(crate) fn regex_for_style(style: ParamStyle) -> &'static Regex {
        match style {
            ParamStyle::DoubleCurly => {
                static RE: OnceCell<Regex> = OnceCell::new();
                RE.get_or_init(|| {
                    Regex::new(r"\{\{(?P<name>[A-Za-z0-9_.:-]+)\}\}").expect("valid regex")
                })
            }
            ParamStyle::Curly => {
                static RE: OnceCell<Regex> = OnceCell::new();
                RE.get_or_init(|| {
                    // Intentionally skip `${...}` and `{{...}}` while matching `{...}`:
                    // the first two alternations match those prefixes without a `name` capture.
                    Regex::new(r"(?:\$\{|\{\{|\{(?P<name>[A-Za-z0-9_.:-]+)\})")
                        .expect("valid regex")
                })
            }
            ParamStyle::Dollar => {
                static RE: OnceCell<Regex> = OnceCell::new();
                RE.get_or_init(|| Regex::new(r"\$(?P<name>[A-Za-z0-9_]+)").unwrap())
            }
            ParamStyle::DollarBrace => {
                static RE: OnceCell<Regex> = OnceCell::new();
                RE.get_or_init(|| {
                    Regex::new(r"\$\{(?P<name>[A-Za-z0-9_.:-]+)\}").expect("valid regex")
                })
            }
            ParamStyle::DoubleUnderscore => {
                static RE: OnceCell<Regex> = OnceCell::new();
                RE.get_or_init(|| Regex::new(r"__(?P<name>[A-Za-z0-9_]+)__").unwrap())
            }
            ParamStyle::Percent => {
                static RE: OnceCell<Regex> = OnceCell::new();
                RE.get_or_init(|| Regex::new(r"%(?P<name>[A-Za-z0-9_]+)%").unwrap())
            }
            ParamStyle::Angle => {
                static RE: OnceCell<Regex> = OnceCell::new();
                RE.get_or_init(|| Regex::new(r"<(?P<name>[A-Za-z0-9_]+)>").unwrap())
            }
            ParamStyle::Other => {
                static RE: OnceCell<Regex> = OnceCell::new();
                RE.get_or_init(|| Regex::new(r"$^").unwrap()) // matches nothing
            }
        }
    }

    pub fn set_task_type_from_ext(&self) -> Self {
        let ext = std::path::Path::new(&self.path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let tt = TaskType::from_extension(ext).unwrap_or(TaskType::Other);
        let mut new_subtask = self.clone();
        if tt != TaskType::Other {
            new_subtask.task_type = Some(tt);
        }
        new_subtask
    }

    /// Extract parameters from path and command, return new Subtask with params set
    pub fn extract_params(&self, styles: Option<&[ParamStyle]>) -> Self {
        let all_params = self.get_params(styles);

        let mut new_subtask = self.clone();
        if !all_params.is_empty() {
            new_subtask.params = Some(all_params);
        }
        new_subtask
    }

    /// Getter method to extract parameters (computed property)
    pub fn get_params(&self, styles: Option<&[ParamStyle]>) -> HashSet<String> {
        let mut all_params = HashSet::new();

        // Extract from path
        let path_params = Self::detect_parameters_in_text(&self.path, styles);
        all_params.extend(path_params);

        // Extract from command if present
        if let Some(cmd) = &self.command {
            let cmd_params = Self::detect_parameters_in_text(cmd, styles);
            all_params.extend(cmd_params);
        }

        // Extract from name (optional - depending on your use case)
        let name_params = Self::detect_parameters_in_text(&self.name, styles);
        all_params.extend(name_params);

        all_params
    }

    /// Find parameter names according to given param styles.
    /// If `styles` is None, uses ParamStyle::default_order()
    pub fn detect_parameters_in_text(text: &str, styles: Option<&[ParamStyle]>) -> HashSet<String> {
        let mut result = HashSet::new();
        let default_styles = Subtask::default_param_styles();
        let use_styles = styles.unwrap_or(&default_styles);
        for &style in use_styles.iter() {
            let re = Subtask::regex_for_style(style);
            for caps in re.captures_iter(text) {
                if let Some(m) = caps.name("name") {
                    result.insert(m.as_str().to_string());
                }
            }
        }
        result
    }

    /// Apply parameters to a given text. Returns (new_text, missing_keys)
    pub fn apply_parameters_to_text(
        text: &str,
        params: &HashMap<String, String>,
        styles: Option<&[ParamStyle]>,
        ignore_missing: bool,
    ) -> (String, Vec<String>) {
        let default_styles = Subtask::default_param_styles();
        let use_styles = styles.unwrap_or(&default_styles);
        // We'll apply replacements one style at a time
        let mut current = text.to_string();
        let mut missing = Vec::new();

        for &style in use_styles.iter() {
            let re = Subtask::regex_for_style(style);
            // replace all matches for this style
            let replaced = re.replace_all(&current, |caps: &regex::Captures| {
                // name capture present?
                if let Some(name_m) = caps.name("name") {
                    let key = name_m.as_str();
                    // Try exact match, then lowercase match
                    if let Some(v) = params.get(key) {
                        return v.to_string();
                    }
                    if let Some(v) = params.get(&key.to_lowercase()) {
                        return v.to_string();
                    }
                    // Keep track of missing keys; caller decides whether to error.
                    missing.push(key.to_string());
                    if !ignore_missing {
                        // continue replacing as identity and report the missing key later
                    }
                    return caps
                        .get(0)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default();
                }
                caps.get(0)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default()
            });
            current = replaced.into_owned();
        }

        (current, missing)
    }

    /// Get the command to execute. Returns rendered_command if available, otherwise command template.
    /// This is a convenience method to avoid checking both fields.
    pub fn get_command(&self) -> Option<&String> {
        self.rendered_command.as_ref().or(self.command.as_ref())
    }

    /// Render this subtask - resolves all templates even if no parameters are needed.
    /// Equivalent to calling apply_parameters with empty params.
    pub fn render(&self) -> Self {
        let empty_params = HashMap::new();
        // Use ignore_missing=true since we're just rendering with no params
        self.apply_parameters(&empty_params, None, true)
            .expect("render should never fail with empty params and ignore_missing=true")
    }

    /// Apply parameters and return a lightweight RenderedSubtask with only the output values.
    /// This is more efficient than apply_parameters() which clones the entire Subtask.
    pub fn render_with_params(
        &self,
        params: &HashMap<String, String>,
        styles: Option<&[ParamStyle]>,
        ignore_missing: bool,
    ) -> Result<RenderedSubtask, String> {
        self.render_with_params_guarded(params, styles, ignore_missing, None)
    }

    /// Same as [`Subtask::render_with_params`], with explicit control over the
    /// SQL guard. `guard: None` means "on for SQL tasks".
    pub fn render_with_params_guarded(
        &self,
        params: &HashMap<String, String>,
        styles: Option<&[ParamStyle]>,
        ignore_missing: bool,
        guard: Option<bool>,
    ) -> Result<RenderedSubtask, String> {
        if self.guard_enabled(guard) {
            self.check_params(params, styles)?;
        }

        let mut all_missing = Vec::new();

        // Apply from ORIGINAL path template
        let (new_path, missing_path) =
            Self::apply_parameters_to_text(&self.original_path, params, styles, ignore_missing);
        all_missing.extend(missing_path);

        // Apply from ORIGINAL name template
        let (new_name, missing_name) =
            Self::apply_parameters_to_text(&self.original_name, params, styles, ignore_missing);
        all_missing.extend(missing_name);

        // command: APPLY FROM TEMPLATE
        let rendered_command = if let Some(template_cmd) = &self.command {
            let (rendered, missing_cmd) =
                Self::apply_parameters_to_text(template_cmd, params, styles, ignore_missing);
            all_missing.extend(missing_cmd);
            Some(rendered)
        } else {
            None
        };

        if !all_missing.is_empty() && !ignore_missing {
            all_missing.sort();
            all_missing.dedup();
            return Err(format!(
                "Missing parameters for keys: {}",
                all_missing.join(", ")
            ));
        }

        Ok(RenderedSubtask {
            name: new_name,
            path: new_path,
            command: rendered_command,
            params: params.clone(),
        })
    }

    /// Lightweight render without parameters. Returns only the rendered values.
    pub fn render_lightweight(&self) -> RenderedSubtask {
        let empty_params = HashMap::new();
        self.render_with_params(&empty_params, None, true).expect(
            "render_lightweight should never fail with empty params and ignore_missing=true",
        )
    }

    /// Apply parameters to this subtask (path, command, and name). Returns new Subtask with applied parameters.
    /// Returns Err if missing parameters and ignore_missing==false
    /// Always applies from original_path, original_name, and command templates, so can be called multiple times
    pub fn apply_parameters(
        &self,
        params: &HashMap<String, String>,
        styles: Option<&[ParamStyle]>,
        ignore_missing: bool,
    ) -> Result<Self, String> {
        self.apply_parameters_guarded(params, styles, ignore_missing, None)
    }

    /// Same as [`Subtask::apply_parameters`], with explicit control over the
    /// SQL guard. `guard: None` means "on for SQL tasks".
    pub fn apply_parameters_guarded(
        &self,
        params: &HashMap<String, String>,
        styles: Option<&[ParamStyle]>,
        ignore_missing: bool,
        guard: Option<bool>,
    ) -> Result<Self, String> {
        if self.guard_enabled(guard) {
            self.check_params(params, styles)?;
        }

        let mut all_missing = Vec::new();

        // Apply from ORIGINAL path template
        let (new_path, missing_path) =
            Self::apply_parameters_to_text(&self.original_path, params, styles, ignore_missing);
        all_missing.extend(missing_path);

        // Apply from ORIGINAL name template
        let (new_name, missing_name) =
            Self::apply_parameters_to_text(&self.original_name, params, styles, ignore_missing);
        all_missing.extend(missing_name);

        // command: APPLY FROM TEMPLATE → STORE IN rendered_command
        let rendered_command = if let Some(template_cmd) = &self.command {
            let (rendered, missing_cmd) =
                Self::apply_parameters_to_text(template_cmd, params, styles, ignore_missing);
            all_missing.extend(missing_cmd);
            Some(rendered)
        } else {
            None
        };

        if !all_missing.is_empty() && !ignore_missing {
            all_missing.sort();
            all_missing.dedup();
            return Err(format!(
                "Missing parameters for keys: {}",
                all_missing.join(", ")
            ));
        }

        Ok(Subtask {
            original_name: self.original_name.clone(),
            original_path: self.original_path.clone(),
            name: new_name,
            path: new_path,
            task_type: self.task_type,
            system_type: self.system_type,
            stage: self.stage,
            entity: self.entity.clone(),
            is_common: self.is_common,
            command: self.command.clone(),
            rendered_command,
            params: self.params.clone(),
            stored_params: Some(params.clone()),
        })
    }

    /// Whether the SQL-injection guard runs for this subtask.
    ///
    /// `Some(v)` forces it either way; `None` enables it exactly for SQL tasks,
    /// where interpolated values end up inside a statement.
    pub fn guard_enabled(&self, guard: Option<bool>) -> bool {
        guard.unwrap_or(self.task_type == Some(TaskType::Sql))
    }

    /// Run the SQL guard over every provided value that the command template
    /// actually references. Values used only in `path`/`name` are not checked:
    /// they never reach the SQL text.
    pub fn check_params(
        &self,
        params: &HashMap<String, String>,
        styles: Option<&[ParamStyle]>,
    ) -> Result<(), String> {
        let Some(command) = &self.command else {
            return Ok(());
        };

        let mut used: Vec<String> = Self::detect_parameters_in_text(command, styles)
            .into_iter()
            .collect();
        used.sort();

        for name in used {
            let value = params
                .get(&name)
                .or_else(|| params.get(&name.to_lowercase()));
            if let Some(value) = value {
                sql_guard::check_value(&name, value)?;
            }
        }
        Ok(())
    }

    /// Rewrite the command template into a driver-ready query plus the ordered
    /// parameter names to bind, instead of interpolating values into the SQL.
    ///
    /// `identifiers` lists parameter names that must be inlined rather than
    /// bound (table/schema/column names — no driver can bind those); their
    /// values are validated as SQL identifiers first.
    pub fn bind_command(
        &self,
        params: &HashMap<String, String>,
        styles: Option<&[ParamStyle]>,
        sql_style: SqlParamStyle,
        identifiers: &[String],
    ) -> Result<BoundQuery, String> {
        let Some(command) = &self.command else {
            return Err(format!(
                "subtask '{}' has no command to prepare; load it first",
                self.name
            ));
        };

        let mut inline: HashMap<String, String> = HashMap::new();
        for name in identifiers {
            let value = params
                .get(name)
                .or_else(|| params.get(&name.to_lowercase()))
                .ok_or_else(|| {
                    format!("identifier '{}' was requested but no value was given", name)
                })?;
            inline.insert(name.clone(), sql_guard::check_identifier(value)?);
        }

        bind_text(command, styles, sql_style, &inline)
    }

    /// The `sqlparser` dialect this subtask's SQL should be read with, derived
    /// from its `system_type`.
    pub fn dialect(&self) -> &'static str {
        sql_analysis::dialect_name(self.system_type)
    }

    /// Candidate command texts to hand the parser, in preference order, each
    /// paired with a map from the sentinel it used back to the placeholder it
    /// stood for.
    ///
    /// A raw template does not parse — `FROM {tbl}` is not SQL — so each
    /// placeholder is replaced by a sentinel. Two passes are tried because no
    /// single sentinel fits every position: a bare identifier works for
    /// `FROM {tbl}`, a number for `LIMIT {n}`. Both stay valid inside quotes,
    /// so `'{name}'` remains a well-formed literal.
    pub fn sentinel_renders(
        &self,
        styles: Option<&[ParamStyle]>,
    ) -> Vec<(String, HashMap<String, String>)> {
        let Some(command) = &self.command else {
            return Vec::new();
        };
        let names = Self::detect_parameters_in_text(command, styles);

        // Pass 1: a per-parameter identifier, so the analysis can name the
        // placeholder a table came from instead of an opaque token.
        let mut named: HashMap<String, String> = HashMap::new();
        let mut back: HashMap<String, String> = HashMap::new();
        for name in &names {
            let sanitized: String = name
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect();
            let sentinel = format!("_p_{}", sanitized);
            back.insert(sentinel.clone(), format!("{{{}}}", name));
            named.insert(name.clone(), sentinel);
        }

        // Pass 2: a plain number, for placeholders that sit where only a
        // literal parses. The sentinel is not reversible, hence no map.
        let numeric: HashMap<String, String> = names
            .iter()
            .map(|name| (name.clone(), "1".to_string()))
            .collect();

        vec![
            (
                Self::apply_parameters_to_text(command, &named, styles, true).0,
                back,
            ),
            (
                Self::apply_parameters_to_text(command, &numeric, styles, true).0,
                HashMap::new(),
            ),
        ]
    }

    /// Parse the command and report what it does.
    ///
    /// With `params`, the rendered command is analysed — which also surfaces
    /// anything an interpolated value added to the statement. Without them, a
    /// sentinel-substituted copy of the template is used instead.
    ///
    /// Always fails open: unparseable SQL yields `parsed = false` and an empty
    /// verdict, never an error.
    pub fn analyze(
        &self,
        params: Option<&HashMap<String, String>>,
        styles: Option<&[ParamStyle]>,
    ) -> SqlAnalysis {
        let dialect = self.dialect();

        // No command is "nothing to analyse", not "an empty statement that
        // parsed fine" — report it the same way as unparseable SQL.
        let Some(command) = &self.command else {
            return SqlAnalysis::unparsed(
                dialect,
                format!("subtask '{}' has no command to analyze", self.name),
            );
        };

        let candidates: Vec<(String, HashMap<String, String>)> = match params {
            Some(params) => vec![(
                Self::apply_parameters_to_text(command, params, styles, true).0,
                HashMap::new(),
            )],
            None => self.sentinel_renders(styles),
        };

        let mut first: Option<SqlAnalysis> = None;
        for (candidate, back) in candidates {
            let mut analysis = sql_analysis::analyze(&candidate, dialect);
            if analysis.parsed {
                Self::restore_placeholders(&mut analysis, &back);
                return analysis;
            }
            first.get_or_insert(analysis);
        }
        first.expect("candidates was checked to be non-empty")
    }

    /// Put the original placeholders back into anything the analysis reports,
    /// so a table read from `FROM {tbl}` is named `{tbl}` and not `_p_tbl`.
    fn restore_placeholders(analysis: &mut SqlAnalysis, back: &HashMap<String, String>) {
        if back.is_empty() {
            return;
        }
        // Longest sentinel first: `_p_a` must not clobber part of `_p_a_b`.
        let mut pairs: Vec<(&String, &String)> = back.iter().collect();
        pairs.sort_by_key(|(sentinel, _)| std::cmp::Reverse(sentinel.len()));

        let restore = |text: &mut String| {
            for (sentinel, original) in &pairs {
                if text.contains(sentinel.as_str()) {
                    *text = text.replace(sentinel.as_str(), original);
                }
            }
        };

        for table in analysis.tables.iter_mut() {
            restore(table);
        }
        for warning in analysis.warnings.iter_mut() {
            restore(warning);
        }
    }

    /// Reject the SQL when it runs a statement kind the caller forbade.
    ///
    /// Fails open by design: SQL that does not parse cannot be judged, so it is
    /// allowed through rather than blocked. Callers that need certainty should
    /// check `analyze().parsed` themselves.
    pub fn check_forbidden(
        &self,
        params: Option<&HashMap<String, String>>,
        styles: Option<&[ParamStyle]>,
        forbid: &[StatementKind],
    ) -> Result<(), String> {
        if forbid.is_empty() {
            return Ok(());
        }

        let analysis = self.analyze(params, styles);
        let hits = analysis.forbidden_hits(forbid);
        if hits.is_empty() {
            return Ok(());
        }

        let names: Vec<&str> = hits.iter().map(|k| k.name()).collect();
        let tables = if analysis.tables.is_empty() {
            String::new()
        } else {
            format!(" on {}", analysis.tables.join(", "))
        };
        Err(format!(
            "subtask '{}' runs forbidden statement(s): {}{}",
            self.name,
            names.join(", "),
            tables
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_subtask_extract_params() {
        let mut subtask = Subtask::new("templates/{env}/{date}_report.sql");
        subtask.command = Some("psql -h $host -U ${user}".into());

        // Extract and store params (returns new instance)
        let subtask = subtask.extract_params(None);

        // Check that params were stored
        assert!(subtask.params.is_some());
        let params = subtask.params.as_ref().unwrap();
        assert!(params.contains("env"));
        assert!(params.contains("date"));
        assert!(params.contains("host"));
        assert!(params.contains("user"));
        assert_eq!(params.len(), 4);

        // Also test getter method
        let computed_params = subtask.get_params(None);
        assert_eq!(computed_params.len(), 4);
    }

    #[test]
    fn test_subtask_get_params_only() {
        let subtask = Subtask {
            original_name: "report_{env}.sql".to_string(),
            original_path: "path/{date}/report_{env}.sql".to_string(),
            name: "report_{env}.sql".to_string(),
            path: "path/{date}/report_{env}.sql".to_string(),
            task_type: None,
            system_type: None,
            stage: None,
            entity: None,
            is_common: false,
            command: Some("run $user".to_string()),
            params: None, // Not pre-extracted
            stored_params: None,
            rendered_command: None,
        };

        // Use getter to compute params
        let params = subtask.get_params(None);
        assert!(params.contains("env"));
        assert!(params.contains("date"));
        assert!(params.contains("user"));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_detect_curly() {
        let text = "path/{env}/file_{date}.sql";
        let params = Subtask::detect_parameters_in_text(text, Some(&[ParamStyle::Curly]));
        assert!(params.contains("env"));
        assert!(params.contains("date"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_detect_dollar() {
        let text = "run $user on $host now";
        let params = Subtask::detect_parameters_in_text(text, Some(&[ParamStyle::Dollar]));
        assert!(params.contains("user"));
        assert!(params.contains("host"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_detect_dollar_brace() {
        let text = "connect to ${db} as ${user}";
        let params = Subtask::detect_parameters_in_text(text, Some(&[ParamStyle::DollarBrace]));
        assert!(params.contains("db"));
        assert!(params.contains("user"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_detect_double_underscore() {
        let text = "Hello __NAME__, your code is __STATUS__";
        let params =
            Subtask::detect_parameters_in_text(text, Some(&[ParamStyle::DoubleUnderscore]));
        assert!(params.contains("NAME"));
        assert!(params.contains("STATUS"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_detect_percent() {
        let text = "%env% and %region%";
        let params = Subtask::detect_parameters_in_text(text, Some(&[ParamStyle::Percent]));
        assert!(params.contains("env"));
        assert!(params.contains("region"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_detect_angle() {
        let text = "deploy to <environment> zone <zone>";
        let params = Subtask::detect_parameters_in_text(text, Some(&[ParamStyle::Angle]));
        assert!(params.contains("environment"));
        assert!(params.contains("zone"));
        assert_eq!(params.len(), 2);
    }

    //
    // Replacement tests
    //

    #[test]
    fn test_apply_curly() {
        let text = "file_{env}_{date}.sql";
        let params = map(&[("env", "prod"), ("date", "2025")]);
        let (out, missing) =
            Subtask::apply_parameters_to_text(text, &params, Some(&[ParamStyle::Curly]), false);
        assert_eq!(missing.len(), 0);
        assert_eq!(out, "file_prod_2025.sql");
    }

    #[test]
    fn test_apply_dollar() {
        let text = "backup $host-$user";
        let params = map(&[("host", "srv"), ("user", "alice")]);
        let (out, missing) =
            Subtask::apply_parameters_to_text(text, &params, Some(&[ParamStyle::Dollar]), false);
        assert_eq!(out, "backup srv-alice");
        assert!(missing.is_empty());
    }

    #[test]
    fn test_apply_dollar_brace() {
        let text = "db=${db}, user=${user}";
        let params = map(&[("db", "prod"), ("user", "bob")]);
        let (out, missing) = Subtask::apply_parameters_to_text(
            text,
            &params,
            Some(&[ParamStyle::DollarBrace]),
            false,
        );
        assert_eq!(out, "db=prod, user=bob");
        assert!(missing.is_empty());
    }

    #[test]
    fn test_apply_double_underscore() {
        let text = "Hello __NAME__, status=__STATUS__";
        let params = map(&[("NAME", "John"), ("STATUS", "OK")]);
        let (out, missing) = Subtask::apply_parameters_to_text(
            text,
            &params,
            Some(&[ParamStyle::DoubleUnderscore]),
            false,
        );
        println!("{}", out);
        assert_eq!(out, "Hello John, status=OK");
        assert!(missing.is_empty());
    }

    #[test]
    fn test_apply_percent() {
        let text = "%env%/%region%";
        let params = map(&[("env", "prod"), ("region", "eu")]);
        let (out, missing) =
            Subtask::apply_parameters_to_text(text, &params, Some(&[ParamStyle::Percent]), false);
        assert_eq!(out, "prod/eu");
        assert!(missing.is_empty());
    }

    #[test]
    fn test_apply_angle() {
        let text = "<stage>-<version>";
        let params = map(&[("stage", "beta"), ("version", "3")]);
        let (out, missing) =
            Subtask::apply_parameters_to_text(text, &params, Some(&[ParamStyle::Angle]), false);
        assert_eq!(out, "beta-3");
        assert!(missing.is_empty());
    }

    //
    // Missing parameter behavior
    //

    #[test]
    fn test_missing_param_error() {
        let text = "Hello {name}";
        let params = map(&[]);
        let (out, missing) =
            Subtask::apply_parameters_to_text(text, &params, Some(&[ParamStyle::Curly]), false);

        assert_eq!(missing, vec!["name"]);
        assert_eq!(out, "Hello {name}");
    }

    #[test]
    fn test_missing_param_ignore() {
        let text = "Hello {name}";
        let params = map(&[]);
        let (out, missing) =
            Subtask::apply_parameters_to_text(text, &params, Some(&[ParamStyle::Curly]), true);

        assert_eq!(missing, vec!["name"]);
        assert_eq!(out, "Hello {name}"); // unchanged
    }

    //
    // Integration: Subtask::apply_parameters
    //

    #[test]
    fn test_subtask_apply_parameters_full() {
        let mut s = Subtask::new("templates/report_{{env}}.sql");
        s.command = Some("psql -h $host -U $user -d ${db}".into());

        let params = map(&[
            ("env", "prod"),
            ("host", "db.example.com"),
            ("user", "alice"),
            ("db", "analytics"),
        ]);

        let s = s.apply_parameters(&params, None, false).unwrap();

        assert_eq!(s.path, "templates/report_prod.sql");
        assert_eq!(
            s.rendered_command.as_ref().unwrap(),
            "psql -h db.example.com -U alice -d analytics"
        );
        assert_eq!(
            s.command.as_ref().unwrap(),
            "psql -h $host -U $user -d ${db}"
        );
        assert_eq!(s.name, "report_prod.sql"); // if name contained placeholders
    }

    #[test]
    fn test_subtask_apply_parameters_missing() {
        let s = Subtask::new("run_{missing}.sql");

        let params = map(&[]);
        let res = s.apply_parameters(&params, None, false);

        assert!(res.is_err());
        assert!(res.unwrap_err().contains("missing"));
    }

    #[test]
    fn test_apply_parameters_multiple_times() {
        let mut s = Subtask::new("templates/report_{{env}}.sql");
        s.command = Some("psql -h $host -U $user".into());

        // First application with prod params
        let params1 = map(&[
            ("env", "prod"),
            ("host", "prod.example.com"),
            ("user", "prod_user"),
        ]);
        let s1 = s.apply_parameters(&params1, None, false).unwrap();

        assert_eq!(s1.path, "templates/report_prod.sql");
        assert_eq!(s1.name, "report_prod.sql");
        assert_eq!(
            s1.rendered_command.as_ref().unwrap(),
            "psql -h prod.example.com -U prod_user"
        );

        // Second application with dev params - should use original templates from s (or s1, both have same originals)
        let params2 = map(&[
            ("env", "dev"),
            ("host", "dev.example.com"),
            ("user", "dev_user"),
        ]);
        let s2 = s.apply_parameters(&params2, None, false).unwrap();

        assert_eq!(s2.path, "templates/report_dev.sql");
        assert_eq!(s2.name, "report_dev.sql");
        assert_eq!(
            s2.rendered_command.as_ref().unwrap(),
            "psql -h dev.example.com -U dev_user"
        );

        // Verify originals are unchanged in both instances
        assert_eq!(s1.original_path, "templates/report_{{env}}.sql");
        assert_eq!(s1.original_name, "report_{{env}}.sql");
        assert_eq!(s1.command.as_ref().unwrap(), "psql -h $host -U $user");

        assert_eq!(s2.original_path, "templates/report_{{env}}.sql");
        assert_eq!(s2.original_name, "report_{{env}}.sql");
        assert_eq!(s2.command.as_ref().unwrap(), "psql -h $host -U $user");
    }

    #[test]
    fn test_originals_preserved() {
        let mut s = Subtask::new("path/{date}/file_{env}.sql");
        s.command = Some("run on $host".into());

        let original_path = s.original_path.clone();
        let original_name = s.original_name.clone();
        let original_cmd = s.command.clone();

        // Apply parameters (returns new instance)
        let params = map(&[("date", "2025"), ("env", "test"), ("host", "localhost")]);
        let applied = s.apply_parameters(&params, None, false).unwrap();

        // Verify originals in the NEW instance are still unchanged
        assert_eq!(applied.original_path, original_path);
        assert_eq!(applied.original_name, original_name);
        assert_eq!(applied.command, original_cmd);

        // Verify rendered values changed in the NEW instance
        assert_eq!(applied.path, "path/2025/file_test.sql");
        assert_eq!(applied.name, "file_test.sql");
        assert_eq!(
            applied.rendered_command.as_ref().unwrap(),
            "run on localhost"
        );

        // Verify original instance is completely unchanged
        assert_eq!(s.path, "path/{date}/file_{env}.sql");
        assert_eq!(s.name, "file_{env}.sql");
        assert_eq!(s.rendered_command, None);
    }

    #[test]
    fn test_get_command() {
        // With no rendered_command, should return command template
        let s = Subtask::new("test.sql");
        assert_eq!(s.get_command(), None);

        let mut s = s;
        s.command = Some("psql -h localhost".into());
        assert_eq!(s.get_command(), Some(&"psql -h localhost".to_string()));

        // After applying params, should return rendered_command
        let params = map(&[("host", "prod.db")]);
        s.command = Some("psql -h $host".into());
        let applied = s.apply_parameters(&params, None, false).unwrap();
        assert_eq!(applied.get_command(), Some(&"psql -h prod.db".to_string()));
        assert_eq!(
            applied.rendered_command,
            Some("psql -h prod.db".to_string())
        );
    }

    #[test]
    fn test_render_no_params() {
        let mut s = Subtask::new("report.sql");
        s.command = Some("psql -h localhost -U admin".into());

        // Render without any parameters
        let rendered = s.render();

        // Should have rendered_command populated even though no params were replaced
        assert_eq!(
            rendered.get_command(),
            Some(&"psql -h localhost -U admin".to_string())
        );
        assert_eq!(
            rendered.rendered_command,
            Some("psql -h localhost -U admin".to_string())
        );
        assert_eq!(rendered.path, "report.sql");
        assert_eq!(rendered.name, "report.sql");
    }

    #[test]
    fn test_render_with_template_but_no_params_provided() {
        let mut s = Subtask::new("report_{env}.sql");
        s.command = Some("psql -h $host".into());

        // Render without providing parameters - should leave placeholders unchanged
        let rendered = s.render();

        // Placeholders remain since we used empty params with ignore_missing=true
        assert_eq!(rendered.path, "report_{env}.sql");
        assert_eq!(rendered.name, "report_{env}.sql");
        assert_eq!(rendered.rendered_command, Some("psql -h $host".to_string()));
    }

    #[test]
    fn test_render_with_params_lightweight() {
        let mut s = Subtask::new("templates/{env}/report_{date}.sql");
        s.command = Some("psql -h $host -U $user -d $db".into());

        let params = map(&[
            ("env", "prod"),
            ("date", "2025-01"),
            ("host", "prod.db.com"),
            ("user", "admin"),
            ("db", "analytics"),
        ]);

        // Use lightweight render
        let rendered = s.render_with_params(&params, None, false).unwrap();

        // Check rendered values
        assert_eq!(rendered.name, "report_2025-01.sql");
        assert_eq!(rendered.path, "templates/prod/report_2025-01.sql");
        assert_eq!(
            rendered.command.as_ref().unwrap(),
            "psql -h prod.db.com -U admin -d analytics"
        );
        assert_eq!(rendered.params.get("env"), Some(&"prod".to_string()));
        assert_eq!(rendered.params.get("date"), Some(&"2025-01".to_string()));
    }

    #[test]
    fn test_render_lightweight_no_params() {
        let mut s = Subtask::new("report.sql");
        s.command = Some("psql -h localhost".into());

        let rendered = s.render_lightweight();

        assert_eq!(rendered.name, "report.sql");
        assert_eq!(rendered.path, "report.sql");
        assert_eq!(rendered.command, Some("psql -h localhost".to_string()));
        assert!(rendered.params.is_empty());
    }

    #[test]
    fn test_analyze_template_with_sentinels() {
        let mut s = Subtask::new("purge.sql");
        s.system_type = Some(SystemType::PostgreSQL);
        s.command = Some("DELETE FROM analytics.{tbl}".into());

        let analysis = s.analyze(None, None);

        assert!(analysis.parsed);
        assert_eq!(analysis.dialect, "postgres");
        assert_eq!(analysis.statements, vec![StatementKind::Delete]);
        // The sentinel is mapped back to the placeholder it stood for.
        assert_eq!(analysis.tables, vec!["analytics.{tbl}"]);
        assert!(analysis
            .warnings
            .iter()
            .any(|w| w.contains("analytics.{tbl}")));
    }

    #[test]
    fn test_analyze_falls_back_to_a_numeric_sentinel() {
        let mut s = Subtask::new("page.sql");
        s.system_type = Some(SystemType::PostgreSQL);
        // An identifier does not parse after LIMIT; the numeric pass does.
        s.command = Some("SELECT * FROM t LIMIT {n}".into());

        let analysis = s.analyze(None, None);

        assert!(analysis.parsed);
        assert_eq!(analysis.tables, vec!["t"]);
    }

    #[test]
    fn test_analyze_rendered_params_sees_injected_statements() {
        let mut s = Subtask::new("q.sql");
        s.system_type = Some(SystemType::PostgreSQL);
        s.command = Some("SELECT * FROM users WHERE n = '{n}'".into());

        let clean = s.analyze(Some(&map(&[("n", "alice")])), None);
        assert_eq!(clean.statements, vec![StatementKind::Select]);

        let injected = s.analyze(Some(&map(&[("n", "x'; DROP TABLE users; --")])), None);
        assert_eq!(
            injected.statements,
            vec![StatementKind::Select, StatementKind::Drop]
        );
    }

    #[test]
    fn test_analyze_without_command_is_not_parsed() {
        let s = Subtask::new("q.sql");
        let analysis = s.analyze(None, None);

        assert!(!analysis.parsed);
        assert!(analysis.error.unwrap().contains("no command to analyze"));
    }

    #[test]
    fn test_check_forbidden_fails_open_on_unparseable_sql() {
        let mut s = Subtask::new("secret.sql");
        s.system_type = Some(SystemType::Duckdb);
        s.command = Some("CREATE PERSISTENT SECRET s (TYPE POSTGRES, PORT 5432)".into());

        assert!(!s.analyze(None, None).parsed);
        // Unjudgeable SQL is allowed through rather than blocked.
        assert!(s
            .check_forbidden(None, None, &[StatementKind::Drop])
            .is_ok());
    }

    #[test]
    fn test_check_forbidden_blocks_and_names_the_statement() {
        let mut s = Subtask::new("purge.sql");
        s.system_type = Some(SystemType::PostgreSQL);
        s.command = Some("TRUNCATE TABLE events".into());

        assert!(s.check_forbidden(None, None, &[]).is_ok());
        assert!(s
            .check_forbidden(None, None, &[StatementKind::Drop])
            .is_ok());

        let err = s
            .check_forbidden(None, None, &[StatementKind::Truncate])
            .unwrap_err();
        assert!(err.contains("truncate"), "{}", err);
        assert!(err.contains("events"), "{}", err);
    }
}

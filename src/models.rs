use crate::enums::{EtlStage, ParamType, SystemType, TaskType};
use once_cell::sync::OnceCell;
use pyo3::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
    fn default_param_styles() -> Vec<ParamType> {
        ParamType::ALL.to_vec()
    }

    fn regex_for_style(style: ParamType) -> &'static Regex {
        match style {
            ParamType::DoubleCurly => {
                static RE: OnceCell<Regex> = OnceCell::new();
                RE.get_or_init(|| {
                    Regex::new(r"\{\{(?P<name>[A-Za-z0-9_.:-]+)\}\}").expect("valid regex")
                })
            }
            ParamType::Curly => {
                static RE: OnceCell<Regex> = OnceCell::new();
                RE.get_or_init(|| {
                    Regex::new(r"(?:\$\{|\{\{|\{(?P<name>[A-Za-z0-9_.:-]+)\})").expect("valid regex")
                })
            }
            ParamType::Dollar => {
                static RE: OnceCell<Regex> = OnceCell::new();
                RE.get_or_init(|| Regex::new(r"\$(?P<name>[A-Za-z0-9_]+)").unwrap())
            }
            ParamType::DollarBrace => {
                static RE: OnceCell<Regex> = OnceCell::new();
                RE.get_or_init(|| {
                    Regex::new(r"\$\{(?P<name>[A-Za-z0-9_.:-]+)\}").expect("valid regex")
                })
            }
            ParamType::DoubleUnderscore => {
                static RE: OnceCell<Regex> = OnceCell::new();
                RE.get_or_init(|| Regex::new(r"__(?P<name>[A-Za-z0-9_]+)__").unwrap())
            }
            ParamType::Percent => {
                static RE: OnceCell<Regex> = OnceCell::new();
                RE.get_or_init(|| Regex::new(r"%(?P<name>[A-Za-z0-9_]+)%").unwrap())
            }
            ParamType::Angle => {
                static RE: OnceCell<Regex> = OnceCell::new();
                RE.get_or_init(|| Regex::new(r"<(?P<name>[A-Za-z0-9_]+)>").unwrap())
            }
            ParamType::Other => {
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
    pub fn extract_params(&self, styles: Option<&[ParamType]>) -> Self {
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

        let mut new_subtask = self.clone();
        if !all_params.is_empty() {
            new_subtask.params = Some(all_params);
        }
        new_subtask
    }

    /// Getter method to extract parameters (computed property)
    pub fn get_params(&self, styles: Option<&[ParamType]>) -> HashSet<String> {
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
    /// If `styles` is None, uses ParamType::default_order()
    pub fn detect_parameters_in_text(text: &str, styles: Option<&[ParamType]>) -> HashSet<String> {
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
        styles: Option<&[ParamType]>,
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
                    // For $name pattern, some regex includes the leading $ in full match,
                    // so return the full capture replacement in case of missing and ignore_missing==true
                    missing.push(key.to_string());
                    if ignore_missing {
                        // return original match unchanged
                        return caps
                            .get(0)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default();
                    } else {
                        // produce a sentinel; caller will detect missing_keys and can error
                        return format!("__MISSING_PARAM_{}__", key);
                    }
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
        styles: Option<&[ParamType]>,
        ignore_missing: bool,
    ) -> Result<RenderedSubtask, String> {
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
        self.render_with_params(&empty_params, None, true)
            .expect("render_lightweight should never fail with empty params and ignore_missing=true")
    }

    /// Apply parameters to this subtask (path, command, and name). Returns new Subtask with applied parameters.
    /// Returns Err if missing parameters and ignore_missing==false
    /// Always applies from original_path, original_name, and command templates, so can be called multiple times
    pub fn apply_parameters(
        &self,
        params: &HashMap<String, String>,
        styles: Option<&[ParamType]>,
        ignore_missing: bool,
    ) -> Result<Self, String> {
        let mut all_missing = Vec::new();

        // Apply from ORIGINAL path template
        let (new_path, missing_path) =
            Self::apply_parameters_to_text(&self.original_path, params, styles, ignore_missing);
        all_missing.extend(missing_path);
    
        // command: APPLY FROM TEMPLATE → STORE IN rendered_command
        let rendered_command = if let Some(template_cmd) = &self.command {
            let (rendered, missing_cmd) =
                Self::apply_parameters_to_text(template_cmd, params, styles, ignore_missing);
            all_missing.extend(missing_cmd);
            Some(rendered)
        } else {
            None
        };

        // Apply from ORIGINAL name template
        let (new_name, missing_name) =
            Self::apply_parameters_to_text(&self.original_name, params, styles, ignore_missing);
        all_missing.extend(missing_name);

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
        let params = Subtask::detect_parameters_in_text(text, Some(&[ParamType::Curly]));
        assert!(params.contains("env"));
        assert!(params.contains("date"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_detect_dollar() {
        let text = "run $user on $host now";
        let params = Subtask::detect_parameters_in_text(text, Some(&[ParamType::Dollar]));
        assert!(params.contains("user"));
        assert!(params.contains("host"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_detect_dollar_brace() {
        let text = "connect to ${db} as ${user}";
        let params = Subtask::detect_parameters_in_text(text, Some(&[ParamType::DollarBrace]));
        assert!(params.contains("db"));
        assert!(params.contains("user"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_detect_double_underscore() {
        let text = "Hello __NAME__, your code is __STATUS__";
        let params = Subtask::detect_parameters_in_text(text, Some(&[ParamType::DoubleUnderscore]));
        assert!(params.contains("NAME"));
        assert!(params.contains("STATUS"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_detect_percent() {
        let text = "%env% and %region%";
        let params = Subtask::detect_parameters_in_text(text, Some(&[ParamType::Percent]));
        assert!(params.contains("env"));
        assert!(params.contains("region"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_detect_angle() {
        let text = "deploy to <environment> zone <zone>";
        let params = Subtask::detect_parameters_in_text(text, Some(&[ParamType::Angle]));
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
            Subtask::apply_parameters_to_text(text, &params, Some(&[ParamType::Curly]), false);
        assert_eq!(missing.len(), 0);
        assert_eq!(out, "file_prod_2025.sql");
    }

    #[test]
    fn test_apply_dollar() {
        let text = "backup $host-$user";
        let params = map(&[("host", "srv"), ("user", "alice")]);
        let (out, missing) =
            Subtask::apply_parameters_to_text(text, &params, Some(&[ParamType::Dollar]), false);
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
            Some(&[ParamType::DollarBrace]),
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
            Some(&[ParamType::DoubleUnderscore]),
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
            Subtask::apply_parameters_to_text(text, &params, Some(&[ParamType::Percent]), false);
        assert_eq!(out, "prod/eu");
        assert!(missing.is_empty());
    }

    #[test]
    fn test_apply_angle() {
        let text = "<stage>-<version>";
        let params = map(&[("stage", "beta"), ("version", "3")]);
        let (out, missing) =
            Subtask::apply_parameters_to_text(text, &params, Some(&[ParamType::Angle]), false);
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
            Subtask::apply_parameters_to_text(text, &params, Some(&[ParamType::Curly]), false);

        assert_eq!(missing, vec!["name"]);
        assert!(out.contains("__MISSING_PARAM_name__"));
    }

    #[test]
    fn test_missing_param_ignore() {
        let text = "Hello {name}";
        let params = map(&[]);
        let (out, missing) =
            Subtask::apply_parameters_to_text(text, &params, Some(&[ParamType::Curly]), true);

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
        assert_eq!(applied.rendered_command.as_ref().unwrap(), "run on localhost");

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
        assert_eq!(applied.rendered_command, Some("psql -h prod.db".to_string()));
    }

    #[test]
    fn test_render_no_params() {
        let mut s = Subtask::new("report.sql");
        s.command = Some("psql -h localhost -U admin".into());

        // Render without any parameters
        let rendered = s.render();

        // Should have rendered_command populated even though no params were replaced
        assert_eq!(rendered.get_command(), Some(&"psql -h localhost -U admin".to_string()));
        assert_eq!(rendered.rendered_command, Some("psql -h localhost -U admin".to_string()));
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
}

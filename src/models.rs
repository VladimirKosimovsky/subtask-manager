use crate::enums::{EtlStage, ParamType, SystemType, TaskType};
use once_cell::sync::OnceCell;
use pyo3::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[pyclass]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subtask {
    #[pyo3(get)]
    pub name: String,
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
    #[pyo3(get)]
    pub command: Option<String>,
    // Add params field
    #[pyo3(get)]
    pub params: Option<HashSet<String>>,
    /// ➕ NEW: persist key-value parameter mappings
    pub stored_params: Option<HashMap<String, String>>,
}

impl Subtask {
    pub fn new(path: &str) -> Self {
        let p = std::path::Path::new(path);
        Subtask {
            name: p
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default(),
            path: path.to_string(),
            task_type: None,
            system_type: None,
            stage: None,
            entity: None,
            is_common: false,
            command: None,
            params: None,
            stored_params: None,
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

    pub fn set_task_type_from_ext(&mut self) {
        let ext = std::path::Path::new(&self.path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let tt = TaskType::from_extension(ext).unwrap_or(TaskType::Other);
        if tt != TaskType::Other {
            self.task_type = Some(tt);
        }
    }

    /// Extract parameters from path and command, store them in self.params
    pub fn extract_params(&mut self, styles: Option<&[ParamType]>) {
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

        if !all_params.is_empty() {
            self.params = Some(all_params);
        }
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

    /// Apply parameters to this subtask (path, command, and name). Returns Err if missing parameters and ignore_missing==false
    pub fn apply_parameters(
        &mut self,
        params: &HashMap<String, String>,
        styles: Option<&[ParamType]>,
        ignore_missing: bool,
    ) -> Result<(), String> {
        let mut all_missing = Vec::new();

        let (new_path, missing_path) =
            Self::apply_parameters_to_text(&self.path, params, styles, ignore_missing);
        if !missing_path.is_empty() {
            all_missing.extend(missing_path);
        }
        self.path = new_path;

        if let Some(cmd) = &self.command {
            let (new_cmd, missing_cmd) =
                Self::apply_parameters_to_text(cmd, params, styles, ignore_missing);
            if !missing_cmd.is_empty() {
                all_missing.extend(missing_cmd);
            }
            self.command = Some(new_cmd);
        }

        // Update name too (optional) - many times name is derived from path, so you may or may not want this.
        let (new_name, missing_name) =
            Self::apply_parameters_to_text(&self.name, params, styles, ignore_missing);
        if !missing_name.is_empty() {
            all_missing.extend(missing_name);
        }
        self.name = new_name;

        if !all_missing.is_empty() && !ignore_missing {
            all_missing.sort();
            all_missing.dedup();
            return Err(format!(
                "Missing parameters for keys: {}",
                all_missing.join(", ")
            ));
        }
        Ok(())
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

        // Extract and store params
        subtask.extract_params(None);

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

        s.apply_parameters(&params, None, false).unwrap();

        assert_eq!(s.path, "templates/report_prod.sql");
        assert_eq!(
            s.command.as_ref().unwrap(),
            "psql -h db.example.com -U alice -d analytics"
        );
        assert_eq!(s.name, "report_prod.sql"); // if name contained placeholders
    }

    #[test]
    fn test_subtask_apply_parameters_missing() {
        let mut s = Subtask::new("run_{missing}.sql");

        let params = map(&[]);
        let res = s.apply_parameters(&params, None, false);

        assert!(res.is_err());
        assert!(res.unwrap_err().contains("missing"));
    }
}

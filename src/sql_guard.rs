//! Baseline SQL-injection guards for values that get **interpolated** into a
//! SQL task body.
//!
//! Interpolation is inherently unsafe: the value becomes part of the statement
//! text. These guards are a safety net, not a replacement for binding — prefer
//! [`crate::sql_binding`] / `Subtask.prepare()` whenever the driver can bind
//! the value.
//!
//! The guard is deliberately conservative: it rejects anything that could end a
//! literal, start a new statement, or open a comment. Values that legitimately
//! contain quotes, semicolons or backslashes must be bound, not interpolated.

use once_cell::sync::OnceCell;
use regex::Regex;
use std::fmt;

/// One reason a value was rejected, with the byte offset it was seen at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardIssue {
    pub kind: &'static str,
    pub detail: String,
    pub offset: usize,
}

impl fmt::Display for GuardIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}) at offset {}",
            self.kind, self.detail, self.offset
        )
    }
}

fn keyword_re() -> &'static Regex {
    static RE: OnceCell<Regex> = OnceCell::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:union\s+(?:all\s+)?select|drop\s+(?:table|database|schema|index|view|user)|truncate\s+table|delete\s+from|insert\s+into|alter\s+(?:table|user|role|system)|create\s+(?:table|database|schema|user|role)|grant\s+all|revoke\s+all|exec(?:ute)?\s*\(|xp_cmdshell|sp_executesql|pg_sleep\s*\(|pg_read_file\s*\(|benchmark\s*\(|waitfor\s+delay|into\s+(?:out|dump)file|load_file\s*\(|information_schema\.)",
        )
        .expect("valid regex")
    })
}

fn tautology_re() -> &'static Regex {
    static RE: OnceCell<Regex> = OnceCell::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?i)\b(?:or|and)\b\s+(?:'[^']*'|"[^"]*"|[A-Za-z0-9_.]+)\s*(?:=|<>|!=|<|>|\blike\b)\s*(?:'[^']*'|"[^"]*"|[A-Za-z0-9_.]+)"#,
        )
        .expect("valid regex")
    })
}

fn identifier_re() -> &'static Regex {
    static RE: OnceCell<Regex> = OnceCell::new();
    RE.get_or_init(|| {
        Regex::new(r"^[A-Za-z_][A-Za-z0-9_$]{0,62}(?:\.[A-Za-z_][A-Za-z0-9_$]{0,62}){0,2}$")
            .expect("valid regex")
    })
}

/// Collect every guard issue found in `value` (empty vec means the value is
/// considered safe to interpolate).
pub fn find_issues(value: &str) -> Vec<GuardIssue> {
    let mut issues = Vec::new();
    let bytes = value.as_bytes();

    for (i, &c) in bytes.iter().enumerate() {
        match c {
            b'\'' | b'"' | b'`' => issues.push(GuardIssue {
                kind: "quote character",
                detail: format!("{:?}", (c as char).to_string()),
                offset: i,
            }),
            b';' => issues.push(GuardIssue {
                kind: "statement terminator",
                detail: "';'".to_string(),
                offset: i,
            }),
            b'\\' => issues.push(GuardIssue {
                kind: "escape character",
                detail: "'\\\\'".to_string(),
                offset: i,
            }),
            0x00 => issues.push(GuardIssue {
                kind: "control character",
                detail: "NUL".to_string(),
                offset: i,
            }),
            c if c < 0x20 && c != b'\t' && c != b'\n' && c != b'\r' => issues.push(GuardIssue {
                kind: "control character",
                detail: format!("0x{:02x}", c),
                offset: i,
            }),
            0x7f => issues.push(GuardIssue {
                kind: "control character",
                detail: "DEL".to_string(),
                offset: i,
            }),
            _ => {}
        }
    }

    for (marker, label) in [
        ("--", "line comment"),
        ("/*", "block comment"),
        ("*/", "block comment"),
    ] {
        if let Some(offset) = value.find(marker) {
            issues.push(GuardIssue {
                kind: "SQL comment",
                detail: format!("{} {:?}", label, marker),
                offset,
            });
        }
    }

    if let Some(m) = keyword_re().find(value) {
        issues.push(GuardIssue {
            kind: "SQL keyword",
            detail: format!("{:?}", m.as_str()),
            offset: m.start(),
        });
    }

    if let Some(m) = tautology_re().find(value) {
        issues.push(GuardIssue {
            kind: "boolean tautology",
            detail: format!("{:?}", m.as_str()),
            offset: m.start(),
        });
    }

    issues.sort_by_key(|i| i.offset);
    issues
}

/// True when `value` carries no guard issues.
pub fn is_safe(value: &str) -> bool {
    find_issues(value).is_empty()
}

/// Validate a single value bound for interpolation into a SQL body.
pub fn check_value(name: &str, value: &str) -> Result<(), String> {
    let issues = find_issues(value);
    if issues.is_empty() {
        return Ok(());
    }
    let reasons: Vec<String> = issues.iter().map(|i| i.to_string()).collect();
    Err(format!(
        "unsafe value for parameter '{}': {}. Bind the value with Subtask.prepare() \
         instead of interpolating it, or pass guard=False to override",
        name,
        reasons.join("; ")
    ))
}

/// Validate an identifier (table/column/schema name) that has to be inlined
/// because no driver can bind it. Returns the identifier unchanged when valid.
///
/// Accepts up to three dot-separated parts, each starting with a letter or `_`.
pub fn check_identifier(value: &str) -> Result<String, String> {
    if identifier_re().is_match(value) {
        Ok(value.to_string())
    } else {
        Err(format!(
            "invalid SQL identifier: {:?}. Identifiers must match \
             [A-Za-z_][A-Za-z0-9_$]* with at most 3 dot-separated parts",
            value
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_values_are_safe() {
        for v in [
            "prod",
            "2025-01-01",
            "localhost",
            "5432",
            "user_1",
            "a b c",
            "",
        ] {
            assert!(is_safe(v), "expected {:?} to be safe", v);
        }
    }

    #[test]
    fn quote_breakout_is_rejected() {
        let issues = find_issues("o'brien");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, "quote character");
    }

    #[test]
    fn stacked_statement_is_rejected() {
        let err = check_value("name", "x'; DROP TABLE users;--").unwrap_err();
        assert!(err.contains("quote character"));
        assert!(err.contains("statement terminator"));
        assert!(err.contains("SQL comment"));
        assert!(err.contains("SQL keyword"));
    }

    #[test]
    fn tautology_is_rejected() {
        assert!(!is_safe(" OR 1=1"));
        assert!(!is_safe("1 or a=a"));
    }

    #[test]
    fn union_select_is_rejected() {
        assert!(!is_safe("1 UNION ALL SELECT password FROM users"));
    }

    #[test]
    fn comments_are_rejected() {
        assert!(!is_safe("abc--def"));
        assert!(!is_safe("abc/*def*/"));
    }

    #[test]
    fn control_characters_are_rejected() {
        assert!(!is_safe("a\0b"));
        assert!(!is_safe("a\x07b"));
        assert!(is_safe("a\nb"));
    }

    #[test]
    fn backslash_is_rejected() {
        assert!(!is_safe(r"C:\data"));
    }

    #[test]
    fn identifiers() {
        assert_eq!(check_identifier("users").unwrap(), "users");
        assert_eq!(check_identifier("public.users").unwrap(), "public.users");
        assert_eq!(
            check_identifier("db.public.users").unwrap(),
            "db.public.users"
        );
        assert!(check_identifier("users; drop table x").is_err());
        assert!(check_identifier("1users").is_err());
        assert!(check_identifier("\"users\"").is_err());
        assert!(check_identifier("a.b.c.d").is_err());
    }
}

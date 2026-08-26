//! Turn a templated SQL body into a driver-ready `(query, params)` pair.
//!
//! Instead of pasting values into the statement text, every placeholder is
//! rewritten to the driver's own placeholder (`?`, `$1`, `:name`, `%s`,
//! `%(name)s`) and the values are collected positionally, so the driver binds
//! them and injection stops being possible at all.
//!
//! Two rewrites need care and are handled here:
//!
//! * `'{name}'` — a placeholder wrapped in quotes must lose the quotes,
//!   otherwise `?` ends up inside a string literal and is never bound.
//! * `'prefix_{name}'` — a placeholder that is only *part* of a literal cannot
//!   be bound at all; that is reported as an error instead of silently
//!   producing `'prefix_?'`.

use crate::enums::{ParamStyle, SqlParamStyle};
use crate::models::Subtask;
use std::collections::HashMap;

/// A SQL body rewritten for binding, plus the parameter names in bind order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundQuery {
    pub query: String,
    /// Parameter names in the order the driver will bind them.
    pub names: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct Placeholder {
    start: usize,
    end: usize,
}

/// Byte ranges of single-quoted string literals, skipping over comments.
/// Ranges include the surrounding quotes.
fn quoted_regions(text: &str) -> Vec<(usize, usize)> {
    let b = text.as_bytes();
    let mut regions = Vec::new();
    let mut i = 0usize;

    while i < b.len() {
        match b[i] {
            b'-' if i + 1 < b.len() && b[i + 1] == b'-' => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < b.len() && b[i + 1] == b'*' => {
                i += 2;
                while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(b.len());
            }
            b'\'' => {
                let start = i;
                i += 1;
                while i < b.len() {
                    if b[i] == b'\'' {
                        // `''` is an escaped quote, not the end of the literal.
                        if i + 1 < b.len() && b[i + 1] == b'\'' {
                            i += 2;
                            continue;
                        }
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                regions.push((start, i));
            }
            _ => i += 1,
        }
    }

    regions
}

/// Every placeholder in `text`, left to right, with overlaps resolved in favour
/// of the match that starts earliest and — on a tie — spans furthest, so
/// `{{env}}` wins over the `{{` prefix and `${db}` over `{`.
fn collect_placeholders(text: &str, styles: Option<&[ParamStyle]>) -> Vec<(Placeholder, String)> {
    let default_styles = Subtask::default_param_styles();
    let use_styles = styles.unwrap_or(&default_styles);

    let mut found: Vec<(Placeholder, String)> = Vec::new();
    for &style in use_styles.iter() {
        let re = Subtask::regex_for_style(style);
        for caps in re.captures_iter(text) {
            let (Some(whole), Some(name)) = (caps.get(0), caps.name("name")) else {
                continue;
            };
            found.push((
                Placeholder {
                    start: whole.start(),
                    end: whole.end(),
                },
                name.as_str().to_string(),
            ));
        }
    }

    found.sort_by(|a, b| {
        a.0.start
            .cmp(&b.0.start)
            .then((b.0.end - b.0.start).cmp(&(a.0.end - a.0.start)))
    });

    let mut selected: Vec<(Placeholder, String)> = Vec::new();
    let mut cursor = 0usize;
    for (ph, name) in found {
        if ph.start < cursor {
            continue;
        }
        cursor = ph.end;
        selected.push((ph, name));
    }
    selected
}

/// Rewrite `text` for binding.
///
/// `inline` maps parameter names that must be pasted verbatim (identifiers —
/// no driver can bind a table name) to their already-validated literal text.
pub fn bind_text(
    text: &str,
    styles: Option<&[ParamStyle]>,
    sql_style: SqlParamStyle,
    inline: &HashMap<String, String>,
) -> Result<BoundQuery, String> {
    let regions = quoted_regions(text);
    let bytes = text.as_bytes();

    let mut query = String::with_capacity(text.len());
    let mut names: Vec<String> = Vec::new();
    let mut cursor = 0usize;

    for (ph, name) in collect_placeholders(text, styles) {
        if let Some(literal) = inline.get(&name) {
            query.push_str(&text[cursor..ph.start]);
            query.push_str(literal);
            cursor = ph.end;
            continue;
        }

        // A placeholder hugged by quotes (`'{name}'`) replaces the whole
        // literal, quotes included.
        let hugged = regions
            .iter()
            .find(|(s, e)| ph.start > 0 && *s == ph.start - 1 && *e == ph.end + 1)
            .copied();

        let range = match hugged {
            Some((s, e)) => Placeholder { start: s, end: e },
            None => {
                if regions.iter().any(|(s, e)| *s < ph.start && *e > ph.end) {
                    return Err(format!(
                        "parameter '{}' sits inside a quoted literal together with other text; \
                         it cannot be bound. Rewrite the template so the placeholder is the whole \
                         literal (e.g. '{{{}}}'), or apply it with apply_parameters()",
                        name, name
                    ));
                }
                if ph.start > 0 && bytes[ph.start - 1] == b'"' && bytes.get(ph.end) == Some(&b'"') {
                    return Err(format!(
                        "parameter '{}' is a quoted identifier; drivers cannot bind identifiers. \
                         Pass it in `identifiers` to inline it after validation",
                        name
                    ));
                }
                ph
            }
        };

        // Placeholders can only ever move forward: overlaps were resolved above
        // and quote-hugging extends by exactly one byte on each side.
        if range.start < cursor {
            return Err(format!(
                "parameter '{}' overlaps a previously bound placeholder",
                name
            ));
        }

        query.push_str(&text[cursor..range.start]);
        let index = if sql_style.is_named() {
            names.iter().position(|n| *n == name).unwrap_or(names.len())
        } else {
            names.len()
        };
        query.push_str(&sql_style.placeholder(&name, index));
        names.push(name);
        cursor = range.end;
    }

    query.push_str(&text[cursor..]);
    Ok(BoundQuery { query, names })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inline_none() -> HashMap<String, String> {
        HashMap::new()
    }

    fn bind(text: &str, style: SqlParamStyle) -> BoundQuery {
        bind_text(text, None, style, &inline_none()).unwrap()
    }

    #[test]
    fn binds_bare_placeholder() {
        let out = bind("SELECT * FROM users WHERE id = {id}", SqlParamStyle::Qmark);
        assert_eq!(out.query, "SELECT * FROM users WHERE id = ?");
        assert_eq!(out.names, vec!["id"]);
    }

    #[test]
    fn strips_quotes_around_placeholder() {
        let out = bind(
            "SELECT * FROM users WHERE name = '{name}' AND login = '{login}'",
            SqlParamStyle::Qmark,
        );
        assert_eq!(
            out.query,
            "SELECT * FROM users WHERE name = ? AND login = ?"
        );
        assert_eq!(out.names, vec!["name", "login"]);
    }

    #[test]
    fn mixed_styles_in_one_pass() {
        let out = bind(
            "SELECT {a}, $b, ${c}, {{d}}, __e__, %f%, <g>",
            SqlParamStyle::Qmark,
        );
        assert_eq!(out.query, "SELECT ?, ?, ?, ?, ?, ?, ?");
        assert_eq!(out.names, vec!["a", "b", "c", "d", "e", "f", "g"]);
    }

    #[test]
    fn numeric_style_numbers_each_occurrence() {
        let out = bind("SELECT {a}, {b}, {a}", SqlParamStyle::Numeric);
        assert_eq!(out.query, "SELECT $1, $2, $3");
        assert_eq!(out.names, vec!["a", "b", "a"]);
    }

    #[test]
    fn named_style_reuses_the_same_placeholder() {
        let out = bind("SELECT {a}, {b}, {a}", SqlParamStyle::Named);
        assert_eq!(out.query, "SELECT :a, :b, :a");

        let out = bind("SELECT {a}, {b}, {a}", SqlParamStyle::Pyformat);
        assert_eq!(out.query, "SELECT %(a)s, %(b)s, %(a)s");
    }

    #[test]
    fn format_style() {
        let out = bind("SELECT * FROM t WHERE a = '{a}'", SqlParamStyle::Format);
        assert_eq!(out.query, "SELECT * FROM t WHERE a = %s");
    }

    #[test]
    fn partial_literal_is_rejected() {
        let err = bind_text(
            "SELECT * FROM t WHERE name LIKE '%{q}%'",
            None,
            SqlParamStyle::Qmark,
            &inline_none(),
        )
        .unwrap_err();
        assert!(err.contains("quoted literal"), "{}", err);
    }

    #[test]
    fn identifiers_are_inlined() {
        let mut inline = HashMap::new();
        inline.insert("tbl".to_string(), "public.users".to_string());
        let out = bind_text(
            "SELECT * FROM {tbl} WHERE id = {id}",
            None,
            SqlParamStyle::Qmark,
            &inline,
        )
        .unwrap();
        assert_eq!(out.query, "SELECT * FROM public.users WHERE id = ?");
        assert_eq!(out.names, vec!["id"]);
    }

    #[test]
    fn quoted_identifier_needs_inlining() {
        let err = bind_text(
            r#"SELECT * FROM "{tbl}""#,
            None,
            SqlParamStyle::Qmark,
            &inline_none(),
        )
        .unwrap_err();
        assert!(err.contains("identifiers"), "{}", err);
    }

    #[test]
    fn comments_and_escaped_quotes_do_not_confuse_the_scanner() {
        let out = bind(
            "-- it's a comment {ignored_in_comment}\nSELECT 'a''b', {id}",
            SqlParamStyle::Qmark,
        );
        // The comment is still text: its placeholder is bound like any other.
        assert_eq!(out.names, vec!["ignored_in_comment", "id"]);
        assert!(out.query.contains("'a''b', ?"), "{}", out.query);
    }

    #[test]
    fn text_without_placeholders_is_untouched() {
        let out = bind("SELECT 1", SqlParamStyle::Qmark);
        assert_eq!(out.query, "SELECT 1");
        assert!(out.names.is_empty());
    }
}

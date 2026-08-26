# Changelog

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.4.0] - 2026-08-26

### Added

- **SQL parameter binding.** `Subtask.prepare()` rewrites a command template
  into a driver-ready statement instead of interpolating values:

  ```python
  query, params = task.prepare({"name": "alice", "id": 7})
  cursor.execute(query, params)
  ```

  Placeholders wrapped in quotes (`'{name}'`) lose their quotes so the value is
  genuinely bound, and values keep their Python types.
- `SqlParamStyle` covering every PEP 249 `paramstyle`: `Qmark` (`?`, default),
  `Numeric` (`$1`), `Named` (`:name`), `Format` (`%s`), `Pyformat`
  (`%(name)s`). Named styles return a `dict` of parameters, positional styles a
  `list`.
- `PreparedQuery`, which unpacks directly: `query, params = task.prepare(...)`.
  Also exposes `.query`, `.params`, `.names`, `.param_style` and `.as_tuple()`.
- `prepare(identifiers=[...])` inlines parameters that no driver can bind
  (table/schema/column names) after validating them as SQL identifiers.
- **Baseline SQL-injection guards** on interpolated values. Enabled
  automatically for `TaskType.Sql`, overridable per call with
  `apply_parameters(..., guard=True/False)` and
  `render_with_params(..., guard=...)`. Rejects quote characters, `;`, comment
  markers, backslashes, control characters, boolean tautologies and statement
  keywords. Only parameters that appear in the command are checked.
- `SqlGuard` exposing the checks directly: `is_safe()`, `find_issues()`,
  `check()`, `check_identifier()`.
- **Dialect-aware statement analysis** via `sqlparser`. `Subtask.analyze()`
  returns a `SqlAnalysis` with the statement kinds a task runs, the tables it
  touches, warnings about risky constructs (unqualified `DELETE`/`UPDATE`,
  `DROP`, `TRUNCATE`, stacked statements), and `is_destructive`. Dialects follow
  `system_type`; Vertica and unknown systems use `generic`.
  Analysis **fails open**: vendor syntax no parser knows yields `parsed = False`
  and an empty verdict, never an exception — `is_destructive == False` there
  means *not analysed*, not *safe*.
- `StatementKind` enum with `is_destructive`, covering select/insert/update/
  delete/merge/truncate/drop/create/alter/grant/revoke/copy/call/transaction.
- `forbid=[StatementKind...]` on `apply_parameters()`, `render_with_params()`
  and `prepare()`, rejecting tasks that run the given statement kinds. Nothing
  is forbidden by default, and the check fails open. Since it runs on the
  rendered statement, it also catches a statement smuggled in by an
  interpolated value.
- `subtask_manager.__version__`.
- `CLAUDE.md` with repository conventions and gotchas.

### Changed

- **`ParamType` is now `ParamStyle`.** It describes the shape of a placeholder,
  not the type of a value. The old name still resolves with a
  `DeprecationWarning` and will be removed in 0.5.0.
- The `Makefile` was replaced by a [`justfile`](https://just.systems). `just`
  lists every recipe; `just test-py` now rebuilds the extension first, so tests
  can no longer run against a stale `.so`.
- `just bump` keeps `subtask_manager/__init__.py` in sync alongside
  `Cargo.toml` and `pyproject.toml`, and `just show-version` fails on drift.
- `just lint` runs `cargo clippy -D warnings` in addition to `ruff check`; the
  existing clippy findings were fixed.
- The release profile now strips symbols and uses thin LTO. Adding the SQL
  parser grew the published Linux wheel from ~1.9 MB to ~4.3 MB.

### Deprecated

- `subtask_manager.ParamType` — use `ParamStyle`. Removal in 0.5.0.

## [0.3.0]

- Lazy loading in `SubtaskManager`, lightweight `RenderedSubtask` renders,
  immutable parameter application, and cross-platform wheel publishing.

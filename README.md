# subtask-manager

`subtask-manager` is a Rust-powered Python package for discovering, classifying, loading, and rendering ETL subtasks from a filesystem structure.

It is designed for ETL projects where task metadata is encoded in folder names (entity, stage, system) and task content lives in files (`.sql`, `.py`, `.sh`, etc.).

---

## Features

- Fast core implementation in Rust (PyO3 extension module)
- Python-friendly API
- Recursive file scanning by supported extensions
- Automatic classification of tasks from folder structure
- Lazy loading of task contents
- Rich filtering (`stage`, `entity`, `system_type`, `task_type`, `is_common`)
- Parameter extraction and rendering with multiple placeholder styles
- Immutable parameter application (returns new objects)
- SQL parameter **binding**: templates become `(query, params)` for any DB-API driver
- Baseline SQL-injection guards on values that are interpolated instead of bound
- Dialect-aware statement analysis: what a task runs, which tables it touches,
  and whether it is destructive

---

## Installation

### From PyPI

```bash
pip install subtask-manager
```

### From source (local dev)

```bash
# Build extension and install in editable/dev mode
maturin develop
```

Or build wheels:

```bash
maturin build --release
```

---

## Supported task types (by extension)

- SQL: `sql`, `psql`, `tsql`, `plpgsql`
- Shell: `sh`
- PowerShell: `ps1`
- Python: `py`
- GraphQL: `graphql`, `gql`
- JSON: `json`, `jsonl`
- YAML: `yaml`, `yml`

---

## Folder conventions

Classification is based on the file path relative to a base directory.

Expected relative folder depth: up to 3 components before the file.

Typical pattern:

```text
<base>/<entity>/<stage>/<system>/<task_file>
```

Examples:

- `customers/01_extract/pg/extract_data.sql`
- `orders/02_transform/duck/normalize.py`

### Common tasks

A file directly under `<base>` is treated as a **common task**:

```text
<base>/shared.yaml
```

---

## Enums and aliases

### `EtlStage`
- `Setup`
- `Extract`
- `Transform`
- `Load`
- `Cleanup`
- `Postprocessing`
- `Other`

Recognized aliases include names like:
- `01_extract`, `extract`, `e`, `01`
- etc.

### `SystemType`
Includes:
- `PostgreSQL`, `Duckdb`, `Clickhouse`, `MySQL`, `OracleDB`, `SQLite`, `SqlServer`, `Vertica`, `Other`

Example aliases:
- `pg`, `postgres`, `duck`, `duckdb`, etc.

### `TaskType`
- `Sql`, `Shell`, `Powershell`, `Python`, `Graphql`, `Json`, `Yaml`, `Other`

---

## Quick usage

```python
from pathlib import Path
from subtask_manager import SubtaskManager, EtlStage, SystemType, ParamStyle

base = Path("tests/test_data/subtasks")
sm = SubtaskManager(base)

print(sm.base_path)
print(sm.num_files)
print(sm.file_paths[:3])

# Lazy-loaded subtasks
tasks = sm.subtasks
print(len(tasks))

# Get a single task
task = sm.get_task("extract_data.sql")
print(task.name, task.entity, task.stage, task.system_type)

# Filter tasks
extract_pg = sm.get_tasks(
    etl_stage=EtlStage.Extract,
    system_type=SystemType.PostgreSQL,
    include_common=False,
)
print(len(extract_pg))

# Inspect parameter names
params = task.get_params()
print(params)

# Apply parameters immutably
rendered = task.apply_parameters(
    {"date": "2025-01-01", "env": "prod"},
    styles=[ParamStyle.Curly, ParamStyle.DollarBrace],
    ignore_missing=True,
)

print(rendered.get_command())
```

---

## Parameter styles

Supported placeholder styles:

- `Curly`: `{name}`
- `Dollar`: `$name`
- `DollarBrace`: `${name}`
- `DoubleCurly`: `{{name}}`
- `DoubleUnderscore`: `__name__`
- `Percent`: `%name%`
- `Angle`: `<name>`

> **Renamed in 0.4.0:** `ParamType` is now `ParamStyle` — it describes the
> *shape of a placeholder*, not the type of a value. The old name still works
> with a `DeprecationWarning` and will be removed in 0.5.0.

Useful methods:

- `subtask.get_params(styles=None) -> set[str]`
- `subtask.apply_parameters(params, styles=None, ignore_missing=False, guard=None) -> Subtask`
- `subtask.prepare(params, styles=None, param_style=SqlParamStyle.Qmark, identifiers=None, ignore_missing=False, forbid=None) -> PreparedQuery`
- `subtask.analyze(params=None, styles=None) -> SqlAnalysis`
- `subtask.render_with_params(params, styles=None, ignore_missing=False, guard=None) -> RenderedSubtask`
- `subtask.render() -> Subtask`
- `subtask.render_lightweight() -> RenderedSubtask`
- `subtask.get_stored_params() -> dict[str, str]`
- `subtask.get_command() -> str | None`

---

## SQL parameters: bind, don't interpolate

`apply_parameters()` pastes values into the statement text. That is fine for
identifiers and shaping a query, but a value that reaches the database that way
is indistinguishable from SQL. `prepare()` instead rewrites the template into a
statement your driver binds:

```python
from subtask_manager import SubtaskManager, SqlParamStyle

task = SubtaskManager("tasks").get_task("find_user.sql")
# SELECT * FROM users WHERE name = '{name}' AND id = {id}

query, params = task.prepare({"name": "alice", "id": 7})

print(query)   # SELECT * FROM users WHERE name = ? AND id = ?
print(params)  # ['alice', 7]

cursor.execute(query, params)
```

Note that `'{name}'` lost its quotes: the value is bound, not spliced into a
string literal. Values also keep their Python types — `7` stays an `int`, `None`
stays `NULL`, a `datetime` is handed to the driver as a `datetime`.

### Driver placeholder styles

`param_style` follows PEP 249 `paramstyle`:

| `SqlParamStyle` | Placeholder | Typical drivers | `params` |
| --- | --- | --- | --- |
| `Qmark` (default) | `?` | sqlite3, duckdb, pyodbc | `list` |
| `Numeric` | `$1`, `$2` | asyncpg, psycopg (native) | `list` |
| `Named` | `:name` | oracledb, SQLAlchemy `text()` | `dict` |
| `Format` | `%s` | psycopg2, mysqlclient | `list` |
| `Pyformat` | `%(name)s` | psycopg2, mysqlclient | `dict` |

```python
task.prepare({"id": 7}, param_style=SqlParamStyle.Numeric).query
# SELECT * FROM users WHERE id = $1
```

### Identifiers

No driver can bind a table or schema name, so those must be inlined. Name them
explicitly and they are validated as identifiers before being pasted in:

```python
task.prepare({"tbl": "public.users", "id": 7}, identifiers=["tbl"])
# query : SELECT * FROM public.users WHERE id = ?
# params: [7]

task.prepare({"tbl": "users; DROP TABLE t"}, identifiers=["tbl"])
# ValueError: invalid SQL identifier: "users; DROP TABLE t"
```

### What `prepare()` refuses

A placeholder that is only *part* of a string literal cannot be bound, so it is
an error rather than a silently broken query:

```python
# command: SELECT * FROM t WHERE name LIKE '%{q}%'
task.prepare({"q": "ab"})
# ValueError: parameter 'q' sits inside a quoted literal together with other text
```

Pass the wildcards in the value instead (`{"q": "%ab%"}` with the template
`LIKE {q}`), or fall back to `apply_parameters()` for that template.

---

## SQL injection guards

When a value *is* interpolated, `subtask-manager` checks it first. The guard is
on automatically for SQL tasks and rejects anything that could end a literal,
start a statement, or open a comment:

```python
task.apply_parameters({"name": "x'; DROP TABLE users;--", "id": 1})
# ValueError: unsafe value for parameter 'name': quote character ("'") at offset 1;
# statement terminator (';') at offset 2; SQL keyword ("DROP TABLE") at offset 4; ...
```

Rejected: quote characters (`'`, `"`, `` ` ``), `;`, `--`, `/*`, `*/`,
backslashes, control characters, boolean tautologies (`OR 1=1`), and statement
keywords (`UNION SELECT`, `DROP TABLE`, `xp_cmdshell`, `pg_sleep(`, ...).

Control it per call with `guard`:

| `guard` | Behaviour |
| --- | --- |
| `None` (default) | On for `TaskType.Sql`, off for everything else |
| `True` | Always on, whatever the task type |
| `False` | Off — you take responsibility for the value |

Only parameters that appear in the **command** are checked; values used solely
in a path or name never reach the SQL text.

The checks are also available directly:

```python
from subtask_manager import SqlGuard

SqlGuard.is_safe("prod")                # True
SqlGuard.find_issues("1 OR 1=1")        # ['boolean tautology ("OR 1=1") at offset 2']
SqlGuard.check(value, name="user_name") # raises ValueError if unsafe
SqlGuard.check_identifier("public.users")
```

The guard is a safety net, not a substitute for binding. A legitimate value
containing an apostrophe (`O'Brien`) is rejected precisely because there is no
dialect-independent way to interpolate it safely — bind it with `prepare()`.

---

## Statement analysis

`subtask.analyze()` parses the task body with the dialect implied by its
`system_type` and reports what it actually does:

```python
task = sm.get_task("purge_events.sql")   # DELETE FROM analytics.{tbl}

analysis = task.analyze()
analysis.parsed          # True
analysis.dialect         # 'postgres'
analysis.statements      # [StatementKind.Delete]
analysis.tables          # ['analytics.{tbl}']
analysis.warnings        # ['DELETE on analytics.{tbl} has no WHERE clause: it removes every row']
analysis.is_destructive  # True
```

A template is not valid SQL on its own, so placeholders are substituted with
sentinels before parsing and mapped back afterwards — which is why the table
above is reported as `{tbl}` rather than a made-up name. Pass `params` to
analyse the *rendered* statement instead.

Dialects follow `system_type`: `postgres`, `mysql`, `duckdb`, `clickhouse`,
`sqlite`, `oracle`, `mssql`. Vertica has no upstream dialect and falls back to
`generic`, as does a task whose system could not be determined.

### Analysis always fails open

Real ETL SQL is full of vendor syntax no parser knows. When the body does not
parse you get a result, not an exception:

```python
analysis.parsed          # False
analysis.error           # 'sql parser error: Expected: an object type after CREATE, ...'
analysis.statements      # []
analysis.is_destructive  # False
```

`is_destructive == False` on an unparsed body means **not analysed**, never
*safe*. Check `parsed` before drawing any conclusion.

### Refusing destructive tasks

`forbid` rejects a task whose SQL runs a given statement kind. It works on
`apply_parameters()`, `render_with_params()` and `prepare()`:

```python
task.apply_parameters(params, forbid=[StatementKind.Drop, StatementKind.Truncate])
# ValueError: subtask 'purge.sql' runs forbidden statement(s): truncate on events
```

Nothing is forbidden by default — ETL legitimately drops and truncates staging
tables. Like analysis itself, `forbid` fails open: SQL that cannot be parsed is
allowed through.

Because the check runs on the *rendered* statement, it also catches a statement
that an interpolated value smuggled in:

```python
task.apply_parameters(
    {"name": "x'; DROP TABLE users; --"}, guard=False, forbid=[StatementKind.Drop]
)
# ValueError: subtask 'q.sql' runs forbidden statement(s): drop on users
```

---

## Public classes

- `SubtaskManager`
- `Subtask`
- `RenderedSubtask`
- `FileScanner`
- `FileClassifier`
- `EtlStage`
- `SystemType`
- `TaskType`
- `ParamStyle`
- `SqlParamStyle`
- `PreparedQuery`
- `SqlGuard`
- `SqlAnalysis`
- `StatementKind`

---

## Development

### Prerequisites

- Rust toolchain
- Python 3.12+
- `uv` (recommended) or `pip`
- `maturin`

### Install dev dependencies

```bash
uv sync --dev
```

- [`just`](https://just.systems) (task runner)

### Run tests

```bash
just test        # Rust + Python, rebuilding the extension first
just test-rust
just test-py
```

### Lint/format

```bash
just lint        # ruff check + clippy -D warnings
just format      # ruff format + cargo fmt
just check       # lint + format check + tests
```

`just` on its own lists every recipe.

---

## Build and release

Cross-platform wheel publishing is automated with GitHub Actions.

See the full runbook:

- [RELEASE.md](./RELEASE.md)

It documents:
- TestPyPI dry runs
- PyPI production release flow
- Trusted Publishing setup
- version/tag conventions

---

## Versioning notes

Keep versions aligned between:

- `Cargo.toml` (`[package].version`)
- `pyproject.toml` (`[project].version`)
- `subtask_manager/__init__.py` (`__version__`)

```bash
just show-version   # fails if they drift
just bump 0.4.1     # sets all three
just bump-patch     # or bump-minor / bump-major
```

Changes are recorded in [CHANGELOG.md](./CHANGELOG.md).

---

## License

MIT
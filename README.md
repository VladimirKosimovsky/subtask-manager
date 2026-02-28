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
from subtask_manager import SubtaskManager, EtlStage, SystemType, ParamType

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
    styles=[ParamType.Curly, ParamType.DollarBrace],
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

Useful methods:

- `subtask.get_params(styles=None) -> set[str]`
- `subtask.apply_parameters(params, styles=None, ignore_missing=False) -> Subtask`
- `subtask.render_with_params(params, styles=None, ignore_missing=False) -> RenderedSubtask`
- `subtask.render() -> Subtask`
- `subtask.render_lightweight() -> RenderedSubtask`
- `subtask.get_stored_params() -> dict[str, str]`
- `subtask.get_command() -> str | None`

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
- `ParamType`

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

### Run tests

```bash
cargo test
uv run -m pytest
```

or:

```bash
make test
```

### Lint/format (Python)

```bash
uv run ruff check .
uv run ruff format .
```

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

Use Makefile version helpers (if present) to bump consistently.

---

## License

MIT
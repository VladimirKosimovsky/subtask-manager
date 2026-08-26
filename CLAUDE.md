# CLAUDE.md

Guidance for Claude Code when working in this repository.

## What this is

`subtask-manager` is a Rust core (PyO3) exposed as the Python package
`subtask_manager`. It discovers ETL task files on disk, derives metadata from
the folder layout, loads their contents, and turns templated bodies into
runnable statements — either by interpolation or by binding them for a DB
driver.

The Rust crate is `subtask_manager`; the compiled module is `_core`, re-exported
by `subtask_manager/__init__.py`.

## Commands

The task runner is [`just`](https://just.systems) (`just` alone lists recipes).
There is no Makefile.

```bash
just test            # cargo test + pytest (rebuilds the extension first)
just test-rust       # cargo test only
just test-py         # pytest only, against a fresh build
just lint            # ruff check + clippy -D warnings
just format          # ruff format + cargo fmt
just check           # lint + format-check + test
just develop         # maturin develop into .venv
just build-release   # release wheel
just demo            # run main.py against tests/test_data
just show-version    # assert Cargo.toml/pyproject.toml/__init__.py agree
just bump 0.4.1      # set the version in all three
```

Python tests import the **compiled** extension. After touching anything under
`src/`, run `just develop` (or `just test-py`, which does it for you) or you
will be testing a stale `.so`.

## Layout

| Path | Role |
| --- | --- |
| `src/lib.rs` | PyO3 surface: `#[pymethods]` blocks and the `_core` module |
| `src/models.rs` | `Subtask` / `RenderedSubtask` and all template logic |
| `src/enums.rs` | `EtlStage`, `SystemType`, `TaskType`, `ParamStyle`, `SqlParamStyle` |
| `src/sql_guard.rs` | Injection heuristics for interpolated values |
| `src/sql_analysis.rs` | `sqlparser`-backed statement analysis and `StatementKind` |
| `src/sql_binding.rs` | Template → `(query, names)` rewriting for driver binding |
| `src/py_sql.rs` | `PreparedQuery` and `SqlGuard` pyclasses |
| `src/file_scanner.rs` | Recursive extension-filtered walk |
| `src/file_classifier.rs` | Path → `Subtask` metadata |
| `src/file_loader.rs` | Reads file content into `Subtask.command` |
| `subtask_manager/_core.pyi` | Hand-maintained stubs — update with every API change |
| `python/` | The original pure-Python prototype. Reference only, not shipped, not imported |

## Conventions

**Rust logic stays in plain `impl` blocks; `#[pymethods]` are thin wrappers.**
`src/models.rs` contains no PyO3 argument handling beyond the derives, so its
logic is unit-testable with `cargo test`. New behaviour goes in the plain impl,
with the Python binding in `src/lib.rs` named `<name>_py` and exported via
`#[pyo3(name = "...")]`.

**`Subtask` is immutable.** Every operation returns a new instance.
`original_name`, `original_path` and `command` are the never-mutated templates;
`name`, `path` and `rendered_command` hold rendered output. Rendering always
reads from the originals, so it can be repeated with different parameters.

**Enums carry a metadata table.** Each enum has a private
`OnceLock<HashMap<Self, …Data>>` holding `id`, `name`, and `aliases` (or
`extensions`), with `from_alias` matching case-insensitively. Follow that shape
when adding a variant or a new enum, and expose `id`/`name`/`aliases` getters
plus `from_alias` in `src/lib.rs`.

**Tests live next to what they test.** Rust logic gets a `#[cfg(test)] mod
tests` in its own file; the Python-facing behaviour gets a file under `tests/`.

## Things that bite

- **Placeholder precedence.** `ParamStyle::Curly`'s regex deliberately matches
  `${` and `{{` in alternations that capture no `name`. That is how `{x}` avoids
  eating `${x}` and `{{x}}`. Matches without a `name` capture must be skipped,
  never treated as parameters.
- **`sql_binding` resolves overlaps by "earliest start, then longest match"**,
  which is what makes `{{env}}` win over the `{{` prefix. Changing a style's
  regex can silently change that ordering — the tests in
  `src/sql_binding.rs` cover it.
- **A placeholder inside a larger string literal cannot be bound.** `'{name}'`
  becomes `?` (quotes consumed); `'%{name}%'` is an error, on purpose. Do not
  "fix" it by emitting `'%?%'`.
- **The SQL guard defaults to on for `TaskType::Sql` only** (`guard=None`).
  It rejects quotes, `;`, comment markers, backslashes, control characters and
  obvious injection patterns — so legitimate values containing those must be
  bound with `prepare()`, not interpolated.
- **Statement analysis fails open, and that is load-bearing.** `sqlparser`
  cannot read plenty of real vendor SQL — this repo's own
  `attach_pg_to_duckdb_with_params.sql` fails in every dialect. So
  `analyze()` returns `parsed = false` rather than raising, and `forbid`
  lets unparseable SQL through. Never invert that: `is_destructive == false`
  on an unparsed body means *not analysed*, not *safe*.
- **Templates are parsed via sentinels.** `FROM {tbl}` is not SQL, so
  placeholders become `_p_<name>` (identifier pass) or `1` (numeric pass, for
  positions like `LIMIT {n}`), and the identifier pass is mapped back so the
  report names `{tbl}` rather than the sentinel. Adding a placeholder style
  means checking both passes still parse.
- **`Statement::Drop` hides its targets from the relation visitor.**
  `visit_relations` does not descend into `Drop.names`, so `relations_of`
  collects them by hand. Other statement kinds may have the same gap.
- **Keep three versions in sync**: `Cargo.toml`, `pyproject.toml`,
  `subtask_manager/__init__.py`. `just bump` does all three; `just show-version`
  fails if they drift.
- **`ParamType` was renamed to `ParamStyle` in 0.4.0.** The old name resolves
  through `__getattr__` in `__init__.py` with a `DeprecationWarning` and is
  scheduled for removal in 0.5.0. Do not reintroduce it in new code.

from collections.abc import Iterator, Mapping, Sequence
from pathlib import Path
from typing import Any

from typing_extensions import override

class TaskType:
    id: int
    name: str
    extensions: list[str]

    Sql: "TaskType"
    Shell: "TaskType"
    Powershell: "TaskType"
    Python: "TaskType"
    Graphql: "TaskType"
    Json: "TaskType"
    Yaml: "TaskType"
    Other: "TaskType"

    def __init__(self, *args: object, **kwargs: object) -> None: ...
    @classmethod
    def from_extension(cls, extension: str) -> "TaskType": ...
    @override
    def __str__(self) -> str: ...
    @override
    def __repr__(self) -> str: ...
    @override
    def __eq__(self, other: object) -> bool: ...
    @override
    def __ne__(self, other: object) -> bool: ...
    @override
    def __hash__(self) -> int: ...

class SystemType:
    id: int
    name: str
    aliases: list[str]

    Clickhouse: "SystemType"
    Duckdb: "SystemType"
    MySQL: "SystemType"
    OracleDB: "SystemType"
    PostgreSQL: "SystemType"
    SQLite: "SystemType"
    SqlServer: "SystemType"
    Vertica: "SystemType"
    Other: "SystemType"

    def __init__(self, *args: object, **kwargs: object) -> None: ...
    @classmethod
    def from_alias(cls, alias: str) -> "SystemType": ...
    @override
    def __str__(self) -> str: ...
    @override
    def __repr__(self) -> str: ...
    @override
    def __eq__(self, other: object) -> bool: ...
    @override
    def __ne__(self, other: object) -> bool: ...
    @override
    def __hash__(self) -> int: ...

class EtlStage:
    id: int
    name: str
    aliases: list[str]

    Setup: "EtlStage"
    Extract: "EtlStage"
    Transform: "EtlStage"
    Load: "EtlStage"
    Cleanup: "EtlStage"
    Postprocessing: "EtlStage"
    Other: "EtlStage"

    def __init__(self, *args: object, **kwargs: object) -> None: ...
    @classmethod
    def from_alias(cls, alias: str) -> "EtlStage": ...
    @override
    def __str__(self) -> str: ...
    @override
    def __repr__(self) -> str: ...
    @override
    def __eq__(self, other: object) -> bool: ...
    @override
    def __ne__(self, other: object) -> bool: ...
    @override
    def __hash__(self) -> int: ...

class ParamStyle:
    id: int
    name: str
    aliases: list[str]

    Curly: "ParamStyle"
    Dollar: "ParamStyle"
    DollarBrace: "ParamStyle"
    DoubleCurly: "ParamStyle"
    DoubleUnderscore: "ParamStyle"
    Percent: "ParamStyle"
    Angle: "ParamStyle"
    Other: "ParamStyle"

    def __init__(self, *args: object, **kwargs: object) -> None: ...
    @classmethod
    def from_alias(cls, alias: str) -> "ParamStyle": ...
    @override
    def __repr__(self) -> str: ...
    @override
    def __str__(self) -> str: ...
    @override
    def __eq__(self, other: object) -> bool: ...
    @override
    def __ne__(self, other: object) -> bool: ...
    @override
    def __hash__(self) -> int: ...

class SqlParamStyle:
    """Placeholder syntax of the target DB driver (PEP 249 `paramstyle`)."""

    id: int
    name: str
    aliases: list[str]
    is_named: bool

    Qmark: "SqlParamStyle"
    """`?` - sqlite3, duckdb, pyodbc."""
    Numeric: "SqlParamStyle"
    """`$1`, `$2` - asyncpg, psycopg native."""
    Named: "SqlParamStyle"
    """`:name` - oracledb, sqlalchemy text()."""
    Format: "SqlParamStyle"
    """`%s` - psycopg2, mysqlclient."""
    Pyformat: "SqlParamStyle"
    """`%(name)s` - psycopg2, mysqlclient (named form)."""

    def __init__(self, *args: object, **kwargs: object) -> None: ...
    @classmethod
    def from_alias(cls, alias: str) -> "SqlParamStyle": ...
    @override
    def __str__(self) -> str: ...
    @override
    def __repr__(self) -> str: ...
    @override
    def __eq__(self, other: object) -> bool: ...
    @override
    def __ne__(self, other: object) -> bool: ...
    @override
    def __hash__(self) -> int: ...

class StatementKind:
    """The kind of statement found in a task body."""

    id: int
    name: str
    aliases: list[str]
    is_destructive: bool
    """True for statements that destroy data or schema irreversibly."""

    Select: "StatementKind"
    Insert: "StatementKind"
    Update: "StatementKind"
    Delete: "StatementKind"
    Merge: "StatementKind"
    Truncate: "StatementKind"
    Drop: "StatementKind"
    Create: "StatementKind"
    Alter: "StatementKind"
    Grant: "StatementKind"
    Revoke: "StatementKind"
    Copy: "StatementKind"
    Call: "StatementKind"
    Transaction: "StatementKind"
    Other: "StatementKind"

    def __init__(self, *args: object, **kwargs: object) -> None: ...
    @classmethod
    def from_alias(cls, alias: str) -> "StatementKind": ...
    @override
    def __str__(self) -> str: ...
    @override
    def __repr__(self) -> str: ...
    @override
    def __eq__(self, other: object) -> bool: ...
    @override
    def __ne__(self, other: object) -> bool: ...
    @override
    def __hash__(self) -> int: ...

class SqlAnalysis:
    """What a SQL body was found to do.

    Analysis always fails open: SQL the dialect cannot parse comes back with
    `parsed = False` and an empty verdict. That means *not analysed*, never
    *safe*.
    """

    parsed: bool
    """False when the dialect could not parse the body."""
    dialect: str
    """The `sqlparser` dialect used, derived from the task's `system_type`."""
    statements: list[StatementKind]
    """Statement kinds, in the order they appear."""
    tables: list[str]
    """Relations referenced, deduplicated. Placeholders are reported by name."""
    warnings: list[str]
    """Notes about risky constructs (unqualified DELETE/UPDATE, DROP, ...)."""
    error: str | None
    """Parser message when `parsed` is False."""
    is_destructive: bool
    """True when any statement is a DROP, TRUNCATE or DELETE."""

    def has(self, kind: StatementKind) -> bool:
        """Whether a given statement kind appears in the body."""
        ...

    @override
    def __repr__(self) -> str: ...
    @override
    def __str__(self) -> str: ...

class PreparedQuery:
    """A driver-ready statement: rewritten SQL plus the values to bind.

    Unpacks straight into a DB-API call::

        query, params = subtask.prepare({"id": 7})
        cursor.execute(query, params)
    """

    query: str
    """SQL with driver placeholders in place of template placeholders."""
    names: list[str]
    """Parameter names in bind order."""
    param_style: SqlParamStyle
    params: Sequence[Any] | Mapping[str, Any]
    """`list` for positional styles, `dict` for named ones."""

    def as_tuple(self) -> tuple[str, Sequence[Any] | Mapping[str, Any]]: ...
    def __len__(self) -> int: ...
    def __iter__(self) -> Iterator[Any]: ...
    def __getitem__(self, index: int) -> Any: ...
    @override
    def __repr__(self) -> str: ...

class SqlGuard:
    """Baseline SQL-injection checks for *interpolated* values.

    Binding via `Subtask.prepare()` is always preferable; this is the safety net
    for values that have to become part of the SQL text.
    """

    @staticmethod
    def is_safe(value: str) -> bool:
        """True when the value carries none of the rejected patterns."""
        ...

    @staticmethod
    def find_issues(value: str) -> list[str]:
        """Every reason the value would be rejected."""
        ...

    @staticmethod
    def check(value: str, name: str = "value") -> None:
        """Raise `ValueError` if the value is unsafe to interpolate."""
        ...

    @staticmethod
    def check_identifier(value: str) -> str:
        """Validate a table/schema/column name that must be inlined."""
        ...

    @override
    def __repr__(self) -> str: ...

class RenderedSubtask:
    """Lightweight structure containing only rendered values after parameter application."""

    name: str
    path: str
    command: str | None
    params: dict[str, str]

    @override
    def __repr__(self) -> str: ...
    @override
    def __str__(self) -> str: ...

class Subtask:
    original_name: str
    original_path: str

    stage: EtlStage | None
    entity: str | None
    system_type: SystemType | None
    task_type: TaskType | None
    is_common: bool

    name: str
    path: str
    command: str | None
    rendered_command: str | None
    params: set[str] | None

    def __init__(
        self,
        stage: EtlStage | None = None,
        entity: str | None = None,
        system_type: SystemType | None = None,
        task_type: TaskType | None = None,
        is_common: bool = False,
        name: str = "",
        path: str = "",
        command: str | None = None,
    ) -> None: ...
    @override
    def __repr__(self) -> str: ...
    @override
    def __str__(self) -> str: ...
    def apply_parameters(
        self,
        params: dict[str, Any],
        styles: list[ParamStyle] | None = None,
        ignore_missing: bool = False,
        guard: bool | None = None,
        forbid: list[StatementKind] | None = None,
    ) -> "Subtask":
        """
        Apply parameters to this subtask and return a new Subtask with applied parameters.
        The original subtask remains unchanged (immutable).

        Values are interpolated into the command text. `guard=None` runs the SQL
        injection guard for SQL tasks only; pass `True`/`False` to force it.
        Raises `ValueError` when a guarded value is unsafe.

        `forbid` rejects the task when the resulting SQL runs one of the given
        statement kinds. It fails open: SQL that does not parse is allowed
        through, since it cannot be judged.

        Prefer `prepare()` whenever the driver can bind the value.
        """
        ...

    def analyze(
        self,
        params: dict[str, Any] | None = None,
        styles: list[ParamStyle] | None = None,
    ) -> SqlAnalysis:
        """
        Parse the command with the dialect implied by `system_type` and report
        what it does: statement kinds, tables touched, and warnings.

        With `params`, the rendered command is analysed, which also surfaces
        anything an interpolated value added to the statement. Without them a
        sentinel-substituted copy of the template is analysed instead.

        Never raises. Check `parsed` before trusting a negative result.
        """
        ...

    def prepare(
        self,
        params: dict[str, Any] | None = None,
        styles: list[ParamStyle] | None = None,
        param_style: SqlParamStyle = ...,
        identifiers: list[str] | None = None,
        ignore_missing: bool = False,
        forbid: list[StatementKind] | None = None,
    ) -> PreparedQuery:
        """
        Rewrite the command into a driver-ready `(query, params)` pair instead of
        interpolating values into the SQL text.

        Placeholders wrapped in quotes (`'{name}'`) lose their quotes so the value
        is genuinely bound. Names listed in `identifiers` are inlined after
        identifier validation, since no driver can bind a table name.

        Raises `ValueError` if a placeholder cannot be bound (it is only part of a
        string literal), if an identifier is invalid, or if a parameter is missing
        and `ignore_missing` is False.
        """
        ...

    def get_params(self, styles: list[ParamStyle] | None = None) -> set[str]:
        """
        Returns a set of parameter names that are used in the subtask fields.
        """
        ...

    def get_stored_params(self) -> dict[str, str]: ...
    def get_command(self) -> str | None:
        """
        Get the command to execute. Returns rendered_command if available, otherwise command template.
        """
        ...

    def render(self) -> "Subtask":
        """
        Render this subtask - resolves all templates even if no parameters are needed.
        Equivalent to calling apply_parameters with empty params.
        """
        ...

    def render_lightweight(self) -> RenderedSubtask:
        """
        Lightweight render without parameters. Returns only the rendered values.
        """
        ...

    def render_with_params(
        self,
        params: dict[str, Any],
        styles: list[ParamStyle] | None = None,
        ignore_missing: bool = False,
        guard: bool | None = None,
        forbid: list[StatementKind] | None = None,
    ) -> RenderedSubtask:
        """
        Apply parameters and return a lightweight RenderedSubtask with only the output values.
        This is more efficient than apply_parameters() which clones the entire Subtask.
        """
        ...

class SubtaskManager:
    base_path: str
    subtasks: list[Subtask]
    file_paths: list[str]
    num_files: int
    classifier: "FileClassifier"

    def __init__(self, base_path: str | Path) -> None: ...
    def load_all(self) -> None: ...
    def get_tasks(
        self,
        etl_stage: EtlStage | None = None,
        entity: str | None = None,
        system_type: SystemType | None = None,
        task_type: TaskType | None = None,
        is_common: bool | None = None,
        include_common: bool | None = True,
    ) -> list[Subtask]: ...
    def get_task(self, name: str, entity: str | None = None) -> Subtask: ...

class FileScanner:
    """Scanner for finding files with specific extensions."""

    def __init__(self, extensions: list[str]) -> None:
        """
        Initialize FileScanner with file extensions to search for.

        Args:
            extensions: List of file extensions (with or without leading dot)
        """
        ...

    def scan_files(self, base_dir: str | Path) -> list[str]:
        """
        Scan directory recursively for files with matching extensions.

        Args:
            base_dir: Root directory to scan (string path or pathlib.Path)

        Returns:
            List of absolute file paths
        """
        ...

    @property
    def extensions(self) -> list[str]:
        """Get the normalized extensions this scanner searches for."""
        ...

class FileClassifier:
    """Classifier for converting file paths into Subtask objects based on folder structure."""

    base_path: str

    def __init__(self, base_path: str | Path) -> None:
        """
        Initialize FileClassifier with a base path.

        Args:
            base_path: Base directory path (string path or pathlib.Path)
        """
        ...

    def classify(self, file_path: str | Path) -> Subtask:
        """
        Classify a file path into a Subtask based on its location relative to base_path.

        Args:
            file_path: Path to the file to classify (string path or pathlib.Path)

        Returns:
            A Subtask object with extracted metadata

        Raises:
            ValueError: If the folder structure is invalid or task type cannot be determined
        """
        ...

    @override
    def __repr__(self) -> str: ...

from typing import Any

from typing_extensions import override

class SystemType:
    id: int
    name: str
    aliases: list[str]
    Clickhouse: SystemType
    Duckdb: SystemType
    MySQL: SystemType
    OracleDB: SystemType
    PostgreSQL: SystemType
    SQLite: SystemType
    SqlServer: SystemType
    Vertica: SystemType
    Other: SystemType
    # Constructor - private to prevent direct instantiation
    def __init__(self, *args: Any, **kwargs: Any) -> None: ...
    
    @classmethod
    def from_alias(
        cls, alias: str,
    ) -> SystemType: ...
    
    # String representation
    @override
    def __str__(self) -> str: ...
    @override
    def __repr__(self) -> str: ...

    # Comparisons
    @override
    def __eq__(self, other: Any) -> bool: ...
    @override
    def __ne__(self, other: Any) -> bool: ...
    @override
    def __hash__(self) -> int: ...
    

class EtlStage:
    name: str
    aliases: list[str]
    # Enum variants as class attributes
    Extract: EtlStage
    Transform: EtlStage
    Load: EtlStage
    Setup: EtlStage
    Cleanup: EtlStage
    Postprocessing: EtlStage

    # Constructor - private to prevent direct instantiation
    def __init__(self, *args: Any, **kwargs: Any) -> None: ...

    # String representation
    @override
    def __str__(self) -> str: ...
    @override
    def __repr__(self) -> str: ...

    # Comparisons
    @override
    def __eq__(self, other: Any) -> bool: ...
    @override
    def __ne__(self, other: Any) -> bool: ...
    @override
    def __hash__(self) -> int: ...


class Subtask:
    stage: str | None
    entity: str | None
    system_type: str | None
    task_type: str | None
    is_common: bool
    name: str
    path: str
    command: str | None

    def __init__(
        self,
        stage: str | None = None,
        entity: str | None = None,
        system_type: str | None = None,
        task_type: str | None = None,
        is_common: bool = False,
        name: str = "",
        path: str = "",
        command: str | None = None,
    ) -> None: ...
    @override
    def __repr__(self) -> str: ...
    @override
    def __str__(self) -> str: ...

class SubtaskManager:
    base_path: str
    subtasks: list[Subtask]

    def __init__(self, base_path: str) -> None: ...
    def get_tasks(
        self,
        etl_stage: str | None = None,
        entity: str | None = None,
        system_type: str | None = None,
        task_type: str | None = None,
        is_common: bool | None = None,
        include_common: bool | None = True,
    ) -> list[Subtask]: ...
    def get_task(
        self,
        name: str,
        entity: str | None = None,
    ) -> Subtask: ...

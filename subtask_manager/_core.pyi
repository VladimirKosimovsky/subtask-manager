from typing_extensions import override

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

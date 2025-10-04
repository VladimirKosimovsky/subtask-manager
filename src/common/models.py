from pathlib import Path

from pydantic import BaseModel, Field

from common.enums import EtlStage, SystemType, TaskType


class Subtask(BaseModel):
    name: str
    path: Path
    task_type: TaskType | None = Field(default=None, description="Task type")
    system_type: SystemType | None = Field(default=None, description="System type")
    stage: EtlStage | None = Field(default=None, description="Stage name")
    entity: str | None = Field(default=None, description="Entity name")
    is_common: bool = Field(default=False, description="Is common")
    command: str | None = Field(default=None, description="Command")

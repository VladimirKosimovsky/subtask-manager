from pathlib import Path

from common.enums import EtlStage, SystemType, TaskType
from common.models import Subtask


class SubtaskManager:
    def _get_file_content(self, file: Path) -> str:
        with open(file, "r", encoding="utf-8") as f:
            return f.read()

    def __init__(self, path: str | Path):
        self.path: Path = Path(path)
        self.subtasks: list[Subtask] = []
        self._classify_subtasks()

    def _get_subtask_candidates(self):
        # Collect all known extensions into a set (faster lookup)
        all_extensions = {ext for task_type in TaskType for ext in task_type.extensions}
    
        # Walk the filesystem once
        return [
            file
            for file in self.path.rglob("*")
            if file.is_file() and file.suffix.lstrip(".").lower() in all_extensions
        ]

    def _classify_subtasks(self):
        for file in self._get_subtask_candidates():
            parts_to_check = file.parts[len(self.path.parts) : -1]
            file_info = Subtask(
                path=file,
                name=file.name,
                stage=None,
                entity=None,
                system_type=None,
                task_type=None,
                is_common=False,
                command=self._get_file_content(file),
            )
            if len(parts_to_check) == 0:
                file_info.is_common = True
            if len(parts_to_check) > 3:
                raise ValueError("Incorrect folder structure")

            checked_parts: list[str] = list()
            for part in parts_to_check:
                if not file_info.stage:
                    for stage in EtlStage:
                        if part.lower() in stage.folder_names:
                            file_info.stage = stage
                            checked_parts.append(part)
                            break
                if not file_info.system_type:
                    for system_type in SystemType:
                        if part.lower() in system_type.aliases:
                            file_info.system_type = system_type
                            checked_parts.append(part)
                            break
                if not file_info.system_type:
                    raise ValueError("Unknown system type")

            system_candidates = [
                item for item in parts_to_check if item not in checked_parts
            ]
            if len(system_candidates) > 1:
                raise ValueError("Incorrect folder structure")

            if len(system_candidates) == 1:
                file_info.entity = system_candidates[0]
            file_extension = file.suffix[1:]
            for task_type in TaskType:
                if file_extension in task_type.extensions:
                    file_info.task_type = task_type
            if not file_info.task_type:
                raise ValueError("Unknown task type")

            self.subtasks.append(file_info)

    def get_tasks(
        self,
        etl_stage: str | None = None,
        entity: str | None = None,
        system_type: str | None = None,
        task_type: str | None = None,
        is_common: bool | None = False,
        include_common: bool = True,
    ) -> dict[str, Subtask]:
        input_etl_stage = EtlStage(etl_stage)
        input_system = SystemType(system_type)
        input_task_type = TaskType(task_type)

        filtered_list = [
            subtask
            for subtask in self.subtasks
            if (etl_stage is None or subtask.stage == input_etl_stage)
            and (entity is None or subtask.entity == entity)
            and (system_type is None or subtask.system_type == input_system)
            and (task_type is None or subtask.task_type == input_task_type)
            and (subtask.is_common == is_common)
        ]

        if include_common:
            filtered_list += [subtask for subtask in self.subtasks if subtask.is_common]

        result: dict[str, Subtask] = {}
        for item in filtered_list:
            result[item.path.stem] = item
        return result

    def get_task(self, name: str, entity: str | None = None) -> Subtask:
        for subtask in self.subtasks:
            if subtask.name == name:
                if entity:
                    if subtask.entity == entity:
                        return subtask
                else:
                    return subtask
        raise ValueError(f"Task with name {name} not found")

from pathlib import Path

from common.enums import EtlStage, SystemType, TaskType
from common.models import Subtask


class FileClassifier:
    """
    Classifies files into Subtask metadata (stage, system_type, entity, task_type).
    """

    def classify(self, base_path: Path, file: Path) -> Subtask:
        parts_to_check = file.parts[len(base_path.absolute().parts) : -1]

        subtask = Subtask(
            path=file,
            name=file.stem,
            stage=None,
            entity=None,
            system_type=None,
            task_type=None,
            is_common=False,
            command=None,  # filled later by FileLoader
        )

        if not parts_to_check:
            subtask.is_common = True
        if len(parts_to_check) > 3:
            raise ValueError("Incorrect folder structure")

        checked_parts: list[str] = []

        for part in parts_to_check:
            if not subtask.stage:
                for stage in EtlStage:
                    if part.lower() in stage.folder_names:
                        subtask.stage = stage
                        checked_parts.append(part)
                        break
            if not subtask.system_type:
                for system_type in SystemType:
                    if part.lower() in system_type.aliases:
                        subtask.system_type = system_type
                        checked_parts.append(part)
                        break

        system_candidates = [p for p in parts_to_check if p not in checked_parts]
        if len(system_candidates) > 1:
            raise ValueError("Incorrect folder structure")

        if system_candidates:
            subtask.entity = system_candidates[0]

        # Detect task type from extension
        ext = file.suffix.lstrip(".").lower()
        for task_type in TaskType:
            if ext in task_type.extensions:
                subtask.task_type = task_type
                break
        if not subtask.task_type:
            raise ValueError(f"Unknown task type for {file}")

        return subtask

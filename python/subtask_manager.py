# file_manager/subtask_manager.py
from pathlib import Path

from common.enums import EtlStage, SystemType, TaskType
from common.models import Subtask
from file_manager.file_classifier import FileClassifier
from file_manager.file_loader import FileLoader
from file_manager.file_scanner import FileScanner


class SubtaskManager:
    """
    High-level manager that discovers, classifies, and loads subtasks.
    """

    def __init__(self, base_path: str | Path):
        self.base_path: Path = Path(base_path)
        self.subtasks: list[Subtask] = []

        self.scanner: FileScanner = FileScanner(
            [f".{ext}" for t in TaskType for ext in t.extensions]
        )
        self.classifier: FileClassifier = FileClassifier()
        self.loader: FileLoader = FileLoader()

        self._discover_subtasks()

    def _discover_subtasks(self) -> None:
        """
        Scan base directory, classify each file, and load its content.
        """
        for file in self.scanner.scan_files(self.base_path):
            subtask = self.classifier.classify(self.base_path, file)
            subtask = self.loader.load(subtask)
            self.subtasks.append(subtask)

    def get_tasks(
        self,
        etl_stage: EtlStage | None = None,
        entity: str | None = None,
        system_type: SystemType | None = None,
        task_type: TaskType | None = None,
        is_common: bool | None = None,
        include_common: bool = True,
    ) -> dict[str, Subtask]:
        """
        Filter subtasks by provided criteria.

        Returns a dict keyed by unique task name. If duplicate names exist,
        later duplicates are suffixed with `#<n>` to avoid silent overwrites.
        """
        filtered = [
            s
            for s in self.subtasks
            if (etl_stage is None or s.stage == etl_stage)
            and (entity is None or s.entity == entity)
            and (system_type is None or s.system_type == system_type)
            and (task_type is None or s.task_type == task_type)
            and (is_common is None or s.is_common == is_common)
        ]

        # Deduplicate by absolute path while preserving order.
        seen_paths: set[Path] = {s.path.resolve() for s in filtered}
        if include_common:
            for s in self.subtasks:
                resolved = s.path.resolve()
                if s.is_common and resolved not in seen_paths:
                    filtered.append(s)
                    seen_paths.add(resolved)

        # Build a stable dict without key collisions by task stem.
        result: dict[str, Subtask] = {}
        key_counts: dict[str, int] = {}
        for s in filtered:
            base_key = s.path.stem
            if base_key not in result:
                result[base_key] = s
                key_counts[base_key] = 1
                continue

            key_counts[base_key] += 1
            candidate = f"{base_key}#{key_counts[base_key]}"
            while candidate in result:
                key_counts[base_key] += 1
                candidate = f"{base_key}#{key_counts[base_key]}"
            result[candidate] = s

        return result

    def get_task(self, name: str, entity: str | None = None) -> Subtask:
        """
        Get a single subtask by filename (optionally filtered by entity).
        """
        for s in self.subtasks:
            if s.name == name and (entity is None or s.entity == entity):
                return s
        raise ValueError(f"Task with name '{name}' not found")

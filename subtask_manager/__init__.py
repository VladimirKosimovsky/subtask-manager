from ._core import (
    EtlStage,
    FileClassifier,
    FileScanner,
    ParamType,
    RenderedSubtask,
    Subtask,
    SubtaskManager,
    SystemType,
    TaskType,
)


def main() -> int:
    """CLI entry point for the package."""
    print("subtask-manager: library package installed and ready to use.")
    return 0


__all__ = [
    "EtlStage",
    "FileClassifier",
    "FileScanner",
    "ParamType",
    "RenderedSubtask",
    "Subtask",
    "SubtaskManager",
    "SystemType",
    "TaskType",
    "main",
]

import warnings
from typing import Any

from ._core import (
    EtlStage,
    FileClassifier,
    FileScanner,
    ParamStyle,
    PreparedQuery,
    RenderedSubtask,
    SqlAnalysis,
    SqlGuard,
    SqlParamStyle,
    StatementKind,
    Subtask,
    SubtaskManager,
    SystemType,
    TaskType,
)

__version__ = "0.4.0"

_DEPRECATED_ALIASES: dict[str, tuple[str, Any]] = {
    # Renamed in 0.4.0: it describes the *style* of a placeholder, not the type
    # of the value. Kept until 0.5.0.
    "ParamType": ("ParamStyle", ParamStyle),
}


def __getattr__(name: str) -> Any:
    """Resolve deprecated aliases, warning once per call site."""
    if name in _DEPRECATED_ALIASES:
        new_name, value = _DEPRECATED_ALIASES[name]
        warnings.warn(
            f"{name} is deprecated and will be removed in 0.5.0; use {new_name} instead.",
            DeprecationWarning,
            stacklevel=2,
        )
        return value
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")


def __dir__() -> list[str]:
    return sorted([*__all__, *_DEPRECATED_ALIASES])


def main() -> int:
    """CLI entry point for the package."""
    print("subtask-manager: library package installed and ready to use.")
    return 0


__all__ = [
    "EtlStage",
    "FileClassifier",
    "FileScanner",
    "ParamStyle",
    "PreparedQuery",
    "RenderedSubtask",
    "SqlAnalysis",
    "SqlGuard",
    "SqlParamStyle",
    "StatementKind",
    "Subtask",
    "SubtaskManager",
    "SystemType",
    "TaskType",
    "__version__",
    "main",
]

# _core.pyi

class Subtask:
    """Represents a single subtask."""
    ...

class SubtaskManager:
    """Main class for scanning and managing subtasks."""

    def __init__(self) -> None: ...
    def scan_files(self, path: str) -> list[str]: ...
    # add more public methods here

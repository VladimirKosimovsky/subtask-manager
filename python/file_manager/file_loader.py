from pathlib import Path

from common.models import Subtask


class FileLoader:
    """
    Loads file contents into Subtask.command field.
    """

    def load(self, subtask: Subtask) -> Subtask:
        subtask.command = self._read_file(subtask.path)
        return subtask

    def _read_file(self, path: Path) -> str:
        with path.open("r", encoding="utf-8") as f:
            return f.read()

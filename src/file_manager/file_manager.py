# file_manager/file_scanner.py
from collections.abc import Iterator
from pathlib import Path


class FileScanner:
    def __init__(self, extensions: list[str]) -> None:
        """
        Initialize scanner with allowed extensions.
        Extensions should be provided with a leading dot, e.g. [".csv", ".txt"].
        """
        self.extensions:set[str] = {ext.lower() for ext in extensions}

    def scan_files(self, directory: str | Path) -> Iterator[Path]:
        """
        Scan directory for files matching given extensions in a single pass.

        Args:
            directory (str): Root directory path.

        Returns:
            Iterator[str]: Yields absolute paths of matching files.
        """
        dir_path = Path(directory)
        if not dir_path.is_dir():
            return iter([])

        for path in dir_path.rglob("*"):
            if path.is_file():
                if path.suffix.lower() in self.extensions:
                    yield path.resolve().absolute()

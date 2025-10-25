# tests/test_file_scanner.py
import os
from pathlib import Path

import pytest

from subtask_manager import FileScanner


@pytest.mark.parametrize("extensions,expected", [
    ([".txt"], ["a.txt", "b.txt"]),
    ([".csv"], ["c.csv"]),
    ([".txt", ".csv"], ["a.txt", "b.txt", "c.csv"]),
    ([".md"], []),
])
def test_scan_files(tmp_path: Path, extensions:list[str], expected:list[str]):
    # Arrange: create test files
    filenames = ["a.txt", "b.txt", "c.csv", "ignore.tmp"]
    for fname in filenames:
        _ = (tmp_path / fname).write_text("test")

    scanner = FileScanner(extensions)

    # Act
    result = scanner.scan_files(tmp_path)

    # Assert
    found_files = {os.path.basename(p) for p in result}
    assert found_files == set(expected)

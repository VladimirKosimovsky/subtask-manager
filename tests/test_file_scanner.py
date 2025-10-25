# tests/test_file_scanner.py
import os
from pathlib import Path
from unittest.mock import patch

import pytest

from python.file_manager.file_scanner import FileScanner


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
    result = scanner.scan_files(str(tmp_path))

    # Assert
    found_files = {os.path.basename(p) for p in result}
    assert found_files == set(expected)


def _is_file_side_effect(self: Path):
    return self.suffix.lower() in {".txt", ".csv"}


def _resolve_side_effect(self:Path):
    return self  # return the Path itself to avoid FS resolution


def test_scan_files_with_mock_side_effects():
    mock_files = [
        Path("/fake/a.txt"),
        Path("/fake/b.csv"),
        Path("/fake/c.tmp"),
        Path("/fake/subdir/d.txt"),
    ]

    scanner = FileScanner([".txt", ".csv"])

    with patch.object(Path, "rglob", return_value=mock_files), \
         patch.object(Path, "is_dir", return_value=True), \
         patch.object(Path, "is_file", new=_is_file_side_effect), \
         patch.object(Path, "resolve", new=_resolve_side_effect):
        result = list(scanner.scan_files("/fake"))

    assert set(result) == {
        Path("/fake/a.txt").resolve(),
        Path("/fake/b.csv").resolve(),
        Path("/fake/subdir/d.txt").resolve(),
    }
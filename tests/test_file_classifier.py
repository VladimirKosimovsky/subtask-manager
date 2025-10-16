from pathlib import Path

import pytest

from subtask_manager import EtlStage, SystemType, TaskType, Subtask
from common.models import Subtask
from file_manager.file_classifier import FileClassifier


def test_classify_simple_sql_file(tmp_path: Path):
    base = tmp_path
    entity_dir = base / "customers"
    stage_dir = entity_dir / "01_extract"
    sys_dir = stage_dir / "pg"
    sys_dir.mkdir(parents=True)

    file_path = sys_dir / "extract_customers.sql"
    _ = file_path.write_text("SELECT * FROM customers;")

    classifier = FileClassifier()
    subtask = classifier.classify(base, file_path)

    assert isinstance(subtask, Subtask)
    assert subtask.stage == EtlStage.EXTRACT
    assert subtask.system_type == SystemType.PG
    assert subtask.task_type == TaskType.SQL
    assert subtask.entity == "customers"
    assert subtask.is_common is False


def test_classify_common_file(tmp_path: Path):
    file_path = tmp_path / "utils.py"
    _ = file_path.write_text("print('common')")

    classifier = FileClassifier()
    subtask = classifier.classify(tmp_path, file_path)

    assert subtask.is_common is True
    assert subtask.task_type == TaskType.PYTHON


def test_classify_invalid_extension(tmp_path: Path):
    file_path = tmp_path / "weirdfile.unknown"
    _ = file_path.write_text("???")

    classifier = FileClassifier()

    with pytest.raises(ValueError, match="Unknown task type"):
        _ =  classifier.classify(tmp_path, file_path)

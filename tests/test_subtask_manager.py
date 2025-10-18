from pathlib import Path
from typing import cast
from unittest.mock import MagicMock, patch

import pytest

from subtask_manager import EtlStage, Subtask, SubtaskManager, SystemType, TaskType


# --------------------------
# Helpers
# --------------------------
def _create_etl_structure(base: Path):
    """Creates a simple ETL folder tree."""
    extract_pg = base / "customers" / "01_extract" / "pg"
    extract_pg.mkdir(parents=True)
    _ = (extract_pg / "get_customers.sql").write_text("SELECT * FROM customers;")

    transform_duck = base / "customers" / "02_transform" / "duck"
    transform_duck.mkdir(parents=True)
    _ = (transform_duck / "sales.py").write_text("print('transform')")

    common_file = base / "shared.yaml"
    _ = common_file.write_text("version: 1")

    return base


# --------------------------
# Test cases
# --------------------------

def test_discovery_and_classification(tmp_path: Path):
    base = _create_etl_structure(tmp_path)

    manager = SubtaskManager(base)

    assert len(manager.subtasks) == 3

    for subtask in manager.subtasks:
        _ = subtask.name
    
    sql_task = next(s for s in manager.subtasks if s.name == "get_customers.sql")
    assert sql_task.stage == EtlStage.Extract
    assert sql_task.system_type == SystemType.PostgreSQL
    assert sql_task.task_type == TaskType.Sql
    assert sql_task.entity == "customers"
    command = sql_task.command
    assert command is not None
    assert "SELECT" in command

    py_task = next(s for s in manager.subtasks if s.name == "sales")
    assert py_task.stage == EtlStage.Transform
    assert py_task.system_type == SystemType.Duckdb
    assert py_task.task_type == TaskType.Python
    command = py_task.command
    assert command is not None
    assert "print" in command

    yaml_task = next(s for s in manager.subtasks if s.name == "shared")
    assert yaml_task.is_common


def test_get_tasks_by_filters(tmp_path: Path):
    base = _create_etl_structure(tmp_path)
    manager = SubtaskManager(base)

    extract_pg = manager.get_tasks(
        etl_stage=EtlStage.Extract,
        system_type=SystemType.PostgreSQL,
    )
    assert len(extract_pg) == 2  # includes common file
    assert all(isinstance(v, Subtask) for v in extract_pg.values())


def test_get_tasks_without_common(tmp_path: Path):
    base = _create_etl_structure(tmp_path)
    manager = SubtaskManager(base)

    extract_pg = manager.get_tasks(
        etl_stage=EtlStage.EXTRACT,
        system_type=SystemType.PG,
        include_common=False,
    )
    assert len(extract_pg) == 1
    assert all(not s.is_common for s in extract_pg.values())


def test_get_task_by_name(tmp_path: Path):
    base = _create_etl_structure(tmp_path)
    manager = SubtaskManager(base)

    subtask = manager.get_task("get_customers")
    assert subtask.path.name == "get_customers.sql"

    with pytest.raises(ValueError):
        _ = manager.get_task("nonexistent.sql")


def test_invalid_folder_structure(tmp_path: Path):
    base = tmp_path / "01_extract" / "pg" / "deep" / "nested"
    base.mkdir(parents=True)
    f = base / "bad.sql"
    _ = f.write_text("SELECT 1;")

    # The classifier should raise ValueError due to deep nesting
    from file_manager.file_classifier import FileClassifier
    classifier = FileClassifier()
    with pytest.raises(ValueError):
        _ =classifier.classify(tmp_path, f)


# --------------------------
# Mock-based integration
# --------------------------

def test_integration_with_mocks(tmp_path: Path):
    base = tmp_path
    fake_file = Path("/fake/script.sh")

    fake_subtask = Subtask(
        name="script",
        path=fake_file,
        task_type=TaskType.SHELL,
        stage=EtlStage.LOAD,
        entity="dummy",
        is_common=False,
        command="echo ok",
    )

    with (
        patch("subtask_manager.subtask_manager.FileScanner") as MockScanner,
        patch("subtask_manager.subtask_manager.FileClassifier") as MockClassifier,
        patch("subtask_manager.subtask_manager.FileLoader") as MockLoader,
    ):
        mock_scanner = cast(MagicMock, MockScanner.return_value)
        mock_classifier = cast(MagicMock, MockClassifier.return_value)
        mock_loader = cast(MagicMock, MockLoader.return_value)

        mock_scanner.scan_files.return_value = [fake_file]
        mock_classifier.classify.return_value = fake_subtask
        mock_loader.load.return_value = fake_subtask

        manager = SubtaskManager(base)

        assert len(manager.subtasks) == 1
        assert manager.subtasks[0].name == "script"
        mock_scanner.scan_files.assert_called_once_with(base)
        mock_classifier.classify.assert_called_once()
        mock_loader.load.assert_called_once()


def test_existing_path():
    base = Path("tests/test_data/subtasks")
    
    sm:SubtaskManager = SubtaskManager(base)
    
    assert len(sm.subtasks) == 4
    
    extract_customers_task = sm.get_task("extract_data")
    assert extract_customers_task.name == "extract_data"
    assert extract_customers_task.entity == "customers"
    
    expected_content="\n".join([
    "select *",
    "from public.customers;"
    ])
    assert extract_customers_task.command == expected_content
    
    tasks = sm.get_tasks(is_common=True)
    assert len(tasks) == 1
    
    tasks = sm.get_tasks(etl_stage=EtlStage.EXTRACT, include_common=False)
    assert len(tasks) == 1
    
    
    tasks = sm.get_tasks(entity="customers")
    assert len(tasks) == 2
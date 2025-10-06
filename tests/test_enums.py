import pytest

from common.enums import EtlStage, SystemType, TaskType


# ------------------------
# TaskType tests
# ------------------------
def test_tasktype_properties():
    assert TaskType.SQL.id == 0
    assert "sql" in TaskType.SQL.extensions
    assert TaskType.PYTHON.id == 3
    assert "py" in TaskType.PYTHON.extensions


def test_tasktype_from_extension_valid():
    assert TaskType.from_extension("sql") is TaskType.SQL
    assert TaskType.from_extension("py") is TaskType.PYTHON
    assert TaskType.from_extension("GRAPHQL") is TaskType.GRAPHQL  # case-insensitive


def test_tasktype_from_extension_invalid():
    with pytest.raises(ValueError, match="Unknown task type"):
        _ = TaskType.from_extension("exe")


# ------------------------
# EtlStage tests
# ------------------------
def test_etlstage_properties():
    assert EtlStage.EXTRACT.id == 1
    assert "extract" in EtlStage.EXTRACT.folder_names


def test_etlstage_from_folder_name_valid():
    assert EtlStage.from_folder_name("extract") is EtlStage.EXTRACT
    assert EtlStage.from_folder_name("01") is EtlStage.EXTRACT
    assert EtlStage.from_folder_name("pp") is EtlStage.POST_PROCESSING


def test_etlstage_from_folder_name_invalid():
    with pytest.raises(ValueError, match="Unknown ETL stage alias: invalid_stage"):
        _ = EtlStage.from_folder_name("invalid_stage")


# ------------------------
# SystemType tests
# ------------------------
def test_systemtype_properties():
    assert SystemType.PG.id == 0
    assert "postgres" in SystemType.PG.aliases


def test_systemtype_from_alias_valid():
    assert SystemType.from_alias("pg") is SystemType.PG
    assert SystemType.from_alias("duckdb") is SystemType.DUCK


def test_systemtype_from_alias_invalid():
    with pytest.raises(ValueError, match="Unknown system type alias: mysql"):
        _ = SystemType.from_alias("mysql")

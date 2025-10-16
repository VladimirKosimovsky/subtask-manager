import pytest

from subtask_manager import EtlStage, SystemType, TaskType


# ------------------------
# TaskType tests
# ------------------------
def test_tasktype_properties():
    assert TaskType.Sql.id == 0
    assert "sql" in TaskType.Sql.extensions
    assert TaskType.Python.id == 3
    assert "py" in TaskType.Python.extensions


def test_tasktype_from_extension_valid():
    assert TaskType.from_extension("sql") == TaskType.Sql
    assert TaskType.from_extension("py") == TaskType.Python
    assert TaskType.from_extension("GRAPHQL") == TaskType.Graphql  # case-insensitive


def test_tasktype_from_extension_invalid():
    with pytest.raises(ValueError, match="Unknown task type"):
        _ = TaskType.from_extension("exe")



# ------------------------
# SystemType tests
# ------------------------
def test_systemtype_properties():
    assert SystemType.PostgreSQL.id == 4
    assert "postgres" in SystemType.PostgreSQL.aliases


def test_systemtype_from_alias_valid():
    assert SystemType.from_alias("pg") == SystemType.PostgreSQL
    assert SystemType.from_alias("duckdb") == SystemType.Duckdb


def test_systemtype_from_alias_invalid():
    with pytest.raises(ValueError, match="Unknown system type alias: blabla"):
        assert SystemType.from_alias("blabla") is SystemType.Other
        
# ------------------------
# EtlStage tests
# ------------------------
def test_etlstage_properties():
    assert EtlStage.Extract.id == 1
    assert "extract" in EtlStage.Extract.aliases


def test_etlstage_from_alias_valid():
    assert EtlStage.from_alias("extract") == EtlStage.Extract
    assert EtlStage.from_alias("01") == EtlStage.Extract
    assert EtlStage.from_alias("pp") == EtlStage.Postprocessing


def test_etlstage_from_folder_name_invalid():
    with pytest.raises(ValueError, match="Unknown ETL stage alias: invalid_stage"):
        _ = EtlStage.from_alias("invalid_stage")
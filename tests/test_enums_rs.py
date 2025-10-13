import pytest

from subtask_manager import EtlStage, SubtaskManager, SystemType


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
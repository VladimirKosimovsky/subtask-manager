"""SQL-injection guard: standalone checks and the automatic guard on SQL tasks."""

from pathlib import Path

import pytest

from subtask_manager import SqlGuard, Subtask, SubtaskManager, TaskType


def get_default_path() -> Path:
    return Path("tests/test_data/subtasks/params_example")


SAFE_VALUES = [
    "prod",
    "2025-01-01",
    "localhost",
    "5432",
    "analytics_db",
    "some plain text",
    "",
]

UNSAFE_VALUES = [
    "o'brien",
    "x'; DROP TABLE users; --",
    "1 OR 1=1",
    "1 UNION ALL SELECT password FROM users",
    "abc--def",
    "/*comment*/",
    'say "hi"',
    r"C:\data\file.csv",
    "a\x00b",
]


@pytest.mark.parametrize("value", SAFE_VALUES)
def test_safe_values_pass(value: str):
    assert SqlGuard.is_safe(value)
    assert SqlGuard.find_issues(value) == []
    SqlGuard.check(value)


@pytest.mark.parametrize("value", UNSAFE_VALUES)
def test_unsafe_values_are_rejected(value: str):
    assert not SqlGuard.is_safe(value)
    assert SqlGuard.find_issues(value)
    with pytest.raises(ValueError):
        SqlGuard.check(value)


def test_check_reports_the_parameter_name():
    with pytest.raises(ValueError, match="unsafe value for parameter 'user_name'"):
        SqlGuard.check("a'b", name="user_name")


def test_check_identifier_accepts_valid_names():
    assert SqlGuard.check_identifier("users") == "users"
    assert SqlGuard.check_identifier("public.users") == "public.users"
    assert SqlGuard.check_identifier("dwh.public.users") == "dwh.public.users"


@pytest.mark.parametrize(
    "value", ["users; DROP TABLE t", "1users", '"users"', "a.b.c.d", "users--"]
)
def test_check_identifier_rejects_invalid_names(value: str):
    with pytest.raises(ValueError, match="invalid SQL identifier"):
        SqlGuard.check_identifier(value)


def test_guard_is_on_by_default_for_sql_tasks():
    sm = SubtaskManager(get_default_path())
    subtask = sm.get_task("non_string_params0.sql")
    assert subtask.task_type == TaskType.Sql

    with pytest.raises(ValueError, match="unsafe value for parameter 'name'"):
        subtask.apply_parameters(
            {"name": "x'; DROP TABLE users;--", "id": 1, "is_active": True}
        )


def test_guard_can_be_disabled_per_call():
    sm = SubtaskManager(get_default_path())
    subtask = sm.get_task("non_string_params0.sql")

    applied = subtask.apply_parameters(
        {"name": "x'--", "id": 1, "is_active": True}, guard=False
    )
    assert "x'--" in applied.rendered_command


def test_guard_also_covers_render_with_params():
    sm = SubtaskManager(get_default_path())
    subtask = sm.get_task("non_string_params0.sql")

    with pytest.raises(ValueError):
        subtask.render_with_params(
            {"name": "1 OR 1=1", "id": 1, "is_active": True},
        )

    rendered = subtask.render_with_params(
        {"name": "1 OR 1=1", "id": 1, "is_active": True}, guard=False
    )
    assert "1 OR 1=1" in rendered.command


def test_guard_is_off_by_default_for_non_sql_tasks():
    subtask = Subtask(name="deploy.sh", command="echo '{msg}'")
    assert subtask.task_type != TaskType.Sql

    applied = subtask.apply_parameters({"msg": "a'; rm -rf /"})
    assert "a'; rm -rf /" in applied.rendered_command


def test_guard_can_be_forced_on_for_non_sql_tasks():
    subtask = Subtask(name="deploy.sh", command="echo '{msg}'")

    with pytest.raises(ValueError, match="unsafe value for parameter 'msg'"):
        subtask.apply_parameters({"msg": "a'; rm -rf /"}, guard=True)


def test_guard_ignores_params_absent_from_the_command():
    """A value only used in the path never reaches the SQL text."""
    subtask = Subtask(
        name="report.sql",
        path="out/{unused}/report.sql",
        task_type=TaskType.Sql,
        command="SELECT 1",
    )

    applied = subtask.apply_parameters({"unused": "x'; DROP TABLE t;--"})
    assert applied.path == "out/x'; DROP TABLE t;--/report.sql"

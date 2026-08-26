"""Binding parameters into a driver-ready (query, params) pair."""

from pathlib import Path

import pytest

from subtask_manager import ParamStyle, SqlParamStyle, Subtask, SubtaskManager


def get_default_path() -> Path:
    return Path("tests/test_data/subtasks/params_example")


def get_subtask_manager() -> SubtaskManager:
    return SubtaskManager(get_default_path())


def test_prepare_binds_quoted_and_bare_placeholders():
    subtask = get_subtask_manager().get_task("non_string_params0.sql")

    prepared = subtask.prepare({"name": "alice", "id": 7, "is_active": True})

    assert prepared.query == (
        "SELECT * FROM users WHERE name = ? AND id = ? and is_active = ?"
    )
    assert prepared.params == ["alice", 7, True]
    assert prepared.names == ["name", "id", "is_active"]


def test_prepare_preserves_python_types():
    """Values are handed to the driver untouched, not stringified."""
    subtask = Subtask(name="q.sql", command="SELECT {a}, {b}, {c}, {d}")

    prepared = subtask.prepare({"a": 1, "b": 2.5, "c": None, "d": False})

    assert prepared.params == [1, 2.5, None, False]
    assert [type(v) for v in prepared.params] == [int, float, type(None), bool]


def test_prepare_unpacks_into_query_and_params():
    subtask = get_subtask_manager().get_task("dollar0.sql")

    query, params = subtask.prepare({"user_id": 42})

    assert query == "SELECT * FROM users WHERE id = ?"
    assert params == [42]
    assert len(subtask.prepare({"user_id": 42})) == 2


def test_prepare_indexing_and_as_tuple():
    subtask = get_subtask_manager().get_task("dollar0.sql")
    prepared = subtask.prepare({"user_id": 42})

    assert prepared[0] == prepared.query
    assert prepared[1] == prepared.params
    assert prepared.as_tuple() == (prepared.query, prepared.params)
    with pytest.raises(IndexError):
        _ = prepared[2]


@pytest.mark.parametrize(
    ("param_style", "expected"),
    [
        (SqlParamStyle.Qmark, "SELECT * FROM users WHERE name = ? AND login = ?"),
        (SqlParamStyle.Numeric, "SELECT * FROM users WHERE name = $1 AND login = $2"),
        (
            SqlParamStyle.Named,
            "SELECT * FROM users WHERE name = :name AND login = :login",
        ),
        (SqlParamStyle.Format, "SELECT * FROM users WHERE name = %s AND login = %s"),
        (
            SqlParamStyle.Pyformat,
            "SELECT * FROM users WHERE name = %(name)s AND login = %(login)s",
        ),
    ],
)
def test_prepare_supports_every_driver_paramstyle(
    param_style: SqlParamStyle, expected: str
):
    subtask = get_subtask_manager().get_task("dollar_brace0.sql")

    prepared = subtask.prepare(
        {"name": "alice", "login": "al"}, param_style=param_style
    )

    assert prepared.query == expected


def test_named_styles_return_a_mapping():
    subtask = get_subtask_manager().get_task("dollar_brace0.sql")

    prepared = subtask.prepare(
        {"name": "alice", "login": "al"}, param_style=SqlParamStyle.Named
    )

    assert prepared.param_style.is_named
    assert prepared.params == {"name": "alice", "login": "al"}


def test_positional_styles_repeat_values_for_repeated_names():
    subtask = Subtask(name="q.sql", command="SELECT {a} WHERE x = {a}")

    prepared = subtask.prepare({"a": 3})
    assert prepared.query == "SELECT ? WHERE x = ?"
    assert prepared.params == [3, 3]

    named = subtask.prepare({"a": 3}, param_style=SqlParamStyle.Named)
    assert named.query == "SELECT :a WHERE x = :a"
    assert named.params == {"a": 3}


def test_prepare_handles_all_placeholder_styles():
    subtask = Subtask(
        name="q.sql",
        command="SELECT '{a}', $b, ${c}, {{d}}, __e__, %f%, <g>",
    )

    prepared = subtask.prepare(dict.fromkeys("abcdefg", 1))

    assert prepared.query == "SELECT ?, ?, ?, ?, ?, ?, ?"
    assert prepared.names == list("abcdefg")


def test_prepare_rejects_a_template_meant_for_interpolation():
    """all_styles0.sql embeds placeholders inside larger literals on purpose."""
    subtask = get_subtask_manager().get_task("all_styles0.sql")

    with pytest.raises(ValueError, match="quoted literal"):
        subtask.prepare({"env": "prod"}, ignore_missing=True)


def test_prepare_restricted_to_selected_styles():
    subtask = Subtask(name="q.sql", command="SELECT {a}, $b")

    prepared = subtask.prepare({"a": 1, "b": 2}, styles=[ParamStyle.Curly])

    assert prepared.query == "SELECT ?, $b"
    assert prepared.names == ["a"]


def test_prepare_rejects_partial_string_literals():
    subtask = Subtask(name="q.sql", command="SELECT * FROM t WHERE n LIKE '%{q}%'")

    with pytest.raises(ValueError, match="quoted literal"):
        subtask.prepare({"q": "abc"})


def test_prepare_inlines_validated_identifiers():
    subtask = Subtask(name="q.sql", command="SELECT * FROM {tbl} WHERE id = {id}")

    prepared = subtask.prepare({"tbl": "public.users", "id": 5}, identifiers=["tbl"])

    assert prepared.query == "SELECT * FROM public.users WHERE id = ?"
    assert prepared.params == [5]


def test_prepare_rejects_unsafe_identifiers():
    subtask = Subtask(name="q.sql", command="SELECT * FROM {tbl}")

    with pytest.raises(ValueError, match="invalid SQL identifier"):
        subtask.prepare({"tbl": "users; DROP TABLE t"}, identifiers=["tbl"])


def test_prepare_reports_missing_parameters():
    subtask = get_subtask_manager().get_task("dollar_brace0.sql")

    with pytest.raises(ValueError, match="Missing parameters for keys: login"):
        subtask.prepare({"name": "alice"})


def test_prepare_binds_none_for_missing_when_ignored():
    subtask = get_subtask_manager().get_task("dollar_brace0.sql")

    prepared = subtask.prepare({"name": "alice"}, ignore_missing=True)

    assert prepared.params == ["alice", None]


def test_prepare_without_command_fails():
    subtask = Subtask(name="q.sql")

    with pytest.raises(ValueError, match="no command to prepare"):
        subtask.prepare({})


def test_prepare_leaves_plain_sql_untouched():
    subtask = SubtaskManager("tests/test_data/subtasks").get_task(
        "attach_pg_to_duckdb.sql"
    )

    prepared = subtask.prepare()

    assert prepared.query == subtask.command
    assert prepared.params == []


def test_prepare_neutralises_an_injection_attempt():
    """The payload stays a value; it never becomes SQL text."""
    subtask = get_subtask_manager().get_task("dollar_brace0.sql")

    payload = "x'; DROP TABLE users; --"
    prepared = subtask.prepare({"name": payload, "login": "al"})

    assert prepared.query == "SELECT * FROM users WHERE name = ? AND login = ?"
    assert prepared.params == [payload, "al"]

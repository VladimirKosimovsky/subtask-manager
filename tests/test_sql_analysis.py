"""Dialect-aware analysis of what a task's SQL actually does."""

from pathlib import Path

import pytest

from subtask_manager import (
    StatementKind,
    Subtask,
    SubtaskManager,
    SystemType,
)


def pg(command: str, name: str = "task.sql") -> Subtask:
    return Subtask(name=name, system_type=SystemType.PostgreSQL, command=command)


def test_detects_statement_kind_and_tables():
    analysis = pg("SELECT * FROM public.users WHERE id = {id}").analyze()

    assert analysis.parsed
    assert analysis.dialect == "postgres"
    assert analysis.statements == [StatementKind.Select]
    assert analysis.tables == ["public.users"]
    assert not analysis.is_destructive
    assert analysis.warnings == []
    assert analysis.error is None


@pytest.mark.parametrize(
    ("command", "kind"),
    [
        ("DROP TABLE users", StatementKind.Drop),
        ("TRUNCATE TABLE users", StatementKind.Truncate),
        ("DELETE FROM users WHERE id = 1", StatementKind.Delete),
    ],
)
def test_destructive_statements_are_flagged(command: str, kind: StatementKind):
    analysis = pg(command).analyze()

    assert analysis.statements == [kind]
    assert analysis.has(kind)
    assert analysis.is_destructive
    assert kind.is_destructive


def test_non_destructive_statements():
    for command, kind in [
        ("SELECT 1", StatementKind.Select),
        ("INSERT INTO t (a) VALUES (1)", StatementKind.Insert),
        ("UPDATE t SET a = 1 WHERE id = 2", StatementKind.Update),
        ("CREATE TABLE t (a int)", StatementKind.Create),
    ]:
        analysis = pg(command).analyze()
        assert analysis.statements == [kind], command
        assert not analysis.is_destructive, command


def test_delete_without_where_is_warned_about():
    analysis = pg("DELETE FROM public.users").analyze()

    assert analysis.statements == [StatementKind.Delete]
    assert any("no WHERE clause" in w for w in analysis.warnings)


def test_update_without_where_is_warned_about():
    analysis = pg("UPDATE public.users SET active = false").analyze()

    assert any("rewrites every row" in w for w in analysis.warnings)


def test_qualified_delete_is_not_warned_about():
    analysis = pg("DELETE FROM public.users WHERE id = {id}").analyze()

    assert not any("no WHERE clause" in w for w in analysis.warnings)


def test_placeholders_are_named_in_the_report():
    """A table that comes from a placeholder is reported as that placeholder."""
    analysis = pg("DELETE FROM analytics.{tbl}").analyze()

    assert analysis.tables == ["analytics.{tbl}"]
    assert any("analytics.{tbl}" in w for w in analysis.warnings)


def test_placeholder_in_a_numeric_position_still_parses():
    analysis = pg("SELECT * FROM t LIMIT {n}").analyze()

    assert analysis.parsed
    assert analysis.tables == ["t"]


def test_stacked_statements_are_counted():
    analysis = pg("SELECT * FROM users; DROP TABLE users").analyze()

    assert analysis.statements == [StatementKind.Select, StatementKind.Drop]
    assert any("2 statements" in w for w in analysis.warnings)


def test_analysis_of_rendered_params_sees_injected_statements():
    task = pg("SELECT * FROM users WHERE name = '{name}'")

    assert task.analyze({"name": "alice"}).statements == [StatementKind.Select]
    assert task.analyze({"name": "x'; DROP TABLE users; --"}).statements == [
        StatementKind.Select,
        StatementKind.Drop,
    ]


def test_dialect_follows_system_type():
    assert pg("SELECT 1").analyze().dialect == "postgres"
    for system_type, dialect in [
        (SystemType.Duckdb, "duckdb"),
        (SystemType.Clickhouse, "clickhouse"),
        (SystemType.MySQL, "mysql"),
        (SystemType.SqlServer, "mssql"),
        (SystemType.OracleDB, "oracle"),
        (SystemType.SQLite, "sqlite"),
        # No Vertica dialect exists upstream; generic is the closest fit.
        (SystemType.Vertica, "generic"),
        (SystemType.Other, "generic"),
    ]:
        task = Subtask(name="t.sql", system_type=system_type, command="SELECT 1")
        assert task.analyze().dialect == dialect, system_type

    # No system detected at all also falls back to generic.
    assert Subtask(name="t.sql", command="SELECT 1").analyze().dialect == "generic"


def test_unparseable_sql_fails_open():
    """Vendor syntax no dialect knows must not be reported as anything."""
    task = SubtaskManager(Path("tests/test_data/subtasks")).get_task(
        "attach_pg_to_duckdb_with_params.sql"
    )

    analysis = task.analyze()

    assert not analysis.parsed
    assert analysis.error is not None
    assert analysis.statements == []
    assert analysis.tables == []
    # "not analysed" must never read as "safe"
    assert not analysis.is_destructive


def test_analysis_never_raises_on_garbage():
    analysis = pg("this is not sql at all").analyze()

    assert not analysis.parsed
    assert "sql parser error" in analysis.error


def test_subtask_without_command_is_not_analysed():
    analysis = Subtask(name="t.sql").analyze()

    assert not analysis.parsed


def test_statement_kind_metadata():
    assert StatementKind.Drop.name == "drop"
    assert StatementKind.Drop.id == 6
    assert StatementKind.Drop.is_destructive
    assert not StatementKind.Select.is_destructive
    assert StatementKind.from_alias("DELETE") == StatementKind.Delete
    assert StatementKind.from_alias("upsert") == StatementKind.Merge
    assert str(StatementKind.Truncate) == "truncate"
    assert repr(StatementKind.Truncate) == "StatementKind.TRUNCATE"
    with pytest.raises(ValueError, match="Unknown StatementKind alias"):
        StatementKind.from_alias("nonesuch")


# --------------------------------------------------------------- forbid ----


def test_forbid_blocks_a_matching_statement():
    task = pg("DELETE FROM {tbl}")

    with pytest.raises(ValueError, match="runs forbidden statement"):
        task.apply_parameters({"tbl": "users"}, forbid=[StatementKind.Delete])


def test_forbid_allows_everything_else():
    task = pg("SELECT * FROM {tbl}")

    applied = task.apply_parameters(
        {"tbl": "users"}, forbid=[StatementKind.Drop, StatementKind.Truncate]
    )
    assert applied.rendered_command == "SELECT * FROM users"


def test_forbid_catches_a_statement_added_by_an_injected_value():
    task = pg("SELECT * FROM users WHERE name = '{name}'")

    with pytest.raises(ValueError, match="drop"):
        task.apply_parameters(
            {"name": "x'; DROP TABLE users; --"},
            guard=False,
            forbid=[StatementKind.Drop],
        )


def test_forbid_works_on_render_with_params():
    task = pg("TRUNCATE TABLE {tbl}")

    with pytest.raises(ValueError, match="truncate"):
        task.render_with_params({"tbl": "events"}, forbid=[StatementKind.Truncate])


def test_forbid_works_on_prepare():
    task = pg("DELETE FROM users WHERE id = {id}")

    with pytest.raises(ValueError, match="delete"):
        task.prepare({"id": 1}, forbid=[StatementKind.Delete])

    assert task.prepare({"id": 1}, forbid=[StatementKind.Drop]).params == [1]


def test_forbid_fails_open_on_unparseable_sql():
    """SQL that cannot be judged is allowed through rather than blocked."""
    task = SubtaskManager(Path("tests/test_data/subtasks")).get_task(
        "attach_pg_to_duckdb_with_params.sql"
    )

    applied = task.apply_parameters(
        {
            "db_name": "dwh",
            "host": "localhost",
            "port": "5432",
            "user": "pg",
            "password": "secret",
        },
        forbid=[StatementKind.Drop, StatementKind.Delete],
    )
    assert "dwh" in applied.rendered_command


def test_empty_forbid_list_is_a_no_op():
    task = pg("DROP TABLE {tbl}")

    applied = task.apply_parameters({"tbl": "t"}, forbid=[])
    assert applied.rendered_command == "DROP TABLE t"

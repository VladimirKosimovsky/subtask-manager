from pathlib import Path

from subtask_manager import (
    EtlStage,
    FileClassifier,
    FileScanner,
    ParamStyle,
    SqlGuard,
    SqlParamStyle,
    Subtask,
    SubtaskManager,
    SystemType,
    TaskType,
)

print(ParamStyle.DollarBrace.aliases)
sm: SubtaskManager = SubtaskManager(
    base_path="tests/test_data/subtasks",
)

subtask: Subtask = sm.get_task("attach_pg_to_duckdb_with_params.sql")
print(subtask.get_params())
_ = subtask.apply_parameters(
    {
        "db_name": "dwh",
        "host": "localhost",
        "port": "5432",
        "user": "postgres",
        "password": "password",
    }
)

print(subtask.render().command)
print(subtask.get_stored_params())

for subtask in sm.subtasks:
    print(subtask.entity)

print(EtlStage.Postprocessing.aliases)

print(SystemType.PostgreSQL.aliases)
print(SystemType.PostgreSQL.id)
print(EtlStage.Cleanup.id)

print(SystemType.from_alias("pg") == SystemType.PostgreSQL)
print(type(SystemType.from_alias("pg")))
print(type(SystemType.PostgreSQL))

print(TaskType.Graphql.extensions)

fs = FileScanner(["py"])
print(fs.extensions)


# Using string path
manager1 = SubtaskManager("tests/test_data/subtasks")
print(manager1.base_path)

# Using pathlib.Path
manager2 = SubtaskManager(Path("tests/test_data/subtasks"))
print(manager2.base_path)

fcs = FileClassifier(Path("tests/test_data/subtasks"))
print(fcs.base_path)


# --- SQL parameter binding -------------------------------------------------
params_sm = SubtaskManager("tests/test_data/subtasks/params_example")
query_task: Subtask = params_sm.get_task("non_string_params0.sql")
print(query_task.command)

query, values = query_task.prepare({"name": "alice", "id": 7, "is_active": True})
print(query, values)

print(
    query_task.prepare(
        {"name": "alice", "id": 7, "is_active": True},
        param_style=SqlParamStyle.Numeric,
    ).query
)

# --- SQL injection guard ---------------------------------------------------
print(SqlGuard.is_safe("prod"), SqlGuard.find_issues("1 OR 1=1"))

try:
    _ = query_task.apply_parameters(
        {"name": "x'; DROP TABLE users;--", "id": 1, "is_active": True}
    )
except ValueError as exc:
    print(f"guard rejected the value: {exc}")

# Binding accepts the very same payload: it stays a value, never SQL.
print(
    query_task.prepare(
        {"name": "x'; DROP TABLE users;--", "id": 1, "is_active": True}
    ).as_tuple()
)

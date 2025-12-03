from pathlib import Path

from subtask_manager import (
    EtlStage,
    FileClassifier,
    FileScanner,
    Subtask,
    SubtaskManager,
    SystemType,
    TaskType,
)

sm: SubtaskManager = SubtaskManager(
    base_path="tests/test_data/subtasks",
)

subtask:Subtask = sm.get_task("attach_pg_to_duckdb_with_params.sql")
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

print(subtask.command)



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

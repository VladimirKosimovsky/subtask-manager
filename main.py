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

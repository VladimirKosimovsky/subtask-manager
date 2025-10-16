from subtask_manager import EtlStage, FileScanner, SubtaskManager, SystemType, TaskType

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

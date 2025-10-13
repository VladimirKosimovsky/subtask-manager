from subtask_manager import EtlStage, SubtaskManager, SystemType

sm:SubtaskManager = SubtaskManager(
    base_path='tests/test_data/subtasks',
)
for subtask in sm.subtasks:
    print(subtask.entity)

print(EtlStage.Postprocessing.aliases)

print(SystemType.PostgreSQL.aliases)

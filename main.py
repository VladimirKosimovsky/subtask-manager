from subtask_manager import SubtaskManager

sm:SubtaskManager = SubtaskManager(
    base_path='tests/test_data/subtasks',
)
for subtask in sm.subtasks:
    print(subtask.entity)

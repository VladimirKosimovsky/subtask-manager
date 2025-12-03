from pathlib import Path

from subtask_manager import Subtask, SubtaskManager


def get_default_path():
    return Path("tests/test_data/subtasks/params_example")

def get_subtask_manager(base: Path) -> SubtaskManager:
    return SubtaskManager(base)

def test_existing_path():
    sm:SubtaskManager = SubtaskManager(get_default_path())
    assert len(sm.subtasks) > 0
    
def test_params_simple():
    sm:SubtaskManager = SubtaskManager(get_default_path())
    test_cases = [
        ("curvy0.sql",{"db_name": "test_db"}, "ATTACH if not exists '' AS test_db (TYPE POSTGRES, SECRET test_db_secret);"),
        ("dollar0.sql", {"user_id": "1"}, "SELECT * FROM users WHERE id = 1"),
        ("dollar_brace0.sql", {"name": "John", "login": "john"}, "SELECT * FROM users WHERE name = John AND login = john"),
        ("double_underscore0.sql", {"name": "John", "login": "john"}, "SELECT * FROM users WHERE name = John AND login = john"),
        ("percent0.sql", {"name": "John", "login": "john"}, "SELECT * FROM users WHERE name = John AND login = john"),
        ("angle0.sql", {"name": "John", "login": "john"}, "SELECT * FROM users WHERE name = John AND login = john"),
    ]
    for test_case in test_cases:
        subtask:Subtask = sm.get_task(test_case[0])
        params = test_case[1]
        expected_command = test_case[2]
    
        subtask.apply_parameters(params)
        print(subtask.command)
        print(expected_command)
        assert subtask.command == expected_command
    
    
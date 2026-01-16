from pathlib import Path

from subtask_manager import RenderedSubtask, Subtask, SubtaskManager


def get_default_path():
    return Path("tests/test_data/subtasks/params_example")


def test_render_with_params_lightweight():
    """Test the lightweight render_with_params that returns RenderedSubtask."""
    sm: SubtaskManager = SubtaskManager(get_default_path())

    test_cases = [
        (
            "curly0.sql",
            {"db_name": "test_db"},
            "ATTACH if not exists '' AS test_db (TYPE POSTGRES, SECRET test_db_secret);",
        ),
        ("dollar0.sql", {"user_id": "1"}, "SELECT * FROM users WHERE id = 1"),
        (
            "dollar_brace0.sql",
            {"name": "John", "login": "john"},
            "SELECT * FROM users WHERE name = 'John' AND login = 'john'",
        ),
    ]

    for test_case in test_cases:
        subtask: Subtask = sm.get_task(test_case[0])
        params = test_case[1]
        expected_command = test_case[2]

        # Use lightweight render
        rendered: RenderedSubtask = subtask.render_with_params(params)
        
        assert rendered.command == expected_command
        assert rendered.name == test_case[0]
        # Verify params were stored
        for key, value in params.items():
            assert rendered.params[key] == str(value)


def test_get_command_helper():
    """Test the get_command() helper that works with or without parameters."""
    sm: SubtaskManager = SubtaskManager(get_default_path())

    # Task without parameters applied
    subtask: Subtask = sm.get_task("dollar0.sql")
    cmd = subtask.get_command()
    assert cmd is not None
    assert "$user_id" in cmd  # Template version

    # Task with parameters applied
    applied = subtask.apply_parameters({"user_id": "42"})
    cmd = applied.get_command()
    assert cmd is not None
    assert "42" in cmd  # Rendered version
    assert "$user_id" not in cmd


def test_render_lightweight():
    """Test render_lightweight for tasks without parameters."""
    sm: SubtaskManager = SubtaskManager(get_default_path())
    
    # Get any task
    subtask: Subtask = sm.get_task("dollar0.sql")
    
    # Render without params
    rendered: RenderedSubtask = subtask.render_lightweight()
    
    assert rendered.name == "dollar0.sql"
    assert rendered.command is not None
    assert len(rendered.params) == 0  # No params provided


def test_immutability():
    """Verify that apply_parameters doesn't modify the original Subtask."""
    sm: SubtaskManager = SubtaskManager(get_default_path())
    
    original: Subtask = sm.get_task("dollar0.sql")
    original_path = original.path
    original_cmd = original.command
    
    # Apply parameters - should return new instance
    applied = original.apply_parameters({"user_id": "999"})
    
    # Original should be unchanged
    assert original.path == original_path
    assert original.command == original_cmd
    assert original.rendered_command is None
    
    # Applied should have rendered values
    assert applied.rendered_command is not None
    assert "999" in applied.rendered_command
    
    # Can apply different parameters to same original
    applied2 = original.apply_parameters({"user_id": "123"})
    assert applied2.rendered_command is not None
    assert "123" in applied2.rendered_command
    assert "999" in applied.rendered_command  # First one unchanged

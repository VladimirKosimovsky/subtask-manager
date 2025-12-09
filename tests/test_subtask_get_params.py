from pathlib import Path

from subtask_manager import ParamType, Subtask, SubtaskManager


def get_default_path():
    return Path("tests/test_data/subtasks/params_example")


def get_subtask_manager(base: Path) -> SubtaskManager:
    return SubtaskManager(base)


def test_existing_path():
    sm: SubtaskManager = SubtaskManager(get_default_path())
    assert len(sm.subtasks) > 0


def test_get_params_simple():
    sm: SubtaskManager = SubtaskManager(get_default_path())
    test_cases = [
        (
            "curly0.sql",
            None,
            set(["db_name"]),
        ),
        ("non_string_params0.sql", None, set(["id", "name", "is_active"])),
    ]
    for test_case in test_cases:
        subtask: Subtask = sm.get_task(test_case[0])
        expected_params = test_case[2]

        parsed_params: set = subtask.get_params(styles=test_case[1])
        assert expected_params == parsed_params


def test_get_params_filtered_styles():
    sm: SubtaskManager = SubtaskManager(get_default_path())

    test_cases = [
        (
            # подменяем параметры только двух типов {{в двойных скобках}}, ${доллар-скобки}
            "all_styles0.sql",
            [ParamType.DoubleCurly, ParamType.DollarBrace],
            set(["db", "env"]),
        ),
        (
            "all_styles0.sql",
            
            [
                ParamType.Curly,
                ParamType.DollarBrace,
                ParamType.DoubleCurly,
            ],
             set(["uuid", "env", "db"]),
        ),
        (
            "all_styles0.sql",
            [
                ParamType.Curly,
                ParamType.Dollar,
                ParamType.DollarBrace,
                ParamType.DoubleCurly,
                ParamType.DoubleUnderscore,
                ParamType.Percent,
                ParamType.Angle,
            ],
            set(["uuid", "env", "db", "host", "user", "id", "table"]),
        ),
    ]
    for test_case in test_cases:
        subtask: Subtask = sm.get_task(test_case[0])
        expected_params = test_case[2]

        parsed_params: set = subtask.get_params(styles=test_case[1])
        assert expected_params == parsed_params


def test_get_stored_params():
    sm: SubtaskManager = SubtaskManager(get_default_path())

    test_cases = [
        (
            "non_string_params0.sql",
            {"name": "Alice", "id": '10'},
            [ParamType.Curly],
        ),
        (
            "non_string_params0.sql",
            {"name": "Bob", "id": '20', "is_active": 'True'},
            [ParamType.Curly, ParamType.DoubleUnderscore],
        ),
        (
            "non_string_params1.sql",
            {"activated_at": "2022-01-01", "balance": '1000.123456'},
            [ParamType.Curly, ParamType.DoubleUnderscore],
        ),
    ]

    for test_case in test_cases:
        subtask: Subtask = sm.get_task(test_case[0])
        params = test_case[1]
        subtask.apply_parameters(params=params, styles=test_case[2])
        expected_params = subtask.get_stored_params()

        assert params == expected_params

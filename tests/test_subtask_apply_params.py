from pathlib import Path

from subtask_manager import ParamType, Subtask, SubtaskManager


def get_default_path():
    return Path("tests/test_data/subtasks/params_example")


def get_subtask_manager(base: Path) -> SubtaskManager:
    return SubtaskManager(base)


def test_existing_path():
    sm: SubtaskManager = SubtaskManager(get_default_path())
    assert len(sm.subtasks) > 0


def test_params_simple():
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
        (
            "double_underscore0.sql",
            {"name": "Fred", "login": "fred"},
            "SELECT * FROM users WHERE name = 'Fred' AND login = 'fred'",
        ),
        (
            "percent0.sql",
            {"name": "George", "login": "george"},
            "SELECT * FROM users WHERE name = 'George' AND login = 'george'",
        ),
        (
            "angle0.sql",
            {"name": "Kit", "login": "kit"},
            "SELECT * FROM users WHERE name = 'Kit' AND login = 'kit'",
        ),
        (
            "double_curly0.sql",
            {"user.name": "Alice", "user.login": "alice"},
            "SELECT * FROM users WHERE name = 'Alice' AND login = 'alice'",
        ),
    ]
    for test_case in test_cases:
        subtask: Subtask = sm.get_task(test_case[0])
        params = test_case[1]
        expected_command = test_case[2]

        subtask.apply_parameters(params)
        print(subtask.command)
        print(expected_command)
        assert subtask.command == expected_command


def test_params_filtered_styles():
    sm: SubtaskManager = SubtaskManager(get_default_path())
    "select '{uuid}' as uuid, 'report_{{env}}.sql' as src, 'psql -h $host -U $user -d ${db}' as cmd, '__USER__' as user, <id> as id from %table%"

    test_cases = [
        (
            # подменяем параметры только двух типов {{в двойных скобках}}, ${доллар-скобки}
            "all_styles0.sql",
            {"env": "dev", "db": "test_db"},
            [ParamType.DoubleCurly, ParamType.DollarBrace],
            "select '{uuid}' as uuid, 'report_dev.sql' as src, 'psql -h $host -U $user -d test_db' as cmd, '__USER__' as user, <id> as id from %table%",
        ),
        (
            "all_styles0.sql",
            {
                "uuid": "51492cf0-a5b1-4b5d-8665-98cfb5858660",
                "env": "dev",
                "db": "test_db",
            },
            [
                ParamType.Curly,
                ParamType.DollarBrace,
                ParamType.DoubleCurly,
            ],
            "select '51492cf0-a5b1-4b5d-8665-98cfb5858660' as uuid, 'report_dev.sql' as src, 'psql -h $host -U $user -d test_db' as cmd, '__USER__' as user, <id> as id from %table%",
        ),
        (
            "all_styles0.sql",
            {
                "uuid": "51492cf0-a5b1-4b5d-8665-98cfb5858660",
                "env": "dev",
                "host": "localhost",
                "user": "postgres",
                "db": "test_db",
                "id": "10",
                "table": "users",
            },
            [
                ParamType.Curly,
                ParamType.Dollar,
                ParamType.DollarBrace,
                ParamType.DoubleCurly,
                ParamType.DoubleUnderscore,
                ParamType.Percent,
                ParamType.Angle,
            ],
            "select '51492cf0-a5b1-4b5d-8665-98cfb5858660' as uuid, 'report_dev.sql' as src, 'psql -h localhost -U postgres -d test_db' as cmd, 'postgres' as user, 10 as id from users",
        ),
    ]
    for test_case in test_cases:
        subtask: Subtask = sm.get_task(test_case[0])
        params = test_case[1]
        expected_command = test_case[3]

        subtask.apply_parameters(params=params, styles=test_case[2])
        print(subtask.command)
        print(expected_command)
        assert subtask.command == expected_command

"""ParamType was renamed to ParamStyle in 0.4.0; the old name still resolves."""

import warnings

import pytest

import subtask_manager
from subtask_manager import ParamStyle


def test_param_type_alias_warns_and_resolves():
    with pytest.warns(DeprecationWarning, match="use ParamStyle instead"):
        alias = subtask_manager.ParamType

    assert alias is ParamStyle


def test_param_style_itself_does_not_warn():
    with warnings.catch_warnings():
        warnings.simplefilter("error")
        assert subtask_manager.ParamStyle is ParamStyle


def test_param_type_is_not_advertised():
    assert "ParamType" not in subtask_manager.__all__
    assert "ParamStyle" in subtask_manager.__all__
    assert "ParamType" in dir(subtask_manager)


def test_unknown_attribute_still_raises():
    with pytest.raises(AttributeError, match="has no attribute 'Nope'"):
        _ = subtask_manager.Nope

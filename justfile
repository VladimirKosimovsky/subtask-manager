# subtask-manager task runner — https://just.systems
#
# `just` with no arguments lists every recipe.

set shell := ["bash", "-euo", "pipefail", "-c"]

_default:
    @just --list --unsorted

# ---------------------------------------------------------------- Quality ----

# Run the Rust and Python test suites
test: test-rust test-py

# Run the Rust unit tests
test-rust:
    cargo test

# Run the Python tests against a freshly built extension
test-py: develop
    uv run -m pytest

# Lint Python (ruff) and Rust (clippy)
lint:
    uv run ruff check .
    cargo clippy --all-targets -- -D warnings

# Format Python and Rust sources in place
format:
    uv run ruff format .
    cargo fmt

# Verify formatting without writing (what CI should run)
format-check:
    uv run ruff format --check .
    cargo fmt --check

# Lint + format check + tests
check: lint format-check test

# ------------------------------------------------------------------ Build ----

# Build the extension into the local venv (editable dev install)
develop:
    uv run maturin develop --quiet

# Build a debug wheel
build:
    uv run maturin build

# Build a release wheel
build-release:
    uv run maturin build --release

# Remove Rust build artifacts and Python caches
clean:
    cargo clean
    rm -rf .pytest_cache .ruff_cache
    find . -name __pycache__ -type d -prune -exec rm -rf {} +

# Run the example script against the test data
demo: develop
    uv run python main.py

# ------------------------------------------------------------- Versioning ----

# Show the version recorded in Cargo.toml and pyproject.toml
show-version:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo_v=$(grep -m1 '^version = "' Cargo.toml | cut -d'"' -f2)
    py_v=$(grep -m1 '^version = "' pyproject.toml | cut -d'"' -f2)
    init_v=$(grep -m1 '^__version__ = "' subtask_manager/__init__.py | cut -d'"' -f2)
    echo "Cargo.toml                : $cargo_v"
    echo "pyproject.toml            : $py_v"
    echo "subtask_manager/__init__  : $init_v"
    if [ "$cargo_v" != "$py_v" ] || [ "$cargo_v" != "$init_v" ]; then
        echo "WARNING: versions are out of sync" >&2
        exit 1
    fi

# Set an exact version everywhere: just bump 0.4.1
bump version:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! [[ "{{ version }}" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][A-Za-z0-9.]+)?$ ]]; then
        echo "ERROR: invalid version '{{ version }}'. Expected X.Y.Z" >&2
        exit 1
    fi
    sed -i -E '0,/^version = "[^"]+"/s//version = "{{ version }}"/' Cargo.toml
    sed -i -E '0,/^version = "[^"]+"/s//version = "{{ version }}"/' pyproject.toml
    sed -i -E '0,/^__version__ = "[^"]+"/s//__version__ = "{{ version }}"/' subtask_manager/__init__.py
    cargo update -p subtask_manager --quiet 2>/dev/null || true
    echo "Updated Cargo.toml, pyproject.toml and __init__.py -> {{ version }}"
    just show-version

# Increase the patch version (0.4.0 -> 0.4.1)
bump-patch:
    #!/usr/bin/env bash
    set -euo pipefail
    v=$(grep -m1 '^version = "' pyproject.toml | cut -d'"' -f2)
    just bump "$(echo "$v" | awk -F. '{print $1"."$2"."$3+1}')"

# Increase the minor version (0.4.1 -> 0.5.0)
bump-minor:
    #!/usr/bin/env bash
    set -euo pipefail
    v=$(grep -m1 '^version = "' pyproject.toml | cut -d'"' -f2)
    just bump "$(echo "$v" | awk -F. '{print $1"."$2+1".0"}')"

# Increase the major version (0.5.0 -> 1.0.0)
bump-major:
    #!/usr/bin/env bash
    set -euo pipefail
    v=$(grep -m1 '^version = "' pyproject.toml | cut -d'"' -f2)
    just bump "$(echo "$v" | awk -F. '{print $1+1".0.0"}')"

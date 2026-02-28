SHELL := /bin/sh

.PHONY: help test test-rust test-py lint format check build build-release clean \
	show-version bump bump-patch bump-minor bump-major

help:
	@echo "subtask-manager Makefile targets"
	@echo ""
	@echo "Quality:"
	@echo "  make test           Run Rust and Python tests"
	@echo "  make test-rust      Run cargo test"
	@echo "  make test-py        Run pytest via uv"
	@echo "  make lint           Run Ruff lint checks + cargo check"
	@echo "  make format         Format Python and Rust code"
	@echo "  make check          Run lint + tests"
	@echo ""
	@echo "Build:"
	@echo "  make build          Build Python extension wheel (debug)"
	@echo "  make build-release  Build Python extension wheel (release)"
	@echo "  make clean          Clean Rust build artifacts"
	@echo ""
	@echo "Versioning:"
	@echo "  make show-version                 Show versions in Cargo.toml and pyproject.toml"
	@echo "  make bump VERSION=X.Y.Z          Set exact version in both files"
	@echo "  make bump-patch                  Increase patch version"
	@echo "  make bump-minor                  Increase minor version, reset patch"
	@echo "  make bump-major                  Increase major version, reset minor/patch"

test: test-rust test-py

test-rust:
	cargo test

test-py:
	uv run -m pytest

lint:
	uv run ruff check .
	cargo check

format:
	uv run ruff format .
	cargo fmt

check: lint test

build:
	maturin build

build-release:
	maturin build --release

clean:
	cargo clean

show-version:
	@CARGO_V=$$(grep -m1 '^version = "' Cargo.toml | cut -d'"' -f2); \
	PY_V=$$(grep -m1 '^version = "' pyproject.toml | cut -d'"' -f2); \
	echo "Cargo.toml      : $$CARGO_V"; \
	echo "pyproject.toml  : $$PY_V"; \
	if [ "$$CARGO_V" != "$$PY_V" ]; then \
		echo "WARNING: versions differ between Cargo.toml and pyproject.toml" >&2; \
	fi

# Usage:
#   make bump VERSION=0.2.3
bump:
	@if [ -z "$(VERSION)" ]; then \
		echo "ERROR: VERSION is required. Example: make bump VERSION=0.2.3"; \
		exit 1; \
	fi
	@case "$(VERSION)" in \
		[0-9]*.[0-9]*.[0-9]*) ;; \
		*) echo "ERROR: invalid VERSION '$(VERSION)'. Expected X.Y.Z"; exit 1 ;; \
	esac
	@sed -i -E '0,/^version = "[^"]+"/s//version = "$(VERSION)"/' Cargo.toml
	@sed -i -E '0,/^version = "[^"]+"/s//version = "$(VERSION)"/' pyproject.toml
	@echo "Updated Cargo.toml and pyproject.toml -> $(VERSION)"
	@$(MAKE) --no-print-directory show-version

bump-patch:
	@V=$$(grep -m1 '^version = "' pyproject.toml | cut -d'"' -f2); \
	MAJOR=$$(echo "$$V" | cut -d. -f1); \
	MINOR=$$(echo "$$V" | cut -d. -f2); \
	PATCH=$$(echo "$$V" | cut -d. -f3); \
	NEXT="$$MAJOR.$$MINOR.$$((PATCH + 1))"; \
	echo "Bumping patch -> $$NEXT"; \
	$(MAKE) --no-print-directory bump VERSION="$$NEXT"

bump-minor:
	@V=$$(grep -m1 '^version = "' pyproject.toml | cut -d'"' -f2); \
	MAJOR=$$(echo "$$V" | cut -d. -f1); \
	MINOR=$$(echo "$$V" | cut -d. -f2); \
	NEXT="$$MAJOR.$$((MINOR + 1)).0"; \
	echo "Bumping minor -> $$NEXT"; \
	$(MAKE) --no-print-directory bump VERSION="$$NEXT"

bump-major:
	@V=$$(grep -m1 '^version = "' pyproject.toml | cut -d'"' -f2); \
	MAJOR=$$(echo "$$V" | cut -d. -f1); \
	NEXT="$$((MAJOR + 1)).0.0"; \
	echo "Bumping major -> $$NEXT"; \
	$(MAKE) --no-print-directory bump VERSION="$$NEXT"

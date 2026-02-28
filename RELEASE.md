# Release Runbook

This project publishes platform-specific wheels (Linux + Windows) and an sdist using GitHub Actions with Trusted Publishing.

## Workflows

- **TestPyPI release workflow**: `.github/workflows/release-testpypi.yml`
  - Triggered by tags: `test-v*`
  - Also supports manual trigger
  - Publishes to TestPyPI

- **PyPI release workflow**: `.github/workflows/release.yml`
  - Triggered by tags: `v*`
  - Also supports manual trigger
  - Publishes to PyPI

---

## One-time setup checklist

## 1) GitHub environments

Create these repository environments:

- `testpypi`
- `pypi`

Optional but recommended:
- add required reviewers for manual approval on production (`pypi`)

## 2) Trusted Publishing on package indexes

Set up trusted publishers for **both** indexes.

### TestPyPI trusted publisher
- Repository: `subtask-manager`
- Workflow file: `release-testpypi.yml`
- Environment: `testpypi`

### PyPI trusted publisher
- Repository: `subtask-manager`
- Workflow file: `release.yml`
- Environment: `pypi`

---

## Versioning policy

- Use **test tags** for dry runs:
  - `test-vX.Y.Z` (example: `test-v0.2.2`)
- Use **production tags** for real releases:
  - `vX.Y.Z` (example: `v0.2.2`)

Recommended:
- Keep `Cargo.toml` and `pyproject.toml` versions aligned.
- Release only from a clean `main` branch state.

---

## Pre-release checklist

Before tagging:

1. Run tests locally
   - `cargo test`
   - `uv run -m pytest`
2. Verify version in:
   - `Cargo.toml`
   - `pyproject.toml`
3. Update docs/changelog as needed.
4. Commit and push all release-related changes.

---

## TestPyPI release (dry run)

## Option A: tag-triggered

1. Create and push a test tag:
   - `git tag test-v0.2.2`
   - `git push origin test-v0.2.2`

2. Wait for workflow:
   - `Release (TestPyPI)`

3. Verify package on TestPyPI page:
   - `https://test.pypi.org/p/subtask-manager`

4. Optional install test:
   - `python -m pip install -i https://test.pypi.org/simple/ subtask-manager==0.2.2`

## Option B: manual trigger

1. Open Actions tab.
2. Run workflow:
   - `Release (TestPyPI)`
3. Validate artifacts and TestPyPI package page.

---

## Production PyPI release

Only do this after successful TestPyPI validation.

## Option A: tag-triggered

1. Create and push production tag:
   - `git tag v0.2.2`
   - `git push origin v0.2.2`

2. Wait for workflow:
   - `Release`

3. Verify package on PyPI page:
   - `https://pypi.org/p/subtask-manager`

4. Optional install test:
   - `python -m pip install subtask-manager==0.2.2`

## Option B: manual trigger

1. Open Actions tab.
2. Run workflow:
   - `Release`
3. Confirm publish job completed successfully.

---

## What gets published

Each release should include:

- Linux wheel(s)
- Windows wheel(s)
- sdist (`.tar.gz`)

Artifacts are built in separate jobs, then collected and published in a final publish job.

---

## Verification after publish

1. Check release files on index page (PyPI/TestPyPI).
2. Confirm expected wheel tags exist (Linux and Windows).
3. Install package in a fresh virtual environment.
4. Import check:
   - `python -c "import subtask_manager; print('ok')"`

---

## Rollback / recovery notes

PyPI does not support replacing an existing version file.
If a bad release is published:

1. Yank the bad version on PyPI.
2. Fix issue in code/workflow.
3. Publish a **new** version (e.g. `0.2.3`).

For TestPyPI, you can re-test using a new version tag as well.

---

## Common failure cases

- Trusted publishing misconfiguration (wrong workflow file/environment on index side)
- Missing environment in GitHub repo
- Version mismatch between `Cargo.toml` and `pyproject.toml`
- Tag format mismatch (`test-v*` vs `v*`)
- Wheels built, but publish blocked by environment approvals

---

## Quick command summary

### Dry run (TestPyPI)
- `git tag test-vX.Y.Z`
- `git push origin test-vX.Y.Z`

### Production
- `git tag vX.Y.Z`
- `git push origin vX.Y.Z`

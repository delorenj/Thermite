# Branch Protection Rules

This document outlines the required branch protection settings for the Thermite project.

## Main Branch Protection

Navigate to: `Settings > Branches > Branch protection rules`

### Required Settings for `main` branch:

**Protect matching branches:**
- ✅ Require a pull request before merging
  - ✅ Require approvals: 1
  - ✅ Dismiss stale pull request approvals when new commits are pushed
  - ✅ Require review from Code Owners (if CODEOWNERS file exists)

- ✅ Require status checks to pass before merging
  - ✅ Require branches to be up to date before merging
  - **Required status checks:**
    - `Rust Tests and Linting`
    - `Python Tests and Linting (auth-service)`
    - `Python Tests and Linting (matchmaking-service)`
    - `Python Tests and Linting (persistence-service)`
    - `Python Tests and Linting (match-orchestrator)`
    - `Docker Image Builds`
    - `Integration Check`

- ✅ Require conversation resolution before merging

- ✅ Require linear history

- ✅ Include administrators (enforce rules for admins too)

## Develop Branch Protection

### Required Settings for `develop` branch:

**Protect matching branches:**
- ✅ Require a pull request before merging
  - ✅ Require approvals: 0 (optional for develop)

- ✅ Require status checks to pass before merging
  - ✅ Require branches to be up to date before merging
  - **Required status checks:**
    - `Rust Tests and Linting`
    - `Python Tests and Linting (auth-service)`
    - `Python Tests and Linting (matchmaking-service)`
    - `Python Tests and Linting (persistence-service)`
    - `Python Tests and Linting (match-orchestrator)`
    - `Docker Image Builds`

- ✅ Require linear history

## Coverage Requirements

All code must meet minimum coverage thresholds:

- **Python services:** 80% minimum (enforced via `--cov-fail-under=80`)
- **Rust game server:** 80% minimum (enforced via `--fail-under 80`)

Builds will fail if coverage drops below these thresholds.

## Setting Up Branch Protection

1. Go to repository Settings
2. Navigate to "Branches" in the left sidebar
3. Click "Add rule" or edit existing rule
4. Enter branch name pattern: `main` or `develop`
5. Enable all checkboxes listed above
6. Select all required status checks from the dropdown
7. Click "Create" or "Save changes"

## Codecov Integration

Coverage reports are automatically uploaded to Codecov on every push/PR:

1. Sign up at https://codecov.io
2. Add Thermite repository
3. Add `CODECOV_TOKEN` to GitHub Secrets (Settings > Secrets and variables > Actions)
4. Coverage badges and PR comments will be automatically generated

## Required Secrets

Add these secrets in: `Settings > Secrets and variables > Actions`

**Required for deployment:**
- `DEPLOY_HOST`: Production server IP/hostname
- `DEPLOY_USER`: SSH username for deployment
- `DEPLOY_SSH_KEY`: Private SSH key for deployment access

**Optional for enhanced coverage reporting:**
- `CODECOV_TOKEN`: Codecov API token for coverage reports

## Troubleshooting

**Status checks not appearing:**
- Trigger a workflow run first (push a commit or open a PR)
- Wait for workflows to complete at least once
- Refresh the branch protection page

**Coverage failing:**
- Check individual service coverage reports in workflow logs
- Run tests locally: `cd services/<service> && uv run pytest --cov`
- Add more tests to increase coverage above 80%

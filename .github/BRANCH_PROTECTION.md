# Branch Protection Configuration

This document describes the branch protection rules that should be configured in GitHub to ensure code quality and prevent broken builds from being merged.

## Required Branch Protection Rules

### For `main` branch:

1. **Require pull request reviews before merging**
   - Required approving reviews: 1 (optional for solo developer)
   - Dismiss stale pull request approvals when new commits are pushed: ✅

2. **Require status checks to pass before merging**
   - Require branches to be up to date before merging: ✅
   - Status checks that are required:
     - `Rust Tests and Linting`
     - `Python Tests and Linting`
     - `Docker Image Builds`
     - `Integration Check`

3. **Require conversation resolution before merging**: ✅

4. **Do not allow bypassing the above settings**: ✅

5. **Restrict who can push to matching branches** (optional):
   - Only allow specific people, teams, or apps to push: Configure as needed

### For `develop` branch:

Same rules as `main` branch to ensure develop branch remains stable.

## How to Configure

1. Go to repository Settings → Branches
2. Click "Add rule" under Branch protection rules
3. Enter branch name pattern: `main`
4. Enable the checkboxes listed above
5. Under "Require status checks to pass before merging":
   - Search for and select the CI job names listed above
6. Click "Create" or "Save changes"
7. Repeat for `develop` branch

## Verification

After configuring:
1. Create a test branch with intentionally failing tests
2. Open a pull request to `main`
3. Verify that the PR shows "Some checks were not successful"
4. Verify that the "Merge" button is disabled
5. Fix the tests
6. Verify that the "Merge" button becomes enabled after all checks pass

## CI/CD Pipeline Jobs

The following jobs are defined in `.github/workflows/ci.yml`:

1. **rust-test**: Runs cargo test, clippy, and formatting checks
2. **python-test**: Runs pytest with coverage, ruff linting, and formatting checks
3. **docker-build**: Builds all Docker images to ensure Dockerfiles are valid
4. **integration-check**: Final verification that all jobs passed

All jobs must pass for a PR to be mergeable when branch protection is enabled.

## Codecov Integration

Coverage reports are uploaded to Codecov. To enable:

1. Sign up at https://codecov.io with your GitHub account
2. Add the Thermite repository
3. Copy the `CODECOV_TOKEN` from the Codecov dashboard
4. Add it as a repository secret in GitHub:
   - Settings → Secrets and variables → Actions → New repository secret
   - Name: `CODECOV_TOKEN`
   - Value: [paste token from Codecov]

Coverage badge will display on README.md once first workflow run completes with coverage data.

## Notes for Solo Developer

For a solo developer project, you may choose to:
- Disable "Require pull request reviews" (since you're reviewing your own code)
- Keep status checks required (maintains code quality)
- Use PRs for feature branches to maintain good git hygiene

However, all status checks should always be required to prevent regressions.

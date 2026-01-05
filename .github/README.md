# CI/CD Pipeline Documentation

This directory contains GitHub Actions workflows for automated testing, building, and deployment of the Thermite project.

## Workflows

### 1. CI Pipeline (`ci.yml`)

**Triggers:** Push or PR to `main` or `develop` branches

**Jobs:**

#### Rust Tests and Linting
- Runs `cargo fmt --check` (code formatting)
- Runs `cargo clippy` (linting)
- Runs `cargo test` (unit and integration tests)
- Runs `cargo tarpaulin` (code coverage with 80% minimum threshold)
- Uploads coverage reports to Codecov

#### Python Tests and Linting (Matrix Strategy)
- Tests all 4 FastAPI services in parallel:
  - auth-service
  - matchmaking-service
  - persistence-service
  - match-orchestrator
- Each service:
  - Runs `ruff check` (linting)
  - Runs `ruff format --check` (formatting)
  - Runs `pytest --cov` (tests with 80% minimum coverage)
  - Uploads coverage to Codecov

#### Docker Image Builds
- Builds Docker images for all services
- Uses BuildKit caching for faster builds
- Validates images can be built successfully

#### Integration Check
- Runs after all other jobs complete
- Verifies all CI checks passed

### 2. Deploy Pipeline (`deploy.yml`)

**Triggers:** Push to `main` or manual workflow dispatch

**Jobs:**

#### Build and Push Images
- Builds Docker images for production
- Pushes to GitHub Container Registry (ghcr.io)
- Tags with branch name, commit SHA, and `latest`
- Uses layer caching for efficiency

#### Deploy to Production
- SSHs into production server
- Pulls latest code and images
- Runs `docker compose up -d`
- Performs health checks on all services

## Local Testing

### Test Python Services Locally

```bash
cd services/auth-service
uv run pytest --cov=app --cov-report=term --cov-fail-under=80
uv run ruff check .
uv run ruff format --check .
```

### Test Rust Server Locally

```bash
cd server
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo tarpaulin --out stdout --fail-under 80
```

### Test Docker Builds Locally

```bash
docker compose build auth-service
docker compose build matchmaking-service
docker compose build persistence-service
docker compose build match-orchestrator
docker compose build game-server
```

### Test Workflows with `act`

Install `act`: https://github.com/nektos/act

```bash
# Test CI workflow
act -j python-test

# Test deployment workflow (dry-run)
act -j build-and-push --secret-file .env.secrets
```

## Required GitHub Secrets

Add these in: `Settings > Secrets and variables > Actions`

### For Deployment:
- `DEPLOY_HOST`: Production server hostname/IP
- `DEPLOY_USER`: SSH username
- `DEPLOY_SSH_KEY`: Private SSH key for deployment

### Optional:
- `CODECOV_TOKEN`: Codecov API token (for enhanced reporting)

## Coverage Requirements

All code must maintain minimum 80% test coverage:

- **Python:** Enforced via `pytest --cov-fail-under=80`
- **Rust:** Enforced via `cargo tarpaulin --fail-under 80`

Coverage reports are uploaded to Codecov automatically.

## Branch Protection

See [BRANCH_PROTECTION.md](./BRANCH_PROTECTION.md) for detailed setup instructions.

**Main branch requires:**
- All CI checks passing
- 1 approval
- Linear history
- Up-to-date branches

## Deployment Process

1. **Develop → Main PR:**
   - Create PR from `develop` to `main`
   - Wait for all CI checks to pass
   - Get 1 approval
   - Merge PR

2. **Automatic Deployment:**
   - Merge to `main` triggers deploy workflow
   - Images built and pushed to ghcr.io
   - Production server updated via SSH
   - Health checks verify deployment

3. **Manual Deployment:**
   - Go to Actions > Deploy to Production
   - Click "Run workflow"
   - Select branch (usually `main`)
   - Click "Run workflow"

## Troubleshooting

### Coverage Failing

```bash
# Check coverage locally
cd services/auth-service
uv run pytest --cov=app --cov-report=html
open htmlcov/index.html

# Add more tests to increase coverage
```

### Docker Build Failing

```bash
# Test build locally
cd services/auth-service
docker build -t auth-service:test .

# Check logs for errors
docker logs auth-service-container
```

### Deployment Failing

```bash
# SSH into production server
ssh user@production-server

# Check service status
cd /opt/thermite
docker compose ps
docker compose logs -f

# Manual deployment
git pull origin main
docker compose pull
docker compose up -d --remove-orphans
```

## Workflow Status Badges

Add to README.md:

```markdown
![CI](https://github.com/username/thermite/actions/workflows/ci.yml/badge.svg)
![Deploy](https://github.com/username/thermite/actions/workflows/deploy.yml/badge.svg)
[![codecov](https://codecov.io/gh/username/thermite/branch/main/graph/badge.svg)](https://codecov.io/gh/username/thermite)
```

## Maintenance

### Updating Workflows

1. Edit workflow files in `.github/workflows/`
2. Test changes in a feature branch
3. Create PR to `develop`
4. Verify workflows run successfully
5. Merge to `develop`, then to `main`

### Adding New Services

1. Add service to Docker build matrix in `ci.yml`
2. Add service to Python test matrix if Python
3. Add service to deploy workflow
4. Add service to health check in deploy workflow
5. Update this README

### Security Updates

- Review and update action versions quarterly
- Monitor security advisories for dependencies
- Run `uv pip list --outdated` for Python deps
- Run `cargo outdated` for Rust deps

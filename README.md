# Thermite

Grid-based tactical extraction shooter combining Bomberman-style combat mechanics with Tarkov-inspired risk/reward gameplay.

## Architecture

**Pattern:** Event-Driven Microservices
**Stack:** Rust (Bevy game client/server), FastAPI (backend services), PostgreSQL, RabbitMQ
**Deployment:** Docker Compose (MVP) → Kubernetes (v2.0)

## Development Environment

### Prerequisites

All tooling is managed via **mise** for consistent version control across the team.

**Required Tools:**
- mise (version management)
- Rust 1.92.0+ (game client/server)
- Python 3.12+ (backend services)
- uv (Python package manager)
- PostgreSQL 16+ (economic data, stash persistence)
- RabbitMQ 3.13+ (event coordination)
- Docker + Docker Compose (containerization)

### Setup Verification

Run these commands to verify your development environment is correctly configured:

#### 1. Mise and Tool Versions

```bash
# Verify mise is installed
mise --version

# List all managed tools
mise ls --current

# Expected output should include:
# rust    1.92.0
# python  3.13.2 (or higher)
# uv      0.9.7 (or higher)
```

#### 2. Rust Toolchain

```bash
# Verify Rust installation (use mise exec to ensure correct version)
mise x -- cargo --version
mise x -- rustc --version

# Expected:
# cargo 1.92.0+
# rustc 1.92.0+

# Test Bevy compilation (quick check)
cargo --version
```

#### 3. Python and uv

```bash
# Verify Python
mise x -- python --version

# Expected: Python 3.12+ (currently 3.13.2)

# Verify uv package manager
uv --version

# Expected: uv 0.9.7+
```

#### 4. PostgreSQL

```bash
# Check PostgreSQL version
psql --version

# Expected: psql (PostgreSQL) 16.0+

# Verify PostgreSQL service is running
systemctl status postgresql

# Expected: Active: active (exited)

# Test thermite database connection
psql -U delorenj -d thermite -c "SELECT version();"

# Expected: PostgreSQL 16+ version string
```

#### 5. RabbitMQ

```bash
# Check RabbitMQ service
systemctl status rabbitmq-server

# Expected: Active: active (running)

# Verify management plugin (web UI)
curl -s http://localhost:15672/ | head -5

# Expected: HTML output (management UI is accessible)

# Access management UI in browser:
# http://localhost:15672
# Default credentials: guest/guest
```

#### 6. Docker

```bash
# Check Docker
docker --version

# Expected: Docker version 20.10+

# Check Docker Compose
docker compose version

# Expected: Docker Compose version v2.0+

# Verify Docker daemon is running
docker ps

# Expected: CONTAINER ID list (may be empty)
```

### Quick Verification Script

Run all verification checks at once:

```bash
#!/bin/bash
echo "=== Mise Tools ==="
mise ls --current | grep -E "rust|python|uv"

echo -e "\n=== Rust Toolchain ==="
mise x -- cargo --version
mise x -- rustc --version

echo -e "\n=== Python & uv ==="
mise x -- python --version
uv --version

echo -e "\n=== PostgreSQL ==="
psql --version
psql -U delorenj -d thermite -c "SELECT 1" && echo "✓ thermite database accessible"

echo -e "\n=== RabbitMQ ==="
systemctl is-active rabbitmq-server && echo "✓ RabbitMQ running"
curl -s http://localhost:15672/ > /dev/null && echo "✓ Management UI accessible at http://localhost:15672"

echo -e "\n=== Docker ==="
docker --version
docker compose version
docker ps > /dev/null && echo "✓ Docker daemon running"

echo -e "\n=== Setup Complete ==="
```

Save as `verify-setup.sh`, make executable (`chmod +x verify-setup.sh`), and run (`./verify-setup.sh`).

## Project Structure

```
Thermite/
├── docs/                    # Planning documentation
│   ├── prd-thermite-2025-12-25.md
│   ├── architecture-thermite-2025-12-26.md
│   └── sprint-plan-thermite-2025-12-26.md
├── client/                  # Bevy game client (Rust)
├── server/                  # Bevy game server (Rust)
├── services/                # Backend microservices
│   ├── stash-service/      # FastAPI - Persistent storage
│   ├── economy-service/    # FastAPI - Currency, shop
│   └── matchmaking/        # FastAPI - Raid coordination
├── docker/                  # Docker Compose configs
└── README.md               # This file
```

## Database Configuration

**PostgreSQL Database:** `thermite`
**User:** `delorenj` (or your system user)
**Connection String:** `postgresql://delorenj@localhost:5432/thermite`

**Schema Management:**
- Migrations tracked in `services/*/migrations/`
- Applied via SQLAlchemy Alembic (FastAPI services)
- Manual schema for game server (Rust/sqlx)

## RabbitMQ Configuration

**Management UI:** http://localhost:15672
**AMQP Port:** 5672
**Default Credentials:** guest/guest (development only)

**Event Flow:**
- Game Server publishes combat events → RabbitMQ
- Backend services consume events → Update persistent state
- Bloodbank event backbone pattern (33GOD ecosystem)

## Docker Deployment

### Quick Start (Development)

**1. Copy environment template:**

```bash
cp .env.example .env
```

**2. Start all services:**

```bash
docker compose up -d
```

**3. Verify services are running:**

```bash
docker compose ps
```

**4. View logs:**

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f postgres
docker compose logs -f matchmaking-service
```

**5. Stop services:**

```bash
docker compose down

# With volume cleanup (removes data)
docker compose down -v
```

### Service Ports

| Service | Port | Description |
|---------|------|-------------|
| PostgreSQL | 5432 | Database |
| Redis | 6379 | Cache/sessions |
| RabbitMQ | 5672 | AMQP message broker |
| RabbitMQ Management | 15672 | Web UI (guest/guest) |
| Traefik Dashboard | 8080 | Reverse proxy UI |
| Pre-Raid UI | 3000 | React frontend |
| Game Server | 9001 | WebSocket (dynamically spawned) |
| Matchmaking Service | 8002 | FastAPI |
| Persistence Service | 8003 | FastAPI |
| Auth Service | 8004 | FastAPI |
| Match Orchestrator | 8005 | FastAPI |
| Economy Service | 8006 | FastAPI |

### Container Health Checks

All services have health checks configured:

```bash
# View service health
docker compose ps

# Expected: All services show "healthy" status
```

### Volumes

Persistent data volumes:

- `postgres_data` - Database storage
- `redis_data` - Redis persistence
- `rabbitmq_data` - Message queue storage
- `traefik_certs` - TLS certificates
- `logs_data` - Application logs

### Networks

- `thermite-internal` - Inter-service communication
- `thermite-external` - Public-facing services (UI, Traefik)

### Development Mode

Services are configured for development with:

- Hot reload (where applicable)
- Volume mounts for live code updates
- Debug logging enabled
- Exposed ports for direct access

### Production Deployment

Before deploying to production:

1. Generate secure credentials (see `.env.example`)
2. Configure TLS certificates in Traefik
3. Set `NODE_ENV=production`
4. Disable Traefik insecure API
5. Review security settings in docker-compose.yml

## Getting Started

**Implementation is tracked via Sprint Planning:**

See `docs/sprint-plan-thermite-2025-12-26.md` for complete story breakdown.

**Current Sprint:** Sprint 1 - Foundation & Infrastructure
**First Story:** STORY-INF-002 (Docker Compose Deployment Setup)

**Next Steps:**
1. Verify environment setup (run verification script above)
2. Review architecture: `docs/architecture-thermite-2025-12-26.md`
3. Set up Docker environment: `docker compose up -d`
4. Begin Sprint 1 implementation

## Development Workflow

**Branch Strategy:** Git flow with develop/master branches
**Sprint Cadence:** 2-week sprints (5 total planned)
**Story Format:** STORY-XXX (see sprint plan)

**Mise Task Integration:**
Project tasks will be managed via `mise tasks` (configured in `.mise.toml`)

## Additional Resources

- **PRD:** `docs/prd-thermite-2025-12-25.md` - Complete requirements
- **Architecture:** `docs/architecture-thermite-2025-12-26.md` - System design
- **Sprint Plan:** `docs/sprint-plan-thermite-2025-12-26.md` - Implementation roadmap
- **Gate Check:** `docs/gate-check-report-thermite-2025-12-26.md` - Architecture validation

## License

[To be determined]

## Contributors

- Jarad DeLorenzo (@delorenj) - Staff Engineer

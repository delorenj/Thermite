# BOMBOUT System Architecture

**Project:** Thermite (BOMBOUT)
**Version:** 1.0
**Date:** 2025-12-26
**Author:** System Architect (BMAD Method v6)
**Status:** Final

**Related Documents:**
- [Product Requirements Document](prd-thermite-2025-12-25.md)

---

## Executive Summary

BOMBOUT is a grid-based extraction shooter combining Bomberman-style combat with Tarkov's high-stakes loot mechanics. This architecture document defines the technical design for the MVP, supporting 80+ concurrent players across 10-12 matches with real-time multiplayer gameplay.

**Key Architectural Decisions:**
- **Pattern:** Event-Driven Microservices with Authoritative Game Server
- **Game Client:** Bevy (Rust) for native desktop performance
- **Game Server:** Rust + Tokio for 20Hz real-time simulation
- **Backend Services:** FastAPI (Python) for REST APIs
- **Database:** PostgreSQL 16 for persistent state
- **Message Broker:** RabbitMQ 3.13 for event distribution
- **Deployment:** Docker Compose (MVP) → Kubernetes (v2)

**Target NFRs:**
- **Input Responsiveness:** < 100ms input latency
- **Bomb Timer Accuracy:** < 50ms sync across clients
- **Match Capacity:** 10+ concurrent matches (80 players)
- **Uptime:** 90% during testing windows
- **Platform Support:** Windows, macOS, Linux desktop

---

## Table of Contents

1. [Architectural Drivers](#architectural-drivers)
2. [High-Level Architecture](#high-level-architecture)
3. [Technology Stack](#technology-stack)
4. [System Components](#system-components)
5. [Data Architecture](#data-architecture)
6. [API Design](#api-design)
7. [NFR Coverage](#nfr-coverage)
8. [Security Architecture](#security-architecture)
9. [Scalability & Performance](#scalability--performance)
10. [Reliability & Availability](#reliability--availability)
11. [Development & Deployment](#development--deployment)
12. [Traceability](#traceability)
13. [Trade-offs](#trade-offs)

---

## Architectural Drivers

Architectural drivers are NFRs that heavily influence design decisions.

### Critical Drivers (Must-Have)

1. **NFR-001: Input Responsiveness**
   - **Requirement:** < 100ms input latency, 20 ticks/second server update rate
   - **Impact:** Dictates client-server architecture, requires client-side prediction
   - **Solution:** WebSocket bi-directional communication, optimistic client updates

2. **NFR-002: Bomb Timer Accuracy**
   - **Requirement:** < 50ms timer deviation across clients
   - **Impact:** Server must be authoritative for all timing-critical state
   - **Solution:** Deterministic tick-based timers, server broadcasts detonation events

3. **NFR-004: Authoritative Server**
   - **Requirement:** Server validates all game-critical state
   - **Impact:** Client cannot be trusted, all state changes verified server-side
   - **Solution:** Command-based input model, server rejects invalid commands

4. **NFR-012: Platform Support**
   - **Requirement:** Cross-platform desktop (Windows, macOS, Linux)
   - **Impact:** Technology choices must support all platforms
   - **Solution:** Bevy game engine (Rust), compiles to all targets

### Important Drivers (Should-Have)

5. **NFR-010: Structured Logging**
   - **Requirement:** JSON logs with match IDs, player IDs, event types
   - **Impact:** Observability architecture, debugging capabilities
   - **Solution:** Structured logging libraries, centralized aggregation (Loki)

6. **NFR-003: Match Capacity**
   - **Requirement:** 10+ concurrent matches (80 players)
   - **Impact:** Process isolation, resource management strategy
   - **Solution:** One Game Server process per match, orchestrator manages pool

7. **NFR-007: Crash Recovery**
   - **Requirement:** Server crash doesn't corrupt player stash
   - **Impact:** Data persistence strategy, transaction boundaries
   - **Solution:** Database ACID transactions, pre-raid loadout snapshots

---

## High-Level Architecture

### Architectural Pattern

**Event-Driven Microservices with Authoritative Game Server**

**Rationale:**
- **Event-Driven:** Match lifecycle events (created, started, ended) coordinate services
- **Microservices:** Independent services (matchmaking, persistence, auth) scale independently
- **Authoritative Game Server:** Separate process per match, owns simulation truth

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         BOMBOUT Architecture                        │
└─────────────────────────────────────────────────────────────────────┘

                        ┌──────────────────┐
                        │   Game Client    │
                        │  (Bevy/Rust)     │
                        │  Desktop App     │
                        └─────────┬────────┘
                                  │
                    WebSocket     │      HTTPS
                    (Game State)  │      (Pre-Raid API)
                                  │
              ┌───────────────────┼───────────────────────┐
              │                   │                       │
              ▼                   ▼                       ▼
    ┌─────────────────┐  ┌────────────────┐   ┌──────────────────┐
    │  Game Server    │  │  Pre-Raid UI   │   │  Traefik Proxy   │
    │  (Rust/Tokio)   │  │  (React SPA)   │   │  (Load Balancer) │
    │  Port 9001      │  │  Port 3000     │   │  Port 443        │
    └────────┬────────┘  └────────┬───────┘   └────────┬─────────┘
             │                    │                     │
             │ Events             │ API Calls           │ Routes
             ▼                    ▼                     ▼
    ┌──────────────────────────────────────────────────────────┐
    │                    RabbitMQ Message Broker               │
    │                    (Event Distribution)                  │
    └────────────────────────┬─────────────────────────────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
         ▼                   ▼                   ▼
┌────────────────┐  ┌────────────────┐  ┌──────────────────┐
│ Match          │  │ Matchmaking    │  │ Persistence      │
│ Orchestrator   │  │ Service        │  │ Service          │
│ (FastAPI)      │  │ (FastAPI)      │  │ (FastAPI)        │
│ Port 8001      │  │ Port 8002      │  │ Port 8003        │
└───────┬────────┘  └───────┬────────┘  └────────┬─────────┘
        │                   │                     │
        │ Queue Mgmt        │ Session Cache       │ Persistent Data
        ▼                   ▼                     ▼
┌─────────────────┐  ┌────────────────┐  ┌──────────────────┐
│  Redis Cache    │  │  Redis Cache   │  │  PostgreSQL 16   │
│  (Match State)  │  │  (Queue)       │  │  (Stash, Users)  │
└─────────────────┘  └────────────────┘  └──────────────────┘

┌────────────────────────────────────────────────────────────────┐
│                  Authentication Service (FastAPI)               │
│                  Port 8004 - JWT Validation                     │
└────────────────────────────────────────────────────────────────┘
```

### Component Interaction Flows

**Match Start Flow:**

```
1. Player → Pre-Raid UI: Select loadout
2. Player → Matchmaking: Join queue (POST /api/v1/queue)
3. Matchmaking → Redis: Add to sorted set
4. [8 players in queue]
5. Matchmaking → Match Orchestrator: match.ready event (RabbitMQ)
6. Match Orchestrator → Game Server: Spawn process with player list
7. Game Server → Players: WebSocket connection handshake
8. Game Server → All Clients: Broadcast initial state (spawn positions)
9. Match begins (20Hz tick loop starts)
```

**Match End Flow:**

```
1. Game Server: Match end condition (timer + extraction/death)
2. Game Server → RabbitMQ: Emit match.ended event
3. Match Orchestrator subscribes: Receives event
4. Match Orchestrator → Persistence Service: POST /matches/{id}/results
5. Persistence Service: Transaction BEGIN
   - Process extractors: Add loot to stash, update currency
   - Process deaths: Remove equipped gear from stash
   - Update match_participants with outcomes
6. Persistence Service: Transaction COMMIT
7. Players: Redirect to Pre-Raid UI (show results)
```

---

## Technology Stack

### Summary Table

| Layer | Technology | Rationale |
|-------|------------|-----------|
| **Game Client** | Bevy (Rust) | Native performance, cross-platform, ECS architecture |
| **Game Server** | Rust + Tokio | Low latency, deterministic ticks, async WebSocket |
| **Backend Services** | FastAPI (Python) | Rapid development, Pydantic validation, async |
| **Pre-Raid UI** | React 19 + TypeScript | Modern SPA, type safety, component reusability |
| **Database** | PostgreSQL 16 | ACID guarantees, JSONB for flexible data |
| **Message Broker** | RabbitMQ 3.13 | Durable queues, pub/sub, battle-tested |
| **Cache** | Redis 7 | Session management, queue state, low latency |
| **Reverse Proxy** | Traefik 2.10 | Auto-discovery, TLS termination, load balancing |
| **Observability** | Loki + Grafana | Structured log aggregation, dashboards |
| **Deployment** | Docker Compose | Simple MVP deployment, version parity |

### Detailed Stack Justification

#### Game Client: Bevy (Rust)

**Choice:** Bevy 0.12 game engine

**Rationale:**
- **Cross-platform:** Single codebase compiles to Windows, macOS, Linux
- **Performance:** Rust zero-cost abstractions, compiled native code
- **ECS Architecture:** Entity-Component-System perfect for game objects
- **2D Rendering:** Bevy 2D renderer efficient for grid-based visuals
- **Community:** Active ecosystem, good documentation

**Trade-offs:**
- ✓ **Gain:** Native performance, deterministic memory management
- ✗ **Lose:** Slower iteration vs. Unity (longer compile times)

**Alternatives Considered:**
- Unity: Rejected (heavier engine, overkill for 2D grid)
- Godot: Rejected (less Rust support, smaller ecosystem)

---

#### Game Server: Rust + Tokio

**Choice:** Rust 1.75+ with Tokio async runtime

**Rationale:**
- **Low Latency:** Predictable performance for 20Hz tick rate
- **Async WebSocket:** Tokio handles thousands of concurrent connections
- **Type Safety:** Rust prevents common bugs (null refs, data races)
- **Ecosystem:** `tokio-tungstenite` for WebSocket, `rmp-serde` for MessagePack

**Trade-offs:**
- ✓ **Gain:** Guaranteed memory safety, zero GC pauses, low CPU
- ✗ **Lose:** Development speed vs. Node.js/Python

**Alternatives Considered:**
- Node.js: Rejected (GC pauses, less predictable latency)
- Go: Considered (good concurrency), but Rust ecosystem better for game logic

---

#### Backend Services: FastAPI (Python)

**Choice:** FastAPI 0.109+ with Pydantic v2

**Rationale:**
- **Rapid Development:** Python productivity for CRUD services
- **Type Safety:** Pydantic validates requests, prevents errors
- **Async Support:** `asyncpg`, `aiohttp` for non-blocking I/O
- **OpenAPI:** Auto-generated API docs

**Trade-offs:**
- ✓ **Gain:** Fast iteration, large ecosystem (Postgres, Redis, RabbitMQ)
- ✗ **Lose:** Performance vs. Rust (acceptable for non-real-time services)

**Alternatives Considered:**
- Django: Rejected (too heavyweight, sync ORM)
- Express (Node.js): Considered, but Python team expertise

---

#### Pre-Raid UI: React 19 + TypeScript

**Choice:** React 19, TypeScript, Vite, Tailwind CSS, shadcn/ui

**Rationale:**
- **Modern SPA:** Fast client-side rendering, reactive updates
- **Type Safety:** TypeScript catches bugs at compile-time
- **Vite:** Fast dev server, optimized builds
- **Tailwind:** Utility-first CSS, rapid UI development
- **shadcn/ui:** Pre-built accessible components (drag-drop, modals)

**Trade-offs:**
- ✓ **Gain:** Best-in-class developer experience, large ecosystem
- ✗ **Lose:** Bundle size vs. vanilla JS (mitigated by code splitting)

**Alternatives Considered:**
- Vue: Rejected (smaller ecosystem, less TypeScript support)
- Svelte: Considered (smaller bundles), but React team familiarity

---

#### Database: PostgreSQL 16

**Choice:** PostgreSQL 16 with JSONB extensions

**Rationale:**
- **ACID Transactions:** Critical for stash updates (prevent duplication)
- **JSONB:** Flexible storage for loadout snapshots, loot data
- **Performance:** Excellent query optimizer, mature indexing
- **Reliability:** Battle-tested for 20+ years

**Trade-offs:**
- ✓ **Gain:** Data integrity, powerful querying, proven reliability
- ✗ **Lose:** Vertical scaling limits (acceptable for MVP scale)

**Alternatives Considered:**
- MongoDB: Rejected (no ACID guarantees for transactions)
- MySQL: Considered, but Postgres JSONB superior

---

#### Message Broker: RabbitMQ 3.13

**Choice:** RabbitMQ with persistent queues

**Rationale:**
- **Durable Queues:** Messages survive broker restart
- **Pub/Sub:** Match lifecycle events fan out to multiple subscribers
- **Dead Letter Queues:** Failed messages routed for retry
- **Battle-Tested:** Used by millions, proven reliability

**Trade-offs:**
- ✓ **Gain:** Event delivery guarantees, mature tooling
- ✗ **Lose:** Operational complexity vs. Redis Pub/Sub

**Alternatives Considered:**
- Redis Pub/Sub: Rejected (no message persistence)
- Kafka: Rejected (overkill for MVP scale, complex ops)

---

#### Infrastructure: Docker Compose → Kubernetes (v2)

**MVP Choice:** Docker Compose

**Rationale:**
- **Simplicity:** Single `docker-compose.yml`, easy local dev
- **Fast Deploy:** `docker compose up -d` deploys all services
- **Cost:** Runs on single host ($50/month cloud VM)

**v2 Migration:** Kubernetes for horizontal scaling

**Trade-offs:**
- ✓ **Gain:** Minimal operational overhead, fast iteration
- ✗ **Lose:** Scaling limits (20 matches max per host)

---

## System Components

### 1. Game Client (Bevy/Rust)

**Purpose:** Native desktop application providing real-time gameplay experience

**Responsibilities:**
- Render grid-based game world at 60 FPS (NFR-001)
- Handle player input (WASD movement, bomb placement)
- Client-side prediction for responsive movement
- Synchronize with authoritative server state
- Display UI overlays (timer, inventory, minimap)
- Play audio feedback (bomb placement, explosions, footsteps)
- Manage WebSocket connection to Game Server
- Render death replay visualization

**Interfaces:**
- WebSocket client (connects to Game Server on port 9001)
- REST API consumer (Pre-Raid UI Service for loadout selection)
- Local file system (settings, cache)

**Dependencies:**
- Game Server (authoritative state)
- Pre-Raid UI Service (loadout data)
- Authentication Service (session tokens)

**FRs Addressed:** FR-005 (grid movement), FR-006 (bomb placement), FR-007 (blast mechanics), FR-013 (death feedback), FR-021 (visual clarity)

---

### 2. Game Server (Rust/Tokio)

**Purpose:** Authoritative simulation of match gameplay logic

**Responsibilities:**
- Execute game tick at 20Hz (50ms intervals) - NFR-002
- Validate all player actions (movement, bomb placement)
- Simulate bomb timers and blast propagation
- Detect player deaths and loot drops
- Manage match state transitions (countdown, active, post-match)
- Broadcast state updates to all clients
- Handle player disconnections with 10-second reconnect window
- Log match events for audit trail

**Interfaces:**
- WebSocket server (port 9001) - bi-directional with clients
- RabbitMQ publisher - emits match lifecycle events
- RabbitMQ consumer - receives match start commands

**Dependencies:**
- RabbitMQ (event distribution)
- Match Orchestrator (match initialization data)
- Persistence Service (loot drop validation)

**FRs Addressed:** FR-001 (match duration), FR-002 (player spawns), FR-005 (movement), FR-006 (bombs), FR-007 (blasts), FR-008 (extraction), FR-009 (death), FR-011 (item loss)

---

### 3. Match Orchestrator Service (FastAPI)

**Purpose:** Manages end-to-end match lifecycle coordination

**Responsibilities:**
- Create match instances from matchmaking queues
- Allocate Game Server processes (spawn/pool)
- Distribute spawn positions and map configuration
- Monitor match health (heartbeat, crash detection)
- Handle graceful shutdown and cleanup
- Collect match results (survivors, deaths, loot extracted)
- Trigger post-match persistence updates
- Maintain match history logs

**Interfaces:**
- REST API (internal, port 8001)
  - POST /matches - create match
  - GET /matches/{id} - match status
  - DELETE /matches/{id} - terminate match
- RabbitMQ consumer - match lifecycle events
- RabbitMQ publisher - match creation/termination events

**Dependencies:**
- Matchmaking Service (player queue data)
- Game Server (spawns instances)
- RabbitMQ (event coordination)
- Redis (match state cache)
- Persistence Service (post-match updates)

**FRs Addressed:** FR-001 (match timing), FR-002 (spawn system), FR-003 (concurrent matches), FR-022 (crash recovery)

---

### 4. Matchmaking Service (FastAPI)

**Purpose:** Queue management and player matching

**Responsibilities:**
- Accept player queue requests
- Match players based on simple FIFO (MVP: no MMR)
- Validate player availability (not in-match, authenticated)
- Enforce match size constraints (2-8 players)
- Handle queue cancellations and timeouts
- Emit match-ready events to Match Orchestrator
- Track queue statistics (wait times, match fill rate)

**Interfaces:**
- REST API (port 8002)
  - POST /queue - join matchmaking
  - DELETE /queue/{player_id} - leave queue
  - GET /queue/status - queue position
- WebSocket notifications (queue status updates to clients)
- RabbitMQ publisher - match ready events

**Dependencies:**
- Authentication Service (validate tokens)
- Match Orchestrator (match creation)
- RabbitMQ (event emission)
- Redis (queue state)

**FRs Addressed:** FR-003 (concurrent matches), NFR-003 (100 concurrent matches = 800 players)

---

### 5. Persistence Service (FastAPI)

**Purpose:** Player progression and inventory management

**Responsibilities:**
- Store/retrieve player stash (equipped + reserve gear)
- Update currency balances (post-match rewards)
- Track player statistics (raids survived, kills, deaths)
- Validate loadout legality before match start
- Apply item losses on death (remove lost items from stash)
- Apply item gains on extraction (add extracted loot)
- Enforce stash size limits
- Maintain transaction log for audit

**Interfaces:**
- REST API (port 8003)
  - GET /players/{id}/stash - retrieve inventory
  - POST /players/{id}/stash/equip - update loadout
  - POST /matches/{id}/results - process match outcome
  - GET /players/{id}/stats - player statistics
- PostgreSQL (read/write)

**Dependencies:**
- PostgreSQL (persistent storage)
- Authentication Service (player identity)
- Match Orchestrator (match results)

**FRs Addressed:** FR-009 (death mechanics), FR-010 (stash system), FR-011 (item loss on death), FR-012 (loot extraction), FR-014 (gear loadout), FR-020 (currency system)

---

### 6. Authentication Service (FastAPI)

**Purpose:** Player account and session management

**Responsibilities:**
- Register new player accounts
- Authenticate login credentials (email + password)
- Issue JWT tokens (24-hour expiry)
- Validate tokens for protected endpoints
- Manage session state in Redis
- Handle password reset flows
- Log authentication events for security audit

**Interfaces:**
- REST API (port 8004)
  - POST /auth/register - create account
  - POST /auth/login - authenticate (returns JWT)
  - POST /auth/validate - token validation
  - POST /auth/logout - invalidate session
- Redis (session cache)
- PostgreSQL (account storage)

**Dependencies:**
- PostgreSQL (user accounts)
- Redis (session tokens)

**FRs Addressed:** FR-015 (account system), NFR-005 (JWT authentication), NFR-006 (password hashing)

---

### 7. Pre-Raid UI Service (React/Node.js)

**Purpose:** Web-based lobby and loadout management interface

**Responsibilities:**
- Serve React SPA for pre-raid screens
- Provide API for loadout selection
- Display player stash with drag-drop UI
- Show player statistics and currency
- Trigger matchmaking queue join
- Display queue status and estimated wait time
- Handle loadout validation client-side

**Interfaces:**
- HTTP server (port 3000) - serves React SPA
- REST API proxy (forwards to backend services)
- WebSocket client (receives queue updates from Matchmaking)

**Dependencies:**
- Persistence Service (stash data)
- Matchmaking Service (queue operations)
- Authentication Service (login flow)

**FRs Addressed:** FR-010 (stash access), FR-014 (loadout selection), FR-015 (account UI), FR-020 (currency display)

---

### 8. Message Broker (RabbitMQ)

**Purpose:** Event-driven communication backbone

**Responsibilities:**
- Route events between services
- Guarantee message delivery (persistent queues)
- Enable pub/sub patterns (fanout exchanges)
- Buffer events during service restarts
- Provide dead-letter queues for failed processing
- Support priority queues for critical events

**Exchanges & Queues:**
- `match.lifecycle` exchange (fanout)
  - Events: match.created, match.started, match.ended
- `player.events` exchange (topic)
  - Events: player.died, player.extracted, player.disconnected
- `matchmaking.ready` queue (direct)

**Interfaces:**
- AMQP protocol (port 5672)
- Management UI (port 15672)

**Dependencies:** None (infrastructure component)

**FRs Addressed:** Enables event-driven architecture for FR-001, FR-003, FR-008, FR-009, FR-022

---

### 9. Cache Layer (Redis)

**Purpose:** High-speed session and state caching

**Responsibilities:**
- Cache player session tokens (24-hour TTL)
- Store matchmaking queue state (sorted sets)
- Cache active match metadata (match IDs, player lists)
- Provide pub/sub for real-time notifications
- Maintain rate limit counters

**Data Structures:**
- `session:{token}` - Hash (player_id, expires_at)
- `queue:matchmaking` - Sorted Set (player_id, timestamp)
- `match:{id}:metadata` - Hash (status, players, server_address)
- `ratelimit:{player_id}:{endpoint}` - Counter (with expiry)

**Interfaces:**
- Redis protocol (port 6379)
- Redis Sentinel (failover)

**Dependencies:** None (infrastructure component)

**FRs Addressed:** Supports NFR-001 (low latency), NFR-005 (session management)

---

### 10. Database (PostgreSQL)

**Purpose:** Persistent storage for all non-ephemeral data

**Responsibilities:**
- Store player accounts (credentials, metadata)
- Store player stash (items, quantities)
- Store currency balances
- Store match history and statistics
- Store audit logs
- Enforce referential integrity
- Provide ACID guarantees for transactions

**Schema (High-Level):**
- `players` table - id, email, password_hash, created_at
- `stashes` table - player_id, item_id, quantity, equipped
- `currencies` table - player_id, currency_type, amount
- `match_history` table - match_id, player_id, outcome, loot_extracted
- `audit_logs` table - event_type, player_id, timestamp, details

**Interfaces:**
- PostgreSQL protocol (port 5432)
- Connection pooling via PgBouncer

**Dependencies:** None (infrastructure component)

**FRs Addressed:** FR-010 (stash persistence), FR-015 (accounts), FR-020 (currency), NFR-008 (data integrity)

---

## Data Architecture

### Entity-Relationship Model

**Core Entities:**

```
Player (1) ──────── (1) Currency
  │
  │ (1:M)
  │
  ▼
StashItem (M) ──── (1) ItemDefinition

Player (M) ──────── (M) Match
  │                  │
  │ through          │
  └─────────────────►MatchParticipant
```

### Database Schema

#### Players Table

```sql
CREATE TABLE players (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    last_login TIMESTAMP,
    is_active BOOLEAN DEFAULT TRUE
);

CREATE INDEX idx_players_email ON players(email);
```

#### Currency Table

```sql
CREATE TABLE currencies (
    player_id UUID PRIMARY KEY REFERENCES players(id) ON DELETE CASCADE,
    rubles INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT positive_balance CHECK (rubles >= 0)
);

CREATE INDEX idx_currency_player ON currencies(player_id);
```

#### ItemDefinition Table

```sql
CREATE TABLE item_definitions (
    id VARCHAR(50) PRIMARY KEY, -- e.g., 'bomb_basic', 'vest_blast'
    name VARCHAR(100) NOT NULL,
    category VARCHAR(20) NOT NULL, -- 'bomb', 'vest', 'consumable'
    tier INTEGER NOT NULL CHECK (tier BETWEEN 1 AND 3),
    value INTEGER NOT NULL, -- base price in rubles
    max_stack INTEGER DEFAULT 1,
    properties JSONB -- bomb stats (range, pierce, etc.)
);
```

#### StashItem Table

```sql
CREATE TABLE stash_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    item_id VARCHAR(50) NOT NULL REFERENCES item_definitions(id),
    quantity INTEGER NOT NULL DEFAULT 1,
    is_equipped BOOLEAN DEFAULT FALSE,
    acquired_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT positive_quantity CHECK (quantity > 0)
);

CREATE INDEX idx_stash_player ON stash_items(player_id);
CREATE INDEX idx_stash_equipped ON stash_items(player_id, is_equipped)
    WHERE is_equipped = TRUE;

-- Prevent duplicate equipped items
CREATE UNIQUE INDEX idx_stash_unique_equipped
    ON stash_items(player_id, item_id)
    WHERE is_equipped = TRUE;
```

#### Match Table

```sql
CREATE TABLE matches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id VARCHAR(50) NOT NULL, -- e.g., 'factory_01'
    status VARCHAR(20) NOT NULL, -- 'initializing', 'active', 'completed', 'aborted'
    started_at TIMESTAMP,
    ended_at TIMESTAMP,
    duration_seconds INTEGER,
    server_address VARCHAR(100), -- Game Server IP:port
    max_players INTEGER DEFAULT 8,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_match_status ON matches(status);
CREATE INDEX idx_match_started ON matches(started_at DESC);
```

#### MatchParticipant Table

```sql
CREATE TABLE match_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    match_id UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    outcome VARCHAR(20), -- 'extracted', 'died', 'disconnected', 'aborted'
    spawn_position JSONB, -- {x, y} grid coordinates
    death_position JSONB, -- {x, y} if died
    loadout JSONB NOT NULL, -- snapshot of equipped items
    loot_extracted JSONB, -- items extracted successfully
    kill_count INTEGER DEFAULT 0,
    damage_dealt INTEGER DEFAULT 0,
    survival_time_seconds INTEGER,
    joined_at TIMESTAMP NOT NULL DEFAULT NOW(),
    left_at TIMESTAMP,
    CONSTRAINT unique_match_player UNIQUE (match_id, player_id)
);

CREATE INDEX idx_participant_match ON match_participants(match_id);
CREATE INDEX idx_participant_player ON match_participants(player_id);
CREATE INDEX idx_participant_outcome ON match_participants(outcome);
```

#### AuditLog Table

```sql
CREATE TABLE audit_logs (
    id BIGSERIAL PRIMARY KEY,
    event_type VARCHAR(100) NOT NULL, -- 'player.died', 'item.extracted'
    player_id UUID REFERENCES players(id),
    match_id UUID REFERENCES matches(id),
    details JSONB,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    severity VARCHAR(20) -- 'info', 'warning', 'error'
);

CREATE INDEX idx_audit_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX idx_audit_event_type ON audit_logs(event_type);
CREATE INDEX idx_audit_player ON audit_logs(player_id) WHERE player_id IS NOT NULL;
```

### Data Flow Diagrams

**Pre-Match Data Flow (Read Path):**

```
Login:
Player → Auth Service → players table (email lookup)
                     → Redis session cache (24h TTL)

Loadout Selection:
Player → Pre-Raid UI → Persistence Service
                    → stash_items table (WHERE player_id AND is_equipped)
                    → Return equipped + inventory

Queue Join:
Player → Matchmaking → Redis (check active match)
                    → Redis sorted set (queue:matchmaking)
```

**Match Data Flow (Write Path):**

```
Match Creation:
Matchmaking → Match Orchestrator → matches table (INSERT, status='initializing')
                                → match_participants (INSERT for all players)
                                → Snapshot loadout into JSONB

Match Start:
Game Server → matches table (UPDATE status='active', started_at=NOW())

Match End:
Game Server → RabbitMQ (match.ended event)
Match Orchestrator → Persistence Service (POST /matches/{id}/results)

Persistence Service (TRANSACTION):
  IF outcome='extracted':
    - INSERT INTO stash_items (loot_extracted items)
    - UPDATE currencies (add loot value)
  IF outcome='died':
    - DELETE FROM stash_items (equipped items from loadout)
  - INSERT INTO audit_logs (item.extracted, item.lost, player.died)
  - UPDATE match_participants (outcome, stats)
COMMIT
```

### Transaction Boundaries

**Critical Transactions (ACID Required):**

```sql
-- Post-Match Item Processing
BEGIN TRANSACTION;

-- Process player A (extracted with loot)
INSERT INTO stash_items (player_id, item_id, quantity)
VALUES ('player-a-id', 'vest_blast', 1);

UPDATE currencies
SET rubles = rubles + 5000
WHERE player_id = 'player-a-id';

-- Process player B (died, lost gear)
DELETE FROM stash_items
WHERE player_id = 'player-b-id'
  AND item_id IN ('bomb_basic', 'vest_basic');

-- Audit trail
INSERT INTO audit_logs (event_type, match_id, details)
VALUES ('match.processed', 'match-123', '{"players": 8}');

COMMIT;
```

**Why ACID Matters:**
- Partial failures would cause item duplication or loss
- ACID guarantees prevent exploits (disconnect to keep loot)
- Rollback ensures consistent state

---

## API Design

### API Architecture

**Protocol:** REST for backend services, WebSocket for real-time game

**Versioning:** URL-based (`/api/v1/...`)

**Authentication:** JWT via Bearer token

**Response Format:** JSON with standard envelope

```json
{
  "success": true,
  "data": { /* actual response */ },
  "error": null,
  "timestamp": "2025-12-26T12:34:56Z"
}
```

### Key API Endpoints

#### Authentication Service (Port 8004)

**POST /api/v1/auth/register**

```
Request:
{
  "email": "player@example.com",
  "username": "Bombmaster",
  "password": "SecureP@ss123"
}

Response (201):
{
  "success": true,
  "data": {
    "player_id": "123e4567-e89b-12d3-a456-426614174000",
    "username": "Bombmaster",
    "created_at": "2025-12-26T12:00:00Z"
  }
}
```

**POST /api/v1/auth/login**

```
Request:
{
  "email": "player@example.com",
  "password": "SecureP@ss123"
}

Response (200):
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIs...",
    "expires_at": "2025-12-27T12:00:00Z",
    "player": {
      "id": "123e...",
      "username": "Bombmaster"
    }
  }
}
```

---

#### Persistence Service (Port 8003)

**GET /api/v1/players/{player_id}/stash**

```
Authorization: Bearer {token}

Response (200):
{
  "success": true,
  "data": {
    "equipped": [
      {
        "item_id": "bomb_basic",
        "name": "Basic Bomb",
        "quantity": 3
      },
      {
        "item_id": "vest_basic",
        "name": "Basic Vest",
        "quantity": 1
      }
    ],
    "inventory": [
      {
        "item_id": "bomb_pierce",
        "name": "Piercing Bomb",
        "quantity": 1,
        "value": 2500
      }
    ],
    "total_slots": 20,
    "used_slots": 5
  }
}
```

**POST /api/v1/players/{player_id}/stash/equip**

```
Authorization: Bearer {token}

Request:
{
  "bomb_type": "bomb_basic",
  "bomb_quantity": 3,
  "vest_type": "vest_blast"
}

Response (200):
{
  "success": true,
  "data": {
    "loadout": {
      "bomb_type": "bomb_basic",
      "bomb_quantity": 3,
      "vest_type": "vest_blast"
    },
    "total_value": 7500
  }
}
```

---

#### Matchmaking Service (Port 8002)

**POST /api/v1/queue**

```
Authorization: Bearer {token}

Request:
{
  "loadout": {
    "bomb_type": "bomb_basic",
    "bomb_quantity": 3,
    "vest_type": "vest_basic"
  }
}

Response (202):
{
  "success": true,
  "data": {
    "queue_position": 3,
    "estimated_wait_seconds": 15,
    "queue_id": "q-789abc"
  }
}
```

**DELETE /api/v1/queue/{player_id}**

```
Authorization: Bearer {token}

Response (200):
{
  "success": true,
  "data": {
    "message": "Removed from queue"
  }
}
```

---

#### Game Server WebSocket API (Port 9001)

**WebSocket Connection:**

```
WS /game
Upgrade: websocket
Authorization: Bearer {token}
```

**Client → Server Messages (MessagePack):**

```json
// Player movement
{
  "type": "move",
  "direction": "north",
  "sequence": 42
}

// Place bomb
{
  "type": "place_bomb",
  "position": {"x": 5, "y": 7},
  "sequence": 43
}

// Request extraction
{
  "type": "extract",
  "extraction_point_id": "ext_1",
  "sequence": 44
}
```

**Server → Client Messages (MessagePack):**

```json
// State update (broadcast every 50ms)
{
  "type": "state_update",
  "tick": 1234,
  "players": [
    {"id": "123e...", "pos": {"x": 5, "y": 7}, "alive": true}
  ],
  "bombs": [
    {"id": "b-001", "pos": {"x": 3, "y": 4}, "timer_ms": 1500}
  ],
  "time_remaining_ms": 240000
}

// Player death
{
  "type": "player_died",
  "player_id": "456f...",
  "killer_id": "123e...",
  "position": {"x": 8, "y": 9}
}
```

---

## NFR Coverage

### NFR-001: Input Responsiveness

**Requirement:** Input latency < 100ms, server tick rate 20 ticks/second

**Solution:**
- Client-side prediction (Bevy renders predicted movement immediately)
- WebSocket binary protocol (MessagePack reduces serialization overhead)
- 20Hz server tick rate (50ms intervals, Tokio async runtime)

**Implementation:**

```rust
// Game Server 50ms tick loop
let mut interval = tokio::time::interval(Duration::from_millis(50));
loop {
    interval.tick().await;
    process_input_queue(&mut game_state);
    update_simulation(&mut game_state, 50);
    broadcast_state_update(&game_state, &connections).await;
}
```

**Validation:** Log timestamp delta keypress → visual feedback, target p95 < 100ms

---

### NFR-002: Bomb Timer Accuracy

**Requirement:** Timer deviation < 50ms across clients

**Solution:**
- Server-authoritative detonation (bomb timer tracked server-side)
- Deterministic timing (fixed 50ms ticks, integer `ticks_remaining`)
- Client prediction with server correction

**Implementation:**

```rust
struct Bomb {
    ticks_remaining: u32,  // Decremented each tick
    owner_id: PlayerId,
}

fn update_bombs(bombs: &mut Vec<Bomb>) {
    for bomb in bombs.iter_mut() {
        bomb.ticks_remaining = bomb.ticks_remaining.saturating_sub(1);
        if bomb.ticks_remaining == 0 {
            detonate_bomb(bomb);
        }
    }
}
```

**Validation:** Record server detonation timestamp, compare across clients (< 50ms window)

---

### NFR-003: Match Capacity

**Requirement:** 10+ concurrent matches (80 players), CPU < 70%

**Solution:**
- Process isolation (each match = separate Game Server process)
- Match Orchestrator limits concurrent matches (max 12 per host)
- Lightweight state (grid-based, no physics engine)

**Validation:** Load test 12 concurrent matches, monitor CPU usage

---

### NFR-004: Authoritative Server

**Requirement:** Server validates all game-critical state

**Solution:**
- Command-based input model (clients send commands, server validates)
- Server rejects invalid moves (wall-clipping, teleportation)
- Loot spawns not revealed until picked up

**Implementation:**

```rust
fn handle_command(cmd: PlayerCommand, state: &mut GameState) -> Result<()> {
    match cmd {
        PlayerCommand::Move(dir) => {
            let target = state.player_position(player_id).step(dir);
            if !state.is_walkable(target) {
                return Err(CommandError::InvalidMove);
            }
            state.move_player(player_id, target);
            Ok(())
        }
    }
}
```

**Validation:** Test client sending invalid commands, verify 100% rejected

---

### NFR-005: Input Validation

**Requirement:** Rate limiting, bounds checking, inventory validation

**Solution:**
- Per-player rate limits (100ms between moves, 1s bomb cooldown)
- Grid bounds checking (reject off-grid positions)
- Inventory authority (server doesn't trust client for item existence)

**Validation:** Bot client spamming commands, verify rate limits enforced

---

### NFR-006: MVP Uptime Target

**Requirement:** 90% uptime during testing windows

**Solution:**
- Docker restart policies (auto-restart on failure)
- Service independence (matches complete even if matchmaking down)
- Health monitoring (/health endpoints, Traefik routing)

**Validation:** Track uptime over 1-week window, target ≥ 90%

---

### NFR-007: Crash Recovery

**Requirement:** Server crash doesn't corrupt player stash

**Solution:**
- Database persistence (stash written to PostgreSQL immediately)
- Pre-raid loadout snapshots (immutable in match_participants)
- Crash = treat as death (rollback to pre-raid state)

**Implementation:**

```python
async def recover_crashed_matches():
    crashed = await db.fetch("SELECT * FROM match_participants WHERE ...")
    for row in crashed:
        await process_death_outcome(row['player_id'], row['loadout'])
```

**Validation:** Kill Game Server mid-match, verify stash not corrupted

---

### NFR-008: Control Scheme

**Requirement:** Responsive keyboard controls (WASD, Spacebar, E)

**Solution:**
- Bevy input handling (event-driven)
- Immediate visual feedback (same frame)
- Single-key actions (no combos)

**Validation:** User testing, 100% testers understand controls within 30s

---

### NFR-009: Visual Clarity

**Requirement:** No mystery deaths, blast radius visible

**Solution:**
- Blast radius visualization (semi-transparent overlay)
- Gear visual identification (sprite changes by vest type)
- Death feedback (3s freeze-frame, "Killed by [Player]'s [Bomb]")

**Validation:** User testing, 0 mystery deaths in 20 test raids

---

### NFR-010: Structured Logging

**Requirement:** JSON logs with timestamps, player IDs, match IDs

**Solution:**
- Structured logging (`tracing` in Rust, `structlog` in Python)
- Docker stdout → Loki aggregation
- Queryable by match_id or player_id

**Implementation:**

```rust
info!(
    event_type = "bomb.placed",
    match_id = match_id,
    player_id = player_id,
    "Player placed bomb"
);
```

**Validation:** Query logs for specific match, verify all events captured

---

### NFR-011: Match Replay Data

**Requirement:** Event stream for debugging

**Solution:**
- Event sourcing (Game Server emits events to audit_logs)
- Events include state deltas (before/after positions)
- Replay tool reconstructs state frame-by-frame

**Validation:** Record match, verify can reconstruct final state

---

### NFR-012: Platform Support

**Requirement:** Windows, macOS, Linux desktop

**Solution:**
- Bevy engine (cross-platform)
- GitHub Actions CI (build artifacts for all platforms)
- 2D rendering (low GPU requirements)

**Validation:** Test on 3 platforms, verify 60 FPS on integrated GPU

---

## Security Architecture

### Authentication (JWT)

**Token Structure:**

```json
{
  "sub": "123e4567-e89b-12d3-a456-426614174000",
  "username": "Bombmaster",
  "type": "player",
  "iat": 1703592000,
  "exp": 1703678400
}
```

**Signing Algorithm:** HMAC-SHA256

**Secret Management:** Environment variable (`JWT_SECRET_KEY`), 256-bit random

**Validation Flow:**

```python
def verify_token(token: str) -> dict:
    payload = jwt.decode(token, SECRET_KEY, algorithms=["HS256"])
    if payload["exp"] < datetime.utcnow().timestamp():
        raise JWTError("Token expired")
    return payload
```

---

### Authorization (RBAC)

**Roles:**
- **Player:** Standard user, own resources only
- **Service:** Internal service-to-service
- **Admin:** Developer/moderator (v2)

**Ownership Enforcement:**

```python
async def verify_stash_ownership(player_id: str, current_user: str):
    if player_id != current_user:
        raise HTTPException(status_code=403)
    return player_id
```

---

### Data Encryption

**At Rest:**
- Password hashing: bcrypt (cost factor 12)
- Database: OS-level disk encryption

**In Transit:**
- HTTPS/TLS 1.3 (Traefik reverse proxy)
- WebSocket Secure (WSS)

**TLS Configuration:**

```yaml
services:
  traefik:
    command:
      - --entrypoints.websecure.address=:443
      - --certificatesresolvers.myresolver.acme.email=admin@bombout.game
```

---

### Security Best Practices

**1. Input Validation (Pydantic):**

```python
class RegisterRequest(BaseModel):
    email: EmailStr
    username: str
    password: str

    @validator('password')
    def validate_password(cls, v):
        if len(v) < 8:
            raise ValueError("Password must be at least 8 characters")
        return v
```

**2. SQL Injection Prevention:**

```python
# ✓ SAFE: Parameterized query
await db.execute("SELECT * FROM players WHERE email = $1", email)

# ✗ UNSAFE: Never do this
await db.execute(f"SELECT * FROM players WHERE email = '{email}'")
```

**3. Rate Limiting:**

```python
@app.post("/api/v1/auth/login")
@limiter.limit("5/minute")  # Max 5 login attempts per minute
async def login(request: Request, credentials: LoginRequest):
    ...
```

**4. Security Headers:**

```python
response.headers["Content-Security-Policy"] = "default-src 'self'"
response.headers["X-Content-Type-Options"] = "nosniff"
response.headers["X-Frame-Options"] = "DENY"
```

---

## Scalability & Performance

### Scaling Strategy

**MVP: Vertical Scaling (Single Host)**

- 8-core CPU, 16 GB RAM, 100 GB SSD
- 12 concurrent matches max (96 players)
- CPU usage: ~30% (2-3 cores)

**v2: Horizontal Scaling (Kubernetes)**

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: matchmaking-hpa
spec:
  scaleTargetRef:
    kind: Deployment
    name: matchmaking-service
  minReplicas: 2
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          averageUtilization: 70
```

---

### Performance Optimization

**Query Optimization:**

```sql
-- Critical indexes
CREATE INDEX idx_players_email ON players(email);
CREATE INDEX idx_stash_player_equipped ON stash_items(player_id, is_equipped);
CREATE INDEX idx_participants_player ON match_participants(player_id, outcome);
```

**N+1 Query Prevention:**

```python
# ✓ GOOD: Single query with JOIN
results = await db.fetch("""
    SELECT m.id, json_agg(mp.*) AS participants
    FROM matches m
    LEFT JOIN match_participants mp ON m.id = mp.match_id
    GROUP BY m.id
""")
```

**Connection Pooling:**

```python
pool = await create_pool(
    dsn="postgresql://...",
    min_size=5,
    max_size=20,
    command_timeout=10.0
)
```

---

### Caching Strategy

**Redis Cache Layers:**

```python
# Session cache (24h TTL)
await redis.setex(f"session:{token}", 86400, player_id)

# Queue state (no TTL)
await redis.zadd("queue:matchmaking", {player_id: time.time()})

# Match metadata (10min TTL)
await redis.setex(f"match:{id}:meta", 600, match_data)
```

**What NOT to Cache:**
- Stash data (economic integrity)
- Currency balances (prevent exploits)

---

### Load Balancing

**Traefik Configuration:**

```yaml
services:
  traefik:
    command:
      - --providers.docker=true
      - --entrypoints.websecure.address=:443

  matchmaking-service:
    labels:
      - "traefik.http.routers.matchmaking.rule=PathPrefix(`/api/v1/queue`)"
      - "traefik.http.services.matchmaking.loadbalancer.healthcheck.path=/health"
```

**Algorithm:** Round-robin (stateless services)

---

## Reliability & Availability

### High Availability (MVP: 90% Uptime)

**Docker Restart Policies:**

```yaml
services:
  matchmaking-service:
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8002/health"]
      interval: 30s
      retries: 3
```

**Service Independence:**
- Matchmaking down → existing matches complete
- Persistence down → results queued in RabbitMQ
- Auth down → logged-in players continue

---

### Disaster Recovery

**RPO:** 24 hours (daily backups)

**RTO:** 2 hours (manual recovery)

**Backup Script:**

```bash
#!/bin/bash
# Daily at 2 AM UTC
BACKUP_FILE="/backups/bombout_db_$(date +%Y%m%d_%H%M%S).sql"
docker exec postgres pg_dump -U bombout bombout_db > ${BACKUP_FILE}
gzip ${BACKUP_FILE}
find /backups -name "*.gz" -mtime +7 -delete
```

**Restore Procedure:**

```bash
gunzip -c /backups/bombout_db_20251226.sql.gz | \
docker exec -i postgres psql -U bombout bombout_db
```

---

### Monitoring & Alerting

**Metrics Tracked:**

- **System:** CPU, memory, disk usage
- **Application:** Match count, queue size, query latency
- **Business:** DAU, survival rate, economy balance

**Structured Logging:**

```json
{
  "timestamp": "2025-12-26T12:34:56Z",
  "level": "info",
  "service": "matchmaking-service",
  "event_type": "match.created",
  "match_id": "m-abc123"
}
```

**Log Aggregation:** Docker → Loki → Grafana

**Alerting Thresholds:**

```yaml
# Critical: Database Down
alert: DatabaseDown
expr: up{job="postgres"} == 0
for: 1m
severity: critical

# Warning: High CPU
alert: HighCPU
expr: container_cpu_usage > 0.8
for: 5m
severity: warning
```

---

## Development & Deployment

### Code Organization

```
thermite/
├── game-client/          # Bevy desktop client (Rust)
├── game-server/          # Authoritative server (Rust)
├── backend-services/     # FastAPI microservices (Python)
│   ├── matchmaking/
│   ├── persistence/
│   ├── authentication/
│   └── match-orchestrator/
├── pre-raid-ui/          # React SPA (TypeScript)
├── infrastructure/       # Docker Compose, K8s
├── database/migrations/  # SQL migrations
├── docs/                 # Architecture, PRD
└── scripts/              # Backup, deploy
```

---

### Testing Strategy

**Coverage Target:** 80%

**Unit Testing:**

```rust
#[test]
fn test_move_player_valid() {
    let mut state = GameState::new(10, 10);
    state.add_player("p1", GridPos { x: 5, y: 5 });
    let result = state.move_player("p1", Direction::North);
    assert!(result.is_ok());
}
```

**Integration Testing:**

```python
@pytest.mark.integration
async def test_stash_persistence():
    async with get_db_connection() as conn:
        await conn.execute("INSERT INTO stash_items ...")
        result = await conn.fetchone("SELECT ...")
        assert result['quantity'] == 3
```

**E2E Testing:**

```python
@pytest.mark.e2e
async def test_complete_match_flow():
    # 1. Login 8 players
    # 2. Equip loadouts
    # 3. Join queue
    # 4. Verify match created
    # 5. Simulate match completion
    # 6. Verify stash updated
```

**Performance Testing (Locust):**

```python
class MatchmakingUser(HttpUser):
    @task
    def join_queue(self):
        self.client.post("/api/v1/queue", ...)
```

---

### CI/CD Pipeline

**GitHub Actions:**

```yaml
jobs:
  test-rust:
    steps:
      - run: cargo test --verbose
      - run: cargo tarpaulin --out Xml

  test-python:
    steps:
      - run: poetry install
      - run: pytest --cov

  build-images:
    needs: [test-rust, test-python]
    steps:
      - run: docker build -t thermite/game-server:latest
```

**Deployment:**

```bash
# Manual deploy (MVP)
docker compose pull
docker compose up -d
docker compose exec persistence-service python migrate.py
```

---

### Environments

| Feature | Development | Staging | Production |
|---------|-------------|---------|------------|
| Docker Compose | ✓ | ✓ | ✓ |
| PostgreSQL | Local | RDS | RDS |
| RabbitMQ | Local | CloudAMQP | CloudAMQP |
| TLS | Self-signed | Let's Encrypt | Let's Encrypt |

**Configuration:**

```bash
# .env.development
DATABASE_URL=postgresql://dev:dev@localhost/bombout_dev
JWT_SECRET_KEY=dev-secret-not-for-production

# .env.production (secure secrets)
DATABASE_URL=postgresql://prod_user:***@rds.amazonaws.com/bombout
JWT_SECRET_KEY=***  # 256-bit random key
```

---

## Traceability

### FR → Component Mapping

| FR ID | FR Name | Components | Implementation |
|-------|---------|------------|----------------|
| FR-001 | Match Duration (5-8 min) | Game Server, Match Orchestrator | Timer enforced in Game Server tick loop |
| FR-002 | Player Spawns | Match Orchestrator, Game Server | Orchestrator assigns positions, Server validates |
| FR-005 | Grid Movement | Game Client, Game Server | Client predicts, Server validates |
| FR-006 | Bomb Placement | Game Server | Server validates cooldown, tile empty |
| FR-007 | Blast Mechanics | Game Server | Server calculates blast pattern, client renders |
| FR-008 | Extraction | Game Server, Pre-Raid UI | Server validates position + timer |
| FR-009 | Death Handling | Game Server, Persistence Service | Server detects death, Persistence removes items |
| FR-010 | Stash System | Persistence Service, Pre-Raid UI | PostgreSQL storage, React drag-drop UI |
| FR-011 | Item Loss on Death | Persistence Service | Transaction removes equipped items |
| FR-012 | Loot Extraction | Game Server, Persistence Service | Server validates extraction, Persistence adds items |
| FR-014 | Gear Loadout | Pre-Raid UI, Persistence Service | React loadout selector, validated server-side |
| FR-015 | Account System | Authentication Service | JWT tokens, PostgreSQL user accounts |
| FR-020 | Currency System | Persistence Service | PostgreSQL currencies table, transactional updates |
| FR-021 | Grid-Based Map | Game Server, Game Client | Server owns map state, client renders |
| FR-022 | Map Zones | Game Server | Server defines loot spawn tiers |

---

### NFR → Solution Mapping

| NFR ID | NFR Name | Solution | Validation |
|--------|----------|----------|------------|
| NFR-001 | Input Responsiveness | Client-side prediction, 20Hz server tick | p95 latency < 100ms |
| NFR-002 | Bomb Timer Accuracy | Deterministic tick-based timers | < 50ms deviation across clients |
| NFR-003 | Match Capacity | Process isolation, 12 concurrent matches | CPU < 70% under load |
| NFR-004 | Authoritative Server | Command validation, server as truth | 100% invalid commands rejected |
| NFR-005 | Input Validation | Rate limiting, bounds checking | Rate limits enforced |
| NFR-006 | MVP Uptime | Docker restart policies, health checks | 90% uptime over 1 week |
| NFR-007 | Crash Recovery | Database transactions, pre-raid snapshots | 0% stash corruption |
| NFR-008 | Control Scheme | Bevy input handling, single-key actions | 100% users understand in 30s |
| NFR-009 | Visual Clarity | Blast radius overlays, death feedback UI | 0 mystery deaths in testing |
| NFR-010 | Structured Logging | JSON logs, Loki aggregation | All events queryable |
| NFR-011 | Match Replay | Event sourcing to audit_logs | Reconstruct final state |
| NFR-012 | Platform Support | Bevy cross-platform, GitHub Actions CI | 60 FPS on all platforms |

---

## Trade-offs

### Decision: Event-Driven Microservices

**Trade-off:**
- ✓ **Gain:** Services scale independently, loose coupling, fault isolation
- ✗ **Lose:** Distributed transactions harder, operational complexity

**Rationale:** Benefits outweigh costs for Level 4 project scale. Event sourcing simplifies coordination.

---

### Decision: Rust for Game Server

**Trade-off:**
- ✓ **Gain:** Predictable performance, memory safety, zero GC pauses
- ✗ **Lose:** Slower iteration vs. Node.js/Python, steeper learning curve

**Rationale:** Real-time 20Hz tick rate requires guaranteed latency. Rust prevents common bugs.

---

### Decision: PostgreSQL over NoSQL

**Trade-off:**
- ✓ **Gain:** ACID transactions, data integrity, proven reliability
- ✗ **Lose:** Vertical scaling limits (acceptable for MVP)

**Rationale:** Economic integrity critical (prevent item duplication). ACID non-negotiable.

---

### Decision: Docker Compose (MVP) → Kubernetes (v2)

**Trade-off:**
- ✓ **Gain (MVP):** Simplicity, low operational overhead, fast iteration
- ✗ **Lose:** Horizontal scaling limited to ~20 matches/host

**Rationale:** MVP needs to ship quickly. Single-host sufficient for 80 players. Kubernetes migration path clear.

---

### Decision: JWT (No Refresh Tokens in MVP)

**Trade-off:**
- ✓ **Gain:** Simplicity, stateless validation
- ✗ **Lose:** User must re-login after 24h (UX friction)

**Rationale:** Refresh tokens add complexity (rotation, revocation). MVP can tolerate re-login.

---

### Decision: Client-Side Prediction

**Trade-off:**
- ✓ **Gain:** Responsive input (< 100ms perceived latency)
- ✗ **Lose:** Mispredictions require rollback (visual "rubber-banding")

**Rationale:** Grid-based movement minimizes mispredictions. Responsive feel critical for skill expression.

---

## Appendix

### Glossary

- **Authoritative Server:** Server owns game truth, clients cannot modify
- **Client-Side Prediction:** Client optimistically renders actions before server confirmation
- **ECS:** Entity-Component-System (Bevy architecture pattern)
- **FIFO:** First-In-First-Out (matchmaking queue ordering)
- **Grid-Based:** Tile-based movement (discrete positions)
- **JWT:** JSON Web Token (authentication)
- **MessagePack:** Binary serialization (faster than JSON)
- **MVP:** Minimum Viable Product
- **NFR:** Non-Functional Requirement
- **RBAC:** Role-Based Access Control
- **RPO:** Recovery Point Objective (max acceptable data loss)
- **RTO:** Recovery Time Objective (max downtime)
- **Tick Rate:** Server update frequency (20Hz = 50ms)
- **WebSocket:** Bi-directional real-time protocol

---

### References

- **PRD:** [prd-thermite-2025-12-25.md](prd-thermite-2025-12-25.md)
- **Bevy Documentation:** https://bevyengine.org/learn/
- **FastAPI Documentation:** https://fastapi.tiangolo.com/
- **PostgreSQL 16 Docs:** https://www.postgresql.org/docs/16/
- **RabbitMQ Tutorials:** https://www.rabbitmq.com/tutorials
- **Tokio Guide:** https://tokio.rs/tokio/tutorial

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-12-26 | System Architect | Initial architecture document |

---

**End of Architecture Document**

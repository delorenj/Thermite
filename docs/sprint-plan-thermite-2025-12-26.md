# Sprint Plan: Thermite (BOMBOUT)

**Project:** Thermite
**Date:** 2025-12-26
**Scrum Master:** Jarad DeLorenzo
**Project Level:** 4 (40+ stories)
**Total Stories:** 36 stories
**Total Points:** 140 points
**Planned Sprints:** 5 sprints (2 weeks each)
**Target Completion:** 10 weeks from sprint start

**Related Documents:**
- [Product Requirements Document](prd-thermite-2025-12-25.md) - 22 FRs, 12 NFRs
- [Architecture Document](architecture-thermite-2025-12-26.md) - Technical design
- [Gate Check Report](gate-check-report-thermite-2025-12-26.md) - Validation

---

## Executive Summary

This sprint plan breaks down BOMBOUT's MVP into 36 implementable user stories across 5 two-week sprints. The plan prioritizes infrastructure foundation, then progressive feature delivery culminating in a fully playable hybrid extraction shooter.

**Key Metrics:**
- **Total Stories:** 36 (32 feature + 4 infrastructure)
- **Total Points:** 140
- **Sprints:** 5 × 2-week sprints
- **Team:** 1 senior developer
- **Capacity:** 30 points per sprint (target 27 with buffer)
- **Velocity:** ~28 points per sprint average
- **Completion Target:** 10 weeks

**Sprint Progression:**
1. **Sprint 1 (Foundation):** Infrastructure, database, map system
2. **Sprint 2 (Combat):** Grid movement, bombs, death mechanics
3. **Sprint 3 (Economy):** Loadout, stash, shop, currency
4. **Sprint 4 (Extraction):** Matchmaking, raid lifecycle, extraction
5. **Sprint 5 (Integration):** Loot system, polish, final features

---

## Table of Contents

1. [Story Inventory](#story-inventory)
2. [Sprint Allocation](#sprint-allocation)
3. [Epic Traceability](#epic-traceability)
4. [Requirements Coverage](#requirements-coverage)
5. [Risks and Mitigation](#risks-and-mitigation)
6. [Dependencies](#dependencies)
7. [Definition of Done](#definition-of-done)
8. [Next Steps](#next-steps)

---

## Story Inventory

All 36 stories with detailed specifications, acceptance criteria, and technical implementation notes.

---

### Infrastructure Stories

#### STORY-000: Development Environment Setup

**Epic:** Infrastructure
**Priority:** Must Have
**Points:** 3

**User Story:**
As a developer
I want a configured development environment
So that I can build and run all services locally

**Acceptance Criteria:**
- [ ] Docker Compose configuration runs all services (PostgreSQL, Redis, RabbitMQ)
- [ ] Environment variables documented in `.env.example`
- [ ] README with setup instructions (prerequisites, commands, verification)
- [ ] Local development works on macOS, Linux, Windows (WSL2)
- [ ] Services auto-restart on code changes (hot reload where applicable)

**Technical Notes:**
- **Services:** PostgreSQL 16, Redis 7, RabbitMQ 3.13, Traefik 2.10
- **Tools:** Docker Compose, mise for task management
- **Config:** `.env` for local, `.env.example` as template
- **Validation:** `mise run health-check` verifies all services running

**Dependencies:** None (foundational)

---

#### STORY-INF-001: Database Schema and Migrations

**Epic:** Infrastructure
**Priority:** Must Have
**Points:** 5

**User Story:**
As a developer
I want a versioned database schema with migrations
So that data persistence is reliable and evolvable

**Acceptance Criteria:**
- [ ] All 7 tables created: players, currencies, item_definitions, stash_items, matches, match_participants, audit_logs
- [ ] Foreign key constraints enforce referential integrity
- [ ] CHECK constraints prevent invalid data (positive balance, positive quantity)
- [ ] Indexes created for common queries (player_id, match_id, email)
- [ ] Migration tool set up (Alembic or sqlx migrate)
- [ ] Rollback capability tested
- [ ] Seed data for development (test items, maps)

**Technical Notes:**
- **Schema:** From architecture document (lines 676-800)
- **Tool:** `sqlx migrate` for Rust or `alembic` for Python
- **Validation:** Run migrations up/down, verify constraints
- **Seed Data:** Basic items (bomb_basic, vest_basic), 1 map template

**Dependencies:** STORY-000 (dev environment)

---

#### STORY-INF-002: Docker Compose Deployment Setup

**Epic:** Infrastructure
**Priority:** Must Have
**Points:** 3

**User Story:**
As a developer
I want to deploy all services with a single command
So that deployment is simple and consistent

**Acceptance Criteria:**
- [ ] `docker-compose.yml` defines all services (Game Server, Matchmaking, Persistence, Auth, Pre-Raid UI, Traefik)
- [ ] Services configured with restart policies (unless-stopped)
- [ ] Health checks configured for all services
- [ ] Network isolation (internal network for service-to-service)
- [ ] Volume mounts for persistent data (PostgreSQL, logs)
- [ ] `docker compose up -d` starts entire stack
- [ ] `docker compose logs -f` works for debugging

**Technical Notes:**
- **Services:** 10 containers total (see architecture diagram)
- **Networking:** Custom bridge network, Traefik as reverse proxy
- **Volumes:** `postgres_data`, `rabbitmq_data`, `logs`
- **Ports:** Traefik :443, Game Server :9001, services internal

**Dependencies:** STORY-000

---

#### STORY-INF-003: CI/CD Pipeline with GitHub Actions

**Epic:** Infrastructure
**Priority:** Must Have
**Points:** 5

**User Story:**
As a developer
I want automated testing and builds on every commit
So that regressions are caught early

**Acceptance Criteria:**
- [ ] GitHub Actions workflow on push to `main` and PRs
- [ ] Rust tests run (game-client, game-server) with coverage report
- [ ] Python tests run (backend services) with pytest coverage
- [ ] Linting enforced (clippy for Rust, ruff for Python)
- [ ] Type checking (mypy for Python)
- [ ] Docker images built for all services
- [ ] Test coverage badge in README
- [ ] Failed builds block merge

**Technical Notes:**
- **Rust:** `cargo test --verbose`, `cargo clippy`, `cargo tarpaulin`
- **Python:** `pytest --cov`, `ruff check`, `mypy`
- **Cache:** GitHub Actions cache for Rust/Python dependencies
- **Artifacts:** Test coverage reports, Docker images (tagged by commit SHA)

**Dependencies:** STORY-000, STORY-INF-001 (schema needed for integration tests)

---

### EPIC-001: Core Combat & Death System

#### STORY-001: Grid-Based Player Movement

**Epic:** EPIC-001: Core Combat & Death System
**Priority:** Must Have
**Points:** 5

**User Story:**
As a player
I want to move on a grid with responsive controls
So that I can position myself tactically

**Acceptance Criteria:**
- [ ] Player moves one tile per WASD keypress (4-directional)
- [ ] Client-side prediction renders movement immediately
- [ ] Server validates movement and rejects invalid moves (walls, other players)
- [ ] Input latency < 100ms (NFR-001)
- [ ] Movement synchronized across all clients via WebSocket
- [ ] Player cannot clip through walls or move off-grid

**Technical Notes:**
- **Client (Bevy):** Bevy input system, ECS for player entity, optimistic update
- **Server (Rust):** Command queue, grid collision detection, state broadcast
- **Protocol:** WebSocket MessagePack, movement command includes sequence number
- **Sync:** Server sends authoritative position every tick (50ms)

**Components:** Game Client (Bevy), Game Server (Rust/Tokio)

**Dependencies:** STORY-000 (dev environment), STORY-028 (grid data structure)

---

#### STORY-002: Bomb Placement Mechanics

**Epic:** EPIC-001
**Priority:** Must Have
**Points:** 3

**User Story:**
As a player
I want to place bombs on tiles
So that I can control space and threaten enemies

**Acceptance Criteria:**
- [ ] Spacebar places bomb on player's current tile
- [ ] Cannot place bomb on occupied tile (wall or existing bomb)
- [ ] Bomb count limited by player stats (default 1 active bomb)
- [ ] Cooldown between placements (1 second, enforced server-side)
- [ ] Bomb remains on tile after player moves away
- [ ] Visual feedback on successful/failed placement

**Technical Notes:**
- **Server:** Bomb entity struct with position, timer, owner_id
- **Validation:** Check tile empty, check cooldown, check bomb count
- **State:** Server maintains bomb list, broadcasts new bombs
- **Client:** Render bomb sprite on tile, play placement sound

**Components:** Game Server, Game Client

**Dependencies:** STORY-001 (movement)

---

#### STORY-003: Bomb Detonation and Blast Propagation

**Epic:** EPIC-001
**Priority:** Must Have
**Points:** 5

**User Story:**
As a player
I want bombs to detonate with clear blast patterns
So that I can predict and use them strategically

**Acceptance Criteria:**
- [ ] Bomb detonates after 3-second timer (configurable)
- [ ] Blast extends in 4 cardinal directions (up/down/left/right)
- [ ] Blast range based on bomb stats (default 2 tiles)
- [ ] Blast stops at walls or destructible blocks
- [ ] Blast damages all players in affected tiles
- [ ] Visual/audio feedback clearly shows blast area
- [ ] Deterministic timing (< 50ms deviation, NFR-002)

**Technical Notes:**
- **Algorithm:** BFS from bomb position in 4 directions
- **Timer:** Tick-based countdown (integer `ticks_remaining`)
- **Damage:** 100 HP for basic bomb
- **Render:** Explosion animation, screen shake, audio
- **Sync:** Server broadcasts detonation event to all clients

**Components:** Game Server, Game Client

**Dependencies:** STORY-002 (bombs), STORY-004 (health system)

---

#### STORY-004: Player Health and Damage System

**Epic:** EPIC-001
**Priority:** Must Have
**Points:** 3

**User Story:**
As a player
I want a health system that responds to damage
So that combat has stakes and skill expression

**Acceptance Criteria:**
- [ ] Players have health stat (default 100 HP)
- [ ] Bomb blasts deal damage (basic bomb = 100 HP)
- [ ] Health displayed in UI (health bar or numeric)
- [ ] Damage calculation considers gear stats (vest adds HP)
- [ ] Zero health triggers death event
- [ ] Invulnerability frames prevent multi-hit exploits (if needed)

**Technical Notes:**
- **Server:** Health component on player entity
- **Damage Calculation:** `final_hp = current_hp - damage`
- **Death Check:** `if hp <= 0 { trigger_death() }`
- **Client:** Update health UI, flash red on damage
- **Gear Modifier:** Blast vest adds +50 HP (from FR-006)

**Components:** Game Server, Game Client

**Dependencies:** STORY-003 (damage source)

---

#### STORY-005: Death Handling and Item Loss

**Epic:** EPIC-001
**Priority:** Must Have
**Points:** 5

**User Story:**
As a player
I want death to be high-stakes with gear loss
So that the extraction shooter tension is maintained

**Acceptance Criteria:**
- [ ] Death event removes player from active raid
- [ ] All equipped gear is lost (removed from stash)
- [ ] All collected loot is lost
- [ ] Death event sent to Persistence Service via RabbitMQ
- [ ] Persistence transaction removes items atomically (ACID)
- [ ] Player returns to stash screen (Pre-Raid UI)
- [ ] Death counted in statistics

**Technical Notes:**
- **Server:** Emit `player.died` event to RabbitMQ
- **Event Data:** player_id, match_id, killer_id, loadout snapshot
- **Persistence:** DELETE FROM stash_items WHERE player_id AND item_id IN (loadout)
- **Transaction:** BEGIN → DELETE → UPDATE stats → COMMIT
- **Rollback:** If crash, transaction aborts (NFR-007 coverage)

**Components:** Game Server, Persistence Service, PostgreSQL

**Dependencies:** STORY-004 (death trigger), STORY-INF-001 (database schema)

---

#### STORY-006: Death Feedback UI with Killer Information

**Epic:** EPIC-001
**Priority:** Must Have
**Points:** 3

**User Story:**
As a player
I want to see exactly what killed me
So that I can learn and improve (death legibility)

**Acceptance Criteria:**
- [ ] 3-second freeze-frame on death showing moment of death
- [ ] Display: "Killed by [PlayerName]'s [BombType]"
- [ ] Show bomb blast radius that killed player
- [ ] Optional: Mini-replay showing last 5 seconds
- [ ] Clear visual feedback (screen darkens, UI overlay)
- [ ] Return to stash button after viewing

**Technical Notes:**
- **Client:** Capture death state (player positions, bomb positions)
- **UI:** Bevy UI overlay with death details
- **Replay:** Store last 5 seconds of state snapshots (optional v2)
- **Data:** killer_id from server, bomb_type from event
- **Goal:** 0 "mystery deaths" in testing (NFR-009)

**Components:** Game Client (Bevy UI)

**Dependencies:** STORY-005 (death event)

---

#### STORY-007: Real-Time Combat Synchronization (WebSocket)

**Epic:** EPIC-001
**Priority:** Must Have
**Points:** 5

**User Story:**
As a developer
I want reliable real-time state sync between clients and server
So that combat feels responsive and fair

**Acceptance Criteria:**
- [ ] WebSocket connection established on match start
- [ ] Server broadcasts state updates at 20Hz (50ms intervals)
- [ ] Client sends commands (move, place bomb, extract) via WebSocket
- [ ] Server acknowledges commands with sequence numbers
- [ ] Client reconciles mispredictions with server state
- [ ] Handles disconnections with 10-second reconnect window
- [ ] MessagePack binary serialization for efficiency

**Technical Notes:**
- **Server:** Tokio-tungstenite for WebSocket, 50ms tick loop
- **Protocol:** MessagePack over WebSocket (binary, compact)
- **Messages:** PlayerCommand (client→server), StateUpdate (server→client)
- **Reconciliation:** Client compares sequence numbers, rolls back mispredictions
- **Architecture:** See architecture lines 1062-1119 (WebSocket API)

**Components:** Game Server (Tokio), Game Client (Bevy + tokio-tungstenite)

**Dependencies:** STORY-001 (movement commands), STORY-003 (detonation events)

---

### EPIC-002: Loadout & Gear System

#### STORY-008: Stash UI with Gear Inventory

**Epic:** EPIC-002: Loadout & Gear System
**Priority:** Must Have
**Points:** 5

**User Story:**
As a player
I want to view my stash inventory
So that I can manage my gear between raids

**Acceptance Criteria:**
- [ ] Stash screen displays all items in player's stash
- [ ] Items shown with icon, name, quantity
- [ ] Equipped items visually distinct from inventory
- [ ] Stash capacity shown (used/total slots)
- [ ] Responsive UI works on 1920x1080 and 1280x720
- [ ] Load stash data from Persistence Service API

**Technical Notes:**
- **Framework:** React 19 + TypeScript + Tailwind CSS
- **API:** GET /api/v1/players/{id}/stash (see architecture line 955)
- **UI Library:** shadcn/ui for components (Card, Badge, Grid)
- **State:** React Context or Zustand for stash state
- **Icons:** Item icons from sprite atlas or SVG

**Components:** Pre-Raid UI (React), Persistence Service (API)

**Dependencies:** STORY-022 (stash schema), STORY-023 (currency system)

---

#### STORY-009: Loadout Selection Drag-Drop Interface

**Epic:** EPIC-002
**Priority:** Must Have
**Points:** 5

**User Story:**
As a player
I want to select my loadout with drag-drop
So that I can customize my gear before raids

**Acceptance Criteria:**
- [ ] Drag items from stash to loadout slots (bomb, vest, 2x utility)
- [ ] Drop validation (correct slot type, item in stash)
- [ ] Visual feedback during drag (ghost item, drop zones)
- [ ] Equip/unequip updates stash state
- [ ] Loadout preview shows total stats (HP, bomb range, etc.)
- [ ] Save loadout button calls Persistence Service

**Technical Notes:**
- **Library:** `react-beautiful-dnd` or `dnd-kit` for drag-drop
- **Slots:** Defined schema (bomb_type, vest_type, utility_1, utility_2)
- **Validation:** Client-side check item ownership, server validates on save
- **API:** POST /api/v1/players/{id}/stash/equip (line 990)
- **State:** Optimistic update, revert on server rejection

**Components:** Pre-Raid UI (React), Persistence Service (validation)

**Dependencies:** STORY-008 (stash UI), STORY-012 (loadout validation)

---

#### STORY-010: Gear Stat Modification System

**Epic:** EPIC-002
**Priority:** Must Have
**Points:** 3

**User Story:**
As a player
I want my gear to modify my in-game stats
So that loadout choices have meaningful gameplay impact

**Acceptance Criteria:**
- [ ] Basic Bomb: range 2, count 1, damage 100
- [ ] Piercing Bomb: range 3, penetrates 1 wall, damage 100
- [ ] Blast Vest: +50 HP (survives 1 basic bomb hit)
- [ ] Gear stats applied when match starts (from loadout snapshot)
- [ ] Stats stored in item_definitions.properties (JSONB)
- [ ] Server reads stats during match initialization

**Technical Notes:**
- **Database:** item_definitions table with JSONB properties column
- **Example:** `{"bomb_range": 3, "bomb_pierce": 1, "bomb_damage": 100}`
- **Server:** Read player loadout, apply stats to player entity
- **Formula:** `player.max_hp = base_hp + vest_bonus`
- **Balance:** "Gear creates options not dominance" (FR-006)

**Components:** Game Server (stat application), PostgreSQL (item definitions)

**Dependencies:** STORY-INF-001 (item_definitions table)

---

#### STORY-011: Visual Gear Identification Sprites

**Epic:** EPIC-002
**Priority:** Must Have
**Points:** 3

**User Story:**
As a player
I want to visually identify enemy gear
So that I can make informed tactical decisions

**Acceptance Criteria:**
- [ ] Player sprite changes based on equipped vest (basic, blast, none)
- [ ] Bomb sprite/color differs by type (basic orange, piercing blue)
- [ ] Gear identification visible from 5+ tiles away
- [ ] Sprite variants loaded from asset atlas
- [ ] Render performance maintains 60 FPS with 8 players

**Technical Notes:**
- **Assets:** Sprite sheet with player variants (3 vest types)
- **Rendering:** Bevy 2D sprite rendering, sprite index based on gear
- **Data:** Match start sends all player loadouts to clients
- **Update:** Sprite component updated when gear changes
- **Performance:** Batch rendering, sprite instancing

**Components:** Game Client (Bevy rendering)

**Dependencies:** STORY-010 (gear stats define visuals)

---

#### STORY-012: Loadout Validation and Equipment Locking

**Epic:** EPIC-002
**Priority:** Must Have
**Points:** 3

**User Story:**
As a player
I want my loadout validated before entering a raid
So that I don't lose gear I don't own

**Acceptance Criteria:**
- [ ] Server validates player owns all equipped items
- [ ] Minimum loadout enforced (at least basic bombs)
- [ ] Items locked while in raid (cannot be traded/sold)
- [ ] Loadout snapshot saved in match_participants table
- [ ] Invalid loadout returns error, blocks queue entry
- [ ] Unlock items on match end (extract or death)

**Technical Notes:**
- **API:** POST /queue validates loadout before adding to matchmaking
- **Check:** `SELECT * FROM stash_items WHERE player_id AND item_id IN (loadout)`
- **Lock:** `is_equipped = TRUE` flag prevents modification
- **Snapshot:** JSONB in match_participants.loadout (immutable)
- **Unlock:** Transaction on match end sets `is_equipped = FALSE`

**Components:** Persistence Service, Matchmaking Service (validation)

**Dependencies:** STORY-009 (loadout selection)

---

### EPIC-003: Extraction & Raid Lifecycle

#### STORY-013: Raid Timer Implementation and Synchronization

**Epic:** EPIC-003: Extraction & Raid Lifecycle
**Priority:** Must Have
**Points:** 3

**User Story:**
As a player
I want a visible raid timer counting down
So that I can make strategic extraction timing decisions

**Acceptance Criteria:**
- [ ] Raid timer starts at 5-8 minutes (configurable per map)
- [ ] Timer displayed prominently in UI (top center)
- [ ] Timer synchronized across all clients (< 1 second deviation)
- [ ] Timer countdown integrated into server tick loop
- [ ] When timer hits 0, raid ends (all remaining players extracted without loot)
- [ ] Visual/audio warnings at 2 min, 1 min, 30 sec

**Technical Notes:**
- **Server:** `time_remaining_ms` decremented each tick (50ms intervals)
- **Sync:** Included in StateUpdate broadcast every tick
- **UI:** Bevy text component, color changes (green → yellow → red)
- **End Condition:** `if time_remaining_ms <= 0 { end_match() }`
- **RabbitMQ:** Emit match.ended event on timer expiry

**Components:** Game Server, Game Client (UI)

**Dependencies:** STORY-007 (WebSocket sync)

---

#### STORY-014: Extraction Point Zones and Validation

**Epic:** EPIC-003
**Priority:** Must Have
**Points:** 3

**User Story:**
As a player
I want designated extraction points on the map
So that I know where to go to secure my loot

**Acceptance Criteria:**
- [ ] 2-4 extraction points per map (defined in map template)
- [ ] Extraction zones visually distinct (green border, icon)
- [ ] Server detects when player enters extraction zone
- [ ] Extraction requires standing in zone for 3 seconds
- [ ] Extraction interrupted if player moves or takes damage
- [ ] Extraction points shown on minimap

**Technical Notes:**
- **Map Data:** Extraction zones in map template JSONB
- **Detection:** Server checks `player.position in extraction_zones`
- **State:** Player extraction timer starts, counts down
- **Interruption:** Movement or damage resets extraction timer
- **Client:** Render extraction zone overlay, countdown bar

**Components:** Game Server, Game Client (rendering)

**Dependencies:** STORY-028 (map data structure)

---

#### STORY-015: Extraction Countdown and Success Flow

**Epic:** EPIC-003
**Priority:** Must Have
**Points:** 5

**User Story:**
As a player
I want a clear extraction process
So that I can secure my loot and feel rewarded

**Acceptance Criteria:**
- [ ] Extraction countdown starts when in zone (3 seconds)
- [ ] Countdown progress shown in UI (circular bar or text)
- [ ] Successful extraction removes player from raid
- [ ] Player keeps all equipped gear + collected loot
- [ ] Server calls Persistence Service to add items to stash
- [ ] Player redirected to stash screen with "Raid Survived" message
- [ ] Extraction counted in statistics

**Technical Notes:**
- **Flow:** Player in zone → 3s countdown → emit extract event → Persistence
- **Event:** `player.extracted` to RabbitMQ with loot data
- **Persistence:** POST /matches/{id}/results (line 834)
- **Transaction:** INSERT INTO stash_items (loot), UPDATE currencies
- **Redirect:** WebSocket sends extract_success, client transitions to stash

**Components:** Game Server, Persistence Service, Pre-Raid UI (redirect)

**Dependencies:** STORY-014 (extraction zones), STORY-026 (post-match processing)

---

#### STORY-016: Matchmaking Queue System (Solo)

**Epic:** EPIC-003
**Priority:** Must Have
**Points:** 5

**User Story:**
As a player
I want to queue for a solo raid
So that I can play with other players

**Acceptance Criteria:**
- [ ] "Find Raid" button in Pre-Raid UI
- [ ] Queue request sent to Matchmaking Service
- [ ] Player added to Redis sorted set (queue:matchmaking, timestamp)
- [ ] Queue position shown in UI
- [ ] When 4-8 players queued (or 60s timeout), match created
- [ ] WebSocket notification when match ready
- [ ] Can cancel queue before match starts

**Technical Notes:**
- **API:** POST /api/v1/queue (line 1020)
- **Queue:** Redis sorted set with player_id, score = timestamp (FIFO)
- **Matching:** Check size >= 4, create match, emit match.ready event
- **Notification:** WebSocket to clients, redirect to loading screen
- **Cancel:** DELETE /api/v1/queue/{player_id}

**Components:** Matchmaking Service, Redis, Pre-Raid UI

**Dependencies:** STORY-012 (loadout validation blocks queue)

---

#### STORY-017: Lobby Countdown and Match Creation

**Epic:** EPIC-003
**Priority:** Must Have
**Points:** 3

**User Story:**
As a player
I want a brief lobby countdown before raid starts
So that I can prepare mentally

**Acceptance Criteria:**
- [ ] 5-10 second lobby countdown after match created
- [ ] Lobby screen shows player count, map name
- [ ] Countdown timer displayed
- [ ] Match Orchestrator spawns Game Server process during countdown
- [ ] Server initializes match state (map, spawns, loot)
- [ ] Raid starts when countdown hits 0
- [ ] Players redirected to in-game view

**Technical Notes:**
- **Event:** match.ready → Match Orchestrator → spawn Game Server
- **Process:** `docker run game-server --match-id {id} --players {list}`
- **Server Init:** Load map template, allocate spawns, generate loot
- **Countdown:** Client-side timer, server sends match_start at 0
- **Redirect:** WebSocket connects clients to Game Server on port 9001

**Components:** Match Orchestrator, Game Server, Pre-Raid UI

**Dependencies:** STORY-016 (matchmaking)

---

#### STORY-018: Player Spawn System with Position Allocation

**Epic:** EPIC-003
**Priority:** Must Have
**Points:** 3

**User Story:**
As a player
I want to spawn at a random location when raid starts
So that gameplay is fair and unpredictable

**Acceptance Criteria:**
- [ ] Map defines 4-8 spawn points (in template)
- [ ] Match Orchestrator randomly assigns spawns to players
- [ ] No two players spawn on same tile
- [ ] Spawn positions sent to Game Server during initialization
- [ ] Players placed on map when raid starts
- [ ] Spawn positions recorded in match_participants table

**Technical Notes:**
- **Algorithm:** Shuffle spawn points, assign to players (first N players)
- **Data:** Spawn positions in map template JSONB
- **Storage:** match_participants.spawn_position = `{"x": 5, "y": 3}`
- **Server:** Place player entities on grid at spawn positions
- **Validation:** Check all spawns are accessible (not blocked)

**Components:** Match Orchestrator, Game Server

**Dependencies:** STORY-017 (match creation), STORY-028 (map spawn points)

---

#### STORY-019: Match End Processing and Reward Distribution

**Epic:** EPIC-003
**Priority:** Must Have
**Points:** 5

**User Story:**
As a player
I want my loot and gear processed correctly after a raid
So that progression is reliable

**Acceptance Criteria:**
- [ ] Match end triggered by timer expiry or all players extracted/dead
- [ ] Game Server emits match.ended event to RabbitMQ
- [ ] Persistence Service processes outcomes for all players
- [ ] Extractors: Loot added to stash, currency updated
- [ ] Deaths: Equipped gear removed, no loot gained
- [ ] Transaction ensures atomicity (all or nothing)
- [ ] match_participants.outcome updated (extracted/died/aborted)

**Technical Notes:**
- **Event:** match.ended includes player outcomes, loot extracted
- **API:** POST /matches/{id}/results (Match Orchestrator → Persistence)
- **Transaction:** BEGIN → process each player → INSERT/DELETE/UPDATE → COMMIT
- **ACID:** PostgreSQL guarantees prevent item duplication (NFR-007)
- **Crash Safety:** Rollback to pre-raid state if transaction fails

**Components:** Game Server, Match Orchestrator, Persistence Service

**Dependencies:** STORY-015 (extraction), STORY-005 (death), STORY-026 (loot processing)

---

### EPIC-004: Economy & Loot System

#### STORY-020: Loot Spawn Generation by Zone

**Epic:** EPIC-004: Economy & Loot System
**Priority:** Must Have
**Points:** 3

**User Story:**
As a player
I want loot to spawn in risk-tiered zones
So that high-risk areas reward better loot

**Acceptance Criteria:**
- [ ] Loot spawns at round start based on map template
- [ ] Edge zones: Common loot (250-350 credits per raid average)
- [ ] Mid zones: Uncommon loot (500-700 credits)
- [ ] Hot zones: Rare loot (1000-1500 credits)
- [ ] Loot types: Currency, gear, trade items
- [ ] Loot visible on map tiles (icons)

**Technical Notes:**
- **Algorithm:** For each loot point in template, roll loot table by tier
- **Tables:** Common (60% currency, 30% gear, 10% trade), Uncommon, Rare
- **Generation:** Server-side on match start, not revealed to clients until pickup
- **Storage:** Server maintains `loot_spawns: Vec<LootItem>` with positions
- **Balance:** Calibrate values for FR-015 (economic floor 15-20min)

**Components:** Game Server

**Dependencies:** STORY-030 (zone tiers), STORY-INF-001 (item_definitions)

---

#### STORY-021: Loot Pickup and Inventory Management

**Epic:** EPIC-004
**Priority:** Must Have
**Points:** 3

**User Story:**
As a player
I want to pick up loot during a raid
So that I can collect rewards to extract

**Acceptance Criteria:**
- [ ] Press 'E' on loot tile to pick up item
- [ ] Loot removed from map tile, added to player raid inventory
- [ ] Inventory has weight/slot limit (configurable)
- [ ] Full inventory blocks pickup (show error message)
- [ ] Can drop items to make room
- [ ] Inventory shown in UI during raid

**Technical Notes:**
- **Server:** Pickup command checks proximity, inventory space
- **State:** Player raid inventory (separate from stash)
- **UI:** Bevy UI overlay showing inventory grid
- **Protocol:** PickupLoot command (client→server), InventoryUpdate (server→client)
- **Drop:** DropLoot command places item back on tile

**Components:** Game Server, Game Client (UI + input)

**Dependencies:** STORY-020 (loot spawns)

---

#### STORY-022: Persistent Stash Database Schema

**Epic:** EPIC-004
**Priority:** Must Have
**Points:** 5

**User Story:**
As a developer
I want a reliable stash schema with item tracking
So that player progression persists across raids

**Acceptance Criteria:**
- [ ] `stash_items` table implemented (see architecture line 720)
- [ ] Foreign keys link to players and item_definitions
- [ ] Constraints prevent invalid data (positive quantity, valid items)
- [ ] Indexes optimize common queries (player_id, is_equipped)
- [ ] UNIQUE constraint prevents duplicate equipped items
- [ ] Seed data includes starter items for new players
- [ ] Migration tested with rollback

**Technical Notes:**
- **Schema:** From architecture document (lines 720-738)
- **Constraints:** `CHECK (quantity > 0)`, `UNIQUE (player_id, item_id) WHERE is_equipped`
- **Indexes:** `idx_stash_player`, `idx_stash_equipped`
- **Seed:** On player registration, INSERT starter loadout (3x bomb_basic, 1x vest_basic)
- **Tool:** `sqlx migrate` or Alembic

**Components:** PostgreSQL, Persistence Service

**Dependencies:** STORY-INF-001 (base schema)

---

#### STORY-023: Currency System and Transactions

**Epic:** EPIC-004
**Priority:** Must Have
**Points:** 3

**User Story:**
As a player
I want a currency system to buy and sell gear
So that I can progress and customize my loadout

**Acceptance Criteria:**
- [ ] `currencies` table tracks player balances
- [ ] Rubles currency implemented (default for MVP)
- [ ] CHECK constraint prevents negative balance
- [ ] Currency displayed in stash UI
- [ ] Sell loot → add currency (transactional)
- [ ] Buy gear → subtract currency (transactional)
- [ ] Starting balance for new players (e.g., 500 rubles)

**Technical Notes:**
- **Schema:** From architecture line 692
- **Constraint:** `CHECK (rubles >= 0)`
- **Transaction:** `BEGIN; UPDATE currencies SET rubles = rubles + {amount}; COMMIT;`
- **Seed:** On registration, INSERT currencies (player_id, rubles = 500)
- **UI:** Display in header (e.g., "1,250 ₽")

**Components:** PostgreSQL, Persistence Service, Pre-Raid UI

**Dependencies:** STORY-022 (stash schema)

---

#### STORY-024: Trader/Shop UI and Purchase Flow

**Epic:** EPIC-004
**Priority:** Must Have
**Points:** 5

**User Story:**
As a player
I want to buy gear from a trader
So that I can replace lost items and upgrade

**Acceptance Criteria:**
- [ ] Shop screen lists available items (bombs, vests, utility)
- [ ] Items show name, stats, cost in rubles
- [ ] Purchase button enabled if sufficient currency
- [ ] Purchase calls Persistence Service API
- [ ] Successful purchase adds item to stash, subtracts currency
- [ ] Failed purchase shows error (insufficient funds, stash full)
- [ ] Basic gear always available (no progression locks)

**Technical Notes:**
- **UI:** React component with item grid, modal for purchase confirmation
- **API:** POST /shop/purchase (or similar endpoint)
- **Data:** Fetch item_definitions from API (GET /items)
- **Validation:** Client checks balance, server enforces transaction
- **Transaction:** `BEGIN; UPDATE currencies; INSERT stash_items; COMMIT;`

**Components:** Pre-Raid UI (React), Persistence Service

**Dependencies:** STORY-023 (currency), STORY-022 (stash)

---

#### STORY-025: Item Selling and Price Calculation

**Epic:** EPIC-004
**Priority:** Must Have
**Points:** 3

**User Story:**
As a player
I want to sell loot for currency
So that I can earn money to buy gear

**Acceptance Criteria:**
- [ ] Stash UI shows "Sell" button on items
- [ ] Sell price calculated (e.g., 60% of base value)
- [ ] Sell confirmation modal shows price
- [ ] Selling removes item from stash, adds currency
- [ ] Transaction atomic (no partial sell/currency update)
- [ ] Equipped items cannot be sold (must unequip first)

**Technical Notes:**
- **Formula:** `sell_price = item_definitions.value * 0.6`
- **API:** POST /stash/sell
- **Transaction:** `BEGIN; DELETE stash_items; UPDATE currencies; COMMIT;`
- **Validation:** Check item not equipped (`is_equipped = FALSE`)
- **UI:** Confirmation modal "Sell {item} for {price} ₽?"

**Components:** Pre-Raid UI, Persistence Service

**Dependencies:** STORY-023 (currency), STORY-024 (shop for price reference)

---

#### STORY-026: Post-Match Loot Processing (Transaction)

**Epic:** EPIC-004
**Priority:** Must Have
**Points:** 5

**User Story:**
As a developer
I want post-match loot processed with ACID guarantees
So that item duplication/loss exploits are prevented

**Acceptance Criteria:**
- [ ] match.ended event triggers loot processing
- [ ] For each player outcome (extract/died):
  - Extractors: INSERT loot into stash, UPDATE currency
  - Deaths: DELETE equipped gear
- [ ] All player updates in single ACID transaction
- [ ] Transaction rollback on error (e.g., database crash)
- [ ] Audit log created for each item change
- [ ] Processing latency < 500ms per match

**Technical Notes:**
- **Transaction:** PostgreSQL `BEGIN; ... COMMIT;` wraps all updates
- **ACID:** Atomicity prevents partial loot grant, Consistency ensures valid state
- **Code:** See architecture line 852 for transaction example
- **Event Source:** audit_logs table records item.extracted, item.lost events
- **Rollback:** If crash, transaction aborts, players revert to pre-raid state

**Components:** Persistence Service, PostgreSQL

**Dependencies:** STORY-019 (match end), STORY-022 (stash schema)

---

#### STORY-027: Economic Floor Calibration and Starter Loadout

**Epic:** EPIC-004
**Priority:** Must Have
**Points:** 3

**User Story:**
As a broke player
I want to rebuild from zero in 15-20 minutes
So that I don't feel stuck after losses (FR-015)

**Acceptance Criteria:**
- [ ] New player starts with starter loadout (3x basic bomb, 1x basic vest) + 500 ₽
- [ ] Broke player (0 ₽, no gear) gets free starter loadout (debt system or grant)
- [ ] Edge loot calibrated to earn ~300 ₽ per 5-min raid
- [ ] Basic loadout cost = 1000 ₽ (achievable in 3-4 edge raids)
- [ ] Formula documented: `rebuild_time = loadout_cost / avg_edge_loot_per_raid`
- [ ] Playtest validates 15-20 minute rebuild

**Technical Notes:**
- **Starter Grant:** On registration, INSERT stash_items (bombs, vest) + currencies (500)
- **Broke Check:** If currency < 100 AND no equipped gear, grant starter
- **Calibration:** Adjust loot table weights to hit 300 ₽/raid average
- **Formula:** 1000 ₽ ÷ 300 ₽/raid = 3.3 raids × 5 min = 16.5 min
- **Playtest:** Track time for 10 broke players to rebuild

**Components:** Persistence Service (logic), Loot tables (balance)

**Dependencies:** STORY-020 (loot spawns), STORY-023 (currency)

---

### EPIC-005: Map System & Zones

#### STORY-028: Grid Map Data Structure and Validation

**Epic:** EPIC-005: Map System & Zones
**Priority:** Must Have
**Points:** 3

**User Story:**
As a developer
I want a robust grid map data structure
So that game logic can query and modify the map reliably

**Acceptance Criteria:**
- [ ] 2D array grid structure (e.g., 20x20 tiles)
- [ ] Tile types: Wall, Floor, Destructible, Loot spawn, Extraction
- [ ] Grid bounds checking prevents out-of-range access
- [ ] Tile query functions (is_walkable, is_occupied, get_tile_at)
- [ ] Serialize/deserialize map templates (JSON or binary)
- [ ] Unit tests cover edge cases (0x0 grid, 1x1 grid, large grids)

**Technical Notes:**
- **Structure:** `Grid { width: usize, height: usize, tiles: Vec<Vec<Tile>> }`
- **Enum:** `Tile { Wall, Floor, Destructible, Loot, Extraction }`
- **Validation:** Check `x < width && y < height` before access
- **Serialization:** `serde` for JSON template loading
- **Tests:** Unit tests in `map_system.rs`

**Components:** Game Server (map module)

**Dependencies:** STORY-000 (dev environment for testing)

---

#### STORY-029: Map Template System with Procedural Variation

**Epic:** EPIC-005
**Priority:** Must Have
**Points:** 5

**User Story:**
As a game designer
I want template-based map generation
So that maps have hand-crafted quality with variation

**Acceptance Criteria:**
- [ ] Map template defines static elements (walls, spawns, extraction points)
- [ ] Procedural variation adds destructible blocks (20-30% variation)
- [ ] Template format: JSON file (e.g., `maps/factory_01.json`)
- [ ] Template specifies loot zones, spawn points, extraction zones
- [ ] Generation validates map (all spawns/extracts accessible)
- [ ] At least 1 complete map template for MVP

**Technical Notes:**
- **Template:** JSON schema with grid layout, zone definitions
- **Variation:** Random placement of destructible blocks in predefined areas
- **Example:** Static layout (walls, floors) + random destructibles + loot points
- **Validation:** Pathfinding check from each spawn to each extraction
- **Storage:** Templates in `maps/` directory, loaded at runtime

**Components:** Game Server (map generation)

**Dependencies:** STORY-028 (grid structure)

---

#### STORY-030: Zone Definition (Edge/Mid/Hot) and Loot Tiers

**Epic:** EPIC-005
**Priority:** Must Have
**Points:** 3

**User Story:**
As a game designer
I want to define risk-tiered zones on maps
So that geography teaches risk (FR-022)

**Acceptance Criteria:**
- [ ] Map template defines zones: Edge (outer ring), Mid (middle), Hot (center)
- [ ] Zone metadata includes loot tier, safety rating
- [ ] Loot spawn points tagged with zone ID
- [ ] Zone boundaries visualized in map editor (future tool) or JSON
- [ ] At least 3 zones per map

**Technical Notes:**
- **Data:** `zones: [{ id: "edge", loot_tier: "common", area: [{x, y}, ...] }]`
- **Loot Mapping:** Loot spawn points reference zone ID
- **Balance:** Edge (safe, low value), Mid (moderate), Hot (dangerous, high value)
- **Geography:** Center = hot zone (convergence), edges = safe (low traffic)
- **Validation:** Ensure each zone has loot spawns

**Components:** Map templates (JSON), Game Server (zone logic)

**Dependencies:** STORY-029 (map templates)

---

#### STORY-031: Map Rendering and Visualization

**Epic:** EPIC-005
**Priority:** Must Have
**Points:** 5

**User Story:**
As a player
I want a clear visual representation of the map
So that I can navigate and understand the environment

**Acceptance Criteria:**
- [ ] 2D top-down view renders entire grid
- [ ] Tile sprites for walls, floors, destructibles
- [ ] Loot items rendered on tiles (icons)
- [ ] Players rendered with correct sprite variants
- [ ] Bombs rendered with blast radius preview (optional)
- [ ] Extraction zones highlighted (green overlay)
- [ ] Camera centered on player, shows full map or nearby area
- [ ] Rendering maintains 60 FPS with 8 players

**Technical Notes:**
- **Engine:** Bevy 2D rendering
- **Sprites:** Tile atlas (walls, floors, destructibles, loot, players)
- **Camera:** Bevy camera with tracking or fixed view
- **Layers:** Background (floor), Walls, Loot, Players, UI
- **Performance:** Sprite batching, culling off-screen tiles

**Components:** Game Client (Bevy rendering)

**Dependencies:** STORY-029 (map data), STORY-011 (player sprites)

---

#### STORY-032: Spawn and Extraction Point Validation

**Epic:** EPIC-005
**Priority:** Must Have
**Points:** 3

**User Story:**
As a developer
I want automated validation of spawn/extraction points
So that maps are always playable

**Acceptance Criteria:**
- [ ] Pathfinding algorithm checks all spawns can reach all extracts
- [ ] Validation runs when map template loaded
- [ ] Invalid maps rejected with error message (list unreachable points)
- [ ] Unit tests cover edge cases (blocked spawn, isolated extract)
- [ ] Validation integrated into map generation pipeline

**Technical Notes:**
- **Algorithm:** BFS or A* from each spawn to each extract
- **Check:** All extracts reachable from all spawns
- **Error:** "Spawn (5,3) cannot reach Extract (18,18) - blocked by walls"
- **Integration:** Run validation after procedural variation applied
- **Tool:** Future map editor runs validation before save

**Components:** Game Server (map validation)

**Dependencies:** STORY-029 (map templates), STORY-028 (grid structure)

---

## Sprint Allocation

---

### Sprint 1: Foundation & Infrastructure (Weeks 1-2)

**Goal:** Establish production-ready infrastructure with deployed services, database schema, and map system foundation enabling development of game features

**Total Points:** 29 / 30 capacity (97% utilization)

**Stories:**
- STORY-000: Development Environment Setup (3 points) - Must Have
- STORY-INF-001: Database Schema and Migrations (5 points) - Must Have
- STORY-INF-002: Docker Compose Deployment Setup (3 points) - Must Have
- STORY-INF-003: CI/CD Pipeline with GitHub Actions (5 points) - Must Have
- STORY-022: Persistent Stash Database Schema (5 points) - Must Have
- STORY-028: Grid Map Data Structure and Validation (3 points) - Must Have
- STORY-029: Map Template System with Procedural Variation (5 points) - Must Have

**Risks:**
- Docker environment issues on different platforms (macOS, Linux, Windows WSL2)
- Database migration complexity with constraints and indexes
- Map generation algorithm edge cases

**Dependencies:**
- No external blockers (foundational sprint)

**Deliverables:**
- Working local dev environment (`docker compose up`)
- Database with all 7 tables, seed data
- CI/CD pipeline running tests on every commit
- At least 1 playable map template

---

### Sprint 2: Core Combat Mechanics (Weeks 3-4)

**Goal:** Deliver functional grid combat with movement, bombs, blast mechanics, and death handling demonstrating core gameplay loop

**Total Points:** 26 / 30 capacity (87% utilization)

**Stories:**
- STORY-001: Grid-Based Player Movement (5 points) - Must Have
- STORY-002: Bomb Placement Mechanics (3 points) - Must Have
- STORY-003: Bomb Detonation and Blast Propagation (5 points) - Must Have
- STORY-004: Player Health and Damage System (3 points) - Must Have
- STORY-005: Death Handling and Item Loss (5 points) - Must Have
- STORY-007: Real-Time Combat Synchronization (5 points) - Must Have

**Risks:**
- Client-side prediction complexity (misprediction reconciliation)
- WebSocket connection stability under load
- Blast propagation algorithm edge cases (wall penetration)

**Dependencies:**
- Sprint 1 must complete (database, map system)
- Sequential: Movement → Bombs → Detonation → Death

**Deliverables:**
- Playable combat demo (1v1 local test)
- Movement with <100ms input latency
- Deterministic bomb blast mechanics
- Death triggers item loss transaction

**Demo:** Two players can move, place bombs, and die in a local match

---

### Sprint 3: Economy & Loadout System (Weeks 5-6)

**Goal:** Enable pre-raid economy with working stash, loadout selection, shop, and currency system allowing players to gear up and progress

**Total Points:** 27 / 30 capacity (90% utilization)

**Stories:**
- STORY-008: Stash UI with Gear Inventory (5 points) - Must Have
- STORY-009: Loadout Selection Drag-Drop Interface (5 points) - Must Have
- STORY-010: Gear Stat Modification System (3 points) - Must Have
- STORY-023: Currency System and Transactions (3 points) - Must Have
- STORY-024: Trader/Shop UI and Purchase Flow (5 points) - Must Have
- STORY-025: Item Selling and Price Calculation (3 points) - Must Have
- STORY-027: Economic Floor Calibration and Starter Loadout (3 points) - Must Have

**Risks:**
- Drag-drop UX complexity (library integration, edge cases)
- Transaction ACID guarantees (testing rollback scenarios)
- Economic balance calibration (may need iteration)

**Dependencies:**
- Sprint 1 stash schema (STORY-022)
- Sequential: Currency → Shop → Selling

**Deliverables:**
- Functional Pre-Raid UI with stash and shop
- Buy/sell transactions working
- Loadout selection updates gear stats
- Economic floor validated (broke → rebuild in 15-20 min)

**Demo:** Player can buy gear, equip loadout, sell loot for currency

---

### Sprint 4: Extraction & Matchmaking (Weeks 7-8)

**Goal:** Complete raid lifecycle from matchmaking queue through extraction with timer pressure and post-match reward processing

**Total Points:** 27 / 30 capacity (90% utilization)

**Stories:**
- STORY-013: Raid Timer Implementation and Synchronization (3 points) - Must Have
- STORY-014: Extraction Point Zones and Validation (3 points) - Must Have
- STORY-015: Extraction Countdown and Success Flow (5 points) - Must Have
- STORY-016: Matchmaking Queue System (5 points) - Must Have
- STORY-017: Lobby Countdown and Match Creation (3 points) - Must Have
- STORY-018: Player Spawn System with Position Allocation (3 points) - Must Have
- STORY-019: Match End Processing and Reward Distribution (5 points) - Must Have

**Risks:**
- Matchmaking queue timing (slow queues in testing with 1 player)
- Match Orchestrator process spawning reliability
- Race conditions in match end processing

**Dependencies:**
- Sprint 2 combat (death events)
- Sprint 3 economy (loot processing)
- Sequential: Queue → Lobby → Spawn → Timer → Extract → Rewards

**Deliverables:**
- Working matchmaking queue (can test with bots or multiple clients)
- Full raid lifecycle (queue → match → extract → rewards)
- Raid timer countdown with extraction pressure
- Post-match loot correctly added to stash

**Demo:** Player queues, joins match, plays raid, extracts with loot, sees stash updated

---

### Sprint 5: Integration & Polish (Weeks 9-10)

**Goal:** Ship playable MVP with loot economy, visual polish, and integrated systems ready for multiplayer testing

**Total Points:** 31 / 30 capacity (103% utilization - acceptable for final sprint)

**Stories:**
- STORY-006: Death Feedback UI with Killer Information (3 points) - Must Have
- STORY-011: Visual Gear Identification Sprites (3 points) - Must Have
- STORY-012: Loadout Validation and Equipment Locking (3 points) - Must Have
- STORY-020: Loot Spawn Generation by Zone (3 points) - Must Have
- STORY-021: Loot Pickup and Inventory Management (3 points) - Must Have
- STORY-026: Post-Match Loot Processing (5 points) - Must Have
- STORY-030: Zone Definition and Loot Tiers (3 points) - Must Have
- STORY-031: Map Rendering and Visualization (5 points) - Must Have
- STORY-032: Spawn and Extraction Point Validation (3 points) - Must Have

**Risks:**
- Integration bugs across 5 epics
- Visual polish time sink (sprites, UI)
- Final balancing may require iteration

**Dependencies:**
- All previous sprints (final integration)
- Loot system depends on zones (STORY-030)
- Death feedback depends on combat (STORY-006)

**Deliverables:**
- Complete loot economy (spawn → pickup → extract → sell)
- Visual polish (death feedback, gear sprites, map rendering)
- Validated map with zone tiers
- All 22 FRs implemented and integrated

**Demo:** Full playable MVP - queue, spawn, loot, combat, extract, sell loot, rebuy gear, repeat

**Acceptance:** Ready for multiplayer playtesting with 4-8 real players

---

## Epic Traceability

| Epic ID | Epic Name | Stories | Total Points | Sprints |
|---------|-----------|---------|--------------|---------|
| Infrastructure | Dev Setup & CI/CD | STORY-000, INF-001, INF-002, INF-003 | 16 points | Sprint 1 |
| EPIC-001 | Core Combat & Death System | STORY-001, 002, 003, 004, 005, 006, 007 | 29 points | Sprint 2, 5 |
| EPIC-002 | Loadout & Gear System | STORY-008, 009, 010, 011, 012 | 19 points | Sprint 3, 5 |
| EPIC-003 | Extraction & Raid Lifecycle | STORY-013, 014, 015, 016, 017, 018, 019 | 27 points | Sprint 4 |
| EPIC-004 | Economy & Loot System | STORY-020, 021, 022, 023, 024, 025, 026, 027 | 30 points | Sprint 1, 3, 5 |
| EPIC-005 | Map System & Zones | STORY-028, 029, 030, 031, 032 | 19 points | Sprint 1, 5 |

**Total:** 140 points across 6 epic groups (5 feature epics + infrastructure)

---

## Requirements Coverage

### Functional Requirements → Story Mapping

| FR ID | FR Name | Stories | Sprint |
|-------|---------|---------|--------|
| FR-001 | Tile-Based Player Movement | STORY-001 | Sprint 2 |
| FR-002 | Bomb Placement | STORY-002 | Sprint 2 |
| FR-003 | Bomb Detonation Pattern | STORY-003 | Sprint 2 |
| FR-004 | Player Health and Death | STORY-004, STORY-005 | Sprint 2 |
| FR-005 | Pre-Raid Loadout Selection | STORY-008, STORY-009, STORY-012 | Sprint 3, 5 |
| FR-006 | Gear Stats Modification | STORY-010 | Sprint 3 |
| FR-007 | Visual Gear Identification | STORY-011 | Sprint 5 |
| FR-008 | Raid Timer | STORY-013 | Sprint 4 |
| FR-009 | Extraction Points | STORY-014 | Sprint 4 |
| FR-010 | Successful Extraction | STORY-015, STORY-019 | Sprint 4 |
| FR-011 | Loot Spawns | STORY-020 | Sprint 5 |
| FR-012 | Loot Pickup | STORY-021 | Sprint 5 |
| FR-013 | Stash & Currency | STORY-022, STORY-023 | Sprint 1, 3 |
| FR-014 | Trader/Shop | STORY-024, STORY-025 | Sprint 3 |
| FR-015 | Economic Floor (15-20min rebuild) | STORY-027 | Sprint 3 |
| FR-016 | Solo Queue | STORY-016 | Sprint 4 |
| FR-017 | Map Selection | STORY-016 (includes map choice) | Sprint 4 |
| FR-018 | Raid Lobby & Spawn | STORY-017, STORY-018 | Sprint 4 |
| FR-019 | Death = Lose All | STORY-005, STORY-019 | Sprint 2, 4 |
| FR-020 | Death Replay/Feedback | STORY-006 | Sprint 5 |
| FR-021 | Grid-Based Map | STORY-028, STORY-029, STORY-031 | Sprint 1, 5 |
| FR-022 | Map Zones (Risk Tiers) | STORY-030 | Sprint 5 |

**Coverage:** 22/22 functional requirements (100%)

### Non-Functional Requirements Validation

| NFR ID | NFR Name | Validation Method | Stories |
|--------|----------|-------------------|---------|
| NFR-001 | Input Responsiveness (< 100ms) | Log keypress → visual feedback timestamp delta | STORY-001, 007 |
| NFR-002 | Bomb Timer Accuracy (< 50ms) | Record detonation timestamps across clients | STORY-003 |
| NFR-003 | Match Capacity (10+ matches) | Load test 12 concurrent matches, monitor CPU | STORY-016, 017 |
| NFR-004 | Authoritative Server | Test invalid commands, verify 100% rejection | STORY-001-007 |
| NFR-005 | Input Validation | Bot spamming commands, verify rate limits | STORY-001, 002 |
| NFR-006 | MVP Uptime (90%) | Track uptime over 1-week window | STORY-INF-002 |
| NFR-007 | Crash Recovery (No corruption) | Kill Game Server mid-match, verify stash integrity | STORY-005, 019, 026 |
| NFR-008 | Control Scheme | User testing, 100% understand in 30s | STORY-001, 002 |
| NFR-009 | Visual Clarity | User testing, 0 mystery deaths | STORY-006, 011 |
| NFR-010 | Structured Logging | Query logs for match, verify all events captured | All stories |
| NFR-011 | Match Replay Data | Record match, reconstruct final state | STORY-005, 019 |
| NFR-012 | Platform Support (Win, Mac, Linux) | Test on 3 platforms, 60 FPS | STORY-000, 031 |

**Coverage:** 12/12 non-functional requirements addressed

---

## Risks and Mitigation

### High-Priority Risks

**RISK-001: Real-Time Networking Complexity**
- **Description:** WebSocket sync, client-side prediction, misprediction reconciliation
- **Impact:** High (core to combat feel)
- **Probability:** Medium
- **Mitigation:**
  - Prototype WebSocket in Sprint 2 early
  - Use proven libraries (tokio-tungstenite, rmp-serde)
  - Reference Bevy networking examples
  - Load test with 8 concurrent clients
- **Owner:** Developer (STORY-007)

**RISK-002: Game Balance Calibration**
- **Description:** Economic floor, loot values, combat balance, gear asymmetry
- **Impact:** High (retention depends on balanced economy)
- **Probability:** High (first-time balancing)
- **Mitigation:**
  - Make all values configurable (JSONB properties, config files)
  - Playtest early with 5+ test raids
  - Track metrics: edge loot per raid, rebuild time, survival rate
  - Iterate in Sprint 5 based on data
- **Owner:** Developer + Playtesters (STORY-027, 020)

**RISK-003: Solo Developer Bandwidth**
- **Description:** 140 points over 5 sprints, no team to delegate
- **Impact:** High (delivery timeline)
- **Probability:** Medium
- **Mitigation:**
  - Strict scope control (defer FR-017 map selection if needed)
  - No feature creep (stick to 36 stories)
  - Use shadcn/ui for Pre-Raid UI (reduce UI work)
  - Defer polish to post-MVP (focus on functional completeness)
- **Owner:** Scrum Master

### Medium-Priority Risks

**RISK-004: Bevy Learning Curve**
- **Mitigation:** Simple 2D rendering, reference Bevy 2D examples, community Discord
- **Status:** Low if familiar with ECS, Medium if new to Bevy

**RISK-005: Database Performance (ACID Transactions)**
- **Mitigation:** PostgreSQL indexing, connection pooling (PgBouncer), test with 12 concurrent matches
- **Status:** Architecture includes indexes, should be sufficient for MVP

**RISK-006: Cross-Platform Builds**
- **Mitigation:** GitHub Actions CI from Sprint 1, test on Windows/macOS/Linux regularly
- **Status:** Bevy compiles to all platforms, CI will catch regressions

### Low-Priority Risks

**RISK-007: RabbitMQ Operational Complexity**
- **Mitigation:** Docker Compose simplifies deployment, well-documented
- **Status:** Monitoring only

**RISK-008: UI/UX Polish Time Sink**
- **Mitigation:** MVP UI only, defer animations/polish to post-MVP
- **Status:** Monitoring (Sprint 5 buffer allows overrun)

---

## Dependencies

### Technical Dependencies

**Runtime Dependencies:**
- **Rust Ecosystem:** Rust 1.75+, Bevy 0.12, Tokio, tokio-tungstenite, rmp-serde, sqlx
- **Python Ecosystem:** Python 3.12+, FastAPI 0.109+, asyncpg, pydantic v2, aiohttp
- **Infrastructure:** Docker, Docker Compose, PostgreSQL 16, Redis 7, RabbitMQ 3.13, Traefik 2.10
- **Frontend:** Node.js (for Vite build), React 19, TypeScript, Tailwind CSS, shadcn/ui

**Development Dependencies:**
- **Tools:** mise (task management), Git, GitHub CLI
- **CI/CD:** GitHub Actions runners
- **Testing:** pytest, cargo test, cargo clippy, ruff, mypy

**No External Third-Party Services:**
- No payment processors
- No email services
- No cloud APIs
- No authentication providers (self-hosted JWT)

### Story Dependencies (Critical Path)

**Sequential Dependencies:**
1. Sprint 1 infrastructure → All feature sprints
2. STORY-028 (grid) → STORY-001 (movement)
3. STORY-001 → STORY-002 → STORY-003 (movement → bombs → detonation)
4. STORY-022 (stash schema) → STORY-008 (stash UI)
5. STORY-023 (currency) → STORY-024 (shop) → STORY-025 (selling)
6. STORY-016 (matchmaking) → STORY-017 (lobby) → STORY-018 (spawn)

**Parallel Work Opportunities:**
- Sprint 1: Database schema + Map system + CI/CD can progress in parallel
- Sprint 3: Stash UI + Shop UI can be built concurrently
- Sprint 5: Loot system + Visual polish + Map rendering mostly independent

---

## Definition of Done

For a story to be considered **complete**, all criteria must be met:

### Code Quality
- [ ] Code implemented and committed to version control
- [ ] Unit tests written and passing (≥80% coverage for business logic)
- [ ] Integration tests passing (for multi-service features)
- [ ] Linting passes (clippy for Rust, ruff for Python)
- [ ] Type checking passes (mypy for Python)
- [ ] No compiler warnings

### Review & Documentation
- [ ] Code reviewed (self-review against architecture doc)
- [ ] Acceptance criteria validated (manual testing or automated)
- [ ] Technical documentation updated (README, architecture notes)
- [ ] API endpoints documented (if applicable)

### Deployment
- [ ] Feature deployed to local dev environment (docker compose up)
- [ ] Health checks passing (if service)
- [ ] No regression in existing features (smoke test)

### Story-Specific
- [ ] All acceptance criteria checked off
- [ ] Dependencies satisfied (linked stories completed first)
- [ ] NFR requirements met (e.g., <100ms input latency for STORY-001)

### Sprint Review Ready
- [ ] Demo-able (can show working feature)
- [ ] No known critical bugs
- [ ] Integrated with rest of system

---

## Next Steps

### Immediate Actions (Before Sprint 1 Start)

1. **Review and Approve This Plan**
   - Validate story breakdown makes sense
   - Confirm point estimates align with complexity understanding
   - Adjust sprint goals if needed

2. **Set Up Project Tracking**
   - Sprint status initialized in `docs/sprint-status.yaml`
   - Story tracking tool (GitHub Projects, Jira, or simple markdown)
   - Daily/weekly progress tracking

3. **Prepare Development Environment**
   - Clone repository, verify tools installed (Rust, Python, Docker, mise)
   - Review architecture document
   - Bookmark reference docs (Bevy, FastAPI, PostgreSQL)

### Starting Sprint 1

**Goal:** Establish production-ready infrastructure

**Day 1:**
- Start STORY-000 (dev environment setup)
- Initialize Docker Compose configuration
- Verify local services running (PostgreSQL, Redis, RabbitMQ)

**Week 1 Focus:**
- Complete infrastructure stories (000, INF-001, INF-002, INF-003)
- Get CI/CD pipeline green
- Database schema with seed data

**Week 2 Focus:**
- Complete map system foundation (STORY-028, STORY-029)
- Stash schema (STORY-022)
- Validate Sprint 1 deliverables

**Sprint 1 Demo:**
- `docker compose up` starts entire stack
- `make test` (or mise task) runs all tests and passes
- Database contains 7 tables with seed data
- 1 map template loads successfully

### Commands to Run Stories

**Individual Story:**
```bash
# Not available yet - /dev-story will be available post-MVP
# For now, implement stories manually following this plan
```

**Sprint Status:**
```bash
# Check current sprint progress (will be available after status file created)
# For now, track progress manually in docs/sprint-status.yaml
```

**Full Workflow:**
1. Review sprint plan (this document)
2. Start Sprint 1 stories in order
3. Track progress in sprint-status.yaml
4. Demo at end of Sprint 1
5. Retrospective and adjust for Sprint 2
6. Repeat for Sprints 2-5

### Post-Sprint 5 (MVP Complete)

**Validation:**
- Run full MVP playtest with 4-8 real players
- Validate all 22 FRs implemented and working
- Verify all 12 NFRs met (performance, uptime, etc.)
- Collect feedback on balance, UX, bugs

**Next Phase:**
- Iterate on balance based on playtest data
- Fix critical bugs
- Plan v2 features (squads, AI enemies, additional maps)
- Production deployment preparation

---

## Appendix

### Sprint Cadence

**Sprint Length:** 2 weeks (10 workdays)

**Sprint Ceremonies:**
- **Sprint Planning:** Monday Week 1 (review plan, commit to stories)
- **Daily Standup:** Daily (self-check: progress, blockers)
- **Sprint Review:** Friday Week 2 (demo completed stories)
- **Sprint Retrospective:** Friday Week 2 (what went well, what to improve)

**Working Hours:**
- 6 productive hours/day (senior developer velocity)
- 60 hours per sprint total
- 30 story points per sprint capacity

### Story Point Reference

**Estimation Guide:**
- **1 point:** Trivial (1-2 hours) - Config change, text update
- **2 points:** Simple (2-4 hours) - Basic CRUD, simple component
- **3 points:** Moderate (4-8 hours) - Complex component, business logic
- **5 points:** Complex (1-2 days) - Feature with multiple components
- **8 points:** Very Complex (2-3 days) - Full feature frontend + backend

**Average Story Size:** 3.9 points (well-balanced)

### Glossary

- **Epic:** Large body of work (2-10 stories)
- **Story:** Implementable unit of work (1-3 days)
- **Sprint:** 2-week iteration
- **Velocity:** Story points completed per sprint
- **Capacity:** Story points team can handle per sprint
- **DoD:** Definition of Done (completion criteria)

---

**This sprint plan was created using BMAD Method v6 - Phase 4 (Implementation Planning)**

**Plan Created:** 2025-12-26
**Plan Owner:** Jarad DeLorenzo (Scrum Master)
**Ready for:** Sprint 1 kickoff

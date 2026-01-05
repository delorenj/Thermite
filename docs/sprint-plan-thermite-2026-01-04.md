# Sprint Plan: Thermite (BOMBOUT)

**Date:** 2026-01-04
**Scrum Master:** Jarad DeLorenzo (Steve)
**Project Level:** 4 (MVP Scope)
**Total Stories:** 37
**Total Points:** 216 points
**Planned Sprints:** 7 sprints (assuming 30 points/sprint capacity)

---

## Executive Summary

This sprint plan breaks down the Thermite MVP into 37 user stories across 5 epics and infrastructure work. The plan follows the validated architecture and PRD requirements, targeting a hybrid extraction shooter combining Bomberman-style grid combat with Tarkov's high-stakes loot mechanics.

**Key Metrics:**
- Total Stories: 37
- Total Points: 216 points
- Epics: 5 (all Must Have)
- Infrastructure Stories: 3
- Estimated Sprints: 7 (2-week sprints)
- Target Completion: ~14 weeks (3.5 months)

**Recommended Sprint Order:**
1. Sprint 1: Infrastructure + Map Foundation
2. Sprint 2: Core Combat Mechanics
3. Sprint 3: Combat Completion + Loadout System
4. Sprint 4: Extraction & Raid Lifecycle
5. Sprint 5: Economy & Loot System (Part 1)
6. Sprint 6: Economy & Loot System (Part 2)
7. Sprint 7: Polish & Integration

---

## Story Inventory

### Infrastructure Stories (3 stories, 21 points)

#### STORY-INF-001: Database Schema and Migrations

**Epic:** Infrastructure
**Priority:** Must Have
**Points:** 8

**User Story:**
As a developer
I want a complete PostgreSQL database schema with migrations
So that the persistence layer is production-ready

**Acceptance Criteria:**
- [ ] All tables created: players, currencies, stash_items, item_definitions, matches, match_participants, audit_logs
- [ ] Foreign key constraints and indexes defined
- [ ] JSONB columns for flexible data (loadout, loot_extracted)
- [ ] Migration scripts using Alembic (Python) or similar
- [ ] Seed data for item_definitions (basic bombs, vests, consumables)
- [ ] Database connection pooling configured
- [ ] Transaction isolation levels tested

**Technical Notes:**
- Reference architecture SQL schemas (Lines 675-801)
- Use SQLAlchemy for Python services
- PgBouncer for connection pooling
- JSONB for loadout snapshots and loot data

**Dependencies:**
- PostgreSQL 16 installed locally and in deployment

---

#### STORY-INF-002: Backend Service Scaffolding

**Epic:** Infrastructure
**Priority:** Must Have
**Points:** 8

**User Story:**
As a developer
I want backend microservices scaffolded with FastAPI
So that I can implement business logic efficiently

**Acceptance Criteria:**
- [ ] 4 FastAPI services created: Authentication, Persistence, Matchmaking, Match Orchestrator
- [ ] Docker Compose configuration for all services
- [ ] Pydantic models for request/response validation
- [ ] Health check endpoints (`/health`) for all services
- [ ] Structured logging configured (JSON format)
- [ ] CORS and security headers configured
- [ ] Environment variable management (.env files)
- [ ] Inter-service communication tested (RabbitMQ events)

**Technical Notes:**
- Services: auth (8004), persistence (8003), matchmaking (8002), match-orchestrator (8001)
- Use Pydantic V2 for validation
- structlog for JSON logging
- Traefik for routing

**Dependencies:**
- Docker Compose installed
- RabbitMQ and Redis containers running

---

#### STORY-INF-003: CI/CD Pipeline

**Epic:** Infrastructure
**Priority:** Should Have
**Points:** 5

**User Story:**
As a developer
I want automated CI/CD pipeline with GitHub Actions
So that code quality is enforced and deployments are automated

**Acceptance Criteria:**
- [ ] GitHub Actions workflow for Rust: cargo test, cargo clippy, cargo tarpaulin (coverage)
- [ ] GitHub Actions workflow for Python: pytest, coverage, ruff linter
- [ ] Docker image builds for all services
- [ ] Deployment script for Docker Compose
- [ ] Branch protection rules enforced (require passing tests)
- [ ] Code coverage reports generated

**Technical Notes:**
- Rust: cargo tarpaulin for coverage (target 80%)
- Python: pytest with pytest-cov
- Fail build if coverage <80% or linting errors
- Docker buildx for multi-arch builds

**Dependencies:**
- GitHub repository created
- Secrets configured (Docker Hub, deployment keys)

---

### EPIC-001: Core Combat & Death System (8 stories, 40 points)

#### STORY-001: Grid-Based Map Data Structure

**Epic:** EPIC-001
**Priority:** Must Have
**Points:** 5
**FR:** FR-001 (Tile-Based Movement), FR-021 (Grid-Based Map)

**User Story:**
As a game developer
I want a grid-based map data structure in Rust
So that I can represent the game world efficiently

**Acceptance Criteria:**
- [ ] Grid data structure (Vec<Vec<Tile>>) with NxM dimensions
- [ ] Tile enum: Wall, Floor, DestructibleBlock
- [ ] Grid coordinate system (x, y) with bounds checking
- [ ] Pathfinding validation (ensure spawn/extraction accessible)
- [ ] Serialization/deserialization for map templates
- [ ] Unit tests for grid operations (get_tile, is_walkable, neighbors)

**Technical Notes:**
- Use Bevy ECS grid component for Game Client
- Game Server owns authoritative grid state
- Grid size: Start with 20x20, configurable
- Reference architecture Line 437 (map structure)

**Dependencies:**
- Bevy engine setup (Game Client)
- Rust project structure

---

#### STORY-002: Player Movement System

**Epic:** EPIC-001
**Priority:** Must Have
**Points:** 5
**FR:** FR-001 (Tile-Based Movement)

**User Story:**
As a player
I want to move on a grid with WASD keys
So that I can navigate the map tactically

**Acceptance Criteria:**
- [ ] WASD input handling in Bevy (or arrow keys)
- [ ] Client sends movement commands to server via WebSocket
- [ ] Server validates moves (bounds checking, wall collision)
- [ ] Client-side prediction for responsive movement (< 100ms perceived latency)
- [ ] Server broadcasts position updates to all clients (20Hz tick rate)
- [ ] Movement rollback/correction if server rejects invalid move
- [ ] Visual feedback for movement (player sprite position updates)

**Technical Notes:**
- WebSocket binary protocol (MessagePack)
- Client prediction: optimistically move player, wait for server confirmation
- Server command validation per architecture Line 1206-1219
- NFR-001: < 100ms input latency

**Dependencies:**
- STORY-001 (Grid data structure)
- WebSocket connection established
- Game Server tick loop running

---

#### STORY-003: Bomb Placement Mechanic

**Epic:** EPIC-001
**Priority:** Must Have
**Points:** 5
**FR:** FR-002 (Bomb Placement)

**User Story:**
As a player
I want to place bombs with Spacebar
So that I can control space and attack enemies

**Acceptance Criteria:**
- [ ] Spacebar key places bomb on current tile
- [ ] Client sends place_bomb command to server
- [ ] Server validates: player has bombs remaining, tile is empty, cooldown elapsed
- [ ] Bomb entity created on server with timer (default 3 seconds = 60 ticks)
- [ ] Bomb visual representation on client (sprite on tile)
- [ ] Server rejects invalid placements (rate limiting, bomb limit)
- [ ] Audio feedback on placement

**Technical Notes:**
- Bomb struct: {position, owner_id, ticks_remaining, bomb_type}
- Server decrements ticks_remaining each tick (50ms intervals)
- Client shows countdown timer overlay on bomb sprite
- Rate limit: 1s cooldown between placements

**Dependencies:**
- STORY-002 (Player movement system)
- Game Server tick loop
- Bomb sprite assets

---

#### STORY-004: Bomb Detonation and Blast Pattern

**Epic:** EPIC-001
**Priority:** Must Have
**Points:** 5
**FR:** FR-003 (Bomb Detonation Pattern)

**User Story:**
As a player
I want bombs to detonate in a cross pattern
So that I can strategically trap and damage enemies

**Acceptance Criteria:**
- [ ] When timer reaches 0, bomb detonates
- [ ] Blast pattern: 4 cardinal directions (up/down/left/right)
- [ ] Blast range configurable per bomb type (default 2 tiles)
- [ ] Blast stops at walls or destructible blocks
- [ ] Blast visual effect (explosion animation)
- [ ] Blast audio effect
- [ ] Server broadcasts detonation event to all clients
- [ ] Chain reactions (bomb detonates bomb) supported

**Technical Notes:**
- Blast propagation algorithm: iterate from bomb position outward in each direction
- Destructible blocks removed from grid on blast
- Blast damage applied to players in affected tiles (next story)
- NFR-002: < 50ms timer deviation across clients

**Dependencies:**
- STORY-003 (Bomb placement)
- Explosion sprite assets

---

#### STORY-005: Player Health and Damage System

**Epic:** EPIC-001
**Priority:** Must Have
**Points:** 3
**FR:** FR-004 (Player Health and Death)

**User Story:**
As a player
I want to take damage from bomb blasts
So that there's risk and consequence to combat

**Acceptance Criteria:**
- [ ] Player has health stat (default 100 HP)
- [ ] Server calculates damage when blast hits player tile
- [ ] Damage values: basic bomb = 100 (one-hit kill), configurable per bomb type
- [ ] Armor reduces damage (vest types modify HP or damage reduction)
- [ ] Server broadcasts damage event to clients
- [ ] Client displays health bar/indicator
- [ ] Death triggered when health <= 0

**Technical Notes:**
- Health stored server-side (authoritative)
- Client shows predicted health, corrects on server update
- Damage calculation: base_damage - armor_reduction
- No partial damage zones (blast is binary: hit or miss per tile)

**Dependencies:**
- STORY-004 (Bomb detonation)
- STORY-002 (Player entities exist)

---

#### STORY-006: Death Handling and Item Loss

**Epic:** EPIC-001
**Priority:** Must Have
**Points:** 5
**FR:** FR-019 (Death = Lose All)

**User Story:**
As a player who dies in a raid
I want to lose all equipped gear and collected loot
So that death has meaningful consequences

**Acceptance Criteria:**
- [ ] On death (health <= 0), server marks player as dead
- [ ] Player entity removed from active game state
- [ ] Server emits player.died event to RabbitMQ
- [ ] Persistence Service processes death: DELETE equipped items from stash
- [ ] Player returned to "out of raid" state (stash screen)
- [ ] Death counts as "raid failed" in statistics
- [ ] No secure container (all loot lost)

**Technical Notes:**
- Server sends match result to Persistence Service POST /matches/{id}/results
- Persistence Service runs ACID transaction to remove items
- Reference architecture Line 487: process_death_outcome
- Loadout snapshot in match_participants table (for audit)

**Dependencies:**
- STORY-005 (Health system)
- STORY-INF-001 (Database schema)
- Persistence Service operational

---

#### STORY-007: Death Feedback and Replay

**Epic:** EPIC-001
**Priority:** Must Have
**Points:** 3
**FR:** FR-020 (Death Replay/Feedback)

**User Story:**
As a player who just died
I want to see what killed me
So that I can learn and improve

**Acceptance Criteria:**
- [ ] 3-5 second freeze-frame when player dies
- [ ] Display: "Killed by [PlayerName]'s [BombType]"
- [ ] Show bomb that dealt killing blow (highlight on grid)
- [ ] Show damage source position
- [ ] Optional: 3-second replay of last 5 seconds (Could Have)
- [ ] Clear "Return to Stash" button after feedback

**Technical Notes:**
- Client stores last N game states (ring buffer)
- On death event, freeze and display overlay
- Educational: help player understand tactical mistakes
- NFR-009: Death must be legible (no mystery deaths)

**Dependencies:**
- STORY-006 (Death handling)
- UI overlay assets

---

#### STORY-008: Game Client Rendering (Bevy)

**Epic:** EPIC-001
**Priority:** Must Have
**Points:** 8

**User Story:**
As a player
I want a visually clear game client
So that I can see the game state and make tactical decisions

**Acceptance Criteria:**
- [ ] Bevy 2D renderer displays grid map
- [ ] Player sprites rendered on correct tiles
- [ ] Bomb sprites rendered with timer countdown overlays
- [ ] Explosion animations play on detonation
- [ ] Health bars visible above players
- [ ] Minimap overlay (optional, Could Have)
- [ ] Raid timer displayed prominently
- [ ] 60 FPS performance on integrated GPU
- [ ] Cross-platform builds: Windows, macOS, Linux

**Technical Notes:**
- Bevy 0.12+ with 2D rendering
- Sprite assets: players (different gear), bombs, explosions, tiles
- NFR-012: Platform support (Windows, macOS, Linux)
- NFR-009: Visual clarity (blast radius visible, gear distinct)

**Dependencies:**
- STORY-001 through STORY-007 (game mechanics implemented)
- Sprite assets created
- Bevy engine integrated

---

### EPIC-002: Loadout & Gear System (5 stories, 25 points)

#### STORY-009: Stash View and Inventory Management

**Epic:** EPIC-002
**Priority:** Must Have
**Points:** 5
**FR:** FR-013 (Stash & Currency)

**User Story:**
As a player
I want to view my stash with all items and currency
So that I can manage my inventory between raids

**Acceptance Criteria:**
- [ ] Pre-Raid UI (React SPA) displays player stash
- [ ] GET /api/v1/players/{player_id}/stash returns equipped + inventory items
- [ ] Currency balance displayed (Rubles)
- [ ] Stash grid UI with drag-drop (shadcn/ui)
- [ ] Item tooltips show stats (bomb range, vest HP, value)
- [ ] Filter/sort by item type
- [ ] Stash slot limit enforced visually (20 slots in MVP)

**Technical Notes:**
- Persistence Service API endpoint
- React component with Tailwind CSS
- Use shadcn/ui drag-drop component
- Reference architecture Line 540-563 (Pre-Raid UI Service)

**Dependencies:**
- STORY-INF-001 (Database schema)
- STORY-INF-002 (Persistence Service)
- React app scaffolded

---

#### STORY-010: Pre-Raid Loadout Selection

**Epic:** EPIC-002
**Priority:** Must Have
**Points:** 5
**FR:** FR-005 (Pre-Raid Loadout Selection), FR-014 (Gear Loadout)

**User Story:**
As a player
I want to select my loadout before entering a raid
So that I can customize my playstyle

**Acceptance Criteria:**
- [ ] Loadout selection UI: bomb type, bomb quantity, vest type
- [ ] Drag items from stash to loadout slots
- [ ] POST /api/v1/players/{player_id}/stash/equip updates equipped status
- [ ] Loadout validation: must have at least basic bombs
- [ ] Loadout value displayed (total risk)
- [ ] "Enter Raid" button enabled only with valid loadout
- [ ] Equipped items locked (cannot be modified once in queue)

**Technical Notes:**
- Loadout structure: {bomb_type, bomb_quantity, vest_type, utility_slots}
- Server validates loadout before matchmaking
- Reference architecture Line 990-1014 (loadout selection API)

**Dependencies:**
- STORY-009 (Stash view)
- Persistence Service operational

---

#### STORY-011: Gear Stats Modification System

**Epic:** EPIC-002
**Priority:** Must Have
**Points:** 5
**FR:** FR-006 (Gear Stats Modification)

**User Story:**
As a player
I want different gear to modify my stats
So that I can choose tactical options

**Acceptance Criteria:**
- [ ] Basic Bomb: range 2, count 1, damage 100
- [ ] Piercing Bomb: range 3, penetrates 1 wall, damage 100
- [ ] Blast Vest: +50 HP (survives 1 basic bomb)
- [ ] Gear stats applied when loadout equipped
- [ ] Stats visible in loadout UI (tooltips, stat bars)
- [ ] Stats modify game behavior (bomb range affects detonation pattern)
- [ ] Stat system extensible for future gear tiers

**Technical Notes:**
- Item definitions in database (item_definitions table)
- Stats stored in JSONB properties field
- Game Server reads loadout stats on match start
- Reference architecture Line 706-715 (item_definitions schema)

**Dependencies:**
- STORY-010 (Loadout selection)
- STORY-INF-001 (Database with item_definitions)

---

#### STORY-012: Visual Gear Identification

**Epic:** EPIC-002
**Priority:** Must Have
**Points:** 5
**FR:** FR-007 (Visual Gear Identification)

**User Story:**
As a player
I want to visually identify enemy gear
So that I can make informed tactical decisions

**Acceptance Criteria:**
- [ ] Player sprites change based on equipped vest (color, visual indicator)
- [ ] Bomb sprites differ by type (basic vs piercing)
- [ ] Hover over player shows gear summary (optional)
- [ ] Gear visually distinct at a glance
- [ ] Educational: new players can learn "that's a piercing bomb" visually
- [ ] Sprite variations for 3 gear tiers (basic, mid, high)

**Technical Notes:**
- Sprite asset variations: player_basic.png, player_blast_vest.png, player_heavy_vest.png
- Bomb sprites: bomb_basic.png, bomb_piercing.png, bomb_blast.png
- Client receives player loadout in initial state broadcast
- NFR-009: Visual clarity requirement

**Dependencies:**
- STORY-011 (Gear stats system)
- STORY-008 (Game Client rendering)
- Sprite assets created

---

#### STORY-013: Trader/Shop System

**Epic:** EPIC-002
**Priority:** Must Have
**Points:** 5
**FR:** FR-014 (Trader/Shop)

**User Story:**
As a player
I want to buy gear with currency
So that I can upgrade my loadout

**Acceptance Criteria:**
- [ ] Shop UI lists all available gear (bombs, vests, consumables)
- [ ] Each item shows cost in Rubles
- [ ] Purchase button enabled if player has sufficient currency
- [ ] POST /api/v1/shop/purchase validates currency and adds item to stash
- [ ] Currency deducted transactionally
- [ ] Basic gear always available (no progression lock)
- [ ] Shop inventory configurable (JSONB config or database)

**Technical Notes:**
- Persistence Service handles purchases
- ACID transaction: UPDATE currencies, INSERT stash_items
- Reference architecture Line 1014 (trader/shop API)
- Economic balance: basic loadout ~1000 credits

**Dependencies:**
- STORY-009 (Stash view)
- STORY-INF-001 (Database schema)
- Currency system functional

---

### EPIC-003: Extraction & Raid Lifecycle (7 stories, 38 points)

#### STORY-014: Raid Timer System

**Epic:** EPIC-003
**Priority:** Must Have
**Points:** 3
**FR:** FR-008 (Raid Timer)

**User Story:**
As a player
I want a visible raid timer
So that I know when the raid will end

**Acceptance Criteria:**
- [ ] Game Server initializes timer at match start (default 5-8 minutes, configurable)
- [ ] Timer decrements each tick (50ms intervals)
- [ ] Server broadcasts time_remaining_ms in state updates
- [ ] Client displays timer prominently (MM:SS format)
- [ ] Visual/audio warnings at 1 minute remaining
- [ ] When timer reaches 0, match ends (all players auto-extract without loot)
- [ ] Timer synchronized across all clients (< 50ms deviation)

**Technical Notes:**
- Timer stored as ticks_remaining (integer countdown)
- NFR-002: < 50ms timer accuracy
- Reference architecture Line 391 (Game Server manages timer)

**Dependencies:**
- STORY-002 (Game Server tick loop)
- STORY-008 (Client UI)

---

#### STORY-015: Extraction Points System

**Epic:** EPIC-003
**Priority:** Must Have
**Points:** 5
**FR:** FR-009 (Extraction Points)

**User Story:**
As a player
I want designated extraction points on the map
So that I can leave the raid successfully

**Acceptance Criteria:**
- [ ] 2-4 extraction points defined on map (specific tiles)
- [ ] Extraction requires standing on tile for N seconds (default 3s = 60 ticks)
- [ ] Extraction interrupted if player moves or takes damage
- [ ] Visual indicator: extraction zone highlighted, progress bar during extraction
- [ ] Audio feedback during extraction countdown
- [ ] Server validates extraction (position, timer, alive status)
- [ ] On successful extraction, player removed from match

**Technical Notes:**
- Extraction zone tiles marked in map template
- Server tracks extraction_progress per player (ticks_extracting)
- Reset progress if player moves or takes damage
- Reference architecture Line 411 (Game Server handles extraction)

**Dependencies:**
- STORY-014 (Raid timer)
- STORY-001 (Map with designated extraction tiles)

---

#### STORY-016: Successful Extraction Flow

**Epic:** EPIC-003
**Priority:** Must Have
**Points:** 5
**FR:** FR-010 (Successful Extraction)

**User Story:**
As a player who successfully extracts
I want to keep all my gear and loot
So that I'm rewarded for surviving

**Acceptance Criteria:**
- [ ] On extraction, server emits player.extracted event to RabbitMQ
- [ ] Persistence Service processes extraction: ADD loot to stash, keep equipped gear
- [ ] Player returned to stash screen with updated inventory
- [ ] Extraction counts as "raid survived" in statistics
- [ ] Loot value displayed in post-raid summary
- [ ] Currency earned from loot sales auto-applied (or manual sale UI)

**Technical Notes:**
- Server sends match result to Persistence Service POST /matches/{id}/results
- Persistence ACID transaction: INSERT stash_items (loot_extracted), update currencies
- Reference architecture Line 837-845 (match end flow)

**Dependencies:**
- STORY-015 (Extraction points)
- STORY-INF-001 (Database schema)
- Persistence Service operational

---

#### STORY-017: Matchmaking Queue System

**Epic:** EPIC-003
**Priority:** Must Have
**Points:** 5
**FR:** FR-016 (Solo Queue)

**User Story:**
As a player
I want to queue for a solo raid
So that I can find a match with other players

**Acceptance Criteria:**
- [ ] POST /api/v1/queue adds player to matchmaking queue (Redis sorted set)
- [ ] Queue position displayed to player
- [ ] Estimated wait time shown (based on queue size)
- [ ] When 4-8 players in queue, matchmaking creates match
- [ ] Match-ready event emitted to Match Orchestrator
- [ ] Player transitions from queue to lobby state
- [ ] DELETE /api/v1/queue/{player_id} allows leaving queue

**Technical Notes:**
- Matchmaking Service (FastAPI) manages Redis queue
- FIFO queue (no MMR in MVP)
- Reference architecture Line 446-476 (Matchmaking Service)
- Queue timeout: 60s, can launch with 4 players minimum

**Dependencies:**
- STORY-INF-002 (Matchmaking Service)
- Redis operational
- Pre-Raid UI can call queue API

---

#### STORY-018: Match Lobby and Player Spawn

**Epic:** EPIC-003
**Priority:** Must Have
**Points:** 5
**FR:** FR-018 (Raid Lobby & Spawn)

**User Story:**
As a player
I want a brief lobby before the match starts
So that I'm prepared when spawning

**Acceptance Criteria:**
- [ ] Match Orchestrator creates match instance (spawn Game Server process)
- [ ] Players receive WebSocket connection URL
- [ ] 5-10 second lobby countdown displayed
- [ ] Players spawn at random spawn points (4-8 locations on map)
- [ ] No two players spawn on same tile
- [ ] Spawn positions balanced (not all near extraction or hot zone)
- [ ] Raid timer starts when all players spawned

**Technical Notes:**
- Match Orchestrator distributes spawn positions
- Reference architecture Line 415-443 (Match Orchestrator)
- Spawn points defined in map template
- Game Server validates spawn positions

**Dependencies:**
- STORY-017 (Matchmaking queue)
- STORY-001 (Map with spawn points)
- Match Orchestrator operational

---

#### STORY-019: Game Server Process Management

**Epic:** EPIC-003
**Priority:** Must Have
**Points:** 8

**User Story:**
As the system
I want to manage Game Server processes for concurrent matches
So that multiple matches can run simultaneously

**Acceptance Criteria:**
- [ ] Match Orchestrator spawns Game Server process per match
- [ ] Game Server accepts WebSocket connections from players
- [ ] Game Server runs 20Hz tick loop (50ms intervals)
- [ ] Game Server broadcasts state updates to all clients
- [ ] Game Server handles disconnections (10s reconnect window)
- [ ] Game Server emits match lifecycle events (started, ended) to RabbitMQ
- [ ] Orchestrator monitors health and cleans up crashed servers
- [ ] Support 10-12 concurrent matches (80-96 players)

**Technical Notes:**
- Rust + Tokio async runtime for Game Server
- WebSocket server on port 9001 (dynamic port allocation per match)
- NFR-003: 10+ concurrent matches, CPU < 70%
- Reference architecture Line 390-411 (Game Server)

**Dependencies:**
- STORY-INF-002 (Match Orchestrator Service)
- RabbitMQ operational
- Game Server binary compiled

---

#### STORY-020: WebSocket Communication Protocol

**Epic:** EPIC-003
**Priority:** Must Have
**Points:** 5

**User Story:**
As a developer
I want a reliable WebSocket protocol for client-server communication
So that real-time gameplay is responsive

**Acceptance Criteria:**
- [ ] WebSocket handshake with JWT authentication
- [ ] Binary protocol using MessagePack (not JSON)
- [ ] Client → Server messages: move, place_bomb, extract
- [ ] Server → Client messages: state_update, player_died, match_ended
- [ ] State updates broadcast at 20Hz (every 50ms)
- [ ] Message sequence numbers for ordering
- [ ] Graceful disconnect handling
- [ ] Reconnect support (10s window)

**Technical Notes:**
- tokio-tungstenite for Rust WebSocket server
- rmp-serde for MessagePack serialization
- NFR-001: < 100ms input latency
- Reference architecture Line 1061-1120 (WebSocket API)

**Dependencies:**
- STORY-019 (Game Server process)
- JWT authentication working

---

### EPIC-004: Economy & Loot System (8 stories, 44 points)

#### STORY-021: Loot Spawn System

**Epic:** EPIC-004
**Priority:** Must Have
**Points:** 5
**FR:** FR-011 (Loot Spawns)

**User Story:**
As a player
I want loot to spawn on the map
So that I have something to collect and extract

**Acceptance Criteria:**
- [ ] Loot spawns at designated points on map (template-based)
- [ ] Loot tiers: Common (edge zones), Uncommon (mid-map), Rare (hot zone)
- [ ] Loot types: currency (Rubles), gear items, trade items
- [ ] Loot quantities vary per spawn point (1-3 items)
- [ ] Loot represented visually on tiles (item sprites)
- [ ] Loot not revealed to clients until nearby (anti-cheat)
- [ ] Loot configuration in database or JSONB config

**Technical Notes:**
- Loot spawn points defined in map template
- Server rolls loot at match start (RNG with weighted tiers)
- Reference architecture Line 415-443 (Match Orchestrator)
- Edge loot: 250-350 credits avg, hot zone: 750-1000 credits avg

**Dependencies:**
- STORY-001 (Map with loot spawn points)
- STORY-INF-001 (item_definitions table)

---

#### STORY-022: Loot Pickup Mechanic

**Epic:** EPIC-004
**Priority:** Must Have
**Points:** 3
**FR:** FR-012 (Loot Pickup)

**User Story:**
As a player
I want to pick up loot from the map
So that I can collect valuable items

**Acceptance Criteria:**
- [ ] Player picks up loot by pressing E (or proximity auto-pickup)
- [ ] Loot removed from map tile
- [ ] Loot added to player's raid inventory (temporary, not stash yet)
- [ ] Inventory has weight/slot limit (configurable, default 10 items)
- [ ] Player can drop loot to make room
- [ ] Visual feedback: loot icon flies to inventory UI
- [ ] Audio feedback on pickup

**Technical Notes:**
- Server validates pickup (player on correct tile, inventory space)
- Raid inventory separate from stash (only added to stash on extraction)
- Reference architecture Line 1012 (loot pickup)

**Dependencies:**
- STORY-021 (Loot spawns)
- STORY-008 (Client UI for inventory)

---

#### STORY-023: Currency System

**Epic:** EPIC-004
**Priority:** Must Have
**Points:** 3
**FR:** FR-013 (Stash & Currency)

**User Story:**
As a player
I want a currency system (Rubles)
So that I can buy gear and track wealth

**Acceptance Criteria:**
- [ ] Currency tracked in currencies table (player_id, rubles)
- [ ] Currency earned on extraction (loot value converted to rubles)
- [ ] Currency deducted on shop purchases
- [ ] Currency balance displayed in stash UI
- [ ] ACID transactions prevent negative balances
- [ ] Starting currency for new players (default 2000 rubles)

**Technical Notes:**
- Persistence Service manages currency
- ACID transaction: UPDATE currencies SET rubles = rubles + {amount}
- Constraint: CHECK (rubles >= 0)
- Reference architecture Line 690-701 (currencies table)

**Dependencies:**
- STORY-INF-001 (Database schema)
- Persistence Service operational

---

#### STORY-024: Loot Value and Sale System

**Epic:** EPIC-004
**Priority:** Must Have
**Points:** 5

**User Story:**
As a player
I want to sell loot for currency
So that I can convert extracted items to buying power

**Acceptance Criteria:**
- [ ] Each item has a value property (stored in item_definitions)
- [ ] Sell button in stash UI for non-equipped items
- [ ] POST /api/v1/stash/sell/{item_id} validates ownership and removes item
- [ ] Currency credited = item value (no trader markup in MVP)
- [ ] ACID transaction: DELETE stash_item, UPDATE currencies
- [ ] Sold items cannot be recovered
- [ ] Confirmation dialog before sale

**Technical Notes:**
- Item values calibrated for economic balance
- Basic gear: ~200-500 rubles
- Mid-tier gear: ~1000-2000 rubles
- Rare loot: ~1500-3000 rubles
- Reference FR-015 (economic floor: rebuild in 3-4 raids)

**Dependencies:**
- STORY-023 (Currency system)
- STORY-009 (Stash UI)

---

#### STORY-025: Economic Floor Calibration

**Epic:** EPIC-004
**Priority:** Should Have
**Points:** 5
**FR:** FR-015 (Economic Floor)

**User Story:**
As a player who lost everything
I want to rebuild a basic loadout in 3-4 low-risk raids
So that I don't feel stuck

**Acceptance Criteria:**
- [ ] Edge zone loot calibrated to yield 250-350 credits per raid (avg)
- [ ] Basic competitive loadout costs ~1000 credits (3x basic bombs + basic vest)
- [ ] Player always has access to starter loadout (free basic bombs or debt system)
- [ ] Economic metrics logged: loot collected, currency earned, loadout cost
- [ ] Playtest validation: player can rebuild from zero in 15-20 minutes
- [ ] Configuration file for economic tuning

**Technical Notes:**
- This is a tuning/balancing story, not pure implementation
- Create economic config YAML with loot values, gear costs
- Implement analytics logging for economic events
- Iterate based on playtesting data
- Reference PRD success metrics (15-20 min rebuild time)

**Dependencies:**
- STORY-021 through STORY-024 (full economy loop functional)
- Multiple playtest sessions

---

#### STORY-026: Stash Persistence and ACID Guarantees

**Epic:** EPIC-004
**Priority:** Must Have
**Points:** 5

**User Story:**
As a player
I want my stash to never be corrupted by server crashes
So that I don't lose items due to bugs

**Acceptance Criteria:**
- [ ] All stash mutations use PostgreSQL ACID transactions
- [ ] Mid-raid crash = treat as death (lose equipped gear, keep pre-raid stash)
- [ ] Pre-raid loadout snapshots stored in match_participants JSONB
- [ ] Rollback on transaction failure (no partial updates)
- [ ] Integration tests for crash scenarios
- [ ] Audit log entries for all stash changes

**Technical Notes:**
- Reference architecture Line 847-873 (transaction boundaries)
- NFR-007: Crash recovery requirement
- Use asyncpg for PostgreSQL transactions
- Test: kill Persistence Service mid-transaction, verify rollback

**Dependencies:**
- STORY-INF-001 (Database schema)
- Persistence Service functional

---

#### STORY-027: Match Results Processing

**Epic:** EPIC-004
**Priority:** Must Have
**Points:** 8

**User Story:**
As the system
I want to process match results reliably
So that player progression is accurate

**Acceptance Criteria:**
- [ ] Game Server emits match.ended event with participant outcomes
- [ ] Persistence Service POST /matches/{id}/results processes all outcomes
- [ ] ACID transaction:
  - For extractors: INSERT loot items, UPDATE currency
  - For deaths: DELETE equipped items
  - UPDATE match_participants with stats
  - INSERT audit_logs
- [ ] Transaction isolation prevents race conditions
- [ ] Failed transactions retry with exponential backoff
- [ ] Match results idempotent (can replay event safely)

**Technical Notes:**
- Reference architecture Line 832-845 (match end flow)
- Critical for economic integrity
- RabbitMQ durable queues ensure event delivery
- Match results processing must complete before player re-queues

**Dependencies:**
- STORY-019 (Game Server emits events)
- STORY-026 (ACID transactions)
- RabbitMQ operational

---

#### STORY-028: Starter Loadout and Debt System

**Epic:** EPIC-004
**Priority:** Must Have
**Points:** 5

**User Story:**
As a new player or broke player
I want access to a starter loadout
So that I can always play

**Acceptance Criteria:**
- [ ] Players with zero rubles can equip "Starter Loadout" (free basic bombs)
- [ ] Starter loadout: 3x basic bombs, no vest
- [ ] Starter loadout cannot be sold (bound, non-transferable)
- [ ] Starter loadout respawns if lost (always available)
- [ ] Optional: Debt system allows buying gear on credit (max 500 rubles debt)
- [ ] Debt repaid automatically from loot extraction

**Technical Notes:**
- Prevents "stuck" state where player has no gear and no currency
- FR-015 economic floor principle
- Starter items marked with is_starter flag in database
- Simple implementation: auto-add starter bombs if player has 0 bombs and <100 rubles

**Dependencies:**
- STORY-023 (Currency system)
- STORY-010 (Loadout selection)

---

### EPIC-005: Map System & Zones (6 stories, 31 points)

#### STORY-029: Map Template System

**Epic:** EPIC-005
**Priority:** Must Have
**Points:** 5
**FR:** FR-021 (Grid-Based Map)

**User Story:**
As a developer
I want a template-based map generation system
So that maps are hand-crafted but varied

**Acceptance Criteria:**
- [ ] Map templates defined in JSON/YAML (grid layout, spawn points, extraction points, loot spawns)
- [ ] Template includes: tile types, spawn positions, extraction positions, loot spawn positions
- [ ] Procedural variation: randomize destructible block positions within zones
- [ ] Map validation: ensure all spawn/extraction points are accessible via pathfinding
- [ ] At least 1 complete map template ("Factory" map for MVP)
- [ ] Map templates loaded at server startup

**Technical Notes:**
- Template structure: {width, height, tiles, spawn_points[], extraction_points[], loot_spawns[]}
- Pathfinding validation: BFS/Dijkstra to verify accessibility
- Reference architecture Line 437 (grid-based map)
- Map size: 20x20 tiles (MVP), expandable

**Dependencies:**
- STORY-001 (Grid data structure)

---

#### STORY-030: Map Zone System (Risk Tiers)

**Epic:** EPIC-005
**Priority:** Should Have
**Points:** 3
**FR:** FR-022 (Map Zones)

**User Story:**
As a player
I want distinct map zones with different risk/reward
So that I can choose my playstyle

**Acceptance Criteria:**
- [ ] Map divided into 3 zones: Edge (safe, low loot), Mid (moderate), Hot (dangerous, high loot)
- [ ] Zone boundaries defined in map template (metadata per tile or region)
- [ ] Loot spawn tiers correspond to zones (edge = common, hot = rare)
- [ ] Visual indicators for zones (color tint, minimap overlay)
- [ ] Spawn points favor edge zones (safer start)
- [ ] Hot zone centrally located (geographic teaches risk)

**Technical Notes:**
- Zone enum: Edge, Mid, Hot
- Loot tier mapping: Edge → Common, Mid → Uncommon, Hot → Rare
- Reference architecture Line 1816 (Game Server defines loot spawn tiers)
- Balance: edge loot 250-350 credits, hot loot 750-1000 credits

**Dependencies:**
- STORY-029 (Map template system)
- STORY-021 (Loot spawns)

---

#### STORY-031: Destructible Blocks System

**Epic:** EPIC-005
**Priority:** Could Have
**Points:** 5

**User Story:**
As a player
I want destructible blocks on the map
So that I can create new paths with bombs

**Acceptance Criteria:**
- [ ] Destructible blocks defined in map template
- [ ] Bomb blast destroys destructible blocks (removed from grid)
- [ ] Visual change when block destroyed (debris sprite or empty tile)
- [ ] Pathfinding updated after destruction (new paths opened)
- [ ] Blocks do not respawn during match
- [ ] Strategic placement (blocks can hide loot or extraction routes)

**Technical Notes:**
- Tile enum includes DestructibleBlock variant
- Blast propagation removes destructible blocks
- Client syncs destroyed blocks via state updates
- Optional: blocks have HP for multi-hit destruction (v2 feature)

**Dependencies:**
- STORY-004 (Bomb detonation)
- STORY-029 (Map template)

---

#### STORY-032: Map Pathfinding Validation

**Epic:** EPIC-005
**Priority:** Must Have
**Points:** 3

**User Story:**
As a developer
I want to validate map accessibility
So that all spawn/extraction points are reachable

**Acceptance Criteria:**
- [ ] Pathfinding algorithm (BFS or Dijkstra) implemented
- [ ] Map validation function: validate_map_connectivity(map) -> Result
- [ ] Validation checks:
  - All spawn points can reach at least one extraction point
  - No isolated tiles (dead ends are ok, isolated regions are not)
- [ ] Validation runs on map template load
- [ ] Server refuses to start match with invalid map
- [ ] Error messages indicate which points are unreachable

**Technical Notes:**
- Run BFS from each spawn point, verify extraction reachable
- Useful for procedural variation (ensure randomized blocks don't block paths)
- Reference architecture Line 437 (map validation ensures accessibility)

**Dependencies:**
- STORY-029 (Map template system)

---

#### STORY-033: Map Loading and Caching

**Epic:** EPIC-005
**Priority:** Must Have
**Points:** 3

**User Story:**
As a developer
I want efficient map loading and caching
So that match creation is fast

**Acceptance Criteria:**
- [ ] Map templates loaded from disk at Game Server startup
- [ ] Parsed templates cached in memory (HashMap<map_id, Map>)
- [ ] Match Orchestrator sends map_id to Game Server on match create
- [ ] Game Server clones cached map for match instance
- [ ] Map changes during match don't affect cached template
- [ ] Support map hot-reload (optional, Could Have)

**Technical Notes:**
- Templates stored in /maps directory (JSON/YAML files)
- Rust: use serde for deserialization
- Clone template for each match instance (destructible blocks modified per match)

**Dependencies:**
- STORY-029 (Map template system)
- STORY-019 (Game Server process)

---

#### STORY-034: Minimap and Map UI

**Epic:** EPIC-005
**Priority:** Could Have
**Points:** 5

**User Story:**
As a player
I want a minimap overlay
So that I can orient myself and plan routes

**Acceptance Criteria:**
- [ ] Minimap displayed in corner of screen
- [ ] Shows full map layout (walls, open tiles)
- [ ] Player position indicator (dot or arrow)
- [ ] Extraction points marked (icons)
- [ ] Zone boundaries visible (color-coded)
- [ ] Optional: Other players visible (fog of war or full visibility)
- [ ] Minimap scales with screen resolution

**Technical Notes:**
- Could Have priority - not critical for MVP
- Render minimap as separate Bevy layer
- Update player position each frame
- Reference NFR-009 (visual clarity requirement)

**Dependencies:**
- STORY-008 (Game Client rendering)
- STORY-029 (Map template)

---

## Sprint Allocation

**Assumption:** 1 senior developer, 2-week sprints, 30 points capacity per sprint

### Sprint 1: Infrastructure Foundation (Weeks 1-2) - 21/30 points (70%)

**Goal:** Establish infrastructure foundation and database layer

**Stories:**
- STORY-INF-001: Database Schema and Migrations (8 points)
- STORY-INF-002: Backend Service Scaffolding (8 points)
- STORY-INF-003: CI/CD Pipeline (5 points)

**Total:** 21 points

**Deliverables:**
- PostgreSQL database with complete schema
- 4 FastAPI services running in Docker Compose
- GitHub Actions CI/CD pipeline enforcing quality

**Risks:**
- Database migration complexity
- Docker Compose networking issues

**Dependencies:**
- PostgreSQL 16 installed
- Docker and Docker Compose installed
- GitHub repository set up

---

### Sprint 2: Map System & Core Movement (Weeks 3-4) - 29/30 points (97%)

**Goal:** Build map foundation and player movement mechanics

**Stories:**
- STORY-001: Grid-Based Map Data Structure (5 points)
- STORY-029: Map Template System (5 points)
- STORY-032: Map Pathfinding Validation (3 points)
- STORY-033: Map Loading and Caching (3 points)
- STORY-002: Player Movement System (5 points)
- STORY-008: Game Client Rendering (Bevy) (8 points)

**Total:** 29 points

**Deliverables:**
- Grid-based map system with templates
- Pathfinding validation
- Player movement with WASD controls
- Bevy client rendering grid and players

**Risks:**
- Bevy learning curve
- WebSocket integration complexity

**Dependencies:**
- Bevy engine integrated
- Rust project structure set up
- Sprite assets available

---

### Sprint 3: Combat Mechanics (Weeks 5-6) - 30/30 points (100%)

**Goal:** Implement core bomb combat system

**Stories:**
- STORY-003: Bomb Placement Mechanic (5 points)
- STORY-004: Bomb Detonation and Blast Pattern (5 points)
- STORY-005: Player Health and Damage System (3 points)
- STORY-006: Death Handling and Item Loss (5 points)
- STORY-007: Death Feedback and Replay (3 points)
- STORY-019: Game Server Process Management (8 points)

**Total:** 29 points

**Deliverables:**
- Bomb placement and detonation
- Health/damage/death system
- Death feedback UI
- Game Server managing matches

**Risks:**
- Bomb timer synchronization (NFR-002)
- Game Server performance under load

**Dependencies:**
- Sprint 2 complete (movement and rendering)
- WebSocket protocol defined

---

### Sprint 4: Matchmaking & Raid Lifecycle (Weeks 7-8) - 28/30 points (93%)

**Goal:** Enable end-to-end raid flow from queue to extraction

**Stories:**
- STORY-014: Raid Timer System (3 points)
- STORY-015: Extraction Points System (5 points)
- STORY-016: Successful Extraction Flow (5 points)
- STORY-017: Matchmaking Queue System (5 points)
- STORY-018: Match Lobby and Player Spawn (5 points)
- STORY-020: WebSocket Communication Protocol (5 points)

**Total:** 28 points

**Deliverables:**
- Matchmaking queue with Redis
- Full raid lifecycle: queue → lobby → spawn → timer → extraction
- WebSocket protocol finalized

**Risks:**
- Matchmaking timing issues (match creation delays)
- WebSocket disconnect handling

**Dependencies:**
- Sprint 3 complete (combat system)
- Redis operational

---

### Sprint 5: Loadout & Gear System (Weeks 9-10) - 25/30 points (83%)

**Goal:** Enable pre-raid loadout customization and gear stats

**Stories:**
- STORY-009: Stash View and Inventory Management (5 points)
- STORY-010: Pre-Raid Loadout Selection (5 points)
- STORY-011: Gear Stats Modification System (5 points)
- STORY-012: Visual Gear Identification (5 points)
- STORY-013: Trader/Shop System (5 points)

**Total:** 25 points

**Deliverables:**
- Pre-Raid UI (React) with stash and loadout
- Gear stats system with multiple bomb/vest types
- Visual gear identification in-game
- Trader/shop for buying gear

**Risks:**
- React drag-drop UI complexity
- Gear balance tuning

**Dependencies:**
- Sprint 1 complete (database and services)
- React app scaffolded

---

### Sprint 6: Economy & Loot System (Weeks 11-12) - 31/30 points (103%)

**Goal:** Implement loot spawns, collection, and economic loop

**Stories:**
- STORY-021: Loot Spawn System (5 points)
- STORY-022: Loot Pickup Mechanic (3 points)
- STORY-023: Currency System (3 points)
- STORY-024: Loot Value and Sale System (5 points)
- STORY-026: Stash Persistence and ACID Guarantees (5 points)
- STORY-027: Match Results Processing (8 points)

**Total:** 29 points

**Deliverables:**
- Loot spawns in map zones
- Loot pickup and raid inventory
- Currency system with buy/sell
- Match results processing with ACID transactions

**Risks:**
- Economic balance calibration
- ACID transaction complexity

**Dependencies:**
- Sprint 5 complete (loadout system)
- Sprint 4 complete (extraction flow)

---

### Sprint 7: Polish & Economic Tuning (Weeks 13-14) - 24/30 points (80%)

**Goal:** Finalize MVP with economic tuning and quality-of-life features

**Stories:**
- STORY-025: Economic Floor Calibration (5 points)
- STORY-028: Starter Loadout and Debt System (5 points)
- STORY-030: Map Zone System (Risk Tiers) (3 points)
- STORY-031: Destructible Blocks System (5 points) - Could Have
- STORY-034: Minimap and Map UI (5 points) - Could Have

**Total:** 23 points

**Deliverables:**
- Economic balance validated (15-20 min rebuild time)
- Starter loadout for broke players
- Map zones with visual indicators
- Optional: Destructible blocks and minimap

**Risks:**
- Economic tuning requires playtesting iterations
- Could Have features may slip

**Dependencies:**
- All prior sprints complete
- Playtesting sessions conducted

---

## Epic Traceability

| Epic ID | Epic Name | Stories | Total Points | Sprints |
|---------|-----------|---------|--------------|---------|
| Infrastructure | Infrastructure Foundation | STORY-INF-001, INF-002, INF-003 | 21 points | Sprint 1 |
| EPIC-001 | Core Combat & Death System | STORY-001, 002, 003, 004, 005, 006, 007, 008 | 40 points | Sprint 2-3 |
| EPIC-002 | Loadout & Gear System | STORY-009, 010, 011, 012, 013 | 25 points | Sprint 5 |
| EPIC-003 | Extraction & Raid Lifecycle | STORY-014, 015, 016, 017, 018, 019, 020 | 38 points | Sprint 3-4 |
| EPIC-004 | Economy & Loot System | STORY-021, 022, 023, 024, 025, 026, 027, 028 | 44 points | Sprint 6-7 |
| EPIC-005 | Map System & Zones | STORY-029, 030, 031, 032, 033, 034 | 31 points | Sprint 2, 7 |

**Total:** 199 points across 37 stories

---

## Functional Requirements Coverage

| FR ID | FR Name | Story | Sprint |
|-------|---------|-------|--------|
| FR-001 | Tile-Based Player Movement | STORY-001, STORY-002 | 2 |
| FR-002 | Bomb Placement | STORY-003 | 3 |
| FR-003 | Bomb Detonation Pattern | STORY-004 | 3 |
| FR-004 | Player Health and Death | STORY-005, STORY-006 | 3 |
| FR-005 | Pre-Raid Loadout Selection | STORY-010 | 5 |
| FR-006 | Gear Stats Modification | STORY-011 | 5 |
| FR-007 | Visual Gear Identification | STORY-012 | 5 |
| FR-008 | Raid Timer | STORY-014 | 4 |
| FR-009 | Extraction Points | STORY-015 | 4 |
| FR-010 | Successful Extraction | STORY-016 | 4 |
| FR-011 | Loot Spawns | STORY-021 | 6 |
| FR-012 | Loot Pickup | STORY-022 | 6 |
| FR-013 | Stash & Currency | STORY-009, STORY-023 | 5, 6 |
| FR-014 | Trader/Shop | STORY-013 | 5 |
| FR-015 | Economic Floor | STORY-025, STORY-028 | 7 |
| FR-016 | Solo Queue | STORY-017 | 4 |
| FR-017 | Map Selection | Not in MVP (Could Have) | - |
| FR-018 | Raid Lobby & Spawn | STORY-018 | 4 |
| FR-019 | Death = Lose All | STORY-006 | 3 |
| FR-020 | Death Replay/Feedback | STORY-007 | 3 |
| FR-021 | Grid-Based Map | STORY-001, STORY-029 | 2 |
| FR-022 | Map Zones (Risk Tiers) | STORY-030 | 7 |

**Coverage:** 21/22 FRs (95.5%) - FR-017 intentionally excluded (Could Have)

---

## Risks and Mitigation

### High Priority Risks

**1. Bomb Timer Synchronization (NFR-002)**
- **Risk:** Timer deviation > 50ms across clients causes unfair deaths
- **Mitigation:**
  - Implement deterministic tick-based timers in Sprint 3
  - Validate with multi-client testing
  - Log timestamp deltas and measure p95 latency
- **Story:** STORY-004 (Bomb Detonation)

**2. Database Transaction Integrity (NFR-007)**
- **Risk:** Server crash during match results processing corrupts stashes
- **Mitigation:**
  - Use PostgreSQL ACID transactions strictly
  - Test crash scenarios (kill Persistence Service mid-transaction)
  - Verify rollback behavior with integration tests
- **Story:** STORY-026, STORY-027

**3. Economic Balance (FR-015)**
- **Risk:** Economy too punishing (can't rebuild) or too generous (no gear fear)
- **Mitigation:**
  - Implement economic logging early (Sprint 6)
  - Run playtesting sessions with rebuild scenarios
  - Tune loot values based on data (Sprint 7)
  - Have configuration file for easy tuning
- **Story:** STORY-025

### Medium Priority Risks

**4. WebSocket Scalability (NFR-003)**
- **Risk:** Server can't handle 10+ concurrent matches under load
- **Mitigation:**
  - Load test Game Server in Sprint 3
  - Monitor CPU usage with 12 concurrent matches
  - Optimize tick loop and broadcast logic if needed
- **Story:** STORY-019

**5. Bevy Learning Curve**
- **Risk:** Team unfamiliar with Bevy, slows Sprint 2
- **Mitigation:**
  - Allocate buffer time in Sprint 2 (29/30 points)
  - Prototype Bevy 2D rendering before sprint starts
  - Reference Bevy documentation and examples
- **Story:** STORY-008

**6. Matchmaking Timing**
- **Risk:** Queue fills slowly, matches don't start
- **Mitigation:**
  - Allow matches with 4 minimum players (not just 8)
  - Implement 60s timeout for match creation
  - Queue status visible to users
- **Story:** STORY-017

### Low Priority Risks

**7. Cross-Platform Build Issues (NFR-012)**
- **Risk:** Bevy builds fail on macOS or Linux
- **Mitigation:**
  - Test builds early in Sprint 2
  - Use GitHub Actions to build all platforms in CI
  - Community support for Bevy cross-platform issues
- **Story:** STORY-008

**8. Sprite Asset Availability**
- **Risk:** No sprite assets for players, bombs, explosions
- **Mitigation:**
  - Use placeholder sprites in early sprints
  - Commission or create final sprites by Sprint 5
  - shadcn/ui for Pre-Raid UI (no custom assets needed)
- **Story:** STORY-008, STORY-012

---

## Dependencies

### External Dependencies

**1. Third-Party Services**
- **PostgreSQL 16:** Required for database (Sprint 1)
- **Redis 7:** Required for matchmaking queue, sessions (Sprint 1)
- **RabbitMQ 3.13:** Required for event distribution (Sprint 1)
- **Traefik 2.10:** Required for reverse proxy (Sprint 1)

**2. Development Tools**
- **Docker & Docker Compose:** Container orchestration (Sprint 1)
- **Rust 1.75+:** Game Server and Game Client (Sprint 2)
- **Python 3.12+:** Backend services (Sprint 1)
- **Node.js 20+:** Pre-Raid UI (React) (Sprint 5)
- **Bevy 0.12+:** Game engine (Sprint 2)

**3. Assets**
- **Sprite Assets:** Player, bombs, explosions, tiles (Sprint 2-5)
- **Audio Assets:** Bomb placement, explosion, footsteps (Sprint 3-5)

### Internal Story Dependencies

**Critical Path:**
1. STORY-INF-001 → STORY-INF-002 (Database before services)
2. STORY-001 → STORY-002 (Map before movement)
3. STORY-002 → STORY-003 (Movement before bomb placement)
4. STORY-003 → STORY-004 (Bomb placement before detonation)
5. STORY-004 → STORY-005 (Detonation before damage)
6. STORY-005 → STORY-006 (Damage before death)
7. STORY-010 → STORY-016 (Loadout before extraction flow)
8. STORY-016 → STORY-027 (Extraction before match results)

**Parallel Paths:**
- UI work (STORY-009, 010, 013) can proceed while combat is implemented
- Map system (STORY-029, 032, 033) can proceed in parallel with infrastructure

---

## Definition of Done

For a story to be considered complete:
- [ ] Code implemented and committed to feature branch
- [ ] Unit tests written and passing (≥80% code coverage)
- [ ] Integration tests passing (where applicable)
- [ ] Code reviewed and approved (self-review acceptable for solo dev)
- [ ] Documentation updated (inline comments for complex logic)
- [ ] Deployed to local development environment (Docker Compose up)
- [ ] Acceptance criteria manually validated
- [ ] No critical bugs or blockers
- [ ] Sprint demo prepared (for sprint review)

---

## Team Configuration

**Assumed Configuration (update as needed):**
- Team Size: 1 senior developer (solo project)
- Sprint Length: 2 weeks
- Capacity: 30 points per sprint (60 productive hours ÷ 2 hours per point)
- Productive Hours: 6 hours/day × 10 workdays = 60 hours
- Experience: Senior (Rust + FastAPI + React expertise)

**Velocity Assumptions:**
- 1 point = ~2 hours (senior developer)
- 30 points/sprint = 60 hours productive work
- 7 sprints × 2 weeks = 14 weeks total (~3.5 months)

**Adjust capacity if:**
- Team size changes (multiply capacity by team size)
- Experience level differs (junior: 1pt=4h, mid: 1pt=3h)
- Sprint length changes (1-week sprints = ~15 points capacity)
- Part-time work (reduce capacity proportionally)

---

## Next Steps

**Immediate:** Begin Sprint 1 - Infrastructure Foundation

### Option 1: Implement Stories Sequentially
Run `/dev-story STORY-INF-001` to start with database schema

### Option 2: Create Detailed Story Documents
Run `/create-story STORY-INF-001` to generate detailed implementation plan

### Option 3: Check Sprint Status
Run `/sprint-status` to view current progress

---

## Sprint Cadence

**Sprint Rhythm:**
- **Sprint Length:** 2 weeks (10 workdays)
- **Sprint Planning:** Monday, Week 1 (identify stories, estimate, commit)
- **Daily Standup:** Daily, 15 minutes (what done, what next, blockers)
- **Sprint Review:** Friday, Week 2 (demo completed stories)
- **Sprint Retrospective:** Friday, Week 2 (what went well, what to improve)
- **Sprint Increment:** Working software deployed to dev environment

**Recommended Practices:**
- Commit code daily
- Run tests before pushing
- Keep stories small (<8 points)
- Demo progress weekly (even if sprint not done)
- Adjust estimates based on actual velocity

---

## Appendix: Story Point Calibration

**Reference Examples:**

**3 points (4-8 hours):**
- STORY-005: Player Health and Damage System
- STORY-014: Raid Timer System
- STORY-023: Currency System

**5 points (1-2 days):**
- STORY-001: Grid-Based Map Data Structure
- STORY-002: Player Movement System
- STORY-010: Pre-Raid Loadout Selection

**8 points (2-3 days):**
- STORY-INF-001: Database Schema and Migrations
- STORY-008: Game Client Rendering (Bevy)
- STORY-019: Game Server Process Management

---

**This plan was created using BMAD Method v6 - Phase 4 (Implementation Planning)**

*Full Plan Path:* `/home/delorenj/code/Thermite/docs/sprint-plan-thermite-2026-01-04.md`

---

**End of Sprint Plan**

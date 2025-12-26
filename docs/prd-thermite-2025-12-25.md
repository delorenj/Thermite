# Product Requirements Document: Thermite

**Date:** 2025-12-25
**Author:** Jarad DeLorenzo
**Version:** 1.0
**Project Type:** game
**Project Level:** 4 (MVP Scope)
**Status:** Draft

---

## Document Overview

This Product Requirements Document (PRD) defines the functional and non-functional requirements for Thermite (codename: BOMBOUT). It serves as the source of truth for what will be built and provides traceability from requirements through implementation.

**Related Documents:**
- Product Brief: N/A (Started from Creative Retreat notes in TASK.md)
- Creative Retreat Session 1: TASK.md
- Open Questions Log: open_questions.md

---

## Executive Summary

**Thermite** (BOMBOUT) is a hybrid extraction shooter combining:
- **Bomberman-style grid combat:** Tile-based movement, strategic bomb placement, deterministic space control
- **Tarkov extraction mechanics:** Gear risk/reward, loot economy, timed extraction pressure
- **Fast iteration cycles:** 5-8 minute raids (vs Tarkov's 40 minutes) enable rapid learning loops

**Core Innovation:** Compressing Tarkov's intensity into arcade format while maintaining skill expression through grid purity. Gear creates tactical options, not dominance. Death is always legible.

**MVP Scope:** Solo PvP only. No AI enemies. No secure container (pure risk). Desktop client. Single map. Focus: validate core hybrid loop.

---

## Product Goals

### Business Objectives

1. **Compress Tarkov intensity into arcade format**
   - 5-8 minute raids enable rapid iteration and learning
   - Lower time commitment barrier to extraction shooter genre

2. **Maintain gear fear in accessible package**
   - Risk/reward without 40-minute commitment
   - Death = lose all (no secure container in MVP)

3. **Preserve skill expression through grid purity**
   - Movement and bomb placement stay tile-based
   - Space control is primary skill, not twitch aim

4. **Create survivable unfairness**
   - Gear asymmetry with counterplay paths
   - New players can farm edges, veterans push hot zones
   - Always have tactical options regardless of gear disadvantage

### Success Metrics

**Core Loop Validation:**
- Average raid duration: 5-8 minutes
- Player can rebuild from zero in 15-20 minutes (3-4 edge-loot raids)
- 70%+ of players understand death cause (legibility test)

**Retention Indicators:**
- Players complete 5+ raids in first session (fast iteration hooks them)
- Players who die can successfully rebuild and re-engage
- Skill expression observable: better players win more regardless of gear

**Economic Balance:**
- Edge loot averages 250-350 credits per raid
- Basic competitive loadout costs ~1000 credits
- High-value hot zone loot 3-5x edge loot value

---

## Functional Requirements

Functional Requirements (FRs) define **what** the system does - specific features and behaviors.

Each requirement includes:
- **ID**: Unique identifier (FR-001, FR-002, etc.)
- **Priority**: Must Have / Should Have / Could Have (MoSCoW)
- **Description**: What the system should do
- **Acceptance Criteria**: How to verify it's complete

---

### FR-001: Tile-Based Player Movement

**Priority:** Must Have

**Description:**
Players move on a fixed grid (tile-based). One tile per move action. Movement is instantaneous (not interpolated animation that blocks).

**Acceptance Criteria:**
- [ ] Player occupies exactly one tile at a time
- [ ] Movement to adjacent tile (4-directional: up/down/left/right) happens in single action
- [ ] Cannot move through walls or other players
- [ ] Movement is responsive (< 50ms input lag)

**Dependencies:** None

---

### FR-002: Bomb Placement

**Priority:** Must Have

**Description:**
Players can place bombs on their current tile. Bombs have a timer before detonation.

**Acceptance Criteria:**
- [ ] Player can place bomb on tile they occupy
- [ ] Bomb remains on tile after player moves away
- [ ] Bomb detonates after timer expires (configurable, default 3 seconds)
- [ ] Player can have limited active bombs (bomb count stat)
- [ ] Cannot place bomb on tile that already has bomb

**Dependencies:** FR-001

---

### FR-003: Bomb Detonation Pattern

**Priority:** Must Have

**Description:**
When bomb detonates, it creates a blast pattern in cardinal directions (up/down/left/right) extending N tiles based on blast range stat.

**Acceptance Criteria:**
- [ ] Blast extends in 4 cardinal directions from bomb tile
- [ ] Blast range is configurable per bomb type (default 2 tiles)
- [ ] Blast stops at walls or destructible obstacles
- [ ] Blast damages players in affected tiles
- [ ] Blast visual/audio feedback is clear

**Dependencies:** FR-002

---

### FR-004: Player Health and Death

**Priority:** Must Have

**Description:**
Players have health. Bomb blasts deal damage. Zero health = death.

**Acceptance Criteria:**
- [ ] Players have health stat (default 100 HP)
- [ ] Bomb blast deals damage (default 100 = one-hit kill for basic bomb)
- [ ] Player dies when health reaches 0
- [ ] Death triggers extraction failure (lose all gear and loot)
- [ ] Dead player respawns in stash (out of raid)

**Dependencies:** FR-003

---

### FR-005: Pre-Raid Loadout Selection

**Priority:** Must Have

**Description:**
Before entering raid, player selects gear from their stash (inventory). Equipped gear determines in-raid stats.

**Acceptance Criteria:**
- [ ] Player can view stash inventory
- [ ] Player can equip bombs, armor, utility items
- [ ] Loadout has defined slots (e.g., bomb type, vest, 2x utility)
- [ ] Cannot enter raid without minimum loadout (at least basic bombs)
- [ ] Equipped items removed from stash, locked until raid ends

**Dependencies:** FR-013 (Stash)

---

### FR-006: Gear Stats Modification

**Priority:** Must Have

**Description:**
Different gear modifies player stats (bomb range, bomb count, movement speed, health).

**Acceptance Criteria:**
- [ ] Basic Bomb: range 2, count 1, damage 100
- [ ] Piercing Bomb: range 3, penetrates 1 wall, damage 100
- [ ] Blast Vest: +50 HP (survives 1 hit from basic bomb)
- [ ] Stats apply when gear equipped
- [ ] Gear balance follows "options not dominance" principle

**Dependencies:** FR-005

---

### FR-007: Visual Gear Identification

**Priority:** Must Have

**Description:**
Enemy gear is visually identifiable. Players can see what bomb types and armor opponents have.

**Acceptance Criteria:**
- [ ] Player sprite/model reflects equipped gear
- [ ] Bomb type visible when placed (different visual/color)
- [ ] Armor visible on player model
- [ ] New players can learn "that player has piercing bombs" visually

**Dependencies:** FR-006

---

### FR-008: Raid Timer

**Priority:** Must Have

**Description:**
Each raid has a countdown timer (5-8 minutes). Timer visible to all players. Raid ends when timer hits zero.

**Acceptance Criteria:**
- [ ] Timer starts when raid begins (all players spawned)
- [ ] Timer displayed prominently in UI
- [ ] Timer synchronized across all clients
- [ ] When timer expires, all remaining players extracted (no loot kept)

**Dependencies:** None

---

### FR-009: Extraction Points

**Priority:** Must Have

**Description:**
Designated tiles on map are extraction points. Player standing on extraction point can extract (leave raid with loot/gear).

**Acceptance Criteria:**
- [ ] 2-4 extraction points per map
- [ ] Extraction requires standing on tile for N seconds (e.g., 3 sec)
- [ ] Extraction interrupted if player moves or takes damage
- [ ] Extraction point locations visible on map
- [ ] Visual/audio feedback during extraction countdown

**Dependencies:** FR-021 (Map)

---

### FR-010: Successful Extraction

**Priority:** Must Have

**Description:**
When player extracts, they keep all equipped gear + collected loot. Returned to stash screen.

**Acceptance Criteria:**
- [ ] All items in inventory added to stash
- [ ] All equipped gear returned to stash
- [ ] Player removed from raid
- [ ] Extraction counts as "raid survived" for stats

**Dependencies:** FR-009, FR-013 (Stash)

---

### FR-011: Loot Spawns

**Priority:** Must Have

**Description:**
Loot items spawn at designated points on map. Loot varies in value (edge loot, mid-map, hot zone).

**Acceptance Criteria:**
- [ ] Loot spawns at round start (template-based variation)
- [ ] Loot tiers: Common (edge), Uncommon (mid), Rare (hot zone)
- [ ] Loot represented visually on tiles
- [ ] Different loot types: currency, gear, trade items

**Dependencies:** FR-021 (Map), FR-022 (Zones)

---

### FR-012: Loot Pickup

**Priority:** Must Have

**Description:**
Player can pick up loot from tiles. Loot added to raid inventory.

**Acceptance Criteria:**
- [ ] Player picks up loot by standing on tile and using pickup action
- [ ] Loot removed from map tile
- [ ] Loot added to player's raid inventory
- [ ] Inventory has weight/slot limit
- [ ] Player can drop loot to make room

**Dependencies:** FR-011

---

### FR-013: Stash & Currency

**Priority:** Must Have

**Description:**
Players have persistent stash (inventory between raids). Currency used to buy gear.

**Acceptance Criteria:**
- [ ] Stash persists across raids
- [ ] Stash has storage limit (expandable in v2)
- [ ] Currency system (e.g., "Credits")
- [ ] Sell loot for credits
- [ ] Buy gear with credits

**Dependencies:** None (foundational system)

---

### FR-014: Trader/Shop

**Priority:** Must Have

**Description:**
Vendor screen where players buy gear before raids.

**Acceptance Criteria:**
- [ ] Shop UI lists available gear
- [ ] Each gear shows cost in credits
- [ ] Player can purchase if sufficient credits
- [ ] Purchased gear added to stash
- [ ] Basic gear always available (no progression lock)

**Dependencies:** FR-013

---

### FR-015: Economic Floor (15-20min rebuild)

**Priority:** Should Have

**Description:**
Player who loses everything can rebuild basic loadout through 3-4 low-risk edge-loot raids.

**Acceptance Criteria:**
- [ ] Edge loot value calibrated to earn basic loadout cost in 3-4 raids
- [ ] Basic loadout cost defined (e.g., 1000 credits)
- [ ] Average edge loot per raid: 250-350 credits
- [ ] Player always has access to starter loadout (debt system or free basic bombs)

**Dependencies:** FR-011, FR-013, FR-014, FR-022

---

### FR-016: Solo Queue

**Priority:** Must Have

**Description:**
Player queues for solo raid. Matchmaking finds 4-8 solo players and starts raid.

**Acceptance Criteria:**
- [ ] Player clicks "Find Raid" from stash screen
- [ ] Matchmaking pairs 4-8 players (configurable)
- [ ] Raid starts when enough players ready or timeout (60 sec)
- [ ] Can launch with minimum players (4) if queue slow

**Dependencies:** None

---

### FR-017: Map Selection

**Priority:** Could Have

**Description:**
Player can select which map to queue for.

**Acceptance Criteria:**
- [ ] At least 1 map available in MVP
- [ ] If multiple maps, player chooses before queue
- [ ] Matchmaking only pairs players on same map choice

**Dependencies:** FR-016, FR-021

---

### FR-018: Raid Lobby & Spawn

**Priority:** Must Have

**Description:**
Brief lobby before raid starts. All players spawn at designated spawn points when raid begins.

**Acceptance Criteria:**
- [ ] 5-10 second countdown before raid start
- [ ] Players spawn at random spawn points (4-8 locations on map)
- [ ] No two players spawn on same tile
- [ ] Raid timer starts when all players spawned

**Dependencies:** FR-016, FR-021

---

### FR-019: Death = Lose All

**Priority:** Must Have

**Description:**
When player dies, they lose all equipped gear and collected loot. No secure container.

**Acceptance Criteria:**
- [ ] Death event removes all items from player
- [ ] Lost gear not returned (no insurance in MVP)
- [ ] Player returns to stash screen empty-handed
- [ ] Death counts as "raid failed" for stats

**Dependencies:** FR-004

---

### FR-020: Death Replay/Feedback

**Priority:** Must Have

**Description:**
When player dies, show 3-5 second replay or freeze-frame showing what killed them.

**Acceptance Criteria:**
- [ ] Show bomb that dealt killing blow
- [ ] Show damage source clearly
- [ ] Show enemy player who placed bomb (if applicable)
- [ ] Educational: player learns from death

**Dependencies:** FR-004

---

### FR-021: Grid-Based Map

**Priority:** Must Have

**Description:**
Map is NxM tile grid with walls, destructible blocks, open tiles, and spawn/extraction/loot points.

**Acceptance Criteria:**
- [ ] Map represented as 2D grid (e.g., 20x20 tiles)
- [ ] Tile types: wall (impassable), destructible block, open floor
- [ ] Template-based generation (hand-crafted skeleton, procedural variation)
- [ ] Map validation ensures all spawn/extraction points accessible

**Dependencies:** None (foundational system)

---

### FR-022: Map Zones (Risk Tiers)

**Priority:** Should Have

**Description:**
Map has distinct zones: edge (low-risk, low-value), mid (moderate), hot zone (high-risk, high-value).

**Acceptance Criteria:**
- [ ] Edge zone: safe starting area, common loot
- [ ] Mid zone: moderate danger, uncommon loot
- [ ] Hot zone: central high-value area, rare loot, high player traffic
- [ ] Geography teaches risk (center = danger)

**Dependencies:** FR-021

---

## Non-Functional Requirements

Non-Functional Requirements (NFRs) define **how** the system performs - quality attributes and constraints.

---

### NFR-001: Input Responsiveness

**Priority:** Must Have

**Description:**
Player input (movement, bomb placement) must feel responsive in real-time grid combat.

**Acceptance Criteria:**
- [ ] Input latency < 100ms from keypress to visual feedback
- [ ] Server tick rate: 20 ticks/second minimum (50ms update interval)
- [ ] Movement commands processed within 1 server tick

**Rationale:**
Grid combat requires precise timing. Lag breaks the skill expression.

---

### NFR-002: Bomb Timer Accuracy

**Priority:** Must Have

**Description:**
Bomb detonation timing must be deterministic and synchronized across all clients.

**Acceptance Criteria:**
- [ ] Bomb timer deviation < 50ms across clients
- [ ] Detonation authoritative on server, clients show prediction
- [ ] Rollback/correction for mispredictions handled gracefully

**Rationale:**
Bomb timing is core skill expression. Desync = frustration.

---

### NFR-003: Match Capacity

**Priority:** Should Have

**Description:**
Server supports 4-8 player matches with acceptable performance.

**Acceptance Criteria:**
- [ ] Server can run 10+ concurrent matches (40-80 total players)
- [ ] CPU usage < 70% under normal load
- [ ] Match performance doesn't degrade with concurrent matches

**Rationale:**
MVP testing requires modest scale, not thousands of concurrent users.

---

### NFR-004: Authoritative Server

**Priority:** Must Have

**Description:**
All game-critical state (positions, bomb placement, health, loot) validated server-side.

**Acceptance Criteria:**
- [ ] Client sends input commands, server validates and applies
- [ ] Server rejects invalid moves (teleportation, wall-clipping)
- [ ] Server is source of truth for all damage calculations
- [ ] Loot spawn locations not sent to client until picked up (or nearby)

**Rationale:**
Grid-based game is easier to validate than free movement. Anti-cheat surface is manageable.

---

### NFR-005: Input Validation

**Priority:** Must Have

**Description:**
Server validates all client inputs against game rules.

**Acceptance Criteria:**
- [ ] Rate limiting on actions (no 100 bombs/second exploits)
- [ ] Movement bounds checking (can't move off-grid)
- [ ] Inventory validation (can't spawn items client-side)
- [ ] State sanity checks (dead players can't move)

**Rationale:**
Prevents trivial exploits. More sophisticated anti-cheat deferred to v2.

---

### NFR-006: MVP Uptime Target

**Priority:** Should Have

**Description:**
Service available for testing during development/alpha.

**Acceptance Criteria:**
- [ ] 90% uptime during testing windows (not 24/7 requirement for MVP)
- [ ] Planned downtime for updates acceptable
- [ ] Match in progress can complete even if matchmaking down

**Rationale:**
MVP doesn't need production SLA. Need enough stability to gather feedback.

---

### NFR-007: Crash Recovery

**Priority:** Should Have

**Description:**
Server crash doesn't corrupt player stash or cause item loss.

**Acceptance Criteria:**
- [ ] Stash state persisted to database (not in-memory only)
- [ ] Mid-raid crash = treat as death (lose gear/loot)
- [ ] Player stash restored to pre-raid state if raid-start crash

**Rationale:**
Prevents catastrophic item loss during unstable MVP testing.

---

### NFR-008: Control Scheme

**Priority:** Must Have

**Description:**
Keyboard controls are responsive and intuitive for grid movement and actions.

**Acceptance Criteria:**
- [ ] 4-directional movement (WASD or arrow keys)
- [ ] Bomb placement on single keypress (Spacebar)
- [ ] Extraction initiation clear (E to extract)
- [ ] Control rebinding available (Could Have for MVP)

**Rationale:**
Tight controls = good game feel. Grid movement is forgiving (no analog input complexity).

---

### NFR-009: Visual Clarity

**Priority:** Must Have

**Description:**
UI communicates game state clearly. No mystery deaths.

**Acceptance Criteria:**
- [ ] Bomb blast radius visible before/during detonation
- [ ] Enemy gear visually distinct (supports FR-007)
- [ ] Death cause shown immediately (what bomb, whose bomb)
- [ ] Timer and extraction status always visible

**Rationale:**
Retreat principle: "Death must always be legible."

---

### NFR-010: Structured Logging

**Priority:** Should Have

**Description:**
All game events logged in structured format for debugging and analysis.

**Acceptance Criteria:**
- [ ] Events: player actions, bomb placements, deaths, extractions, loot pickups
- [ ] Log format: JSON with timestamps, player IDs, match IDs, event types
- [ ] Centralized log aggregation (stdout → log collector)
- [ ] Queryable by match ID or player ID

**Rationale:**
MVP will have bugs. Logs enable rapid debugging and balance tuning.

---

### NFR-011: Match Replay Data

**Priority:** Could Have

**Description:**
Server records match events for post-match replay analysis.

**Acceptance Criteria:**
- [ ] Event stream stored per match (positions, actions, outcomes)
- [ ] Can reconstruct match state from event log
- [ ] Used for debugging and balance analysis

**Rationale:**
Helpful for debugging sync issues and balancing gear. Not critical for MVP launch.

---

### NFR-012: Platform Support

**Priority:** Must Have

**Description:**
Desktop client for Windows, macOS, Linux.

**Acceptance Criteria:**
- [ ] Desktop client: Windows, macOS, Linux support
- [ ] Minimum spec: Modern CPU, 4GB RAM, integrated GPU acceptable
- [ ] Renderer: 2D grid rendering (low GPU requirements)
- [ ] Distribution: Direct download for MVP (Steam/itch.io later)

**Rationale:**
Desktop provides better performance and "real game" feel for MVP validation.

---

## Epics

Epics are logical groupings of related functionality that will be broken down into user stories during sprint planning (Phase 4).

Each epic maps to multiple functional requirements and will generate 2-10 stories.

---

### EPIC-001: Core Combat & Death System

**Description:**
Implement tile-based grid combat with bomb mechanics, player health, death handling, and death feedback. This is the foundational innovation - Bomberman-style combat on a deterministic grid.

**Functional Requirements:**
- FR-001: Tile-Based Player Movement
- FR-002: Bomb Placement
- FR-003: Bomb Detonation Pattern
- FR-004: Player Health and Death
- FR-019: Death = Lose All
- FR-020: Death Replay/Feedback

**Story Count Estimate:** 6-9 stories

**Priority:** Must Have

**Business Value:**
This epic validates the central hypothesis: does grid-based combat feel good in an extraction context? Without this, there's no game. Includes death legibility (FR-020) which is critical for learning loops.

---

### EPIC-002: Loadout & Gear System

**Description:**
Pre-raid gear selection, stat modification system, and visual gear identification. Implements "gear creates options not dominance" principle from retreat notes.

**Functional Requirements:**
- FR-005: Pre-Raid Loadout Selection
- FR-006: Gear Stats Modification
- FR-007: Visual Gear Identification

**Story Count Estimate:** 4-6 stories

**Priority:** Must Have

**Business Value:**
Enables the extraction loop's risk/reward. Players choose what to risk before raid. Visual identification supports skill expression and death legibility.

---

### EPIC-003: Extraction & Raid Lifecycle

**Description:**
Raid timer, extraction points, successful extraction flow, matchmaking queue, lobby, and spawn system. The full lifecycle from queue to extraction or death.

**Functional Requirements:**
- FR-008: Raid Timer
- FR-009: Extraction Points
- FR-010: Successful Extraction
- FR-016: Solo Queue
- FR-018: Raid Lobby & Spawn

**Story Count Estimate:** 5-8 stories

**Priority:** Must Have

**Business Value:**
This epic creates the "extraction shooter" half of the hybrid. Timer pressure + extraction points = strategic tension. Matchmaking enables multiplayer testing.

---

### EPIC-004: Economy & Loot System

**Description:**
Loot spawns, pickup, persistent stash, currency, trader/shop, and economic floor calibration. The progression and rebuild loop.

**Functional Requirements:**
- FR-011: Loot Spawns
- FR-012: Loot Pickup
- FR-013: Stash & Currency
- FR-014: Trader/Shop
- FR-015: Economic Floor (15-20min rebuild)

**Story Count Estimate:** 6-9 stories

**Priority:** Must Have

**Business Value:**
Enables the "risk/reward" economy. Players farm loot, sell for currency, buy better gear, risk it in raids. Economic floor (FR-015) ensures struggling players can recover - critical for retention.

---

### EPIC-005: Map System & Zones

**Description:**
Grid-based map generation (template + variation), map validation, and risk-tiered zones (edge/mid/hot). The geography that teaches risk.

**Functional Requirements:**
- FR-021: Grid-Based Map
- FR-022: Map Zones (Risk Tiers)

**Story Count Estimate:** 4-7 stories

**Priority:** Must Have

**Business Value:**
Maps encode the learning curve. New players farm edges safely, veterans push hot zones. Template-based generation balances hand-crafted quality with variation.

---

## User Stories (High-Level)

User stories follow the format: "As a [user type], I want [goal] so that [benefit]."

These are preliminary stories. Detailed stories will be created in Phase 4 (Implementation).

---

**From EPIC-001 (Combat):**
- As a player, I want to move on a grid with responsive controls so that I can position myself tactically
- As a player, I want to place bombs with clear visual feedback so that I can control space effectively
- As a player, I want to see exactly what killed me when I die so that I can learn and improve

**From EPIC-002 (Loadout):**
- As a player, I want to choose my gear before a raid so that I can customize my playstyle
- As a player, I want to visually identify enemy gear so that I can make informed tactical decisions

**From EPIC-003 (Extraction):**
- As a player, I want a clear extraction process so that I can secure my loot and feel rewarded
- As a player, I want to see a raid timer so that I can make strategic decisions about when to extract

**From EPIC-004 (Economy):**
- As a broke player, I want to rebuild from zero in ~15-20 minutes so that I don't feel stuck
- As a successful player, I want to sell loot and buy better gear so that I can progress and take bigger risks

**From EPIC-005 (Maps):**
- As a new player, I want safe edge zones to learn the game so that I don't get destroyed immediately
- As a veteran player, I want high-value hot zones so that I can take risks for bigger rewards

---

## User Personas

**Primary Persona: The Learner**
- New to extraction shooters or Bomberman-style games
- Wants fast iteration to learn mechanics (5-8 min raids vs Tarkov's 40 min)
- Needs death legibility to improve
- Will farm edge loot until comfortable, then push deeper
- Success: Can rebuild from zero and understand why they died

**Secondary Persona: The Veteran**
- Experienced with extraction shooters or competitive Bomberman
- Values skill expression through positioning and prediction
- Seeks high-risk/high-reward plays (hot zone loot)
- Wants gear to create tactical options, not dominance
- Success: Outplay geared opponents through superior tactics

---

## User Flows

### Flow 1: First Raid (New Player)

1. Tutorial explains tile movement, bomb placement, extraction
2. Player equips starter loadout (free basic bombs)
3. Queue for raid → Lobby → Spawn on edge of map
4. Explore edge zone, find common loot
5. Encounter another player → combat or avoid
6. Extract with loot OR die and lose gear
7. Return to stash, sell loot or restart with starter gear

### Flow 2: Core Gameplay Loop (Experienced Player)

1. View stash, check currency
2. Buy upgraded gear (Piercing Bombs, Blast Vest) from trader
3. Equip loadout, queue for raid
4. Spawn, navigate to mid/hot zone for better loot
5. PvP encounters - use gear tactically
6. Extract with valuable loot OR die and lose gear
7. Sell loot, buy replacement gear, repeat

### Flow 3: Economic Recovery (Broke Player)

1. Lost all gear in previous raid, zero currency
2. Equip starter loadout (always available)
3. Farm edge loot safely for 3-4 raids (avoid hot zones)
4. Accumulate 1000+ credits
5. Buy basic competitive loadout
6. Resume normal gameplay loop

---

## Dependencies

### Internal Dependencies

- Game engine / rendering framework (TBD - needs architecture phase)
- Authoritative server framework (TBD)
- Database for persistent stash/currency (Postgres preferred)
- Matchmaking service (custom or library)
- Event-driven messaging if microservices (RabbitMQ preferred)

### External Dependencies

- None for MVP (no third-party services, payment processing, or analytics in scope)

---

## Assumptions

1. **Network Assumption:** Players have stable internet (50ms+ latency acceptable for grid-based game, not as sensitive as FPS)
2. **Skill Assumption:** Grid combat's deterministic nature makes it easier to validate server-side than free movement (anti-cheat advantage)
3. **Balance Assumption:** Gear differences can be balanced for skill expression with iteration - not expecting perfect balance in MVP
4. **Economic Assumption:** 15-20 minute rebuild time is acceptable to players (needs validation through playtesting)
5. **Platform Assumption:** Desktop distribution via direct download is acceptable for MVP (no store integration needed)
6. **Tech Stack Assumption:** Will be defined in architecture phase (Phase 3) based on requirements here

---

## Out of Scope

Explicitly NOT in MVP (deferred to v2+):
- AI enemies / PvE content
- Squad play / team modes
- Secure container
- Insurance system
- Scav karma / reputation system
- Key system / locked areas
- Skill/progression beyond gear
- Flea market / player trading
- Hideout upgrades
- Multiple maps (single map for MVP acceptable)
- Voice chat / advanced communication
- Ranked/competitive modes
- Seasonal content / battle pass
- Cosmetic customization
- Replay system (NFR-011 is Could Have, not MVP blocker)

---

## Open Questions

### Technical (for Architecture Phase)

- Tech stack selection (engine, language, server framework)
- Network architecture specifics (WebSockets, UDP, hybrid?)
- Database schema design
- Deployment strategy (cloud provider, bare metal, hybrid?)

### Design (for Sprint Planning / Iteration)

- Exact bomb timer values (3 sec? configurable?)
- Exact blast ranges for bomb types
- Exact HP values for armor tiers
- Map grid size (20x20? 30x30?)
- Exact loot value calibration
- Extraction timer duration (3 sec? 5 sec?)

---

## Approval & Sign-off

### Stakeholders

**For MVP:**
- Product Owner: Jarad DeLorenzo
- Engineering Lead: Jarad DeLorenzo
- QA Lead: TBD (or self-testing for MVP)
- Creative Director: Creative retreat team (simulated)

### Approval Status

- [ ] Product Owner
- [ ] Engineering Lead
- [ ] Design Lead
- [ ] QA Lead

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-12-25 | Jarad DeLorenzo | Initial PRD |

---

## Next Steps

### Phase 3: Architecture

Run `/architecture` to create system architecture based on these requirements.

The architecture will address:
- All functional requirements (FRs)
- All non-functional requirements (NFRs)
- Technical stack decisions
- Data models and APIs
- System components

### Phase 4: Sprint Planning

After architecture is complete, run `/sprint-planning` to:
- Break epics into detailed user stories
- Estimate story complexity
- Plan sprint iterations
- Begin implementation

---

**This document was created using BMAD Method v6 - Phase 2 (Planning)**

*To continue: Run `/workflow-status` to see your progress and next recommended workflow.*

---

## Appendix A: Requirements Traceability Matrix

| Epic ID | Epic Name | Functional Requirements | Story Count (Est.) |
|---------|-----------|-------------------------|-------------------|
| EPIC-001 | Core Combat & Death System | FR-001, FR-002, FR-003, FR-004, FR-019, FR-020 | 6-9 stories |
| EPIC-002 | Loadout & Gear System | FR-005, FR-006, FR-007 | 4-6 stories |
| EPIC-003 | Extraction & Raid Lifecycle | FR-008, FR-009, FR-010, FR-016, FR-018 | 5-8 stories |
| EPIC-004 | Economy & Loot System | FR-011, FR-012, FR-013, FR-014, FR-015 | 6-9 stories |
| EPIC-005 | Map System & Zones | FR-021, FR-022 | 4-7 stories |

**Total Estimated Stories:** 25-39 stories

---

## Appendix B: Prioritization Details

### Functional Requirements Summary

**Must Have:** 19 FRs
- FR-001 through FR-014 (Core systems)
- FR-016 (Matchmaking)
- FR-018 through FR-021 (Raid lifecycle and maps)

**Should Have:** 2 FRs
- FR-015 (Economic floor calibration)
- FR-022 (Map zones)

**Could Have:** 1 FR
- FR-017 (Map selection)

**Total FRs:** 22

### Non-Functional Requirements Summary

**Must Have:** 7 NFRs
- NFR-001, NFR-002 (Performance)
- NFR-004, NFR-005 (Security)
- NFR-008, NFR-009 (Usability)
- NFR-012 (Platform)

**Should Have:** 4 NFRs
- NFR-003 (Capacity)
- NFR-006 (Uptime)
- NFR-007 (Crash recovery)
- NFR-010 (Logging)

**Could Have:** 1 NFR
- NFR-011 (Replay data)

**Total NFRs:** 12

### Epic Priority

All 5 epics are **Must Have** for MVP.

**Recommended Implementation Order:**
1. EPIC-001 (Combat) - Foundation
2. EPIC-005 (Maps) - Playable space
3. EPIC-002 (Loadout) & EPIC-003 (Extraction) - Parallel development
4. EPIC-004 (Economy) - After core loop validated

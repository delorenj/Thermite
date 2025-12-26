-- Thermite Database Schema
-- Version: 1.0
-- Date: 2025-12-26
-- Source: architecture-thermite-2025-12-26.md Section 5 (Data Architecture)

-- Enable UUID generation extension
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ================================================================
-- Core Player and Authentication Tables
-- ================================================================

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

COMMENT ON TABLE players IS 'Core player authentication and profile data';
COMMENT ON COLUMN players.password_hash IS 'bcrypt hashed password';

-- ================================================================
-- Economy Tables
-- ================================================================

CREATE TABLE currencies (
    player_id UUID PRIMARY KEY REFERENCES players(id) ON DELETE CASCADE,
    rubles INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT positive_balance CHECK (rubles >= 0)
);

CREATE INDEX idx_currency_player ON currencies(player_id);

COMMENT ON TABLE currencies IS 'Player currency balances with economic floor enforcement';
COMMENT ON CONSTRAINT positive_balance ON currencies IS 'Prevents negative currency (economic floor)';

-- ================================================================
-- Item System Tables
-- ================================================================

CREATE TABLE item_definitions (
    id VARCHAR(50) PRIMARY KEY, -- e.g., 'bomb_basic', 'vest_blast'
    name VARCHAR(100) NOT NULL,
    category VARCHAR(20) NOT NULL, -- 'bomb', 'vest', 'consumable'
    tier INTEGER NOT NULL CHECK (tier BETWEEN 1 AND 3),
    value INTEGER NOT NULL, -- base price in rubles
    max_stack INTEGER DEFAULT 1,
    properties JSONB -- bomb stats (range, pierce, etc.)
);

COMMENT ON TABLE item_definitions IS 'Static item definitions with stats and metadata';
COMMENT ON COLUMN item_definitions.properties IS 'Item-specific stats in JSON (e.g., bomb range, vest absorption)';

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

-- Prevent duplicate equipped items (one helmet, one vest, etc.)
CREATE UNIQUE INDEX idx_stash_unique_equipped
    ON stash_items(player_id, item_id)
    WHERE is_equipped = TRUE;

COMMENT ON TABLE stash_items IS 'Player persistent inventory (stash)';
COMMENT ON COLUMN stash_items.is_equipped IS 'Items equipped for next raid';
COMMENT ON INDEX idx_stash_unique_equipped IS 'Prevents equipping duplicate items (e.g., two helmets)';

-- ================================================================
-- Match and Matchmaking Tables
-- ================================================================

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

COMMENT ON TABLE matches IS 'Match metadata and lifecycle tracking';
COMMENT ON COLUMN matches.status IS 'Match state machine: initializing → active → completed/aborted';
COMMENT ON COLUMN matches.server_address IS 'Game Server process endpoint (IP:port)';

CREATE TABLE match_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    match_id UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    outcome VARCHAR(20), -- 'extracted', 'died', 'disconnected', 'aborted'
    spawn_position JSONB, -- {x, y} grid coordinates
    death_position JSONB, -- {x, y} if died
    loadout JSONB NOT NULL, -- snapshot of equipped items at match start
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

COMMENT ON TABLE match_participants IS 'Player participation in matches with outcome tracking';
COMMENT ON COLUMN match_participants.loadout IS 'Immutable snapshot of gear at match start (for death = lose all)';
COMMENT ON COLUMN match_participants.loot_extracted IS 'Items successfully extracted from match';
COMMENT ON CONSTRAINT unique_match_player ON match_participants IS 'Prevents duplicate player entries in same match';

-- ================================================================
-- Audit and Observability Tables
-- ================================================================

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

COMMENT ON TABLE audit_logs IS 'Structured event log for debugging and analytics';
COMMENT ON COLUMN audit_logs.event_type IS 'Dot-notation event name (e.g., player.died, item.extracted)';
COMMENT ON COLUMN audit_logs.details IS 'Event-specific metadata in JSON format';

-- ================================================================
-- Seed Data: Starter Loadout Items
-- ================================================================

-- Bomb types
INSERT INTO item_definitions (id, name, category, tier, value, max_stack, properties) VALUES
('bomb_basic', 'Basic Bomb', 'bomb', 1, 100, 3, '{"range": 3, "pierce": false, "fuse_ticks": 60}'),
('bomb_pierce', 'Pierce Bomb', 'bomb', 2, 300, 2, '{"range": 3, "pierce": true, "fuse_ticks": 60}'),
('bomb_long', 'Long-Range Bomb', 'bomb', 2, 250, 2, '{"range": 5, "pierce": false, "fuse_ticks": 60}'),
('bomb_cluster', 'Cluster Bomb', 'bomb', 3, 600, 1, '{"range": 3, "pierce": false, "fuse_ticks": 60, "cluster_count": 4}');

-- Vests (armor)
INSERT INTO item_definitions (id, name, category, tier, value, max_stack, properties) VALUES
('vest_basic', 'Basic Vest', 'vest', 1, 150, 1, '{"hp_bonus": 25, "blast_reduction": 0.2}'),
('vest_tactical', 'Tactical Vest', 'vest', 2, 400, 1, '{"hp_bonus": 50, "blast_reduction": 0.35}'),
('vest_heavy', 'Heavy Vest', 'vest', 3, 800, 1, '{"hp_bonus": 100, "blast_reduction": 0.5}');

-- Movement items
INSERT INTO item_definitions (id, name, category, tier, value, max_stack, properties) VALUES
('boots_speed', 'Speed Boots', 'movement', 2, 300, 1, '{"speed_multiplier": 1.2}'),
('boots_heavy', 'Heavy Boots', 'movement', 2, 250, 1, '{"knockback_resistance": 0.5}');

-- Consumables
INSERT INTO item_definitions (id, name, category, tier, value, max_stack, properties) VALUES
('medkit_small', 'Small Medkit', 'consumable', 1, 100, 5, '{"hp_restore": 25}'),
('medkit_large', 'Large Medkit', 'consumable', 2, 250, 3, '{"hp_restore": 50}');

-- ================================================================
-- Transaction Templates (Documented)
-- ================================================================

COMMENT ON DATABASE thermite IS 'Thermite game database - PostgreSQL 16+ with ACID transactions';

-- Critical Transaction: Post-Match Loot Distribution
-- TRANSACTION:
--   IF outcome='extracted':
--     - INSERT INTO stash_items (loot_extracted items)
--     - UPDATE currencies (add loot value)
--   IF outcome='died':
--     - DELETE FROM stash_items (equipped items from loadout)
--   - INSERT INTO audit_logs (item.extracted, item.lost, player.died)
--   - UPDATE match_participants (outcome, stats)
-- COMMIT

-- Critical Transaction: Pre-Raid Loadout Snapshot
-- TRANSACTION:
--   - SELECT stash_items WHERE player_id AND is_equipped
--   - INSERT INTO match_participants (loadout = JSONB snapshot)
--   - (Items remain in stash until match outcome determined)
-- COMMIT

-- ================================================================
-- Database Integrity Validation Queries
-- ================================================================

-- Verify no negative currency balances
-- SELECT player_id, rubles FROM currencies WHERE rubles < 0;

-- Verify no duplicate equipped items
-- SELECT player_id, item_id, COUNT(*) FROM stash_items
-- WHERE is_equipped = TRUE
-- GROUP BY player_id, item_id HAVING COUNT(*) > 1;

-- Verify all match participants have loadout snapshots
-- SELECT id FROM match_participants WHERE loadout IS NULL;

-- Verify all item definitions have required properties
-- SELECT id FROM item_definitions WHERE properties IS NULL;

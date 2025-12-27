-- Thermite Database Schema - Initial Migration
-- All 7 tables: players, currencies, item_definitions, stash_items, matches, match_participants, audit_logs
-- Source: database/schema/001_initial_schema.sql

-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Players table
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

-- Currencies table
CREATE TABLE currencies (
    player_id UUID PRIMARY KEY REFERENCES players(id) ON DELETE CASCADE,
    rubles INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT positive_balance CHECK (rubles >= 0)
);

CREATE INDEX idx_currency_player ON currencies(player_id);

-- Item definitions table
CREATE TABLE item_definitions (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    category VARCHAR(20) NOT NULL,
    tier INTEGER NOT NULL CHECK (tier BETWEEN 1 AND 3),
    value INTEGER NOT NULL,
    max_stack INTEGER DEFAULT 1,
    properties JSONB
);

-- Stash items table
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
CREATE INDEX idx_stash_equipped ON stash_items(player_id, is_equipped) WHERE is_equipped = TRUE;
CREATE UNIQUE INDEX idx_stash_unique_equipped ON stash_items(player_id, item_id) WHERE is_equipped = TRUE;

-- Matches table
CREATE TABLE matches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    map_id VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL,
    started_at TIMESTAMP,
    ended_at TIMESTAMP,
    duration_seconds INTEGER,
    server_address VARCHAR(100),
    max_players INTEGER DEFAULT 8,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_match_status ON matches(status);
CREATE INDEX idx_match_started ON matches(started_at DESC);

-- Match participants table
CREATE TABLE match_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    match_id UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    outcome VARCHAR(20),
    spawn_position JSONB,
    death_position JSONB,
    loadout JSONB NOT NULL,
    loot_extracted JSONB,
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

-- Audit logs table
CREATE TABLE audit_logs (
    id BIGSERIAL PRIMARY KEY,
    event_type VARCHAR(100) NOT NULL,
    player_id UUID REFERENCES players(id),
    match_id UUID REFERENCES matches(id),
    details JSONB,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    severity VARCHAR(20)
);

CREATE INDEX idx_audit_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX idx_audit_event_type ON audit_logs(event_type);
CREATE INDEX idx_audit_player ON audit_logs(player_id) WHERE player_id IS NOT NULL;

-- Seed data
INSERT INTO item_definitions (id, name, category, tier, value, max_stack, properties) VALUES
('bomb_basic', 'Basic Bomb', 'bomb', 1, 100, 3, '{"range": 3, "pierce": false, "fuse_ticks": 60}'),
('bomb_pierce', 'Pierce Bomb', 'bomb', 2, 300, 2, '{"range": 3, "pierce": true, "fuse_ticks": 60}'),
('bomb_long', 'Long-Range Bomb', 'bomb', 2, 250, 2, '{"range": 5, "pierce": false, "fuse_ticks": 60}'),
('bomb_cluster', 'Cluster Bomb', 'bomb', 3, 600, 1, '{"range": 3, "pierce": false, "fuse_ticks": 60, "cluster_count": 4}'),
('vest_basic', 'Basic Vest', 'vest', 1, 150, 1, '{"hp_bonus": 25, "blast_reduction": 0.2}'),
('vest_tactical', 'Tactical Vest', 'vest', 2, 400, 1, '{"hp_bonus": 50, "blast_reduction": 0.35}'),
('vest_heavy', 'Heavy Vest', 'vest', 3, 800, 1, '{"hp_bonus": 100, "blast_reduction": 0.5}'),
('boots_speed', 'Speed Boots', 'movement', 2, 300, 1, '{"speed_multiplier": 1.2}'),
('boots_heavy', 'Heavy Boots', 'movement', 2, 250, 1, '{"knockback_resistance": 0.5}'),
('medkit_small', 'Small Medkit', 'consumable', 1, 100, 5, '{"hp_restore": 25}'),
('medkit_large', 'Large Medkit', 'consumable', 2, 250, 3, '{"hp_restore": 50}');

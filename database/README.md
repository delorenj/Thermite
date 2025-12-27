# Thermite Database Migrations

## Schema Overview

Production PostgreSQL 16+ database with 7 core tables:

1. **players** - Auth and profile
2. **currencies** - Economy with floor enforcement
3. **item_definitions** - Static item catalog
4. **stash_items** - Player persistent inventory
5. **matches** - Match lifecycle tracking
6. **match_participants** - Player participation records
7. **audit_logs** - Event logging for analytics

## Initial Setup

```bash
# Apply initial schema
psql postgresql://user@localhost:5432/thermite < database/schema/001_initial_schema.sql
```

## Verification

```bash
# List all tables
psql postgresql://user@localhost:5432/thermite -c "\dt"

# Check constraints
psql postgresql://user@localhost:5432/thermite -c "SELECT conname, contype FROM pg_constraint WHERE contype='c';"

# Verify seed data
psql postgresql://user@localhost:5432/thermite -c "SELECT COUNT(*) FROM item_definitions;"
```

## Key Constraints

- **Foreign Keys**: 7 total (CASCADE delete for player data)
- **CHECK Constraints**: 
  - `positive_balance` - Prevents negative currency
  - `positive_quantity` - Prevents zero/negative item stacks
- **UNIQUE Constraints**:
  - `unique_match_player` - One entry per player per match
  - `idx_stash_unique_equipped` - Only one equipped item per type

## Indexes

24 total indexes for query optimization:
- Player lookups: `idx_players_email`
- Match queries: `idx_match_status`, `idx_match_started`
- Audit logs: `idx_audit_timestamp`, `idx_audit_event_type`
- Stash queries: `idx_stash_player`, `idx_stash_equipped`

## Seed Data

11 starter items across 4 categories:
- **Bombs**: basic, pierce, long, cluster (4)
- **Vests**: basic, tactical, heavy (3)
- **Movement**: speed boots, heavy boots (2)
- **Consumables**: small/large medkits (2)

All items have JSONB properties for stats (range, damage, hp_bonus, etc.)

## Migration Strategy

- **sqlx** for Rust services (`server/migrations/`)
- **Alembic** for Python services (`services/stash-service/migrations/`)
- Both systems track the same base schema via `database/schema/`
- Forward-only migrations (no automatic rollback in production)

## Testing

```bash
# Test constraint enforcement
psql postgresql://user@localhost:5432/thermite -c "INSERT INTO currencies (player_id, rubles) VALUES (gen_random_uuid(), -100);"
# Expected: ERROR - violates check constraint "positive_balance"
```

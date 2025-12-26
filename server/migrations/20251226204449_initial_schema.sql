-- Thermite Server Migrations (sqlx)
-- Initial schema tracking
-- Note: Schema already created via database/schema/001_initial_schema.sql
-- This migration tracks the existing schema state for sqlx

-- Verify schema exists
DO $$
BEGIN
    -- Check that core tables exist
    IF NOT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'players') THEN
        RAISE EXCEPTION 'Schema not initialized. Run database/schema/001_initial_schema.sql first';
    END IF;

    IF NOT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'matches') THEN
        RAISE EXCEPTION 'Schema not initialized. Run database/schema/001_initial_schema.sql first';
    END IF;

    IF NOT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'match_participants') THEN
        RAISE EXCEPTION 'Schema not initialized. Run database/schema/001_initial_schema.sql first';
    END IF;
END $$;

-- This migration is a no-op since schema is already applied
-- Future migrations will be incremental changes to the schema

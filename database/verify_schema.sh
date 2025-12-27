#!/bin/bash
# Thermite Database Schema Verification Script

set -e

DB_URL="${DATABASE_URL:-postgresql://delorenj@localhost:5432/thermite}"

echo "=== Thermite Database Schema Verification ==="
echo "Database: $DB_URL"
echo ""

echo "1. Checking tables..."
TABLE_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public' AND table_type='BASE TABLE';")
echo "   Found $TABLE_COUNT tables (expected: 7 + migrations table)"

echo ""
echo "2. Checking foreign keys..."
FK_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM pg_constraint WHERE contype='f';")
echo "   Found $FK_COUNT foreign keys (expected: 7)"

echo ""
echo "3. Checking CHECK constraints..."
psql "$DB_URL" -c "SELECT conname, contype FROM pg_constraint WHERE contype='c';" | grep positive || echo "   ❌ CHECK constraints missing!"

echo ""
echo "4. Checking indexes..."
INDEX_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM pg_indexes WHERE schemaname='public';")
echo "   Found $INDEX_COUNT indexes (expected: ~24)"

echo ""
echo "5. Checking seed data..."
ITEM_COUNT=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM item_definitions;")
echo "   Found $ITEM_COUNT items (expected: 11)"

echo ""
echo "6. Testing constraint enforcement..."
psql "$DB_URL" -c "INSERT INTO currencies (player_id, rubles) VALUES (gen_random_uuid(), -100);" 2>&1 | grep -q "positive_balance" && echo "   ✅ Negative balance blocked" || echo "   ❌ Constraint failed!"

echo ""
echo "=== Verification Complete ==="

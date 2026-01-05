"""
Integration tests for Thermite database schema.

Tests all 7 tables, constraints, indexes, relationships, and ACID transactions.
"""

import pytest
import pytest_asyncio
from sqlalchemy import inspect, select, text
from sqlalchemy.exc import IntegrityError
from uuid import uuid4

from thermite_stash_service.database import AsyncSessionLocal, Base, engine
from thermite_stash_service.models import (
    AuditLog,
    Currency,
    ItemDefinition,
    Match,
    MatchParticipant,
    Player,
    StashItem,
)


@pytest_asyncio.fixture(scope="function")
async def db_session():
    """Provide a clean database session for each test."""
    async with AsyncSessionLocal() as session:
        yield session
        await session.rollback()


@pytest_asyncio.fixture(scope="function")
async def clean_db():
    """Clean database before each test."""
    async with engine.begin() as conn:
        # Delete all data in reverse foreign key order
        await conn.execute(text("DELETE FROM audit_logs"))
        await conn.execute(text("DELETE FROM match_participants"))
        await conn.execute(text("DELETE FROM matches"))
        await conn.execute(text("DELETE FROM stash_items"))
        await conn.execute(text("DELETE FROM item_definitions"))
        await conn.execute(text("DELETE FROM currencies"))
        await conn.execute(text("DELETE FROM players"))
    yield


class TestTableStructure:
    """Test that all 7 tables exist with correct structure."""

    @pytest.mark.asyncio
    async def test_all_tables_exist(self):
        """Verify all 7 tables are created."""
        async with engine.begin() as conn:

            def get_tables(sync_conn):
                inspector = inspect(sync_conn)
                return inspector.get_table_names()

            tables = await conn.run_sync(get_tables)

        expected_tables = {
            "players",
            "currencies",
            "item_definitions",
            "stash_items",
            "matches",
            "match_participants",
            "audit_logs",
        }
        assert expected_tables.issubset(set(tables)), "Missing expected tables"

    @pytest.mark.asyncio
    async def test_player_table_columns(self):
        """Verify players table has correct columns."""
        async with engine.begin() as conn:

            def get_columns(sync_conn):
                inspector = inspect(sync_conn)
                return [col["name"] for col in inspector.get_columns("players")]

            columns = await conn.run_sync(get_columns)

        expected_columns = {
            "id",
            "email",
            "username",
            "password_hash",
            "created_at",
            "last_login",
            "is_active",
        }
        assert expected_columns == set(
            columns
        ), f"Expected {expected_columns}, got {set(columns)}"


class TestForeignKeyConstraints:
    """Test foreign key relationships and cascades."""

    @pytest.mark.asyncio
    async def test_currency_fk_cascade_delete(self, db_session, clean_db):
        """Deleting player should cascade to currency."""
        # Create player with currency
        player = Player(
            id=uuid4(),
            email="test@example.com",
            username="testuser",
            password_hash="hashed",
        )
        db_session.add(player)
        await db_session.flush()

        currency = Currency(player_id=player.id, rubles=1000)
        db_session.add(currency)
        await db_session.commit()

        # Delete player
        await db_session.delete(player)
        await db_session.commit()

        # Currency should be deleted
        result = await db_session.execute(
            select(Currency).where(Currency.player_id == player.id)
        )
        assert result.scalar_one_or_none() is None

    @pytest.mark.asyncio
    async def test_stash_item_fk_to_player(self, db_session, clean_db):
        """StashItem requires valid player_id."""
        # Create item definition
        item = ItemDefinition(
            id="helmet_basic", name="Basic Helmet", category="armor", tier=1, value=100
        )
        db_session.add(item)
        await db_session.commit()

        # Try to create stash item with invalid player_id
        stash_item = StashItem(
            id=uuid4(),
            player_id=uuid4(),  # Non-existent player
            item_id="helmet_basic",
            quantity=1,
        )
        db_session.add(stash_item)

        with pytest.raises(IntegrityError):
            await db_session.commit()


class TestCheckConstraints:
    """Test CHECK constraints for business rules."""

    @pytest.mark.asyncio
    async def test_positive_balance_constraint(self, db_session, clean_db):
        """Currency rubles must be >= 0."""
        player = Player(
            id=uuid4(),
            email="test@example.com",
            username="testuser",
            password_hash="hashed",
        )
        db_session.add(player)
        await db_session.flush()

        # Try to create negative balance
        currency = Currency(player_id=player.id, rubles=-100)
        db_session.add(currency)

        with pytest.raises(IntegrityError, match="positive_balance"):
            await db_session.commit()

    @pytest.mark.asyncio
    async def test_tier_range_constraint(self, db_session, clean_db):
        """ItemDefinition tier must be 1-3."""
        # Invalid tier (too high)
        item = ItemDefinition(
            id="invalid_item",
            name="Invalid Item",
            category="armor",
            tier=5,  # Invalid
            value=100,
        )
        db_session.add(item)

        with pytest.raises(IntegrityError, match="tier_check"):
            await db_session.commit()

    @pytest.mark.asyncio
    async def test_positive_quantity_constraint(self, db_session, clean_db):
        """StashItem quantity must be > 0."""
        player = Player(
            id=uuid4(),
            email="test@example.com",
            username="testuser",
            password_hash="hashed",
        )
        item = ItemDefinition(
            id="helmet_basic", name="Basic Helmet", category="armor", tier=1, value=100
        )
        db_session.add_all([player, item])
        await db_session.flush()

        # Try to create zero quantity
        stash_item = StashItem(
            id=uuid4(),
            player_id=player.id,
            item_id="helmet_basic",
            quantity=0,  # Invalid
        )
        db_session.add(stash_item)

        with pytest.raises(IntegrityError, match="positive_quantity"):
            await db_session.commit()


class TestUniqueConstraints:
    """Test unique constraints."""

    @pytest.mark.asyncio
    async def test_unique_email(self, db_session, clean_db):
        """Player email must be unique."""
        player1 = Player(
            id=uuid4(),
            email="test@example.com",
            username="user1",
            password_hash="hashed",
        )
        player2 = Player(
            id=uuid4(),
            email="test@example.com",  # Duplicate
            username="user2",
            password_hash="hashed",
        )
        db_session.add_all([player1, player2])

        with pytest.raises(IntegrityError):
            await db_session.commit()

    @pytest.mark.asyncio
    async def test_unique_match_player(self, db_session, clean_db):
        """MatchParticipant: unique (match_id, player_id)."""
        player = Player(
            id=uuid4(),
            email="test@example.com",
            username="testuser",
            password_hash="hashed",
        )
        match = Match(id=uuid4(), map_id="map_basic", status="active")
        db_session.add_all([player, match])
        await db_session.flush()

        participant1 = MatchParticipant(
            id=uuid4(),
            match_id=match.id,
            player_id=player.id,
            loadout={"helmet": "helmet_basic"},
        )
        participant2 = MatchParticipant(
            id=uuid4(),
            match_id=match.id,
            player_id=player.id,  # Duplicate
            loadout={"helmet": "helmet_basic"},
        )
        db_session.add_all([participant1, participant2])

        with pytest.raises(IntegrityError, match="unique_match_player"):
            await db_session.commit()


class TestIndexes:
    """Test indexes are created for performance."""

    @pytest.mark.asyncio
    async def test_player_email_index_exists(self):
        """Verify idx_players_email exists."""
        async with engine.begin() as conn:

            def get_indexes(sync_conn):
                inspector = inspect(sync_conn)
                return inspector.get_indexes("players")

            indexes = await conn.run_sync(get_indexes)

        index_names = [idx["name"] for idx in indexes]
        assert "idx_players_email" in index_names

    @pytest.mark.asyncio
    async def test_stash_player_index_exists(self):
        """Verify idx_stash_player exists."""
        async with engine.begin() as conn:

            def get_indexes(sync_conn):
                inspector = inspect(sync_conn)
                return inspector.get_indexes("stash_items")

            indexes = await conn.run_sync(get_indexes)

        index_names = [idx["name"] for idx in indexes]
        assert "idx_stash_player" in index_names


class TestJSONBColumns:
    """Test JSONB column storage and retrieval."""

    @pytest.mark.asyncio
    async def test_item_definition_properties_jsonb(self, db_session, clean_db):
        """ItemDefinition.properties stores JSON."""
        item = ItemDefinition(
            id="bomb_tier2",
            name="Tier 2 Bomb",
            category="bomb",
            tier=2,
            value=500,
            properties={"blast_radius": 3, "fuse_ticks": 40},
        )
        db_session.add(item)
        await db_session.commit()

        # Retrieve and verify
        result = await db_session.execute(
            select(ItemDefinition).where(ItemDefinition.id == "bomb_tier2")
        )
        retrieved = result.scalar_one()

        assert retrieved.properties == {"blast_radius": 3, "fuse_ticks": 40}

    @pytest.mark.asyncio
    async def test_match_participant_loadout_jsonb(self, db_session, clean_db):
        """MatchParticipant.loadout stores complex JSON."""
        player = Player(
            id=uuid4(),
            email="test@example.com",
            username="testuser",
            password_hash="hashed",
        )
        match = Match(id=uuid4(), map_id="map_basic", status="active")
        db_session.add_all([player, match])
        await db_session.flush()

        participant = MatchParticipant(
            id=uuid4(),
            match_id=match.id,
            player_id=player.id,
            loadout={
                "helmet": "helmet_tier2",
                "vest": "vest_tier1",
                "bombs": ["bomb_basic"] * 3,
            },
        )
        db_session.add(participant)
        await db_session.commit()

        # Retrieve and verify
        result = await db_session.execute(
            select(MatchParticipant).where(MatchParticipant.id == participant.id)
        )
        retrieved = result.scalar_one()

        assert retrieved.loadout["helmet"] == "helmet_tier2"
        assert len(retrieved.loadout["bombs"]) == 3


class TestRelationships:
    """Test SQLAlchemy relationships between models."""

    @pytest.mark.asyncio
    async def test_player_currency_relationship(self, db_session, clean_db):
        """Player has one Currency (1:1)."""
        player = Player(
            id=uuid4(),
            email="test@example.com",
            username="testuser",
            password_hash="hashed",
        )
        currency = Currency(player_id=player.id, rubles=5000)
        db_session.add_all([player, currency])
        await db_session.commit()

        # Access relationship
        result = await db_session.execute(select(Player).where(Player.id == player.id))
        retrieved_player = result.scalar_one()

        # Eagerly load currency
        await db_session.refresh(retrieved_player, ["currency"])
        assert retrieved_player.currency.rubles == 5000

    @pytest.mark.asyncio
    async def test_player_stash_items_relationship(self, db_session, clean_db):
        """Player has many StashItems (1:M)."""
        player = Player(
            id=uuid4(),
            email="test@example.com",
            username="testuser",
            password_hash="hashed",
        )
        item = ItemDefinition(
            id="helmet_basic", name="Basic Helmet", category="armor", tier=1, value=100
        )
        db_session.add_all([player, item])
        await db_session.flush()

        stash1 = StashItem(
            id=uuid4(), player_id=player.id, item_id="helmet_basic", quantity=1
        )
        stash2 = StashItem(
            id=uuid4(), player_id=player.id, item_id="helmet_basic", quantity=2
        )
        db_session.add_all([stash1, stash2])
        await db_session.commit()

        # Access relationship
        result = await db_session.execute(select(Player).where(Player.id == player.id))
        retrieved_player = result.scalar_one()

        # Eagerly load stash_items
        await db_session.refresh(retrieved_player, ["stash_items"])
        assert len(retrieved_player.stash_items) == 2


class TestACIDTransactions:
    """Test ACID transaction guarantees."""

    @pytest.mark.asyncio
    async def test_transaction_rollback_on_error(self, db_session, clean_db):
        """Failed transaction rolls back all changes."""
        player = Player(
            id=uuid4(),
            email="test@example.com",
            username="testuser",
            password_hash="hashed",
        )
        db_session.add(player)
        await db_session.flush()

        try:
            # Valid currency
            currency = Currency(player_id=player.id, rubles=1000)
            db_session.add(currency)

            # Invalid currency (negative balance)
            invalid_currency = Currency(
                player_id=player.id, rubles=-500  # Will fail CHECK constraint
            )
            db_session.add(invalid_currency)

            await db_session.commit()
        except IntegrityError:
            await db_session.rollback()

        # Verify nothing was committed
        result = await db_session.execute(
            select(Currency).where(Currency.player_id == player.id)
        )
        assert result.scalar_one_or_none() is None

    @pytest.mark.asyncio
    async def test_transaction_commit_atomicity(self, db_session, clean_db):
        """Successful transaction commits all or nothing."""
        player_id = uuid4()

        # Transaction 1: Create player and currency
        player = Player(
            id=player_id,
            email="test@example.com",
            username="testuser",
            password_hash="hashed",
        )
        currency = Currency(player_id=player_id, rubles=1000)
        db_session.add_all([player, currency])
        await db_session.commit()

        # Verify both were committed
        result_player = await db_session.execute(
            select(Player).where(Player.id == player_id)
        )
        result_currency = await db_session.execute(
            select(Currency).where(Currency.player_id == player_id)
        )

        assert result_player.scalar_one() is not None
        assert result_currency.scalar_one().rubles == 1000


class TestSeedData:
    """Test seed data for item_definitions."""

    @pytest.mark.skip(reason="Seed data not yet implemented (future story)")
    @pytest.mark.asyncio
    async def test_item_definitions_seeded(self, db_session):
        """Verify item_definitions has seed data."""
        result = await db_session.execute(select(ItemDefinition))
        items = result.scalars().all()

        # Should have basic items from seed data
        assert len(items) > 0

        # Check for specific seed items
        item_ids = [item.id for item in items]
        assert "bomb_tier1" in item_ids

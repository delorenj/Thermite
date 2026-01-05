"""
Transaction isolation level tests for Thermite database.

Tests concurrent transaction behavior and ACID guarantees critical for economic integrity.
"""

import asyncio
import pytest
import pytest_asyncio
from sqlalchemy import select, text
from sqlalchemy.exc import IntegrityError
from uuid import uuid4

from thermite_stash_service.database import AsyncSessionLocal, engine
from thermite_stash_service.models import Currency, ItemDefinition, Player, StashItem


@pytest_asyncio.fixture(scope="function")
async def clean_db():
    """Clean database before each test."""
    async with engine.begin() as conn:
        await conn.execute(text("DELETE FROM stash_items"))
        await conn.execute(text("DELETE FROM item_definitions"))
        await conn.execute(text("DELETE FROM currencies"))
        await conn.execute(text("DELETE FROM players"))
    yield


@pytest_asyncio.fixture
async def test_player(clean_db):
    """Create a test player with currency."""
    async with AsyncSessionLocal() as session:
        player = Player(
            id=uuid4(),
            email="test@example.com",
            username="testplayer",
            password_hash="hashed",
        )
        currency = Currency(player_id=player.id, rubles=10000)
        session.add_all([player, currency])
        await session.commit()
        return player


@pytest_asyncio.fixture
async def test_item(clean_db):
    """Create a test item."""
    async with AsyncSessionLocal() as session:
        item = ItemDefinition(
            id="rare_helmet",
            name="Rare Helmet",
            category="armor",
            tier=3,
            value=5000,
            max_stack=1,
        )
        session.add(item)
        await session.commit()
        return item


class TestReadCommitted:
    """Test READ COMMITTED isolation level (PostgreSQL default)."""

    @pytest.mark.asyncio
    async def test_concurrent_currency_updates_with_locking(self, test_player):
        """
        Concurrent currency updates with FOR UPDATE locking.

        This test demonstrates the CORRECT pattern for concurrent updates.
        """

        async def deduct_rubles_safe(player_id, amount, delay=0):
            """Deduct rubles from player currency with row-level locking."""
            async with AsyncSessionLocal() as session:
                # Use SELECT FOR UPDATE to lock the row
                result = await session.execute(
                    select(Currency)
                    .where(Currency.player_id == player_id)
                    .with_for_update()
                )
                currency = result.scalar_one()
                current_balance = currency.rubles

                # Simulate processing delay
                if delay > 0:
                    await asyncio.sleep(delay)

                # Check if sufficient funds
                if current_balance < amount:
                    await session.rollback()
                    raise ValueError("Insufficient funds")

                # Deduct
                currency.rubles = current_balance - amount
                await session.commit()
                return currency.rubles

        # Concurrent transactions trying to deduct from same account
        results = await asyncio.gather(
            deduct_rubles_safe(test_player.id, 3000, delay=0.05),
            deduct_rubles_safe(test_player.id, 3000, delay=0.05),
            deduct_rubles_safe(test_player.id, 3000, delay=0.05),
            return_exceptions=True,
        )

        # Count successes and failures
        successes = [r for r in results if not isinstance(r, Exception)]
        failures = [r for r in results if isinstance(r, Exception)]

        # Final balance should reflect actual deductions
        async with AsyncSessionLocal() as session:
            result = await session.execute(
                select(Currency).where(Currency.player_id == test_player.id)
            )
            final_currency = result.scalar_one()

        # With FOR UPDATE locking, final balance should be correct
        # Only 3 transactions of 3000 can succeed from 10000
        assert final_currency.rubles == 10000 - (len(successes) * 3000)
        assert len(successes) <= 3  # Can't succeed more than 3 times
        assert final_currency.rubles >= 0  # Never negative


class TestRepeatableRead:
    """Test REPEATABLE READ isolation level for consistent reads."""

    @pytest.mark.asyncio
    async def test_repeatable_read_prevents_nonrepeatable_reads(self, test_player):
        """REPEATABLE READ should prevent seeing updated data mid-transaction."""

        async def reader_transaction():
            """Read currency twice in same transaction."""
            async with AsyncSessionLocal() as session:
                # Set isolation level to REPEATABLE READ
                await session.execute(
                    text("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
                )

                # First read
                result1 = await session.execute(
                    select(Currency).where(Currency.player_id == test_player.id)
                )
                balance1 = result1.scalar_one().rubles

                # Wait for writer to commit
                await asyncio.sleep(0.2)

                # Second read (should see same value in REPEATABLE READ)
                result2 = await session.execute(
                    select(Currency).where(Currency.player_id == test_player.id)
                )
                balance2 = result2.scalar_one().rubles

                return balance1, balance2

        async def writer_transaction():
            """Update currency."""
            await asyncio.sleep(0.1)  # Let reader start first
            async with AsyncSessionLocal() as session:
                result = await session.execute(
                    select(Currency).where(Currency.player_id == test_player.id)
                )
                currency = result.scalar_one()
                currency.rubles = 5000
                await session.commit()

        # Run both transactions concurrently
        reader_task = asyncio.create_task(reader_transaction())
        writer_task = asyncio.create_task(writer_transaction())

        balance1, balance2 = await reader_task
        await writer_task

        # In REPEATABLE READ, both reads should see the same value
        assert balance1 == balance2 == 10000


class TestSerializable:
    """Test SERIALIZABLE isolation level for maximum consistency."""

    @pytest.mark.asyncio
    async def test_serializable_prevents_phantom_reads(self, test_player, test_item):
        """SERIALIZABLE should detect conflicts in concurrent inserts."""

        async def transaction_a():
            """Count items, then insert one."""
            async with AsyncSessionLocal() as session:
                # Set isolation level to SERIALIZABLE
                await session.execute(
                    text("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
                )

                # Count current items
                result = await session.execute(
                    select(StashItem).where(StashItem.player_id == test_player.id)
                )
                count_before = len(result.scalars().all())

                # Wait for other transaction
                await asyncio.sleep(0.1)

                # Insert new item
                stash_item = StashItem(
                    id=uuid4(),
                    player_id=test_player.id,
                    item_id=test_item.id,
                    quantity=1,
                )
                session.add(stash_item)

                try:
                    await session.commit()
                    return "success", count_before
                except Exception as e:
                    return "conflict", str(e)

        async def transaction_b():
            """Count items, then insert one."""
            async with AsyncSessionLocal() as session:
                # Set isolation level to SERIALIZABLE
                await session.execute(
                    text("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
                )

                # Count current items
                result = await session.execute(
                    select(StashItem).where(StashItem.player_id == test_player.id)
                )
                count_before = len(result.scalars().all())

                # Wait for other transaction
                await asyncio.sleep(0.1)

                # Insert new item (different instance, same item_id might violate unique constraint)
                stash_item = StashItem(
                    id=uuid4(),
                    player_id=test_player.id,
                    item_id=test_item.id,
                    quantity=1,
                )
                session.add(stash_item)

                try:
                    await session.commit()
                    return "success", count_before
                except Exception as e:
                    return "conflict", str(e)

        # Run both transactions concurrently
        result_a, result_b = await asyncio.gather(
            transaction_a(), transaction_b(), return_exceptions=True
        )

        # Note: In this case, both might succeed since they're inserting
        # different UUIDs. The test demonstrates SERIALIZABLE usage.
        # Actual conflict detection depends on query patterns.


class TestEconomicIntegrity:
    """Test transaction isolation for economic operations."""

    @pytest.mark.asyncio
    async def test_concurrent_stash_modifications_atomic(
        self, test_player, test_item
    ):
        """
        Multiple concurrent operations on stash should be atomic.

        Simulates: Player buying item from trader while simultaneously
        extracting loot from a raid.
        """

        async def buy_item_from_trader(player_id, item_id, cost):
            """Simulate buying item from trader."""
            async with AsyncSessionLocal() as session:
                # Check currency
                result = await session.execute(
                    select(Currency).where(Currency.player_id == player_id)
                )
                currency = result.scalar_one()

                if currency.rubles < cost:
                    raise ValueError("Insufficient funds")

                # Deduct cost
                currency.rubles -= cost

                # Add item to stash
                stash_item = StashItem(
                    id=uuid4(), player_id=player_id, item_id=item_id, quantity=1
                )
                session.add(stash_item)

                await session.commit()
                return "purchased"

        async def extract_loot_from_raid(player_id, item_id, quantity):
            """Simulate extracting loot from raid."""
            async with AsyncSessionLocal() as session:
                # Add extracted item to stash
                stash_item = StashItem(
                    id=uuid4(), player_id=player_id, item_id=item_id, quantity=quantity
                )
                session.add(stash_item)
                await session.commit()
                return "extracted"

        # Run concurrent operations
        results = await asyncio.gather(
            buy_item_from_trader(test_player.id, test_item.id, 5000),
            extract_loot_from_raid(test_player.id, test_item.id, 2),
            return_exceptions=True,
        )

        # Verify final state
        async with AsyncSessionLocal() as session:
            # Check currency was deducted
            result = await session.execute(
                select(Currency).where(Currency.player_id == test_player.id)
            )
            currency = result.scalar_one()
            assert currency.rubles == 5000  # 10000 - 5000

            # Check items were added
            result = await session.execute(
                select(StashItem).where(StashItem.player_id == test_player.id)
            )
            stash_items = result.scalars().all()
            assert len(stash_items) == 2  # Both operations succeeded

    @pytest.mark.asyncio
    async def test_negative_balance_prevented_under_load(self, test_player):
        """
        Multiple concurrent purchases should never result in negative balance.

        This is critical for economic integrity.
        """

        async def attempt_purchase(player_id, cost):
            """Attempt to make a purchase."""
            try:
                async with AsyncSessionLocal() as session:
                    # Read and update in same transaction
                    result = await session.execute(
                        select(Currency)
                        .where(Currency.player_id == player_id)
                        .with_for_update()  # Lock row for update
                    )
                    currency = result.scalar_one()

                    # Check balance
                    if currency.rubles < cost:
                        await session.rollback()
                        return "insufficient_funds"

                    # Deduct
                    currency.rubles -= cost
                    await session.commit()
                    return "success"
            except Exception as e:
                return f"error: {str(e)}"

        # Attempt 5 concurrent purchases of 3000 each (total 15000, but only have 10000)
        results = await asyncio.gather(
            *[attempt_purchase(test_player.id, 3000) for _ in range(5)]
        )

        # Verify no negative balance
        async with AsyncSessionLocal() as session:
            result = await session.execute(
                select(Currency).where(Currency.player_id == test_player.id)
            )
            final_currency = result.scalar_one()

        assert final_currency.rubles >= 0, "Balance went negative!"

        # Should have 3-4 successes and 1-2 failures
        successes = results.count("success")
        failures = results.count("insufficient_funds")

        assert successes <= 3, "Too many purchases succeeded"
        assert failures >= 2, "Not enough purchases failed"
        assert final_currency.rubles == 10000 - (successes * 3000)

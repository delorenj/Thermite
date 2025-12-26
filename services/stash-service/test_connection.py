"""Test database connection for Thermite Stash Service."""

import asyncio

from thermite_stash_service.database import close_db, init_db


async def test_connection():
    """Test database connection and query execution."""
    print("Testing Thermite database connection...")

    try:
        await init_db()
        print("✓ Connection successful!")
        print("✓ Database initialized")

        # List tables
        from sqlalchemy import text

        from thermite_stash_service.database import engine

        async with engine.begin() as conn:
            result = await conn.execute(
                text(
                    """
                SELECT table_name
                FROM information_schema.tables
                WHERE table_schema = 'public'
                ORDER BY table_name
            """
                )
            )
            tables = [row[0] for row in result]
            print(f"\n✓ Found {len(tables)} tables:")
            for table in tables:
                print(f"  - {table}")

            # Count item definitions
            result = await conn.execute(text("SELECT COUNT(*) FROM item_definitions"))
            count = result.scalar()
            print(f"\n✓ Item definitions: {count} items")

    except Exception as e:
        print(f"✗ Connection failed: {e}")
        raise
    finally:
        await close_db()
        print("\n✓ Database connection closed")


if __name__ == "__main__":
    asyncio.run(test_connection())

"""
Database connection and session management for Thermite Stash Service.

Implements connection pooling with SQLAlchemy 2.0 async engine.
"""

from typing import AsyncGenerator

from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.orm import DeclarativeBase


class Base(DeclarativeBase):
    """SQLAlchemy declarative base for all models."""

    pass


# Database configuration
# In production, use pydantic-settings to load from environment
DATABASE_URL = "postgresql+psycopg://delorenj@localhost:5432/thermite"

# Create async engine with connection pooling
# Pool size tuned for FastAPI service (25 concurrent connections)
engine = create_async_engine(
    DATABASE_URL,
    echo=False,  # Set to True for SQL query logging during development
    pool_size=25,
    max_overflow=10,
    pool_pre_ping=True,  # Verify connections before use
    pool_recycle=3600,  # Recycle connections after 1 hour
)

# Session factory for dependency injection
AsyncSessionLocal = async_sessionmaker(
    engine,
    class_=AsyncSession,
    expire_on_commit=False,
    autocommit=False,
    autoflush=False,
)


async def get_db() -> AsyncGenerator[AsyncSession, None]:
    """
    Dependency for FastAPI routes to get database session.

    Usage in FastAPI:
    ```python
    @app.get("/stash")
    async def get_stash(db: AsyncSession = Depends(get_db)):
        result = await db.execute(select(StashItem))
        return result.scalars().all()
    ```

    Yields:
        AsyncSession: Database session with automatic commit/rollback
    """
    async with AsyncSessionLocal() as session:
        try:
            yield session
            await session.commit()
        except Exception:
            await session.rollback()
            raise
        finally:
            await session.close()


async def init_db() -> None:
    """
    Initialize database connection.

    Call this on application startup to verify database connectivity.
    """
    async with engine.begin() as conn:
        # Verify connection by running a simple query
        from sqlalchemy import text

        result = await conn.execute(text("SELECT 1"))
        assert result.scalar() == 1


async def close_db() -> None:
    """
    Close database connections.

    Call this on application shutdown.
    """
    await engine.dispose()

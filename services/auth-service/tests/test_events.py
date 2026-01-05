"""RabbitMQ event communication tests."""

import asyncio
import pytest
import pytest_asyncio
from unittest.mock import AsyncMock, MagicMock

from app.events import EventBus


@pytest_asyncio.fixture
async def event_bus():
    """Event bus fixture for testing."""
    bus = EventBus()
    # Mock the connection for unit tests
    bus.connection = AsyncMock()
    bus.channel = AsyncMock()
    return bus


@pytest.mark.asyncio
async def test_event_bus_publish():
    """Test publishing an event to RabbitMQ."""
    bus = EventBus()
    bus.connection = AsyncMock()
    bus.channel = AsyncMock()

    mock_exchange = AsyncMock()
    bus.exchanges["test_exchange"] = mock_exchange

    await bus.publish(
        "test_exchange",
        "user.created",
        {"user_id": "123", "email": "test@example.com"}
    )

    # Verify publish was called
    mock_exchange.publish.assert_called_once()


def test_event_bus_structure():
    """Test event bus has required methods."""
    bus = EventBus()
    assert hasattr(bus, "connect")
    assert hasattr(bus, "disconnect")
    assert hasattr(bus, "publish")
    assert hasattr(bus, "subscribe")
    assert hasattr(bus, "get_or_create_exchange")

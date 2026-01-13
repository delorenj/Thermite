"""Integration tests for queue API endpoints."""

from unittest.mock import AsyncMock, patch

import pytest
from fastapi.testclient import TestClient

from app.main import app
from app.queue_manager import get_queue_manager


@pytest.fixture
def client():
    """Create test client."""
    return TestClient(app)


@pytest.fixture
def mock_queue_manager():
    """Mock queue manager for testing."""
    with patch("app.main.get_queue_manager") as mock:
        manager = AsyncMock()
        mock.return_value = manager
        yield manager


def test_join_queue_success(client, mock_queue_manager):
    """Test successfully joining the queue."""
    mock_queue_manager.add_player = AsyncMock(return_value=(1, 60))

    response = client.post(
        "/api/v1/queue",
        json={
            "player_id": "test-player-123",
            "loadout": {"weapon": "rifle", "armor": "vest"},
        },
    )

    assert response.status_code == 200
    data = response.json()
    assert data["queue_position"] == 1
    assert data["estimated_wait_seconds"] == 60
    assert data["queue_id"] == "test-player-123"


def test_join_queue_already_in_queue(client, mock_queue_manager):
    """Test joining queue when already in queue."""
    mock_queue_manager.add_player = AsyncMock(
        side_effect=ValueError("Player test-player-123 already in queue")
    )

    response = client.post(
        "/api/v1/queue",
        json={
            "player_id": "test-player-123",
            "loadout": {"weapon": "rifle"},
        },
    )

    assert response.status_code == 409
    assert "already in queue" in response.json()["detail"]


def test_join_queue_invalid_request(client):
    """Test joining queue with invalid request data."""
    response = client.post(
        "/api/v1/queue",
        json={
            "player_id": "test-player-123",
            # Missing loadout field
        },
    )

    assert response.status_code == 422  # Validation error


def test_leave_queue_success(client, mock_queue_manager):
    """Test successfully leaving the queue."""
    mock_queue_manager.remove_player = AsyncMock(return_value=True)

    response = client.delete("/api/v1/queue/test-player-123")

    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "success"
    assert "removed from queue" in data["message"]


def test_leave_queue_not_in_queue(client, mock_queue_manager):
    """Test leaving queue when not in queue."""
    mock_queue_manager.remove_player = AsyncMock(return_value=False)

    response = client.delete("/api/v1/queue/test-player-123")

    assert response.status_code == 404
    assert "not in queue" in response.json()["detail"]


def test_health_endpoint(client):
    """Test health check endpoint."""
    response = client.get("/health")

    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "healthy"
    assert data["service"] == "matchmaking-service"


def test_root_endpoint(client):
    """Test root endpoint with service information."""
    response = client.get("/")

    assert response.status_code == 200
    data = response.json()
    assert data["service"] == "Thermite Matchmaking Service"
    assert "join_queue" in data["endpoints"]
    assert "leave_queue" in data["endpoints"]


@pytest.mark.asyncio
async def test_queue_flow_multiple_players(mock_queue_manager):
    """Test complete queue flow with multiple players."""
    from app.queue_manager import QueueManager

    manager = QueueManager()

    # Mock Redis
    mock_redis = AsyncMock()
    mock_redis.zscore = AsyncMock(return_value=None)
    mock_redis.zadd = AsyncMock(return_value=1)
    mock_redis.hset = AsyncMock(return_value=1)
    mock_redis.expire = AsyncMock(return_value=1)
    mock_redis.zrank = AsyncMock(side_effect=[0, 1, 2, 3])
    mock_redis.zcard = AsyncMock(side_effect=[1, 2, 3, 4])
    mock_redis.close = AsyncMock()

    manager._redis = mock_redis
    manager._running = True

    # Add 4 players
    positions = []
    for i in range(4):
        position, wait_time = await manager.add_player(
            f"player-{i}",
            {"weapon": "rifle"},
        )
        positions.append(position)

    # Verify positions are sequential
    assert positions == [1, 2, 3, 4]

    await manager.stop()

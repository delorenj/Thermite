"""Unit tests for QueueManager."""

import asyncio
from unittest.mock import AsyncMock, MagicMock, patch

import pytest
import pytest_asyncio

from app.queue_manager import QueueManager


@pytest_asyncio.fixture
async def queue_manager():
    """Create a QueueManager instance with mocked Redis."""
    manager = QueueManager()

    # Mock Redis client
    mock_redis = AsyncMock()
    mock_redis.zscore = AsyncMock(return_value=None)
    mock_redis.zadd = AsyncMock(return_value=1)
    mock_redis.hset = AsyncMock(return_value=1)
    mock_redis.expire = AsyncMock(return_value=1)
    mock_redis.zrank = AsyncMock(return_value=0)
    mock_redis.zcard = AsyncMock(return_value=1)
    mock_redis.zrem = AsyncMock(return_value=1)
    mock_redis.delete = AsyncMock(return_value=1)
    mock_redis.zrange = AsyncMock(return_value=[])
    mock_redis.hget = AsyncMock(return_value=None)
    mock_redis.close = AsyncMock()

    manager._redis = mock_redis
    manager._running = True

    yield manager

    manager._running = False
    if manager._match_check_task:
        manager._match_check_task.cancel()
        try:
            await manager._match_check_task
        except asyncio.CancelledError:
            pass


@pytest.mark.asyncio
async def test_add_player_success(queue_manager):
    """Test successfully adding a player to queue."""
    player_id = "test-player-123"
    loadout = {"weapon": "rifle", "armor": "vest"}

    # Mock Redis responses
    queue_manager._redis.zscore = AsyncMock(return_value=None)  # Not in queue
    queue_manager._redis.zrank = AsyncMock(return_value=0)  # First in queue

    position, wait_time = await queue_manager.add_player(player_id, loadout)

    assert position == 1
    assert wait_time >= 10
    queue_manager._redis.zadd.assert_called_once()
    queue_manager._redis.hset.assert_called_once()


@pytest.mark.asyncio
async def test_add_player_already_in_queue(queue_manager):
    """Test adding a player who is already in queue."""
    player_id = "test-player-123"
    loadout = {"weapon": "rifle"}

    # Mock player already in queue
    queue_manager._redis.zscore = AsyncMock(return_value=1234567890.0)

    with pytest.raises(ValueError, match="already in queue"):
        await queue_manager.add_player(player_id, loadout)


@pytest.mark.asyncio
async def test_remove_player_success(queue_manager):
    """Test successfully removing a player from queue."""
    player_id = "test-player-123"

    queue_manager._redis.zrem = AsyncMock(return_value=1)  # Player removed

    result = await queue_manager.remove_player(player_id)

    assert result is True
    queue_manager._redis.zrem.assert_called_once_with(
        queue_manager._queue_key, player_id
    )
    queue_manager._redis.delete.assert_called_once()


@pytest.mark.asyncio
async def test_remove_player_not_in_queue(queue_manager):
    """Test removing a player who is not in queue."""
    player_id = "test-player-123"

    queue_manager._redis.zrem = AsyncMock(return_value=0)  # Player not found

    result = await queue_manager.remove_player(player_id)

    assert result is False


@pytest.mark.asyncio
async def test_get_queue_size(queue_manager):
    """Test getting queue size."""
    queue_manager._redis.zcard = AsyncMock(return_value=5)

    size = await queue_manager.get_queue_size()

    assert size == 5
    queue_manager._redis.zcard.assert_called_once_with(queue_manager._queue_key)


def test_estimate_wait_time():
    """Test wait time estimation logic."""
    manager = QueueManager()

    # Position 1: minimum wait time (0 matches ahead: 1 // 2 = 0)
    wait_time = manager._estimate_wait_time(1)
    assert wait_time == 10

    # Position 2: 1 match ahead (2 // 2 = 1)
    wait_time = manager._estimate_wait_time(2)
    assert wait_time == 60

    # Position 3: 1 match ahead (3 // 2 = 1)
    wait_time = manager._estimate_wait_time(3)
    assert wait_time == 60

    # Position 10: 5 matches ahead (10 // 2 = 5)
    wait_time = manager._estimate_wait_time(10)
    assert wait_time == 300


@pytest.mark.asyncio
async def test_match_creation_with_8_players(queue_manager):
    """Test match creation when 8 players are in queue."""
    # Mock 8 players in queue
    player_ids = [f"player-{i}" for i in range(8)]
    queue_manager._redis.zcard = AsyncMock(return_value=8)
    queue_manager._redis.zrange = AsyncMock(return_value=player_ids)
    queue_manager._redis.hget = AsyncMock(return_value='{"weapon": "rifle"}')

    # Mock httpx client
    with patch("app.queue_manager.httpx.AsyncClient") as mock_client:
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"match_id": "test-match", "port": 9001}

        mock_client.return_value.__aenter__.return_value.post = AsyncMock(
            return_value=mock_response
        )

        await queue_manager._try_create_match()

        # Verify players were removed from queue
        queue_manager._redis.zrem.assert_called_once()
        assert len(queue_manager._redis.zrem.call_args[0][1:]) == 8


@pytest.mark.asyncio
async def test_match_creation_with_4_players(queue_manager):
    """Test match creation with minimum 4 players."""
    # Mock 4 players in queue
    player_ids = [f"player-{i}" for i in range(4)]
    queue_manager._redis.zcard = AsyncMock(return_value=4)
    queue_manager._redis.zrange = AsyncMock(return_value=player_ids)
    queue_manager._redis.hget = AsyncMock(return_value='{"weapon": "rifle"}')

    # Mock httpx client
    with patch("app.queue_manager.httpx.AsyncClient") as mock_client:
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {"match_id": "test-match", "port": 9001}

        mock_client.return_value.__aenter__.return_value.post = AsyncMock(
            return_value=mock_response
        )

        await queue_manager._try_create_match()

        # Verify match was created with 4 players
        queue_manager._redis.zrem.assert_called_once()
        assert len(queue_manager._redis.zrem.call_args[0][1:]) == 4


@pytest.mark.asyncio
async def test_no_match_creation_with_3_players(queue_manager):
    """Test that no match is created with only 3 players."""
    # Mock 3 players in queue (less than minimum)
    queue_manager._redis.zcard = AsyncMock(return_value=3)

    await queue_manager._try_create_match()

    # Verify no match was created
    queue_manager._redis.zrem.assert_not_called()


@pytest.mark.asyncio
async def test_manager_not_started():
    """Test that operations fail when manager not started."""
    manager = QueueManager()

    with pytest.raises(RuntimeError, match="not started"):
        await manager.add_player("player-1", {})

    with pytest.raises(RuntimeError, match="not started"):
        await manager.remove_player("player-1")

    with pytest.raises(RuntimeError, match="not started"):
        await manager.get_queue_size()

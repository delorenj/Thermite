"""Queue management for matchmaking using Redis sorted sets."""

import asyncio
import time
import uuid
from typing import Dict, List, Optional

import httpx
from redis import asyncio as aioredis

from app.config import settings
from app.logging_config import get_logger

logger = get_logger(__name__)


class QueueManager:
    """Manages matchmaking queue using Redis sorted sets."""

    def __init__(self):
        self._redis: Optional[aioredis.Redis] = None
        self._queue_key = "matchmaking:queue"
        self._player_data_prefix = "matchmaking:player:"
        self._match_check_task: Optional[asyncio.Task] = None
        self._running = False

    async def start(self) -> None:
        """Initialize Redis connection and start match checking loop."""
        logger.info("starting_queue_manager")
        self._redis = await aioredis.from_url(
            settings.redis_url,
            encoding="utf-8",
            decode_responses=True,
        )
        self._running = True
        self._match_check_task = asyncio.create_task(self._match_check_loop())

    async def stop(self) -> None:
        """Stop queue manager and cleanup."""
        logger.info("stopping_queue_manager")
        self._running = False

        if self._match_check_task:
            self._match_check_task.cancel()
            try:
                await self._match_check_task
            except asyncio.CancelledError:
                pass

        if self._redis:
            await self._redis.close()

    async def add_player(self, player_id: str, loadout: dict) -> tuple[int, int]:
        """
        Add a player to the matchmaking queue.

        Args:
            player_id: Player UUID
            loadout: Player loadout configuration

        Returns:
            Tuple of (queue_position, estimated_wait_seconds)

        Raises:
            ValueError: If player is already in queue
        """
        if not self._redis:
            raise RuntimeError("Queue manager not started")

        # Check if player already in queue
        score = await self._redis.zscore(self._queue_key, player_id)
        if score is not None:
            raise ValueError(f"Player {player_id} already in queue")

        # Add player to sorted set with current timestamp as score (FIFO)
        timestamp = time.time()
        await self._redis.zadd(self._queue_key, {player_id: timestamp})

        # Store player loadout data
        player_key = f"{self._player_data_prefix}{player_id}"
        await self._redis.hset(
            player_key,
            mapping={
                "loadout": str(loadout),
                "joined_at": str(timestamp),
            },
        )
        await self._redis.expire(player_key, settings.queue_timeout_seconds)

        # Calculate queue position and wait time
        position = await self._get_queue_position(player_id)
        wait_time = self._estimate_wait_time(position)

        logger.info(
            "player_joined_queue",
            player_id=player_id,
            position=position,
            queue_size=await self.get_queue_size(),
        )

        return position, wait_time

    async def remove_player(self, player_id: str) -> bool:
        """
        Remove a player from the queue.

        Args:
            player_id: Player UUID

        Returns:
            True if player was removed, False if not in queue
        """
        if not self._redis:
            raise RuntimeError("Queue manager not started")

        # Remove from sorted set
        removed = await self._redis.zrem(self._queue_key, player_id)

        if removed:
            # Cleanup player data
            player_key = f"{self._player_data_prefix}{player_id}"
            await self._redis.delete(player_key)

            logger.info("player_left_queue", player_id=player_id)
            return True

        return False

    async def get_queue_size(self) -> int:
        """Get current number of players in queue."""
        if not self._redis:
            raise RuntimeError("Queue manager not started")

        return await self._redis.zcard(self._queue_key)

    async def _get_queue_position(self, player_id: str) -> int:
        """Get player's position in queue (0-indexed)."""
        if not self._redis:
            raise RuntimeError("Queue manager not started")

        rank = await self._redis.zrank(self._queue_key, player_id)
        return rank + 1 if rank is not None else 0

    def _estimate_wait_time(self, position: int) -> int:
        """
        Estimate wait time based on queue position.

        Args:
            position: Player's position in queue (1-indexed)

        Returns:
            Estimated wait time in seconds
        """
        # Simple estimation: 60 seconds per match ahead
        # Assumes matches form every 60 seconds on average
        matches_ahead = position // settings.min_players_per_match
        return max(matches_ahead * 60, 10)  # Minimum 10 seconds

    async def _match_check_loop(self) -> None:
        """Periodic check for creating matches from queue."""
        while self._running:
            try:
                await asyncio.sleep(5)  # Check every 5 seconds

                queue_size = await self.get_queue_size()

                # Need at least min_players to form a match
                if queue_size >= settings.min_players_per_match:
                    await self._try_create_match()

            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error("match_check_error", error=str(e))

    async def _try_create_match(self) -> None:
        """Attempt to create a match from queued players."""
        if not self._redis:
            return

        queue_size = await self.get_queue_size()

        # Determine match size (prefer 8, minimum 4 per story requirements)
        if queue_size >= 8:
            match_size = 8
        elif queue_size >= 4:
            match_size = min(queue_size, 8)
        else:
            # Less than 4 players, don't create match yet
            return

        # Get oldest players from queue (FIFO)
        player_ids = await self._redis.zrange(self._queue_key, 0, match_size - 1)

        if len(player_ids) < settings.min_players_per_match:
            return

        # Fetch player loadouts
        player_loadouts = {}
        for player_id in player_ids:
            player_key = f"{self._player_data_prefix}{player_id}"
            loadout_str = await self._redis.hget(player_key, "loadout")
            if loadout_str:
                # Simple string storage for now
                player_loadouts[player_id] = {"data": loadout_str}

        # Remove players from queue
        await self._redis.zrem(self._queue_key, *player_ids)

        # Cleanup player data
        for player_id in player_ids:
            player_key = f"{self._player_data_prefix}{player_id}"
            await self._redis.delete(player_key)

        # Create match via Match Orchestrator
        match_id = uuid.uuid4()

        logger.info(
            "creating_match_from_queue",
            match_id=str(match_id),
            player_count=len(player_ids),
            player_ids=[str(p) for p in player_ids],
        )

        try:
            match_info = await self._create_match_request(
                match_id, list(player_ids), player_loadouts
            )

            # Store match info in Redis for players to retrieve
            match_key = f"match:info:{match_id}"
            await self._redis.hset(
                match_key,
                mapping={
                    "websocket_url": match_info.get("websocket_url", ""),
                    "port": str(match_info.get("port", 0)),
                    "status": match_info.get("status", ""),
                },
            )
            await self._redis.expire(match_key, 300)  # 5 minute expiry

        except Exception as e:
            logger.error(
                "failed_to_create_match",
                match_id=str(match_id),
                error=str(e),
            )
            # TODO: Re-queue players on failure

    async def _create_match_request(
        self,
        match_id: uuid.UUID,
        player_ids: List[str],
        player_loadouts: Dict[str, dict],
    ) -> dict:
        """
        Send match creation request to Match Orchestrator.

        Args:
            match_id: UUID for the match
            player_ids: List of player UUIDs
            player_loadouts: Player loadout configurations

        Returns:
            Match information dictionary with websocket_url, port, spawn_assignments
        """
        # Match Orchestrator endpoint (from docker-compose)
        orchestrator_url = "http://match-orchestrator:8005"

        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{orchestrator_url}/matches",
                json={
                    "match_id": str(match_id),
                    "map_name": "factory_01",  # Default map for MVP
                    "player_ids": player_ids,
                },
                timeout=10.0,
            )

            if response.status_code == 200:
                match_data = response.json()
                logger.info(
                    "match_created_successfully",
                    match_id=str(match_id),
                    port=match_data.get("port"),
                    player_count=len(player_ids),
                    websocket_url=match_data.get("websocket_url"),
                )
                return match_data
            else:
                logger.error(
                    "match_creation_failed",
                    match_id=str(match_id),
                    status_code=response.status_code,
                    error=response.text,
                )
                raise RuntimeError(f"Match creation failed: {response.text}")


# Global singleton instance
_queue_manager: Optional[QueueManager] = None


def get_queue_manager() -> QueueManager:
    """Get the global QueueManager instance."""
    global _queue_manager
    if _queue_manager is None:
        _queue_manager = QueueManager()
    return _queue_manager

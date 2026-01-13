"""Match lifecycle manager for spawning and monitoring Game Server processes."""

import asyncio
import subprocess
import uuid
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from pathlib import Path
from typing import Dict, Optional

from app.config import settings
from app.logging_config import get_logger

logger = get_logger(__name__)


class MatchStatus(str, Enum):
    """Match status enum."""

    STARTING = "starting"
    RUNNING = "running"
    ENDED = "ended"
    CRASHED = "crashed"


@dataclass
class MatchProcess:
    """Represents a running Game Server process."""

    match_id: uuid.UUID
    port: int
    process: subprocess.Popen
    status: MatchStatus = MatchStatus.STARTING
    started_at: datetime = field(default_factory=datetime.utcnow)
    player_count: int = 0
    map_name: str = ""
    player_ids: List[str] = field(default_factory=list)
    spawn_assignments: Dict[str, Tuple[int, int]] = field(default_factory=dict)

    @property
    def is_alive(self) -> bool:
        """Check if process is still running."""
        return self.process.poll() is None

    def check_health(self) -> MatchStatus:
        """Check process health and update status."""
        if not self.is_alive:
            if self.status == MatchStatus.RUNNING:
                self.status = MatchStatus.CRASHED
                logger.warning(
                    "match_process_crashed",
                    match_id=str(self.match_id),
                    exit_code=self.process.returncode,
                )
            return self.status

        # If process is alive and was starting, mark as running
        if self.status == MatchStatus.STARTING:
            self.status = MatchStatus.RUNNING
            logger.info("match_running", match_id=str(self.match_id), port=self.port)

        return self.status

    async def terminate(self) -> None:
        """Gracefully terminate the process."""
        if self.is_alive:
            logger.info("terminating_match", match_id=str(self.match_id))
            self.process.terminate()
            try:
                # Wait up to 5 seconds for graceful shutdown
                await asyncio.wait_for(
                    asyncio.create_task(asyncio.to_thread(self.process.wait)),
                    timeout=5.0,
                )
            except asyncio.TimeoutError:
                logger.warning("force_killing_match", match_id=str(self.match_id))
                self.process.kill()
                await asyncio.to_thread(self.process.wait)

        self.status = MatchStatus.ENDED


class MatchManager:
    """Manages Game Server process lifecycle."""

    def __init__(self):
        self._matches: Dict[uuid.UUID, MatchProcess] = {}
        self._next_port = 9001
        self._health_check_task: Optional[asyncio.Task] = None

    async def start(self) -> None:
        """Start the match manager."""
        logger.info("starting_match_manager")
        self._health_check_task = asyncio.create_task(self._health_check_loop())

    async def stop(self) -> None:
        """Stop the match manager and cleanup all matches."""
        logger.info("stopping_match_manager", active_matches=len(self._matches))

        if self._health_check_task:
            self._health_check_task.cancel()
            try:
                await self._health_check_task
            except asyncio.CancelledError:
                pass

        # Terminate all running matches
        tasks = [match.terminate() for match in self._matches.values()]
        if tasks:
            await asyncio.gather(*tasks, return_exceptions=True)

        self._matches.clear()

    async def create_match(
        self,
        match_id: Optional[uuid.UUID] = None,
        map_name: str = "factory_01",
        player_ids: Optional[List[str]] = None,
    ) -> MatchProcess:
        """
        Spawn a new Game Server process.

        Args:
            match_id: Optional match ID (generates new if not provided)
            map_name: Map template to use
            player_ids: Optional list of player UUIDs for spawn distribution

        Returns:
            MatchProcess instance

        Raises:
            RuntimeError: If process spawn fails
            ValueError: If spawn distribution fails
        """
        if match_id is None:
            match_id = uuid.uuid4()

        if match_id in self._matches:
            raise ValueError(f"Match {match_id} already exists")

        if player_ids is None:
            player_ids = []

        port = self._allocate_port()
        map_path = Path(settings.maps_directory) / f"{map_name}.json"

        if not map_path.exists():
            raise FileNotFoundError(f"Map template not found: {map_path}")

        # Distribute spawn points if players provided
        spawn_assignments = {}
        if player_ids:
            from app.spawn_manager import SpawnManager

            # Map player indices to player_ids
            spawn_by_index = SpawnManager.distribute_spawn_points(
                map_name, len(player_ids)
            )
            spawn_assignments = {
                player_ids[int(idx)]: pos for idx, pos in spawn_by_index.items()
            }

        # Build command to spawn Game Server
        rabbitmq_url = f"amqp://{settings.rabbitmq_user}:{settings.rabbitmq_password}@{settings.rabbitmq_host}:{settings.rabbitmq_port}"

        cmd = [
            settings.game_server_binary,
            str(match_id),
            str(port),
            str(map_path),
            rabbitmq_url,
        ]

        logger.info(
            "spawning_game_server",
            match_id=str(match_id),
            port=port,
            map=map_name,
            player_count=len(player_ids),
            spawn_assignments=spawn_assignments,
        )

        try:
            process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )

            match_process = MatchProcess(
                match_id=match_id,
                port=port,
                process=process,
                map_name=map_name,
                player_ids=player_ids,
                player_count=len(player_ids),
                spawn_assignments=spawn_assignments,
            )

            self._matches[match_id] = match_process

            # Wait a moment to ensure process didn't immediately crash
            await asyncio.sleep(0.5)
            if not match_process.is_alive:
                stderr = process.stderr.read() if process.stderr else "No error output"
                raise RuntimeError(f"Game Server process crashed immediately: {stderr}")

            logger.info(
                "match_created",
                match_id=str(match_id),
                port=port,
                pid=process.pid,
            )

            return match_process

        except Exception as e:
            logger.error(
                "failed_to_spawn_match",
                match_id=str(match_id),
                error=str(e),
            )
            # Cleanup if process was created
            if match_id in self._matches:
                await self._matches[match_id].terminate()
                del self._matches[match_id]
            raise

    async def get_match(self, match_id: uuid.UUID) -> Optional[MatchProcess]:
        """Get match by ID."""
        return self._matches.get(match_id)

    async def list_matches(self) -> Dict[uuid.UUID, MatchProcess]:
        """List all active matches."""
        return self._matches.copy()

    async def terminate_match(self, match_id: uuid.UUID) -> bool:
        """Terminate a specific match."""
        match = self._matches.get(match_id)
        if not match:
            return False

        await match.terminate()
        del self._matches[match_id]
        logger.info("match_terminated", match_id=str(match_id))
        return True

    def _allocate_port(self) -> int:
        """Allocate next available port for Game Server."""
        port = self._next_port
        self._next_port += 1
        return port

    async def _health_check_loop(self) -> None:
        """Periodic health check for all running matches."""
        while True:
            try:
                await asyncio.sleep(10)  # Check every 10 seconds

                crashed_matches = []
                for match_id, match in self._matches.items():
                    status = match.check_health()
                    if status == MatchStatus.CRASHED:
                        crashed_matches.append(match_id)

                # Cleanup crashed matches
                for match_id in crashed_matches:
                    logger.warning("cleaning_up_crashed_match", match_id=str(match_id))
                    del self._matches[match_id]

            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error("health_check_error", error=str(e))


# Global singleton instance
_match_manager: Optional[MatchManager] = None


def get_match_manager() -> MatchManager:
    """Get the global MatchManager instance."""
    global _match_manager
    if _match_manager is None:
        _match_manager = MatchManager()
    return _match_manager

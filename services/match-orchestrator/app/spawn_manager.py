"""Spawn point management for match lobbies."""

import json
import random
from pathlib import Path
from typing import Dict, List, Tuple

from app.config import settings
from app.logging_config import get_logger

logger = get_logger(__name__)


class SpawnManager:
    """Manages spawn point distribution for match lobbies."""

    @staticmethod
    def load_map_template(map_name: str) -> dict:
        """
        Load map template from filesystem.

        Args:
            map_name: Map template name (without .json extension)

        Returns:
            Map template dictionary

        Raises:
            FileNotFoundError: If map template doesn't exist
            ValueError: If map template is invalid
        """
        map_path = Path(settings.maps_directory) / f"{map_name}.json"

        if not map_path.exists():
            raise FileNotFoundError(f"Map template not found: {map_path}")

        try:
            with open(map_path, "r") as f:
                map_data = json.load(f)
        except json.JSONDecodeError as e:
            raise ValueError(f"Invalid map template JSON: {e}")

        # Validate required fields
        if "spawn_points" not in map_data:
            raise ValueError(f"Map template missing spawn_points: {map_name}")

        if not isinstance(map_data["spawn_points"], list):
            raise ValueError("spawn_points must be a list")

        if len(map_data["spawn_points"]) < 1:
            raise ValueError("Map must have at least 1 spawn point")

        return map_data

    @staticmethod
    def distribute_spawn_points(
        map_name: str, player_count: int
    ) -> Dict[str, Tuple[int, int]]:
        """
        Distribute spawn points to players.

        Args:
            map_name: Map template name
            player_count: Number of players (4-8)

        Returns:
            Dictionary mapping player indices to (x, y) spawn positions
            Example: {"0": (2, 2), "1": (17, 2), ...}

        Raises:
            ValueError: If not enough spawn points or invalid player count
        """
        if player_count < 4 or player_count > 8:
            raise ValueError(f"Invalid player count: {player_count} (must be 4-8)")

        # Load map template
        map_data = SpawnManager.load_map_template(map_name)
        spawn_points = map_data["spawn_points"]

        if len(spawn_points) < player_count:
            raise ValueError(
                f"Not enough spawn points: map has {len(spawn_points)}, "
                f"need {player_count}"
            )

        # Randomly select spawn points without replacement
        selected_spawns = random.sample(spawn_points, player_count)

        # Create spawn assignments
        spawn_assignments = {}
        for i, spawn_point in enumerate(selected_spawns):
            spawn_assignments[str(i)] = (spawn_point["x"], spawn_point["y"])

        logger.info(
            "spawn_points_distributed",
            map_name=map_name,
            player_count=player_count,
            assignments=spawn_assignments,
        )

        return spawn_assignments

    @staticmethod
    def validate_spawn_positions(
        spawn_assignments: Dict[str, Tuple[int, int]],
        map_width: int,
        map_height: int,
    ) -> bool:
        """
        Validate spawn positions are within map bounds and unique.

        Args:
            spawn_assignments: Dictionary of player index to (x, y) positions
            map_width: Map width
            map_height: Map height

        Returns:
            True if all spawn positions are valid

        Raises:
            ValueError: If any spawn position is invalid
        """
        seen_positions = set()

        for player_idx, (x, y) in spawn_assignments.items():
            # Check bounds
            if x < 0 or x >= map_width:
                raise ValueError(
                    f"Spawn position out of bounds for player {player_idx}: "
                    f"x={x} (width={map_width})"
                )

            if y < 0 or y >= map_height:
                raise ValueError(
                    f"Spawn position out of bounds for player {player_idx}: "
                    f"y={y} (height={map_height})"
                )

            # Check uniqueness
            position = (x, y)
            if position in seen_positions:
                raise ValueError(
                    f"Duplicate spawn position for player {player_idx}: {position}"
                )

            seen_positions.add(position)

        return True

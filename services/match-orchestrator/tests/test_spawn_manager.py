"""Unit tests for SpawnManager."""

import json
from pathlib import Path
from unittest.mock import mock_open, patch

import pytest

from app.spawn_manager import SpawnManager


@pytest.fixture
def mock_map_data():
    """Mock map template data."""
    return {
        "name": "Test Map",
        "width": 20,
        "height": 20,
        "spawn_points": [
            {"x": 2, "y": 2},
            {"x": 17, "y": 2},
            {"x": 2, "y": 17},
            {"x": 17, "y": 17},
            {"x": 6, "y": 3},
            {"x": 13, "y": 3},
            {"x": 6, "y": 16},
            {"x": 13, "y": 16},
        ],
        "walls": [],
        "extraction_points": [],
    }


def test_load_map_template_success(mock_map_data):
    """Test successfully loading a map template."""
    map_json = json.dumps(mock_map_data)

    with patch("builtins.open", mock_open(read_data=map_json)):
        with patch("pathlib.Path.exists", return_value=True):
            map_data = SpawnManager.load_map_template("factory_01")

            assert map_data["name"] == "Test Map"
            assert len(map_data["spawn_points"]) == 8
            assert map_data["width"] == 20


def test_load_map_template_file_not_found():
    """Test loading a non-existent map template."""
    with patch("pathlib.Path.exists", return_value=False):
        with pytest.raises(FileNotFoundError, match="Map template not found"):
            SpawnManager.load_map_template("nonexistent")


def test_load_map_template_invalid_json():
    """Test loading a map template with invalid JSON."""
    invalid_json = "{invalid json"

    with patch("builtins.open", mock_open(read_data=invalid_json)):
        with patch("pathlib.Path.exists", return_value=True):
            with pytest.raises(ValueError, match="Invalid map template JSON"):
                SpawnManager.load_map_template("factory_01")


def test_load_map_template_missing_spawn_points():
    """Test loading a map template without spawn_points field."""
    map_data = {"name": "Test Map", "width": 20, "height": 20}
    map_json = json.dumps(map_data)

    with patch("builtins.open", mock_open(read_data=map_json)):
        with patch("pathlib.Path.exists", return_value=True):
            with pytest.raises(ValueError, match="missing spawn_points"):
                SpawnManager.load_map_template("factory_01")


def test_distribute_spawn_points_4_players(mock_map_data):
    """Test distributing spawn points for 4 players."""
    map_json = json.dumps(mock_map_data)

    with patch("builtins.open", mock_open(read_data=map_json)):
        with patch("pathlib.Path.exists", return_value=True):
            spawn_assignments = SpawnManager.distribute_spawn_points("factory_01", 4)

            # Should have 4 spawn assignments
            assert len(spawn_assignments) == 4

            # Keys should be string indices
            assert all(key in ["0", "1", "2", "3"] for key in spawn_assignments.keys())

            # Values should be tuples of (x, y)
            for pos in spawn_assignments.values():
                assert isinstance(pos, tuple)
                assert len(pos) == 2
                assert isinstance(pos[0], int)
                assert isinstance(pos[1], int)

            # All positions should be unique
            positions = list(spawn_assignments.values())
            assert len(positions) == len(set(positions))


def test_distribute_spawn_points_8_players(mock_map_data):
    """Test distributing spawn points for 8 players (max)."""
    map_json = json.dumps(mock_map_data)

    with patch("builtins.open", mock_open(read_data=map_json)):
        with patch("pathlib.Path.exists", return_value=True):
            spawn_assignments = SpawnManager.distribute_spawn_points("factory_01", 8)

            # Should have 8 spawn assignments
            assert len(spawn_assignments) == 8

            # All positions should be unique
            positions = list(spawn_assignments.values())
            assert len(positions) == len(set(positions))


def test_distribute_spawn_points_invalid_player_count(mock_map_data):
    """Test distributing spawn points with invalid player count."""
    map_json = json.dumps(mock_map_data)

    with patch("builtins.open", mock_open(read_data=map_json)):
        with patch("pathlib.Path.exists", return_value=True):
            # Too few players
            with pytest.raises(ValueError, match="must be 4-8"):
                SpawnManager.distribute_spawn_points("factory_01", 3)

            # Too many players
            with pytest.raises(ValueError, match="must be 4-8"):
                SpawnManager.distribute_spawn_points("factory_01", 9)


def test_distribute_spawn_points_not_enough_spawns():
    """Test distributing spawn points when map has too few spawn points."""
    map_data = {
        "name": "Small Map",
        "width": 10,
        "height": 10,
        "spawn_points": [
            {"x": 1, "y": 1},
            {"x": 8, "y": 1},
        ],  # Only 2 spawn points
    }
    map_json = json.dumps(map_data)

    with patch("builtins.open", mock_open(read_data=map_json)):
        with patch("pathlib.Path.exists", return_value=True):
            with pytest.raises(ValueError, match="Not enough spawn points"):
                SpawnManager.distribute_spawn_points("small_map", 4)


def test_validate_spawn_positions_success():
    """Test validating correct spawn positions."""
    spawn_assignments = {
        "0": (2, 2),
        "1": (17, 2),
        "2": (2, 17),
        "3": (17, 17),
    }

    result = SpawnManager.validate_spawn_positions(spawn_assignments, 20, 20)
    assert result is True


def test_validate_spawn_positions_out_of_bounds():
    """Test validating spawn positions that are out of map bounds."""
    spawn_assignments = {
        "0": (2, 2),
        "1": (25, 2),  # x out of bounds
    }

    with pytest.raises(ValueError, match="out of bounds"):
        SpawnManager.validate_spawn_positions(spawn_assignments, 20, 20)


def test_validate_spawn_positions_duplicate():
    """Test validating duplicate spawn positions."""
    spawn_assignments = {
        "0": (2, 2),
        "1": (2, 2),  # Duplicate position
    }

    with pytest.raises(ValueError, match="Duplicate spawn position"):
        SpawnManager.validate_spawn_positions(spawn_assignments, 20, 20)


def test_spawn_distribution_randomness(mock_map_data):
    """Test that spawn distribution is random."""
    map_json = json.dumps(mock_map_data)

    # Run distribution multiple times and collect results
    results = []
    with patch("builtins.open", mock_open(read_data=map_json)):
        with patch("pathlib.Path.exists", return_value=True):
            for _ in range(10):
                spawn_assignments = SpawnManager.distribute_spawn_points("factory_01", 4)
                # Convert to tuple of positions for comparison
                positions = tuple(sorted(spawn_assignments.values()))
                results.append(positions)

    # Should have some variation (not all identical)
    unique_results = set(results)
    assert len(unique_results) > 1, "Spawn distribution should be random"

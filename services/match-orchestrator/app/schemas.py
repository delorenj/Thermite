"""Pydantic models for request/response validation."""

import uuid
from datetime import datetime
from typing import Optional

from pydantic import BaseModel, Field


class HealthResponse(BaseModel):
    """Health check response model."""

    status: str = Field(..., description="Service health status")
    service: str = Field(..., description="Service name")
    version: str = Field(..., description="Service version")


class EventPayload(BaseModel):
    """Base event payload model."""

    event_type: str = Field(..., description="Type of event")
    service: str = Field(..., description="Source service name")
    payload: dict = Field(..., description="Event data")
    timestamp: datetime | None = Field(default=None, description="Event timestamp")


class MatchCreateRequest(BaseModel):
    """Match creation request."""

    map_id: str = Field(..., description="Map identifier")
    player_ids: list[str] = Field(..., description="List of player UUIDs")
    player_loadouts: dict[str, dict] = Field(..., description="Player loadout configurations")


class CreateMatchRequest(BaseModel):
    """Request to create a new match (simplified)."""

    match_id: Optional[uuid.UUID] = Field(default=None, description="Optional match UUID")
    map_name: str = Field(default="factory_01", description="Map template name")
    player_ids: list[str] = Field(default_factory=list, description="List of player UUIDs")


class MatchResponse(BaseModel):
    """Match status response."""

    match_id: uuid.UUID = Field(..., description="Match UUID")
    status: str = Field(..., description="Match status")
    port: int = Field(..., description="WebSocket port")
    started_at: datetime = Field(..., description="Match start timestamp")
    player_count: int = Field(default=0, description="Current player count")
    map_name: str = Field(..., description="Map name")
    pid: Optional[int] = Field(default=None, description="Process ID")
    websocket_url: Optional[str] = Field(default=None, description="WebSocket connection URL")
    spawn_assignments: Optional[dict[str, tuple[int, int]]] = Field(
        default=None, description="Player spawn positions (player_id -> (x, y))"
    )

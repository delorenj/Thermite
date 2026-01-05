"""Pydantic models for request/response validation."""

from datetime import datetime

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

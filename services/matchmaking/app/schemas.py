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


class QueueRequest(BaseModel):
    """Matchmaking queue join request."""

    player_id: str = Field(..., description="Player UUID")
    loadout: dict = Field(..., description="Player loadout configuration")


class QueueResponse(BaseModel):
    """Matchmaking queue join response."""

    queue_position: int = Field(..., description="Position in queue")
    estimated_wait_seconds: int = Field(..., description="Estimated wait time")
    queue_id: str = Field(..., description="Queue identifier")

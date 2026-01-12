"""FastAPI application for Match Orchestrator Service."""

import uuid
from contextlib import asynccontextmanager
from typing import AsyncIterator, Optional

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse

from app.config import settings
from app.logging_config import configure_logging, get_logger
from app.match_manager import MatchStatus, get_match_manager
from app.schemas import CreateMatchRequest, MatchResponse


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncIterator[None]:
    """Application lifespan manager."""
    logger = get_logger(__name__)
    logger.info(
        "starting_service",
        service=settings.service_name,
        port=settings.service_port,
    )

    # Start match manager
    manager = get_match_manager()
    await manager.start()

    yield

    # Stop match manager
    await manager.stop()
    logger.info("shutting_down_service", service=settings.service_name)


configure_logging(settings.log_level)
logger = get_logger(__name__)

app = FastAPI(
    title="Thermite Match Orchestrator",
    description="Match lifecycle coordination",
    version="0.1.0",
    lifespan=lifespan,
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_origins,
    allow_credentials=settings.cors_allow_credentials,
    allow_methods=settings.cors_allow_methods,
    allow_headers=settings.cors_allow_headers,
)


@app.middleware("http")
async def add_security_headers(request, call_next):
    """Add security headers to all responses."""
    response = await call_next(request)
    response.headers["X-Content-Type-Options"] = "nosniff"
    response.headers["X-Frame-Options"] = "DENY"
    response.headers["X-XSS-Protection"] = "1; mode=block"
    response.headers["Strict-Transport-Security"] = "max-age=31536000; includeSubDomains"
    return response


@app.get("/health")
async def health_check() -> JSONResponse:
    """Health check endpoint.

    Returns:
        JSON response with service status
    """
    return JSONResponse(
        status_code=200,
        content={
            "status": "healthy",
            "service": settings.service_name,
            "version": "0.1.0",
        },
    )


@app.get("/")
async def root() -> JSONResponse:
    """Root endpoint with service information.

    Returns:
        JSON response with service details
    """
    return JSONResponse(
        content={
            "service": "Thermite Match Orchestrator",
            "version": "0.1.0",
            "endpoints": {
                "health": "/health",
                "docs": "/docs",
                "redoc": "/redoc",
                "create_match": "POST /matches",
                "get_match": "GET /matches/{match_id}",
                "list_matches": "GET /matches",
                "delete_match": "DELETE /matches/{match_id}",
            },
        }
    )


@app.post("/matches", response_model=MatchResponse)
async def create_match(request: CreateMatchRequest) -> MatchResponse:
    """
    Create a new match and spawn Game Server process.

    Args:
        request: Match creation parameters

    Returns:
        Match status response

    Raises:
        HTTPException: If match creation fails
    """
    manager = get_match_manager()

    try:
        match_process = await manager.create_match(
            match_id=request.match_id,
            map_name=request.map_name,
        )

        return MatchResponse(
            match_id=match_process.match_id,
            status=match_process.status.value,
            port=match_process.port,
            started_at=match_process.started_at,
            player_count=match_process.player_count,
            map_name=match_process.map_name,
            pid=match_process.process.pid if match_process.is_alive else None,
        )

    except Exception as e:
        logger.error("match_creation_failed", error=str(e))
        raise HTTPException(status_code=500, detail=f"Failed to create match: {str(e)}")


@app.get("/matches/{match_id}", response_model=MatchResponse)
async def get_match(match_id: uuid.UUID) -> MatchResponse:
    """
    Get match status by ID.

    Args:
        match_id: Match UUID

    Returns:
        Match status response

    Raises:
        HTTPException: If match not found
    """
    manager = get_match_manager()
    match_process = await manager.get_match(match_id)

    if not match_process:
        raise HTTPException(status_code=404, detail=f"Match {match_id} not found")

    return MatchResponse(
        match_id=match_process.match_id,
        status=match_process.status.value,
        port=match_process.port,
        started_at=match_process.started_at,
        player_count=match_process.player_count,
        map_name=match_process.map_name,
        pid=match_process.process.pid if match_process.is_alive else None,
    )


@app.get("/matches", response_model=list[MatchResponse])
async def list_matches() -> list[MatchResponse]:
    """
    List all active matches.

    Returns:
        List of match status responses
    """
    manager = get_match_manager()
    matches = await manager.list_matches()

    return [
        MatchResponse(
            match_id=match.match_id,
            status=match.status.value,
            port=match.port,
            started_at=match.started_at,
            player_count=match.player_count,
            map_name=match.map_name,
            pid=match.process.pid if match.is_alive else None,
        )
        for match in matches.values()
    ]


@app.delete("/matches/{match_id}")
async def delete_match(match_id: uuid.UUID) -> JSONResponse:
    """
    Terminate a match.

    Args:
        match_id: Match UUID

    Returns:
        Success confirmation

    Raises:
        HTTPException: If match not found
    """
    manager = get_match_manager()
    success = await manager.terminate_match(match_id)

    if not success:
        raise HTTPException(status_code=404, detail=f"Match {match_id} not found")

    return JSONResponse(
        content={
            "status": "success",
            "message": f"Match {match_id} terminated",
        }
    )

"""FastAPI application for Matchmaking Service."""

from contextlib import asynccontextmanager
from typing import AsyncIterator

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse

from app.config import settings
from app.logging_config import configure_logging, get_logger
from app.queue_manager import get_queue_manager
from app.schemas import QueueRequest, QueueResponse


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncIterator[None]:
    """Application lifespan manager."""
    logger = get_logger(__name__)
    logger.info(
        "starting_service",
        service=settings.service_name,
        port=settings.service_port,
    )

    # Start queue manager
    manager = get_queue_manager()
    await manager.start()

    yield

    # Stop queue manager
    await manager.stop()
    logger.info("shutting_down_service", service=settings.service_name)


configure_logging(settings.log_level)
logger = get_logger(__name__)

app = FastAPI(
    title="Thermite Matchmaking Service",
    description="Queue management and player matching",
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
            "service": "Thermite Matchmaking Service",
            "version": "0.1.0",
            "endpoints": {
                "health": "/health",
                "docs": "/docs",
                "redoc": "/redoc",
                "join_queue": "POST /api/v1/queue",
                "leave_queue": "DELETE /api/v1/queue/{player_id}",
            },
        }
    )


@app.post("/api/v1/queue", response_model=QueueResponse)
async def join_queue(request: QueueRequest) -> QueueResponse:
    """
    Add a player to the matchmaking queue.

    Args:
        request: Queue join request with player_id and loadout

    Returns:
        Queue position and estimated wait time

    Raises:
        HTTPException: If player already in queue or queue error
    """
    manager = get_queue_manager()

    try:
        position, wait_time = await manager.add_player(
            player_id=request.player_id,
            loadout=request.loadout,
        )

        logger.info(
            "player_joined_queue_api",
            player_id=request.player_id,
            position=position,
        )

        return QueueResponse(
            queue_position=position,
            estimated_wait_seconds=wait_time,
            queue_id=request.player_id,
        )

    except ValueError as e:
        logger.warning("queue_join_failed", player_id=request.player_id, error=str(e))
        raise HTTPException(status_code=409, detail=str(e))
    except Exception as e:
        logger.error("queue_join_error", player_id=request.player_id, error=str(e))
        raise HTTPException(status_code=500, detail=f"Failed to join queue: {str(e)}")


@app.delete("/api/v1/queue/{player_id}")
async def leave_queue(player_id: str) -> JSONResponse:
    """
    Remove a player from the matchmaking queue.

    Args:
        player_id: Player UUID

    Returns:
        Success confirmation

    Raises:
        HTTPException: If player not in queue
    """
    manager = get_queue_manager()

    try:
        removed = await manager.remove_player(player_id)

        if not removed:
            raise HTTPException(
                status_code=404,
                detail=f"Player {player_id} not in queue",
            )

        logger.info("player_left_queue_api", player_id=player_id)

        return JSONResponse(
            content={
                "status": "success",
                "message": f"Player {player_id} removed from queue",
            }
        )

    except HTTPException:
        raise
    except Exception as e:
        logger.error("queue_leave_error", player_id=player_id, error=str(e))
        raise HTTPException(status_code=500, detail=f"Failed to leave queue: {str(e)}")

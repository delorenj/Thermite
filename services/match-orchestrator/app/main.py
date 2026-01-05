"""FastAPI application for Match Orchestrator Service."""

from contextlib import asynccontextmanager
from typing import AsyncIterator

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse

from app.config import settings
from app.logging_config import configure_logging, get_logger


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncIterator[None]:
    """Application lifespan manager."""
    logger = get_logger(__name__)
    logger.info(
        "starting_service",
        service=settings.service_name,
        port=settings.service_port,
    )
    yield
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
            },
        }
    )

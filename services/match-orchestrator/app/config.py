"""Configuration management for Match Orchestrator Service."""

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Service configuration from environment variables."""

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
        extra="ignore",
    )

    # Service Configuration
    service_name: str = "match-orchestrator"
    service_port: int = 8001
    log_level: str = "INFO"

    # Database Configuration
    database_url: str = "postgresql://thermite:thermite_dev_password@postgres:5432/thermite"

    # Redis Configuration
    redis_url: str = "redis://redis:6379"

    # RabbitMQ Configuration
    rabbitmq_url: str = "amqp://thermite:thermite_dev_password@rabbitmq:5672"

    # Game Server Configuration
    game_server_image: str = "thermite/game-server:latest"
    max_concurrent_matches: int = 12

    # CORS Configuration
    cors_origins: list[str] = ["*"]
    cors_allow_credentials: bool = True
    cors_allow_methods: list[str] = ["*"]
    cors_allow_headers: list[str] = ["*"]


settings = Settings()

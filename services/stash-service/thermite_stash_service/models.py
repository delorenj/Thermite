"""
SQLAlchemy ORM models for Thermite database.

Maps to schema defined in database/schema/001_initial_schema.sql
Supports all 7 tables: players, currencies, item_definitions, stash_items,
matches, match_participants, audit_logs
"""

from datetime import datetime
from typing import Optional
from uuid import UUID, uuid4

from sqlalchemy import (
    BigInteger,
    Boolean,
    CheckConstraint,
    DateTime,
    ForeignKey,
    Index,
    Integer,
    String,
    UniqueConstraint,
)
from sqlalchemy.dialects.postgresql import JSONB, UUID as PGUUID
from sqlalchemy.orm import Mapped, mapped_column, relationship
from sqlalchemy.sql import func

from .database import Base


class Player(Base):
    """Player authentication and profile data."""

    __tablename__ = "players"

    id: Mapped[UUID] = mapped_column(
        PGUUID(as_uuid=True), primary_key=True, default=uuid4
    )
    email: Mapped[str] = mapped_column(String(255), unique=True, nullable=False)
    username: Mapped[str] = mapped_column(String(50), unique=True, nullable=False)
    password_hash: Mapped[str] = mapped_column(String(255), nullable=False)
    created_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, server_default=func.now()
    )
    last_login: Mapped[Optional[datetime]] = mapped_column(DateTime, nullable=True)
    is_active: Mapped[bool] = mapped_column(Boolean, default=True)

    # Relationships
    currency: Mapped[Optional["Currency"]] = relationship(
        "Currency", back_populates="player", uselist=False, cascade="all, delete-orphan"
    )
    stash_items: Mapped[list["StashItem"]] = relationship(
        "StashItem", back_populates="player", cascade="all, delete-orphan"
    )
    match_participations: Mapped[list["MatchParticipant"]] = relationship(
        "MatchParticipant", back_populates="player", cascade="all, delete-orphan"
    )

    __table_args__ = (Index("idx_players_email", "email"),)

    def __repr__(self) -> str:
        return f"<Player(id={self.id}, username='{self.username}', email='{self.email}')>"


class Currency(Base):
    """Player currency balances with economic floor enforcement."""

    __tablename__ = "currencies"

    player_id: Mapped[UUID] = mapped_column(
        PGUUID(as_uuid=True),
        ForeignKey("players.id", ondelete="CASCADE"),
        primary_key=True,
    )
    rubles: Mapped[int] = mapped_column(Integer, nullable=False, default=0)
    updated_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, server_default=func.now()
    )

    # Relationships
    player: Mapped["Player"] = relationship("Player", back_populates="currency")

    __table_args__ = (
        CheckConstraint("rubles >= 0", name="positive_balance"),
        Index("idx_currency_player", "player_id"),
    )

    def __repr__(self) -> str:
        return f"<Currency(player_id={self.player_id}, rubles={self.rubles})>"


class ItemDefinition(Base):
    """Static item definitions with stats and metadata."""

    __tablename__ = "item_definitions"

    id: Mapped[str] = mapped_column(String(50), primary_key=True)
    name: Mapped[str] = mapped_column(String(100), nullable=False)
    category: Mapped[str] = mapped_column(String(20), nullable=False)
    tier: Mapped[int] = mapped_column(Integer, nullable=False)
    value: Mapped[int] = mapped_column(Integer, nullable=False)
    max_stack: Mapped[int] = mapped_column(Integer, default=1)
    properties: Mapped[Optional[dict]] = mapped_column(JSONB, nullable=True)

    # Relationships
    stash_items: Mapped[list["StashItem"]] = relationship(
        "StashItem", back_populates="item_definition"
    )

    __table_args__ = (CheckConstraint("tier BETWEEN 1 AND 3", name="tier_range"),)

    def __repr__(self) -> str:
        return f"<ItemDefinition(id='{self.id}', name='{self.name}', category='{self.category}')>"


class StashItem(Base):
    """Player persistent inventory (stash)."""

    __tablename__ = "stash_items"

    id: Mapped[UUID] = mapped_column(
        PGUUID(as_uuid=True), primary_key=True, default=uuid4
    )
    player_id: Mapped[UUID] = mapped_column(
        PGUUID(as_uuid=True), ForeignKey("players.id", ondelete="CASCADE"), nullable=False
    )
    item_id: Mapped[str] = mapped_column(
        String(50), ForeignKey("item_definitions.id"), nullable=False
    )
    quantity: Mapped[int] = mapped_column(Integer, nullable=False, default=1)
    is_equipped: Mapped[bool] = mapped_column(Boolean, default=False)
    acquired_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, server_default=func.now()
    )

    # Relationships
    player: Mapped["Player"] = relationship("Player", back_populates="stash_items")
    item_definition: Mapped["ItemDefinition"] = relationship(
        "ItemDefinition", back_populates="stash_items"
    )

    __table_args__ = (
        CheckConstraint("quantity > 0", name="positive_quantity"),
        Index("idx_stash_player", "player_id"),
        Index(
            "idx_stash_equipped",
            "player_id",
            "is_equipped",
            postgresql_where="is_equipped = TRUE",
        ),
        # Prevent duplicate equipped items (one helmet, one vest, etc.)
        Index(
            "idx_stash_unique_equipped",
            "player_id",
            "item_id",
            unique=True,
            postgresql_where="is_equipped = TRUE",
        ),
    )

    def __repr__(self) -> str:
        return f"<StashItem(id={self.id}, player_id={self.player_id}, item_id='{self.item_id}', quantity={self.quantity})>"


class Match(Base):
    """Match metadata and lifecycle tracking."""

    __tablename__ = "matches"

    id: Mapped[UUID] = mapped_column(
        PGUUID(as_uuid=True), primary_key=True, default=uuid4
    )
    map_id: Mapped[str] = mapped_column(String(50), nullable=False)
    status: Mapped[str] = mapped_column(String(20), nullable=False)
    started_at: Mapped[Optional[datetime]] = mapped_column(DateTime, nullable=True)
    ended_at: Mapped[Optional[datetime]] = mapped_column(DateTime, nullable=True)
    duration_seconds: Mapped[Optional[int]] = mapped_column(Integer, nullable=True)
    server_address: Mapped[Optional[str]] = mapped_column(String(100), nullable=True)
    max_players: Mapped[int] = mapped_column(Integer, default=8)
    created_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, server_default=func.now()
    )

    # Relationships
    participants: Mapped[list["MatchParticipant"]] = relationship(
        "MatchParticipant", back_populates="match", cascade="all, delete-orphan"
    )

    __table_args__ = (
        Index("idx_match_status", "status"),
        Index("idx_match_started", "started_at", postgresql_ops={"started_at": "DESC"}),
    )

    def __repr__(self) -> str:
        return f"<Match(id={self.id}, map_id='{self.map_id}', status='{self.status}')>"


class MatchParticipant(Base):
    """Player participation in matches with outcome tracking."""

    __tablename__ = "match_participants"

    id: Mapped[UUID] = mapped_column(
        PGUUID(as_uuid=True), primary_key=True, default=uuid4
    )
    match_id: Mapped[UUID] = mapped_column(
        PGUUID(as_uuid=True), ForeignKey("matches.id", ondelete="CASCADE"), nullable=False
    )
    player_id: Mapped[UUID] = mapped_column(
        PGUUID(as_uuid=True), ForeignKey("players.id", ondelete="CASCADE"), nullable=False
    )
    outcome: Mapped[Optional[str]] = mapped_column(String(20), nullable=True)
    spawn_position: Mapped[Optional[dict]] = mapped_column(JSONB, nullable=True)
    death_position: Mapped[Optional[dict]] = mapped_column(JSONB, nullable=True)
    loadout: Mapped[dict] = mapped_column(JSONB, nullable=False)
    loot_extracted: Mapped[Optional[dict]] = mapped_column(JSONB, nullable=True)
    kill_count: Mapped[int] = mapped_column(Integer, default=0)
    damage_dealt: Mapped[int] = mapped_column(Integer, default=0)
    survival_time_seconds: Mapped[Optional[int]] = mapped_column(Integer, nullable=True)
    joined_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, server_default=func.now()
    )
    left_at: Mapped[Optional[datetime]] = mapped_column(DateTime, nullable=True)

    # Relationships
    match: Mapped["Match"] = relationship("Match", back_populates="participants")
    player: Mapped["Player"] = relationship(
        "Player", back_populates="match_participations"
    )

    __table_args__ = (
        UniqueConstraint("match_id", "player_id", name="unique_match_player"),
        Index("idx_participant_match", "match_id"),
        Index("idx_participant_player", "player_id"),
        Index("idx_participant_outcome", "outcome"),
    )

    def __repr__(self) -> str:
        return f"<MatchParticipant(id={self.id}, match_id={self.match_id}, player_id={self.player_id}, outcome='{self.outcome}')>"


class AuditLog(Base):
    """Structured event log for debugging and analytics."""

    __tablename__ = "audit_logs"

    id: Mapped[int] = mapped_column(BigInteger, primary_key=True, autoincrement=True)
    event_type: Mapped[str] = mapped_column(String(100), nullable=False)
    player_id: Mapped[Optional[UUID]] = mapped_column(
        PGUUID(as_uuid=True), ForeignKey("players.id"), nullable=True
    )
    match_id: Mapped[Optional[UUID]] = mapped_column(
        PGUUID(as_uuid=True), ForeignKey("matches.id"), nullable=True
    )
    details: Mapped[Optional[dict]] = mapped_column(JSONB, nullable=True)
    timestamp: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, server_default=func.now()
    )
    severity: Mapped[Optional[str]] = mapped_column(String(20), nullable=True)

    __table_args__ = (
        Index("idx_audit_timestamp", "timestamp", postgresql_ops={"timestamp": "DESC"}),
        Index("idx_audit_event_type", "event_type"),
        Index("idx_audit_player", "player_id", postgresql_where="player_id IS NOT NULL"),
    )

    def __repr__(self) -> str:
        return f"<AuditLog(id={self.id}, event_type='{self.event_type}', timestamp={self.timestamp})>"

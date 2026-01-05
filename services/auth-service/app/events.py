"""RabbitMQ event publishing and subscribing infrastructure."""

import asyncio
import json
from typing import Any, Callable

import aio_pika
from aio_pika import DeliveryMode, ExchangeType, Message
from aio_pika.abc import AbstractChannel, AbstractConnection, AbstractQueue

from app.config import settings
from app.logging_config import get_logger

logger = get_logger(__name__)


class EventBus:
    """RabbitMQ event bus for inter-service communication."""

    def __init__(self) -> None:
        """Initialize the event bus."""
        self.connection: AbstractConnection | None = None
        self.channel: AbstractChannel | None = None
        self.exchanges: dict[str, aio_pika.abc.AbstractExchange] = {}

    async def connect(self) -> None:
        """Establish connection to RabbitMQ."""
        try:
            logger.info("connecting_to_rabbitmq", url=settings.rabbitmq_url)
            self.connection = await aio_pika.connect_robust(settings.rabbitmq_url)
            self.channel = await self.connection.channel()
            await self.channel.set_qos(prefetch_count=10)
            logger.info("rabbitmq_connected")
        except Exception as e:
            logger.error("rabbitmq_connection_failed", error=str(e))
            raise

    async def disconnect(self) -> None:
        """Close RabbitMQ connection."""
        if self.channel:
            await self.channel.close()
        if self.connection:
            await self.connection.close()
        logger.info("rabbitmq_disconnected")

    async def get_or_create_exchange(
        self, name: str, exchange_type: ExchangeType = ExchangeType.FANOUT
    ) -> aio_pika.abc.AbstractExchange:
        """Get or create an exchange.

        Args:
            name: Exchange name
            exchange_type: Type of exchange (fanout, direct, topic)

        Returns:
            Exchange instance
        """
        if name not in self.exchanges:
            if not self.channel:
                await self.connect()
            assert self.channel is not None
            self.exchanges[name] = await self.channel.declare_exchange(
                name, exchange_type, durable=True
            )
        return self.exchanges[name]

    async def publish(
        self, exchange_name: str, event_type: str, payload: dict[str, Any]
    ) -> None:
        """Publish an event to an exchange.

        Args:
            exchange_name: Name of the exchange
            event_type: Type/name of the event
            payload: Event data
        """
        exchange = await self.get_or_create_exchange(exchange_name)

        message_body = json.dumps(
            {
                "event_type": event_type,
                "service": settings.service_name,
                "payload": payload,
            }
        )

        message = Message(
            message_body.encode(),
            delivery_mode=DeliveryMode.PERSISTENT,
            content_type="application/json",
        )

        await exchange.publish(message, routing_key="")
        logger.info(
            "event_published",
            exchange=exchange_name,
            event_type=event_type,
        )

    async def subscribe(
        self,
        exchange_name: str,
        queue_name: str,
        callback: Callable[[dict[str, Any]], None],
    ) -> AbstractQueue:
        """Subscribe to events from an exchange.

        Args:
            exchange_name: Name of the exchange
            queue_name: Name of the queue to create
            callback: Async function to handle incoming messages

        Returns:
            Queue instance
        """
        if not self.channel:
            await self.connect()
        assert self.channel is not None

        exchange = await self.get_or_create_exchange(exchange_name)

        queue = await self.channel.declare_queue(queue_name, durable=True)
        await queue.bind(exchange)

        async def message_handler(message: aio_pika.abc.AbstractIncomingMessage) -> None:
            async with message.process():
                try:
                    data = json.loads(message.body.decode())
                    logger.info(
                        "event_received",
                        exchange=exchange_name,
                        event_type=data.get("event_type"),
                    )
                    await callback(data)
                except Exception as e:
                    logger.error(
                        "event_processing_failed",
                        error=str(e),
                        exchange=exchange_name,
                    )

        await queue.consume(message_handler)
        logger.info("subscribed_to_exchange", exchange=exchange_name, queue=queue_name)
        return queue


# Global event bus instance
event_bus = EventBus()

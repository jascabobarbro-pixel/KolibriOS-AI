"""
Message Bus for Unified Mind.

Provides a message-based communication system for broadcasting commands
and receiving events between system components.
"""

import asyncio
import logging
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from typing import Any, Optional, Callable, Awaitable

logger = logging.getLogger(__name__)


class MessageType(str, Enum):
    """Types of messages in the system."""
    COMMAND = "command"
    EVENT = "event"
    QUERY = "query"
    RESPONSE = "response"
    HEARTBEAT = "heartbeat"
    METRIC = "metric"
    ALERT = "alert"
    LEARNING = "learning"


@dataclass
class Message:
    """A message in the bus."""
    id: str
    type: MessageType
    source: str
    target: Optional[str] = None  # None for broadcast
    payload: dict[str, Any] = field(default_factory=dict)
    timestamp: datetime = field(default_factory=datetime.now)
    priority: int = 0
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass
class Subscription:
    """A subscription to a message type."""
    id: str
    message_type: MessageType
    callback: Callable[[Message], Awaitable[None]]
    active: bool = True
    created_at: datetime = field(default_factory=datetime.now)
    filter: Optional[Callable[[Message], bool]] = None


    async_callback: bool = False


class MessageBus:
    """
    Message Bus for inter-component communication.
    
    This provides a pub/sub style messaging system for:
    - Broadcasting commands to cells
    - Receiving events from cells
    - Querying system state
    - Handling alerts and metrics
    """

    def __init__(self, max_queue_size: int = 1000) -> None:
        """Initialize the message bus."""
        self._queue: asyncio.Queue = asyncio.Queue(max_queue_size)
        self._subscriptions: dict[str, list[Subscription]] = {}
        self._handlers: dict[MessageType, Callable] = {}
        self._running = False
        self._tasks: list[asyncio.Task] = []
        
        # Register default handlers
        self._register_default_handlers()

    async def start(self) -> None:
        """Start the message bus."""
        if self._running:
            return
        
        self._running = True
        self._tasks = [
            asyncio.create_task(self._process_loop()),
        ]
        logger.info("Message Bus started")

    async def stop(self) -> None:
        """Stop the message bus."""
        self._running = False
        
        for task in self._tasks:
            task.cancel()
            try:
                await task
            except asyncio.CancelledError:
                pass
        
        logger.info("Message Bus stopped")

    async def publish(self, message: Message) -> bool:
        """
        Publish a message to the bus.
        
        Args:
            message: The message to publish
        
        Returns:
            True if published successfully
        """
        if not self._running:
            logger.warning("Message Bus not running, cannot publish")
            return False
        
        # Add timestamp
        message.timestamp = datetime.now()
        
        # Queue the message
        try:
            await asyncio.wait_for(
                self._queue.put(message),
                timeout=1.0,
            )
        except asyncio.TimeoutError:
            logger.warning("Message queue full, dropping message")
            return False
        
        logger.debug(f"Published {message.type.value} message from {message.source}")
        return True

    async def subscribe(
        self,
        message_type: MessageType,
        callback: Callable[[Message], Awaitable[None]],
        filter_func: Optional[Callable[[Message], bool]] = None,
    ) -> str:
        """
        Subscribe to a message type.
        
        Args:
            message_type: Type of messages to subscribe to
            callback: Function to call when message is received
            filter_func: Optional filter function
        
        Returns:
            Subscription ID
        """
        sub_id = f"sub-{message_type.value}-{datetime.now().timestamp()}"
        
        subscription = Subscription(
            id=sub_id,
            message_type=message_type,
            callback=callback,
            filter=filter_func,
        )
        
        if message_type not in self._subscriptions:
            self._subscriptions[message_type] = []
        
        self._subscriptions[message_type].append(subscription)
        
        logger.info(f"Subscribed to {message_type.value} messages")
        return sub_id

    def unsubscribe(self, subscription_id: str) -> bool:
        """
        Unsubscribe from messages.
        
        Args:
            subscription_id: ID of subscription to cancel
        
        Returns:
            True if unsubscribed successfully
        """
        for msg_type, subs in self._subscriptions.items():
            for i, sub in enumerate(subs):
                if sub.id == subscription_id:
                    subs.pop(i)
                    logger.info(f"Unsubscribed from {msg_type.value} messages")
                    return True
        return False

    def register_handler(
        self,
        message_type: MessageType,
        handler: Callable[[Message], Awaitable[None]],
    ) -> None:
        """
        Register a handler for a message type.
        
        Args:
            message_type: Type of messages to handle
            handler: Function to handle messages
        """
        self._handlers[message_type] = handler
        logger.info(f"Registered handler for {message_type.value} messages")

    async def _process_loop(self) -> None:
        """Main processing loop for the message bus."""
        while self._running:
            try:
                message = await asyncio.wait_for(
                    self._queue.get(),
                    timeout=1.0,
                )
                
                # Route to subscribers
                if message.type in self._subscriptions:
                    for sub in self._subscriptions[message.type]:
                        if sub.active:
                            # Apply filter if present
                            if sub.filter:
                                should_process = await sub.filter(message)
                                if not should_process:
                                    continue
                            
                            # Call callback
                            try:
                                if asyncio.iscoroutinefunction(sub.callback):
                                    await sub.callback(message)
                                else:
                                    sub.callback(message)
                            except Exception as e:
                                logger.error(f"Error in subscription callback: {e}")
                
                # Call direct handler if registered
                if message.type in self._handlers:
                    try:
                        handler = self._handlers[message.type]
                        if asyncio.iscoroutinefunction(handler):
                            await handler(message)
                        else:
                            handler(message)
                    except Exception as e:
                        logger.error(f"Error in message handler: {e}")
                
            except asyncio.CancelledError:
                break
            except asyncio.TimeoutError:
                pass
            except Exception as e:
                logger.error(f"Message processing error: {e}")
                await asyncio.sleep(0.1)

    def _register_default_handlers(self) -> None:
        """Register default message handlers."""
        async def handle_command(message: Message) -> None:
            """Handle command messages."""
            logger.info(f"Command received: {message.payload}")
            # Command handling logic
        
        async def handle_event(message: Message) -> None:
            """Handle event messages."""
            logger.debug(f"Event received: {message.payload}")
        
        async def handle_metric(message: Message) -> None:
            """Handle metric messages."""
            # Process metrics
        
        async def handle_alert(message: Message) -> None:
            """Handle alert messages."""
            logger.warning(f"Alert: {message.payload}")
        
        self._handlers[MessageType.COMMAND] = handle_command
        self._handlers[MessageType.EVENT] = handle_event
        self._handlers[MessageType.METRIC] = handle_metric
        self._handlers[MessageType.ALERT] = handle_alert

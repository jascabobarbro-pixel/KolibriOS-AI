#!/usr/bin/env python3
"""
Comprehensive Tests for Communication Module.

Tests cover:
- gRPC client
- Message bus
- Channel management
"""

import asyncio
import pytest
from datetime import datetime
from unittest.mock import AsyncMock, MagicMock, patch
from typing import Optional

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from communication.grpc_client import (
    GrpcConfig, GrpcClient, CndClient, KernelClient,
    create_cnd_client, create_kernel_client
)
from communication.message_bus import (
    Message, MessageType, Subscription, MessageBus
)


# ============================================================================
# Fixtures
# ============================================================================

@pytest.fixture
def grpc_config():
    """Create a GrpcConfig for testing."""
    return GrpcConfig(
        endpoint="localhost:50051",
        timeout=10.0,
        max_retries=3,
        retry_delay=1.0,
        enable_tls=False,
    )


@pytest.fixture
def message_bus():
    """Create a MessageBus for testing."""
    return MessageBus(max_queue_size=100)


@pytest.fixture
def sample_message():
    """Create a sample Message for testing."""
    return Message(
        id="msg-001",
        type=MessageType.COMMAND,
        source="test-source",
        target="test-target",
        payload={"command": "test", "param": "value"},
        priority=1,
    )


# ============================================================================
# GrpcConfig Tests
# ============================================================================

class TestGrpcConfig:
    """Tests for GrpcConfig."""

    def test_config_creation(self):
        """Test GrpcConfig can be created."""
        config = GrpcConfig(
            endpoint="localhost:50051",
            timeout=5.0,
            max_retries=5,
        )
        assert config.endpoint == "localhost:50051"
        assert config.timeout == 5.0
        assert config.max_retries == 5

    def test_config_defaults(self):
        """Test GrpcConfig has correct defaults."""
        config = GrpcConfig(endpoint="localhost:50051")
        assert config.timeout == 10.0
        assert config.max_retries == 3
        assert config.retry_delay == 1.0
        assert config.enable_tls is False
        assert config.cert_path is None

    def test_config_with_tls(self, tmp_path):
        """Test GrpcConfig with TLS settings."""
        cert_file = tmp_path / "cert.pem"
        cert_file.write_text("fake certificate")
        
        config = GrpcConfig(
            endpoint="localhost:50051",
            enable_tls=True,
            cert_path=str(cert_file),
        )
        assert config.enable_tls is True
        assert config.cert_path == str(cert_file)


# ============================================================================
# GrpcClient Tests
# ============================================================================

class TestGrpcClient:
    """Tests for GrpcClient base class."""

    def test_client_creation(self, grpc_config):
        """Test GrpcClient can be created."""
        client = GrpcClient(grpc_config)
        assert client.config == grpc_config
        assert client._channel is None
        assert client._connected is False
        assert client._connection_attempts == 0

    @pytest.mark.asyncio
    async def test_connect_insecure(self, grpc_config):
        """Test insecure connection."""
        client = GrpcClient(grpc_config)
        
        mock_channel = AsyncMock()
        mock_channel.channel_ready = AsyncMock()
        
        with patch('grpc.aio.insecure_channel', return_value=mock_channel):
            result = await client.connect()
            
            assert result is True
            assert client._connected is True

    @pytest.mark.asyncio
    async def test_connect_secure(self, tmp_path):
        """Test secure connection with TLS."""
        cert_file = tmp_path / "cert.pem"
        cert_file.write_text("fake certificate")
        
        config = GrpcConfig(
            endpoint="localhost:50051",
            enable_tls=True,
            cert_path=str(cert_file),
        )
        client = GrpcClient(config)
        
        mock_channel = AsyncMock()
        mock_channel.channel_ready = AsyncMock()
        
        with patch('grpc.ssl_channel_credentials') as mock_cred:
            with patch('grpc.aio.secure_channel', return_value=mock_channel):
                result = await client.connect()
                
                assert result is True
                mock_cred.assert_called_once()

    @pytest.mark.asyncio
    async def test_connect_timeout(self, grpc_config):
        """Test connection timeout."""
        grpc_config.timeout = 0.1
        client = GrpcClient(grpc_config)
        
        mock_channel = AsyncMock()
        mock_channel.channel_ready = AsyncMock(
            side_effect=asyncio.TimeoutError()
        )
        
        with patch('grpc.aio.insecure_channel', return_value=mock_channel):
            result = await client.connect()
            
            assert result is False
            assert client._connected is False

    @pytest.mark.asyncio
    async def test_disconnect(self, grpc_config):
        """Test disconnection."""
        client = GrpcClient(grpc_config)
        client._connected = True
        client._channel = AsyncMock()
        client._channel.close = AsyncMock()
        
        await client.disconnect()
        
        assert client._connected is False
        assert client._channel is None

    @pytest.mark.asyncio
    async def test_reconnect_success(self, grpc_config):
        """Test successful reconnection."""
        client = GrpcClient(grpc_config)
        
        mock_channel = AsyncMock()
        mock_channel.channel_ready = AsyncMock()
        
        with patch('grpc.aio.insecure_channel', return_value=mock_channel):
            result = await client.reconnect()
            
            assert result is True
            assert client._connection_attempts == 1

    @pytest.mark.asyncio
    async def test_reconnect_max_retries(self, grpc_config):
        """Test reconnection respects max retries."""
        grpc_config.max_retries = 2
        client = GrpcClient(grpc_config)
        client._connection_attempts = 3  # Already exceeded
        
        result = await client.reconnect()
        
        assert result is False

    def test_is_connected_property(self, grpc_config):
        """Test is_connected property."""
        client = GrpcClient(grpc_config)
        
        assert client.is_connected is False
        
        client._connected = True
        client._channel = MagicMock()
        
        assert client.is_connected is True


# ============================================================================
# CndClient Tests
# ============================================================================

class TestCndClient:
    """Tests for CndClient."""

    def test_client_creation(self):
        """Test CndClient can be created."""
        client = CndClient(
            endpoint="localhost:50051",
            timeout=10.0,
        )
        assert client.config.endpoint == "localhost:50051"
        assert client.config.timeout == 10.0

    @pytest.mark.asyncio
    async def test_get_status_disconnected(self):
        """Test get_status when disconnected."""
        client = CndClient()
        
        status = await client.get_status()
        
        assert status["status"] == "disconnected"

    @pytest.mark.asyncio
    async def test_get_status_connected(self):
        """Test get_status when connected."""
        client = CndClient()
        client._connected = True
        
        status = await client.get_status()
        
        assert status["status"] == "running"

    @pytest.mark.asyncio
    async def test_register_cell_disconnected(self):
        """Test register_cell when disconnected."""
        client = CndClient()
        
        result = await client.register_cell(
            cell_id="test-cell",
            cell_type="memory",
            endpoint="localhost:50051",
        )
        
        # Should attempt connection and return error if fails
        assert result["success"] is False

    @pytest.mark.asyncio
    async def test_register_cell_connected(self):
        """Test register_cell when connected."""
        client = CndClient()
        client._connected = True
        
        result = await client.register_cell(
            cell_id="test-cell",
            cell_type="memory",
            endpoint="localhost:50051",
            capabilities={"pools": "kernel,user"},
        )
        
        assert result["success"] is True
        assert result["cell_id"] == "test-cell"

    @pytest.mark.asyncio
    async def test_send_command_disconnected(self):
        """Test send_command when disconnected."""
        client = CndClient()
        
        result = await client.send_command(
            cell_id="test-cell",
            command="get_stats",
        )
        
        assert result["success"] is False

    @pytest.mark.asyncio
    async def test_send_command_connected(self):
        """Test send_command when connected."""
        client = CndClient()
        client._connected = True
        
        result = await client.send_command(
            cell_id="test-cell",
            command="get_stats",
            parameters={"detail": "full"},
        )
        
        assert result["success"] is True
        assert result["cell_id"] == "test-cell"

    @pytest.mark.asyncio
    async def test_get_system_metrics_disconnected(self):
        """Test get_system_metrics when disconnected."""
        client = CndClient()
        
        result = await client.get_system_metrics()
        
        assert result["status"] == "disconnected"

    @pytest.mark.asyncio
    async def test_get_system_metrics_connected(self):
        """Test get_system_metrics when connected."""
        client = CndClient()
        client._connected = True
        
        result = await client.get_system_metrics()
        
        assert "total_memory" in result
        assert "cpu_utilization" in result
        assert "overall_health" in result

    @pytest.mark.asyncio
    async def test_broadcast_command_disconnected(self):
        """Test broadcast_command when disconnected."""
        client = CndClient()
        
        result = await client.broadcast_command("heartbeat")
        
        assert "error" in result

    @pytest.mark.asyncio
    async def test_broadcast_command_connected(self):
        """Test broadcast_command when connected."""
        client = CndClient()
        client._connected = True
        
        result = await client.broadcast_command(
            command="heartbeat",
            parameters={"interval": "5s"},
            cell_type="memory",
        )
        
        assert "broadcast_id" in result
        assert result["command"] == "heartbeat"

    @pytest.mark.asyncio
    async def test_run_diagnostics_disconnected(self):
        """Test run_diagnostics when disconnected."""
        client = CndClient()
        
        result = await client.run_diagnostics()
        
        assert result["healthy"] is False
        assert "error" in result

    @pytest.mark.asyncio
    async def test_run_diagnostics_connected(self):
        """Test run_diagnostics when connected."""
        client = CndClient()
        client._connected = True
        
        result = await client.run_diagnostics()
        
        assert "healthy" in result
        assert "cells" in result
        assert "recommendations" in result


# ============================================================================
# KernelClient Tests
# ============================================================================

class TestKernelClient:
    """Tests for KernelClient."""

    def test_client_creation(self):
        """Test KernelClient can be created."""
        client = KernelClient(
            endpoint="localhost:50052",
            timeout=10.0,
        )
        assert client.config.endpoint == "localhost:50052"
        assert client.config.timeout == 10.0

    @pytest.mark.asyncio
    async def test_get_status_disconnected(self):
        """Test get_status when disconnected."""
        client = KernelClient()
        
        status = await client.get_status()
        
        assert status["status"] == "disconnected"

    @pytest.mark.asyncio
    async def test_get_status_connected(self):
        """Test get_status when connected."""
        client = KernelClient()
        client._connected = True
        
        status = await client.get_status()
        
        assert status["status"] == "running"
        assert "neural_scheduler_active" in status

    @pytest.mark.asyncio
    async def test_get_gene_states_disconnected(self):
        """Test get_gene_states when disconnected."""
        client = KernelClient()
        
        result = await client.get_gene_states()
        
        assert "error" in result

    @pytest.mark.asyncio
    async def test_get_gene_states_connected(self):
        """Test get_gene_states when connected."""
        client = KernelClient()
        client._connected = True
        
        result = await client.get_gene_states()
        
        assert "process_gene" in result
        assert "memory_gene" in result
        assert "io_gene" in result

    @pytest.mark.asyncio
    async def test_activate_gene_disconnected(self):
        """Test activate_gene when disconnected."""
        client = KernelClient()
        
        result = await client.activate_gene("memory_gene")
        
        assert result["success"] is False

    @pytest.mark.asyncio
    async def test_activate_gene_connected(self):
        """Test activate_gene when connected."""
        client = KernelClient()
        client._connected = True
        
        result = await client.activate_gene("memory_gene")
        
        assert result["success"] is True
        assert result["gene"] == "memory_gene"

    @pytest.mark.asyncio
    async def test_get_scheduler_decision_disconnected(self):
        """Test get_scheduler_decision when disconnected."""
        client = KernelClient()
        
        result = await client.get_scheduler_decision({"cpu": 50})
        
        assert result["decision"] == "balance_load"

    @pytest.mark.asyncio
    async def test_get_scheduler_decision_connected(self):
        """Test get_scheduler_decision when connected."""
        client = KernelClient()
        client._connected = True
        
        result = await client.get_scheduler_decision({
            "cpu_utilization": 75,
            "memory_utilization": 50,
        })
        
        assert "decision" in result
        assert "confidence" in result
        assert result["confidence"] >= 0

    @pytest.mark.asyncio
    async def test_allocate_memory_disconnected(self):
        """Test allocate_memory when disconnected."""
        client = KernelClient()
        
        result = await client.allocate_memory(1024, "user")
        
        assert result["success"] is False

    @pytest.mark.asyncio
    async def test_allocate_memory_connected(self):
        """Test allocate_memory when connected."""
        client = KernelClient()
        client._connected = True
        
        result = await client.allocate_memory(1024, "user")
        
        assert result["success"] is True
        assert "pointer" in result
        assert result["zone"] == "user"


# ============================================================================
# Utility Function Tests
# ============================================================================

class TestUtilityFunctions:
    """Tests for utility functions."""

    @pytest.mark.asyncio
    async def test_create_cnd_client(self):
        """Test create_cnd_client utility function."""
        mock_channel = AsyncMock()
        mock_channel.channel_ready = AsyncMock()
        
        with patch('grpc.aio.insecure_channel', return_value=mock_channel):
            client = await create_cnd_client("localhost:50051")
            
            assert isinstance(client, CndClient)
            assert client.config.endpoint == "localhost:50051"

    @pytest.mark.asyncio
    async def test_create_kernel_client(self):
        """Test create_kernel_client utility function."""
        mock_channel = AsyncMock()
        mock_channel.channel_ready = AsyncMock()
        
        with patch('grpc.aio.insecure_channel', return_value=mock_channel):
            client = await create_kernel_client("localhost:50052")
            
            assert isinstance(client, KernelClient)
            assert client.config.endpoint == "localhost:50052"


# ============================================================================
# MessageType Tests
# ============================================================================

class TestMessageType:
    """Tests for MessageType enum."""

    def test_message_type_values(self):
        """Test MessageType enum values."""
        assert MessageType.COMMAND.value == "command"
        assert MessageType.EVENT.value == "event"
        assert MessageType.QUERY.value == "query"
        assert MessageType.RESPONSE.value == "response"
        assert MessageType.HEARTBEAT.value == "heartbeat"
        assert MessageType.METRIC.value == "metric"
        assert MessageType.ALERT.value == "alert"
        assert MessageType.LEARNING.value == "learning"


# ============================================================================
# Message Tests
# ============================================================================

class TestMessage:
    """Tests for Message dataclass."""

    def test_message_creation(self):
        """Test Message can be created."""
        msg = Message(
            id="msg-001",
            type=MessageType.COMMAND,
            source="source-1",
            target="target-1",
            payload={"key": "value"},
        )
        assert msg.id == "msg-001"
        assert msg.type == MessageType.COMMAND
        assert msg.source == "source-1"
        assert msg.target == "target-1"

    def test_message_defaults(self):
        """Test Message has correct defaults."""
        msg = Message(
            id="msg-001",
            type=MessageType.COMMAND,
            source="source-1",
        )
        assert msg.target is None
        assert msg.payload == {}
        assert msg.priority == 0
        assert msg.metadata == {}

    def test_message_broadcast(self):
        """Test broadcast message (no target)."""
        msg = Message(
            id="msg-broadcast",
            type=MessageType.EVENT,
            source="system",
        )
        assert msg.target is None


# ============================================================================
# Subscription Tests
# ============================================================================

class TestSubscription:
    """Tests for Subscription dataclass."""

    def test_subscription_creation(self):
        """Test Subscription can be created."""
        async def callback(msg):
            pass
        
        sub = Subscription(
            id="sub-001",
            message_type=MessageType.COMMAND,
            callback=callback,
        )
        assert sub.id == "sub-001"
        assert sub.message_type == MessageType.COMMAND
        assert sub.active is True

    def test_subscription_defaults(self):
        """Test Subscription has correct defaults."""
        sub = Subscription(
            id="sub-001",
            message_type=MessageType.COMMAND,
            callback=lambda x: None,
        )
        assert sub.active is True
        assert sub.filter is None


# ============================================================================
# MessageBus Tests
# ============================================================================

class TestMessageBus:
    """Tests for MessageBus."""

    def test_bus_creation(self):
        """Test MessageBus can be created."""
        bus = MessageBus(max_queue_size=100)
        assert bus._running is False

    @pytest.mark.asyncio
    async def test_start_stop(self, message_bus):
        """Test starting and stopping MessageBus."""
        await message_bus.start()
        assert message_bus._running is True
        
        await message_bus.stop()
        assert message_bus._running is False

    @pytest.mark.asyncio
    async def test_publish_not_running(self, message_bus, sample_message):
        """Test publish fails when not running."""
        result = await message_bus.publish(sample_message)
        assert result is False

    @pytest.mark.asyncio
    async def test_publish_success(self, message_bus, sample_message):
        """Test successful message publish."""
        await message_bus.start()
        
        result = await message_bus.publish(sample_message)
        assert result is True

    @pytest.mark.asyncio
    async def test_subscribe(self, message_bus):
        """Test subscribing to messages."""
        async def callback(msg):
            pass
        
        sub_id = await message_bus.subscribe(
            MessageType.COMMAND,
            callback,
        )
        
        assert sub_id.startswith("sub-")
        assert MessageType.COMMAND in message_bus._subscriptions

    @pytest.mark.asyncio
    async def test_subscribe_with_filter(self, message_bus):
        """Test subscribing with filter function."""
        async def callback(msg):
            pass
        
        def filter_func(msg):
            return msg.priority > 0
        
        sub_id = await message_bus.subscribe(
            MessageType.COMMAND,
            callback,
            filter_func=filter_func,
        )
        
        assert sub_id is not None

    def test_unsubscribe(self, message_bus):
        """Test unsubscribing from messages."""
        message_bus._subscriptions[MessageType.COMMAND] = [
            Subscription(
                id="sub-001",
                message_type=MessageType.COMMAND,
                callback=lambda x: None,
            )
        ]
        
        result = message_bus.unsubscribe("sub-001")
        assert result is True
        assert len(message_bus._subscriptions[MessageType.COMMAND]) == 0

    def test_unsubscribe_nonexistent(self, message_bus):
        """Test unsubscribing non-existent subscription."""
        result = message_bus.unsubscribe("nonexistent")
        assert result is False

    def test_register_handler(self, message_bus):
        """Test registering a handler."""
        async def handler(msg):
            pass
        
        message_bus.register_handler(MessageType.COMMAND, handler)
        
        assert MessageType.COMMAND in message_bus._handlers

    @pytest.mark.asyncio
    async def test_message_delivery(self, message_bus, sample_message):
        """Test message is delivered to subscriber."""
        received = []
        
        async def callback(msg):
            received.append(msg)
        
        await message_bus.subscribe(MessageType.COMMAND, callback)
        await message_bus.start()
        
        # Give time for the process loop to start
        await asyncio.sleep(0.05)
        
        await message_bus.publish(sample_message)
        
        # Give time for processing
        await asyncio.sleep(0.1)
        
        assert len(received) == 1
        assert received[0].id == sample_message.id
        
        await message_bus.stop()

    @pytest.mark.asyncio
    async def test_message_filtering(self, message_bus):
        """Test message filtering."""
        received = []
        
        async def callback(msg):
            received.append(msg)
        
        def filter_func(msg):
            return msg.priority > 0
        
        await message_bus.subscribe(MessageType.COMMAND, callback, filter_func=filter_func)
        await message_bus.start()
        
        await asyncio.sleep(0.05)
        
        # Low priority message - should be filtered
        low_priority = Message(
            id="low",
            type=MessageType.COMMAND,
            source="test",
            priority=0,
        )
        await message_bus.publish(low_priority)
        
        # High priority message - should pass
        high_priority = Message(
            id="high",
            type=MessageType.COMMAND,
            source="test",
            priority=1,
        )
        await message_bus.publish(high_priority)
        
        await asyncio.sleep(0.1)
        
        assert len(received) == 1
        assert received[0].id == "high"
        
        await message_bus.stop()

    @pytest.mark.asyncio
    async def test_handler_invocation(self, message_bus, sample_message):
        """Test handler is invoked for messages."""
        handled = []
        
        async def handler(msg):
            handled.append(msg)
        
        message_bus.register_handler(MessageType.COMMAND, handler)
        await message_bus.start()
        
        await asyncio.sleep(0.05)
        await message_bus.publish(sample_message)
        await asyncio.sleep(0.1)
        
        assert len(handled) == 1
        
        await message_bus.stop()

    @pytest.mark.asyncio
    async def test_queue_full(self):
        """Test handling when queue is full."""
        bus = MessageBus(max_queue_size=1)
        await bus.start()
        
        await asyncio.sleep(0.05)
        
        # Fill the queue
        msg = Message(id="1", type=MessageType.COMMAND, source="test")
        await bus.publish(msg)
        
        # Second publish might fail or block
        # This tests that it handles gracefully
        
        await bus.stop()

    @pytest.mark.asyncio
    async def test_callback_error_handling(self, message_bus, sample_message):
        """Test error handling in callback."""
        error_raised = []
        
        async def error_callback(msg):
            error_raised.append(msg)
            raise Exception("Test error")
        
        async def success_callback(msg):
            pass
        
        await message_bus.subscribe(MessageType.COMMAND, error_callback)
        await message_bus.subscribe(MessageType.COMMAND, success_callback)
        await message_bus.start()
        
        await asyncio.sleep(0.05)
        await message_bus.publish(sample_message)
        await asyncio.sleep(0.1)
        
        # Should not crash, error is logged
        assert message_bus._running is True
        
        await message_bus.stop()


# ============================================================================
# Integration Tests
# ============================================================================

class TestIntegration:
    """Integration tests for communication module."""

    @pytest.mark.asyncio
    async def test_full_message_flow(self, message_bus):
        """Test complete message flow."""
        received_commands = []
        received_events = []
        
        async def command_handler(msg):
            received_commands.append(msg)
        
        async def event_handler(msg):
            received_events.append(msg)
        
        await message_bus.subscribe(MessageType.COMMAND, command_handler)
        await message_bus.subscribe(MessageType.EVENT, event_handler)
        await message_bus.start()
        
        await asyncio.sleep(0.05)
        
        # Publish messages
        cmd_msg = Message(id="cmd-1", type=MessageType.COMMAND, source="test")
        evt_msg = Message(id="evt-1", type=MessageType.EVENT, source="test")
        
        await message_bus.publish(cmd_msg)
        await message_bus.publish(evt_msg)
        
        await asyncio.sleep(0.15)
        
        assert len(received_commands) == 1
        assert len(received_events) == 1
        
        await message_bus.stop()

    @pytest.mark.asyncio
    async def test_grpc_client_full_lifecycle(self):
        """Test full gRPC client lifecycle."""
        mock_channel = AsyncMock()
        mock_channel.channel_ready = AsyncMock()
        mock_channel.close = AsyncMock()
        
        with patch('grpc.aio.insecure_channel', return_value=mock_channel):
            client = await create_cnd_client("localhost:50051")
            
            assert client.is_connected
            
            # Use the client
            status = await client.get_status()
            assert status["status"] == "running"
            
            # Disconnect
            await client.disconnect()
            assert not client.is_connected


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])

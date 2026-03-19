"""
Communication Module for Unified Mind.

Provides gRPC and other communication protocols for inter-component
communication between Unified Mind, CND, Living Cells, and Living Kernel.
"""

from .grpc_client import GrpcClient, CndClient, KernelClient
from .message_bus import MessageBus, Message

__all__ = [
    "GrpcClient",
    "CndClient",
    "KernelClient",
    "MessageBus",
    "Message",
]

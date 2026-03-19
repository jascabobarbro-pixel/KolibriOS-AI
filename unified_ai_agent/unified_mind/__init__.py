"""
Unified Mind - The Central AI Intelligence of KolibriOS

This module provides the core AI orchestration capabilities for KolibriOS AI,
acting as the "brain" that coordinates all system components and provides
intelligent responses to user queries.
"""

__version__ = "0.1.0"
__author__ = "KolibriOS AI Team"

from .core.unified_mind import UnifiedMind
from .core.config import UnifiedMindConfig
from .core.state import MindState, SystemState

__all__ = [
    "UnifiedMind",
    "UnifiedMindConfig",
    "MindState",
    "SystemState",
]

"""
State management for Unified Mind.

Provides state tracking for the AI agent and system state monitoring.
"""

from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from typing import Any, Optional
import json


class MindState(str, Enum):
    """States of the Unified Mind agent."""
    INITIALIZING = "initializing"
    READY = "ready"
    PROCESSING = "processing"
    LEARNING = "learning"
    ERROR = "error"
    SHUTDOWN = "shutdown"


class SystemHealth(str, Enum):
    """System health status."""
    HEALTHY = "healthy"
    WARNING = "warning"
    CRITICAL = "critical"
    UNKNOWN = "unknown"


@dataclass
class CellState:
    """State of a single cell."""
    cell_id: str
    cell_type: str
    status: str = "unknown"
    health: SystemHealth = SystemHealth.UNKNOWN
    last_heartbeat: Optional[datetime] = None
    metrics: dict[str, float] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        return {
            "cell_id": self.cell_id,
            "cell_type": self.cell_type,
            "status": self.status,
            "health": self.health.value,
            "last_heartbeat": self.last_heartbeat.isoformat() if self.last_heartbeat else None,
            "metrics": self.metrics,
        }


@dataclass
class SystemState:
    """
    Complete system state for the Unified Mind.

    This class aggregates state information from all system components
    including cells, kernel, and memory.
    """
    timestamp: datetime = field(default_factory=datetime.now)

    # Overall system health
    health: SystemHealth = SystemHealth.UNKNOWN

    # Memory state
    total_memory: int = 0
    used_memory: int = 0
    memory_utilization: float = 0.0

    # CPU state
    total_cores: int = 0
    active_cores: int = 0
    cpu_utilization: float = 0.0

    # Task state
    running_tasks: int = 0
    pending_tasks: int = 0
    completed_tasks: int = 0

    # Network state
    active_connections: int = 0
    bytes_sent: int = 0
    bytes_received: int = 0

    # Cell states
    cells: dict[str, CellState] = field(default_factory=dict)

    # AI/Kernel state
    neural_scheduler_active: bool = False
    gene_states: dict[str, str] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        return {
            "timestamp": self.timestamp.isoformat(),
            "health": self.health.value,
            "memory": {
                "total": self.total_memory,
                "used": self.used_memory,
                "utilization": self.memory_utilization,
            },
            "cpu": {
                "total_cores": self.total_cores,
                "active_cores": self.active_cores,
                "utilization": self.cpu_utilization,
            },
            "tasks": {
                "running": self.running_tasks,
                "pending": self.pending_tasks,
                "completed": self.completed_tasks,
            },
            "network": {
                "active_connections": self.active_connections,
                "bytes_sent": self.bytes_sent,
                "bytes_received": self.bytes_received,
            },
            "cells": {k: v.to_dict() for k, v in self.cells.items()},
            "ai": {
                "neural_scheduler_active": self.neural_scheduler_active,
                "gene_states": self.gene_states,
            },
        }

    def to_json(self) -> str:
        """Convert to JSON string."""
        return json.dumps(self.to_dict(), indent=2)

    def update_from_cnd_metrics(self, metrics: dict[str, Any]) -> None:
        """Update state from CND orchestrator metrics."""
        if "total_memory" in metrics:
            self.total_memory = metrics["total_memory"]
        if "used_memory" in metrics:
            self.used_memory = metrics["used_memory"]
        if "memory_utilization" in metrics:
            self.memory_utilization = metrics["memory_utilization"]
        if "total_cores" in metrics:
            self.total_cores = metrics["total_cores"]
        if "active_cores" in metrics:
            self.active_cores = metrics["active_cores"]
        if "cpu_utilization" in metrics:
            self.cpu_utilization = metrics["cpu_utilization"]

        self.timestamp = datetime.now()
        self._update_health()

    def _update_health(self) -> None:
        """Update overall health based on metrics."""
        issues = []

        if self.memory_utilization > 90:
            issues.append("high_memory")
        if self.cpu_utilization > 90:
            issues.append("high_cpu")

        unhealthy_cells = sum(
            1 for c in self.cells.values()
            if c.health == SystemHealth.CRITICAL
        )
        if unhealthy_cells > 0:
            issues.append(f"unhealthy_cells:{unhealthy_cells}")

        if len(issues) == 0:
            self.health = SystemHealth.HEALTHY
        elif len(issues) <= 2:
            self.health = SystemHealth.WARNING
        else:
            self.health = SystemHealth.CRITICAL

    def get_summary(self) -> str:
        """Get a human-readable summary of system state."""
        lines = [
            f"System Health: {self.health.value.upper()}",
            f"Memory: {self.used_memory / (1024**3):.1f}GB / {self.total_memory / (1024**3):.1f}GB ({self.memory_utilization:.1f}%)",
            f"CPU: {self.active_cores}/{self.total_cores} cores ({self.cpu_utilization:.1f}%)",
            f"Tasks: {self.running_tasks} running, {self.pending_tasks} pending",
            f"Cells: {len(self.cells)} registered",
        ]
        return "\n".join(lines)


@dataclass
class ConversationTurn:
    """A single turn in the conversation."""
    role: str  # "user", "assistant", "system"
    content: str
    timestamp: datetime = field(default_factory=datetime.now)
    metadata: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        return {
            "role": self.role,
            "content": self.content,
            "timestamp": self.timestamp.isoformat(),
            "metadata": self.metadata,
        }


@dataclass
class ConversationHistory:
    """History of conversation turns."""
    turns: list[ConversationTurn] = field(default_factory=list)
    max_turns: int = 100

    def add_turn(self, role: str, content: str, metadata: Optional[dict] = None) -> None:
        """Add a turn to the history."""
        turn = ConversationTurn(
            role=role,
            content=content,
            metadata=metadata or {},
        )
        self.turns.append(turn)

        # Trim if needed
        if len(self.turns) > self.max_turns:
            self.turns = self.turns[-self.max_turns:]

    def get_recent(self, n: int = 10) -> list[ConversationTurn]:
        """Get the most recent n turns."""
        return self.turns[-n:]

    def get_context_string(self, max_turns: int = 20) -> str:
        """Get conversation as context string."""
        recent = self.get_recent(max_turns)
        lines = []
        for turn in recent:
            prefix = "User" if turn.role == "user" else "Assistant"
            lines.append(f"{prefix}: {turn.content}")
        return "\n".join(lines)

    def clear(self) -> None:
        """Clear conversation history."""
        self.turns.clear()


@dataclass
class UserPreferences:
    """User preferences for the AI interaction."""
    preferred_name: Optional[str] = None
    language: str = "en"
    timezone: str = "UTC"
    theme: str = "dark"
    notifications_enabled: bool = True
    proactive_suggestions: bool = True
    auto_optimization: bool = False

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        return {
            "preferred_name": self.preferred_name,
            "language": self.language,
            "timezone": self.timezone,
            "theme": self.theme,
            "notifications_enabled": self.notifications_enabled,
            "proactive_suggestions": self.proactive_suggestions,
            "auto_optimization": self.auto_optimization,
        }

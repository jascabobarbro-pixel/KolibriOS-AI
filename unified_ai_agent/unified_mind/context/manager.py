"""
Context Module for Unified Mind.

Provides contextual understanding and memory management for the AI agent.
"""

from dataclasses import dataclass, field
from datetime import datetime
from typing import Any, Optional
from enum import Enum
import json
import math


class ContextType(str, Enum):
    """Types of context."""
    SYSTEM = "system"
    USER = "user"
    CONVERSATION = "conversation"
    TASK = "task"
    ENVIRONMENT = "environment"


@dataclass
class ContextEntry:
    """A single context entry."""
    id: str
    type: ContextType
    content: str
    timestamp: datetime = field(default_factory=datetime.now)
    relevance: float = 1.0
    metadata: dict[str, Any] = field(default_factory=dict)
    expires_at: Optional[datetime] = None

    def is_expired(self) -> bool:
        """Check if context entry has expired."""
        if self.expires_at is None:
            return False
        return datetime.now() > self.expires_at

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        return {
            "id": self.id,
            "type": self.type.value,
            "content": self.content,
            "timestamp": self.timestamp.isoformat(),
            "relevance": self.relevance,
            "metadata": self.metadata,
            "expires_at": self.expires_at.isoformat() if self.expires_at else None,
        }


@dataclass
class UserContext:
    """Context about the user."""
    user_id: Optional[str] = None
    preferred_name: Optional[str] = None
    language: str = "en"
    timezone: str = "UTC"
    preferences: dict[str, Any] = field(default_factory=dict)
    history_summary: dict[str, Any] = field(default_factory=dict)
    
    # Interaction patterns
    common_queries: list[str] = field(default_factory=list)
    preferred_verbosity: str = "balanced"
    expertise_level: str = "intermediate"


@dataclass
class SystemContext:
    """Context about the system state."""
    # Hardware
    total_memory_gb: float = 0.0
    available_memory_gb: float = 0.0
    cpu_cores: int = 0
    gpu_available: bool = False
    
    # Software
    os_version: str = "KolibriOS AI 0.1.0"
    kernel_version: str = "0.1.0"
    
    # Runtime state
    running_processes: int = 0
    active_connections: int = 0
    last_updated: datetime = field(default_factory=datetime.now)
    
    # Health
    overall_health: str = "unknown"
    alerts: list[dict[str, Any]] = field(default_factory=list)

    def update_from_metrics(self, metrics: dict[str, Any]) -> None:
        """Update from system metrics."""
        self.total_memory_gb = metrics.get("total_memory", 0) / (1024**3)
        self.available_memory_gb = metrics.get("available_memory", 0) / (1024**3)
        self.cpu_cores = metrics.get("cpu_cores", 0)
        self.running_processes = metrics.get("running_processes", 0)
        self.active_connections = metrics.get("active_connections", 0)
        self.overall_health = metrics.get("overall_health", "unknown")
        self.last_updated = datetime.now()


@dataclass
class ConversationContext:
    """Context for the current conversation."""
    session_id: str
    started_at: datetime = field(default_factory=datetime.now)
    turns: list[dict[str, Any]] = field(default_factory=list)
    topics_discussed: list[str] = field(default_factory=list)
    entities_mentioned: dict[str, int] = field(default_factory=dict)
    sentiment: str = "neutral"
    intent_history: list[str] = field(default_factory=list)
    
    def add_turn(self, role: str, content: str, intent: Optional[str] = None) -> None:
        """Add a turn to the conversation."""
        turn = {
            "role": role,
            "content": content,
            "timestamp": datetime.now().isoformat(),
        }
        self.turns.append(turn)
        
        if intent:
            self.intent_history.append(intent)
    
    def get_summary(self) -> str:
        """Get a summary of the conversation."""
        lines = [
            f"Session: {self.session_id}",
            f"Started: {self.started_at.strftime('%Y-%m-%d %H:%M:%S')}",
            f"Turns: {len(self.turns)}",
            f"Topics: {', '.join(self.topics_discussed[:5]) or 'None'}",
            f"Sentiment: {self.sentiment}",
        ]
        return "\n".join(lines)


class ContextManager:
    """
    Manages all context for the Unified Mind.
    
    This class coordinates:
    - User context and preferences
    - System context and state
    - Conversation context and history
    - Task-specific context
    """
    
    def __init__(
        self,
        max_context_entries: int = 1000,
        context_window_tokens: int = 8192,
        relevance_threshold: float = 0.7,
    ) -> None:
        """Initialize the context manager."""
        self.max_entries = max_context_entries
        self.context_window = context_window_tokens
        self.relevance_threshold = relevance_threshold
        
        # Context stores
        self._entries: dict[str, ContextEntry] = {}
        self._user_context = UserContext()
        self._system_context = SystemContext()
        self._conversation_context: Optional[ConversationContext] = None
        
        # Vector store for semantic search (simplified)
        self._embeddings: dict[str, list[float]] = {}
    
    def add_entry(
        self,
        content: str,
        context_type: ContextType,
        relevance: float = 1.0,
        metadata: Optional[dict[str, Any]] = None,
        expires_in_seconds: Optional[int] = None,
    ) -> ContextEntry:
        """
        Add a context entry.
        
        Args:
            content: The context content
            context_type: Type of context
            relevance: Relevance score (0-1)
            metadata: Additional metadata
            expires_in_seconds: Optional expiration time
        
        Returns:
            The created context entry
        """
        entry_id = f"ctx-{context_type.value}-{datetime.now().timestamp()}"
        
        expires_at = None
        if expires_in_seconds:
            from datetime import timedelta
            expires_at = datetime.now() + timedelta(seconds=expires_in_seconds)
        
        entry = ContextEntry(
            id=entry_id,
            type=context_type,
            content=content,
            relevance=relevance,
            metadata=metadata or {},
            expires_at=expires_at,
        )
        
        self._entries[entry_id] = entry
        
        # Cleanup old entries if needed
        if len(self._entries) > self.max_entries:
            self._cleanup_entries()
        
        return entry
    
    def get_relevant_context(
        self,
        query: str,
        max_entries: int = 10,
        context_types: Optional[list[ContextType]] = None,
    ) -> list[ContextEntry]:
        """
        Get context relevant to a query.
        
        Args:
            query: The query to match against
            max_entries: Maximum entries to return
            context_types: Optional filter by context types
        
        Returns:
            List of relevant context entries
        """
        # Filter by type if specified
        candidates = [
            entry for entry in self._entries.values()
            if not entry.is_expired()
        ]
        
        if context_types:
            candidates = [
                entry for entry in candidates
                if entry.type in context_types
            ]
        
        # Score by relevance and recency
        scored = []
        for entry in candidates:
            score = self._calculate_relevance(entry, query)
            if score >= self.relevance_threshold:
                scored.append((entry, score))
        
        # Sort by score and return top entries
        scored.sort(key=lambda x: x[1], reverse=True)
        return [entry for entry, _ in scored[:max_entries]]
    
    def _calculate_relevance(self, entry: ContextEntry, query: str) -> float:
        """Calculate relevance of an entry to a query."""
        query_words = set(query.lower().split())
        content_words = set(entry.content.lower().split())
        
        # Simple word overlap score
        if not query_words or not content_words:
            return 0.0
        
        overlap = len(query_words & content_words)
        score = overlap / len(query_words)
        
        # Boost by entry's base relevance
        score *= entry.relevance
        
        # Decay by age
        age_hours = (datetime.now() - entry.timestamp).total_seconds() / 3600
        age_factor = math.exp(-age_hours / 24)  # Half-life of 24 hours
        score *= age_factor
        
        return score
    
    def _cleanup_entries(self) -> None:
        """Remove expired and low-relevance entries."""
        # Remove expired entries first
        expired = [
            id for id, entry in self._entries.items()
            if entry.is_expired()
        ]
        for id in expired:
            del self._entries[id]
        
        # If still over limit, remove lowest relevance
        if len(self._entries) > self.max_entries:
            sorted_entries = sorted(
                self._entries.items(),
                key=lambda x: x[1].relevance,
            )
            to_remove = len(self._entries) - self.max_entries
            for id, _ in sorted_entries[:to_remove]:
                del self._entries[id]
    
    def start_conversation(self, session_id: Optional[str] = None) -> ConversationContext:
        """Start a new conversation context."""
        session_id = session_id or f"session-{datetime.now().timestamp()}"
        self._conversation_context = ConversationContext(session_id=session_id)
        return self._conversation_context
    
    def get_conversation_context(self) -> Optional[ConversationContext]:
        """Get the current conversation context."""
        return self._conversation_context
    
    def update_user_context(self, updates: dict[str, Any]) -> None:
        """Update user context."""
        for key, value in updates.items():
            if hasattr(self._user_context, key):
                setattr(self._user_context, key, value)
            else:
                self._user_context.preferences[key] = value
    
    def update_system_context(self, metrics: dict[str, Any]) -> None:
        """Update system context from metrics."""
        self._system_context.update_from_metrics(metrics)
    
    def get_user_context(self) -> UserContext:
        """Get user context."""
        return self._user_context
    
    def get_system_context(self) -> SystemContext:
        """Get system context."""
        return self._system_context
    
    def build_full_context(
        self,
        query: str,
        include_user: bool = True,
        include_system: bool = True,
        include_conversation: bool = True,
        include_relevant: bool = True,
    ) -> str:
        """
        Build full context string for LLM.
        
        Args:
            query: The current query
            include_user: Include user context
            include_system: Include system context
            include_conversation: Include conversation context
            include_relevant: Include relevant context entries
        
        Returns:
            Formatted context string
        """
        parts = []
        
        # System context
        if include_system:
            parts.append(f"""System Context:
- OS: {self._system_context.os_version}
- Memory: {self._system_context.available_memory_gb:.1f}GB / {self._system_context.total_memory_gb:.1f}GB
- CPU Cores: {self._system_context.cpu_cores}
- Running Processes: {self._system_context.running_processes}
- Health: {self._system_context.overall_health}
""")
        
        # User context
        if include_user and self._user_context.preferred_name:
            parts.append(f"User: {self._user_context.preferred_name}")
        
        # Conversation context
        if include_conversation and self._conversation_context:
            # Get last few turns
            recent_turns = self._conversation_context.turns[-5:]
            if recent_turns:
                turns_str = "\n".join(
                    f"{t['role']}: {t['content']}"
                    for t in recent_turns
                )
                parts.append(f"Recent Conversation:\n{turns_str}")
        
        # Relevant context
        if include_relevant:
            relevant = self.get_relevant_context(query, max_entries=5)
            if relevant:
                relevant_str = "\n".join(f"- {e.content}" for e in relevant)
                parts.append(f"Relevant Context:\n{relevant_str}")
        
        return "\n\n".join(parts)
    
    def save_to_file(self, filepath: str) -> None:
        """Save context to a file."""
        data = {
            "user_context": {
                "user_id": self._user_context.user_id,
                "preferred_name": self._user_context.preferred_name,
                "language": self._user_context.language,
                "timezone": self._user_context.timezone,
                "preferences": self._user_context.preferences,
            },
            "entries": [e.to_dict() for e in self._entries.values()],
        }
        
        with open(filepath, "w") as f:
            json.dump(data, f, indent=2)
    
    def load_from_file(self, filepath: str) -> None:
        """Load context from a file."""
        with open(filepath, "r") as f:
            data = json.load(f)
        
        if "user_context" in data:
            uc = data["user_context"]
            self._user_context.user_id = uc.get("user_id")
            self._user_context.preferred_name = uc.get("preferred_name")
            self._user_context.language = uc.get("language", "en")
            self._user_context.timezone = uc.get("timezone", "UTC")
            self._user_context.preferences = uc.get("preferences", {})
        
        if "entries" in data:
            for entry_data in data["entries"]:
                entry = ContextEntry(
                    id=entry_data["id"],
                    type=ContextType(entry_data["type"]),
                    content=entry_data["content"],
                    relevance=entry_data.get("relevance", 1.0),
                    metadata=entry_data.get("metadata", {}),
                )
                self._entries[entry.id] = entry

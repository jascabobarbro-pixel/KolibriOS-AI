"""
Base LLM Client Interface.

Defines the interface that all LLM clients must implement.
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Optional, Any
from enum import Enum


class MessageRole(str, Enum):
    """Message roles in conversation."""
    SYSTEM = "system"
    USER = "user"
    ASSISTANT = "assistant"


@dataclass
class ChatMessage:
    """A message in the conversation."""
    role: MessageRole
    content: str
    name: Optional[str] = None

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        result = {"role": self.role.value, "content": self.content}
        if self.name:
            result["name"] = self.name
        return result


@dataclass
class LLMResponse:
    """Response from LLM."""
    content: str
    model: str
    tokens_used: int = 0
    finish_reason: str = "stop"
    metadata: dict[str, Any] = None  # type: ignore

    def __post_init__(self):
        if self.metadata is None:
            self.metadata = {}


class BaseLLMClient(ABC):
    """
    Abstract base class for LLM clients.

    All LLM integrations must implement this interface to ensure
    consistent behavior across different providers.
    """

    def __init__(
        self,
        model: str,
        max_tokens: int = 4096,
        temperature: float = 0.7,
        top_p: float = 0.95,
        timeout: float = 30.0,
    ) -> None:
        """Initialize the LLM client."""
        self.model = model
        self.max_tokens = max_tokens
        self.temperature = temperature
        self.top_p = top_p
        self.timeout = timeout

    @abstractmethod
    async def generate(
        self,
        prompt: str,
        context: Optional[str] = None,
        system_prompt: Optional[str] = None,
    ) -> str:
        """
        Generate a response from the LLM.

        Args:
            prompt: The user's input prompt
            context: Additional context for the conversation
            system_prompt: System instructions for the LLM

        Returns:
            The generated response text
        """
        pass

    @abstractmethod
    async def generate_with_history(
        self,
        messages: list[ChatMessage],
        system_prompt: Optional[str] = None,
    ) -> LLMResponse:
        """
        Generate a response with full conversation history.

        Args:
            messages: List of previous messages in the conversation
            system_prompt: System instructions for the LLM

        Returns:
            Full LLM response with metadata
        """
        pass

    @abstractmethod
    async def is_available(self) -> bool:
        """
        Check if the LLM is available.

        Returns:
            True if the LLM can be used, False otherwise
        """
        pass

    def get_system_prompt(self, context: Optional[str] = None) -> str:
        """
        Get the default system prompt.

        Args:
            context: Additional context to include

        Returns:
            The system prompt string
        """
        base_prompt = """You are Kolibri, the unified AI assistant for KolibriOS AI.

You are an intelligent, helpful assistant integrated into the KolibriOS AI operating system.
Your role is to:
1. Help users manage and optimize their system
2. Provide clear, accurate information about system state
3. Execute commands and report results
4. Learn from interactions to improve future responses

Guidelines:
- Be concise but thorough
- Prioritize user privacy and security
- Explain technical concepts clearly
- When in doubt, ask for clarification
- Report errors honestly and suggest solutions
"""
        if context:
            return f"{base_prompt}\n\nCurrent System Context:\n{context}"
        return base_prompt

    def build_prompt_with_context(
        self,
        user_input: str,
        context: Optional[str] = None,
        system_prompt: Optional[str] = None,
    ) -> str:
        """
        Build a complete prompt with context.

        Args:
            user_input: The user's input
            context: Additional context
            system_prompt: Custom system prompt

        Returns:
            The complete prompt string
        """
        parts = []

        if system_prompt:
            parts.append(system_prompt)
        else:
            parts.append(self.get_system_prompt(context))

        if context:
            parts.append(f"\nContext:\n{context}")

        parts.append(f"\nUser: {user_input}")
        parts.append("\nAssistant:")

        return "\n".join(parts)

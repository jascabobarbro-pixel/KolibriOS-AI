"""
LLM Integration Module.

Provides integrations with various LLM providers:
- Google Gemini
- OpenAI (GPT-4, GPT-3.5)
- Anthropic Claude
- Ollama (Local LLMs)
- Local Llama (llama.cpp)
"""

from .base import BaseLLMClient, ChatMessage, LLMResponse, MessageRole
from .gemini_client import GeminiClient

# OpenAI client
try:
    from .openai_client import OpenAIClient
except ImportError:
    OpenAIClient = None  # type: ignore

# Anthropic client
try:
    from .anthropic_client import AnthropicClient
except ImportError:
    AnthropicClient = None  # type: ignore

# Ollama client
try:
    from .ollama_client import OllamaClient
except ImportError:
    OllamaClient = None  # type: ignore

# Local Llama client
try:
    from .llama_client import LlamaClient
except ImportError:
    LlamaClient = None  # type: ignore


# Factory function to create clients
def create_client(
    provider: str,
    model: str = None,
    **kwargs,
) -> BaseLLMClient:
    """
    Create an LLM client based on provider.

    Args:
        provider: Provider name ("gemini", "openai", "anthropic", "ollama", "llama")
        model: Model name (optional, uses defaults)
        **kwargs: Additional arguments passed to the client

    Returns:
        Configured LLM client

    Raises:
        ValueError: If provider is not supported
        ImportError: If required package is not installed
    """
    provider = provider.lower()

    if provider == "gemini":
        return GeminiClient(model=model or "gemini-1.5-flash", **kwargs)
    elif provider == "openai":
        if OpenAIClient is None:
            raise ImportError("OpenAI package not installed. pip install openai")
        return OpenAIClient(model=model or "gpt-4o-mini", **kwargs)
    elif provider == "anthropic":
        if AnthropicClient is None:
            raise ImportError("Anthropic package not installed. pip install anthropic")
        return AnthropicClient(model=model or "claude-3-5-sonnet-20241022", **kwargs)
    elif provider == "ollama":
        if OllamaClient is None:
            raise ImportError("aiohttp package not installed. pip install aiohttp")
        return OllamaClient(model=model or "llama3", **kwargs)
    elif provider == "llama" or provider == "local":
        if LlamaClient is None:
            raise ImportError("llama-cpp-python not installed. pip install llama-cpp-python")
        return LlamaClient(model=model or "local-llama", **kwargs)
    else:
        raise ValueError(
            f"Unknown provider: {provider}. "
            f"Supported: gemini, openai, anthropic, ollama, llama"
        )


# Multi-provider client for fallback/routing
class MultiProviderClient:
    """
    Client that can route requests to multiple LLM providers.
    
    Supports:
    - Automatic fallback when a provider is unavailable
    - Load balancing across providers
    - Cost-based routing
    - Latency-based routing
    """

    def __init__(
        self,
        providers: list[dict[str, Any]],
        fallback_order: list[str] = None,
        routing_strategy: str = "fallback",
    ) -> None:
        """
        Initialize multi-provider client.

        Args:
            providers: List of provider configs
            fallback_order: Order of providers for fallback
            routing_strategy: "fallback", "round_robin", "cost", "latency"
        """
        self.providers = {}
        self.routing_strategy = routing_strategy
        self.fallback_order = fallback_order or []
        self._current_index = 0

        for config in providers:
            provider_name = config.pop("provider")
            client = create_client(provider_name, **config)
            self.providers[provider_name] = client
            if provider_name not in self.fallback_order:
                self.fallback_order.append(provider_name)

    async def generate(
        self,
        prompt: str,
        context: Optional[str] = None,
        system_prompt: Optional[str] = None,
    ) -> str:
        """Generate using the configured routing strategy."""
        if self.routing_strategy == "fallback":
            return await self._generate_with_fallback(prompt, context, system_prompt)
        elif self.routing_strategy == "round_robin":
            return await self._generate_round_robin(prompt, context, system_prompt)
        else:
            return await self._generate_with_fallback(prompt, context, system_prompt)

    async def _generate_with_fallback(
        self,
        prompt: str,
        context: Optional[str],
        system_prompt: Optional[str],
    ) -> str:
        """Generate with automatic fallback."""
        last_error = None

        for provider_name in self.fallback_order:
            client = self.providers.get(provider_name)
            if client is None:
                continue

            try:
                if await client.is_available():
                    return await client.generate(prompt, context, system_prompt)
            except Exception as e:
                last_error = e
                continue

        raise RuntimeError(
            f"All providers failed. Last error: {last_error}"
        )

    async def _generate_round_robin(
        self,
        prompt: str,
        context: Optional[str],
        system_prompt: Optional[str],
    ) -> str:
        """Generate using round-robin selection."""
        providers = list(self.providers.keys())
        
        for _ in range(len(providers)):
            provider_name = providers[self._current_index]
            self._current_index = (self._current_index + 1) % len(providers)
            
            client = self.providers[provider_name]
            try:
                if await client.is_available():
                    return await client.generate(prompt, context, system_prompt)
            except Exception:
                continue

        raise RuntimeError("No available providers")

    async def is_available(self) -> bool:
        """Check if any provider is available."""
        for client in self.providers.values():
            if await client.is_available():
                return True
        return False


__all__ = [
    "BaseLLMClient",
    "ChatMessage",
    "LLMResponse",
    "MessageRole",
    "GeminiClient",
    "OpenAIClient",
    "AnthropicClient",
    "OllamaClient",
    "LlamaClient",
    "create_client",
    "MultiProviderClient",
]

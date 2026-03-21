"""
Anthropic Claude API Client for Unified Mind.

Provides integration with Anthropic's Claude models.
"""

import logging
import os
from typing import Optional, Any, AsyncGenerator

from .base import BaseLLMClient, ChatMessage, LLMResponse, MessageRole

logger = logging.getLogger(__name__)


class AnthropicClient(BaseLLMClient):
    """
    Client for Anthropic Claude API.

    This client provides access to Anthropic's Claude family of models
    including Claude 3 Opus, Sonnet, and Haiku.
    """

    # Available Claude models
    AVAILABLE_MODELS = {
        "claude-3-opus-20240229": {"context": 200000, "max_output": 4096},
        "claude-3-sonnet-20240229": {"context": 200000, "max_output": 4096},
        "claude-3-haiku-20240307": {"context": 200000, "max_output": 4096},
        "claude-3-5-sonnet-20241022": {"context": 200000, "max_output": 8192},
        "claude-3-5-haiku-20241022": {"context": 200000, "max_output": 8192},
        "claude-2.1": {"context": 200000, "max_output": 4096},
        "claude-2.0": {"context": 100000, "max_output": 4096},
        "claude-instant-1.2": {"context": 100000, "max_output": 4096},
    }

    def __init__(
        self,
        api_key: Optional[str] = None,
        model: str = "claude-3-5-sonnet-20241022",
        max_tokens: int = 4096,
        temperature: float = 0.7,
        top_p: float = 0.95,
        timeout: float = 120.0,
        base_url: Optional[str] = None,
    ) -> None:
        """
        Initialize the Anthropic client.

        Args:
            api_key: Anthropic API key (optional, can be from env)
            model: Model name to use
            max_tokens: Maximum tokens to generate
            temperature: Sampling temperature
            top_p: Top-p sampling parameter
            timeout: Request timeout
            base_url: Custom base URL
        """
        super().__init__(
            model=model,
            max_tokens=max_tokens,
            temperature=temperature,
            top_p=top_p,
            timeout=timeout,
        )
        self.api_key = api_key or os.environ.get("ANTHROPIC_API_KEY")
        self.base_url = base_url or os.environ.get("ANTHROPIC_BASE_URL")
        self._client = None

    async def _initialize_client(self) -> None:
        """Initialize the Anthropic client lazily."""
        if self._client is not None:
            return

        try:
            from anthropic import AsyncAnthropic

            client_kwargs = {}
            if self.api_key:
                client_kwargs["api_key"] = self.api_key
            if self.base_url:
                client_kwargs["base_url"] = self.base_url

            self._client = AsyncAnthropic(**client_kwargs)
            logger.info(f"Anthropic client initialized with model: {self.model}")

        except ImportError:
            raise ImportError(
                "Anthropic package not installed. "
                "Install with: pip install anthropic"
            )
        except Exception as e:
            logger.error(f"Failed to initialize Anthropic client: {e}")
            raise

    async def generate(
        self,
        prompt: str,
        context: Optional[str] = None,
        system_prompt: Optional[str] = None,
    ) -> str:
        """
        Generate a response from Claude.

        Args:
            prompt: The user's input prompt
            context: Additional context for the conversation
            system_prompt: System instructions for the LLM

        Returns:
            The generated response text
        """
        await self._initialize_client()

        # Build system prompt
        full_system = system_prompt or self.get_system_prompt(context)

        if context and not system_prompt:
            full_system = f"{full_system}\n\nContext:\n{context}"

        try:
            response = await self._client.messages.create(
                model=self.model,
                max_tokens=self.max_tokens,
                temperature=self.temperature,
                system=full_system,
                messages=[
                    {"role": "user", "content": prompt}
                ],
            )

            return response.content[0].text

        except Exception as e:
            logger.error(f"Anthropic generation error: {e}")
            raise

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
        await self._initialize_client()

        # Convert messages to Anthropic format
        api_messages = []

        for msg in messages:
            # Anthropic doesn't support system messages in the messages array
            if msg.role != MessageRole.SYSTEM:
                api_messages.append({
                    "role": msg.role.value,
                    "content": msg.content,
                })

        # Get system prompt
        system = system_prompt or self.get_system_prompt()

        try:
            response = await self._client.messages.create(
                model=self.model,
                max_tokens=self.max_tokens,
                temperature=self.temperature,
                system=system,
                messages=api_messages,
            )

            # Extract token usage
            input_tokens = response.usage.input_tokens if response.usage else 0
            output_tokens = response.usage.output_tokens if response.usage else 0

            return LLMResponse(
                content=response.content[0].text,
                model=self.model,
                tokens_used=input_tokens + output_tokens,
                finish_reason=response.stop_reason or "end_turn",
                metadata={
                    "provider": "anthropic",
                    "prompt_tokens": input_tokens,
                    "completion_tokens": output_tokens,
                    "model_used": response.model,
                    "stop_sequence": response.stop_sequence,
                },
            )

        except Exception as e:
            logger.error(f"Anthropic chat generation error: {e}")
            raise

    async def generate_stream(
        self,
        messages: list[ChatMessage],
        system_prompt: Optional[str] = None,
    ) -> AsyncGenerator[str, None]:
        """
        Generate a streaming response.

        Args:
            messages: List of previous messages
            system_prompt: System instructions

        Yields:
            Chunks of generated text
        """
        await self._initialize_client()

        api_messages = []

        for msg in messages:
            if msg.role != MessageRole.SYSTEM:
                api_messages.append({
                    "role": msg.role.value,
                    "content": msg.content,
                })

        system = system_prompt or self.get_system_prompt()

        try:
            async with self._client.messages.stream(
                model=self.model,
                max_tokens=self.max_tokens,
                temperature=self.temperature,
                system=system,
                messages=api_messages,
            ) as stream:
                async for text in stream.text_stream:
                    yield text

        except Exception as e:
            logger.error(f"Anthropic streaming error: {e}")
            raise

    async def is_available(self) -> bool:
        """Check if Anthropic API is available."""
        try:
            await self._initialize_client()

            # Try a minimal request
            response = await self._client.messages.create(
                model=self.model,
                max_tokens=10,
                messages=[{"role": "user", "content": "Hello"}],
            )
            return len(response.content) > 0

        except Exception as e:
            logger.warning(f"Anthropic availability check failed: {e}")
            return False

    async def count_tokens(self, text: str) -> int:
        """
        Count tokens using Anthropic's tokenizer.

        Args:
            text: Text to count tokens for

        Returns:
            Number of tokens
        """
        await self._initialize_client()

        try:
            response = await self._client.messages.count_tokens(
                model=self.model,
                messages=[{"role": "user", "content": text}],
            )
            return response.input_tokens

        except Exception as e:
            logger.warning(f"Token counting failed: {e}")
            # Fallback approximation
            return len(text.split()) * 2

    async def create_embedding(self, text: str) -> list[float]:
        """
        Create an embedding for the given text.

        Note: Anthropic doesn't provide an embedding API, so this
        uses a fallback method.

        Args:
            text: Text to embed

        Returns:
            Embedding vector (placeholder)
        """
        # Anthropic doesn't have an embedding API yet
        # This is a placeholder that raises an informative error
        raise NotImplementedError(
            "Anthropic does not provide an embedding API. "
            "Use OpenAI or a local model for embeddings."
        )

    async def vision_analysis(
        self,
        image_data: str,
        prompt: str,
        media_type: str = "image/jpeg",
        system_prompt: Optional[str] = None,
    ) -> str:
        """
        Analyze an image using Claude's vision capabilities.

        Args:
            image_data: Base64 encoded image data
            prompt: Question or instruction about the image
            media_type: Image media type (image/jpeg, image/png, etc.)
            system_prompt: System instructions

        Returns:
            Analysis result
        """
        await self._initialize_client()

        system = system_prompt or "You are a helpful AI assistant with vision capabilities."

        try:
            response = await self._client.messages.create(
                model=self.model,
                max_tokens=self.max_tokens,
                system=system,
                messages=[
                    {
                        "role": "user",
                        "content": [
                            {
                                "type": "image",
                                "source": {
                                    "type": "base64",
                                    "media_type": media_type,
                                    "data": image_data,
                                },
                            },
                            {
                                "type": "text",
                                "text": prompt,
                            },
                        ],
                    }
                ],
            )

            return response.content[0].text

        except Exception as e:
            logger.error(f"Anthropic vision analysis error: {e}")
            raise

    async def tool_use(
        self,
        messages: list[ChatMessage],
        tools: list[dict[str, Any]],
        system_prompt: Optional[str] = None,
    ) -> tuple[str, list[dict[str, Any]]]:
        """
        Generate a response with tool use capabilities.

        Args:
            messages: Conversation history
            tools: List of tool definitions
            system_prompt: System instructions

        Returns:
            Tuple of (content, tool_use_results)
        """
        await self._initialize_client()

        api_messages = []

        for msg in messages:
            if msg.role != MessageRole.SYSTEM:
                api_messages.append({
                    "role": msg.role.value,
                    "content": msg.content,
                })

        system = system_prompt or self.get_system_prompt()

        try:
            response = await self._client.messages.create(
                model=self.model,
                max_tokens=self.max_tokens,
                temperature=self.temperature,
                system=system,
                messages=api_messages,
                tools=tools,
            )

            # Extract content and tool use blocks
            text_content = ""
            tool_results = []

            for block in response.content:
                if hasattr(block, 'text'):
                    text_content += block.text
                elif block.type == "tool_use":
                    tool_results.append({
                        "id": block.id,
                        "name": block.name,
                        "input": block.input,
                    })

            return text_content, tool_results

        except Exception as e:
            logger.error(f"Anthropic tool use error: {e}")
            raise


def create_anthropic_client(
    api_key: Optional[str] = None,
    model: str = "claude-3-5-sonnet-20241022",
    **kwargs: Any,
) -> AnthropicClient:
    """
    Create an Anthropic client with default settings.

    Args:
        api_key: Anthropic API key (optional, can be from env)
        model: Model name to use
        **kwargs: Additional arguments passed to AnthropicClient

    Returns:
        Configured AnthropicClient instance
    """
    return AnthropicClient(
        api_key=api_key,
        model=model,
        **kwargs,
    )

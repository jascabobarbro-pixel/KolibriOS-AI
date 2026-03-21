"""
OpenAI API Client for Unified Mind.

Provides integration with OpenAI's GPT models.
"""

import logging
import os
from typing import Optional, Any, AsyncGenerator

from .base import BaseLLMClient, ChatMessage, LLMResponse, MessageRole

logger = logging.getLogger(__name__)


class OpenAIClient(BaseLLMClient):
    """
    Client for OpenAI API.

    This client provides access to OpenAI's GPT family of models
    including GPT-4, GPT-4 Turbo, and GPT-3.5 Turbo.
    """

    # Available models with their context windows
    AVAILABLE_MODELS = {
        "gpt-4": {"context": 8192, "max_output": 4096},
        "gpt-4-turbo": {"context": 128000, "max_output": 4096},
        "gpt-4-turbo-preview": {"context": 128000, "max_output": 4096},
        "gpt-4o": {"context": 128000, "max_output": 4096},
        "gpt-4o-mini": {"context": 128000, "max_output": 16384},
        "gpt-3.5-turbo": {"context": 16385, "max_output": 4096},
        "gpt-3.5-turbo-16k": {"context": 16385, "max_output": 4096},
    }

    def __init__(
        self,
        api_key: Optional[str] = None,
        model: str = "gpt-4o-mini",
        max_tokens: int = 4096,
        temperature: float = 0.7,
        top_p: float = 0.95,
        timeout: float = 60.0,
        organization: Optional[str] = None,
        base_url: Optional[str] = None,
    ) -> None:
        """
        Initialize the OpenAI client.

        Args:
            api_key: OpenAI API key (optional, can be from env)
            model: Model name to use
            max_tokens: Maximum tokens to generate
            temperature: Sampling temperature
            top_p: Top-p sampling parameter
            timeout: Request timeout
            organization: OpenAI organization ID
            base_url: Custom base URL (for proxies/Azure)
        """
        super().__init__(
            model=model,
            max_tokens=max_tokens,
            temperature=temperature,
            top_p=top_p,
            timeout=timeout,
        )
        self.api_key = api_key or os.environ.get("OPENAI_API_KEY")
        self.organization = organization or os.environ.get("OPENAI_ORG_ID")
        self.base_url = base_url or os.environ.get("OPENAI_BASE_URL")
        self._client = None

    async def _initialize_client(self) -> None:
        """Initialize the OpenAI client lazily."""
        if self._client is not None:
            return

        try:
            from openai import AsyncOpenAI

            client_kwargs = {}
            if self.api_key:
                client_kwargs["api_key"] = self.api_key
            if self.organization:
                client_kwargs["organization"] = self.organization
            if self.base_url:
                client_kwargs["base_url"] = self.base_url

            self._client = AsyncOpenAI(**client_kwargs)
            logger.info(f"OpenAI client initialized with model: {self.model}")

        except ImportError:
            raise ImportError(
                "OpenAI package not installed. "
                "Install with: pip install openai"
            )
        except Exception as e:
            logger.error(f"Failed to initialize OpenAI client: {e}")
            raise

    async def generate(
        self,
        prompt: str,
        context: Optional[str] = None,
        system_prompt: Optional[str] = None,
    ) -> str:
        """
        Generate a response from OpenAI.

        Args:
            prompt: The user's input prompt
            context: Additional context for the conversation
            system_prompt: System instructions for the LLM

        Returns:
            The generated response text
        """
        await self._initialize_client()

        messages = []

        # Add system message
        if system_prompt:
            messages.append({"role": "system", "content": system_prompt})
        else:
            messages.append({
                "role": "system",
                "content": self.get_system_prompt(context)
            })

        # Add context if provided
        if context:
            messages.append({
                "role": "system",
                "content": f"Context:\n{context}"
            })

        # Add user message
        messages.append({"role": "user", "content": prompt})

        try:
            response = await self._client.chat.completions.create(
                model=self.model,
                messages=messages,
                max_tokens=self.max_tokens,
                temperature=self.temperature,
                top_p=self.top_p,
            )

            return response.choices[0].message.content

        except Exception as e:
            logger.error(f"OpenAI generation error: {e}")
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

        # Convert messages to OpenAI format
        api_messages = []

        if system_prompt:
            api_messages.append({"role": "system", "content": system_prompt})

        for msg in messages:
            api_messages.append({
                "role": msg.role.value,
                "content": msg.content,
            })

        try:
            response = await self._client.chat.completions.create(
                model=self.model,
                messages=api_messages,
                max_tokens=self.max_tokens,
                temperature=self.temperature,
                top_p=self.top_p,
            )

            choice = response.choices[0]

            return LLMResponse(
                content=choice.message.content,
                model=self.model,
                tokens_used=response.usage.total_tokens if response.usage else 0,
                finish_reason=choice.finish_reason or "stop",
                metadata={
                    "provider": "openai",
                    "prompt_tokens": response.usage.prompt_tokens if response.usage else 0,
                    "completion_tokens": response.usage.completion_tokens if response.usage else 0,
                    "model_used": response.model,
                },
            )

        except Exception as e:
            logger.error(f"OpenAI chat generation error: {e}")
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

        if system_prompt:
            api_messages.append({"role": "system", "content": system_prompt})

        for msg in messages:
            api_messages.append({
                "role": msg.role.value,
                "content": msg.content,
            })

        try:
            stream = await self._client.chat.completions.create(
                model=self.model,
                messages=api_messages,
                max_tokens=self.max_tokens,
                temperature=self.temperature,
                top_p=self.top_p,
                stream=True,
            )

            async for chunk in stream:
                if chunk.choices[0].delta.content:
                    yield chunk.choices[0].delta.content

        except Exception as e:
            logger.error(f"OpenAI streaming error: {e}")
            raise

    async def is_available(self) -> bool:
        """Check if OpenAI API is available."""
        try:
            await self._initialize_client()

            # List models to verify API key works
            models = await self._client.models.list()
            return len(models.data) > 0

        except Exception as e:
            logger.warning(f"OpenAI availability check failed: {e}")
            return False

    async def count_tokens(self, text: str) -> int:
        """
        Count tokens using tiktoken.

        Args:
            text: Text to count tokens for

        Returns:
            Number of tokens
        """
        try:
            import tiktoken

            # Get encoding for model
            try:
                encoding = tiktoken.encoding_for_model(self.model)
            except KeyError:
                # Fallback to cl100k_base (GPT-4 encoding)
                encoding = tiktoken.get_encoding("cl100k_base")

            return len(encoding.encode(text))

        except ImportError:
            # Fallback to approximation
            return len(text.split()) * 2
        except Exception as e:
            logger.warning(f"Token counting failed: {e}")
            return len(text.split()) * 2

    async def create_embedding(
        self,
        text: str,
        model: str = "text-embedding-3-small",
    ) -> list[float]:
        """
        Create an embedding for the given text.

        Args:
            text: Text to embed
            model: Embedding model to use

        Returns:
            Embedding vector
        """
        await self._initialize_client()

        try:
            response = await self._client.embeddings.create(
                model=model,
                input=text,
            )

            return response.data[0].embedding

        except Exception as e:
            logger.error(f"OpenAI embedding error: {e}")
            raise

    async def create_embeddings(
        self,
        texts: list[str],
        model: str = "text-embedding-3-small",
    ) -> list[list[float]]:
        """
        Create embeddings for multiple texts.

        Args:
            texts: List of texts to embed
            model: Embedding model to use

        Returns:
            List of embedding vectors
        """
        await self._initialize_client()

        try:
            response = await self._client.embeddings.create(
                model=model,
                input=texts,
            )

            return [item.embedding for item in response.data]

        except Exception as e:
            logger.error(f"OpenAI batch embedding error: {e}")
            raise

    async def list_models(self) -> list[dict[str, Any]]:
        """List available OpenAI models."""
        await self._initialize_client()

        try:
            models = await self._client.models.list()
            return [
                {
                    "id": model.id,
                    "created": model.created,
                    "owned_by": model.owned_by,
                }
                for model in models.data
            ]
        except Exception as e:
            logger.error(f"Failed to list models: {e}")
            raise

    async def function_call(
        self,
        messages: list[ChatMessage],
        functions: list[dict[str, Any]],
        function_call: str = "auto",
        system_prompt: Optional[str] = None,
    ) -> tuple[Optional[str], Optional[dict[str, Any]]]:
        """
        Generate a response with function calling.

        Args:
            messages: Conversation history
            functions: List of function definitions
            function_call: Function call mode ("auto", "none", or function name)
            system_prompt: System instructions

        Returns:
            Tuple of (content, function_call_result)
        """
        await self._initialize_client()

        api_messages = []

        if system_prompt:
            api_messages.append({"role": "system", "content": system_prompt})

        for msg in messages:
            api_messages.append({
                "role": msg.role.value,
                "content": msg.content,
            })

        try:
            response = await self._client.chat.completions.create(
                model=self.model,
                messages=api_messages,
                functions=functions,
                function_call=function_call,
                max_tokens=self.max_tokens,
                temperature=self.temperature,
            )

            choice = response.choices[0]
            content = choice.message.content

            function_result = None
            if choice.message.function_call:
                import json
                function_result = {
                    "name": choice.message.function_call.name,
                    "arguments": json.loads(choice.message.function_call.arguments),
                }

            return content, function_result

        except Exception as e:
            logger.error(f"OpenAI function call error: {e}")
            raise


def create_openai_client(
    api_key: Optional[str] = None,
    model: str = "gpt-4o-mini",
    **kwargs: Any,
) -> OpenAIClient:
    """
    Create an OpenAI client with default settings.

    Args:
        api_key: OpenAI API key (optional, can be from env)
        model: Model name to use
        **kwargs: Additional arguments passed to OpenAIClient

    Returns:
        Configured OpenAIClient instance
    """
    return OpenAIClient(
        api_key=api_key,
        model=model,
        **kwargs,
    )

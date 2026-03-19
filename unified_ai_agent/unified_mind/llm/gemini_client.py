"""
Gemini API Client for Unified Mind.

Provides integration with Google's Gemini LLM API.
"""

import logging
from typing import Optional, Any

from .base import BaseLLMClient, ChatMessage, LLMResponse

logger = logging.getLogger(__name__)


class GeminiClient(BaseLLMClient):
    """
    Client for Google Gemini API.

    This client provides access to Google's Gemini family of LLMs
    for natural language understanding and generation.
    """

    def __init__(
        self,
        api_key: Optional[str] = None,
        model: str = "gemini-1.5-flash",
        max_tokens: int = 4096,
        temperature: float = 0.7,
        top_p: float = 0.95,
        timeout: float = 30.0,
    ) -> None:
        """Initialize the Gemini client."""
        super().__init__(
            model=model,
            max_tokens=max_tokens,
            temperature=temperature,
            top_p=top_p,
            timeout=timeout,
        )
        self.api_key = api_key
        self._client = None
        self._model_instance = None

    async def _initialize_client(self) -> None:
        """Initialize the Gemini client lazily."""
        if self._client is not None:
            return

        try:
            import google.generativeai as genai

            if self.api_key:
                genai.configure(api_key=self.api_key)

            self._client = genai
            self._model_instance = genai.GenerativeModel(self.model)

            logger.info(f"Gemini client initialized with model: {self.model}")

        except ImportError:
            raise ImportError(
                "Google Generative AI package not installed. "
                "Install with: pip install google-generativeai"
            )
        except Exception as e:
            logger.error(f"Failed to initialize Gemini client: {e}")
            raise

    async def generate(
        self,
        prompt: str,
        context: Optional[str] = None,
        system_prompt: Optional[str] = None,
    ) -> str:
        """
        Generate a response from Gemini.

        Args:
            prompt: The user's input prompt
            context: Additional context for the conversation
            system_prompt: System instructions for the LLM

        Returns:
            The generated response text
        """
        await self._initialize_client()

        # Build the full prompt
        full_prompt = self.build_prompt_with_context(
            user_input=prompt,
            context=context,
            system_prompt=system_prompt,
        )

        try:
            # Configure generation
            generation_config = {
                "temperature": self.temperature,
                "top_p": self.top_p,
                "max_output_tokens": self.max_tokens,
            }

            # Generate response
            response = await self._generate_async(full_prompt, generation_config)

            return response

        except Exception as e:
            logger.error(f"Gemini generation error: {e}")
            raise

    async def _generate_async(
        self,
        prompt: str,
        config: dict[str, Any],
    ) -> str:
        """Generate response asynchronously."""
        import asyncio

        # Run synchronous generation in executor
        loop = asyncio.get_event_loop()

        def _sync_generate():
            response = self._model_instance.generate_content(
                prompt,
                generation_config=config,
            )
            return response.text

        return await loop.run_in_executor(None, _sync_generate)

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

        # Convert messages to Gemini format
        history = []
        for msg in messages:
            role = "user" if msg.role.value == "user" else "model"
            history.append({"role": role, "parts": [msg.content]})

        try:
            # Start chat with history
            chat = self._model_instance.start_chat(history=history)

            # Configure generation
            generation_config = {
                "temperature": self.temperature,
                "top_p": self.top_p,
                "max_output_tokens": self.max_tokens,
            }

            # Get last user message
            last_user_msg = None
            for msg in reversed(messages):
                if msg.role.value == "user":
                    last_user_msg = msg.content
                    break

            if not last_user_msg:
                raise ValueError("No user message found in history")

            # Generate response
            import asyncio
            loop = asyncio.get_event_loop()

            def _sync_generate():
                response = chat.send_message(
                    last_user_msg,
                    generation_config=generation_config,
                )
                return response

            response = await loop.run_in_executor(None, _sync_generate)

            # Extract token usage if available
            tokens_used = 0
            if hasattr(response, 'usage_metadata'):
                tokens_used = getattr(response.usage_metadata, 'total_token_count', 0)

            return LLMResponse(
                content=response.text,
                model=self.model,
                tokens_used=tokens_used,
                finish_reason="stop",
                metadata={
                    "provider": "gemini",
                    "prompt_tokens": getattr(response.usage_metadata, 'prompt_token_count', 0) if hasattr(response, 'usage_metadata') else 0,
                    "completion_tokens": getattr(response.usage_metadata, 'candidates_token_count', 0) if hasattr(response, 'usage_metadata') else 0,
                },
            )

        except Exception as e:
            logger.error(f"Gemini chat generation error: {e}")
            raise

    async def is_available(self) -> bool:
        """Check if Gemini API is available."""
        try:
            await self._initialize_client()

            # Try a simple generation
            response = await self._generate_async(
                "Hello",
                {"max_output_tokens": 10},
            )
            return len(response) > 0

        except Exception as e:
            logger.warning(f"Gemini availability check failed: {e}")
            return False

    async def count_tokens(self, text: str) -> int:
        """Count tokens in text using Gemini's tokenizer."""
        await self._initialize_client()

        try:
            return self._client.count_tokens(text).total_tokens
        except Exception:
            # Fallback to approximation
            return len(text.split()) * 2  # Rough approximation

    async def embed_text(self, text: str) -> list[float]:
        """Generate embeddings for text using Gemini's embedding model."""
        await self._initialize_client()

        try:
            embedding_model = self._client.GenerativeModel('embedding-001')
            result = await self._embed_async(embedding_model, text)
            return result

        except Exception as e:
            logger.error(f"Gemini embedding error: {e}")
            raise

    async def _embed_async(self, model: Any, text: str) -> list[float]:
        """Generate embeddings asynchronously."""
        import asyncio

        loop = asyncio.get_event_loop()

        def _sync_embed():
            result = model.embed_content(text)
            return result.embedding

        return await loop.run_in_executor(None, _sync_embed)


# Utility functions
def create_gemini_client(
    api_key: Optional[str] = None,
    model: str = "gemini-1.5-flash",
    **kwargs: Any,
) -> GeminiClient:
    """
    Create a Gemini client with default settings.

    Args:
        api_key: Gemini API key (optional, can be set via environment)
        model: Model name to use
        **kwargs: Additional arguments passed to GeminiClient

    Returns:
        Configured GeminiClient instance
    """
    return GeminiClient(
        api_key=api_key,
        model=model,
        **kwargs,
    )

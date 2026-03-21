"""
Ollama API Client for Unified Mind.

Provides integration with local Ollama server for running LLMs locally.
"""

import logging
import os
from typing import Optional, Any, AsyncGenerator
import aiohttp

from .base import BaseLLMClient, ChatMessage, LLMResponse, MessageRole

logger = logging.getLogger(__name__)


class OllamaClient(BaseLLMClient):
    """
    Client for Ollama API.

    This client provides access to Ollama's local LLM server
    for running models like Llama 2, Mistral, Gemma, etc. locally.
    """

    # Popular Ollama models
    POPULAR_MODELS = [
        "llama2",
        "llama2:13b",
        "llama2:70b",
        "llama3",
        "llama3:8b",
        "llama3:70b",
        "mistral",
        "mixtral",
        "codellama",
        "deepseek-coder",
        "gemma",
        "gemma:7b",
        "phi3",
        "qwen2",
    ]

    def __init__(
        self,
        model: str = "llama3",
        max_tokens: int = 4096,
        temperature: float = 0.7,
        top_p: float = 0.95,
        timeout: float = 120.0,
        base_url: Optional[str] = None,
        keep_alive: str = "5m",
    ) -> None:
        """
        Initialize the Ollama client.

        Args:
            model: Model name to use
            max_tokens: Maximum tokens to generate
            temperature: Sampling temperature
            top_p: Top-p sampling parameter
            timeout: Request timeout
            base_url: Ollama server URL
            keep_alive: How long to keep model loaded
        """
        super().__init__(
            model=model,
            max_tokens=max_tokens,
            temperature=temperature,
            top_p=top_p,
            timeout=timeout,
        )
        self.base_url = base_url or os.environ.get(
            "OLLAMA_BASE_URL",
            "http://localhost:11434"
        )
        self.keep_alive = keep_alive
        self._session: Optional[aiohttp.ClientSession] = None

    async def _get_session(self) -> aiohttp.ClientSession:
        """Get or create HTTP session."""
        if self._session is None or self._session.closed:
            timeout = aiohttp.ClientTimeout(total=self.timeout)
            self._session = aiohttp.ClientSession(timeout=timeout)
        return self._session

    async def close(self) -> None:
        """Close the HTTP session."""
        if self._session and not self._session.closed:
            await self._session.close()

    async def _request(
        self,
        endpoint: str,
        data: dict[str, Any],
        stream: bool = False,
    ) -> Any:
        """
        Make a request to the Ollama API.

        Args:
            endpoint: API endpoint
            data: Request data
            stream: Whether to stream the response

        Returns:
            Response data
        """
        session = await self._get_session()
        url = f"{self.base_url}/api/{endpoint}"

        try:
            async with session.post(url, json=data) as response:
                response.raise_for_status()

                if stream:
                    return response
                else:
                    return await response.json()

        except aiohttp.ClientError as e:
            logger.error(f"Ollama API error: {e}")
            raise

    async def generate(
        self,
        prompt: str,
        context: Optional[str] = None,
        system_prompt: Optional[str] = None,
    ) -> str:
        """
        Generate a response from Ollama.

        Args:
            prompt: The user's input prompt
            context: Additional context for the conversation
            system_prompt: System instructions for the LLM

        Returns:
            The generated response text
        """
        # Build the full prompt
        full_prompt = self.build_prompt_with_context(
            user_input=prompt,
            context=context,
            system_prompt=system_prompt,
        )

        data = {
            "model": self.model,
            "prompt": full_prompt,
            "stream": False,
            "options": {
                "num_predict": self.max_tokens,
                "temperature": self.temperature,
                "top_p": self.top_p,
            },
            "keep_alive": self.keep_alive,
        }

        try:
            response = await self._request("generate", data)
            return response.get("response", "")

        except Exception as e:
            logger.error(f"Ollama generation error: {e}")
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
        # Convert messages to Ollama format
        ollama_messages = []

        if system_prompt:
            ollama_messages.append({"role": "system", "content": system_prompt})

        for msg in messages:
            ollama_messages.append({
                "role": msg.role.value,
                "content": msg.content,
            })

        data = {
            "model": self.model,
            "messages": ollama_messages,
            "stream": False,
            "options": {
                "num_predict": self.max_tokens,
                "temperature": self.temperature,
                "top_p": self.top_p,
            },
            "keep_alive": self.keep_alive,
        }

        try:
            response = await self._request("chat", data)

            message = response.get("message", {})
            content = message.get("content", "")

            # Extract token usage
            eval_count = response.get("eval_count", 0)
            prompt_eval_count = response.get("prompt_eval_count", 0)
            total_tokens = eval_count + prompt_eval_count

            return LLMResponse(
                content=content,
                model=self.model,
                tokens_used=total_tokens,
                finish_reason="stop",
                metadata={
                    "provider": "ollama",
                    "prompt_tokens": prompt_eval_count,
                    "completion_tokens": eval_count,
                    "model_info": response.get("model", ""),
                },
            )

        except Exception as e:
            logger.error(f"Ollama chat generation error: {e}")
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
        ollama_messages = []

        if system_prompt:
            ollama_messages.append({"role": "system", "content": system_prompt})

        for msg in messages:
            ollama_messages.append({
                "role": msg.role.value,
                "content": msg.content,
            })

        data = {
            "model": self.model,
            "messages": ollama_messages,
            "stream": True,
            "options": {
                "num_predict": self.max_tokens,
                "temperature": self.temperature,
                "top_p": self.top_p,
            },
        }

        session = await self._get_session()
        url = f"{self.base_url}/api/chat"

        try:
            async with session.post(url, json=data) as response:
                response.raise_for_status()

                async for line in response.content:
                    if line:
                        import json
                        try:
                            chunk = json.loads(line)
                            if "message" in chunk:
                                content = chunk["message"].get("content", "")
                                if content:
                                    yield content
                        except json.JSONDecodeError:
                            continue

        except Exception as e:
            logger.error(f"Ollama streaming error: {e}")
            raise

    async def is_available(self) -> bool:
        """Check if Ollama server is available."""
        try:
            session = await self._get_session()
            url = f"{self.base_url}/api/tags"

            async with session.get(url) as response:
                return response.status == 200

        except Exception as e:
            logger.warning(f"Ollama availability check failed: {e}")
            return False

    async def list_models(self) -> list[dict[str, Any]]:
        """List available Ollama models."""
        try:
            session = await self._get_session()
            url = f"{self.base_url}/api/tags"

            async with session.get(url) as response:
                response.raise_for_status()
                data = await response.json()

                return [
                    {
                        "name": model.get("name", ""),
                        "size": model.get("size", 0),
                        "modified_at": model.get("modified_at", ""),
                        "details": model.get("details", {}),
                    }
                    for model in data.get("models", [])
                ]

        except Exception as e:
            logger.error(f"Failed to list Ollama models: {e}")
            raise

    async def pull_model(self, model_name: str) -> bool:
        """
        Pull/download a model from Ollama registry.

        Args:
            model_name: Name of the model to pull

        Returns:
            True if successful
        """
        session = await self._get_session()
        url = f"{self.base_url}/api/pull"

        try:
            async with session.post(url, json={"name": model_name}) as response:
                response.raise_for_status()

                # Read the streaming response
                async for line in response.content:
                    if line:
                        import json
                        try:
                            status = json.loads(line).get("status", "")
                            logger.info(f"Pull progress: {status}")
                        except json.JSONDecodeError:
                            continue

                return True

        except Exception as e:
            logger.error(f"Failed to pull model {model_name}: {e}")
            return False

    async def create_embedding(
        self,
        text: str,
        model: Optional[str] = None,
    ) -> list[float]:
        """
        Create an embedding for the given text.

        Args:
            text: Text to embed
            model: Embedding model (defaults to current model)

        Returns:
            Embedding vector
        """
        data = {
            "model": model or self.model,
            "prompt": text,
        }

        try:
            response = await self._request("embeddings", data)
            return response.get("embedding", [])

        except Exception as e:
            logger.error(f"Ollama embedding error: {e}")
            raise

    async def create_embeddings(
        self,
        texts: list[str],
        model: Optional[str] = None,
    ) -> list[list[float]]:
        """
        Create embeddings for multiple texts.

        Args:
            texts: List of texts to embed
            model: Embedding model

        Returns:
            List of embedding vectors
        """
        embeddings = []
        for text in texts:
            embedding = await self.create_embedding(text, model)
            embeddings.append(embedding)
        return embeddings

    async def show_model_info(self, model_name: Optional[str] = None) -> dict[str, Any]:
        """
        Get information about a model.

        Args:
            model_name: Model name (defaults to current model)

        Returns:
            Model information
        """
        session = await self._get_session()
        url = f"{self.base_url}/api/show"

        try:
            async with session.post(url, json={"name": model_name or self.model}) as response:
                response.raise_for_status()
                return await response.json()

        except Exception as e:
            logger.error(f"Failed to get model info: {e}")
            raise

    async def count_tokens(self, text: str) -> int:
        """
        Count tokens in text.

        Note: Ollama doesn't provide a direct token counting API,
        so this uses an approximation.

        Args:
            text: Text to count tokens for

        Returns:
            Approximate token count
        """
        # Rough approximation: ~4 characters per token
        return len(text) // 4

    async def is_model_loaded(self, model_name: Optional[str] = None) -> bool:
        """Check if a model is currently loaded in memory."""
        try:
            info = await self.show_model_info(model_name)
            return "details" in info
        except Exception:
            return False

    async def unload_model(self, model_name: Optional[str] = None) -> bool:
        """Unload a model from memory."""
        session = await self._get_session()
        url = f"{self.base_url}/api/generate"

        try:
            async with session.post(url, json={
                "model": model_name or self.model,
                "keep_alive": 0,
            }) as response:
                return response.status == 200
        except Exception:
            return False


def create_ollama_client(
    model: str = "llama3",
    **kwargs: Any,
) -> OllamaClient:
    """
    Create an Ollama client with default settings.

    Args:
        model: Model name to use
        **kwargs: Additional arguments passed to OllamaClient

    Returns:
        Configured OllamaClient instance
    """
    return OllamaClient(
        model=model,
        **kwargs,
    )


# Utility function to check Ollama installation
async def check_ollama_installed() -> tuple[bool, str]:
    """
    Check if Ollama is installed and running.

    Returns:
        Tuple of (is_installed, message)
    """
    client = OllamaClient()

    try:
        available = await client.is_available()
        if available:
            models = await client.list_models()
            return True, f"Ollama is running with {len(models)} models available"
        else:
            return False, "Ollama server is not running. Start with: ollama serve"
    except Exception as e:
        return False, f"Ollama check failed: {str(e)}"
    finally:
        await client.close()

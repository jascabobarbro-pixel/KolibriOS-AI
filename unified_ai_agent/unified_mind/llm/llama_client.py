"""
Local Llama Client for Unified Mind.

Provides integration with local Llama models using llama-cpp-python.
"""

import logging
import os
from pathlib import Path
from typing import Optional, Any

from .base import BaseLLMClient, ChatMessage, LLMResponse, MessageRole

logger = logging.getLogger(__name__)


class LlamaClient(BaseLLMClient):
    """
    Client for local Llama models.

    This client provides access to local Llama models using llama-cpp-python
    for on-device AI processing without external API calls.
    """

    def __init__(
        self,
        model_path: Optional[str] = None,
        model: str = "local-llama",
        max_tokens: int = 4096,
        temperature: float = 0.7,
        top_p: float = 0.95,
        timeout: float = 60.0,
        n_ctx: int = 4096,
        n_gpu_layers: int = -1,  # -1 for all layers
        n_threads: int = 4,
        n_batch: int = 512,
        verbose: bool = False,
    ) -> None:
        """
        Initialize the local Llama client.

        Args:
            model_path: Path to the GGUF model file
            model: Model name identifier
            max_tokens: Maximum tokens to generate
            temperature: Sampling temperature
            top_p: Top-p sampling parameter
            timeout: Generation timeout
            n_ctx: Context window size
            n_gpu_layers: Number of GPU layers (-1 for all)
            n_threads: Number of CPU threads
            n_batch: Batch size for prompt processing
            verbose: Enable verbose output
        """
        super().__init__(
            model=model,
            max_tokens=max_tokens,
            temperature=temperature,
            top_p=top_p,
            timeout=timeout,
        )
        self.model_path = model_path
        self.n_ctx = n_ctx
        self.n_gpu_layers = n_gpu_layers
        self.n_threads = n_threads
        self.n_batch = n_batch
        self.verbose = verbose
        self._llama = None
        self._initialized = False

    async def _initialize_model(self) -> None:
        """Initialize the Llama model lazily."""
        if self._initialized and self._llama is not None:
            return

        try:
            from llama_cpp import Llama

            # Find model path
            model_path = self._resolve_model_path()
            if not model_path:
                raise FileNotFoundError(
                    "No model path specified and no default model found. "
                    "Please provide a model_path or set LLAMA_MODEL_PATH environment variable."
                )

            logger.info(f"Loading Llama model from: {model_path}")

            # Initialize Llama
            self._llama = Llama(
                model_path=str(model_path),
                n_ctx=self.n_ctx,
                n_gpu_layers=self.n_gpu_layers,
                n_threads=self.n_threads,
                n_batch=self.n_batch,
                verbose=self.verbose,
            )

            self._initialized = True
            logger.info(f"Llama model loaded successfully: {self.model}")

        except ImportError:
            raise ImportError(
                "llama-cpp-python not installed. "
                "Install with: pip install llama-cpp-python\n"
                "For GPU support: CMAKE_ARGS='-DLLAMA_CUBLAS=on' pip install llama-cpp-python"
            )
        except Exception as e:
            logger.error(f"Failed to initialize Llama model: {e}")
            raise

    def _resolve_model_path(self) -> Optional[Path]:
        """Resolve the model path from various sources."""
        if self.model_path:
            path = Path(self.model_path)
            if path.exists():
                return path

        # Check environment variable
        env_path = os.environ.get("LLAMA_MODEL_PATH")
        if env_path:
            path = Path(env_path)
            if path.exists():
                return path

        # Check common locations
        common_paths = [
            Path.home() / ".cache" / "llama" / "model.gguf",
            Path("/usr/local/share/llama/model.gguf"),
            Path("./models/llama.gguf"),
        ]

        for path in common_paths:
            if path.exists():
                return path

        return None

    async def generate(
        self,
        prompt: str,
        context: Optional[str] = None,
        system_prompt: Optional[str] = None,
    ) -> str:
        """
        Generate a response from the local Llama model.

        Args:
            prompt: The user's input prompt
            context: Additional context for the conversation
            system_prompt: System instructions for the LLM

        Returns:
            The generated response text
        """
        await self._initialize_model()

        # Build the full prompt
        full_prompt = self._build_chat_prompt(
            user_input=prompt,
            context=context,
            system_prompt=system_prompt,
        )

        try:
            import asyncio

            # Run synchronous generation in executor
            loop = asyncio.get_event_loop()

            def _sync_generate():
                output = self._llama(
                    full_prompt,
                    max_tokens=self.max_tokens,
                    temperature=self.temperature,
                    top_p=self.top_p,
                    stop=["</s>", "\nUser:", "\nuser:"],
                    echo=False,
                )
                return output

            output = await loop.run_in_executor(None, _sync_generate)

            # Extract generated text
            generated_text = output["choices"][0]["text"].strip()

            return generated_text

        except Exception as e:
            logger.error(f"Llama generation error: {e}")
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
        await self._initialize_model()

        # Build prompt from messages
        prompt_parts = []

        if system_prompt:
            prompt_parts.append(f"<|system|>\n{system_prompt}</s>\n")

        for msg in messages:
            if msg.role == MessageRole.SYSTEM:
                prompt_parts.append(f"<|system|>\n{msg.content}</s>\n")
            elif msg.role == MessageRole.USER:
                prompt_parts.append(f"<|user|>\n{msg.content}</s>\n")
            elif msg.role == MessageRole.ASSISTANT:
                prompt_parts.append(f"<|assistant|)\n{msg.content}</s>\n")

        prompt_parts.append("<|assistant|)\n")

        full_prompt = "".join(prompt_parts)

        try:
            import asyncio

            loop = asyncio.get_event_loop()

            def _sync_generate():
                output = self._llama(
                    full_prompt,
                    max_tokens=self.max_tokens,
                    temperature=self.temperature,
                    top_p=self.top_p,
                    stop=["</s>", "<|user|>"],
                    echo=False,
                )
                return output

            output = await loop.run_in_executor(None, _sync_generate)

            generated_text = output["choices"][0]["text"].strip()

            # Extract token usage
            tokens_used = output.get("usage", {}).get("total_tokens", 0)

            return LLMResponse(
                content=generated_text,
                model=self.model,
                tokens_used=tokens_used,
                finish_reason=output["choices"][0].get("finish_reason", "stop"),
                metadata={
                    "provider": "local_llama",
                    "model_path": str(self.model_path),
                    "prompt_tokens": output.get("usage", {}).get("prompt_tokens", 0),
                    "completion_tokens": output.get("usage", {}).get("completion_tokens", 0),
                },
            )

        except Exception as e:
            logger.error(f"Llama chat generation error: {e}")
            raise

    def _build_chat_prompt(
        self,
        user_input: str,
        context: Optional[str] = None,
        system_prompt: Optional[str] = None,
    ) -> str:
        """Build a chat-formatted prompt for the model."""
        parts = []

        # System prompt
        if system_prompt:
            parts.append(f"<|system|>\n{system_prompt}</s>\n")
        else:
            parts.append(f"<|system|>\n{self.get_system_prompt(context)}</s>\n")

        # Context
        if context:
            parts.append(f"<|system|>\nContext:\n{context}</s>\n")

        # User input
        parts.append(f"<|user|>\n{user_input}</s>\n")
        parts.append("<|assistant|)\n")

        return "".join(parts)

    async def is_available(self) -> bool:
        """Check if the Llama model is available."""
        try:
            await self._initialize_model()
            return self._llama is not None
        except Exception as e:
            logger.warning(f"Llama availability check failed: {e}")
            return False

    async def count_tokens(self, text: str) -> int:
        """Count tokens in text using the model's tokenizer."""
        await self._initialize_model()

        try:
            tokens = self._llama.tokenize(text.encode("utf-8"))
            return len(tokens)
        except Exception:
            # Fallback to approximation
            return len(text.split()) * 2

    def get_model_info(self) -> dict[str, Any]:
        """Get information about the loaded model."""
        if not self._initialized:
            return {"status": "not_initialized"}

        return {
            "status": "initialized",
            "model_path": str(self.model_path),
            "n_ctx": self.n_ctx,
            "n_gpu_layers": self.n_gpu_layers,
            "n_threads": self.n_threads,
        }

    async def create_embedding(self, text: str) -> list[float]:
        """Create an embedding for the given text."""
        await self._initialize_model()

        try:
            import asyncio

            loop = asyncio.get_event_loop()

            def _sync_embed():
                return self._llama.embed(text)

            embedding = await loop.run_in_executor(None, _sync_embed)
            return embedding.tolist() if hasattr(embedding, 'tolist') else list(embedding)

        except Exception as e:
            logger.error(f"Llama embedding error: {e}")
            raise


# Model download utilities
async def download_model(
    model_url: str,
    output_path: str,
    chunk_size: int = 8192,
) -> Path:
    """
    Download a Llama model from a URL.

    Args:
        model_url: URL to download the model from
        output_path: Path to save the model
        chunk_size: Download chunk size

    Returns:
        Path to the downloaded model
    """
    import aiohttp

    output_file = Path(output_path)
    output_file.parent.mkdir(parents=True, exist_ok=True)

    logger.info(f"Downloading model from {model_url}...")

    async with aiohttp.ClientSession() as session:
        async with session.get(model_url) as response:
            response.raise_for_status()

            total_size = int(response.headers.get("content-length", 0))
            downloaded = 0

            with open(output_file, "wb") as f:
                async for chunk in response.content.iter_chunked(chunk_size):
                    f.write(chunk)
                    downloaded += len(chunk)

                    if total_size:
                        progress = (downloaded / total_size) * 100
                        logger.info(f"Download progress: {progress:.1f}%")

    logger.info(f"Model downloaded to {output_file}")
    return output_file


# Utility functions
def create_llama_client(
    model_path: Optional[str] = None,
    **kwargs: Any,
) -> LlamaClient:
    """
    Create a Llama client with default settings.

    Args:
        model_path: Path to the GGUF model file
        **kwargs: Additional arguments passed to LlamaClient

    Returns:
        Configured LlamaClient instance
    """
    return LlamaClient(
        model_path=model_path,
        **kwargs,
    )


# Recommended models
RECOMMENDED_MODELS = {
    "tiny": {
        "name": "TinyLlama-1.1B",
        "url": "https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf",
        "size_mb": 670,
        "ram_required_gb": 1.5,
        "description": "Smallest model, good for testing",
    },
    "small": {
        "name": "Llama-2-7B-Chat",
        "url": "https://huggingface.co/TheBloke/Llama-2-7B-Chat-GGUF/resolve/main/llama-2-7b-chat.Q4_K_M.gguf",
        "size_mb": 4080,
        "ram_required_gb": 6,
        "description": "Good balance of speed and quality",
    },
    "medium": {
        "name": "Llama-2-13B-Chat",
        "url": "https://huggingface.co/TheBloke/Llama-2-13B-chat-GGUF/resolve/main/llama-2-13b-chat.Q4_K_M.gguf",
        "size_mb": 7860,
        "ram_required_gb": 10,
        "description": "Higher quality responses",
    },
    "mistral": {
        "name": "Mistral-7B-Instruct",
        "url": "https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.2-GGUF/resolve/main/mistral-7b-instruct-v0.2.Q4_K_M.gguf",
        "size_mb": 4370,
        "ram_required_gb": 6,
        "description": "Excellent instruction following",
    },
}

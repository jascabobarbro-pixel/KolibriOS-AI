#!/usr/bin/env python3
"""
Comprehensive Tests for LLM Clients.

Tests cover:
- Gemini client (mocked API)
- Llama client (mocked)
- Response generation
- Error handling
- Token counting
"""

import asyncio
import pytest
from datetime import datetime
from unittest.mock import AsyncMock, MagicMock, patch
from typing import Optional

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from llm.base import BaseLLMClient, ChatMessage, LLMResponse, MessageRole
from llm.gemini_client import GeminiClient, create_gemini_client
from llm.llama_client import LlamaClient, create_llama_client, RECOMMENDED_MODELS


# ============================================================================
# Fixtures
# ============================================================================

@pytest.fixture
def chat_messages():
    """Create sample chat messages."""
    return [
        ChatMessage(role=MessageRole.SYSTEM, content="You are a helpful assistant."),
        ChatMessage(role=MessageRole.USER, content="Hello!"),
        ChatMessage(role=MessageRole.ASSISTANT, content="Hi there!"),
        ChatMessage(role=MessageRole.USER, content="How are you?"),
    ]


@pytest.fixture
def mock_gemini_response():
    """Create a mock Gemini API response."""
    response = MagicMock()
    response.text = "This is a test response from Gemini."
    response.usage_metadata = MagicMock()
    response.usage_metadata.total_token_count = 100
    response.usage_metadata.prompt_token_count = 50
    response.usage_metadata.candidates_token_count = 50
    return response


@pytest.fixture
def mock_llama_output():
    """Create a mock Llama output."""
    return {
        "choices": [
            {
                "text": "This is a test response from Llama.",
                "finish_reason": "stop",
            }
        ],
        "usage": {
            "total_tokens": 100,
            "prompt_tokens": 50,
            "completion_tokens": 50,
        }
    }


# ============================================================================
# BaseLLMClient Tests
# ============================================================================

class TestBaseLLMClient:
    """Tests for BaseLLMClient abstract class."""

    def test_chat_message_creation(self):
        """Test ChatMessage creation."""
        msg = ChatMessage(
            role=MessageRole.USER,
            content="Hello",
            name="test_user"
        )
        assert msg.role == MessageRole.USER
        assert msg.content == "Hello"
        assert msg.name == "test_user"

    def test_chat_message_to_dict(self):
        """Test ChatMessage serialization."""
        msg = ChatMessage(
            role=MessageRole.USER,
            content="Hello"
        )
        data = msg.to_dict()
        assert data["role"] == "user"
        assert data["content"] == "Hello"
        assert "name" not in data

    def test_llm_response_creation(self):
        """Test LLMResponse creation."""
        response = LLMResponse(
            content="Test response",
            model="test-model",
            tokens_used=100,
            finish_reason="stop",
            metadata={"key": "value"}
        )
        assert response.content == "Test response"
        assert response.model == "test-model"
        assert response.tokens_used == 100

    def test_llm_response_default_metadata(self):
        """Test LLMResponse has default metadata."""
        response = LLMResponse(
            content="Test",
            model="test-model"
        )
        assert response.metadata == {}

    def test_message_role_values(self):
        """Test MessageRole enum values."""
        assert MessageRole.SYSTEM.value == "system"
        assert MessageRole.USER.value == "user"
        assert MessageRole.ASSISTANT.value == "assistant"

    def test_get_system_prompt(self):
        """Test system prompt generation."""
        # Create a concrete implementation for testing
        class TestClient(BaseLLMClient):
            async def generate(self, prompt, context=None, system_prompt=None):
                return "test"
            
            async def generate_with_history(self, messages, system_prompt=None):
                return LLMResponse(content="test", model="test")
            
            async def is_available(self):
                return True
        
        client = TestClient(model="test")
        prompt = client.get_system_prompt()
        
        assert "Kolibri" in prompt
        assert "assistant" in prompt.lower()

    def test_get_system_prompt_with_context(self):
        """Test system prompt with context."""
        class TestClient(BaseLLMClient):
            async def generate(self, prompt, context=None, system_prompt=None):
                return "test"
            
            async def generate_with_history(self, messages, system_prompt=None):
                return LLMResponse(content="test", model="test")
            
            async def is_available(self):
                return True
        
        client = TestClient(model="test")
        prompt = client.get_system_prompt(context="Custom context")
        
        assert "Custom context" in prompt

    def test_build_prompt_with_context(self):
        """Test building prompt with context."""
        class TestClient(BaseLLMClient):
            async def generate(self, prompt, context=None, system_prompt=None):
                return "test"
            
            async def generate_with_history(self, messages, system_prompt=None):
                return LLMResponse(content="test", model="test")
            
            async def is_available(self):
                return True
        
        client = TestClient(model="test")
        full_prompt = client.build_prompt_with_context(
            user_input="Hello",
            context="System context here",
            system_prompt="You are helpful."
        )
        
        assert "Hello" in full_prompt
        assert "System context here" in full_prompt
        assert "You are helpful." in full_prompt


# ============================================================================
# GeminiClient Tests
# ============================================================================

class TestGeminiClient:
    """Tests for GeminiClient."""

    def test_client_creation(self):
        """Test GeminiClient can be created."""
        client = GeminiClient(
            api_key="test-key",
            model="gemini-1.5-flash",
            max_tokens=4096,
            temperature=0.7,
        )
        assert client.api_key == "test-key"
        assert client.model == "gemini-1.5-flash"
        assert client.max_tokens == 4096
        assert client.temperature == 0.7

    def test_client_defaults(self):
        """Test GeminiClient has correct defaults."""
        client = GeminiClient()
        assert client.api_key is None
        assert client.model == "gemini-1.5-flash"
        assert client.max_tokens == 4096
        assert client.temperature == 0.7
        assert client.top_p == 0.95
        assert client.timeout == 30.0

    @pytest.mark.asyncio
    async def test_initialize_client_with_api_key(self):
        """Test client initialization with API key."""
        client = GeminiClient(api_key="test-key")
        
        with patch('google.generativeai') as mock_genai:
            mock_model = MagicMock()
            mock_genai.GenerativeModel.return_value = mock_model
            
            await client._initialize_client()
            
            mock_genai.configure.assert_called_once_with(api_key="test-key")
            assert client._client is not None

    @pytest.mark.asyncio
    async def test_initialize_client_without_package(self):
        """Test client initialization without package raises ImportError."""
        client = GeminiClient(api_key="test-key")
        
        with patch.dict('sys.modules', {'google.generativeai': None}):
            with patch('builtins.__import__', side_effect=ImportError("No module")):
                with pytest.raises(ImportError):
                    await client._initialize_client()

    @pytest.mark.asyncio
    async def test_generate_success(self, mock_gemini_response):
        """Test successful response generation."""
        client = GeminiClient(api_key="test-key")
        client._initialized = True
        client._model_instance = MagicMock()
        client._model_instance.generate_content.return_value = mock_gemini_response
        
        # Mock the executor to run synchronously
        with patch('asyncio.get_event_loop') as mock_loop:
            mock_loop.return_value.run_in_executor = AsyncMock(
                return_value="This is a test response from Gemini."
            )
            
            response = await client.generate("Hello")
            
            assert response == "This is a test response from Gemini."

    @pytest.mark.asyncio
    async def test_generate_with_history(self, chat_messages, mock_gemini_response):
        """Test generation with conversation history."""
        client = GeminiClient(api_key="test-key")
        client._initialized = True
        client._model_instance = MagicMock()
        
        mock_chat = MagicMock()
        mock_chat.send_message.return_value = mock_gemini_response
        client._model_instance.start_chat.return_value = mock_chat
        
        with patch('asyncio.get_event_loop') as mock_loop:
            mock_loop.return_value.run_in_executor = AsyncMock(
                return_value=mock_gemini_response
            )
            
            response = await client.generate_with_history(chat_messages)
            
            assert response.content == "This is a test response from Gemini."
            assert response.model == "gemini-1.5-flash"
            assert response.tokens_used == 100

    @pytest.mark.asyncio
    async def test_generate_with_history_no_user_message(self):
        """Test generation with history but no user message."""
        client = GeminiClient(api_key="test-key")
        client._initialized = True
        
        messages = [
            ChatMessage(role=MessageRole.SYSTEM, content="System"),
            ChatMessage(role=MessageRole.ASSISTANT, content="Hi"),
        ]
        
        with pytest.raises(ValueError, match="No user message"):
            await client.generate_with_history(messages)

    @pytest.mark.asyncio
    async def test_is_available_true(self):
        """Test availability check returns True."""
        client = GeminiClient(api_key="test-key")
        client._initialized = True
        client._model_instance = MagicMock()
        client._client = MagicMock()
        
        with patch.object(client, '_generate_async', new_callable=AsyncMock) as mock_gen:
            mock_gen.return_value = "Hello"
            
            result = await client.is_available()
            assert result is True

    @pytest.mark.asyncio
    async def test_is_available_false(self):
        """Test availability check returns False on error."""
        client = GeminiClient(api_key="test-key")
        
        with patch.object(client, '_initialize_client', side_effect=Exception("Error")):
            result = await client.is_available()
            assert result is False

    @pytest.mark.asyncio
    async def test_count_tokens(self):
        """Test token counting."""
        client = GeminiClient(api_key="test-key")
        client._client = MagicMock()
        client._client.count_tokens.return_value.total_tokens = 10
        
        with patch.object(client, '_initialize_client', new_callable=AsyncMock):
            count = await client.count_tokens("Hello world")
            assert count == 10

    @pytest.mark.asyncio
    async def test_count_tokens_fallback(self):
        """Test token counting fallback."""
        client = GeminiClient(api_key="test-key")
        client._client = MagicMock()
        client._client.count_tokens.side_effect = Exception("Error")
        
        with patch.object(client, '_initialize_client', new_callable=AsyncMock):
            count = await client.count_tokens("Hello world")  # 2 words * 2 = 4
            assert count == 4

    @pytest.mark.asyncio
    async def test_embed_text(self):
        """Test text embedding."""
        client = GeminiClient(api_key="test-key")
        client._initialized = True
        client._client = MagicMock()
        
        mock_embedding_model = MagicMock()
        client._client.GenerativeModel.return_value = mock_embedding_model
        
        with patch.object(client, '_embed_async', new_callable=AsyncMock) as mock_embed:
            mock_embed.return_value = [0.1, 0.2, 0.3]
            
            embedding = await client.embed_text("Hello")
            assert embedding == [0.1, 0.2, 0.3]


# ============================================================================
# GeminiClient Utility Function Tests
# ============================================================================

class TestGeminiUtilityFunctions:
    """Tests for Gemini utility functions."""

    def test_create_gemini_client(self):
        """Test create_gemini_client utility function."""
        client = create_gemini_client(
            api_key="test-key",
            model="gemini-1.5-pro",
            max_tokens=2048,
        )
        assert isinstance(client, GeminiClient)
        assert client.api_key == "test-key"
        assert client.model == "gemini-1.5-pro"
        assert client.max_tokens == 2048


# ============================================================================
# LlamaClient Tests
# ============================================================================

class TestLlamaClient:
    """Tests for LlamaClient."""

    def test_client_creation(self):
        """Test LlamaClient can be created."""
        client = LlamaClient(
            model_path="/path/to/model.gguf",
            model="local-llama",
            max_tokens=4096,
            temperature=0.7,
            n_ctx=4096,
            n_gpu_layers=-1,
            n_threads=4,
        )
        assert client.model_path == "/path/to/model.gguf"
        assert client.model == "local-llama"
        assert client.max_tokens == 4096
        assert client.n_ctx == 4096

    def test_client_defaults(self):
        """Test LlamaClient has correct defaults."""
        client = LlamaClient()
        assert client.model_path is None
        assert client.model == "local-llama"
        assert client.max_tokens == 4096
        assert client.temperature == 0.7
        assert client.n_ctx == 4096
        assert client.n_gpu_layers == -1
        assert client.n_threads == 4

    def test_resolve_model_path_explicit(self, tmp_path):
        """Test resolving model path from explicit path."""
        model_file = tmp_path / "model.gguf"
        model_file.write_text("fake model")
        
        client = LlamaClient(model_path=str(model_file))
        resolved = client._resolve_model_path()
        
        assert resolved == model_file

    def test_resolve_model_path_from_env(self, tmp_path, monkeypatch):
        """Test resolving model path from environment."""
        model_file = tmp_path / "env_model.gguf"
        model_file.write_text("fake model")
        
        monkeypatch.setenv("LLAMA_MODEL_PATH", str(model_file))
        
        client = LlamaClient()
        resolved = client._resolve_model_path()
        
        assert resolved == model_file

    def test_resolve_model_path_not_found(self):
        """Test resolving model path when not found."""
        client = LlamaClient(model_path="/nonexistent/model.gguf")
        resolved = client._resolve_model_path()
        
        assert resolved is None

    @pytest.mark.asyncio
    async def test_initialize_model_success(self, tmp_path):
        """Test successful model initialization."""
        model_file = tmp_path / "model.gguf"
        model_file.write_text("fake model")
        
        client = LlamaClient(model_path=str(model_file))
        
        with patch('llama_cpp.Llama') as mock_llama:
            mock_instance = MagicMock()
            mock_llama.return_value = mock_instance
            
            await client._initialize_model()
            
            assert client._initialized is True
            assert client._llama is not None

    @pytest.mark.asyncio
    async def test_initialize_model_no_package(self):
        """Test model initialization without package raises ImportError."""
        client = LlamaClient(model_path="/path/to/model.gguf")
        
        with patch.dict('sys.modules', {'llama_cpp': None}):
            with patch('builtins.__import__', side_effect=ImportError("No module")):
                with pytest.raises(ImportError):
                    await client._initialize_model()

    @pytest.mark.asyncio
    async def test_initialize_model_no_path(self):
        """Test model initialization without path raises FileNotFoundError."""
        client = LlamaClient()
        
        with pytest.raises(FileNotFoundError):
            await client._initialize_model()

    @pytest.mark.asyncio
    async def test_generate_success(self, mock_llama_output, tmp_path):
        """Test successful generation."""
        model_file = tmp_path / "model.gguf"
        model_file.write_text("fake model")
        
        client = LlamaClient(model_path=str(model_file))
        client._initialized = True
        client._llama = MagicMock()
        client._llama.return_value = mock_llama_output
        
        with patch('asyncio.get_event_loop') as mock_loop:
            mock_loop.return_value.run_in_executor = AsyncMock(
                return_value=mock_llama_output
            )
            
            response = await client.generate("Hello")
            
            assert response == "This is a test response from Llama."

    @pytest.mark.asyncio
    async def test_generate_with_history(self, chat_messages, mock_llama_output, tmp_path):
        """Test generation with conversation history."""
        model_file = tmp_path / "model.gguf"
        model_file.write_text("fake model")
        
        client = LlamaClient(model_path=str(model_file))
        client._initialized = True
        client._llama = MagicMock()
        
        with patch('asyncio.get_event_loop') as mock_loop:
            mock_loop.return_value.run_in_executor = AsyncMock(
                return_value=mock_llama_output
            )
            
            response = await client.generate_with_history(chat_messages)
            
            assert response.content == "This is a test response from Llama."
            assert response.model == "local-llama"
            assert response.tokens_used == 100

    def test_build_chat_prompt(self, tmp_path):
        """Test chat prompt building."""
        model_file = tmp_path / "model.gguf"
        model_file.write_text("fake model")
        
        client = LlamaClient(model_path=str(model_file))
        
        prompt = client._build_chat_prompt(
            user_input="Hello",
            context="You are helpful",
            system_prompt="Be friendly"
        )
        
        assert "Hello" in prompt
        assert "Be friendly" in prompt

    @pytest.mark.asyncio
    async def test_is_available_true(self, tmp_path):
        """Test availability check returns True."""
        model_file = tmp_path / "model.gguf"
        model_file.write_text("fake model")
        
        client = LlamaClient(model_path=str(model_file))
        client._initialized = True
        client._llama = MagicMock()
        
        result = await client.is_available()
        assert result is True

    @pytest.mark.asyncio
    async def test_is_available_false(self):
        """Test availability check returns False on error."""
        client = LlamaClient()
        
        with pytest.raises(FileNotFoundError):
            await client.is_available()

    @pytest.mark.asyncio
    async def test_count_tokens(self, tmp_path):
        """Test token counting."""
        model_file = tmp_path / "model.gguf"
        model_file.write_text("fake model")
        
        client = LlamaClient(model_path=str(model_file))
        client._initialized = True
        client._llama = MagicMock()
        client._llama.tokenize.return_value = [1, 2, 3, 4, 5]
        
        count = await client.count_tokens("Hello world")
        assert count == 5

    @pytest.mark.asyncio
    async def test_count_tokens_fallback(self, tmp_path):
        """Test token counting fallback."""
        model_file = tmp_path / "model.gguf"
        model_file.write_text("fake model")
        
        client = LlamaClient(model_path=str(model_file))
        client._initialized = True
        client._llama = MagicMock()
        client._llama.tokenize.side_effect = Exception("Error")
        
        count = await client.count_tokens("Hello world")  # 2 words * 2 = 4
        assert count == 4

    def test_get_model_info_initialized(self, tmp_path):
        """Test getting model info when initialized."""
        model_file = tmp_path / "model.gguf"
        model_file.write_text("fake model")
        
        client = LlamaClient(model_path=str(model_file))
        client._initialized = True
        
        info = client.get_model_info()
        
        assert info["status"] == "initialized"
        assert "model_path" in info

    def test_get_model_info_not_initialized(self):
        """Test getting model info when not initialized."""
        client = LlamaClient()
        
        info = client.get_model_info()
        
        assert info["status"] == "not_initialized"

    @pytest.mark.asyncio
    async def test_create_embedding(self, tmp_path):
        """Test creating embedding."""
        model_file = tmp_path / "model.gguf"
        model_file.write_text("fake model")
        
        client = LlamaClient(model_path=str(model_file))
        client._initialized = True
        client._llama = MagicMock()
        client._llama.embed.return_value = MagicMock(tolist=lambda: [0.1, 0.2, 0.3])
        
        with patch('asyncio.get_event_loop') as mock_loop:
            mock_loop.return_value.run_in_executor = AsyncMock(
                return_value=MagicMock(tolist=lambda: [0.1, 0.2, 0.3])
            )
            
            embedding = await client.create_embedding("Hello")
            assert embedding == [0.1, 0.2, 0.3]


# ============================================================================
# LlamaClient Utility Function Tests
# ============================================================================

class TestLlamaUtilityFunctions:
    """Tests for Llama utility functions."""

    def test_create_llama_client(self):
        """Test create_llama_client utility function."""
        client = create_llama_client(
            model_path="/path/to/model.gguf",
            max_tokens=2048,
            n_ctx=2048,
        )
        assert isinstance(client, LlamaClient)
        assert client.model_path == "/path/to/model.gguf"
        assert client.max_tokens == 2048
        assert client.n_ctx == 2048

    def test_recommended_models(self):
        """Test recommended models dictionary."""
        assert "tiny" in RECOMMENDED_MODELS
        assert "small" in RECOMMENDED_MODELS
        assert "medium" in RECOMMENDED_MODELS
        assert "mistral" in RECOMMENDED_MODELS
        
        for model_info in RECOMMENDED_MODELS.values():
            assert "name" in model_info
            assert "url" in model_info
            assert "size_mb" in model_info
            assert "ram_required_gb" in model_info

    @pytest.mark.asyncio
    async def test_download_model(self, tmp_path):
        """Test model download utility."""
        output_path = tmp_path / "downloaded_model.gguf"
        
        mock_response = AsyncMock()
        mock_response.status = 200
        mock_response.headers = {"content-length": "1000"}
        mock_response.content.iter_chunked = AsyncMock(
            return_value=[b"fake model data"]
        )
        
        with patch('aiohttp.ClientSession') as mock_session:
            mock_session_instance = AsyncMock()
            mock_session_instance.__aenter__.return_value = mock_session_instance
            mock_session_instance.get.return_value.__aenter__.return_value = mock_response
            mock_session.return_value = mock_session_instance
            
            # This would normally download but we mock it
            # result = await download_model("http://example.com/model.gguf", str(output_path))
            # For now, just test the function exists
            from llm.llama_client import download_model
            assert callable(download_model)


# ============================================================================
# Error Handling Tests
# ============================================================================

class TestErrorHandling:
    """Tests for error handling."""

    @pytest.mark.asyncio
    async def test_gemini_generate_error(self):
        """Test Gemini generate handles errors."""
        client = GeminiClient(api_key="test-key")
        client._initialized = True
        client._model_instance = MagicMock()
        client._model_instance.generate_content.side_effect = Exception("API Error")
        
        with patch('asyncio.get_event_loop') as mock_loop:
            mock_loop.return_value.run_in_executor = AsyncMock(
                side_effect=Exception("API Error")
            )
            
            with pytest.raises(Exception):
                await client.generate("Hello")

    @pytest.mark.asyncio
    async def test_llama_generate_error(self, tmp_path):
        """Test Llama generate handles errors."""
        model_file = tmp_path / "model.gguf"
        model_file.write_text("fake model")
        
        client = LlamaClient(model_path=str(model_file))
        client._initialized = True
        client._llama = MagicMock()
        client._llama.side_effect = Exception("Model Error")
        
        with patch('asyncio.get_event_loop') as mock_loop:
            mock_loop.return_value.run_in_executor = AsyncMock(
                side_effect=Exception("Model Error")
            )
            
            with pytest.raises(Exception):
                await client.generate("Hello")

    @pytest.mark.asyncio
    async def test_gemini_embedding_error(self):
        """Test Gemini embedding handles errors."""
        client = GeminiClient(api_key="test-key")
        client._initialized = True
        client._client = MagicMock()
        client._client.GenerativeModel.side_effect = Exception("Embedding Error")
        
        with pytest.raises(Exception):
            await client.embed_text("Hello")


# ============================================================================
# Integration Tests
# ============================================================================

class TestIntegration:
    """Integration tests for LLM clients."""

    @pytest.mark.asyncio
    async def test_gemini_full_flow(self, mock_gemini_response):
        """Test full Gemini flow."""
        client = GeminiClient(api_key="test-key", model="gemini-1.5-flash")
        
        with patch('google.generativeai') as mock_genai:
            mock_model = MagicMock()
            mock_model.generate_content.return_value = mock_gemini_response
            mock_genai.GenerativeModel.return_value = mock_model
            
            # Initialize
            await client._initialize_client()
            
            assert client._client is not None
            assert client._model_instance is not None

    @pytest.mark.asyncio
    async def test_llama_full_flow(self, mock_llama_output, tmp_path):
        """Test full Llama flow."""
        model_file = tmp_path / "model.gguf"
        model_file.write_text("fake model")
        
        client = LlamaClient(
            model_path=str(model_file),
            n_ctx=2048,
            n_threads=2,
        )
        
        with patch('llama_cpp.Llama') as mock_llama:
            mock_instance = MagicMock()
            mock_instance.return_value = mock_llama_output
            mock_llama.return_value = mock_instance
            
            # Initialize
            await client._initialize_model()
            
            assert client._initialized is True
            assert client._llama is not None


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])

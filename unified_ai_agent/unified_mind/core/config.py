"""
Configuration for Unified Mind.

Provides configuration classes for all components of the Unified Mind system.
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional
import os


class LLMProvider(str, Enum):
    """Supported LLM providers."""
    GEMINI = "gemini"
    OPENAI = "openai"
    LOCAL_LLAMA = "local_llama"
    AUTO = "auto"


class PersonalityType(str, Enum):
    """AI personality types."""
    PROFESSIONAL = "professional"
    FRIENDLY = "friendly"
    TECHNICAL = "technical"
    CASUAL = "casual"


class VerbosityLevel(str, Enum):
    """Response verbosity levels."""
    CONCISE = "concise"
    BALANCED = "balanced"
    DETAILED = "detailed"


@dataclass
class LLMConfig:
    """Configuration for LLM integration."""
    provider: LLMProvider = LLMProvider.AUTO
    api_key: Optional[str] = None
    model_name: str = "gemini-1.5-flash"
    max_tokens: int = 4096
    temperature: float = 0.7
    top_p: float = 0.95
    timeout: float = 30.0

    # Local Llama settings
    llama_model_path: Optional[str] = None
    llama_n_ctx: int = 4096
    llama_n_gpu_layers: int = -1  # -1 for all layers
    llama_n_threads: int = 4

    @classmethod
    def from_env(cls) -> "LLMConfig":
        """Create LLMConfig from environment variables."""
        provider_str = os.environ.get("LLM_PROVIDER", "auto").lower()
        provider = LLMProvider(provider_str) if provider_str in [p.value for p in LLMProvider] else LLMProvider.AUTO

        return cls(
            provider=provider,
            api_key=os.environ.get("GEMINI_API_KEY") or os.environ.get("OPENAI_API_KEY"),
            model_name=os.environ.get("LLM_MODEL", "gemini-1.5-flash"),
            max_tokens=int(os.environ.get("LLM_MAX_TOKENS", "4096")),
            temperature=float(os.environ.get("LLM_TEMPERATURE", "0.7")),
            llama_model_path=os.environ.get("LLAMA_MODEL_PATH"),
        )


@dataclass
class CommunicationConfig:
    """Configuration for inter-component communication."""
    cnd_endpoint: str = "localhost:50051"
    kernel_endpoint: str = "localhost:50052"
    enable_grpc: bool = True
    grpc_timeout: float = 10.0
    enable_rest: bool = True
    rest_port: int = 8080
    message_queue_size: int = 1000
    heartbeat_interval: float = 5.0


@dataclass
class ContextConfig:
    """Configuration for contextual understanding."""
    max_history_length: int = 100
    max_context_tokens: int = 8192
    enable_learning: bool = True
    memory_persistence_path: Optional[str] = None
    context_window_strategy: str = "sliding"  # sliding, summarization, hierarchical
    relevance_threshold: float = 0.7


@dataclass
class InterfaceConfig:
    """Configuration for user interface."""
    enable_cli: bool = True
    enable_web: bool = False
    enable_voice: bool = False
    web_port: int = 3000
    voice_sample_rate: int = 16000
    welcome_message: str = "Welcome to KolibriOS AI. How can I assist you today?"


@dataclass
class UnifiedMindConfig:
    """
    Complete configuration for the Unified Mind system.

    This class aggregates all configuration for the different components
    of the Unified Mind AI agent.
    """
    name: str = "Kolibri"
    version: str = "0.1.0"
    personality: PersonalityType = PersonalityType.FRIENDLY
    verbosity: VerbosityLevel = VerbosityLevel.BALANCED

    # Component configurations
    llm: LLMConfig = field(default_factory=LLMConfig)
    communication: CommunicationConfig = field(default_factory=CommunicationConfig)
    context: ContextConfig = field(default_factory=ContextConfig)
    interface: InterfaceConfig = field(default_factory=InterfaceConfig)

    # Runtime settings
    debug: bool = False
    log_level: str = "INFO"
    enable_metrics: bool = True
    metrics_port: int = 9091

    @classmethod
    def from_env(cls) -> "UnifiedMindConfig":
        """Create UnifiedMindConfig from environment variables."""
        config = cls()

        # Load LLM config from env
        config.llm = LLMConfig.from_env()

        # Load other settings
        config.debug = os.environ.get("MIND_DEBUG", "false").lower() == "true"
        config.log_level = os.environ.get("MIND_LOG_LEVEL", "INFO")

        # Communication settings
        config.communication.cnd_endpoint = os.environ.get("CND_ENDPOINT", "localhost:50051")
        config.communication.kernel_endpoint = os.environ.get("KERNEL_ENDPOINT", "localhost:50052")

        return config

    def validate(self) -> bool:
        """Validate the configuration."""
        if self.llm.provider == LLMProvider.GEMINI and not self.llm.api_key:
            # API key might be set later, warn but don't fail
            pass

        if self.llm.provider == LLMProvider.LOCAL_LLAMA and not self.llm.llama_model_path:
            # Model path might be set later
            pass

        if self.context.max_history_length < 1:
            return False

        if self.llm.temperature < 0 or self.llm.temperature > 2:
            return False

        return True

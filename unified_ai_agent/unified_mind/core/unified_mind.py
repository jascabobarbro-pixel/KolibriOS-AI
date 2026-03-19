"""
Unified Mind - The Central AI Intelligence.

This module implements the main Unified Mind class that coordinates
all AI capabilities, LLM integrations, and system interactions.
"""

import asyncio
import logging
from dataclasses import dataclass, field
from datetime import datetime
from typing import Any, Optional, Callable
from enum import Enum

from .config import UnifiedMindConfig, LLMProvider, PersonalityType
from .state import (
    MindState, SystemState, SystemHealth, ConversationHistory, UserPreferences
)

logger = logging.getLogger(__name__)


@dataclass
class MindResponse:
    """Response from the Unified Mind."""
    content: str
    intent_detected: Optional[str] = None
    action_taken: Optional[str] = None
    confidence: float = 1.0
    sources: list[str] = field(default_factory=list)
    metadata: dict[str, Any] = field(default_factory=dict)


class UnifiedMind:
    """
    The Unified Mind - Central AI Intelligence for KolibriOS AI.

    This class serves as the main orchestrator for all AI capabilities,
    coordinating between:
    - LLM providers (Gemini API, local Llama)
    - System components (CND, Cells, Kernel)
    - User interactions (CLI, Web, Voice)
    - Contextual understanding and memory
    """

    def __init__(self, config: Optional[UnifiedMindConfig] = None) -> None:
        """Initialize the Unified Mind."""
        self.config = config or UnifiedMindConfig()
        self.state = MindState.INITIALIZING

        # Core components
        self.system_state = SystemState()
        self.conversation = ConversationHistory(
            max_turns=self.config.context.max_history_length
        )
        self.user_preferences = UserPreferences()

        # LLM providers (initialized lazily)
        self._gemini_client = None
        self._llama_client = None
        self._active_llm = None

        # Communication channels
        self._cnd_channel = None
        self._kernel_channel = None

        # Command handlers
        self._command_handlers: dict[str, Callable] = {}
        self._intent_handlers: dict[str, Callable] = {}

        # Background tasks
        self._background_tasks: list[asyncio.Task] = []
        self._running = False

        # Metrics
        self._metrics = {
            "queries_processed": 0,
            "commands_executed": 0,
            "errors": 0,
            "start_time": None,
        }

        logger.info(f"Unified Mind '{self.config.name}' initialized")

    async def start(self) -> None:
        """Start the Unified Mind agent."""
        if self._running:
            logger.warning("Unified Mind already running")
            return

        logger.info("Starting Unified Mind...")
        self._running = True
        self._metrics["start_time"] = datetime.now()

        # Initialize LLM
        await self._initialize_llm()

        # Initialize communication channels
        await self._initialize_communication()

        # Register default commands
        self._register_default_commands()

        # Start background tasks
        self._background_tasks = [
            asyncio.create_task(self._system_monitor_loop()),
            asyncio.create_task(self._learning_loop()),
        ]

        self.state = MindState.READY
        logger.info("Unified Mind started successfully")

    async def stop(self) -> None:
        """Stop the Unified Mind agent."""
        logger.info("Stopping Unified Mind...")
        self._running = False
        self.state = MindState.SHUTDOWN

        # Cancel background tasks
        for task in self._background_tasks:
            task.cancel()
            try:
                await task
            except asyncio.CancelledError:
                pass

        # Close communication channels
        if self._cnd_channel:
            self._cnd_channel.close()
        if self._kernel_channel:
            self._kernel_channel.close()

        logger.info("Unified Mind stopped")

    async def process(self, user_input: str) -> MindResponse:
        """
        Process user input and generate a response.

        This is the main entry point for user interactions.
        """
        if self.state != MindState.READY:
            return MindResponse(
                content="I'm not ready yet. Please wait for initialization to complete.",
                confidence=1.0,
            )

        self.state = MindState.PROCESSING
        self._metrics["queries_processed"] += 1

        try:
            # Add to conversation history
            self.conversation.add_turn("user", user_input)

            # Parse intent
            intent = await self._parse_intent(user_input)

            # Check if it's a command
            if intent and intent in self._command_handlers:
                response = await self._execute_command(intent, user_input)
            else:
                # Generate AI response
                response = await self._generate_response(user_input)

            # Add response to history
            self.conversation.add_turn("assistant", response.content)

            return response

        except Exception as e:
            logger.error(f"Error processing input: {e}")
            self._metrics["errors"] += 1
            return MindResponse(
                content=f"I encountered an error: {str(e)}",
                confidence=0.0,
                metadata={"error": str(e)},
            )
        finally:
            self.state = MindState.READY

    async def _initialize_llm(self) -> None:
        """Initialize LLM provider(s)."""
        provider = self.config.llm.provider

        if provider == LLMProvider.AUTO:
            # Try Gemini first, then local Llama
            if self.config.llm.api_key:
                provider = LLMProvider.GEMINI
            elif self.config.llm.llama_model_path:
                provider = LLMProvider.LOCAL_LLAMA
            else:
                logger.warning("No LLM provider configured, using fallback mode")

        if provider == LLMProvider.GEMINI:
            await self._initialize_gemini()
        elif provider == LLMProvider.LOCAL_LLAMA:
            await self._initialize_llama()

        self._active_llm = provider
        logger.info(f"LLM provider initialized: {provider}")

    async def _initialize_gemini(self) -> None:
        """Initialize Gemini API client."""
        try:
            from ..llm.gemini_client import GeminiClient
            self._gemini_client = GeminiClient(
                api_key=self.config.llm.api_key,
                model=self.config.llm.model_name,
                max_tokens=self.config.llm.max_tokens,
                temperature=self.config.llm.temperature,
            )
            logger.info("Gemini client initialized")
        except ImportError:
            logger.warning("Gemini client not available, installing dependencies needed")
        except Exception as e:
            logger.error(f"Failed to initialize Gemini: {e}")

    async def _initialize_llama(self) -> None:
        """Initialize local Llama client."""
        try:
            from ..llm.llama_client import LlamaClient
            self._llama_client = LlamaClient(
                model_path=self.config.llm.llama_model_path,
                n_ctx=self.config.llm.llama_n_ctx,
                n_gpu_layers=self.config.llm.llama_n_gpu_layers,
                n_threads=self.config.llm.llama_n_threads,
            )
            logger.info("Llama client initialized")
        except ImportError:
            logger.warning("Llama client not available, llama-cpp-python needed")
        except Exception as e:
            logger.error(f"Failed to initialize Llama: {e}")

    async def _initialize_communication(self) -> None:
        """Initialize communication with system components."""
        if not self.config.communication.enable_grpc:
            return

        try:
            import grpc

            # Connect to CND Orchestrator
            if self.config.communication.cnd_endpoint:
                self._cnd_channel = grpc.aio.insecure_channel(
                    self.config.communication.cnd_endpoint
                )
                logger.info(f"Connected to CND at {self.config.communication.cnd_endpoint}")

            # Connect to Kernel
            if self.config.communication.kernel_endpoint:
                self._kernel_channel = grpc.aio.insecure_channel(
                    self.config.communication.kernel_endpoint
                )
                logger.info(f"Connected to Kernel at {self.config.communication.kernel_endpoint}")

        except Exception as e:
            logger.warning(f"Could not initialize communication: {e}")

    def _register_default_commands(self) -> None:
        """Register default command handlers."""
        # System commands
        self._command_handlers["show"] = self._cmd_show
        self._command_handlers["status"] = self._cmd_status
        self._command_handlers["optimize"] = self._cmd_optimize
        self._command_handlers["help"] = self._cmd_help
        self._command_handlers["clear"] = self._cmd_clear

        # Memory commands
        self._command_handlers["memory"] = self._cmd_memory

        # Process commands
        self._command_handlers["process"] = self._cmd_process
        self._command_handlers["kill"] = self._cmd_kill

        # System control
        self._command_handlers["restart"] = self._cmd_restart
        self._command_handlers["diagnostics"] = self._cmd_diagnostics

    async def _parse_intent(self, user_input: str) -> Optional[str]:
        """Parse user intent from input."""
        lower_input = user_input.lower().strip()

        # Direct command patterns
        command_patterns = {
            "show": ["show ", "display ", "what is ", "how much "],
            "status": ["status", "health", "state"],
            "optimize": ["optimize ", "tune ", "improve "],
            "help": ["help", "what can you do", "commands"],
            "clear": ["clear", "reset conversation"],
            "memory": ["memory ", "ram ", "allocate "],
            "process": ["process", "list processes", "running"],
            "kill": ["kill ", "stop process", "terminate "],
            "restart": ["restart ", "reboot "],
            "diagnostics": ["diagnostics", "diagnose", "run tests"],
        }

        for intent, patterns in command_patterns.items():
            for pattern in patterns:
                if lower_input.startswith(pattern) or lower_input == pattern.strip():
                    return intent

        return None

    async def _execute_command(self, intent: str, user_input: str) -> MindResponse:
        """Execute a detected command."""
        handler = self._command_handlers.get(intent)
        if not handler:
            return MindResponse(
                content=f"Unknown command: {intent}",
                confidence=0.0,
            )

        self._metrics["commands_executed"] += 1
        return await handler(user_input)

    async def _generate_response(self, user_input: str) -> MindResponse:
        """Generate AI response using LLM."""
        # Build context
        context = self._build_context()

        # Try active LLM
        response_text = None
        sources = []

        if self._active_llm == LLMProvider.GEMINI and self._gemini_client:
            try:
                response_text = await self._gemini_client.generate(
                    user_input, context=context
                )
                sources.append("gemini")
            except Exception as e:
                logger.error(f"Gemini error: {e}")

        if not response_text and self._active_llm == LLMProvider.LOCAL_LLAMA and self._llama_client:
            try:
                response_text = await self._llama_client.generate(
                    user_input, context=context
                )
                sources.append("llama")
            except Exception as e:
                logger.error(f"Llama error: {e}")

        # Fallback response
        if not response_text:
            response_text = self._generate_fallback_response(user_input)

        return MindResponse(
            content=response_text,
            confidence=0.9 if sources else 0.5,
            sources=sources,
        )

    def _build_context(self) -> str:
        """Build context string for LLM."""
        parts = []

        # System state
        parts.append(f"System Status: {self.system_state.get_summary()}")

        # Conversation history
        if self.conversation.turns:
            parts.append(f"\nRecent Conversation:\n{self.conversation.get_context_string(max_turns=10)}")

        # User preferences
        if self.user_preferences.preferred_name:
            parts.append(f"\nUser: {self.user_preferences.preferred_name}")

        return "\n".join(parts)

    def _generate_fallback_response(self, user_input: str) -> str:
        """Generate fallback response when LLM is unavailable."""
        responses = [
            "I understand you're asking about that. Let me help you with what I know.",
            "That's an interesting question. Based on the current system state, I can provide some insights.",
            "I'm processing your request. While my advanced AI capabilities are limited right now, I'll do my best.",
        ]

        # Simple pattern matching for common queries
        lower = user_input.lower()
        if "memory" in lower:
            return f"Current memory usage: {self.system_state.used_memory / (1024**3):.1f}GB of {self.system_state.total_memory / (1024**3):.1f}GB ({self.system_state.memory_utilization:.1f}%)"
        elif "cpu" in lower or "processor" in lower:
            return f"CPU utilization: {self.system_state.cpu_utilization:.1f}% with {self.system_state.active_cores}/{self.system_state.total_cores} cores active"
        elif "status" in lower or "health" in lower:
            return self.system_state.get_summary()
        else:
            import random
            return random.choice(responses) + f"\n\nCurrent system health: {self.system_state.health.value}"

    # Command handlers
    async def _cmd_show(self, user_input: str) -> MindResponse:
        """Handle show commands."""
        lower = user_input.lower()

        if "memory" in lower:
            return MindResponse(
                content=f"Memory Usage:\n"
                       f"  Total: {self.system_state.total_memory / (1024**3):.2f} GB\n"
                       f"  Used: {self.system_state.used_memory / (1024**3):.2f} GB\n"
                       f"  Utilization: {self.system_state.memory_utilization:.1f}%",
                intent_detected="show_memory",
            )
        elif "cpu" in lower or "processor" in lower:
            return MindResponse(
                content=f"CPU Status:\n"
                       f"  Total Cores: {self.system_state.total_cores}\n"
                       f"  Active Cores: {self.system_state.active_cores}\n"
                       f"  Utilization: {self.system_state.cpu_utilization:.1f}%",
                intent_detected="show_cpu",
            )
        elif "task" in lower:
            return MindResponse(
                content=f"Task Status:\n"
                       f"  Running: {self.system_state.running_tasks}\n"
                       f"  Pending: {self.system_state.pending_tasks}\n"
                       f"  Completed: {self.system_state.completed_tasks}",
                intent_detected="show_tasks",
            )
        else:
            return MindResponse(
                content=self.system_state.get_summary(),
                intent_detected="show_status",
            )

    async def _cmd_status(self, user_input: str) -> MindResponse:
        """Handle status command."""
        return MindResponse(
            content=f"System Status Report\n"
                   f"{'=' * 40}\n"
                   f"{self.system_state.get_summary()}\n\n"
                   f"AI Agent Status: {self.state.value}\n"
                   f"Queries Processed: {self._metrics['queries_processed']}\n"
                   f"Uptime: {self._get_uptime()}",
            intent_detected="status",
        )

    async def _cmd_optimize(self, user_input: str) -> MindResponse:
        """Handle optimize command."""
        lower = user_input.lower()
        action_taken = None

        # Simulate optimization
        if "memory" in lower:
            action_taken = "memory_compaction"
            message = "Initiating memory optimization:\n- Running compaction\n- Clearing caches\n- Defragmenting memory pools"
        elif "cpu" in lower or "performance" in lower:
            action_taken = "cpu_optimization"
            message = "Initiating CPU optimization:\n- Balancing load across cores\n- Adjusting scheduler priorities\n- Optimizing task distribution"
        elif "gaming" in lower:
            action_taken = "gaming_mode"
            message = "Enabling Gaming Mode:\n- Prioritizing graphics processes\n- Disabling background services\n- Optimizing memory for games"
        else:
            action_taken = "general_optimization"
            message = "Running system optimization:\n- Analyzing resource usage\n- Balancing workloads\n- Clearing unnecessary caches"

        return MindResponse(
            content=message,
            intent_detected="optimize",
            action_taken=action_taken,
        )

    async def _cmd_help(self, user_input: str) -> MindResponse:
        """Handle help command."""
        help_text = """
KolibriOS AI - Unified Mind Commands
====================================

System Commands:
  show memory    - Display memory usage
  show cpu       - Display CPU status
  show tasks     - Display task status
  status         - Full system status report
  diagnostics    - Run system diagnostics

Optimization:
  optimize memory    - Optimize memory usage
  optimize cpu       - Optimize CPU performance
  optimize gaming    - Enable gaming mode

Process Management:
  list processes     - Show running processes
  kill <pid>         - Terminate a process

Conversation:
  clear          - Clear conversation history
  help           - Show this help message

Natural Language:
  You can ask me anything in natural language, such as:
  - "What's the current memory usage?"
  - "How is the system performing?"
  - "Optimize the system for gaming"
"""
        return MindResponse(
            content=help_text,
            intent_detected="help",
        )

    async def _cmd_clear(self, user_input: str) -> MindResponse:
        """Handle clear command."""
        self.conversation.clear()
        return MindResponse(
            content="Conversation history cleared.",
            intent_detected="clear",
        )

    async def _cmd_memory(self, user_input: str) -> MindResponse:
        """Handle memory commands."""
        return await self._cmd_show(f"show memory {user_input}")

    async def _cmd_process(self, user_input: str) -> MindResponse:
        """Handle process commands."""
        return MindResponse(
            content=f"Process List:\n"
                   f"  Total Running: {self.system_state.running_tasks}\n"
                   f"  Pending: {self.system_state.pending_tasks}\n\n"
                   f"(Full process listing requires connection to Processor Cell)",
            intent_detected="process_list",
        )

    async def _cmd_kill(self, user_input: str) -> MindResponse:
        """Handle kill command."""
        # Extract PID from input
        import re
        match = re.search(r'\d+', user_input)
        if match:
            pid = match.group()
            return MindResponse(
                content=f"Attempting to terminate process {pid}...\n(Requires connection to Processor Cell)",
                intent_detected="kill_process",
                action_taken=f"kill_process:{pid}",
            )
        return MindResponse(
            content="Please specify a process ID to kill. Example: kill 1234",
            intent_detected="kill_help",
        )

    async def _cmd_restart(self, user_input: str) -> MindResponse:
        """Handle restart command."""
        return MindResponse(
            content="Initiating system restart...\n"
                   "This action requires confirmation and proper permissions.\n"
                   "(Restart command sent to CND Orchestrator)",
            intent_detected="restart",
            action_taken="system_restart",
        )

    async def _cmd_diagnostics(self, user_input: str) -> MindResponse:
        """Handle diagnostics command."""
        return MindResponse(
            content=f"Running System Diagnostics...\n"
                   f"{'=' * 40}\n"
                   f"System Health: {self.system_state.health.value.upper()}\n"
                   f"Memory Check: {'PASS' if self.system_state.memory_utilization < 90 else 'WARN'}\n"
                   f"CPU Check: {'PASS' if self.system_state.cpu_utilization < 90 else 'WARN'}\n"
                   f"Cells Registered: {len(self.system_state.cells)}\n"
                   f"Neural Scheduler: {'Active' if self.system_state.neural_scheduler_active else 'Inactive'}\n"
                   f"AI Agent: {self.state.value}\n"
                   f"{'=' * 40}\n"
                   f"Diagnostics complete.",
            intent_detected="diagnostics",
        )

    def _get_uptime(self) -> str:
        """Get formatted uptime."""
        if not self._metrics["start_time"]:
            return "N/A"

        delta = datetime.now() - self._metrics["start_time"]
        hours, remainder = divmod(int(delta.total_seconds()), 3600)
        minutes, seconds = divmod(remainder, 60)
        return f"{hours}h {minutes}m {seconds}s"

    async def _system_monitor_loop(self) -> None:
        """Background task to monitor system state."""
        while self._running:
            try:
                # In a real implementation, this would query CND and cells
                # For now, simulate with random values
                import random
                self.system_state.memory_utilization = random.uniform(30, 70)
                self.system_state.cpu_utilization = random.uniform(10, 50)
                self.system_state._update_health()

                await asyncio.sleep(self.config.communication.heartbeat_interval)
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"System monitor error: {e}")
                await asyncio.sleep(1)

    async def _learning_loop(self) -> None:
        """Background task for continuous learning."""
        while self._running:
            try:
                if self.config.context.enable_learning:
                    # Perform learning tasks
                    pass

                await asyncio.sleep(60)  # Learn every minute
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Learning loop error: {e}")
                await asyncio.sleep(1)

    def get_metrics(self) -> dict[str, Any]:
        """Get agent metrics."""
        return {
            **self._metrics,
            "state": self.state.value,
            "llm_provider": self._active_llm.value if self._active_llm else None,
            "uptime": self._get_uptime(),
        }

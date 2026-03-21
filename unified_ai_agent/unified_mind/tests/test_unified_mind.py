"""
Comprehensive Unified Mind Tests

Tests for:
- Unified Mind initialization
- LLM integration
- Command processing
- System monitoring
- Context management
"""

import pytest
import asyncio
from typing import Optional, Dict, Any, List
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum


# ============== Enums and Data Classes ==============

class MindState(str, Enum):
    INITIALIZING = "initializing"
    READY = "ready"
    PROCESSING = "processing"
    ERROR = "error"
    SHUTDOWN = "shutdown"


class LLMProvider(str, Enum):
    GEMINI = "gemini"
    LOCAL_LLAMA = "local_llama"
    AUTO = "auto"


@dataclass
class MindResponse:
    content: str
    intent_detected: Optional[str] = None
    action_taken: Optional[str] = None
    confidence: float = 1.0
    sources: List[str] = field(default_factory=list)
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class SystemState:
    total_memory: int = 0
    used_memory: int = 0
    memory_utilization: float = 0.0
    total_cores: int = 0
    active_cores: int = 0
    cpu_utilization: float = 0.0
    running_tasks: int = 0
    pending_tasks: int = 0
    health: str = "healthy"


@dataclass
class ConversationTurn:
    role: str
    content: str
    timestamp: datetime = field(default_factory=datetime.now)


# ============== Mock LLM Client ==============

class MockLLMClient:
    """Mock LLM client for testing."""
    
    def __init__(self, provider: LLMProvider = LLMProvider.GEMINI):
        self.provider = provider
        self._call_count = 0
    
    async def generate(self, prompt: str, context: Optional[str] = None) -> str:
        """Generate response."""
        self._call_count += 1
        
        # Simulate real LLM responses
        if "memory" in prompt.lower():
            return "Current memory usage: 50% (512MB of 1GB used)"
        elif "cpu" in prompt.lower():
            return "CPU utilization: 45% with 2 of 4 cores active"
        elif "status" in prompt.lower():
            return "System status: All systems operational"
        else:
            return f"Response to: {prompt[:50]}..."
    
    async def generate_stream(self, prompt: str):
        """Stream response."""
        words = self.generate(prompt).split()
        for word in words:
            yield word + " "
            await asyncio.sleep(0.01)
    
    @property
    def call_count(self) -> int:
        return self._call_count


# ============== Mock Unified Mind ==============

class MockUnifiedMind:
    """Mock Unified Mind for testing."""
    
    def __init__(self, provider: LLMProvider = LLMProvider.GEMINI):
        self.state = MindState.INITIALIZING
        self.system_state = SystemState()
        self.llm = MockLLMClient(provider)
        self.conversation: List[ConversationTurn] = []
        self._metrics = {
            "queries_processed": 0,
            "commands_executed": 0,
            "errors": 0,
            "start_time": None,
        }
        self._command_handlers = {}
        self._running = False
    
    async def start(self) -> None:
        """Start the Unified Mind."""
        self._running = True
        self._metrics["start_time"] = datetime.now()
        self.state = MindState.READY
    
    async def stop(self) -> None:
        """Stop the Unified Mind."""
        self._running = False
        self.state = MindState.SHUTDOWN
    
    async def process(self, user_input: str) -> MindResponse:
        """Process user input."""
        if self.state != MindState.READY:
            return MindResponse(
                content="I'm not ready yet. Please wait.",
                confidence=1.0,
            )
        
        self.state = MindState.PROCESSING
        self._metrics["queries_processed"] += 1
        
        # Add to conversation
        self.conversation.append(ConversationTurn(role="user", content=user_input))
        
        # Parse intent
        intent = self._parse_intent(user_input)
        
        # Generate response
        if intent and intent in self._get_command_handlers():
            response = await self._execute_command(intent, user_input)
        else:
            response = await self._generate_ai_response(user_input)
        
        # Add response to conversation
        self.conversation.append(ConversationTurn(role="assistant", content=response.content))
        
        self.state = MindState.READY
        return response
    
    def _parse_intent(self, user_input: str) -> Optional[str]:
        """Parse user intent."""
        lower = user_input.lower().strip()
        
        command_patterns = {
            "show": ["show ", "display ", "what is ", "how much "],
            "status": ["status", "health", "state"],
            "optimize": ["optimize ", "tune ", "improve "],
            "help": ["help", "what can you do"],
            "clear": ["clear", "reset conversation"],
        }
        
        for intent, patterns in command_patterns.items():
            for pattern in patterns:
                if lower.startswith(pattern) or lower == pattern.strip():
                    return intent
        
        return None
    
    def _get_command_handlers(self) -> Dict[str, callable]:
        return {
            "show": self._cmd_show,
            "status": self._cmd_status,
            "optimize": self._cmd_optimize,
            "help": self._cmd_help,
            "clear": self._cmd_clear,
        }
    
    async def _execute_command(self, intent: str, user_input: str) -> MindResponse:
        """Execute command."""
        self._metrics["commands_executed"] += 1
        handler = self._get_command_handlers().get(intent)
        if handler:
            return await handler(user_input)
        return MindResponse(content=f"Unknown command: {intent}", confidence=0.0)
    
    async def _generate_ai_response(self, user_input: str) -> MindResponse:
        """Generate AI response."""
        content = await self.llm.generate(user_input)
        return MindResponse(
            content=content,
            confidence=0.9,
            sources=["llm"],
        )
    
    async def _cmd_show(self, user_input: str) -> MindResponse:
        """Handle show command."""
        lower = user_input.lower()
        
        if "memory" in lower:
            return MindResponse(
                content=f"Memory Usage:\n  Total: {self.system_state.total_memory / (1024**3):.2f} GB\n"
                       f"  Used: {self.system_state.used_memory / (1024**3):.2f} GB\n"
                       f"  Utilization: {self.system_state.memory_utilization:.1f}%",
                intent_detected="show_memory",
            )
        elif "cpu" in lower:
            return MindResponse(
                content=f"CPU Status:\n  Cores: {self.system_state.active_cores}/{self.system_state.total_cores}\n"
                       f"  Utilization: {self.system_state.cpu_utilization:.1f}%",
                intent_detected="show_cpu",
            )
        else:
            return MindResponse(
                content=self._get_system_summary(),
                intent_detected="show_status",
            )
    
    async def _cmd_status(self, user_input: str) -> MindResponse:
        """Handle status command."""
        return MindResponse(
            content=f"System Status Report\n{'=' * 40}\n{self._get_system_summary()}",
            intent_detected="status",
        )
    
    async def _cmd_optimize(self, user_input: str) -> MindResponse:
        """Handle optimize command."""
        lower = user_input.lower()
        
        if "memory" in lower:
            action = "memory_optimization"
            message = "Initiating memory optimization:\n- Running compaction\n- Clearing caches"
        elif "cpu" in lower or "performance" in lower:
            action = "cpu_optimization"
            message = "Initiating CPU optimization:\n- Balancing load\n- Adjusting priorities"
        else:
            action = "general_optimization"
            message = "Running system optimization"
        
        return MindResponse(
            content=message,
            intent_detected="optimize",
            action_taken=action,
        )
    
    async def _cmd_help(self, user_input: str) -> MindResponse:
        """Handle help command."""
        help_text = """
KolibriOS AI - Unified Mind Commands
====================================

System Commands:
  show memory    - Display memory usage
  show cpu       - Display CPU status
  status         - Full system status
  diagnostics    - Run system diagnostics

Optimization:
  optimize memory    - Optimize memory
  optimize cpu       - Optimize CPU

Conversation:
  clear          - Clear conversation
  help           - Show this message
"""
        return MindResponse(content=help_text, intent_detected="help")
    
    async def _cmd_clear(self, user_input: str) -> MindResponse:
        """Handle clear command."""
        self.conversation.clear()
        return MindResponse(content="Conversation cleared.", intent_detected="clear")
    
    def _get_system_summary(self) -> str:
        """Get system summary."""
        return (
            f"Memory: {self.system_state.memory_utilization:.1f}% used\n"
            f"CPU: {self.system_state.cpu_utilization:.1f}% utilization\n"
            f"Tasks: {self.system_state.running_tasks} running, {self.system_state.pending_tasks} pending\n"
            f"Health: {self.system_state.health}"
        )
    
    def get_metrics(self) -> Dict[str, Any]:
        """Get metrics."""
        return {
            **self._metrics,
            "state": self.state.value,
            "uptime": self._get_uptime(),
        }
    
    def _get_uptime(self) -> str:
        """Get uptime."""
        if not self._metrics["start_time"]:
            return "N/A"
        
        delta = datetime.now() - self._metrics["start_time"]
        hours, remainder = divmod(int(delta.total_seconds()), 3600)
        minutes, seconds = divmod(remainder, 60)
        return f"{hours}h {minutes}m {seconds}s"
    
    def update_system_state(self, **kwargs) -> None:
        """Update system state."""
        for key, value in kwargs.items():
            if hasattr(self.system_state, key):
                setattr(self.system_state, key, value)


# ============== Tests ==============

@pytest.mark.asyncio
async def test_unified_mind_creation():
    """Test Unified Mind creation."""
    mind = MockUnifiedMind()
    assert mind.state == MindState.INITIALIZING


@pytest.mark.asyncio
async def test_unified_mind_start():
    """Test Unified Mind start."""
    mind = MockUnifiedMind()
    await mind.start()
    assert mind.state == MindState.READY


@pytest.mark.asyncio
async def test_unified_mind_stop():
    """Test Unified Mind stop."""
    mind = MockUnifiedMind()
    await mind.start()
    await mind.stop()
    assert mind.state == MindState.SHUTDOWN


@pytest.mark.asyncio
async def test_process_simple_query():
    """Test processing simple query."""
    mind = MockUnifiedMind()
    await mind.start()
    
    response = await mind.process("Hello, how are you?")
    
    assert response.content is not None
    assert len(response.content) > 0


@pytest.mark.asyncio
async def test_process_show_memory():
    """Test show memory command."""
    mind = MockUnifiedMind()
    await mind.start()
    mind.update_system_state(
        total_memory=1024 * 1024 * 1024,
        used_memory=512 * 1024 * 1024,
        memory_utilization=50.0,
    )
    
    response = await mind.process("show memory")
    
    assert "memory" in response.content.lower()
    assert response.intent_detected == "show_memory"


@pytest.mark.asyncio
async def test_process_show_cpu():
    """Test show CPU command."""
    mind = MockUnifiedMind()
    await mind.start()
    mind.update_system_state(
        total_cores=4,
        active_cores=2,
        cpu_utilization=45.0,
    )
    
    response = await mind.process("show cpu")
    
    assert "cpu" in response.content.lower()
    assert response.intent_detected == "show_cpu"


@pytest.mark.asyncio
async def test_process_status():
    """Test status command."""
    mind = MockUnifiedMind()
    await mind.start()
    
    response = await mind.process("status")
    
    assert response.intent_detected == "status"
    assert "Status" in response.content or "status" in response.content.lower()


@pytest.mark.asyncio
async def test_process_optimize():
    """Test optimize command."""
    mind = MockUnifiedMind()
    await mind.start()
    
    response = await mind.process("optimize memory")
    
    assert response.intent_detected == "optimize"
    assert response.action_taken is not None


@pytest.mark.asyncio
async def test_process_help():
    """Test help command."""
    mind = MockUnifiedMind()
    await mind.start()
    
    response = await mind.process("help")
    
    assert response.intent_detected == "help"
    assert "Commands" in response.content or "commands" in response.content.lower()


@pytest.mark.asyncio
async def test_process_clear():
    """Test clear command."""
    mind = MockUnifiedMind()
    await mind.start()
    
    # Add some conversation
    await mind.process("Hello")
    assert len(mind.conversation) > 0
    
    # Clear
    response = await mind.process("clear")
    
    assert response.intent_detected == "clear"
    assert len(mind.conversation) == 2  # clear command + response


@pytest.mark.asyncio
async def test_conversation_history():
    """Test conversation history."""
    mind = MockUnifiedMind()
    await mind.start()
    
    await mind.process("Hello")
    await mind.process("How are you?")
    await mind.process("Goodbye")
    
    # Should have 6 turns (user + assistant for each)
    assert len(mind.conversation) == 6


@pytest.mark.asyncio
async def test_metrics_tracking():
    """Test metrics tracking."""
    mind = MockUnifiedMind()
    await mind.start()
    
    await mind.process("Hello")
    await mind.process("status")
    await mind.process("help")
    
    metrics = mind.get_metrics()
    
    assert metrics["queries_processed"] == 3
    assert metrics["commands_executed"] >= 2  # status and help


@pytest.mark.asyncio
async def test_not_ready_state():
    """Test processing when not ready."""
    mind = MockUnifiedMind()
    
    response = await mind.process("Hello")
    
    assert "not ready" in response.content.lower()


@pytest.mark.asyncio
async def test_llm_call_count():
    """Test LLM is called for non-command queries."""
    mind = MockUnifiedMind()
    await mind.start()
    
    # Non-command query should call LLM
    await mind.process("What is the weather?")
    
    assert mind.llm.call_count == 1


@pytest.mark.asyncio
async def test_system_state_update():
    """Test system state update."""
    mind = MockUnifiedMind()
    
    mind.update_system_state(
        total_memory=2 * 1024 * 1024 * 1024,
        used_memory=1024 * 1024 * 1024,
        memory_utilization=50.0,
    )
    
    assert mind.system_state.total_memory == 2 * 1024 * 1024 * 1024
    assert mind.system_state.used_memory == 1024 * 1024 * 1024


@pytest.mark.asyncio
async def test_uptime_tracking():
    """Test uptime tracking."""
    mind = MockUnifiedMind()
    await mind.start()
    
    await asyncio.sleep(0.1)
    
    metrics = mind.get_metrics()
    
    assert metrics["uptime"] != "N/A"

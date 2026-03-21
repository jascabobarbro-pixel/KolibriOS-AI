# Unified Mind - KolibriOS AI Agent

The **Unified Mind** is the central AI intelligence system for KolibriOS AI. It serves as the primary interface between users and the operating system, providing natural language interaction, system monitoring, and intelligent automation.

## Features

### 🧠 Core Capabilities
- **Natural Language Understanding**: Process and understand user queries in natural language
- **Context Management**: Maintain conversation history and system state context
- **Multi-LLM Support**: Works with Gemini API, local Llama models, or hybrid setups
- **Intelligent Commands**: Execute system commands through natural language

### 🔗 Integration
- **CND Orchestrator**: Communicate with the Central Neural Device
- **Living Cells**: Interact with Memory, Processor, I/O, Network, and AI cells
- **Living Kernel**: Access kernel genes and neural scheduler

### 🖥️ Interfaces
- **CLI**: Rich command-line interface with history and completion
- **Web API**: REST API for web-based interaction
- **Voice**: (Planned) Voice input/output support

## Installation

```bash
# Clone the repository
git clone https://github.com/jascabobarbro-pixel/KolibriOS-AI.git
cd KolibriOS-AI/unified_ai_agent/unified_mind

# Install dependencies
pip install -e ".[all]"

# Or for minimal installation
pip install -e .
```

## Quick Start

### CLI Mode

```bash
# Start interactive CLI
python -m unified_mind

# Or use the script
unified-mind
```

### Single Query

```bash
python -m unified_mind -q "show memory usage"
```

### Server Mode

```bash
python -m unified_mind --server --port 8080
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GEMINI_API_KEY` | Gemini API key | None |
| `LLAMA_MODEL_PATH` | Path to local Llama model | None |
| `LLM_PROVIDER` | LLM provider (gemini, local_llama, auto) | auto |
| `LLM_MODEL` | Model name | gemini-1.5-flash |
| `LLM_TEMPERATURE` | Generation temperature | 0.7 |
| `LLM_MAX_TOKENS` | Maximum tokens | 4096 |
| `MIND_DEBUG` | Enable debug mode | false |
| `MIND_LOG_LEVEL` | Log level | INFO |
| `CND_ENDPOINT` | CND gRPC endpoint | localhost:50051 |
| `KERNEL_ENDPOINT` | Kernel gRPC endpoint | localhost:50052 |

### Configuration File

Create a `config.json` file:

```json
{
  "name": "Kolibri",
  "llm": {
    "provider": "auto",
    "model": "gemini-1.5-flash",
    "temperature": 0.7,
    "max_tokens": 4096
  },
  "interface": {
    "enable_cli": true,
    "prompt_string": "kolibri> "
  }
}
```

Load with: `python -m unified_mind --config config.json`

## Usage Examples

### System Queries

```
kolibri> show memory usage
Memory Usage:
  Total: 16.00 GB
  Used: 8.42 GB
  Utilization: 52.6%

kolibri> show cpu status
CPU Status:
  Total Cores: 8
  Active Cores: 6
  Utilization: 45.2%
```

### System Optimization

```
kolibri> optimize for gaming
Enabling Gaming Mode:
- Prioritizing graphics processes
- Disabling background services
- Optimizing memory for games

kolibri> optimize memory
Initiating memory optimization:
- Running compaction
- Clearing caches
- Defragmenting memory pools
```

### Natural Language Queries

```
kolibri> How is the system performing?
System Health: HEALTHY
Memory: 8.4GB / 16.0GB (52.6%)
CPU: 6/8 cores (45.2%)
Tasks: 24 running, 3 pending

kolibri> Can you help me free up some memory?
I can help free up memory. Here are some options:
1. Clear system caches
2. Stop non-essential processes
3. Compress memory pools

Would you like me to proceed with any of these?
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Unified Mind                             │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌────────────┐  ┌────────────────────────┐  │
│  │   LLM   │  │  Context   │  │     Communication     │  │
│  │ Layer   │  │  Manager   │  │       Layer           │  │
│  └────┬────┘  └─────┬──────┘  └───────────┬───────────┘  │
│       │             │                      │                │
│       └─────────────┴──────────────────────┘                │
│                          │                                   │
│                          ▼                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                   Interface Layer                     │   │
│  │        CLI          │        Web        │    Voice    │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    KolibriOS AI System                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │   CND    │  │  Cells   │  │  Kernel  │  │   Koli   │   │
│  │Orchestr. │  │ (Memory, │  │  Genes,  │  │ Language │   │
│  │          │  │CPU,etc.) │  │Scheduler │  │          │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## API Reference

### UnifiedMind Class

```python
from unified_mind import UnifiedMind, UnifiedMindConfig

config = UnifiedMindConfig(
    name="Kolibri",
    llm=LLMConfig(provider="gemini", api_key="your-key"),
)

mind = UnifiedMind(config)
await mind.start()

response = await mind.process("show memory usage")
print(response.content)

await mind.stop()
```

### Context Manager

```python
from unified_mind.context import ContextManager, ContextType

ctx = ContextManager()

# Add context
ctx.add_entry(
    content="User prefers dark mode",
    context_type=ContextType.USER,
    relevance=0.9,
)

# Get relevant context
relevant = ctx.get_relevant_context("theme preferences")
```

### LLM Clients

```python
from unified_mind.llm import GeminiClient, LlamaClient

# Gemini API
gemini = GeminiClient(api_key="your-key", model="gemini-1.5-flash")
response = await gemini.generate("Hello, KolibriOS!")

# Local Llama
llama = LlamaClient(model_path="/path/to/model.gguf")
response = await llama.generate("Hello, KolibriOS!")
```

## Development

### Running Tests

```bash
pytest tests/ -v --cov=unified_mind
```

### Code Style

```bash
# Format code
black unified_mind/

# Sort imports
isort unified_mind/

# Type checking
mypy unified_mind/
```

## License

MIT License - See LICENSE file for details.

## Contributing

See CONTRIBUTING.md for guidelines on contributing to this project.

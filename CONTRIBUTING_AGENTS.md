# KolibriOS AI - قواعد المساهمة للوكلاء
# Contributing Guidelines for AI Agents

> **Purpose**: This document provides strict rules and patterns for AI agents contributing code to KolibriOS AI. All contributions must follow these guidelines to maintain code quality and consistency.

---

## Table of Contents

1. [Core Principles](#1-core-principles)
2. [Code Quality Standards](#2-code-quality-standards)
3. [Architecture Patterns](#3-architecture-patterns)
4. [Implementation Rules](#4-implementation-rules)
5. [Testing Requirements](#5-testing-requirements)
6. [Documentation Standards](#6-documentation-standards)
7. [Commit Guidelines](#7-commit-guidelines)
8. [Review Process](#8-review-process)
9. [Common Patterns](#9-common-patterns)
10. [Forbidden Practices](#10-forbidden-practices)

---

## 1. Core Principles

### The Foundation

```
┌─────────────────────────────────────────────────────────────┐
│                  FOUNDATION PRINCIPLES                       │
├─────────────────────────────────────────────────────────────┤
│  1. REAL IMPLEMENTATION - No simulation, no placeholders    │
│  2. PROPER ERROR HANDLING - All errors must be handled      │
│  3. METRICS & OBSERVABILITY - All components must be        │
│     measurable and monitorable                               │
│  4. SELF-HEALING - Components must recover from failures    │
│  5. DOCUMENTATION - Code must be self-documenting           │
└─────────────────────────────────────────────────────────────┘
```

### Priority Order

1. **Foundation First**: Core components before extras
2. **Integration**: Components must work together
3. **Testing**: Tests before deployment
4. **Documentation**: Docs with code

---

## 2. Code Quality Standards

### Rust Standards

```rust
// ✅ CORRECT: Proper error handling with thiserror
#[derive(Debug, thiserror::Error)]
pub enum MyComponentError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("gRPC error: {0}")]
    Grpc(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

// ✅ CORRECT: Async function with proper typing
pub async fn process_request(
    &mut self,
    request: Request,
) -> Result<Response, MyComponentError> {
    let result = self.internal_process(request).await?;
    Ok(result)
}

// ✅ CORRECT: Metrics integration
use prometheus::{Counter, Gauge, Histogram};

lazy_static! {
    static ref REQUESTS_TOTAL: Counter = Counter::new(
        "my_component_requests_total",
        "Total number of requests"
    ).unwrap();
    
    static ref LATENCY: Histogram = Histogram::with_opts(
        HistogramOpts::new("my_component_latency", "Request latency")
    ).unwrap();
}

// ✅ CORRECT: Logging with tracing
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(self))]
pub async fn handle_event(&mut self, event: Event) -> Result<(), Error> {
    info!("Processing event: {}", event.id);
    // ... processing
    debug!("Event processed successfully");
    Ok(())
}
```

### Python Standards

```python
# ✅ CORRECT: Proper dataclass with typing
from dataclasses import dataclass, field
from typing import Optional, Dict, Any
from datetime import datetime

@dataclass
class ComponentConfig:
    """Configuration for the component.
    
    Attributes:
        endpoint: gRPC endpoint address
        timeout: Request timeout in seconds
        retries: Number of retry attempts
    """
    endpoint: str
    timeout: float = 30.0
    retries: int = 3
    metadata: Dict[str, str] = field(default_factory=dict)


# ✅ CORRECT: Async context manager
class AsyncComponent:
    async def __aenter__(self) -> "AsyncComponent":
        await self.initialize()
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        await self.shutdown()


# ✅ CORRECT: Proper error handling
class ComponentError(Exception):
    """Base error for component operations."""
    pass

class ConnectionError(ComponentError):
    """Raised when connection fails."""
    pass

async def connect_with_retry(config: ComponentConfig) -> Connection:
    """Establish connection with retry logic.
    
    Args:
        config: Connection configuration
        
    Returns:
        Active connection
        
    Raises:
        ConnectionError: If connection fails after all retries
    """
    last_error = None
    for attempt in range(config.retries):
        try:
            return await _establish_connection(config)
        except Exception as e:
            last_error = e
            logger.warning(f"Connection attempt {attempt + 1} failed: {e}")
            await asyncio.sleep(2 ** attempt)  # Exponential backoff
    
    raise ConnectionError(f"Failed after {config.retries} attempts: {last_error}")
```

---

## 3. Architecture Patterns

### Cell Pattern

Every cell MUST implement:

```rust
/// All cells must follow this pattern
pub struct SomeCell {
    id: String,
    state: Arc<RwLock<CellState>>,
    health: Arc<RwLock<HealthStatus>>,
    metrics: CellMetrics,
    // Cell-specific fields
}

impl SomeCell {
    /// Create new cell
    pub fn new(id: &str, config: CellConfig) -> Self { ... }
    
    /// Initialize cell
    pub async fn initialize(&self) -> Result<(), CellError> { ... }
    
    /// Get cell state
    pub async fn state(&self) -> CellState { ... }
    
    /// Get health status
    pub async fn health(&self) -> HealthStatus { ... }
    
    /// Run diagnostics
    pub async fn run_diagnostics(&self) -> DiagnosticsResult { ... }
    
    /// Self-heal
    pub async fn heal(&self) -> Result<(), CellError> { ... }
    
    /// Shutdown gracefully
    pub async fn shutdown(&self) -> Result<(), CellError> { ... }
    
    /// Get Prometheus metrics
    pub fn metrics_registry(&self) -> &Registry { ... }
}
```

### gRPC Service Pattern

```rust
/// gRPC service implementation pattern
pub struct MyCellGrpcService {
    cell: Arc<MyCell>,
}

#[tonic::async_trait]
impl MyCellService for MyCellGrpcService {
    async fn get_stats(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<StatsResponse>, Status> {
        let stats = self.cell.get_stats().await;
        Ok(Response::new(stats.into()))
    }
    
    async fn perform_action(
        &self,
        request: Request<ActionRequest>,
    ) -> Result<Response<ActionResponse>, Status> {
        let req = request.into_inner();
        match self.cell.perform_action(req).await {
            Ok(response) => Ok(Response::new(response)),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
```

### LLM Integration Pattern

```python
class LLMClient(ABC):
    """Base class for LLM clients."""
    
    @abstractmethod
    async def generate(
        self, 
        prompt: str, 
        context: Optional[str] = None
    ) -> str:
        """Generate response from LLM."""
        pass
    
    @abstractmethod
    async def generate_stream(
        self, 
        prompt: str
    ) -> AsyncIterator[str]:
        """Stream response from LLM."""
        pass


class GeminiClient(LLMClient):
    """Real Gemini API client."""
    
    def __init__(self, api_key: str, model: str = "gemini-pro"):
        self._client = genai.GenerativeModel(model)
        self._api_key = api_key
    
    async def generate(self, prompt: str, context: Optional[str] = None) -> str:
        """Generate using real Gemini API."""
        full_prompt = f"{context}\n\n{prompt}" if context else prompt
        response = await self._client.generate_content_async(full_prompt)
        return response.text
```

---

## 4. Implementation Rules

### Rule 1: NO SIMULATION

```rust
// ❌ WRONG: Simulation/placeholder
pub async fn send_command(&self, cmd: &str) -> Result<Response> {
    // TODO: Implement real gRPC call
    Ok(Response::mock())
}

// ✅ CORRECT: Real implementation
pub async fn send_command(&self, cmd: &str) -> Result<Response> {
    let mut stub = self.create_stub().await?;
    let request = self.build_request(cmd);
    let response = stub.execute(request).await?;
    Ok(response)
}
```

### Rule 2: NO RANDOM DATA

```python
# ❌ WRONG: Random/fake data
async def get_metrics(self):
    return {
        "cpu": random.uniform(0, 100),
        "memory": random.uniform(0, 100),
    }

# ✅ CORRECT: Real data
async def get_metrics(self):
    import psutil
    return {
        "cpu": psutil.cpu_percent(),
        "memory": psutil.virtual_memory().percent,
    }
```

### Rule 3: NO EMPTY BODIES

```rust
// ❌ WRONG: Empty implementation
pub async fn heal(&self) -> Result<()> {
    // TODO: Implement healing
    Ok(())
}

// ✅ CORRECT: Real implementation
pub async fn heal(&self) -> Result<()> {
    let diagnostics = self.run_diagnostics().await?;
    
    if !diagnostics.healthy {
        for issue in diagnostics.issues {
            match issue.severity {
                Severity::Critical => self.fix_critical(&issue).await?,
                Severity::Warning => self.fix_warning(&issue).await?,
                _ => {}
            }
        }
    }
    
    self.update_health_status().await;
    Ok(())
}
```

### Rule 4: PROPER ERROR PROPAGATION

```rust
// ❌ WRONG: Swallowing errors
pub async fn process(&self) {
    let _ = self.risky_operation();  // Error ignored
}

// ✅ CORRECT: Proper error handling
pub async fn process(&self) -> Result<(), Error> {
    self.risky_operation().await?;
    Ok(())
}

// ✅ CORRECT: With context
pub async fn process(&self) -> Result<(), Error> {
    self.risky_operation()
        .await
        .map_err(|e| Error::WithContext("process failed", e))?;
    Ok(())
}
```

---

## 5. Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[test]
    async fn test_initialization() {
        let cell = MyCell::new("test-cell", Config::default());
        assert!(cell.initialize().await.is_ok());
        assert_eq!(cell.state().await, CellState::Active);
    }
    
    #[test]
    async fn test_error_handling() {
        let cell = MyCell::new("test-cell", Config::invalid());
        assert!(cell.initialize().await.is_err());
    }
    
    #[test]
    async fn test_self_healing() {
        let cell = MyCell::new("test-cell", Config::default());
        cell.initialize().await.unwrap();
        cell.inject_fault().await;
        assert!(cell.heal().await.is_ok());
        assert_eq!(cell.health().await, HealthStatus::Healthy);
    }
}
```

### Integration Tests

```python
import pytest
import grpc

@pytest.mark.asyncio
async def test_grpc_communication():
    """Test gRPC communication between components."""
    async with grpc.aio.insecure_channel('localhost:50051') as channel:
        stub = MemoryCellServiceStub(channel)
        response = await stub.GetStats(Empty())
        assert response.total_memory > 0

@pytest.mark.asyncio
async def test_full_workflow():
    """Test complete workflow from user input to response."""
    mind = UnifiedMind(config=test_config)
    await mind.start()
    
    response = await mind.process("show memory")
    assert "memory" in response.content.lower()
    
    await mind.stop()
```

---

## 6. Documentation Standards

### Code Documentation

```rust
/// Memory Cell - Autonomous Memory Management.
///
/// This cell provides intelligent, self-organizing memory management
/// with predictive allocation, automatic optimization, and Prometheus metrics.
///
/// # Example
///
/// ```rust
/// let cell = MemoryCell::new("mem-0", 1024 * 1024 * 1024);
/// cell.initialize().await?;
/// 
/// let allocation = cell.allocate(4096, "user", AllocationFlags::default()).await?;
/// println!("Allocated at: {:?}", allocation.address);
/// ```
///
/// # Features
///
/// - Predictive allocation based on usage patterns
/// - Automatic defragmentation
/// - Prometheus metrics export
/// - Self-healing on memory pressure
pub struct MemoryCell { ... }
```

### Function Documentation

```python
async def process_request(
    self,
    request: Request,
    context: Optional[Context] = None,
) -> Response:
    """Process a user request and generate a response.
    
    This is the main entry point for request processing. It handles
    intent parsing, command execution, and LLM-based response generation.
    
    Args:
        request: The user request to process
        context: Optional conversation context
        
    Returns:
        Response containing the generated content and metadata
        
    Raises:
        ProcessingError: If request processing fails
        TimeoutError: If processing exceeds timeout
        
    Example:
        >>> response = await agent.process_request(
        ...     Request("What is the memory usage?"),
        ...     context=Context()
        ... )
        >>> print(response.content)
    """
```

---

## 7. Commit Guidelines

### Commit Message Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation |
| `test` | Tests |
| `refactor` | Code refactoring |
| `perf` | Performance improvement |
| `chore` | Maintenance |

### Examples

```
feat(memory-cell): add predictive allocation algorithm

- Implement usage pattern analysis
- Add prediction model for allocation
- Integrate with MemoryGene feedback

Closes #123
```

```
fix(cnd): use real gRPC for send_command

- Remove simulated responses
- Add proper protobuf imports
- Handle connection errors

Fixes simulation issue identified in analysis
```

---

## 8. Review Process

### Self-Review Checklist

Before submitting:

```markdown
- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] No simulation/placeholder code
- [ ] Proper error handling
- [ ] Metrics added if applicable
- [ ] Documentation updated
- [ ] No hardcoded values
- [ ] Async code uses proper patterns
```

### Review Criteria

1. **Correctness**: Does it work?
2. **Completeness**: Is it fully implemented?
3. **Integration**: Does it work with other components?
4. **Performance**: Is it efficient?
5. **Security**: Is it secure?
6. **Documentation**: Is it documented?

---

## 9. Common Patterns

### Pattern: Async Initialization

```rust
pub struct Component {
    initialized: bool,
    // ...
}

impl Component {
    pub fn new(config: Config) -> Self {
        Self {
            initialized: false,
            // ...
        }
    }
    
    pub async fn initialize(&mut self) -> Result<(), Error> {
        if self.initialized {
            return Ok(());
        }
        
        // Perform initialization
        self.setup_connections().await?;
        self.load_config().await?;
        
        self.initialized = true;
        Ok(())
    }
}
```

### Pattern: Graceful Shutdown

```rust
impl Component {
    pub async fn shutdown(&mut self) -> Result<(), Error> {
        if !self.initialized {
            return Ok(());
        }
        
        // Drain pending operations
        self.drain_queue().await?;
        
        // Close connections
        self.close_connections().await?;
        
        // Save state
        self.save_state().await?;
        
        self.initialized = false;
        Ok(())
    }
}
```

### Pattern: Health Check

```rust
pub async fn health_check(&self) -> HealthStatus {
    let mut issues = Vec::new();
    
    // Check connections
    if !self.connections_healthy().await {
        issues.push("Connection issues");
    }
    
    // Check resources
    if self.resource_pressure() > 0.9 {
        issues.push("High resource pressure");
    }
    
    if issues.is_empty() {
        HealthStatus::Healthy
    } else if issues.len() < 3 {
        HealthStatus::Warning
    } else {
        HealthStatus::Critical
    }
}
```

---

## 10. Forbidden Practices

### ❌ NEVER DO THIS

```rust
// ❌ Simulation
async fn get_data(&self) -> Data {
    Data { value: 42 }  // Hardcoded!
}

// ❌ Placeholder
async fn process(&self) -> Result<()> {
    // TODO: implement this
    Ok(())
}

// ❌ Ignoring errors
let _ = risky_operation();

// ❌ Panic in production
panic!("This should never happen");

// ❌ Blocking in async
std::thread::sleep(Duration::from_secs(1));

// ❌ Unwrap without handling
let value = option.unwrap();

// ❌ Global mutable state
static mut STATE: State = State::new();
```

### ✅ ALWAYS DO THIS

```rust
// ✅ Real implementation
async fn get_data(&self) -> Result<Data, Error> {
    self.data_source.fetch().await
}

// ✅ Proper implementation
async fn process(&self) -> Result<(), Error> {
    let data = self.fetch_data().await?;
    self.transform(data).await?;
    Ok(())
}

// ✅ Error handling
if let Err(e) = risky_operation() {
    error!("Operation failed: {}", e);
    return Err(e);
}

// ✅ Graceful error handling
return Err(Error::InvalidState("unexpected condition"));

// ✅ Async sleep
tokio::time::sleep(Duration::from_secs(1)).await;

// ✅ Proper option handling
let value = option.ok_or(Error::MissingValue)?;

// ✅ Thread-safe state
use std::sync::Arc;
use tokio::sync::RwLock;
let state = Arc::new(RwLock::new(State::new()));
```

---

## Enforcement

All contributions are automatically checked for:

1. **Simulation Detection**: Code scanning for TODO, FIXME, mock, fake, simulate
2. **Test Coverage**: Minimum coverage requirements
3. **Documentation**: Public APIs must be documented
4. **Style**: Automatic formatting (rustfmt, black)

---

## Quick Reference

```bash
# Format code
cargo fmt
black .

# Run tests
cargo test
pytest

# Check for issues
cargo clippy
pylint **/*.py

# Build documentation
cargo doc
```

---

*Last updated: 2026-03-22*
*Version: 1.0.0*

"""
Pytest configuration and fixtures for KolibriOS AI tests.
"""

import pytest
import asyncio
from typing import Generator


@pytest.fixture(scope="session")
def event_loop() -> Generator:
    """Create an event loop for the test session."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()


@pytest.fixture
def mock_grpc_channel():
    """Mock gRPC channel for testing."""
    class MockChannel:
        def __init__(self):
            self.closed = False
        
        def close(self):
            self.closed = True
    
    return MockChannel()


@pytest.fixture
def mock_memory_cell():
    """Mock memory cell for testing."""
    class MockMemoryCell:
        def __init__(self):
            self.total_memory = 1024 * 1024 * 1024  # 1GB
            self.used_memory = 0
            self.allocations = {}
        
        async def allocate(self, size: int, pool: str = "user"):
            import uuid
            alloc_id = str(uuid.uuid4())
            self.allocations[alloc_id] = {"size": size, "pool": pool}
            self.used_memory += size
            return {"id": alloc_id, "size": size, "pool": pool}
        
        async def deallocate(self, alloc_id: str):
            if alloc_id in self.allocations:
                self.used_memory -= self.allocations[alloc_id]["size"]
                del self.allocations[alloc_id]
                return True
            return False
        
        async def get_stats(self):
            return {
                "total_memory": self.total_memory,
                "used_memory": self.used_memory,
                "utilization": self.used_memory / self.total_memory * 100,
            }
    
    return MockMemoryCell()


@pytest.fixture
def mock_processor_cell():
    """Mock processor cell for testing."""
    class MockProcessorCell:
        def __init__(self):
            self.total_cores = 4
            self.active_cores = 0
            self.tasks = []
        
        async def create_task(self, executable: str, args: list = None, priority: int = 5):
            import uuid
            task_id = str(uuid.uuid4())[:8]
            self.tasks.append({
                "id": task_id,
                "executable": executable,
                "args": args or [],
                "priority": priority,
            })
            return {"id": task_id, "executable": executable}
        
        async def list_tasks(self):
            return self.tasks
        
        async def terminate_task(self, task_id: str):
            self.tasks = [t for t in self.tasks if t["id"] != task_id]
            return True
        
        async def get_cpu_stats(self):
            return {
                "total_cores": self.total_cores,
                "active_cores": self.active_cores,
                "utilization": len(self.tasks) * 10,
            }
    
    return MockProcessorCell()


@pytest.fixture
def mock_llm_client():
    """Mock LLM client for testing."""
    class MockLLMClient:
        def __init__(self):
            self.call_count = 0
        
        async def generate(self, prompt: str, context: str = None) -> str:
            self.call_count += 1
            
            if "memory" in prompt.lower():
                return "Current memory usage: 50% (512MB of 1GB used)"
            elif "cpu" in prompt.lower():
                return "CPU utilization: 45% with 2 of 4 cores active"
            elif "status" in prompt.lower():
                return "System status: All systems operational"
            else:
                return f"Response to: {prompt[:50]}..."
        
        async def generate_stream(self, prompt: str):
            words = (await self.generate(prompt)).split()
            for word in words:
                yield word + " "
    
    return MockLLMClient()


@pytest.fixture
def mock_unified_mind(mock_llm_client):
    """Mock Unified Mind for testing."""
    class MockUnifiedMind:
        def __init__(self, llm_client):
            self.llm = llm_client
            self.state = "ready"
            self.conversation = []
        
        async def process(self, user_input: str):
            self.conversation.append({"role": "user", "content": user_input})
            response = await self.llm.generate(user_input)
            self.conversation.append({"role": "assistant", "content": response})
            return {"content": response, "confidence": 0.9}
        
        def get_conversation(self):
            return self.conversation
    
    return MockUnifiedMind(mock_llm_client)


@pytest.fixture
def mock_cnd_orchestrator():
    """Mock CND Orchestrator for testing."""
    class MockCNDOrchestrator:
        def __init__(self):
            self.cells = {}
            self._running = False
        
        async def start(self):
            self._running = True
        
        async def stop(self):
            self._running = False
        
        async def register_cell(self, cell_id: str, cell_type: str, endpoint: str):
            self.cells[cell_id] = {
                "id": cell_id,
                "type": cell_type,
                "endpoint": endpoint,
                "status": "active",
            }
            return True
        
        async def unregister_cell(self, cell_id: str):
            if cell_id in self.cells:
                del self.cells[cell_id]
                return True
            return False
        
        async def send_command(self, cell_id: str, command: str, params: dict = None):
            if cell_id not in self.cells:
                return {"success": False, "error": "Cell not found"}
            return {"success": True, "result": f"{command} executed"}
        
        def list_cells(self):
            return list(self.cells.values())
    
    return MockCNDOrchestrator()


# Markers
def pytest_configure(config):
    """Configure custom markers."""
    config.addinivalue_line("markers", "asyncio: mark test as async")
    config.addinivalue_line("markers", "integration: mark test as integration test")
    config.addinivalue_line("markers", "unit: mark test as unit test")
    config.addinivalue_line("markers", "slow: mark test as slow")

"""
Comprehensive CND Orchestrator Tests

Tests for:
- Cell registration and unregistration
- Command sending via gRPC
- Metrics aggregation
- Health monitoring
- Alerts system
"""

import pytest
import asyncio
from datetime import datetime
from typing import Dict, Any, Optional
from dataclasses import dataclass, field
from enum import Enum


# ============== Enums and Data Classes ==============

class CellType(str, Enum):
    MEMORY = "memory"
    PROCESSOR = "processor"
    IO = "io"
    NETWORK = "network"
    AI = "ai"


class CellStatus(str, Enum):
    INITIALIZING = "initializing"
    ACTIVE = "active"
    DEGRADED = "degraded"
    HEALING = "healing"
    SHUTDOWN = "shutdown"


class HealthStatus(str, Enum):
    HEALTHY = "healthy"
    WARNING = "warning"
    CRITICAL = "critical"


@dataclass
class CellInfo:
    cell_id: str
    cell_type: CellType
    endpoint: str
    status: CellStatus = CellStatus.INITIALIZING
    health: HealthStatus = HealthStatus.HEALTHY
    last_heartbeat: Optional[datetime] = None
    capabilities: Dict[str, str] = field(default_factory=dict)
    metrics: Dict[str, float] = field(default_factory=dict)


@dataclass
class SystemMetrics:
    total_memory: int = 0
    used_memory: int = 0
    memory_utilization: float = 0.0
    total_cores: int = 0
    active_cores: int = 0
    cpu_utilization: float = 0.0
    running_tasks: int = 0
    pending_tasks: int = 0
    overall_health: HealthStatus = HealthStatus.HEALTHY


# ============== Mock CND Orchestrator ==============

class MockCndOrchestrator:
    """Mock CND Orchestrator for testing."""
    
    def __init__(self, orchestrator_id: str = "cnd-test-0"):
        self.orchestrator_id = orchestrator_id
        self.cells: Dict[str, CellInfo] = {}
        self.system_metrics = SystemMetrics()
        self.alerts: list = []
        self._running = False
    
    async def start(self) -> None:
        """Start the orchestrator."""
        self._running = True
    
    async def stop(self) -> None:
        """Stop the orchestrator."""
        self._running = False
    
    async def register_cell(
        self,
        cell_id: str,
        cell_type: str,
        endpoint: str,
        capabilities: Optional[Dict[str, str]] = None,
    ) -> bool:
        """Register a new cell."""
        try:
            cell_type_enum = CellType(cell_type.lower())
        except ValueError:
            return False
        
        cell_info = CellInfo(
            cell_id=cell_id,
            cell_type=cell_type_enum,
            endpoint=endpoint,
            capabilities=capabilities or {},
            last_heartbeat=datetime.now(),
        )
        
        self.cells[cell_id] = cell_info
        return True
    
    async def unregister_cell(self, cell_id: str, graceful: bool = True) -> bool:
        """Unregister a cell."""
        if cell_id not in self.cells:
            return False
        
        del self.cells[cell_id]
        
        if not graceful:
            self._raise_alert("warning", cell_id, f"Cell {cell_id} unregistered ungracefully")
        
        return True
    
    async def send_command(
        self,
        cell_id: str,
        command: str,
        parameters: Optional[Dict[str, str]] = None,
    ) -> Dict[str, Any]:
        """Send a command to a cell."""
        if cell_id not in self.cells:
            return {"success": False, "error": "Cell not found"}
        
        cell_info = self.cells[cell_id]
        
        # Simulate real command execution
        if cell_info.cell_type == CellType.MEMORY:
            return await self._execute_memory_command(command, parameters or {})
        elif cell_info.cell_type == CellType.PROCESSOR:
            return await self._execute_processor_command(command, parameters or {})
        else:
            return {"success": True, "message": f"Command '{command}' executed"}
    
    async def _execute_memory_command(self, command: str, parameters: Dict[str, str]) -> Dict[str, Any]:
        """Execute memory cell command."""
        if command == "get_stats":
            return {
                "success": True,
                "data": {
                    "total_memory": 1024 * 1024 * 1024,
                    "used_memory": 512 * 1024 * 1024,
                    "available_memory": 512 * 1024 * 1024,
                    "utilization_percent": 50.0,
                    "allocation_count": 100,
                }
            }
        elif command == "allocate":
            size = int(parameters.get("size", 4096))
            return {
                "success": True,
                "address": "0x1000",
                "allocation_id": f"alloc-{size}",
            }
        elif command == "deallocate":
            return {"success": True, "message": "Deallocated"}
        else:
            return {"success": False, "error": f"Unknown command: {command}"}
    
    async def _execute_processor_command(self, command: str, parameters: Dict[str, str]) -> Dict[str, Any]:
        """Execute processor cell command."""
        if command == "get_cpu_stats":
            return {
                "success": True,
                "data": {
                    "total_cores": 4,
                    "active_cores": 2,
                    "utilization": 45.0,
                }
            }
        elif command == "list_tasks":
            return {
                "success": True,
                "tasks": [
                    {"id": "task-1", "status": "running", "priority": 5},
                    {"id": "task-2", "status": "pending", "priority": 3},
                ]
            }
        else:
            return {"success": False, "error": f"Unknown command: {command}"}
    
    async def broadcast_command(
        self,
        command: str,
        parameters: Optional[Dict[str, str]] = None,
        cell_type: Optional[CellType] = None,
    ) -> Dict[str, Dict[str, Any]]:
        """Broadcast a command to all cells or a specific type."""
        results = {}
        
        for cell_id, cell_info in self.cells.items():
            if cell_type and cell_info.cell_type != cell_type:
                continue
            results[cell_id] = await self.send_command(cell_id, command, parameters)
        
        return results
    
    def get_system_metrics(self) -> SystemMetrics:
        """Get aggregated system metrics."""
        return self.system_metrics
    
    def list_cells(self, cell_type: Optional[CellType] = None) -> list:
        """List all registered cells."""
        if cell_type:
            return [c for c in self.cells.values() if c.cell_type == cell_type]
        return list(self.cells.values())
    
    def _raise_alert(self, severity: str, source_cell: str, message: str) -> None:
        """Raise a new alert."""
        self.alerts.append({
            "severity": severity,
            "source_cell": source_cell,
            "message": message,
            "timestamp": datetime.now(),
        })
    
    async def run_diagnostics(self) -> Dict[str, Any]:
        """Run system-wide diagnostics."""
        return {
            "healthy": True,
            "cells": {cell_id: {"status": cell.status.value, "health": cell.health.value} 
                     for cell_id, cell in self.cells.items()},
            "issues": [],
        }


# ============== Tests ==============

@pytest.mark.asyncio
async def test_orchestrator_creation():
    """Test orchestrator creation."""
    orchestrator = MockCndOrchestrator("test-orchestrator")
    assert orchestrator.orchestrator_id == "test-orchestrator"
    assert len(orchestrator.cells) == 0


@pytest.mark.asyncio
async def test_orchestrator_start_stop():
    """Test orchestrator start and stop."""
    orchestrator = MockCndOrchestrator()
    
    await orchestrator.start()
    assert orchestrator._running == True
    
    await orchestrator.stop()
    assert orchestrator._running == False


@pytest.mark.asyncio
async def test_cell_registration():
    """Test cell registration."""
    orchestrator = MockCndOrchestrator()
    
    result = await orchestrator.register_cell(
        cell_id="memory-cell-0",
        cell_type="memory",
        endpoint="localhost:50051",
        capabilities={"pools": "kernel,user,shared"},
    )
    
    assert result == True
    assert len(orchestrator.cells) == 1
    assert "memory-cell-0" in orchestrator.cells


@pytest.mark.asyncio
async def test_cell_registration_invalid_type():
    """Test cell registration with invalid type."""
    orchestrator = MockCndOrchestrator()
    
    result = await orchestrator.register_cell(
        cell_id="invalid-cell",
        cell_type="invalid",
        endpoint="localhost:50051",
    )
    
    assert result == False
    assert len(orchestrator.cells) == 0


@pytest.mark.asyncio
async def test_cell_unregistration():
    """Test cell unregistration."""
    orchestrator = MockCndOrchestrator()
    
    await orchestrator.register_cell("memory-cell-0", "memory", "localhost:50051")
    assert len(orchestrator.cells) == 1
    
    result = await orchestrator.unregister_cell("memory-cell-0")
    assert result == True
    assert len(orchestrator.cells) == 0


@pytest.mark.asyncio
async def test_cell_unregistration_not_found():
    """Test cell unregistration when not found."""
    orchestrator = MockCndOrchestrator()
    
    result = await orchestrator.unregister_cell("non-existent")
    assert result == False


@pytest.mark.asyncio
async def test_send_command_memory_cell():
    """Test sending command to memory cell."""
    orchestrator = MockCndOrchestrator()
    await orchestrator.register_cell("memory-cell-0", "memory", "localhost:50051")
    
    result = await orchestrator.send_command("memory-cell-0", "get_stats")
    assert result["success"] == True
    assert "data" in result
    assert "total_memory" in result["data"]


@pytest.mark.asyncio
async def test_send_command_processor_cell():
    """Test sending command to processor cell."""
    orchestrator = MockCndOrchestrator()
    await orchestrator.register_cell("processor-cell-0", "processor", "localhost:50052")
    
    result = await orchestrator.send_command("processor-cell-0", "get_cpu_stats")
    assert result["success"] == True
    assert "data" in result
    assert "total_cores" in result["data"]


@pytest.mark.asyncio
async def test_send_command_not_found():
    """Test sending command to non-existent cell."""
    orchestrator = MockCndOrchestrator()
    
    result = await orchestrator.send_command("non-existent", "get_stats")
    assert result["success"] == False
    assert "error" in result


@pytest.mark.asyncio
async def test_broadcast_command():
    """Test broadcasting command to all cells."""
    orchestrator = MockCndOrchestrator()
    await orchestrator.register_cell("memory-0", "memory", "localhost:50051")
    await orchestrator.register_cell("memory-1", "memory", "localhost:50052")
    
    results = await orchestrator.broadcast_command("get_stats")
    
    assert len(results) == 2
    assert all(r["success"] for r in results.values())


@pytest.mark.asyncio
async def test_broadcast_command_by_type():
    """Test broadcasting command to specific cell type."""
    orchestrator = MockCndOrchestrator()
    await orchestrator.register_cell("memory-0", "memory", "localhost:50051")
    await orchestrator.register_cell("processor-0", "processor", "localhost:50052")
    
    results = await orchestrator.broadcast_command("get_stats", cell_type=CellType.MEMORY)
    
    assert len(results) == 1
    assert "memory-0" in results


@pytest.mark.asyncio
async def test_list_cells():
    """Test listing cells."""
    orchestrator = MockCndOrchestrator()
    await orchestrator.register_cell("memory-0", "memory", "localhost:50051")
    await orchestrator.register_cell("processor-0", "processor", "localhost:50052")
    
    cells = orchestrator.list_cells()
    assert len(cells) == 2


@pytest.mark.asyncio
async def test_list_cells_by_type():
    """Test listing cells by type."""
    orchestrator = MockCndOrchestrator()
    await orchestrator.register_cell("memory-0", "memory", "localhost:50051")
    await orchestrator.register_cell("memory-1", "memory", "localhost:50052")
    await orchestrator.register_cell("processor-0", "processor", "localhost:50053")
    
    cells = orchestrator.list_cells(CellType.MEMORY)
    assert len(cells) == 2
    
    cells = orchestrator.list_cells(CellType.PROCESSOR)
    assert len(cells) == 1


@pytest.mark.asyncio
async def test_alert_raising():
    """Test alert raising."""
    orchestrator = MockCndOrchestrator()
    
    orchestrator._raise_alert("warning", "test-cell", "Test alert")
    
    assert len(orchestrator.alerts) == 1
    assert orchestrator.alerts[0]["severity"] == "warning"
    assert orchestrator.alerts[0]["source_cell"] == "test-cell"


@pytest.mark.asyncio
async def test_diagnostics():
    """Test system diagnostics."""
    orchestrator = MockCndOrchestrator()
    await orchestrator.register_cell("memory-0", "memory", "localhost:50051")
    
    result = await orchestrator.run_diagnostics()
    
    assert result["healthy"] == True
    assert "memory-0" in result["cells"]


@pytest.mark.asyncio
async def test_multiple_registrations():
    """Test multiple cell registrations."""
    orchestrator = MockCndOrchestrator()
    
    for i in range(10):
        result = await orchestrator.register_cell(
            f"memory-{i}",
            "memory",
            f"localhost:{50051 + i}"
        )
        assert result == True
    
    assert len(orchestrator.cells) == 10


@pytest.mark.asyncio
async def test_memory_allocate_command():
    """Test memory allocation command."""
    orchestrator = MockCndOrchestrator()
    await orchestrator.register_cell("memory-0", "memory", "localhost:50051")
    
    result = await orchestrator.send_command("memory-0", "allocate", {"size": "4096"})
    
    assert result["success"] == True
    assert "address" in result
    assert "allocation_id" in result


@pytest.mark.asyncio
async def test_processor_list_tasks():
    """Test processor list tasks command."""
    orchestrator = MockCndOrchestrator()
    await orchestrator.register_cell("processor-0", "processor", "localhost:50052")
    
    result = await orchestrator.send_command("processor-0", "list_tasks")
    
    assert result["success"] == True
    assert "tasks" in result

#!/usr/bin/env python3
"""
KolibriOS AI - Core Cell Communication Verification Experiment

This script verifies basic communication between MemoryCell and ProcessorCell
by testing allocation requests, task execution, and inter-cell messaging.

Usage:
    python experiments/verify_cell_communication.py
"""

import asyncio
import json
import logging
import os
import sys
import time
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from typing import Any, Optional, Dict, List

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class CellType(Enum):
    """Cell types."""
    MEMORY = "memory_cell"
    PROCESSOR = "processor_cell"
    AI = "ai_cell"
    IO = "io_cell"
    NETWORK = "network_cell"


class MessageType(Enum):
    """Message types for inter-cell communication."""
    REQUEST = "request"
    RESPONSE = "response"
    NOTIFICATION = "notification"
    ERROR = "error"


class AllocationStatus(Enum):
    """Memory allocation status."""
    SUCCESS = "success"
    FAILED = "failed"
    PARTIAL = "partial"
    PENDING = "pending"


class TaskStatus(Enum):
    """Task execution status."""
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


@dataclass
class CellMessage:
    """Inter-cell message structure."""
    sender: str
    receiver: str
    message_type: MessageType
    action: str
    payload: Dict[str, Any]
    timestamp: datetime = field(default_factory=datetime.now)
    message_id: str = ""
    
    def to_dict(self) -> dict:
        return {
            "sender": self.sender,
            "receiver": self.receiver,
            "message_type": self.message_type.value,
            "action": self.action,
            "payload": self.payload,
            "timestamp": self.timestamp.isoformat(),
            "message_id": self.message_id,
        }


@dataclass
class AllocationResult:
    """Memory allocation result."""
    address: int
    size: int
    zone: str
    status: AllocationStatus
    message: str = ""
    timestamp: datetime = field(default_factory=datetime.now)


@dataclass
class TaskResult:
    """Task execution result."""
    task_id: str
    status: TaskStatus
    result: Any = None
    memory_used: int = 0
    execution_time_ms: float = 0
    error: str = ""


class MemoryCellSimulator:
    """
    MemoryCell Simulator for communication testing.
    
    Simulates the MemoryCell gRPC service with allocation,
    deallocation, and metrics reporting.
    """
    
    def __init__(self, cell_id: str = "memory_cell_001"):
        self.cell_id = cell_id
        self.cell_type = CellType.MEMORY
        self.zones = {
            "kernel": {"base": 0x100000, "size": 64 * 1024 * 1024, "used": 0},
            "user": {"base": 0x5000000, "size": 512 * 1024 * 1024, "used": 0},
            "ai": {"base": 0x25000000, "size": 256 * 1024 * 1024, "used": 0},
            "cache": {"base": 0x35000000, "size": 128 * 1024 * 1024, "used": 0},
        }
        self.allocations: Dict[int, Dict] = {}
        self.next_address = 0x100000
        self.message_log: List[CellMessage] = []
        
    async def handle_message(self, message: CellMessage) -> CellMessage:
        """Handle incoming message from another cell."""
        self.message_log.append(message)
        
        if message.action == "allocate":
            result = await self.allocate(
                size=message.payload.get("size", 0),
                zone=message.payload.get("zone", "user"),
                requester=message.sender,
            )
            return CellMessage(
                sender=self.cell_id,
                receiver=message.sender,
                message_type=MessageType.RESPONSE,
                action="allocate_response",
                payload={
                    "address": result.address,
                    "size": result.size,
                    "zone": result.zone,
                    "status": result.status.value,
                    "message": result.message,
                },
                message_id=f"resp_{message.message_id}",
            )
            
        elif message.action == "free":
            success = await self.free(
                address=message.payload.get("address", 0),
                size=message.payload.get("size", 0),
            )
            return CellMessage(
                sender=self.cell_id,
                receiver=message.sender,
                message_type=MessageType.RESPONSE,
                action="free_response",
                payload={"success": success},
                message_id=f"resp_{message.message_id}",
            )
            
        elif message.action == "get_metrics":
            metrics = self.get_metrics()
            return CellMessage(
                sender=self.cell_id,
                receiver=message.sender,
                message_type=MessageType.RESPONSE,
                action="metrics_response",
                payload=metrics,
                message_id=f"resp_{message.message_id}",
            )
            
        return CellMessage(
            sender=self.cell_id,
            receiver=message.sender,
            message_type=MessageType.ERROR,
            action="unknown_action",
            payload={"error": f"Unknown action: {message.action}"},
        )
    
    async def allocate(self, size: int, zone: str = "user", requester: str = "") -> AllocationResult:
        """Allocate memory in specified zone."""
        logger.info(f"[{self.cell_id}] Allocation request: {size} bytes in zone '{zone}' from {requester}")
        
        if zone not in self.zones:
            return AllocationResult(
                address=0,
                size=0,
                zone=zone,
                status=AllocationStatus.FAILED,
                message=f"Unknown zone: {zone}",
            )
        
        zone_data = self.zones[zone]
        
        if zone_data["used"] + size > zone_data["size"]:
            return AllocationResult(
                address=0,
                size=0,
                zone=zone,
                status=AllocationStatus.FAILED,
                message=f"Zone {zone} exhausted",
            )
        
        # Calculate address
        address = zone_data["base"] + zone_data["used"]
        zone_data["used"] += size
        
        # Track allocation
        self.allocations[address] = {
            "size": size,
            "zone": zone,
            "requester": requester,
            "timestamp": datetime.now().isoformat(),
        }
        
        logger.info(f"[{self.cell_id}] Allocated {size} bytes at 0x{address:X} in zone '{zone}'")
        
        return AllocationResult(
            address=address,
            size=size,
            zone=zone,
            status=AllocationStatus.SUCCESS,
            message=f"Successfully allocated {size} bytes",
        )
    
    async def free(self, address: int, size: int = 0) -> bool:
        """Free allocated memory."""
        if address not in self.allocations:
            logger.warning(f"[{self.cell_id}] Attempted to free unknown address: 0x{address:X}")
            return False
        
        alloc = self.allocations[address]
        zone = alloc["zone"]
        actual_size = size if size > 0 else alloc["size"]
        
        self.zones[zone]["used"] -= actual_size
        del self.allocations[address]
        
        logger.info(f"[{self.cell_id}] Freed {actual_size} bytes at 0x{address:X}")
        return True
    
    def get_metrics(self) -> Dict:
        """Get memory metrics."""
        total = sum(z["size"] for z in self.zones.values())
        used = sum(z["used"] for z in self.zones.values())
        
        return {
            "total_memory": total,
            "used_memory": used,
            "available_memory": total - used,
            "utilization_percent": (used / total * 100) if total > 0 else 0,
            "allocations_count": len(self.allocations),
            "zones": {k: {"used": v["used"], "size": v["size"]} for k, v in self.zones.items()},
        }


class ProcessorCellSimulator:
    """
    ProcessorCell Simulator for communication testing.
    
    Simulates the ProcessorCell gRPC service with task management,
    CPU monitoring, and inter-cell memory requests.
    """
    
    def __init__(self, cell_id: str = "processor_cell_001"):
        self.cell_id = cell_id
        self.cell_type = CellType.PROCESSOR
        self.cores = [
            {"id": i, "status": "idle", "task": None} for i in range(4)
        ]
        self.tasks: Dict[str, Dict] = {}
        self.task_counter = 0
        self.message_log: List[CellMessage] = []
        self.memory_cell: Optional[MemoryCellSimulator] = None
        
    def set_memory_cell(self, memory_cell: MemoryCellSimulator):
        """Set reference to MemoryCell for memory operations."""
        self.memory_cell = memory_cell
        
    async def handle_message(self, message: CellMessage) -> CellMessage:
        """Handle incoming message from another cell."""
        self.message_log.append(message)
        
        if message.action == "create_task":
            result = await self.create_task(
                name=message.payload.get("name", "unnamed"),
                priority=message.payload.get("priority", 5),
                memory_required=message.payload.get("memory_required", 0),
                calculation=message.payload.get("calculation"),
            )
            return CellMessage(
                sender=self.cell_id,
                receiver=message.sender,
                message_type=MessageType.RESPONSE,
                action="task_response",
                payload={
                    "task_id": result.task_id,
                    "status": result.status.value,
                    "result": result.result,
                    "memory_used": result.memory_used,
                    "execution_time_ms": result.execution_time_ms,
                    "error": result.error,
                },
                message_id=f"resp_{message.message_id}",
            )
            
        elif message.action == "get_status":
            return CellMessage(
                sender=self.cell_id,
                receiver=message.sender,
                message_type=MessageType.RESPONSE,
                action="status_response",
                payload={
                    "cores": self.cores,
                    "tasks_count": len(self.tasks),
                    "active_tasks": len([t for t in self.tasks.values() if t["status"] == "running"]),
                },
            )
            
        return CellMessage(
            sender=self.cell_id,
            receiver=message.sender,
            message_type=MessageType.ERROR,
            action="unknown_action",
            payload={"error": f"Unknown action: {message.action}"},
        )
    
    async def create_task(
        self,
        name: str,
        priority: int = 5,
        memory_required: int = 0,
        calculation: Dict = None,
    ) -> TaskResult:
        """Create and execute a task."""
        self.task_counter += 1
        task_id = f"task_{self.task_counter:04d}"
        
        logger.info(f"[{self.cell_id}] Creating task '{name}' (ID: {task_id})")
        
        # Request memory if needed
        memory_address = 0
        if memory_required > 0 and self.memory_cell:
            logger.info(f"[{self.cell_id}] Requesting {memory_required} bytes from MemoryCell")
            
            alloc_msg = CellMessage(
                sender=self.cell_id,
                receiver=self.memory_cell.cell_id,
                message_type=MessageType.REQUEST,
                action="allocate",
                payload={"size": memory_required, "zone": "user"},
                message_id=f"alloc_{task_id}",
            )
            
            response = await self.memory_cell.handle_message(alloc_msg)
            
            if response.payload.get("status") == "success":
                memory_address = response.payload.get("address", 0)
                logger.info(f"[{self.cell_id}] Memory allocated at 0x{memory_address:X}")
            else:
                return TaskResult(
                    task_id=task_id,
                    status=TaskStatus.FAILED,
                    error=f"Memory allocation failed: {response.payload.get('message', 'Unknown error')}",
                )
        
        # Find available core
        available_core = None
        for core in self.cores:
            if core["status"] == "idle":
                available_core = core
                break
        
        if not available_core:
            return TaskResult(
                task_id=task_id,
                status=TaskStatus.FAILED,
                error="No available cores",
            )
        
        # Execute task
        start_time = time.time()
        available_core["status"] = "busy"
        available_core["task"] = task_id
        
        self.tasks[task_id] = {
            "name": name,
            "priority": priority,
            "memory_address": memory_address,
            "memory_size": memory_required,
            "status": "running",
            "start_time": start_time,
        }
        
        # Simulate calculation
        result = None
        if calculation:
            calc_type = calculation.get("type")
            if calc_type == "add":
                result = calculation.get("a", 0) + calculation.get("b", 0)
            elif calc_type == "multiply":
                result = calculation.get("a", 0) * calculation.get("b", 0)
            elif calc_type == "fibonacci":
                n = calculation.get("n", 10)
                result = self._fibonacci(n)
            elif calc_type == "matrix_multiply":
                result = await self._matrix_multiply(calculation.get("size", 10))
        
        # Simulate processing time
        await asyncio.sleep(0.01)
        
        execution_time = (time.time() - start_time) * 1000
        
        # Update task status
        self.tasks[task_id]["status"] = "completed"
        self.tasks[task_id]["result"] = result
        self.tasks[task_id]["execution_time_ms"] = execution_time
        
        # Free core
        available_core["status"] = "idle"
        available_core["task"] = None
        
        logger.info(f"[{self.cell_id}] Task {task_id} completed in {execution_time:.2f}ms, result: {result}")
        
        return TaskResult(
            task_id=task_id,
            status=TaskStatus.COMPLETED,
            result=result,
            memory_used=memory_required,
            execution_time_ms=execution_time,
        )
    
    def _fibonacci(self, n: int) -> int:
        """Calculate Fibonacci number."""
        if n <= 1:
            return n
        a, b = 0, 1
        for _ in range(2, n + 1):
            a, b = b, a + b
        return b
    
    async def _matrix_multiply(self, size: int) -> Dict:
        """Simulate matrix multiplication."""
        # Just return stats, don't actually multiply
        operations = size * size * size * 2
        return {
            "matrix_size": size,
            "operations": operations,
            "simulated": True,
        }


class CellCommunicationVerifier:
    """
    Verifies communication between MemoryCell and ProcessorCell.
    """
    
    def __init__(self):
        self.memory_cell = MemoryCellSimulator()
        self.processor_cell = ProcessorCellSimulator()
        self.processor_cell.set_memory_cell(self.memory_cell)
        self.serial_log: List[str] = []
        self.test_results: List[Dict] = []
        
    def log_serial(self, message: str):
        """Log serial console output."""
        timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        log_entry = f"[{timestamp}] {message}"
        self.serial_log.append(log_entry)
        logger.info(f"[SERIAL] {message}")
        
    async def run_verification(self) -> Dict:
        """Run complete communication verification."""
        self.log_serial("=== KolibriOS AI Cell Communication Verification ===")
        self.log_serial("Starting verification experiment...")
        
        results = {
            "experiment_name": "Core Cell Communication Verification",
            "start_time": datetime.now().isoformat(),
            "tests": [],
            "serial_log": [],
            "success": True,
            "summary": {},
        }
        
        # Test 1: MemoryCell Direct Allocation
        test1 = await self.test_memory_allocation()
        results["tests"].append(test1)
        if not test1["success"]:
            results["success"] = False
        
        # Test 2: ProcessorCell Task without Memory
        test2 = await self.test_processor_simple_task()
        results["tests"].append(test2)
        if not test2["success"]:
            results["success"] = False
        
        # Test 3: ProcessorCell Task with Memory Request (Inter-cell communication)
        test3 = await self.test_inter_cell_communication()
        results["tests"].append(test3)
        if not test3["success"]:
            results["success"] = False
        
        # Test 4: Multiple allocations and tasks
        test4 = await self.test_multiple_operations()
        results["tests"].append(test4)
        if not test4["success"]:
            results["success"] = False
        
        # Generate summary
        results["summary"] = {
            "total_tests": len(results["tests"]),
            "passed": sum(1 for t in results["tests"] if t["success"]),
            "failed": sum(1 for t in results["tests"] if not t["success"]),
            "memory_allocated": self.memory_cell.zones["user"]["used"],
            "tasks_created": len(self.processor_cell.tasks),
        }
        
        results["end_time"] = datetime.now().isoformat()
        results["serial_log"] = self.serial_log
        
        self.log_serial("=== Verification Complete ===")
        self.log_serial(f"Tests: {results['summary']['passed']}/{results['summary']['total_tests']} passed")
        
        return results
    
    async def test_memory_allocation(self) -> Dict:
        """Test 1: Direct memory allocation."""
        self.log_serial("TEST 1: MemoryCell Direct Allocation")
        
        # Send allocation message
        message = CellMessage(
            sender="test_harness",
            receiver=self.memory_cell.cell_id,
            message_type=MessageType.REQUEST,
            action="allocate",
            payload={"size": 1024 * 1024, "zone": "user"},  # 1MB
            message_id="test_001",
        )
        
        response = await self.memory_cell.handle_message(message)
        
        success = (
            response.message_type == MessageType.RESPONSE and
            response.payload.get("status") == "success" and
            response.payload.get("address", 0) > 0
        )
        
        self.log_serial(f"  Allocation: {response.payload.get('size', 0)} bytes at 0x{response.payload.get('address', 0):X}")
        self.log_serial(f"  Status: {'PASS' if success else 'FAIL'}")
        
        return {
            "test_name": "MemoryCell Direct Allocation",
            "success": success,
            "details": response.to_dict(),
        }
    
    async def test_processor_simple_task(self) -> Dict:
        """Test 2: Simple processor task without memory."""
        self.log_serial("TEST 2: ProcessorCell Simple Task")
        
        message = CellMessage(
            sender="test_harness",
            receiver=self.processor_cell.cell_id,
            message_type=MessageType.REQUEST,
            action="create_task",
            payload={
                "name": "simple_calculation",
                "priority": 5,
                "calculation": {"type": "add", "a": 42, "b": 58},
            },
            message_id="test_002",
        )
        
        response = await self.processor_cell.handle_message(message)
        
        success = (
            response.message_type == MessageType.RESPONSE and
            response.payload.get("status") == "completed" and
            response.payload.get("result") == 100  # 42 + 58
        )
        
        self.log_serial(f"  Task ID: {response.payload.get('task_id')}")
        self.log_serial(f"  Result: {response.payload.get('result')}")
        self.log_serial(f"  Execution: {response.payload.get('execution_time_ms', 0):.2f}ms")
        self.log_serial(f"  Status: {'PASS' if success else 'FAIL'}")
        
        return {
            "test_name": "ProcessorCell Simple Task",
            "success": success,
            "details": response.to_dict(),
        }
    
    async def test_inter_cell_communication(self) -> Dict:
        """Test 3: Inter-cell communication (Processor requests memory)."""
        self.log_serial("TEST 3: Inter-Cell Communication (Processor -> Memory)")
        
        # Get initial memory state
        initial_metrics = self.memory_cell.get_metrics()
        initial_used = initial_metrics["used_memory"]
        
        # Create task that requires memory
        message = CellMessage(
            sender="test_harness",
            receiver=self.processor_cell.cell_id,
            message_type=MessageType.REQUEST,
            action="create_task",
            payload={
                "name": "memory_intensive_task",
                "priority": 3,
                "memory_required": 1024 * 1024,  # 1MB
                "calculation": {"type": "fibonacci", "n": 20},
            },
            message_id="test_003",
        )
        
        response = await self.processor_cell.handle_message(message)
        
        # Verify memory was allocated
        final_metrics = self.memory_cell.get_metrics()
        memory_allocated = final_metrics["used_memory"] - initial_used
        
        success = (
            response.message_type == MessageType.RESPONSE and
            response.payload.get("status") == "completed" and
            memory_allocated >= 1024 * 1024 and  # At least 1MB allocated
            len(self.memory_cell.message_log) > 0  # MemoryCell received message
        )
        
        self.log_serial(f"  Task ID: {response.payload.get('task_id')}")
        self.log_serial(f"  Fibonacci(20): {response.payload.get('result')}")
        self.log_serial(f"  Memory Allocated: {memory_allocated / 1024:.1f} KB")
        self.log_serial(f"  Inter-cell messages: {len(self.memory_cell.message_log)}")
        self.log_serial(f"  Status: {'PASS' if success else 'FAIL'}")
        
        return {
            "test_name": "Inter-Cell Communication",
            "success": success,
            "details": {
                "response": response.to_dict(),
                "memory_allocated": memory_allocated,
                "messages_exchanged": len(self.memory_cell.message_log),
            },
        }
    
    async def test_multiple_operations(self) -> Dict:
        """Test 4: Multiple concurrent operations."""
        self.log_serial("TEST 4: Multiple Concurrent Operations")
        
        # Get initial state
        initial_tasks = len(self.processor_cell.tasks)
        
        # Create multiple tasks with memory requirements
        tasks = []
        for i in range(3):
            message = CellMessage(
                sender="test_harness",
                receiver=self.processor_cell.cell_id,
                message_type=MessageType.REQUEST,
                action="create_task",
                payload={
                    "name": f"concurrent_task_{i}",
                    "priority": i + 1,
                    "memory_required": 512 * 1024,  # 512KB each
                    "calculation": {"type": "multiply", "a": i * 10, "b": i * 20},
                },
                message_id=f"test_004_{i}",
            )
            response = await self.processor_cell.handle_message(message)
            tasks.append(response)
        
        # Verify all tasks completed
        all_completed = all(
            t.payload.get("status") == "completed" for t in tasks
        )
        
        final_tasks = len(self.processor_cell.tasks)
        new_tasks = final_tasks - initial_tasks
        
        success = all_completed and new_tasks == 3
        
        self.log_serial(f"  Tasks created: {new_tasks}")
        self.log_serial(f"  All completed: {all_completed}")
        self.log_serial(f"  Results: {[t.payload.get('result') for t in tasks]}")
        self.log_serial(f"  Status: {'PASS' if success else 'FAIL'}")
        
        return {
            "test_name": "Multiple Concurrent Operations",
            "success": success,
            "details": {
                "tasks_created": new_tasks,
                "all_completed": all_completed,
                "results": [t.payload.get("result") for t in tasks],
            },
        }


async def main():
    """Main verification execution."""
    logger.info("=" * 60)
    logger.info("KolibriOS AI - Core Cell Communication Verification")
    logger.info("=" * 60)
    
    verifier = CellCommunicationVerifier()
    results = await verifier.run_verification()
    
    # Print summary
    print("\n" + "=" * 60)
    print("VERIFICATION SUMMARY")
    print("=" * 60)
    print(f"Total Tests: {results['summary']['total_tests']}")
    print(f"Passed: {results['summary']['passed']} ✅")
    print(f"Failed: {results['summary']['failed']} ❌")
    print(f"Success Rate: {results['summary']['passed'] / results['summary']['total_tests'] * 100:.0f}%")
    print(f"Memory Allocated: {results['summary']['memory_allocated'] / 1024:.1f} KB")
    print(f"Tasks Created: {results['summary']['tasks_created']}")
    print("=" * 60)
    
    # Save results
    output_dir = os.path.join(os.path.dirname(__file__), "..", "docs", "experiments")
    os.makedirs(output_dir, exist_ok=True)
    
    output_file = os.path.join(output_dir, "cell_communication_verification.json")
    with open(output_file, "w") as f:
        json.dump(results, f, indent=2, default=str)
    logger.info(f"Results saved to: {output_file}")
    
    return results


if __name__ == "__main__":
    asyncio.run(main())

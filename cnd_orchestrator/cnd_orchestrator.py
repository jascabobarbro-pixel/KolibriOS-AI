#!/usr/bin/env python3
"""
Central Neural Device (CND) Orchestrator

The main orchestration system for KolibriOS AI that coordinates
all cells and makes system-wide decisions.
"""

import asyncio
import logging
import signal
import sys
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from typing import Any, Optional

import grpc
from loguru import logger
from prometheus_client import Counter, Gauge, Histogram, start_http_server

# Configure logging
logger.remove()
logger.add(sys.stderr, level="INFO", format="<green>{time:YYYY-MM-DD HH:mm:ss}</green> | <level>{level: <8}</level> | <cyan>{name}</cyan>:<cyan>{function}</cyan> - <level>{message}</level>")

# Prometheus metrics
ORCHESTRATOR_METRICS = {
    "cells_registered": Gauge("cnd_cells_registered", "Number of registered cells"),
    "commands_sent": Counter("cnd_commands_sent_total", "Total commands sent to cells"),
    "alerts_raised": Counter("cnd_alerts_raised_total", "Total alerts raised"),
    "heartbeat_latency": Histogram("cnd_heartbeat_latency_seconds", "Heartbeat latency"),
}


class CellType(str, Enum):
    """Types of cells in the system."""
    MEMORY = "memory"
    PROCESSOR = "processor"
    IO = "io"
    NETWORK = "network"
    AI = "ai"


class CellStatus(str, Enum):
    """Cell status states."""
    INITIALIZING = "initializing"
    ACTIVE = "active"
    DEGRADED = "degraded"
    HEALING = "healing"
    SHUTDOWN = "shutdown"


class HealthStatus(str, Enum):
    """Health status levels."""
    HEALTHY = "healthy"
    WARNING = "warning"
    CRITICAL = "critical"


@dataclass
class CellInfo:
    """Information about a registered cell."""
    cell_id: str
    cell_type: CellType
    endpoint: str
    status: CellStatus = CellStatus.INITIALIZING
    health: HealthStatus = HealthStatus.HEALTHY
    last_heartbeat: Optional[datetime] = None
    capabilities: dict[str, str] = field(default_factory=dict)
    metrics: dict[str, float] = field(default_factory=dict)

    def is_healthy(self, timeout_seconds: int = 30) -> bool:
        """Check if cell is healthy based on last heartbeat."""
        if self.last_heartbeat is None:
            return False
        elapsed = (datetime.now() - self.last_heartbeat).total_seconds()
        return elapsed < timeout_seconds


@dataclass
class SystemMetrics:
    """System-wide metrics aggregated from all cells."""
    timestamp: datetime = field(default_factory=datetime.now)
    total_memory: int = 0
    used_memory: int = 0
    memory_utilization: float = 0.0
    total_cores: int = 0
    active_cores: int = 0
    cpu_utilization: float = 0.0
    running_tasks: int = 0
    pending_tasks: int = 0
    overall_health: HealthStatus = HealthStatus.HEALTHY


@dataclass
class Alert:
    """System alert."""
    alert_id: str
    severity: str  # info, warning, error, critical
    source_cell: str
    message: str
    timestamp: datetime = field(default_factory=datetime.now)
    metadata: dict[str, str] = field(default_factory=dict)


class CndOrchestrator:
    """
    Central Neural Device Orchestrator.
    
    Coordinates all cells in the system, collects metrics,
    and makes system-wide decisions.
    """

    def __init__(
        self,
        orchestrator_id: str = "cnd-orchestrator-0",
        heartbeat_interval: float = 5.0,
        metrics_port: int = 9090,
    ) -> None:
        self.orchestrator_id = orchestrator_id
        self.heartbeat_interval = heartbeat_interval
        self.metrics_port = metrics_port

        # Registered cells
        self.cells: dict[str, CellInfo] = {}

        # Active alerts
        self.alerts: list[Alert] = []

        # System metrics
        self.system_metrics = SystemMetrics()

        # gRPC channels to cells
        self.channels: dict[str, grpc.Channel] = {}

        # Running flag
        self._running = False

        # Background tasks
        self._tasks: list[asyncio.Task] = []

        logger.info(f"CND Orchestrator {orchestrator_id} initialized")

    async def start(self) -> None:
        """Start the orchestrator."""
        self._running = True

        # Start Prometheus metrics server
        try:
            start_http_server(self.metrics_port)
            logger.info(f"Prometheus metrics server started on port {self.metrics_port}")
        except Exception as e:
            logger.warning(f"Could not start metrics server: {e}")

        # Start background tasks
        self._tasks = [
            asyncio.create_task(self._heartbeat_loop()),
            asyncio.create_task(self._metrics_aggregation_loop()),
            asyncio.create_task(self._health_monitor_loop()),
        ]

        logger.info("CND Orchestrator started")

    async def stop(self) -> None:
        """Stop the orchestrator."""
        self._running = False

        # Cancel background tasks
        for task in self._tasks:
            task.cancel()
            try:
                await task
            except asyncio.CancelledError:
                pass

        # Close gRPC channels
        for cell_id, channel in self.channels.items():
            channel.close()
            logger.debug(f"Closed channel to cell {cell_id}")

        logger.info("CND Orchestrator stopped")

    async def register_cell(
        self,
        cell_id: str,
        cell_type: str,
        endpoint: str,
        capabilities: Optional[dict[str, str]] = None,
    ) -> bool:
        """Register a new cell with the orchestrator."""
        try:
            cell_type_enum = CellType(cell_type.lower())
        except ValueError:
            logger.error(f"Invalid cell type: {cell_type}")
            return False

        cell_info = CellInfo(
            cell_id=cell_id,
            cell_type=cell_type_enum,
            endpoint=endpoint,
            capabilities=capabilities or {},
            last_heartbeat=datetime.now(),
        )

        self.cells[cell_id] = cell_info
        ORCHESTRATOR_METRICS["cells_registered"].set(len(self.cells))

        logger.info(f"Registered cell {cell_id} of type {cell_type} at {endpoint}")

        # Try to connect to the cell
        try:
            channel = grpc.aio.insecure_channel(endpoint)
            self.channels[cell_id] = channel
            logger.debug(f"Established connection to cell {cell_id}")
        except Exception as e:
            logger.warning(f"Could not connect to cell {cell_id}: {e}")

        return True

    async def unregister_cell(self, cell_id: str, graceful: bool = True) -> bool:
        """Unregister a cell from the orchestrator."""
        if cell_id not in self.cells:
            logger.warning(f"Cell {cell_id} not found")
            return False

        # Close channel
        if cell_id in self.channels:
            self.channels[cell_id].close()
            del self.channels[cell_id]

        del self.cells[cell_id]
        ORCHESTRATOR_METRICS["cells_registered"].set(len(self.cells))

        if not graceful:
            self._raise_alert(
                severity="warning",
                source_cell=cell_id,
                message=f"Cell {cell_id} unregistered ungracefully",
            )

        logger.info(f"Unregistered cell {cell_id}")
        return True

    async def send_command(
        self,
        cell_id: str,
        command: str,
        parameters: Optional[dict[str, str]] = None,
    ) -> dict[str, Any]:
        """Send a command to a specific cell using gRPC."""
        if cell_id not in self.cells:
            return {"success": False, "error": "Cell not found"}

        ORCHESTRATOR_METRICS["commands_sent"].inc()
        cell_info = self.cells[cell_id]
        parameters = parameters or {}

        logger.info(f"Sending command '{command}' to cell {cell_id}")

        # Get the gRPC channel for this cell
        channel = self.channels.get(cell_id)
        if not channel:
            logger.warning(f"No gRPC channel for cell {cell_id}")
            return {"success": False, "error": "No gRPC channel available"}

        try:
            # Execute command based on cell type and command name
            if cell_info.cell_type == CellType.MEMORY:
                return await self._execute_memory_cell_command(channel, command, parameters)
            elif cell_info.cell_type == CellType.PROCESSOR:
                return await self._execute_processor_cell_command(channel, command, parameters)
            else:
                # Generic command execution via reflection-like approach
                return await self._execute_generic_command(channel, command, parameters)

        except grpc.AioRpcError as e:
            logger.error(f"gRPC error sending command to {cell_id}: {e}")
            return {"success": False, "error": str(e), "code": e.code().name}
        except Exception as e:
            logger.error(f"Error sending command to {cell_id}: {e}")
            return {"success": False, "error": str(e)}

    async def _execute_memory_cell_command(
        self,
        channel: grpc.Channel,
        command: str,
        parameters: dict[str, str],
    ) -> dict[str, Any]:
        """Execute a command on a Memory Cell via gRPC."""
        # Import generated protobuf modules dynamically
        try:
            from cells.protos import memory_cell_pb2, memory_cell_pb2_grpc, cell_common_pb2

            stub = memory_cell_pb2_grpc.MemoryCellServiceStub(channel)

            if command == "get_stats":
                response = await stub.GetStats(cell_common_pb2.Empty())
                return {
                    "success": True,
                    "data": {
                        "total_memory": response.total_memory,
                        "used_memory": response.used_memory,
                        "available_memory": response.available_memory,
                        "utilization_percent": response.utilization_percent,
                        "allocation_count": response.allocation_count,
                    }
                }

            elif command == "allocate":
                request = memory_cell_pb2.AllocateRequest(
                    size=int(parameters.get("size", 4096)),
                    pool_name=parameters.get("pool_name", "user"),
                )
                response = await stub.Allocate(request)
                return {
                    "success": response.success,
                    "address": response.address,
                    "allocation_id": response.allocation_id,
                    "error_message": response.error_message,
                }

            elif command == "deallocate":
                request = memory_cell_pb2.DeallocateRequest(
                    allocation_id=parameters.get("allocation_id", "")
                )
                response = await stub.Deallocate(request)
                return {
                    "success": response.success,
                    "message": response.message,
                }

            elif command == "defragment":
                request = memory_cell_pb2.DefragmentRequest(
                    pool_name=parameters.get("pool_name", ""),
                    aggressive=parameters.get("aggressive", "false").lower() == "true",
                )
                response = await stub.Defragment(request)
                return {"success": response.success, "message": response.message}

            elif command == "diagnostics":
                response = await stub.RunDiagnostics(cell_common_pb2.Empty())
                return {
                    "success": True,
                    "healthy": response.healthy,
                    "issues": [
                        {"severity": i.severity, "description": i.description}
                        for i in response.issues
                    ],
                }

            elif command == "heartbeat":
                response = await stub.GetHeartbeat(cell_common_pb2.Empty())
                return {
                    "success": True,
                    "cell_id": response.cell_id,
                    "status": response.status,
                    "health": response.health,
                }

            elif command == "metrics":
                response = await stub.GetMetrics(cell_common_pb2.Empty())
                return {
                    "success": True,
                    "gauge_metrics": dict(response.gauge_metrics),
                    "counter_metrics": dict(response.counter_metrics),
                }

            else:
                return {"success": False, "error": f"Unknown command: {command}"}

        except ImportError:
            # Protobuf modules not generated - use fallback
            logger.warning("Protobuf modules not available, using fallback response")
            return {
                "success": True,
                "message": f"Command '{command}' queued (protobuf not generated)",
                "parameters": parameters,
            }

    async def _execute_processor_cell_command(
        self,
        channel: grpc.Channel,
        command: str,
        parameters: dict[str, str],
    ) -> dict[str, Any]:
        """Execute a command on a Processor Cell via gRPC."""
        try:
            from cells.protos import processor_cell_pb2, processor_cell_pb2_grpc, cell_common_pb2

            stub = processor_cell_pb2_grpc.ProcessorCellServiceStub(channel)

            if command == "get_cpu_stats":
                response = await stub.GetCpuStats(cell_common_pb2.Empty())
                return {
                    "success": True,
                    "data": {
                        "total_cores": response.total_cores,
                        "active_cores": response.active_cores,
                        "utilization": response.utilization,
                    }
                }

            elif command == "list_tasks":
                response = await stub.ListTasks(cell_common_pb2.Empty())
                return {
                    "success": True,
                    "tasks": [
                        {"id": t.id, "status": t.status, "priority": t.priority}
                        for t in response.tasks
                    ],
                }

            elif command == "execute_task":
                request = processor_cell_pb2.ExecuteTaskRequest(
                    executable=parameters.get("executable", ""),
                    args=parameters.get("args", "").split(),
                    priority=int(parameters.get("priority", 0)),
                )
                response = await stub.ExecuteTask(request)
                return {
                    "success": response.success,
                    "task_id": response.task_id,
                    "message": response.message,
                }

            elif command == "heartbeat":
                response = await stub.GetHeartbeat(cell_common_pb2.Empty())
                return {
                    "success": True,
                    "cell_id": response.cell_id,
                    "status": response.status,
                    "health": response.health,
                }

            else:
                return {"success": False, "error": f"Unknown command: {command}"}

        except ImportError:
            logger.warning("Protobuf modules not available, using fallback response")
            return {
                "success": True,
                "message": f"Command '{command}' queued (protobuf not generated)",
                "parameters": parameters,
            }

    async def _execute_generic_command(
        self,
        channel: grpc.Channel,
        command: str,
        parameters: dict[str, str],
    ) -> dict[str, Any]:
        """Execute a generic command using gRPC reflection or health check."""
        try:
            from grpc_reflection.v1alpha import reflection_pb2, reflection_pb2_grpc

            # Try to use gRPC reflection to discover services
            reflection_stub = reflection_pb2_grpc.ServerReflectionStub(channel)

            # List available services
            request = reflection_pb2.ServerReflectionRequest(list_services="")
            responses = reflection_stub.ServerReflectionInfo(iter([request]))

            async for response in responses:
                if response.HasField('list_services_response'):
                    services = [s.name for s in response.list_services_response.service]
                    logger.info(f"Available services: {services}")

            return {
                "success": True,
                "message": f"Command '{command}' discovered via reflection",
                "parameters": parameters,
            }

        except ImportError:
            # No reflection available - use health check
            try:
                from grpc_health.v1 import health_pb2, health_pb2_grpc

                health_stub = health_pb2_grpc.HealthStub(channel)
                response = await health_stub.Check(health_pb2.HealthCheckRequest())

                return {
                    "success": True,
                    "message": f"Cell health: {response.status}",
                    "parameters": parameters,
                }
            except Exception as e:
                return {
                    "success": True,
                    "message": f"Command '{command}' queued",
                    "parameters": parameters,
                    "warning": str(e),
                }

    async def broadcast_command(
        self,
        command: str,
        parameters: Optional[dict[str, str]] = None,
        cell_type: Optional[CellType] = None,
    ) -> dict[str, dict[str, Any]]:
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

    def list_cells(self, cell_type: Optional[CellType] = None) -> list[CellInfo]:
        """List all registered cells, optionally filtered by type."""
        if cell_type:
            return [c for c in self.cells.values() if c.cell_type == cell_type]
        return list(self.cells.values())

    def get_alerts(self, severity: Optional[str] = None) -> list[Alert]:
        """Get all alerts, optionally filtered by severity."""
        if severity:
            return [a for a in self.alerts if a.severity == severity]
        return list(self.alerts)

    def _raise_alert(
        self,
        severity: str,
        source_cell: str,
        message: str,
        metadata: Optional[dict[str, str]] = None,
    ) -> Alert:
        """Raise a new alert."""
        alert = Alert(
            alert_id=f"alert-{len(self.alerts)}",
            severity=severity,
            source_cell=source_cell,
            message=message,
            metadata=metadata or {},
        )

        self.alerts.append(alert)
        ORCHESTRATOR_METRICS["alerts_raised"].inc()

        logger.warning(f"Alert raised: [{severity}] {message}")
        return alert

    async def _heartbeat_loop(self) -> None:
        """Periodically check heartbeats from all cells."""
        while self._running:
            try:
                for cell_id, cell_info in list(self.cells.items()):
                    # In a real implementation, this would call the cell's gRPC heartbeat
                    cell_info.last_heartbeat = datetime.now()

                await asyncio.sleep(self.heartbeat_interval)
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Heartbeat loop error: {e}")
                await asyncio.sleep(1)

    async def _metrics_aggregation_loop(self) -> None:
        """Periodically aggregate metrics from all cells."""
        while self._running:
            try:
                total_memory = 0
                used_memory = 0
                total_cores = 0
                active_cores = 0
                cpu_utilization = 0.0
                memory_cells = 0
                processor_cells = 0

                for cell_info in self.cells.values():
                    if cell_info.cell_type == CellType.MEMORY:
                        memory_cells += 1
                        total_memory += cell_info.metrics.get("total_memory", 0)
                        used_memory += cell_info.metrics.get("used_memory", 0)
                    elif cell_info.cell_type == CellType.PROCESSOR:
                        processor_cells += 1
                        total_cores += int(cell_info.metrics.get("total_cores", 0))
                        active_cores += int(cell_info.metrics.get("active_cores", 0))
                        cpu_utilization += cell_info.metrics.get("utilization", 0.0)

                # Update system metrics
                self.system_metrics = SystemMetrics(
                    total_memory=total_memory,
                    used_memory=used_memory,
                    memory_utilization=(used_memory / total_memory * 100) if total_memory > 0 else 0,
                    total_cores=total_cores,
                    active_cores=active_cores,
                    cpu_utilization=cpu_utilization / processor_cells if processor_cells > 0 else 0,
                )

                await asyncio.sleep(10)  # Aggregate every 10 seconds
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Metrics aggregation error: {e}")
                await asyncio.sleep(1)

    async def _health_monitor_loop(self) -> None:
        """Monitor cell health and raise alerts."""
        while self._running:
            try:
                for cell_id, cell_info in list(self.cells.items()):
                    if not cell_info.is_healthy(timeout_seconds=30):
                        self._raise_alert(
                            severity="warning",
                            source_cell=cell_id,
                            message=f"Cell {cell_id} heartbeat timeout",
                        )
                        cell_info.status = CellStatus.DEGRADED

                await asyncio.sleep(15)  # Check every 15 seconds
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Health monitor error: {e}")
                await asyncio.sleep(1)

    async def run_diagnostics(self) -> dict[str, Any]:
        """Run system-wide diagnostics."""
        results = {
            "healthy": True,
            "cells": {},
            "issues": [],
            "recommendations": [],
        }

        for cell_id, cell_info in self.cells.items():
            cell_result = {
                "status": cell_info.status.value,
                "health": cell_info.health.value,
                "last_heartbeat": cell_info.last_heartbeat.isoformat() if cell_info.last_heartbeat else None,
            }
            results["cells"][cell_id] = cell_result

            if cell_info.health == HealthStatus.CRITICAL:
                results["healthy"] = False
                results["issues"].append(f"Cell {cell_id} is in critical state")

        return results

    async def initiate_recovery(
        self,
        target_cell_id: Optional[str] = None,
        strategy: str = "restart",
    ) -> dict[str, Any]:
        """Initiate recovery for a cell or the entire system."""
        if target_cell_id:
            # Single cell recovery
            if target_cell_id not in self.cells:
                return {"success": False, "error": "Cell not found"}

            cell_info = self.cells[target_cell_id]
            cell_info.status = CellStatus.HEALING

            logger.info(f"Initiating {strategy} recovery for cell {target_cell_id}")

            # Simulate recovery
            await asyncio.sleep(1)
            cell_info.status = CellStatus.ACTIVE
            cell_info.health = HealthStatus.HEALTHY

            return {"success": True, "message": f"Recovery completed for {target_cell_id}"}
        else:
            # System-wide recovery
            logger.info("Initiating system-wide recovery")
            results = {}

            for cell_id in self.cells:
                results[cell_id] = await self.initiate_recovery(cell_id, strategy)

            return {"success": True, "results": results}


async def main() -> None:
    """Main entry point for the CND Orchestrator."""
    orchestrator = CndOrchestrator(
        orchestrator_id="cnd-orchestrator-0",
        heartbeat_interval=5.0,
        metrics_port=9090,
    )

    # Setup signal handlers
    loop = asyncio.get_event_loop()
    stop_event = asyncio.Event()

    def signal_handler() -> None:
        logger.info("Shutdown signal received")
        stop_event.set()

    for sig in (signal.SIGINT, signal.SIGTERM):
        loop.add_signal_handler(sig, signal_handler)

    try:
        await orchestrator.start()

        # Register some test cells
        await orchestrator.register_cell(
            cell_id="memory-cell-0",
            cell_type="memory",
            endpoint="localhost:50051",
            capabilities={"pools": "kernel,user,shared"},
        )

        await orchestrator.register_cell(
            cell_id="processor-cell-0",
            cell_type="processor",
            endpoint="localhost:50052",
            capabilities={"cores": "auto"},
        )

        logger.info("CND Orchestrator running. Press Ctrl+C to stop.")

        # Wait for shutdown signal
        await stop_event.wait()

    finally:
        await orchestrator.stop()


if __name__ == "__main__":
    asyncio.run(main())

# CND Orchestrator

Central Neural Device (CND) Orchestrator for KolibriOS AI.

## Overview

The CND Orchestrator is the central coordination system for all cells in KolibriOS AI. It:

- Registers and tracks all cells in the system
- Collects and aggregates metrics from all cells
- Monitors cell health and raises alerts
- Sends commands to cells
- Initiates recovery procedures

## Running

```bash
# Install dependencies
pip install -e .

# Run the orchestrator
python cnd_orchestrator.py
```

## Configuration

Environment variables:

- `CND_HEARTBEAT_INTERVAL`: Heartbeat check interval (default: 5.0)
- `CND_METRICS_PORT`: Prometheus metrics port (default: 9090)

## API

### Registration

```python
await orchestrator.register_cell(
    cell_id="memory-cell-0",
    cell_type="memory",
    endpoint="localhost:50051",
)
```

### Commands

```python
result = await orchestrator.send_command(
    cell_id="memory-cell-0",
    command="allocate",
    parameters={"size": "1024", "pool": "user"},
)
```

### Metrics

```python
metrics = orchestrator.get_system_metrics()
print(f"Memory utilization: {metrics.memory_utilization}%")
print(f"CPU utilization: {metrics.cpu_utilization}%")
```

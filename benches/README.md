# KolibriOS AI Benchmarks

This directory contains performance benchmarks for all major components.

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench kernel_bench
cargo bench --bench memory_cell_bench
cargo bench --bench neural_scheduler_bench

# Run with criterion output
cargo bench -- --verbose
```

## Benchmark Categories

| Benchmark | Description |
|-----------|-------------|
| `kernel_bench` | Kernel operations, gene updates, IPC |
| `memory_cell_bench` | Memory allocation, pool management |
| `processor_cell_bench` | Task scheduling, CPU management |
| `neural_scheduler_bench` | Neural network inference |
| `koli_lang_bench` | Lexer, parser, codegen |
| `grpc_bench` | gRPC communication |

## Performance Targets

| Operation | Target | Unit |
|-----------|--------|------|
| IPC latency | < 1 | μs |
| Context switch | < 100 | ns |
| Memory allocation | < 50 | ns |
| Neural inference | < 10 | μs |
| Lexer | > 1M | tokens/s |
| Parser | > 500K | AST nodes/s |

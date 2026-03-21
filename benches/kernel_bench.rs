//! Kernel Benchmarks
//!
//! Performance benchmarks for kernel operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};

// Simulate kernel components for benchmarking

#[derive(Debug, Clone, Default)]
struct SystemStateInput {
    cpu_utilization: [f32; 4],
    memory_pressure: f32,
    ready_tasks: f32,
    blocked_tasks: f32,
    avg_priority: f32,
    io_bound_ratio: f32,
    cache_hit_ratio: f32,
    load_average: f32,
    context_switches: f32,
    interrupt_rate: f32,
    time_since_decision: f32,
}

impl SystemStateInput {
    fn default_test() -> Self {
        Self {
            cpu_utilization: [0.5, 0.3, 0.7, 0.4],
            memory_pressure: 0.6,
            ready_tasks: 10.0,
            blocked_tasks: 2.0,
            avg_priority: 0.5,
            io_bound_ratio: 0.3,
            cache_hit_ratio: 0.85,
            load_average: 0.5,
            context_switches: 100.0,
            interrupt_rate: 50.0,
            time_since_decision: 1.0,
        }
    }
}

/// Neural scheduler decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SchedulingDecision {
    RunPriority,
    RunIoBound,
    RunCpuBound,
    BalanceLoad,
    Preempt,
    Idle,
    Batch,
    Interactive,
}

/// Simple neural network for scheduling
struct NeuralScheduler {
    weights_layer1: Vec<Vec<f32>>,
    weights_layer2: Vec<Vec<f32>>,
    weights_output: Vec<Vec<f32>>,
}

impl NeuralScheduler {
    fn new() -> Self {
        // Initialize with small random-like weights
        let mut weights_layer1 = Vec::with_capacity(64);
        for _ in 0..64 {
            let mut row = Vec::with_capacity(12);
            for j in 0..12 {
                row.push((j as f32 % 10.0) / 10.0);
            }
            weights_layer1.push(row);
        }
        
        let mut weights_layer2 = Vec::with_capacity(32);
        for _ in 0..32 {
            let mut row = Vec::with_capacity(64);
            for j in 0..64 {
                row.push((j as f32 % 10.0) / 10.0);
            }
            weights_layer2.push(row);
        }
        
        let mut weights_output = Vec::with_capacity(8);
        for _ in 0..8 {
            let mut row = Vec::with_capacity(32);
            for j in 0..32 {
                row.push((j as f32 % 10.0) / 10.0);
            }
            weights_output.push(row);
        }
        
        Self { weights_layer1, weights_layer2, weights_output }
    }
    
    fn relu(x: f32) -> f32 {
        if x > 0.0 { x } else { 0.0 }
    }
    
    fn softmax(x: &[f32]) -> Vec<f32> {
        let max = x.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let exp_sum: f32 = x.iter().map(|&v| (v - max).exp()).sum();
        x.iter().map(|&v| (v - max).exp() / exp_sum).collect()
    }
    
    fn decide(&self, state: &SystemStateInput) -> SchedulingDecision {
        // Flatten input
        let mut input = Vec::with_capacity(12);
        input.extend_from_slice(&state.cpu_utilization);
        input.push(state.memory_pressure);
        input.push(state.ready_tasks);
        input.push(state.blocked_tasks);
        input.push(state.avg_priority);
        input.push(state.io_bound_ratio);
        input.push(state.cache_hit_ratio);
        input.push(state.load_average);
        input.push(state.context_switches);
        input.push(state.interrupt_rate);
        input.push(state.time_since_decision);
        
        // Layer 1
        let mut hidden1 = vec![0.0f32; 64];
        for (i, weights) in self.weights_layer1.iter().enumerate() {
            let sum: f32 = input.iter().zip(weights.iter()).map(|(a, b)| a * b).sum();
            hidden1[i] = Self::relu(sum);
        }
        
        // Layer 2
        let mut hidden2 = vec![0.0f32; 32];
        for (i, weights) in self.weights_layer2.iter().enumerate() {
            let sum: f32 = hidden1.iter().zip(weights.iter()).map(|(a, b)| a * b).sum();
            hidden2[i] = Self::relu(sum);
        }
        
        // Output
        let mut output = vec![0.0f32; 8];
        for (i, weights) in self.weights_output.iter().enumerate() {
            let sum: f32 = hidden2.iter().zip(weights.iter()).map(|(a, b)| a * b).sum();
            output[i] = sum;
        }
        
        let probs = Self::softmax(&output);
        let max_idx = probs.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        
        match max_idx {
            0 => SchedulingDecision::RunPriority,
            1 => SchedulingDecision::RunIoBound,
            2 => SchedulingDecision::RunCpuBound,
            3 => SchedulingDecision::BalanceLoad,
            4 => SchedulingDecision::Preempt,
            5 => SchedulingDecision::Idle,
            6 => SchedulingDecision::Batch,
            _ => SchedulingDecision::Interactive,
        }
    }
}

/// IPC Message simulation
#[derive(Debug, Clone)]
struct IpcMessage {
    sender: u64,
    receiver: u64,
    data: Vec<u8>,
}

impl IpcMessage {
    fn new(sender: u64, receiver: u64, size: usize) -> Self {
        Self {
            sender,
            receiver,
            data: vec![0u8; size],
        }
    }
}

/// IPC Channel simulation
struct IpcChannel {
    messages: std::collections::VecDeque<IpcMessage>,
}

impl IpcChannel {
    fn new() -> Self {
        Self {
            messages: std::collections::VecDeque::new(),
        }
    }
    
    fn send(&mut self, msg: IpcMessage) {
        self.messages.push_back(msg);
    }
    
    fn recv(&mut self) -> Option<IpcMessage> {
        self.messages.pop_front()
    }
}

/// Gene simulation
#[derive(Debug, Clone)]
struct Gene {
    id: String,
    activity: f32,
}

impl Gene {
    fn update(&mut self, dt: f32) -> f32 {
        self.activity = (self.activity + dt).min(1.0);
        self.activity
    }
}

// Benchmarks

fn bench_neural_scheduler(c: &mut Criterion) {
    let scheduler = NeuralScheduler::new();
    let state = SystemStateInput::default_test();
    
    c.bench_function("neural_scheduler_decide", |b| {
        b.iter(|| scheduler.decide(black_box(&state)))
    });
}

fn bench_ipc_messaging(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipc");
    
    for size in [64, 256, 1024, 4096].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut channel = IpcChannel::new();
            b.iter(|| {
                channel.send(IpcMessage::new(1, 2, size));
                channel.recv()
            });
        });
    }
    
    group.finish();
}

fn bench_gene_updates(c: &mut Criterion) {
    let mut genes: Vec<Gene> = (0..100)
        .map(|i| Gene {
            id: format!("gene-{}", i),
            activity: 0.0,
        })
        .collect();
    
    c.bench_function("gene_batch_update_100", |b| {
        b.iter(|| {
            for gene in &mut genes {
                black_box(gene.update(0.016));
            }
        });
    });
}

fn bench_state_input_creation(c: &mut Criterion) {
    c.bench_function("system_state_input_creation", |b| {
        b.iter(|| SystemStateInput::default_test())
    });
}

fn bench_softmax(c: &mut Criterion) {
    let input = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    
    c.bench_function("softmax_8", |b| {
        b.iter(|| NeuralScheduler::softmax(black_box(&input)))
    });
}

fn bench_relu_batch(c: &mut Criterion) {
    let input: Vec<f32> = (0..64).map(|i| (i as f32 - 32.0) / 10.0).collect();
    
    c.bench_function("relu_batch_64", |b| {
        b.iter(|| {
            input.iter().map(|&x| NeuralScheduler::relu(x)).collect::<Vec<_>>()
        })
    });
}

criterion_group!(
    benches,
    bench_neural_scheduler,
    bench_ipc_messaging,
    bench_gene_updates,
    bench_state_input_creation,
    bench_softmax,
    bench_relu_batch,
);

criterion_main!(benches);

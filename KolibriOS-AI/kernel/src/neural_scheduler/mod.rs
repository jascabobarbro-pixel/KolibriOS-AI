//! Neural Scheduler - AI-powered task scheduling

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// Neural Scheduler Configuration
#[derive(Debug, Clone)]
pub struct NeuralSchedulerConfig {
    pub input_size: usize,
    pub hidden_sizes: Vec<usize>,
    pub output_size: usize,
    pub learning_rate: f32,
    pub use_relu: bool,
}

impl Default for NeuralSchedulerConfig {
    fn default() -> Self {
        Self { input_size: 12, hidden_sizes: vec![64, 32], output_size: 8, learning_rate: 0.001, use_relu: true }
    }
}

/// Neural Network Layer
#[derive(Debug, Clone)]
pub struct Layer {
    pub weights: Vec<Vec<f32>>, pub biases: Vec<f32>, pub size: usize, pub input_size: usize,
}

impl Layer {
    pub fn new(input_size: usize, output_size: usize) -> Self {
        let scale = (2.0 / (input_size + output_size) as f32).sqrt();
        let weights: Vec<Vec<f32>> = (0..output_size)
            .map(|_| (0..input_size).map(|_| scale * 0.5).collect()).collect();
        Self { weights, biases: vec![0.0; output_size], size: output_size, input_size }
    }

    pub fn forward(&self, input: &[f32], use_relu: bool) -> Vec<f32> {
        let mut output = Vec::with_capacity(self.size);
        for i in 0..self.size {
            let mut sum = self.biases[i];
            for (j, &inp) in input.iter().enumerate() { sum += self.weights[i][j] * inp; }
            output.push(if use_relu { sum.max(0.0) } else { sum });
        }
        output
    }
}

/// Neural Network
#[derive(Debug, Clone)]
pub struct NeuralNetwork {
    pub layers: Vec<Layer>,
    pub config: NeuralSchedulerConfig,
}

impl NeuralNetwork {
    pub fn new(config: NeuralSchedulerConfig) -> Self {
        let mut layers = Vec::new();
        let mut prev_size = config.input_size;
        for &size in &config.hidden_sizes { layers.push(Layer::new(prev_size, size)); prev_size = size; }
        layers.push(Layer::new(prev_size, config.output_size));
        Self { layers, config }
    }

    pub fn forward(&self, input: &[f32]) -> Vec<f32> {
        let mut current = input.to_vec();
        for (i, layer) in self.layers.iter().enumerate() {
            let is_last = i == self.layers.len() - 1;
            current = layer.forward(&current, !is_last && self.config.use_relu);
        }
        softmax(&current)
    }

    pub fn predict(&self, input: &[f32]) -> usize {
        let output = self.forward(input);
        output.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(i, _)| i).unwrap_or(0)
    }
}

fn softmax(values: &[f32]) -> Vec<f32> {
    let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exp_sum: f32 = values.iter().map(|v| (v - max).exp()).sum();
    values.iter().map(|v| (v - max).exp() / exp_sum).collect()
}

/// System state input
#[derive(Debug, Clone)]
pub struct SystemStateInput {
    pub cpu_utilization: Vec<f32>, pub memory_pressure: f32, pub ready_tasks: f32,
    pub blocked_tasks: f32, pub avg_priority: f32, pub io_bound_ratio: f32,
    pub cache_hit_ratio: f32, pub load_average: f32, pub context_switches: f32,
    pub interrupt_rate: f32, pub time_since_decision: f32,
}

impl SystemStateInput {
    pub fn to_features(&self) -> Vec<f32> {
        vec![
            self.cpu_utilization.get(0).copied().unwrap_or(0.0),
            self.cpu_utilization.get(1).copied().unwrap_or(0.0),
            self.cpu_utilization.get(2).copied().unwrap_or(0.0),
            self.cpu_utilization.get(3).copied().unwrap_or(0.0),
            self.memory_pressure, self.ready_tasks, self.blocked_tasks,
            self.avg_priority, self.io_bound_ratio, self.cache_hit_ratio,
            self.load_average, self.context_switches,
        ]
    }
}

/// Scheduling decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingDecision {
    RunHighestPriority = 0, RunIoBound = 1, RunCpuBound = 2, BalanceLoad = 3,
    PreemptCurrent = 4, Idle = 5, RunBatch = 6, RunInteractive = 7,
}

impl SchedulingDecision {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::RunHighestPriority, 1 => Self::RunIoBound, 2 => Self::RunCpuBound,
            3 => Self::BalanceLoad, 4 => Self::PreemptCurrent, 5 => Self::Idle,
            6 => Self::RunBatch, 7 => Self::RunInteractive, _ => Self::RunHighestPriority,
        }
    }
}

/// Neural Scheduler
#[derive(Debug, Clone)]
pub struct NeuralScheduler {
    pub network: NeuralNetwork,
    pub decision_history: Vec<(SystemStateInput, SchedulingDecision, f32)>,
    pub confidence_threshold: f32, pub fallback_enabled: bool,
}

impl NeuralScheduler {
    pub fn new(config: NeuralSchedulerConfig) -> Self {
        Self { network: NeuralNetwork::new(config), decision_history: Vec::new(),
            confidence_threshold: 0.6, fallback_enabled: true }
    }

    pub fn decide(&mut self, state: &SystemStateInput) -> SchedulingDecision {
        let features = state.to_features();
        let output = self.network.forward(&features);
        let (decision_idx, confidence) = output.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, &p)| (i, p)).unwrap_or((0, 0.0));
        let decision = SchedulingDecision::from_index(decision_idx);
        self.decision_history.push((state.clone(), decision, confidence));
        if confidence < self.confidence_threshold && self.fallback_enabled {
            if state.memory_pressure > 0.8 { SchedulingDecision::RunIoBound }
            else if state.ready_tasks > 0.7 { SchedulingDecision::BalanceLoad }
            else { decision }
        } else { decision }
    }

    pub fn get_confidence(&self, state: &SystemStateInput) -> f32 {
        let features = state.to_features();
        self.network.forward(&features).into_iter().fold(0.0, f32::max)
    }

    pub fn stats(&self) -> SchedulerStats {
        let total = self.decision_history.len();
        let high_confidence = self.decision_history.iter().filter(|(_, _, c)| *c > self.confidence_threshold).count();
        let mut decision_counts = BTreeMap::new();
        for (_, decision, _) in &self.decision_history { *decision_counts.entry(*decision).or_insert(0) += 1; }
        SchedulerStats { total_decisions: total, high_confidence_count: high_confidence, decision_counts }
    }

    pub fn clear_history(&mut self) { self.decision_history.clear(); }
}

/// Scheduler statistics
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub total_decisions: usize, pub high_confidence_count: usize,
    pub decision_counts: BTreeMap<SchedulingDecision, usize>,
}

impl Default for NeuralScheduler { fn default() -> Self { Self::new(NeuralSchedulerConfig::default()) } }

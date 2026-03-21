//! Memory Cell Benchmarks
//!
//! Performance benchmarks for memory operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Memory Pool
#[derive(Debug, Clone)]
struct MemoryPool {
    name: String,
    total_size: u64,
    used_size: u64,
    allocations: HashMap<String, Allocation>,
}

#[derive(Debug, Clone)]
struct Allocation {
    id: String,
    size: u64,
    offset: u64,
    timestamp: Instant,
}

impl MemoryPool {
    fn new(name: &str, total_size: u64) -> Self {
        Self {
            name: name.to_string(),
            total_size,
            used_size: 0,
            allocations: HashMap::new(),
        }
    }
    
    fn allocate(&mut self, size: u64) -> Option<String> {
        if self.used_size + size > self.total_size {
            return None;
        }
        
        let id = format!("alloc-{}", uuid::Uuid::new_v4());
        let offset = self.used_size;
        
        self.allocations.insert(id.clone(), Allocation {
            id: id.clone(),
            size,
            offset,
            timestamp: Instant::now(),
        });
        
        self.used_size += size;
        Some(id)
    }
    
    fn deallocate(&mut self, id: &str) -> bool {
        if let Some(alloc) = self.allocations.remove(id) {
            self.used_size -= alloc.size;
            true
        } else {
            false
        }
    }
    
    fn utilization(&self) -> f32 {
        if self.total_size == 0 {
            0.0
        } else {
            self.used_size as f32 / self.total_size as f32
        }
    }
}

/// Memory Manager
struct MemoryManager {
    pools: HashMap<String, MemoryPool>,
}

impl MemoryManager {
    fn new() -> Self {
        Self {
            pools: HashMap::new(),
        }
    }
    
    fn create_pool(&mut self, name: &str, size: u64) {
        self.pools.insert(name.to_string(), MemoryPool::new(name, size));
    }
    
    fn allocate(&mut self, pool_name: &str, size: u64) -> Option<String> {
        self.pools.get_mut(pool_name)?.allocate(size)
    }
    
    fn deallocate(&mut self, pool_name: &str, id: &str) -> bool {
        self.pools.get_mut(pool_name)?.deallocate(id)?;
        Some(true).unwrap_or(false)
    }
}

// Simulate uuid module
mod uuid {
    use std::sync::atomic::{AtomicU64, Ordering};
    
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    
    pub struct Uuid(u64);
    
    impl Uuid {
        pub fn new_v4() -> Self {
            Self(COUNTER.fetch_add(1, Ordering::SeqCst))
        }
    }
    
    impl std::fmt::Display for Uuid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:016x}", self.0)
        }
    }
}

// Benchmarks

fn bench_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    
    for size in [64, 256, 1024, 4096, 16384, 65536].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut manager = MemoryManager::new();
            manager.create_pool("test", 1024 * 1024 * 1024); // 1GB
            
            b.iter(|| {
                let id = manager.allocate("test", size as u64);
                if let Some(id) = id {
                    manager.deallocate("test", &id);
                }
                black_box(id)
            });
        });
    }
    
    group.finish();
}

fn bench_pool_creation(c: &mut Criterion) {
    c.bench_function("pool_creation_1gb", |b| {
        b.iter(|| MemoryPool::new("test", 1024 * 1024 * 1024))
    });
}

fn bench_utilization_calculation(c: &mut Criterion) {
    let mut pool = MemoryPool::new("test", 1024 * 1024 * 1024);
    
    // Allocate some memory
    for _ in 0..100 {
        pool.allocate(1024 * 1024); // 1MB each
    }
    
    c.bench_function("utilization_calculation", |b| {
        b.iter(|| black_box(pool.utilization()))
    });
}

fn bench_concurrent_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_allocations");
    
    for count in [10, 100, 1000].iter() {
        group.bench_function(format!("{}_allocations", count), |b| {
            let mut manager = MemoryManager::new();
            manager.create_pool("test", 1024 * 1024 * 1024);
            
            b.iter(|| {
                let mut ids = Vec::new();
                for _ in 0..*count {
                    if let Some(id) = manager.allocate("test", 4096) {
                        ids.push(id);
                    }
                }
                for id in &ids {
                    manager.deallocate("test", id);
                }
                black_box(ids.len())
            });
        });
    }
    
    group.finish();
}

fn bench_deallocation(c: &mut Criterion) {
    let mut manager = MemoryManager::new();
    manager.create_pool("test", 1024 * 1024 * 1024);
    
    c.bench_function("deallocation", |b| {
        b.iter_batched(
            || manager.allocate("test", 4096).unwrap(),
            |id| {
                manager.deallocate("test", &id);
                black_box(id)
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_fragmentation_simulation(c: &mut Criterion) {
    c.bench_function("fragmentation_simulation", |b| {
        b.iter(|| {
            let mut pool = MemoryPool::new("test", 1024 * 1024 * 100);
            
            // Simulate fragmented allocation pattern
            let mut ids = Vec::new();
            for i in 0..1000 {
                let size = if i % 3 == 0 { 1024 } else if i % 3 == 1 { 4096 } else { 16384 };
                if let Some(id) = pool.allocate(size) {
                    ids.push(id);
                }
            }
            
            // Deallocate every other allocation
            let mut i = 0;
            ids.retain(|id| {
                i += 1;
                if i % 2 == 0 {
                    pool.deallocate(id);
                    false
                } else {
                    true
                }
            });
            
            black_box(pool.utilization())
        });
    });
}

criterion_group!(
    benches,
    bench_allocation,
    bench_pool_creation,
    bench_utilization_calculation,
    bench_concurrent_allocations,
    bench_deallocation,
    bench_fragmentation_simulation,
);

criterion_main!(benches);

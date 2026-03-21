//! Memory Gene - Adaptive memory management with Living Memory capabilities

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::gene_trait::Gene;
use super::{GeneActivation, GeneDNA, GeneError, GeneId, GeneRNA, GeneValue};

/// Memory Gene - Adaptive memory management with Living Memory capabilities
pub struct MemoryGene {
    id: GeneId,
    dna: GeneDNA,
    rna: GeneRNA,
    zones: BTreeMap<String, MemoryZone>,
    total_memory: usize,
    pressure: MemoryPressure,
    // Living Memory enhancements
    allocation_history: Vec<AllocationRecord>,
    leak_detection: LeakDetector,
    cache_manager: CacheManager,
    predictive_allocator: PredictiveAllocator,
    defragmenter: Defragmenter,
}

/// Memory zone
#[derive(Debug, Clone)]
pub struct MemoryZone {
    pub name: String,
    pub base: usize,
    pub size: usize,
    pub used: usize,
    pub zone_type: ZoneType,
    pub adaptive: bool,
    pub growth_factor: f32,
    pub fragmentation_score: f32,
    pub last_defrag: u64,
}

/// Memory zone type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneType { Kernel, User, Shared, AI, Cache }

/// Memory pressure level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPressure { None, Low, Medium, High, Critical }

/// Memory allocation result
#[derive(Debug, Clone)]
pub struct AllocationResult { pub address: usize, pub size: usize, pub zone: String }

/// Memory feedback from cells
#[derive(Debug, Clone)]
pub struct MemoryFeedback {
    pub cell_id: String, pub pressure: f32, pub recommendation: String, pub timestamp: u64,
}

/// Allocation record for leak detection
#[derive(Debug, Clone)]
pub struct AllocationRecord {
    pub address: usize,
    pub size: usize,
    pub zone: String,
    pub timestamp: u64,
    pub freed: bool,
    pub access_count: u32,
    pub last_access: u64,
}

/// Leak detector for identifying memory leaks
#[derive(Debug, Clone, Default)]
pub struct LeakDetector {
    pub suspected_leaks: Vec<LeakCandidate>,
    pub detection_threshold_ms: u64,
    pub min_leak_size: usize,
    pub total_leaked_bytes: usize,
}

/// Potential memory leak candidate
#[derive(Debug, Clone)]
pub struct LeakCandidate {
    pub address: usize,
    pub size: usize,
    pub zone: String,
    pub allocation_time: u64,
    pub last_access: u64,
    pub confidence: f32,
}

/// Cache manager with LRU/LFU eviction
#[derive(Debug, Clone)]
pub struct CacheManager {
    pub cache_entries: Vec<CacheEntry>,
    pub max_cache_size: usize,
    pub current_cache_size: usize,
    pub hit_rate: f32,
    pub eviction_policy: EvictionPolicy,
}

impl Default for CacheManager {
    fn default() -> Self {
        Self {
            cache_entries: Vec::new(),
            max_cache_size: 0,
            current_cache_size: 0,
            hit_rate: 0.0,
            eviction_policy: EvictionPolicy::Adaptive,
        }
    }
}

/// Cache entry
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: String,
    pub data_ptr: usize,
    pub size: usize,
    pub access_count: u32,
    pub last_access: u64,
    pub priority: CachePriority,
}

/// Cache eviction policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy { LRU, LFU, Adaptive }

/// Cache priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachePriority { Low, Normal, High, Critical }

/// Predictive allocator for proactive memory management
#[derive(Debug, Clone, Default)]
pub struct PredictiveAllocator {
    pub allocation_patterns: Vec<AllocationPattern>,
    pub prediction_window_ms: u64,
    pub prediction_accuracy: f32,
    pub preallocated_pools: Vec<PreallocatedPool>,
}

impl PredictiveAllocator {
    /// Find a suitable preallocated pool for the given size
    pub fn find_suitable_pool(&self, size: usize) -> Option<&PreallocatedPool> {
        self.preallocated_pools.iter()
            .find(|p| p.size - p.used >= size)
    }
}

/// Learned allocation pattern
#[derive(Debug, Clone)]
pub struct AllocationPattern {
    pub pattern_id: String,
    pub typical_sizes: Vec<usize>,
    pub typical_times: Vec<u64>,
    pub frequency: f32,
    pub confidence: f32,
}

/// Preallocated memory pool for predicted needs
#[derive(Debug, Clone)]
pub struct PreallocatedPool {
    pub pool_id: String,
    pub base_address: usize,
    pub size: usize,
    pub used: usize,
    pub purpose: String,
}

/// Defragmenter for memory optimization
#[derive(Debug, Clone, Default)]
pub struct Defragmenter {
    pub defrag_threshold: f32,
    pub last_defrag_time: u64,
    pub defrag_count: u64,
    pub bytes_recovered: usize,
    pub auto_defrag_enabled: bool,
}

impl MemoryGene {
    /// Create a new MemoryGene with Living Memory capabilities
    pub fn new(total_memory: usize) -> Self {
        let mut config = BTreeMap::new();
        config.insert(String::from("total_memory_bytes"), GeneValue::Integer(total_memory as i64));
        config.insert(String::from("pressure_threshold_low"), GeneValue::Float(0.6));
        config.insert(String::from("pressure_threshold_high"), GeneValue::Float(0.85));
        config.insert(String::from("enable_adaptive"), GeneValue::Boolean(true));
        config.insert(String::from("enable_leak_detection"), GeneValue::Boolean(true));
        config.insert(String::from("enable_predictive"), GeneValue::Boolean(true));
        config.insert(String::from("enable_defrag"), GeneValue::Boolean(true));

        let mut gene = Self {
            id: GeneId::new(),
            dna: GeneDNA { name: String::from("memory"), version: 2, config,
                activation_threshold: 0.4, critical: true },
            rna: GeneRNA::default(),
            zones: BTreeMap::new(), 
            total_memory, 
            pressure: MemoryPressure::None,
            allocation_history: Vec::new(),
            leak_detection: LeakDetector {
                suspected_leaks: Vec::new(),
                detection_threshold_ms: 60000, // 1 minute
                min_leak_size: 1024, // 1KB minimum
                total_leaked_bytes: 0,
            },
            cache_manager: CacheManager {
                cache_entries: Vec::new(),
                max_cache_size: total_memory / 8, // 12.5% for cache
                current_cache_size: 0,
                hit_rate: 0.0,
                eviction_policy: EvictionPolicy::Adaptive,
            },
            predictive_allocator: PredictiveAllocator {
                allocation_patterns: Vec::new(),
                prediction_window_ms: 5000,
                prediction_accuracy: 0.0,
                preallocated_pools: Vec::new(),
            },
            defragmenter: Defragmenter {
                defrag_threshold: 0.3, // 30% fragmentation
                last_defrag_time: 0,
                defrag_count: 0,
                bytes_recovered: 0,
                auto_defrag_enabled: true,
            },
        };
        gene.initialize_zones();
        gene
    }

    fn initialize_zones(&mut self) {
        let kernel_size = self.total_memory / 16;
        let user_size = self.total_memory / 2;
        let shared_size = self.total_memory / 8;
        let ai_size = self.total_memory / 4;
        let cache_size = self.total_memory - kernel_size - user_size - shared_size - ai_size;
        let mut base = 0;
        let zones = vec![
            ("kernel", kernel_size, ZoneType::Kernel, false),
            ("user", user_size, ZoneType::User, true),
            ("shared", shared_size, ZoneType::Shared, true),
            ("ai", ai_size, ZoneType::AI, true),
            ("cache", cache_size, ZoneType::Cache, true),
        ];
        for (name, size, zone_type, adaptive) in zones {
            self.zones.insert(String::from(name), MemoryZone {
                name: String::from(name), base, size, used: 0, zone_type, adaptive, 
                growth_factor: 1.0, fragmentation_score: 0.0, last_defrag: 0,
            });
            base += size;
        }
    }

    /// Allocate memory with Living Memory tracking
    pub fn allocate(&mut self, size: usize, zone_name: &str) -> Result<AllocationResult, GeneError> {
        // Try predictive allocation first
        if let Some(pool) = self.predictive_allocator.find_suitable_pool(size) {
            return self.allocate_from_pool(size, zone_name, &pool.pool_id);
        }

        if let Some(zone) = self.zones.get_mut(zone_name) {
            if zone.used + size > zone.size {
                if zone.adaptive && self.can_grow_zone(zone_name, size) { 
                    self.grow_zone(zone_name, size); 
                }
                else { 
                    // Try to recover memory before failing
                    self.attempt_memory_recovery(zone_name, size);
                    if zone.used + size > zone.size {
                        return Err(GeneError::ResourceUnavailable(alloc::format!("Zone {} exhausted", zone_name))); 
                    }
                }
            }
            let address = zone.base + zone.used;
            zone.used += size;
            
            // Update fragmentation score
            zone.fragmentation_score = self.calculate_fragmentation(zone_name);
            
            // Track allocation for leak detection
            self.allocation_history.push(AllocationRecord {
                address,
                size,
                zone: String::from(zone_name),
                timestamp: 0, // Would be actual timestamp
                freed: false,
                access_count: 0,
                last_access: 0,
            });
            
            // Learn allocation pattern
            self.learn_allocation_pattern(size, zone_name);
            
            self.update_pressure();
            self.rna.activation_count += 1;
            Ok(AllocationResult { address, size, zone: String::from(zone_name) })
        } else { Err(GeneError::NotFound(format!("Zone {}", zone_name))) }
    }

    /// Free memory with leak tracking
    pub fn free(&mut self, address: usize, zone_name: &str, size: usize) -> Result<(), GeneError> {
        if let Some(zone) = self.zones.get_mut(zone_name) {
            zone.used = zone.used.saturating_sub(size);
            zone.fragmentation_score = self.calculate_fragmentation(zone_name);
            
            // Mark allocation as freed
            for record in &mut self.allocation_history {
                if record.address == address && !record.freed {
                    record.freed = true;
                    break;
                }
            }
            
            self.update_pressure();
            Ok(())
        } else { Err(GeneError::NotFound(format!("Zone {}", zone_name))) }
    }

    fn can_grow_zone(&self, _zone_name: &str, required: usize) -> bool {
        if !self.dna.config.get("enable_adaptive").and_then(|v| v.as_bool()).unwrap_or(false) { return false; }
        self.zones.get("cache").map(|z| z.size - z.used).unwrap_or(0) >= required
    }

    fn grow_zone(&mut self, zone_name: &str, additional: usize) {
        if let Some(cache) = self.zones.get_mut("cache") { if cache.size >= additional { cache.size -= additional; } }
        if let Some(zone) = self.zones.get_mut(zone_name) {
            zone.size += additional;
            zone.growth_factor = zone.size as f32 / (zone.size - additional) as f32;
        }
    }

    fn update_pressure(&mut self) {
        let used: usize = self.zones.values().map(|z| z.used).sum();
        let ratio = used as f32 / self.total_memory as f32;
        self.pressure = if ratio > 0.95 { MemoryPressure::Critical }
            else if ratio > 0.85 { MemoryPressure::High }
            else if ratio > 0.6 { MemoryPressure::Medium }
            else if ratio > 0.4 { MemoryPressure::Low }
            else { MemoryPressure::None };
        self.rna.activity = ratio;
    }

    /// Detect potential memory leaks
    pub fn detect_leaks(&mut self, current_time: u64) -> Vec<LeakCandidate> {
        let mut new_leaks = Vec::new();
        
        for record in &self.allocation_history {
            if !record.freed && 
               current_time - record.timestamp > self.leak_detection.detection_threshold_ms &&
               record.access_count == 0 &&
               record.size >= self.leak_detection.min_leak_size {
                
                let confidence = if current_time - record.last_access > self.leak_detection.detection_threshold_ms * 2 {
                    0.9
                } else if record.access_count == 0 {
                    0.7
                } else {
                    0.5
                };
                
                new_leaks.push(LeakCandidate {
                    address: record.address,
                    size: record.size,
                    zone: record.zone.clone(),
                    allocation_time: record.timestamp,
                    last_access: record.last_access,
                    confidence,
                });
            }
        }
        
        self.leak_detection.suspected_leaks = new_leaks.clone();
        self.leak_detection.total_leaked_bytes = new_leaks.iter().map(|l| l.size).sum();
        
        new_leaks
    }

    /// Self-healing: Attempt to recover leaked memory
    pub fn heal_leaks(&mut self) -> usize {
        let mut recovered = 0;
        let leaks_to_heal: Vec<_> = self.leak_detection.suspected_leaks
            .iter()
            .filter(|l| l.confidence > 0.8)
            .collect();
        
        for leak in leaks_to_heal {
            // Mark as freed (in real implementation, would actually free)
            for record in &mut self.allocation_history {
                if record.address == leak.address && !record.freed {
                    record.freed = true;
                    recovered += record.size;
                    
                    // Update zone usage
                    if let Some(zone) = self.zones.get_mut(&record.zone) {
                        zone.used = zone.used.saturating_sub(record.size);
                    }
                    break;
                }
            }
        }
        
        self.leak_detection.total_leaked_bytes = self.leak_detection.total_leaked_bytes.saturating_sub(recovered);
        self.update_pressure();
        recovered
    }

    /// Calculate fragmentation score for a zone
    fn calculate_fragmentation(&self, zone_name: &str) -> f32 {
        if let Some(zone) = self.zones.get(zone_name) {
            let allocations: Vec<_> = self.allocation_history
                .iter()
                .filter(|r| r.zone == zone_name && !r.freed)
                .collect();
            
            if allocations.len() < 2 {
                return 0.0;
            }
            
            // Simple fragmentation metric based on allocation gaps
            let mut gaps = 0;
            let mut prev_end = zone.base;
            
            for alloc in allocations.iter() {
                if alloc.address > prev_end {
                    gaps += 1;
                }
                prev_end = alloc.address + alloc.size;
            }
            
            gaps as f32 / allocations.len() as f32
        } else {
            0.0
        }
    }

    /// Run defragmentation if needed
    pub fn defragment(&mut self, zone_name: &str, current_time: u64) -> usize {
        if !self.defragmenter.auto_defrag_enabled {
            return 0;
        }
        
        let fragmentation = self.calculate_fragmentation(zone_name);
        if fragmentation < self.defragmenter.defrag_threshold {
            return 0;
        }
        
        let mut recovered = 0;
        
        // Compact allocations (simplified - would move memory in real implementation)
        if let Some(zone) = self.zones.get_mut(zone_name) {
            let allocations: Vec<_> = self.allocation_history
                .iter_mut()
                .filter(|r| r.zone == zone_name && !r.freed)
                .collect();
            
            let mut new_base = zone.base;
            for alloc in allocations {
                let gap = alloc.address - new_base;
                recovered += gap;
                alloc.address = new_base;
                new_base += alloc.size;
            }
            
            zone.fragmentation_score = 0.0;
            zone.last_defrag = current_time;
        }
        
        self.defragmenter.defrag_count += 1;
        self.defragmenter.last_defrag_time = current_time;
        self.defragmenter.bytes_recovered += recovered;
        
        recovered
    }

    /// Learn allocation pattern for prediction
    fn learn_allocation_pattern(&mut self, size: usize, zone: &str) {
        let pattern_id = alloc::format!("{}_{}b", zone, size / 1024);
        
        if let Some(pattern) = self.predictive_allocator.allocation_patterns.iter_mut()
            .find(|p| p.pattern_id == pattern_id) {
            pattern.frequency += 0.1;
            pattern.confidence = (pattern.confidence + 0.05).min(1.0);
        } else {
            self.predictive_allocator.allocation_patterns.push(AllocationPattern {
                pattern_id,
                typical_sizes: vec![size],
                typical_times: vec![0], // Would be actual time
                frequency: 0.1,
                confidence: 0.1,
            });
        }
    }

    /// Attempt to recover memory when allocation would fail
    fn attempt_memory_recovery(&mut self, zone_name: &str, required: usize) {
        // Try defragmentation first
        let recovered = self.defragment(zone_name, 0);
        if recovered >= required {
            return;
        }
        
        // Try healing leaks
        let leak_recovered = self.heal_leaks();
        if leak_recovered >= required - recovered {
            return;
        }
        
        // Try cache eviction
        self.evict_cache(required - recovered - leak_recovered);
    }

    /// Evict cache entries to free memory
    fn evict_cache(&mut self, required: usize) -> usize {
        let mut evicted = 0;
        
        match self.cache_manager.eviction_policy {
            EvictionPolicy::LRU => {
                // Sort by last access time
                self.cache_manager.cache_entries.sort_by_key(|e| e.last_access);
            }
            EvictionPolicy::LFU => {
                // Sort by access count
                self.cache_manager.cache_entries.sort_by_key(|e| e.access_count);
            }
            EvictionPolicy::Adaptive => {
                // Consider both factors and priority
                self.cache_manager.cache_entries.sort_by(|a, b| {
                    let score_a = a.access_count as i32 - (a.priority as i32 * 10);
                    let score_b = b.access_count as i32 - (b.priority as i32 * 10);
                    score_a.cmp(&score_b)
                });
            }
        }
        
        while evicted < required && !self.cache_manager.cache_entries.is_empty() {
            if let Some(entry) = self.cache_manager.cache_entries.pop() {
                evicted += entry.size;
                self.cache_manager.current_cache_size -= entry.size;
            }
        }
        
        evicted
    }

    /// Allocate from preallocated pool
    fn allocate_from_pool(&mut self, size: usize, zone_name: &str, pool_id: &str) -> Result<AllocationResult, GeneError> {
        if let Some(pool) = self.predictive_allocator.preallocated_pools.iter_mut()
            .find(|p| p.pool_id == pool_id && p.size - p.used >= size) {
            
            let address = pool.base_address + pool.used;
            pool.used += size;
            
            return Ok(AllocationResult {
                address,
                size,
                zone: String::from(zone_name),
            });
        }
        
        // Fall back to regular allocation
        self.allocate(size, zone_name)
    }

    /// Process feedback from MemoryCell
    pub fn process_feedback(&mut self, feedback: &MemoryFeedback) {
        if feedback.pressure > 0.8 {
            if let Some(user_zone) = self.zones.get_mut("user") {
                if user_zone.adaptive { user_zone.growth_factor *= 1.1; }
            }
        }
        self.update_pressure();
    }

    /// Get memory statistics
    pub fn stats(&self) -> MemoryStats {
        let total_used: usize = self.zones.values().map(|z| z.used).sum();
        MemoryStats {
            total: self.total_memory, used: total_used,
            available: self.total_memory - total_used, pressure: self.pressure,
            zones: self.zones.values().cloned().collect(),
        }
    }

    /// Get Living Memory statistics
    pub fn living_stats(&self) -> LivingMemoryStats {
        LivingMemoryStats {
            total_allocations: self.allocation_history.len(),
            active_allocations: self.allocation_history.iter().filter(|r| !r.freed).count(),
            suspected_leaks: self.leak_detection.suspected_leaks.len(),
            leaked_bytes: self.leak_detection.total_leaked_bytes,
            cache_entries: self.cache_manager.cache_entries.len(),
            cache_hit_rate: self.cache_manager.hit_rate,
            cache_size: self.cache_manager.current_cache_size,
            learned_patterns: self.predictive_allocator.allocation_patterns.len(),
            prediction_accuracy: self.predictive_allocator.prediction_accuracy,
            defrag_count: self.defragmenter.defrag_count,
            bytes_recovered: self.defragmenter.bytes_recovered,
        }
    }

    pub fn get_zone(&self, name: &str) -> Option<&MemoryZone> { self.zones.get(name) }
}

/// Memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total: usize, pub used: usize, pub available: usize,
    pub pressure: MemoryPressure, pub zones: Vec<MemoryZone>,
}

/// Living Memory statistics
#[derive(Debug, Clone)]
pub struct LivingMemoryStats {
    pub total_allocations: usize,
    pub active_allocations: usize,
    pub suspected_leaks: usize,
    pub leaked_bytes: usize,
    pub cache_entries: usize,
    pub cache_hit_rate: f32,
    pub cache_size: usize,
    pub learned_patterns: usize,
    pub prediction_accuracy: f32,
    pub defrag_count: u64,
    pub bytes_recovered: usize,
}

impl Gene for MemoryGene {
    fn id(&self) -> GeneId { self.id }
    fn name(&self) -> &str { &self.dna.name }
    fn dna(&self) -> &GeneDNA { &self.dna }
    fn dna_mut(&mut self) -> &mut GeneDNA { &mut self.dna }
    fn rna(&self) -> &GeneRNA { &self.rna }
    fn rna_mut(&mut self) -> &mut GeneRNA { &mut self.rna }

    fn activate(&mut self, input: Option<&GeneValue>) -> Result<GeneActivation, GeneError> {
        self.rna.activity = 1.0;
        self.rna.activation_count += 1;
        let result = input.and_then(|v| v.as_string()).map(|s| {
            match s.as_str() {
                "stats" => {
                    let stats = self.stats();
                    GeneValue::String(alloc::format!("total:{} used:{} pressure:{:?}", stats.total, stats.used, stats.pressure))
                }
                "pressure" => GeneValue::String(alloc::format!("{:?}", self.pressure)),
                "living_stats" => {
                    let stats = self.living_stats();
                    GeneValue::String(alloc::format!(
                        "allocations:{} leaks:{} cache_hit:{:.2}% patterns:{} defrag:{}",
                        stats.active_allocations, stats.suspected_leaks, 
                        stats.cache_hit_rate * 100.0, stats.learned_patterns, stats.defrag_count
                    ))
                }
                "detect_leaks" => {
                    let leaks = self.detect_leaks(0); // Would use actual time
                    GeneValue::String(alloc::format!("detected {} potential leaks", leaks.len()))
                }
                "heal_leaks" => {
                    let recovered = self.heal_leaks();
                    GeneValue::String(alloc::format!("recovered {} bytes from leaks", recovered))
                }
                "defrag" => {
                    let recovered = self.defragment("user", 0); // Would use actual time
                    GeneValue::String(alloc::format!("defrag recovered {} bytes", recovered))
                }
                _ => GeneValue::String(String::from("unknown command")),
            }
        });
        Ok(GeneActivation { activated: true, result, effects: Vec::new() })
    }

    fn update(&mut self, delta_ms: u64) -> Result<(), GeneError> {
        self.update_pressure();
        self.rna.activity *= 0.9;
        
        // Living Memory periodic operations
        // Auto-defragment high fragmentation zones
        for (zone_name, zone) in &self.zones.clone() {
            if zone.fragmentation_score > self.defragmenter.defrag_threshold {
                self.defragment(zone_name, delta_ms);
            }
        }
        
        // Periodic leak detection (every 60 seconds simulated)
        if delta_ms % 60000 < 100 {
            self.detect_leaks(delta_ms);
            
            // Auto-heal high confidence leaks
            let high_confidence_leaks: Vec<_> = self.leak_detection.suspected_leaks
                .iter()
                .filter(|l| l.confidence > 0.9)
                .collect();
            
            if high_confidence_leaks.len() > 3 {
                self.heal_leaks();
            }
        }
        
        // Update prediction accuracy based on cache hits
        if !self.cache_manager.cache_entries.is_empty() {
            let hits = self.cache_manager.cache_entries.iter()
                .map(|e| e.access_count)
                .sum::<u32>();
            let total = self.allocation_history.len() as u32;
            if total > 0 {
                self.cache_manager.hit_rate = hits as f32 / total as f32;
            }
        }
        
        Ok(())
    }
}

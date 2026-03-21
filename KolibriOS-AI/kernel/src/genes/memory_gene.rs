//! Memory Gene - Adaptive memory management

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::gene_trait::Gene;
use super::{GeneActivation, GeneDNA, GeneError, GeneId, GeneRNA, GeneValue};

/// Memory Gene - Adaptive memory management
pub struct MemoryGene {
    id: GeneId,
    dna: GeneDNA,
    rna: GeneRNA,
    zones: BTreeMap<String, MemoryZone>,
    total_memory: usize,
    pressure: MemoryPressure,
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

impl MemoryGene {
    /// Create a new MemoryGene
    pub fn new(total_memory: usize) -> Self {
        let mut config = BTreeMap::new();
        config.insert(String::from("total_memory_bytes"), GeneValue::Integer(total_memory as i64));
        config.insert(String::from("pressure_threshold_low"), GeneValue::Float(0.6));
        config.insert(String::from("pressure_threshold_high"), GeneValue::Float(0.85));
        config.insert(String::from("enable_adaptive"), GeneValue::Boolean(true));

        let mut gene = Self {
            id: GeneId::new(),
            dna: GeneDNA { name: String::from("memory"), version: 1, config,
                activation_threshold: 0.4, critical: true },
            rna: GeneRNA::default(),
            zones: BTreeMap::new(), total_memory, pressure: MemoryPressure::None,
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
                name: String::from(name), base, size, used: 0, zone_type, adaptive, growth_factor: 1.0,
            });
            base += size;
        }
    }

    /// Allocate memory
    pub fn allocate(&mut self, size: usize, zone_name: &str) -> Result<AllocationResult, GeneError> {
        if let Some(zone) = self.zones.get_mut(zone_name) {
            if zone.used + size > zone.size {
                if zone.adaptive && self.can_grow_zone(zone_name, size) { self.grow_zone(zone_name, size); }
                else { return Err(GeneError::ResourceUnavailable(alloc::format!("Zone {} exhausted", zone_name))); }
            }
            let address = zone.base + zone.used;
            zone.used += size;
            self.update_pressure();
            self.rna.activation_count += 1;
            Ok(AllocationResult { address, size, zone: String::from(zone_name) })
        } else { Err(GeneError::NotFound(format!("Zone {}", zone_name))) }
    }

    /// Free memory
    pub fn free(&mut self, _address: usize, zone_name: &str, size: usize) -> Result<(), GeneError> {
        if let Some(zone) = self.zones.get_mut(zone_name) {
            zone.used = zone.used.saturating_sub(size);
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

    pub fn get_zone(&self, name: &str) -> Option<&MemoryZone> { self.zones.get(name) }
}

/// Memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total: usize, pub used: usize, pub available: usize,
    pub pressure: MemoryPressure, pub zones: Vec<MemoryZone>,
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
                _ => GeneValue::String(String::from("unknown command")),
            }
        });
        Ok(GeneActivation { activated: true, result, effects: Vec::new() })
    }

    fn update(&mut self, _delta_ms: u64) -> Result<(), GeneError> {
        self.update_pressure();
        self.rna.activity *= 0.9;
        Ok(())
    }
}

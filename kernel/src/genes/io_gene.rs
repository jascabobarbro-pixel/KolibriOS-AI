//! I/O Gene - Device and I/O management

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::gene_trait::Gene;
use super::{GeneActivation, GeneDNA, GeneError, GeneId, GeneRNA, GeneValue};

/// I/O Gene
pub struct IOGene {
    id: GeneId, dna: GeneDNA, rna: GeneRNA,
    devices: BTreeMap<u32, Device>, pending_ops: Vec<IoOperation>, next_device_id: u32,
}

#[derive(Debug, Clone)]
pub struct Device {
    pub id: u32, pub name: String, pub device_type: DeviceType, pub state: DeviceState,
    pub driver: Option<String>, pub irq: Option<u8>, pub base_address: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType { Block, Character, Network, Usb, Gpu, Audio, Input }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState { Offline, Initializing, Ready, Busy, Error }

#[derive(Debug, Clone)]
pub struct IoOperation {
    pub id: u64, pub device_id: u32, pub op_type: IoOpType, pub status: IoStatus,
    pub buffer: Option<usize>, pub size: usize, pub offset: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoOpType { Read, Write, Seek, Flush, Ioctl }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoStatus { Pending, InProgress, Completed, Failed }

impl IOGene {
    pub fn new() -> Self {
        let mut config = BTreeMap::new();
        config.insert(String::from("max_pending_ops"), GeneValue::Integer(256));
        config.insert(String::from("enable_async"), GeneValue::Boolean(true));
        config.insert(String::from("default_timeout_ms"), GeneValue::Integer(5000));
        Self {
            id: GeneId::new(),
            dna: GeneDNA { name: String::from("io"), version: 1, config,
                activation_threshold: 0.2, critical: true },
            rna: GeneRNA::default(),
            devices: BTreeMap::new(), pending_ops: Vec::new(), next_device_id: 1,
        }
    }

    pub fn register_device(&mut self, name: &str, device_type: DeviceType, irq: Option<u8>, base_address: Option<usize>) -> u32 {
        let id = self.next_device_id; self.next_device_id += 1;
        let device = Device { id, name: String::from(name), device_type, state: DeviceState::Initializing,
            driver: None, irq, base_address };
        self.devices.insert(id, device); id
    }

    pub fn get_device(&self, id: u32) -> Option<&Device> { self.devices.get(&id) }

    pub fn set_device_state(&mut self, id: u32, state: DeviceState) -> Result<(), GeneError> {
        if let Some(device) = self.devices.get_mut(&id) { device.state = state; Ok(()) }
        else { Err(GeneError::NotFound(format!("Device {}", id))) }
    }

    pub fn submit_io(&mut self, device_id: u32, op_type: IoOpType, buffer: Option<usize>, size: usize, offset: u64) -> Result<u64, GeneError> {
        if !self.devices.contains_key(&device_id) { return Err(GeneError::NotFound(format!("Device {}", device_id))); }
        let max_ops = self.dna.config.get("max_pending_ops").and_then(|v| v.as_integer()).unwrap_or(256) as usize;
        if self.pending_ops.len() >= max_ops {
            return Err(GeneError::ResourceUnavailable(String::from("I/O queue full")));
        }
        let id = self.rna.activation_count as u64; self.rna.activation_count += 1;
        self.pending_ops.push(IoOperation { id, device_id, op_type, status: IoStatus::Pending, buffer, size, offset });
        Ok(id)
    }

    pub fn process_pending(&mut self) -> Vec<IoOperation> {
        let mut completed = Vec::new(); let mut remaining = Vec::new();
        for mut op in self.pending_ops.drain(..) {
            match op.status {
                IoStatus::Pending => { op.status = IoStatus::InProgress; op.status = IoStatus::Completed; completed.push(op); }
                IoStatus::InProgress => remaining.push(op),
                _ => completed.push(op),
            }
        }
        self.pending_ops = remaining; completed
    }

    pub fn list_devices(&self) -> Vec<&Device> { self.devices.values().collect() }
    pub fn devices_by_type(&self, device_type: DeviceType) -> Vec<&Device> {
        self.devices.values().filter(|d| d.device_type == device_type).collect()
    }
    pub fn handle_interrupt(&mut self, irq: u8) -> Result<(), GeneError> {
        if let Some(device) = self.devices.values_mut().find(|d| d.irq == Some(irq)) {
            device.state = DeviceState::Ready; self.rna.activation_count += 1; Ok(())
        } else { Err(GeneError::NotFound(format!("No device for IRQ {}", irq))) }
    }
}

impl Default for IOGene { fn default() -> Self { Self::new() } }

impl Gene for IOGene {
    fn id(&self) -> GeneId { self.id }
    fn name(&self) -> &str { &self.dna.name }
    fn dna(&self) -> &GeneDNA { &self.dna }
    fn dna_mut(&mut self) -> &mut GeneDNA { &mut self.dna }
    fn rna(&self) -> &GeneRNA { &self.rna }
    fn rna_mut(&mut self) -> &mut GeneRNA { &mut self.rna }

    fn activate(&mut self, input: Option<&GeneValue>) -> Result<GeneActivation, GeneError> {
        self.rna.activity = 1.0; self.rna.activation_count += 1;
        let result = input.and_then(|v| v.as_string()).map(|s| {
            match s.as_str() {
                "devices" => {
                    let devices: Vec<String> = self.devices.values().map(|d| format!("{}({})", d.name, d.id)).collect();
                    GeneValue::String(devices.join(","))
                }
                "pending" => GeneValue::String(alloc::format!("{} pending operations", self.pending_ops.len())),
                _ => GeneValue::String(String::from("unknown command")),
            }
        });
        Ok(GeneActivation { activated: true, result, effects: Vec::new() })
    }

    fn update(&mut self, _delta_ms: u64) -> Result<(), GeneError> {
        self.process_pending(); self.rna.activity *= 0.95; Ok(())
    }
}

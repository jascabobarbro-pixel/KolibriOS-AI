//! Garbage Collector for Koli Runtime

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::value::Value;

/// Garbage Collector
pub struct GarbageCollector {
    heap: Vec<GcObject>,
    free_list: Vec<usize>,
    total_allocated: usize,
    threshold: usize,
}

/// Garbage-collected object
pub struct GcObject {
    pub value: Value,
    pub marked: bool,
    pub forward_addr: Option<usize>,
}

impl GarbageCollector {
    /// Create a new GC
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            free_list: Vec::new(),
            total_allocated: 0,
            threshold: 1024 * 1024, // 1 MB
        }
    }

    /// Allocate a new object
    pub fn alloc(&mut self, value: Value) -> GcRef {
        if self.total_allocated > self.threshold {
            self.collect();
        }

        let obj = GcObject {
            value,
            marked: false,
            forward_addr: None,
        };

        if let Some(idx) = self.free_list.pop() {
            self.heap[idx] = obj;
            GcRef(idx)
        } else {
            let idx = self.heap.len();
            self.heap.push(obj);
            self.total_allocated += 1;
            GcRef(idx)
        }
    }

    /// Get a reference to an object
    pub fn get(&self, reference: GcRef) -> Option<&Value> {
        self.heap.get(reference.0).map(|obj| &obj.value)
    }

    /// Get a mutable reference to an object
    pub fn get_mut(&mut self, reference: GcRef) -> Option<&mut Value> {
        self.heap.get_mut(reference.0).map(|obj| &mut obj.value)
    }

    /// Run garbage collection
    pub fn collect(&mut self) {
        // Mark phase
        self.mark_all();

        // Sweep phase
        self.sweep();
    }

    /// Mark all reachable objects
    fn mark_all(&mut self) {
        // In a real implementation, this would trace from roots
        // (stack, globals, etc.) to mark reachable objects
    }

    /// Sweep unmarked objects
    fn sweep(&mut self) {
        for (idx, obj) in self.heap.iter_mut().enumerate() {
            if obj.marked {
                obj.marked = false;
            } else {
                obj.value = Value::Nil;
                self.free_list.push(idx);
            }
        }
    }

    /// Get total allocated bytes
    pub fn used(&self) -> usize {
        self.total_allocated
    }

    /// Get number of allocations
    pub fn allocations(&self) -> usize {
        self.heap.len() - self.free_list.len()
    }
}

impl Default for GarbageCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Reference to a GC object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GcRef(pub usize);

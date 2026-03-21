//! Memory Store for AI Agent

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::AgentError;

/// Memory store
pub struct MemoryStore {
    short_term: BTreeMap<String, String>,
    long_term: Vec<MemoryEntry>,
    facts: BTreeMap<String, Fact>,
}

/// Memory entry
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub timestamp: u64,
    pub importance: f32,
    pub access_count: u32,
}

/// Fact
pub struct Fact {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f32,
}

impl MemoryStore {
    /// Create a new memory store
    pub fn new() -> Self {
        Self {
            short_term: BTreeMap::new(),
            long_term: Vec::new(),
            facts: BTreeMap::new(),
        }
    }

    /// Store in short-term memory
    pub fn store_short_term(&mut self, key: &str, value: &str) {
        self.short_term.insert(String::from(key), String::from(value));
    }

    /// Retrieve from short-term memory
    pub fn get_short_term(&self, key: &str) -> Option<&String> {
        self.short_term.get(key)
    }

    /// Store in long-term memory
    pub fn store_long_term(&mut self, key: &str, value: &str, importance: f32) {
        self.long_term.push(MemoryEntry {
            key: String::from(key),
            value: String::from(value),
            timestamp: 0,
            importance,
            access_count: 0,
        });
    }

    /// Search long-term memory
    pub fn search_long_term(&self, query: &str) -> Vec<&MemoryEntry> {
        self.long_term
            .iter()
            .filter(|e| e.key.contains(query) || e.value.contains(query))
            .collect()
    }

    /// Add a fact
    pub fn add_fact(&mut self, subject: &str, predicate: &str, object: &str, confidence: f32) {
        let key = alloc::format!("{}:{}", subject, predicate);
        self.facts.insert(key, Fact {
            subject: String::from(subject),
            predicate: String::from(predicate),
            object: String::from(object),
            confidence,
        });
    }

    /// Query facts
    pub fn query_facts(&self, subject: Option<&str>, predicate: Option<&str>) -> Vec<&Fact> {
        self.facts.values()
            .filter(|f| {
                subject.map_or(true, |s| f.subject == s) &&
                predicate.map_or(true, |p| f.predicate == p)
            })
            .collect()
    }

    /// Persist memory to storage
    pub fn store(&self) -> Result<(), AgentError> {
        // In a real implementation, this would write to disk
        Ok(())
    }

    /// Load memory from storage
    pub fn load(&mut self) -> Result<(), AgentError> {
        // In a real implementation, this would read from disk
        Ok(())
    }

    /// Clear all memory
    pub fn clear(&mut self) {
        self.short_term.clear();
        self.long_term.clear();
        self.facts.clear();
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

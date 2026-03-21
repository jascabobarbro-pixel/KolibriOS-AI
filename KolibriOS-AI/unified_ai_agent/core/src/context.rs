//! Context Management for AI Agent

use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;

use super::agent::Role;

/// Context manager
pub struct ContextManager {
    messages: VecDeque<Message>,
    max_messages: usize,
    summary: Option<String>,
}

/// Message in context
pub struct Message {
    pub role: Role,
    pub content: String,
    pub timestamp: u64,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages: 100,
            summary: None,
        }
    }

    /// Add a message to context
    pub fn add_message(&mut self, role: Role, content: &str) {
        self.messages.push_back(Message {
            role,
            content: String::from(content),
            timestamp: 0, // Would be actual timestamp
        });

        // Trim if over limit
        while self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }
    }

    /// Get all messages
    pub fn messages(&self) -> &VecDeque<Message> {
        &self.messages
    }

    /// Get messages by role
    pub fn get_by_role(&self, role: Role) -> Vec<&Message> {
        self.messages.iter().filter(|m| m.role == role).collect()
    }

    /// Get recent messages
    pub fn recent(&self, count: usize) -> Vec<&Message> {
        self.messages.iter().rev().take(count).rev().collect()
    }

    /// Clear context
    pub fn clear(&mut self) {
        self.messages.clear();
        self.summary = None;
    }

    /// Generate summary
    pub fn summarize(&mut self) -> String {
        // In a real implementation, this would use AI to generate summary
        let summary = alloc::format!("Conversation with {} messages", self.messages.len());
        self.summary = Some(summary.clone());
        summary
    }

    /// Get token count (approximate)
    pub fn token_count(&self) -> usize {
        self.messages.iter().map(|m| m.content.len() / 4).sum()
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

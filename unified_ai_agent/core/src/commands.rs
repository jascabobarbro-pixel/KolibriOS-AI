//! Command Registry for AI Agent

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::agent::AgentResponse;
use super::AgentError;

/// Command handler type
pub type CommandHandler = fn(Vec<String>) -> Result<AgentResponse, AgentError>;

/// Command registry
pub struct CommandRegistry {
    commands: BTreeMap<String, CommandInfo>,
}

/// Command information
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub handler: CommandHandler,
    pub aliases: Vec<String>,
}

impl CommandRegistry {
    /// Create a new command registry
    pub fn new() -> Self {
        Self {
            commands: BTreeMap::new(),
        }
    }

    /// Register a command
    pub fn register(&mut self, info: CommandInfo) {
        self.commands.insert(info.name.clone(), info);
    }

    /// Get a command handler
    pub fn get(&self, name: &str) -> Option<CommandHandler> {
        // Check direct name
        if let Some(info) = self.commands.get(name) {
            return Some(info.handler);
        }
        
        // Check aliases
        for info in self.commands.values() {
            if info.aliases.iter().any(|a| a == name) {
                return Some(info.handler);
            }
        }
        
        None
    }

    /// Get command info
    pub fn get_info(&self, name: &str) -> Option<&CommandInfo> {
        self.commands.get(name)
    }

    /// List all commands
    pub fn list(&self) -> Vec<&CommandInfo> {
        self.commands.values().collect()
    }

    /// Check if command exists
    pub fn exists(&self, name: &str) -> bool {
        self.commands.contains_key(name) || 
        self.commands.values().any(|c| c.aliases.iter().any(|a| a == name))
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

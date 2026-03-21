//! Context Management
//!
//! Manages creative context and session history.

use std::collections::VecDeque;
use std::time::Instant;

use serde::{Deserialize, Serialize};

/// Creative Context
pub struct CreativeContext {
    /// Current session
    session: CreativeSession,
    
    /// History of requests
    history: VecDeque<HistoryEntry>,
    
    /// Working memory (current project context)
    working_memory: WorkingMemory,
    
    /// Max history size
    max_history: usize,
}

/// Creative session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativeSession {
    pub id: String,
    pub started_at: Instant,
    pub project_name: Option<String>,
    pub project_type: Option<ProjectType>,
    pub active_document: Option<String>,
    pub goals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    Writing,
    VisualArt,
    Design,
    Marketing,
    Code,
    General,
}

/// History entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_type: String,
    pub prompt: String,
    pub response: String,
    pub tokens_used: u32,
    pub feedback: Option<Feedback>,
}

/// User feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub rating: u8, // 1-5
    pub accepted: bool,
    pub edited: bool,
    pub notes: Option<String>,
}

/// Working memory
#[derive(Debug, Clone, Default)]
pub struct WorkingMemory {
    /// Key concepts/keywords
    keywords: Vec<String>,
    
    /// Characters (for fiction)
    characters: Vec<CharacterInfo>,
    
    /// Style preferences
    style_preferences: StylePreferences,
    
    /// Project notes
    notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterInfo {
    pub name: String,
    pub description: String,
    pub traits: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StylePreferences {
    pub tone: Option<String>,
    pub formality: Option<String>,
    pub target_audience: Option<String>,
    pub vocabulary_level: Option<String>,
}

impl CreativeContext {
    pub fn new(max_history: usize) -> Self {
        Self {
            session: CreativeSession::new(),
            history: VecDeque::with_capacity(max_history),
            working_memory: WorkingMemory::default(),
            max_history,
        }
    }
    
    /// Add entry to history
    pub fn add_history(&mut self, entry: HistoryEntry) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(entry);
    }
    
    /// Get recent history
    pub fn recent_history(&self, count: usize) -> Vec<&HistoryEntry> {
        self.history
            .iter()
            .rev()
            .take(count)
            .collect()
    }
    
    /// Add keyword
    pub fn add_keyword(&mut self, keyword: String) {
        if !self.working_memory.keywords.contains(&keyword) {
            self.working_memory.keywords.push(keyword);
        }
    }
    
    /// Get keywords
    pub fn keywords(&self) -> &[String] {
        &self.working_memory.keywords
    }
    
    /// Add note
    pub fn add_note(&mut self, note: String) {
        self.working_memory.notes.push(note);
    }
    
    /// Get notes
    pub fn notes(&self) -> &[String] {
        &self.working_memory.notes
    }
    
    /// Update style preferences
    pub fn update_style(&mut self, prefs: StylePreferences) {
        self.working_memory.style_preferences = prefs;
    }
    
    /// Get style preferences
    pub fn style_preferences(&self) -> &StylePreferences {
        &self.working_memory.style_preferences
    }
    
    /// Get current session
    pub fn session(&self) -> &CreativeSession {
        &self.session
    }
    
    /// Start new session
    pub fn new_session(&mut self, project_name: Option<String>, project_type: Option<ProjectType>) {
        self.session = CreativeSession {
            id: uuid::Uuid::new_v4().to_string(),
            started_at: Instant::now(),
            project_name,
            project_type,
            active_document: None,
            goals: Vec::new(),
        };
        self.history.clear();
        self.working_memory = WorkingMemory::default();
    }
    
    /// Add goal
    pub fn add_goal(&mut self, goal: String) {
        self.session.goals.push(goal);
    }
    
    /// Set active document
    pub fn set_active_document(&mut self, doc: Option<String>) {
        self.session.active_document = doc;
    }
    
    /// Build context string for LLM
    pub fn build_context_string(&self) -> String {
        let mut parts = Vec::new();
        
        if let Some(name) = &self.session.project_name {
            parts.push(format!("Project: {}", name));
        }
        
        if let Some(pt) = &self.session.project_type {
            let pt_str = match pt {
                ProjectType::Writing => "Writing",
                ProjectType::VisualArt => "Visual Art",
                ProjectType::Design => "Design",
                ProjectType::Marketing => "Marketing",
                ProjectType::Code => "Code",
                ProjectType::General => "General",
            };
            parts.push(format!("Project Type: {}", pt_str));
        }
        
        if !self.working_memory.keywords.is_empty() {
            parts.push(format!("Key Concepts: {}", self.working_memory.keywords.join(", ")));
        }
        
        if !self.session.goals.is_empty() {
            parts.push(format!("Goals: {}", self.session.goals.join("; ")));
        }
        
        if let Some(tone) = &self.working_memory.style_preferences.tone {
            parts.push(format!("Tone: {}", tone));
        }
        
        parts.join("\n")
    }
}

impl CreativeSession {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            started_at: Instant::now(),
            project_name: None,
            project_type: None,
            active_document: None,
            goals: Vec::new(),
        }
    }
}

impl Default for CreativeSession {
    fn default() -> Self {
        Self::new()
    }
}

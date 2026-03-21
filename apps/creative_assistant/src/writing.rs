//! Writing Assistant Module
//!
//! Provides AI-powered writing assistance.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::debug;

use super::{WritingTask, ContentLength, ImprovementAspect};

/// Writing Assistant
pub struct WritingAssistant {
    /// Prompt templates
    templates: HashMap<String, String>,
}

/// Writing context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingContext {
    pub genre: Option<String>,
    pub target_audience: Option<String>,
    pub tone: Option<String>,
    pub keywords: Vec<String>,
    pub reference_style: Option<String>,
}

/// Writing suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingSuggestion {
    pub suggestion_type: SuggestionType,
    pub content: String,
    pub position: Option<TextPosition>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    Grammar,
    Style,
    Clarity,
    Engagement,
    Vocabulary,
    Structure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPosition {
    pub start: usize,
    pub end: usize,
}

impl WritingAssistant {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        templates.insert(
            "generate".to_string(),
            "Create {{style}} content about: {{prompt}}\n\
             Length: {{length}}\n\
             Target audience: {{audience}}\n\
             \nContent:\n".to_string()
        );
        
        templates.insert(
            "improve".to_string(),
            "Improve the following text focusing on: {{aspects}}\n\
             \nOriginal text:\n{{content}}\n\
             \nImproved version:\n".to_string()
        );
        
        templates.insert(
            "tone".to_string(),
            "Rewrite the following text with a {{tone}} tone:\n\
             \nOriginal:\n{{content}}\n\
             \nRewritten:\n".to_string()
        );
        
        templates.insert(
            "outline".to_string(),
            "Create a detailed outline for: {{topic}}\n\
             Number of sections: {{sections}}\n\
             \nOutline:\n".to_string()
        );
        
        Self { templates }
    }
    
    /// Build prompt for writing task
    pub fn build_prompt(
        &self,
        task: &WritingTask,
        content: Option<&str>,
        context: Option<&WritingContext>,
    ) -> String {
        match task {
            WritingTask::GenerateContent { prompt, style, length } => {
                let style_text = style.as_deref().unwrap_or("engaging");
                let length_text = match length {
                    ContentLength::Short => "approximately 100 words",
                    ContentLength::Medium => "approximately 300 words",
                    ContentLength::Long => "approximately 600 words",
                    ContentLength::Extended => "approximately 1200 words",
                };
                let audience = context
                    .and_then(|c| c.target_audience.as_deref())
                    .unwrap_or("general readers");
                
                format!(
                    "Create {} content about: {}\n\
                     Length: {}\n\
                     Target audience: {}\n\
                     \nContent:\n",
                    style_text, prompt, length_text, audience
                )
            }
            
            WritingTask::ImproveContent { aspects } => {
                let aspects_text: Vec<String> = aspects
                    .iter()
                    .map(|a| match a {
                        ImprovementAspect::Clarity => "clarity",
                        ImprovementAspect::Engagement => "engagement",
                        ImprovementAspect::Persuasion => "persuasion",
                        ImprovementAspect::Readability => "readability",
                        ImprovementAspect::Vocabulary => "vocabulary richness",
                        ImprovementAspect::Structure => "structure",
                    })
                    .map(|s| s.to_string())
                    .collect();
                
                format!(
                    "Improve the following text focusing on: {}\n\
                     \nOriginal text:\n{}\n\
                     \nImproved version:\n",
                    aspects_text.join(", "),
                    content.unwrap_or("")
                )
            }
            
            WritingTask::ChangeTone { target_tone } => {
                format!(
                    "Rewrite the following text with a {} tone:\n\
                     \nOriginal:\n{}\n\
                     \nRewritten:\n",
                    target_tone,
                    content.unwrap_or("")
                )
            }
            
            WritingTask::FixGrammar => {
                format!(
                    "Fix all grammar, spelling, and punctuation errors in the following text:\n\
                     \nOriginal:\n{}\n\
                     \nCorrected:\n",
                    content.unwrap_or("")
                )
            }
            
            WritingTask::GenerateOutline { topic, sections } => {
                format!(
                    "Create a detailed outline for: {}\n\
                     Number of sections: {}\n\
                     \nOutline:\n",
                    topic, sections
                )
            }
            
            WritingTask::ContinueStory { genre } => {
                format!(
                    "Continue this story in the {} genre:\n\
                     \nStory so far:\n{}\n\
                     \nContinuation:\n",
                    genre,
                    content.unwrap_or("")
                )
            }
        }
    }
    
    /// Extract suggestions from LLM response
    pub fn extract_suggestions(&self, response: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        // Look for suggestion patterns
        for line in response.lines() {
            let line_lower = line.to_lowercase();
            
            if line_lower.starts_with("suggestion:")
                || line_lower.starts_with("tip:")
                || line_lower.starts_with("note:")
                || line_lower.starts_with("recommendation:")
            {
                suggestions.push(line.trim().to_string());
            }
        }
        
        // If no explicit suggestions, extract key phrases
        if suggestions.is_empty() {
            let key_phrases = [
                "consider", "try", "avoid", "remember to",
                "make sure", "don't forget", "important to",
            ];
            
            for line in response.lines() {
                for phrase in &key_phrases {
                    if line.to_lowercase().contains(phrase) {
                        suggestions.push(line.trim().to_string());
                        break;
                    }
                }
            }
        }
        
        suggestions.truncate(5);
        suggestions
    }
    
    /// Extract alternative versions from response
    pub fn extract_alternatives(&self, response: &str) -> Vec<String> {
        let mut alternatives = Vec::new();
        
        // Look for alternative patterns
        let patterns = ["alternative:", "option:", "variation:", "or: ", "another way:"];
        
        for line in response.lines() {
            let line_lower = line.to_lowercase();
            for pattern in &patterns {
                if line_lower.starts_with(pattern) {
                    alternatives.push(line.trim().to_string());
                    break;
                }
            }
        }
        
        alternatives
    }
}

impl Default for WritingAssistant {
    fn default() -> Self {
        Self::new()
    }
}

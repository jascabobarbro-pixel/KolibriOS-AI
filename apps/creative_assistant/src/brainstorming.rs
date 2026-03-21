//! Brainstorming Module
//!
//! AI-powered creative brainstorming assistance.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::debug;

/// Brainstormer
pub struct Brainstormer {
    /// Idea templates by category
    templates: HashMap<String, Vec<String>>,
}

/// Single idea
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Idea {
    pub title: String,
    pub description: String,
    pub category: IdeaCategory,
    pub feasibility: f32,
    pub novelty: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdeaCategory {
    Innovation,
    Process,
    Product,
    Marketing,
    Technical,
    Creative,
}

/// Brainstorm session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainstormSession {
    pub id: String,
    pub topic: String,
    pub constraints: Vec<String>,
    pub ideas: Vec<Idea>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Brainstormer {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        templates.insert(
            "innovation".to_string(),
            vec![
                "What if we combined {} with {}?".to_string(),
                "How might {} work in a completely different context?".to_string(),
                "What would {} look like if money was no object?".to_string(),
            ]
        );
        
        templates.insert(
            "problem_solving".to_string(),
            vec![
                "What's the root cause of {}?".to_string(),
                "Who else has solved a similar problem to {}?".to_string(),
                "What constraints can we remove from {}?".to_string(),
            ]
        );
        
        Self { templates }
    }
    
    /// Build prompt for brainstorming
    pub fn build_prompt(
        &self,
        topic: &str,
        constraints: &[String],
        num_ideas: usize,
    ) -> String {
        let constraints_text = if constraints.is_empty() {
            "No specific constraints.".to_string()
        } else {
            format!("Constraints to consider:\n{}", constraints
                .iter()
                .map(|c| format!("- {}", c))
                .collect::<Vec<_>>()
                .join("\n"))
        };
        
        format!(
            "Brainstorm {} creative ideas for the following topic:\n\
             \nTopic: {}\n\
             \n{}\n\
             \nFor each idea, provide:\n\
             1. A clear title\n\
             2. A brief description (2-3 sentences)\n\
             3. The category (innovation/process/product/marketing/technical/creative)\n\
             4. Feasibility rating (0.0-1.0)\n\
             5. Novelty rating (0.0-1.0)\n\
             \nIdeas:\n",
            num_ideas, topic, constraints_text
        )
    }
    
    /// Extract ideas from LLM response
    pub fn extract_ideas(&self, response: &str) -> Vec<Idea> {
        let mut ideas = Vec::new();
        
        // Parse numbered items or idea blocks
        let mut current_idea: Option<Idea> = None;
        
        for line in response.lines() {
            let line = line.trim();
            
            // Check for new idea start
            if line.starts_with(|c: char| c.is_ascii_digit())
                && line.contains('.')
            {
                // Save previous idea if exists
                if let Some(idea) = current_idea.take() {
                    ideas.push(idea);
                }
                
                // Start new idea
                let title = line
                    .split('.')
                    .nth(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "Untitled Idea".to_string());
                
                current_idea = Some(Idea {
                    title,
                    description: String::new(),
                    category: IdeaCategory::Creative,
                    feasibility: 0.5,
                    novelty: 0.5,
                });
            } else if let Some(ref mut idea) = current_idea {
                // Add to description
                if !idea.description.is_empty() {
                    idea.description.push(' ');
                }
                idea.description.push_str(line);
                
                // Try to extract category
                let lower = line.to_lowercase();
                if lower.contains("innovation") || lower.contains("innovative") {
                    idea.category = IdeaCategory::Innovation;
                } else if lower.contains("process") {
                    idea.category = IdeaCategory::Process;
                } else if lower.contains("product") {
                    idea.category = IdeaCategory::Product;
                } else if lower.contains("marketing") {
                    idea.category = IdeaCategory::Marketing;
                } else if lower.contains("technical") {
                    idea.category = IdeaCategory::Technical;
                }
                
                // Try to extract feasibility
                if let Some(f) = self.extract_rating(&lower, "feasibility") {
                    idea.feasibility = f;
                }
                
                // Try to extract novelty
                if let Some(f) = self.extract_rating(&lower, "novelty") {
                    idea.novelty = f;
                }
            }
        }
        
        // Don't forget the last idea
        if let Some(idea) = current_idea {
            ideas.push(idea);
        }
        
        ideas
    }
    
    /// Extract rating from text
    fn extract_rating(&self, text: &str, label: &str) -> Option<f32> {
        if let Some(pos) = text.find(label) {
            let after = &text[pos + label.len()..];
            // Look for number pattern
            for part in after.split_whitespace().take(5) {
                if let Ok(num) = part.trim_end_matches(|c: char| !c.is_numeric()).parse::<f32>() {
                    if num >= 0.0 && num <= 1.0 {
                        return Some(num);
                    }
                }
            }
        }
        None
    }
}

impl Default for Brainstormer {
    fn default() -> Self {
        Self::new()
    }
}

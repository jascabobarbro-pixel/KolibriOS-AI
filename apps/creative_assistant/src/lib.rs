//! Creative Assistant
//!
//! A living application that leverages Unified Mind's LLM capabilities
//! to assist with creative tasks:
//! - Writing assistance (content generation, editing, summarization)
//! - Image generation suggestions
//! - Creative brainstorming
//! - Style and tone adjustment

pub mod writing;
pub mod brainstorming;
pub mod style;
pub mod image_suggestions;
pub mod context;
pub mod llm_bridge;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub use writing::{WritingAssistant, WritingContext, WritingSuggestion};
pub use brainstorming::{Brainstormer, Idea, BrainstormSession};
pub use style::{StyleAnalyzer, StyleProfile, ToneAdjustment};
pub use image_suggestions::{ImageSuggester, ImagePrompt, ImageStyle};
pub use context::{CreativeContext, CreativeSession};
pub use llm_bridge::{LLMBridge, LLMProvider, LLMResponse};

/// Main Creative Assistant
pub struct CreativeAssistant {
    /// Writing assistant module
    writer: WritingAssistant,
    
    /// Brainstorming module
    brainstormer: Brainstormer,
    
    /// Style analyzer
    style_analyzer: StyleAnalyzer,
    
    /// Image suggestion generator
    image_suggester: ImageSuggester,
    
    /// Context manager
    context: CreativeContext,
    
    /// LLM bridge for Unified Mind
    llm: LLMBridge,
    
    /// Configuration
    config: CreativeConfig,
    
    /// Statistics
    stats: CreativeStats,
}

/// Creative Assistant Configuration
#[derive(Debug, Clone)]
pub struct CreativeConfig {
    /// Unified Mind endpoint
    pub mind_endpoint: String,
    
    /// Default LLM provider
    pub default_provider: LLMProvider,
    
    /// Enable writing assistance
    pub writing_enabled: bool,
    
    /// Enable brainstorming
    pub brainstorming_enabled: bool,
    
    /// Enable image suggestions
    pub image_suggestions_enabled: bool,
    
    /// Max history items
    pub max_history: usize,
    
    /// Response timeout
    pub timeout: Duration,
    
    /// Temperature for creative generation
    pub creative_temperature: f32,
    
    /// Max tokens for responses
    pub max_tokens: u32,
}

impl Default for CreativeConfig {
    fn default() -> Self {
        Self {
            mind_endpoint: "http://localhost:50052".to_string(),
            default_provider: LLMProvider::Gemini,
            writing_enabled: true,
            brainstorming_enabled: true,
            image_suggestions_enabled: true,
            max_history: 100,
            timeout: Duration::from_secs(30),
            creative_temperature: 0.8,
            max_tokens: 2048,
        }
    }
}

/// Statistics for Creative Assistant
#[derive(Debug, Clone, Default)]
pub struct CreativeStats {
    pub total_requests: u64,
    pub writing_requests: u64,
    pub brainstorm_requests: u64,
    pub image_requests: u64,
    pub avg_response_time_ms: u64,
    pub tokens_used: u64,
    pub errors: u64,
}

/// Creative request type
#[derive(Debug, Clone)]
pub enum CreativeRequest {
    /// Writing assistance request
    Writing {
        content: Option<String>,
        task: WritingTask,
        context: Option<WritingContext>,
    },
    
    /// Brainstorming request
    Brainstorm {
        topic: String,
        constraints: Vec<String>,
        num_ideas: usize,
    },
    
    /// Style analysis request
    StyleAnalysis {
        content: String,
    },
    
    /// Image prompt generation
    ImagePrompt {
        description: String,
        style: Option<ImageStyle>,
        num_variations: usize,
    },
    
    /// Creative expansion
    Expand {
        content: String,
        direction: ExpansionDirection,
    },
    
    /// Summary generation
    Summarize {
        content: String,
        max_length: usize,
    },
}

/// Writing task types
#[derive(Debug, Clone)]
pub enum WritingTask {
    GenerateContent {
        prompt: String,
        style: Option<String>,
        length: ContentLength,
    },
    ImproveContent {
        aspects: Vec<ImprovementAspect>,
    },
    ChangeTone {
        target_tone: String,
    },
    FixGrammar,
    GenerateOutline {
        topic: String,
        sections: usize,
    },
    ContinueStory {
        genre: String,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ContentLength {
    Short,    // ~100 words
    Medium,   // ~300 words
    Long,     // ~600 words
    Extended, // ~1200 words
}

#[derive(Debug, Clone)]
pub enum ImprovementAspect {
    Clarity,
    Engagement,
    Persuasion,
    Readability,
    Vocabulary,
    Structure,
}

#[derive(Debug, Clone)]
pub enum ExpansionDirection {
    MoreDetail,
    MoreExamples,
    DeeperAnalysis,
    BroaderContext,
}

/// Creative response
#[derive(Debug, Clone)]
pub struct CreativeResponse {
    pub content: String,
    pub suggestions: Vec<String>,
    pub alternatives: Vec<String>,
    pub confidence: f32,
    pub tokens_used: u32,
    pub provider: LLMProvider,
    pub processing_time_ms: u64,
}

impl CreativeAssistant {
    /// Create a new Creative Assistant
    pub fn new(config: CreativeConfig) -> Self {
        Self {
            writer: WritingAssistant::new(),
            brainstormer: Brainstormer::new(),
            style_analyzer: StyleAnalyzer::new(),
            image_suggester: ImageSuggester::new(),
            context: CreativeContext::new(config.max_history),
            llm: LLMBridge::new(&config.mind_endpoint, config.default_provider.clone()),
            config,
            stats: CreativeStats::default(),
        }
    }
    
    /// Initialize the Creative Assistant
    pub async fn initialize(&mut self) -> Result<(), CreativeError> {
        info!("Initializing Creative Assistant");
        
        // Connect to Unified Mind
        self.llm.connect().await?;
        
        info!("Creative Assistant initialized successfully");
        Ok(())
    }
    
    /// Process a creative request
    pub async fn process(&mut self, request: CreativeRequest) -> Result<CreativeResponse, CreativeError> {
        let start = Instant::now();
        self.stats.total_requests += 1;
        
        let response = match request {
            CreativeRequest::Writing { content, task, context } => {
                self.stats.writing_requests += 1;
                self.process_writing(content, task, context).await?
            }
            
            CreativeRequest::Brainstorm { topic, constraints, num_ideas } => {
                self.stats.brainstorm_requests += 1;
                self.process_brainstorm(topic, constraints, num_ideas).await?
            }
            
            CreativeRequest::StyleAnalysis { content } => {
                self.process_style_analysis(content).await?
            }
            
            CreativeRequest::ImagePrompt { description, style, num_variations } => {
                self.stats.image_requests += 1;
                self.process_image_prompt(description, style, num_variations).await?
            }
            
            CreativeRequest::Expand { content, direction } => {
                self.process_expansion(content, direction).await?
            }
            
            CreativeRequest::Summarize { content, max_length } => {
                self.process_summary(content, max_length).await?
            }
        };
        
        // Update stats
        let elapsed = start.elapsed().as_millis() as u64;
        self.stats.avg_response_time_ms = 
            (self.stats.avg_response_time_ms * (self.stats.total_requests - 1) + elapsed) 
            / self.stats.total_requests;
        self.stats.tokens_used += response.tokens_used as u64;
        
        Ok(response)
    }
    
    /// Process writing request
    async fn process_writing(
        &mut self,
        content: Option<String>,
        task: WritingTask,
        context: Option<WritingContext>,
    ) -> Result<CreativeResponse, CreativeError> {
        let prompt = self.writer.build_prompt(&task, content.as_deref(), context.as_ref());
        
        let llm_response = self.llm.generate(&prompt).await?;
        
        // Parse and enhance response
        let suggestions = self.writer.extract_suggestions(&llm_response.content);
        let alternatives = self.writer.extract_alternatives(&llm_response.content);
        
        Ok(CreativeResponse {
            content: llm_response.content,
            suggestions,
            alternatives,
            confidence: llm_response.confidence,
            tokens_used: llm_response.tokens_used,
            provider: llm_response.provider,
            processing_time_ms: llm_response.processing_time_ms,
        })
    }
    
    /// Process brainstorming request
    async fn process_brainstorm(
        &mut self,
        topic: String,
        constraints: Vec<String>,
        num_ideas: usize,
    ) -> Result<CreativeResponse, CreativeError> {
        let prompt = self.brainstormer.build_prompt(&topic, &constraints, num_ideas);
        
        let llm_response = self.llm.generate(&prompt).await?;
        
        // Extract ideas
        let ideas = self.brainstormer.extract_ideas(&llm_response.content);
        let suggestions = ideas.iter().map(|i| i.title.clone()).collect();
        
        Ok(CreativeResponse {
            content: llm_response.content,
            suggestions,
            alternatives: Vec::new(),
            confidence: llm_response.confidence,
            tokens_used: llm_response.tokens_used,
            provider: llm_response.provider,
            processing_time_ms: llm_response.processing_time_ms,
        })
    }
    
    /// Process style analysis
    async fn process_style_analysis(
        &mut self,
        content: String,
    ) -> Result<CreativeResponse, CreativeError> {
        let analysis = self.style_analyzer.analyze(&content);
        
        let response_text = format!(
            "Style Analysis:\n\n\
            Tone: {}\n\
            Readability Score: {:.1}/100\n\
            Average Sentence Length: {:.1} words\n\
            Vocabulary Level: {}\n\n\
            Recommendations:\n{}",
            analysis.tone,
            analysis.readability_score,
            analysis.avg_sentence_length,
            analysis.vocabulary_level,
            analysis.recommendations.join("\n")
        );
        
        Ok(CreativeResponse {
            content: response_text,
            suggestions: analysis.recommendations,
            alternatives: Vec::new(),
            confidence: 0.95,
            tokens_used: 0,
            provider: LLMProvider::Local,
            processing_time_ms: analysis.processing_time_ms,
        })
    }
    
    /// Process image prompt generation
    async fn process_image_prompt(
        &mut self,
        description: String,
        style: Option<ImageStyle>,
        num_variations: usize,
    ) -> Result<CreativeResponse, CreativeError> {
        let prompt = self.image_suggester.build_prompt(&description, style.as_ref());
        
        let llm_response = self.llm.generate(&prompt).await?;
        
        // Extract prompts
        let prompts = self.image_suggester.extract_prompts(&llm_response.content, num_variations);
        
        let suggestions: Vec<String> = prompts.iter().map(|p| p.prompt.clone()).collect();
        
        Ok(CreativeResponse {
            content: llm_response.content,
            suggestions,
            alternatives: Vec::new(),
            confidence: llm_response.confidence,
            tokens_used: llm_response.tokens_used,
            provider: llm_response.provider,
            processing_time_ms: llm_response.processing_time_ms,
        })
    }
    
    /// Process content expansion
    async fn process_expansion(
        &mut self,
        content: String,
        direction: ExpansionDirection,
    ) -> Result<CreativeResponse, CreativeError> {
        let direction_text = match direction {
            ExpansionDirection::MoreDetail => "Add more specific details and examples",
            ExpansionDirection::MoreExamples => "Provide additional examples and illustrations",
            ExpansionDirection::DeeperAnalysis => "Provide deeper analysis and insights",
            ExpansionDirection::BroaderContext => "Add broader context and connections",
        };
        
        let prompt = format!(
            "Expand the following content. {}.\n\nOriginal content:\n{}\n\nExpanded version:",
            direction_text, content
        );
        
        let llm_response = self.llm.generate(&prompt).await?;
        
        Ok(CreativeResponse {
            content: llm_response.content,
            suggestions: Vec::new(),
            alternatives: Vec::new(),
            confidence: llm_response.confidence,
            tokens_used: llm_response.tokens_used,
            provider: llm_response.provider,
            processing_time_ms: llm_response.processing_time_ms,
        })
    }
    
    /// Process summary generation
    async fn process_summary(
        &mut self,
        content: String,
        max_length: usize,
    ) -> Result<CreativeResponse, CreativeError> {
        let prompt = format!(
            "Summarize the following content in approximately {} words or less:\n\n{}\n\nSummary:",
            max_length / 5, // Rough word estimate
            content
        );
        
        let llm_response = self.llm.generate(&prompt).await?;
        
        Ok(CreativeResponse {
            content: llm_response.content,
            suggestions: Vec::new(),
            alternatives: Vec::new(),
            confidence: llm_response.confidence,
            tokens_used: llm_response.tokens_used,
            provider: llm_response.provider,
            processing_time_ms: llm_response.processing_time_ms,
        })
    }
    
    /// Get current session context
    pub fn context(&self) -> &CreativeContext {
        &self.context
    }
    
    /// Get statistics
    pub fn stats(&self) -> &CreativeStats {
        &self.stats
    }
    
    /// Shutdown
    pub async fn shutdown(&mut self) -> Result<(), CreativeError> {
        info!("Shutting down Creative Assistant");
        self.llm.disconnect().await?;
        info!("Creative Assistant shutdown complete");
        Ok(())
    }
}

impl Default for CreativeAssistant {
    fn default() -> Self {
        Self::new(CreativeConfig::default())
    }
}

/// Creative Assistant Error
#[derive(Debug, thiserror::Error)]
pub enum CreativeError {
    #[error("LLM error: {0}")]
    LLM(String),
    
    #[error("gRPC error: {0}")]
    Grpc(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Timeout")]
    Timeout,
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

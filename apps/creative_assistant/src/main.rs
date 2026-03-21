//! Creative Assistant - Main Entry Point
//!
//! AI-powered creative assistance application.

use std::time::Duration;

use tracing::{info, warn, error};
use tracing_subscriber::fmt;

mod lib;

use creative_assistant::{
    CreativeAssistant, CreativeConfig, CreativeRequest,
    WritingTask, ContentLength, ExpansionDirection,
    LLMProvider,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    fmt::init();
    
    info!("Starting Creative Assistant");
    
    // Configuration
    let config = CreativeConfig {
        mind_endpoint: "http://localhost:50052".to_string(),
        default_provider: LLMProvider::Gemini,
        writing_enabled: true,
        brainstorming_enabled: true,
        image_suggestions_enabled: true,
        max_history: 100,
        timeout: Duration::from_secs(30),
        creative_temperature: 0.8,
        max_tokens: 2048,
    };
    
    // Create assistant
    let mut assistant = CreativeAssistant::new(config);
    
    // Initialize
    match assistant.initialize().await {
        Ok(_) => info!("Creative Assistant initialized successfully"),
        Err(e) => {
            warn!("Failed to initialize (continuing in offline mode): {}", e);
        }
    }
    
    // Demo: Writing assistance
    info!("=== Writing Assistance Demo ===");
    
    let writing_request = CreativeRequest::Writing {
        content: None,
        task: WritingTask::GenerateContent {
            prompt: "The benefits of AI in creative industries".to_string(),
            style: Some("engaging and informative".to_string()),
            length: ContentLength::Medium,
        },
        context: None,
    };
    
    match assistant.process(writing_request).await {
        Ok(response) => {
            info!("Generated content:\n{}\n", response.content);
            info!("Confidence: {:.2}", response.confidence);
        }
        Err(e) => {
            warn!("Writing request failed: {}", e);
        }
    }
    
    // Demo: Brainstorming
    info!("=== Brainstorming Demo ===");
    
    let brainstorm_request = CreativeRequest::Brainstorm {
        topic: "Innovative applications of generative AI".to_string(),
        constraints: vec![
            "Must be feasible with current technology".to_string(),
            "Focus on positive societal impact".to_string(),
        ],
        num_ideas: 5,
    };
    
    match assistant.process(brainstorm_request).await {
        Ok(response) => {
            info!("Brainstorming results:\n{}\n", response.content);
            info!("Suggestions: {:?}", response.suggestions);
        }
        Err(e) => {
            warn!("Brainstorming request failed: {}", e);
        }
    }
    
    // Demo: Style Analysis
    info!("=== Style Analysis Demo ===");
    
    let sample_text = r#"
    The implementation of artificial intelligence in modern society represents 
    a transformative shift in how we approach complex problems. Through the 
    sophisticated application of machine learning algorithms and neural networks, 
    we can now analyze vast datasets with unprecedented accuracy and speed.
    "#;
    
    let style_request = CreativeRequest::StyleAnalysis {
        content: sample_text.to_string(),
    };
    
    match assistant.process(style_request).await {
        Ok(response) => {
            info!("Style Analysis:\n{}\n", response.content);
        }
        Err(e) => {
            warn!("Style analysis failed: {}", e);
        }
    }
    
    // Demo: Image Prompt Generation
    info!("=== Image Prompt Demo ===");
    
    let image_request = CreativeRequest::ImagePrompt {
        description: "A futuristic city with flying cars and vertical gardens".to_string(),
        style: None,
        num_variations: 3,
    };
    
    match assistant.process(image_request).await {
        Ok(response) => {
            info!("Image prompts:\n{}\n", response.content);
        }
        Err(e) => {
            warn!("Image prompt request failed: {}", e);
        }
    }
    
    // Demo: Summary
    info!("=== Summary Demo ===");
    
    let long_text = r#"
    Artificial intelligence has evolved from a theoretical concept to a practical 
    technology that impacts nearly every aspect of modern life. Machine learning 
    algorithms now power recommendation systems, autonomous vehicles, medical 
    diagnostics, and creative tools. The rapid advancement of large language 
    models has particularly transformed natural language processing, enabling 
    sophisticated text generation, translation, and understanding capabilities.
    These developments raise important questions about the future of work, 
    creativity, and human-machine collaboration.
    "#;
    
    let summary_request = CreativeRequest::Summarize {
        content: long_text.to_string(),
        max_length: 50,
    };
    
    match assistant.process(summary_request).await {
        Ok(response) => {
            info!("Summary:\n{}\n", response.content);
        }
        Err(e) => {
            warn!("Summary request failed: {}", e);
        }
    }
    
    // Print statistics
    let stats = assistant.stats();
    info!("=== Statistics ===");
    info!("Total requests: {}", stats.total_requests);
    info!("Writing requests: {}", stats.writing_requests);
    info!("Brainstorm requests: {}", stats.brainstorm_requests);
    info!("Image requests: {}", stats.image_requests);
    info!("Average response time: {}ms", stats.avg_response_time_ms);
    info!("Tokens used: {}", stats.tokens_used);
    
    // Shutdown
    assistant.shutdown().await?;
    
    info!("Creative Assistant shutdown complete");
    
    Ok(())
}

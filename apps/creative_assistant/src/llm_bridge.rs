//! LLM Bridge
//!
//! Bridge to Unified Mind for LLM capabilities.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tonic::transport::Channel;
use tracing::{debug, error, info, warn};

use super::CreativeError;

/// LLM Provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LLMProvider {
    Gemini,
    LocalLlama,
    Auto,
    Local, // For local processing without LLM
}

/// LLM Response
#[derive(Debug, Clone)]
pub struct LLMResponse {
    pub content: String,
    pub confidence: f32,
    pub tokens_used: u32,
    pub provider: LLMProvider,
    pub processing_time_ms: u64,
}

/// LLM Bridge to Unified Mind
pub struct LLMBridge {
    /// Endpoint
    endpoint: String,
    
    /// Default provider
    default_provider: LLMProvider,
    
    /// gRPC channel
    channel: Option<Arc<RwLock<Option<Channel>>>>,
    
    /// Connected flag
    connected: bool,
    
    /// Configuration
    config: BridgeConfig,
}

#[derive(Debug, Clone)]
pub struct BridgeConfig {
    pub timeout: Duration,
    pub max_retries: u32,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_retries: 3,
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

impl LLMBridge {
    pub fn new(endpoint: &str, default_provider: LLMProvider) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            default_provider,
            channel: None,
            connected: false,
            config: BridgeConfig::default(),
        }
    }
    
    /// Connect to Unified Mind
    pub async fn connect(&mut self) -> Result<(), CreativeError> {
        info!("Connecting to Unified Mind at {}", self.endpoint);
        
        match Channel::from_shared(self.endpoint.clone())
            .map_err(|e| CreativeError::Grpc(e.to_string()))?
            .timeout(self.config.timeout)
            .connect()
            .await
        {
            Ok(channel) => {
                self.channel = Some(Arc::new(RwLock::new(Some(channel))));
                self.connected = true;
                info!("Connected to Unified Mind");
                Ok(())
            }
            Err(e) => {
                warn!("Failed to connect to Unified Mind: {}", e);
                // Work in offline mode
                self.connected = false;
                Ok(())
            }
        }
    }
    
    /// Disconnect from Unified Mind
    pub async fn disconnect(&mut self) -> Result<(), CreativeError> {
        self.connected = false;
        if let Some(channel) = &self.channel {
            let mut ch = channel.write().await;
            *ch = None;
        }
        self.channel = None;
        Ok(())
    }
    
    /// Generate content using LLM
    pub async fn generate(&self, prompt: &str) -> Result<LLMResponse, CreativeError> {
        let start = Instant::now();
        
        if self.connected && self.channel.is_some() {
            // Use Unified Mind gRPC
            self.generate_via_grpc(prompt, start).await
        } else {
            // Fallback to local generation
            self.generate_local(prompt, start).await
        }
    }
    
    /// Generate via gRPC
    async fn generate_via_grpc(&self, prompt: &str, start: Instant) -> Result<LLMResponse, CreativeError> {
        // In production, this would call the actual Unified Mind gRPC service
        // For now, simulate with direct LLM call
        
        debug!("Generating via gRPC for prompt: {}...", &prompt[..prompt.len().min(50)]);
        
        // Try to use Gemini API if available
        if let Ok(response) = self.call_gemini_api(prompt).await {
            return Ok(response);
        }
        
        // Fallback to local
        self.generate_local(prompt, start).await
    }
    
    /// Call Gemini API (real implementation)
    async fn call_gemini_api(&self, prompt: &str) -> Result<LLMResponse, CreativeError> {
        // Check if API key is available
        let api_key = std::env::var("GEMINI_API_KEY");
        
        if api_key.is_err() {
            return Err(CreativeError::LLM("No Gemini API key configured".to_string()));
        }
        
        let api_key = api_key.unwrap();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
            api_key
        );
        
        let request_body = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }],
            "generationConfig": {
                "temperature": self.config.temperature,
                "maxOutputTokens": self.config.max_tokens,
            }
        });
        
        let client = reqwest::Client::new();
        
        let response = client
            .post(&url)
            .json(&request_body)
            .timeout(self.config.timeout)
            .send()
            .await
            .map_err(|e| CreativeError::LLM(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CreativeError::LLM(format!("API error {}: {}", status, body)));
        }
        
        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| CreativeError::LLM(e.to_string()))?;
        
        // Extract text from response
        let text = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();
        
        let tokens_used = json["usageMetadata"]["totalTokenCount"]
            .as_u64()
            .unwrap_or(0) as u32;
        
        Ok(LLMResponse {
            content: text,
            confidence: 0.9,
            tokens_used,
            provider: LLMProvider::Gemini,
            processing_time_ms: start.elapsed().as_millis() as u64,
        })
    }
    
    /// Generate locally (fallback)
    async fn generate_local(&self, prompt: &str, start: Instant) -> Result<LLMResponse, CreativeError> {
        debug!("Generating locally for prompt: {}...", &prompt[..prompt.len().min(50)]);
        
        // Check for local Llama model
        let llama_path = std::env::var("LLAMA_MODEL_PATH");
        
        if let Ok(path) = llama_path {
            // Would use llama-cpp-python here
            // For now, return placeholder
            return Ok(LLMResponse {
                content: format!("Local generation with model at: {}", path),
                confidence: 0.7,
                tokens_used: 0,
                provider: LLMProvider::LocalLlama,
                processing_time_ms: start.elapsed().as_millis() as u64,
            });
        }
        
        // Simple template-based response for demo
        let response = self.template_response(prompt);
        
        Ok(LLMResponse {
            content: response,
            confidence: 0.5,
            tokens_used: 0,
            provider: LLMProvider::Local,
            processing_time_ms: start.elapsed().as_millis() as u64,
        })
    }
    
    /// Template-based response (fallback when no LLM available)
    fn template_response(&self, prompt: &str) -> String {
        // Simple template matching for common requests
        let prompt_lower = prompt.to_lowercase();
        
        if prompt_lower.contains("brainstorm") || prompt_lower.contains("ideas") {
            "Here are some creative ideas to consider:\n\n\
             1. Explore unconventional combinations of existing concepts\n\
             2. Look for inspiration in nature and organic processes\n\
             3. Consider the opposite of conventional approaches\n\
             4. Draw connections between seemingly unrelated fields\n\n\
             Would you like me to elaborate on any of these directions?"
                .to_string()
        } else if prompt_lower.contains("summarize") || prompt_lower.contains("summary") {
            "To provide a better summary, I would need access to the full content. \
             In offline mode, I can suggest:\n\
             - Focus on the main thesis or argument\n\
             - Identify 3-5 key supporting points\n\
             - Note any significant conclusions or implications"
                .to_string()
        } else if prompt_lower.contains("improve") || prompt_lower.contains("edit") {
            "Here are some general improvement suggestions:\n\n\
             - Strengthen your opening to grab attention\n\
             - Use active voice for more engaging prose\n\
             - Replace generic words with specific, vivid language\n\
             - Ensure each paragraph has a clear purpose\n\
             - Check for smooth transitions between ideas\n\n\
             Connect to Unified Mind for more specific suggestions."
                .to_string()
        } else {
            format!(
                "I'm currently in offline mode. For the best creative assistance, \
                 please connect to Unified Mind or configure an LLM provider.\n\n\
                 Your prompt: {}\n\n\
                 I can provide template-based suggestions in offline mode.",
                prompt
            )
        }
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    /// Get default provider
    pub fn default_provider(&self) -> LLMProvider {
        self.default_provider
    }
}

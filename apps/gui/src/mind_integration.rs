//! Unified Mind Integration
//!
//! Connects the GUI to the Unified Mind for contextual awareness and adaptive behavior.

use std::sync::Arc;
use std::time::Duration;

use iced::Subscription;
use tokio::sync::RwLock;
use tonic::transport::Channel;

use super::adaptive::AdaptiveState;
use super::dashboard::{DashboardData, SystemMetrics, CellStatus, NeuralSchedulerStatus, UnifiedMindStatus};
use super::notifications::Notification;
use super::Message;

/// Mind Client for Unified Mind communication
#[derive(Debug, Clone)]
pub struct MindClient {
    endpoint: String,
    channel: Option<Arc<RwLock<Option<Channel>>>>,
    connected: bool,
    last_directive: Option<MindDirective>,
    adaptive_state: AdaptiveState,
}

/// Mind Directive - Commands from the Unified Mind
#[derive(Debug, Clone)]
pub enum MindDirective {
    /// Switch to a specific view
    SwitchView(String),
    
    /// Show a notification
    ShowNotification(Notification),
    
    /// Update the theme
    UpdateTheme(String),
    
    /// Optimize storage
    OptimizeStorage,
    
    /// Suggest files to the user
    SuggestFiles(Vec<String>),
    
    /// Send a creative prompt
    CreativePrompt(String),
    
    /// No action
    None,
}

impl MindClient {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            channel: None,
            connected: false,
            last_directive: None,
            adaptive_state: AdaptiveState::default(),
        }
    }
    
    /// Connect to the Unified Mind
    pub async fn connect(&mut self) -> Result<(), String> {
        let endpoint = self.endpoint.clone();
        
        match Channel::from_shared(endpoint.clone())
            .map_err(|e| format!("Invalid endpoint: {}", e))?
            .timeout(Duration::from_secs(5))
            .connect()
            .await
        {
            Ok(channel) => {
                self.channel = Some(Arc::new(RwLock::new(Some(channel))));
                self.connected = true;
                Ok(())
            }
            Err(e) => {
                Err(format!("Connection failed: {}", e))
            }
        }
    }
    
    /// Disconnect from the Unified Mind
    pub async fn disconnect(&mut self) {
        self.connected = false;
        if let Some(channel) = &self.channel {
            let mut ch = channel.write().await;
            *ch = None;
        }
        self.channel = None;
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    /// Get adaptive state
    pub fn adaptive_state(&self) -> &AdaptiveState {
        &self.adaptive_state
    }
    
    /// Update adaptive state from system metrics
    pub fn update_adaptive_state(&mut self, metrics: &SystemMetrics) {
        self.adaptive_state.cpu_load = metrics.cpu_usage / 100.0;
        self.adaptive_state.memory_pressure = metrics.memory_usage / 100.0;
        
        // Determine time context
        let hour = chrono::Local::now().hour();
        self.adaptive_state.time_context = match hour {
            5..=11 => super::adaptive::TimeContext::Morning,
            12..=17 => super::adaptive::TimeContext::Afternoon,
            18..=21 => super::adaptive::TimeContext::Evening,
            _ => super::adaptive::TimeContext::Night,
        };
        
        // Set focus mode if CPU is high (user is likely doing intensive work)
        self.adaptive_state.focus_mode = metrics.cpu_usage > 70.0;
        
        // Set performance mode if system is under load
        self.adaptive_state.performance_mode = metrics.memory_usage > 80.0 || metrics.cpu_usage > 80.0;
    }
    
    /// Fetch dashboard data from Unified Mind
    pub async fn fetch_dashboard_data(&self) -> Result<DashboardData, String> {
        // In real implementation, this would call gRPC methods
        // For now, we simulate based on system state
        
        let mut data = DashboardData::default();
        
        // Get real system metrics using sysinfo
        let system = sysinfo::System::new_all();
        system.refresh_all();
        
        data.system = SystemMetrics {
            cpu_usage: system.global_cpu_usage() as f32,
            memory_usage: (system.used_memory() as f64 / system.total_memory() as f64 * 100.0) as f32,
            disk_usage: 0.0, // Would calculate from disk info
            network_usage: 0.0,
            uptime_seconds: 0,
            process_count: system.processes().len() as u32,
            thread_count: 0, // Would need to sum threads
            load_average: [0.0, 0.0, 0.0], // Platform-specific
        };
        
        // Simulate cell status based on connection
        data.memory_cell = CellStatus {
            name: "Memory Cell".to_string(),
            state: super::dashboard::CellState::Active,
            health: super::dashboard::HealthStatus::Healthy,
            last_heartbeat: Some(std::time::Instant::now()),
            metrics: Default::default(),
        };
        
        data.processor_cell = CellStatus {
            name: "Processor Cell".to_string(),
            state: super::dashboard::CellState::Active,
            health: super::dashboard::HealthStatus::Healthy,
            last_heartbeat: Some(std::time::Instant::now()),
            metrics: Default::default(),
        };
        
        data.neural_scheduler = NeuralSchedulerStatus {
            active: true,
            current_decision: "BalanceLoad".to_string(),
            confidence: 0.85,
            decisions_today: 1523,
            avg_confidence: 0.78,
        };
        
        data.unified_mind = UnifiedMindStatus {
            state: "Ready".to_string(),
            llm_provider: "Gemini".to_string(),
            queries_processed: 42,
            uptime: "2h 15m".to_string(),
            context_items: 7,
        };
        
        Ok(data)
    }
    
    /// Send a command to the Unified Mind
    pub async fn send_command(&self, command: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
        if !self.connected {
            return Err("Not connected to Unified Mind".to_string());
        }
        
        // In real implementation, this would use gRPC
        // For now, return a simulated response
        match command {
            "get_suggestions" => {
                Ok(serde_json::json!({
                    "suggestions": [
                        {"type": "file", "title": "Recent Project", "path": "/home/user/project"},
                        {"type": "action", "title": "Optimize Memory", "action": "optimize_memory"},
                    ]
                }))
            }
            "get_context" => {
                Ok(serde_json::json!({
                    "context": {
                        "current_task": "Development",
                        "active_apps": ["IDE", "Terminal"],
                        "focus_score": 0.8
                    }
                }))
            }
            _ => Ok(serde_json::json!({"status": "ok"}))
        }
    }
    
    /// Request file suggestions based on context
    pub async fn get_file_suggestions(&self, context: &str) -> Result<Vec<FileSuggestion>, String> {
        // Would call Unified Mind's file suggestion capability
        Ok(vec![
            FileSuggestion {
                path: "/home/user/project/src/main.rs".to_string(),
                relevance: 0.95,
                reason: "Recently edited".to_string(),
                suggestion_type: FileSuggestionType::Recent,
            },
            FileSuggestion {
                path: "/home/user/project/docs/README.md".to_string(),
                relevance: 0.80,
                reason: "Related to current task".to_string(),
                suggestion_type: FileSuggestionType::Contextual,
            },
        ])
    }
    
    /// Request creative assistance
    pub async fn get_creative_assistance(&self, prompt: &str) -> Result<CreativeResponse, String> {
        // Would call Unified Mind's LLM capabilities
        Ok(CreativeResponse {
            content: format!("Based on your request: \"{}\", here are some suggestions...", prompt),
            suggestions: vec![
                "Consider structuring your content with clear sections".to_string(),
                "Add visual elements to enhance readability".to_string(),
            ],
            confidence: 0.85,
        })
    }
    
    /// Subscription for mind directives
    pub fn subscription(&self) -> Subscription<Message> {
        // In real implementation, this would subscribe to gRPC stream
        Subscription::none()
    }
}

/// File suggestion from Unified Mind
#[derive(Debug, Clone)]
pub struct FileSuggestion {
    pub path: String,
    pub relevance: f32,
    pub reason: String,
    pub suggestion_type: FileSuggestionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSuggestionType {
    Recent,
    Contextual,
    Frequent,
    Related,
    Predicted,
}

/// Creative response from Unified Mind
#[derive(Debug, Clone)]
pub struct CreativeResponse {
    pub content: String,
    pub suggestions: Vec<String>,
    pub confidence: f32,
}

impl Default for MindClient {
    fn default() -> Self {
        Self::new("http://localhost:50051")
    }
}

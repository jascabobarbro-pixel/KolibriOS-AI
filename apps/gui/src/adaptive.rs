//! Adaptive UI Components
//!
//! Components that dynamically adapt their appearance, layout, and functionality
//! based on context, user behavior, and Unified Mind directives.

use std::time::{Duration, Instant};

/// Adaptive State - The current context for adaptation
#[derive(Debug, Clone, Default)]
pub struct AdaptiveState {
    /// User activity level (0.0 - 1.0)
    pub activity_level: f32,
    
    /// Time of day context
    pub time_context: TimeContext,
    
    /// Focus mode enabled
    pub focus_mode: bool,
    
    /// Performance mode (reduced animations)
    pub performance_mode: bool,
    
    /// Battery level (for laptops)
    pub battery_level: Option<f32>,
    
    /// Network status
    pub network_status: NetworkStatus,
    
    /// Memory pressure
    pub memory_pressure: f32,
    
    /// CPU load
    pub cpu_load: f32,
    
    /// Recent interactions for pattern detection
    pub interaction_patterns: Vec<InteractionPattern>,
    
    /// Current task context
    pub task_context: Option<TaskContext>,
}

/// Time context for adaptive UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeContext {
    #[default]
    Morning,
    Afternoon,
    Evening,
    Night,
}

/// Network status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NetworkStatus {
    #[default]
    Online,
    Offline,
    Limited,
    HighLatency,
}

/// Interaction pattern for learning
#[derive(Debug, Clone)]
pub struct InteractionPattern {
    pub action_type: ActionType,
    pub timestamp: Instant,
    pub context: String,
    pub frequency: u32,
}

/// Types of user actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    FileOpen,
    FileSave,
    AppSwitch,
    Search,
    Command,
    Creative,
    Navigation,
}

/// Task context
#[derive(Debug, Clone)]
pub struct TaskContext {
    pub task_type: String,
    pub priority: u8,
    pub deadline: Option<Instant>,
    pub related_files: Vec<String>,
}

/// Adaptive Component Trait
pub trait AdaptiveComponent {
    /// Get current priority for this component (higher = more important)
    fn priority(&self, state: &AdaptiveState) -> f32;
    
    /// Should this component be visible?
    fn should_show(&self, state: &AdaptiveState) -> bool;
    
    /// Get recommended size factor (0.0 - 2.0)
    fn size_factor(&self, state: &AdaptiveState) -> f32;
    
    /// Get animation speed multiplier
    fn animation_speed(&self, state: &AdaptiveState) -> f32 {
        if state.performance_mode {
            0.2
        } else {
            1.0 - (state.cpu_load * 0.5)
        }
    }
}

/// Adaptive Container
#[derive(Debug, Clone)]
pub struct AdaptiveContainer {
    pub id: String,
    pub content: ContentType,
    pub visibility_rules: Vec<VisibilityRule>,
    pub size_rules: Vec<SizeRule>,
    pub position_rules: Vec<PositionRule>,
    pub animation_config: AnimationConfig,
    pub priority: f32,
    pub visible: bool,
    pub current_size: (f32, f32),
    pub target_size: (f32, f32),
    pub animation_progress: f32,
}

/// Content type for adaptive containers
#[derive(Debug, Clone)]
pub enum ContentType {
    Dashboard,
    Notification,
    FileManager,
    CreativeAssistant,
    Settings,
    Custom(String),
}

/// Visibility rule
#[derive(Debug, Clone)]
pub enum VisibilityRule {
    Always,
    WhenFocused,
    WhenIdle,
    WhenMemoryLow,
    WhenBatteryLow,
    WhenActivityAbove(f32),
    WhenActivityBelow(f32),
    Custom(Box<dyn Fn(&AdaptiveState) -> bool + Send + Sync>),
}

/// Size rule
#[derive(Debug, Clone)]
pub enum SizeRule {
    Fixed(f32, f32),
    Proportional(f32, f32),
    BasedOnContent,
    BasedOnActivity(f32, f32), // min, max based on activity
    AdaptiveMemory, // Grow when memory available, shrink when low
}

/// Position rule
#[derive(Debug, Clone)]
pub enum PositionRule {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
    Sidebar,
    Floating,
    Grid(usize, usize), // row, col
}

/// Animation configuration
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    pub enabled: bool,
    pub duration_ms: u32,
    pub easing: EasingFunction,
    pub delay_ms: u32,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            duration_ms: 200,
            easing: EasingFunction::EaseOutCubic,
            delay_ms: 0,
        }
    }
}

/// Easing functions for animations
#[derive(Debug, Clone, Copy)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    Bounce,
    Elastic,
}

impl EasingFunction {
    /// Apply easing function to progress (0.0 - 1.0)
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseIn => t * t,
            EasingFunction::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            EasingFunction::EaseInCubic => t * t * t,
            EasingFunction::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            EasingFunction::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            EasingFunction::Bounce => {
                const N1: f32 = 7.5625;
                const D1: f32 = 2.75;
                
                if t < 1.0 / D1 {
                    N1 * t * t
                } else if t < 2.0 / D1 {
                    let t = t - 1.5 / D1;
                    N1 * t * t + 0.75
                } else if t < 2.5 / D1 {
                    let t = t - 2.25 / D1;
                    N1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / D1;
                    N1 * t * t + 0.984375
                }
            }
            EasingFunction::Elastic => {
                const C4: f32 = 2.0 * std::f32::consts::PI / 3.0;
                
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    -(2.0_f32.powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * C4).sin()
                }
            }
        }
    }
}

impl AdaptiveComponent for AdaptiveContainer {
    fn priority(&self, state: &AdaptiveState) -> f32 {
        let mut p = self.priority;
        
        // Boost priority based on task context
        if let Some(ref task) = state.task_context {
            if self.content.matches_task(&task.task_type) {
                p *= 1.5;
            }
        }
        
        // Reduce priority in focus mode for non-essential components
        if state.focus_mode && !self.is_essential() {
            p *= 0.5;
        }
        
        p
    }
    
    fn should_show(&self, state: &AdaptiveState) -> bool {
        for rule in &self.visibility_rules {
            match rule {
                VisibilityRule::Always => return true,
                VisibilityRule::WhenFocused => {
                    if state.focus_mode {
                        return true;
                    }
                }
                VisibilityRule::WhenIdle => {
                    if state.activity_level < 0.3 {
                        return true;
                    }
                }
                VisibilityRule::WhenMemoryLow => {
                    if state.memory_pressure > 0.8 {
                        return false; // Hide to save memory
                    }
                }
                VisibilityRule::WhenBatteryLow => {
                    if let Some(battery) = state.battery_level {
                        if battery < 0.2 {
                            return false; // Hide to save battery
                        }
                    }
                }
                VisibilityRule::WhenActivityAbove(threshold) => {
                    if state.activity_level > *threshold {
                        return true;
                    }
                }
                VisibilityRule::WhenActivityBelow(threshold) => {
                    if state.activity_level < *threshold {
                        return true;
                    }
                }
                VisibilityRule::Custom(func) => {
                    if func(state) {
                        return true;
                    }
                }
            }
        }
        
        true
    }
    
    fn size_factor(&self, state: &AdaptiveState) -> f32 {
        let mut factor = 1.0;
        
        // Adjust based on activity
        factor *= 0.8 + (state.activity_level * 0.4);
        
        // Adjust based on memory pressure
        if state.memory_pressure > 0.7 {
            factor *= 1.0 - ((state.memory_pressure - 0.7) * 0.5);
        }
        
        // Adjust based on battery
        if let Some(battery) = state.battery_level {
            if battery < 0.3 {
                factor *= 0.8; // Smaller = less rendering work
            }
        }
        
        factor.clamp(0.5, 1.5)
    }
}

impl AdaptiveContainer {
    fn is_essential(&self) -> bool {
        matches!(self.content, ContentType::Dashboard | ContentType::FileManager)
    }
    
    /// Update animation progress
    pub fn update_animation(&mut self, delta_ms: u32) {
        if !self.animation_config.enabled {
            self.animation_progress = 1.0;
            self.current_size = self.target_size;
            return;
        }
        
        let progress_delta = delta_ms as f32 / self.animation_config.duration_ms as f32;
        self.animation_progress = (self.animation_progress + progress_delta).min(1.0);
        
        let eased_progress = self.animation_config.easing.apply(self.animation_progress);
        
        self.current_size = (
            self.current_size.0 + (self.target_size.0 - self.current_size.0) * eased_progress,
            self.current_size.1 + (self.target_size.1 - self.current_size.1) * eased_progress,
        );
    }
}

impl ContentType {
    fn matches_task(&self, task_type: &str) -> bool {
        match self {
            ContentType::FileManager => {
                task_type.contains("file") || task_type.contains("document") || task_type.contains("folder")
            }
            ContentType::CreativeAssistant => {
                task_type.contains("creative") || task_type.contains("write") || task_type.contains("design")
            }
            ContentType::Dashboard => {
                task_type.contains("monitor") || task_type.contains("status") || task_type.contains("overview")
            }
            _ => false,
        }
    }
}

/// Adaptive Layout Manager
#[derive(Debug, Default)]
pub struct AdaptiveLayoutManager {
    containers: Vec<AdaptiveContainer>,
    state: AdaptiveState,
    last_update: Instant,
}

impl AdaptiveLayoutManager {
    pub fn new() -> Self {
        Self {
            containers: Vec::new(),
            state: AdaptiveState::default(),
            last_update: Instant::now(),
        }
    }
    
    /// Add a container to the layout
    pub fn add_container(&mut self, container: AdaptiveContainer) {
        self.containers.push(container);
    }
    
    /// Update the adaptive state
    pub fn update_state(&mut self, state: AdaptiveState) {
        self.state = state;
        self.apply_adaptations();
    }
    
    /// Apply adaptations to all containers
    pub fn apply_adaptations(&mut self) {
        for container in &mut self.containers {
            // Update visibility
            container.visible = container.should_show(&self.state);
            
            // Update size based on rules
            if container.visible {
                let size_factor = container.size_factor(&self.state);
                container.target_size = container.calculate_target_size(&self.state, size_factor);
            }
        }
        
        // Sort by priority
        self.containers.sort_by(|a, b| {
            b.priority(&self.state).partial_cmp(&a.priority(&self.state)).unwrap()
        });
    }
    
    /// Tick the animation system
    pub fn tick(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_update);
        self.last_update = now;
        
        let delta_ms = delta.as_millis() as u32;
        
        for container in &mut self.containers {
            container.update_animation(delta_ms);
        }
    }
    
    /// Get visible containers sorted by priority
    pub fn get_visible_containers(&self) -> Vec<&AdaptiveContainer> {
        self.containers
            .iter()
            .filter(|c| c.visible)
            .collect()
    }
}

impl AdaptiveContainer {
    fn calculate_target_size(&self, state: &AdaptiveState, size_factor: f32) -> (f32, f32) {
        for rule in &self.size_rules {
            match rule {
                SizeRule::Fixed(w, h) => {
                    return (*w * size_factor, *h * size_factor);
                }
                SizeRule::Proportional(w, h) => {
                    // Would need screen dimensions here
                    return (*w * 100.0 * size_factor, *h * 100.0 * size_factor);
                }
                SizeRule::BasedOnContent => {
                    // Default size, will be adjusted by content
                    return (300.0 * size_factor, 200.0 * size_factor);
                }
                SizeRule::BasedOnActivity(min, max) => {
                    let factor = min + (max - min) * state.activity_level;
                    return (factor, factor);
                }
                SizeRule::AdaptiveMemory => {
                    // Larger when memory available
                    let memory_factor = 1.0 - state.memory_pressure;
                    return (300.0 * memory_factor * size_factor, 200.0 * memory_factor * size_factor);
                }
            }
        }
        
        (300.0, 200.0) // Default
    }
}

//! Animation System
//!
//! Smooth animations and transitions for adaptive UI components.

use std::time::{Duration, Instant};

/// Animation controller
#[derive(Debug, Clone)]
pub struct AnimationController {
    /// Active animations
    animations: Vec<Animation>,
    
    /// Global animation speed multiplier
    speed_multiplier: f32,
    
    /// Animations enabled
    enabled: bool,
    
    /// Reduced motion preference
    reduced_motion: bool,
}

/// Single animation
#[derive(Debug, Clone)]
pub struct Animation {
    pub id: String,
    pub animation_type: AnimationType,
    pub state: AnimationState,
    pub duration: Duration,
    pub elapsed: Duration,
    pub easing: Easing,
    pub delay: Duration,
    pub remaining_delay: Duration,
    pub on_complete: Option<AnimationCompleteAction>,
}

/// Animation types
#[derive(Debug, Clone)]
pub enum AnimationType {
    /// Fade in/out
    Fade { from: f32, to: f32 },
    
    /// Slide animation
    Slide { from_x: f32, from_y: f32, to_x: f32, to_y: f32 },
    
    /// Scale animation
    Scale { from: f32, to: f32 },
    
    /// Rotation animation
    Rotate { from_degrees: f32, to_degrees: f32 },
    
    /// Combined fade and slide
    FadeSlide {
        from_alpha: f32,
        to_alpha: f32,
        from_x: f32,
        from_y: f32,
        to_x: f32,
        to_y: f32,
    },
    
    /// Pulse animation (scale up and down)
    Pulse { scale: f32 },
    
    /// Progress animation
    Progress { from: f32, to: f32 },
    
    /// Color transition
    Color {
        from_r: f32, from_g: f32, from_b: f32, from_a: f32,
        to_r: f32, to_g: f32, to_b: f32, to_a: f32,
    },
}

/// Animation state
#[derive(Debug, Clone)]
pub struct AnimationState {
    pub progress: f32,
    pub current_value: AnimationValue,
    pub is_complete: bool,
}

/// Current animation value
#[derive(Debug, Clone)]
pub enum AnimationValue {
    Float(f32),
    Position { x: f32, y: f32 },
    Scale { x: f32, y: f32 },
    Rotation(f32),
    Combined { alpha: f32, x: f32, y: f32 },
    Color { r: f32, g: f32, b: f32, a: f32 },
}

/// Easing functions
#[derive(Debug, Clone, Copy)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    Spring { tension: f32, friction: f32 },
    Bounce,
    Elastic,
}

/// Action to perform when animation completes
#[derive(Debug, Clone)]
pub enum AnimationCompleteAction {
    Remove,
    Reverse,
    Callback(String),
    Chain(Box<Animation>),
}

impl Default for AnimationController {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationController {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            speed_multiplier: 1.0,
            enabled: true,
            reduced_motion: false,
        }
    }
    
    /// Set reduced motion preference
    pub fn set_reduced_motion(&mut self, enabled: bool) {
        self.reduced_motion = enabled;
        if enabled {
            // Complete all animations immediately
            for animation in &mut self.animations {
                animation.state.progress = 1.0;
                animation.state.is_complete = true;
            }
        }
    }
    
    /// Add an animation
    pub fn add(&mut self, animation: Animation) {
        if !self.enabled || self.reduced_motion {
            return;
        }
        self.animations.push(animation);
    }
    
    /// Create a fade animation
    pub fn fade(id: impl Into<String>, from: f32, to: f32, duration: Duration) -> Animation {
        Animation {
            id: id.into(),
            animation_type: AnimationType::Fade { from, to },
            state: AnimationState {
                progress: 0.0,
                current_value: AnimationValue::Float(from),
                is_complete: false,
            },
            duration,
            elapsed: Duration::ZERO,
            easing: Easing::EaseOutCubic,
            delay: Duration::ZERO,
            remaining_delay: Duration::ZERO,
            on_complete: Some(AnimationCompleteAction::Remove),
        }
    }
    
    /// Create a slide animation
    pub fn slide(
        id: impl Into<String>,
        from_x: f32,
        from_y: f32,
        to_x: f32,
        to_y: f32,
        duration: Duration,
    ) -> Animation {
        Animation {
            id: id.into(),
            animation_type: AnimationType::Slide { from_x, from_y, to_x, to_y },
            state: AnimationState {
                progress: 0.0,
                current_value: AnimationValue::Position { x: from_x, y: from_y },
                is_complete: false,
            },
            duration,
            elapsed: Duration::ZERO,
            easing: Easing::EaseOutCubic,
            delay: Duration::ZERO,
            remaining_delay: Duration::ZERO,
            on_complete: Some(AnimationCompleteAction::Remove),
        }
    }
    
    /// Create a scale animation
    pub fn scale(id: impl Into<String>, from: f32, to: f32, duration: Duration) -> Animation {
        Animation {
            id: id.into(),
            animation_type: AnimationType::Scale { from, to },
            state: AnimationState {
                progress: 0.0,
                current_value: AnimationValue::Scale { x: from, y: from },
                is_complete: false,
            },
            duration,
            elapsed: Duration::ZERO,
            easing: Easing::EaseOutBack,
            delay: Duration::ZERO,
            remaining_delay: Duration::ZERO,
            on_complete: Some(AnimationCompleteAction::Remove),
        }
    }
    
    /// Update all animations
    pub fn tick(&mut self, delta: Duration) {
        let adjusted_delta = Duration::from_secs_f32(delta.as_secs_f32() * self.speed_multiplier);
        
        for animation in &mut self.animations {
            if animation.state.is_complete {
                continue;
            }
            
            // Handle delay
            if animation.remaining_delay > Duration::ZERO {
                animation.remaining_delay = animation.remaining_delay.saturating_sub(adjusted_delta);
                continue;
            }
            
            animation.elapsed += adjusted_delta;
            animation.state.progress = (animation.elapsed.as_secs_f32() / animation.duration.as_secs_f32())
                .min(1.0);
            
            // Apply easing
            let eased_progress = animation.easing.apply(animation.state.progress);
            
            // Calculate current value
            animation.state.current_value = animation.calculate_value(eased_progress);
            
            if animation.state.progress >= 1.0 {
                animation.state.is_complete = true;
            }
        }
        
        // Handle completed animations
        self.animations.retain(|a| !a.state.is_complete || a.on_complete.is_some());
    }
    
    /// Get animation by ID
    pub fn get(&self, id: &str) -> Option<&Animation> {
        self.animations.iter().find(|a| a.id == id)
    }
    
    /// Remove animation by ID
    pub fn remove(&mut self, id: &str) {
        self.animations.retain(|a| a.id != id);
    }
    
    /// Check if animation is running
    pub fn is_running(&self, id: &str) -> bool {
        self.animations.iter().any(|a| a.id == id && !a.state.is_complete)
    }
}

impl Animation {
    fn calculate_value(&self, progress: f32) -> AnimationValue {
        match &self.animation_type {
            AnimationType::Fade { from, to } => {
                AnimationValue::Float(from + (to - from) * progress)
            }
            
            AnimationType::Slide { from_x, from_y, to_x, to_y } => {
                AnimationValue::Position {
                    x: from_x + (to_x - from_x) * progress,
                    y: from_y + (to_y - from_y) * progress,
                }
            }
            
            AnimationType::Scale { from, to } => {
                let value = from + (to - from) * progress;
                AnimationValue::Scale { x: value, y: value }
            }
            
            AnimationType::Rotate { from_degrees, to_degrees } => {
                AnimationValue::Rotation(from_degrees + (to_degrees - from_degrees) * progress)
            }
            
            AnimationType::FadeSlide {
                from_alpha, to_alpha,
                from_x, from_y, to_x, to_y,
            } => {
                AnimationValue::Combined {
                    alpha: from_alpha + (to_alpha - from_alpha) * progress,
                    x: from_x + (to_x - from_x) * progress,
                    y: from_y + (to_y - from_y) * progress,
                }
            }
            
            AnimationType::Pulse { scale } => {
                let pulse = (progress * std::f32::consts::PI).sin() * 0.5 + 0.5;
                AnimationValue::Scale {
                    x: 1.0 + scale * pulse,
                    y: 1.0 + scale * pulse,
                }
            }
            
            AnimationType::Progress { from, to } => {
                AnimationValue::Float(from + (to - from) * progress)
            }
            
            AnimationType::Color {
                from_r, from_g, from_b, from_a,
                to_r, to_g, to_b, to_a,
            } => {
                AnimationValue::Color {
                    r: from_r + (to_r - from_r) * progress,
                    g: from_g + (to_g - from_g) * progress,
                    b: from_b + (to_b - from_b) * progress,
                    a: from_a + (to_a - from_a) * progress,
                }
            }
        }
    }
}

impl Easing {
    /// Apply easing function to progress (0.0 - 1.0)
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            
            Easing::EaseIn => t * t,
            
            Easing::EaseOut => 1.0 - (1.0 - t).powi(2),
            
            Easing::EaseInOut => {
                if t < 0.5 { 2.0 * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 }
            }
            
            Easing::EaseInQuad => t * t,
            
            Easing::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            
            Easing::EaseInOutQuad => {
                if t < 0.5 { 2.0 * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 }
            }
            
            Easing::EaseInCubic => t * t * t,
            
            Easing::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            
            Easing::EaseInOutCubic => {
                if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
            }
            
            Easing::EaseInQuart => t * t * t * t,
            
            Easing::EaseOutQuart => 1.0 - (1.0 - t).powi(4),
            
            Easing::EaseInOutQuart => {
                if t < 0.5 { 8.0 * t * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(4) / 2.0 }
            }
            
            Easing::Spring { tension, friction } => {
                // Simplified spring animation
                let omega = (tension / 1.0).sqrt();
                let decay = (-friction * t / 100.0).exp();
                1.0 - decay * (omega * t * 10.0).cos()
            }
            
            Easing::Bounce => {
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
            
            Easing::Elastic => {
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

/// Custom easing for ease-out-back (slightly overshoots)
impl Easing {
    pub fn ease_out_back(t: f32) -> f32 {
        const C1: f32 = 1.70158;
        const C3: f32 = C1 + 1.0;
        
        1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
    }
}

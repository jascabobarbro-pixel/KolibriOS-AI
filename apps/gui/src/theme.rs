//! Adaptive Theme System
//!
//! Dynamic themes that change based on context, time of day, and user preferences.

use iced::Color;

/// Kolibri Theme
#[derive(Debug, Clone)]
pub struct KolibriTheme {
    pub name: String,
    pub colors: ColorPalette,
    pub typography: Typography,
    pub spacing: Spacing,
    pub radius: BorderRadius,
    pub shadows: ShadowConfig,
    pub animations: AnimationTheme,
}

/// Color palette
#[derive(Debug, Clone)]
pub struct ColorPalette {
    // Primary colors
    pub primary: Color,
    pub primary_hover: Color,
    pub primary_active: Color,
    
    // Secondary colors
    pub secondary: Color,
    pub secondary_hover: Color,
    
    // Background colors
    pub background: Color,
    pub surface: Color,
    pub surface_hover: Color,
    pub surface_elevated: Color,
    
    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    
    // Status colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    
    // Border colors
    pub border: Color,
    pub border_focus: Color,
    
    // Adaptive colors (change based on context)
    pub accent: Color,
}

/// Typography configuration
#[derive(Debug, Clone)]
pub struct Typography {
    pub font_family: String,
    pub font_sizes: FontSizes,
    pub font_weights: FontWeights,
    pub line_heights: LineHeights,
}

#[derive(Debug, Clone)]
pub struct FontSizes {
    pub xs: f32,
    pub sm: f32,
    pub base: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
    pub display: f32,
}

impl Default for FontSizes {
    fn default() -> Self {
        Self {
            xs: 10.0,
            sm: 12.0,
            base: 14.0,
            lg: 16.0,
            xl: 20.0,
            xxl: 24.0,
            display: 32.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FontWeights {
    pub light: f32,
    pub regular: f32,
    pub medium: f32,
    pub semibold: f32,
    pub bold: f32,
}

impl Default for FontWeights {
    fn default() -> Self {
        Self {
            light: 300.0,
            regular: 400.0,
            medium: 500.0,
            semibold: 600.0,
            bold: 700.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LineHeights {
    pub tight: f32,
    pub normal: f32,
    pub relaxed: f32,
}

impl Default for LineHeights {
    fn default() -> Self {
        Self {
            tight: 1.2,
            normal: 1.5,
            relaxed: 1.75,
        }
    }
}

/// Spacing configuration
#[derive(Debug, Clone)]
pub struct Spacing {
    pub xxs: f32,
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            xxs: 2.0,
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
            xxl: 48.0,
        }
    }
}

/// Border radius configuration
#[derive(Debug, Clone)]
pub struct BorderRadius {
    pub none: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub full: f32,
}

impl Default for BorderRadius {
    fn default() -> Self {
        Self {
            none: 0.0,
            sm: 4.0,
            md: 8.0,
            lg: 12.0,
            xl: 16.0,
            full: 9999.0,
        }
    }
}

/// Shadow configuration
#[derive(Debug, Clone)]
pub struct ShadowConfig {
    pub none: Shadow,
    pub sm: Shadow,
    pub md: Shadow,
    pub lg: Shadow,
    pub xl: Shadow,
}

#[derive(Debug, Clone)]
pub struct Shadow {
    pub color: Color,
    pub offset: (f32, f32),
    pub blur: f32,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        let shadow_color = Color::from_rgba(0.0, 0.0, 0.0, 0.1);
        Self {
            none: Shadow { color: Color::TRANSPARENT, offset: (0.0, 0.0), blur: 0.0 },
            sm: Shadow { color: shadow_color, offset: (0.0, 1.0), blur: 2.0 },
            md: Shadow { color: shadow_color, offset: (0.0, 4.0), blur: 8.0 },
            lg: Shadow { color: shadow_color, offset: (0.0, 8.0), blur: 16.0 },
            xl: Shadow { color: shadow_color, offset: (0.0, 16.0), blur: 32.0 },
        }
    }
}

/// Animation theme
#[derive(Debug, Clone)]
pub struct AnimationTheme {
    pub duration_fast: u32,
    pub duration_normal: u32,
    pub duration_slow: u32,
    pub easing_default: EasingType,
}

#[derive(Debug, Clone, Copy)]
pub enum EasingType {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl Default for AnimationTheme {
    fn default() -> Self {
        Self {
            duration_fast: 100,
            duration_normal: 200,
            duration_slow: 400,
            easing_default: EasingType::EaseOut,
        }
    }
}

impl Default for KolibriTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl KolibriTheme {
    /// Create a dark theme
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            colors: ColorPalette {
                primary: Color::from_rgb(0.4, 0.6, 1.0),
                primary_hover: Color::from_rgb(0.5, 0.7, 1.0),
                primary_active: Color::from_rgb(0.3, 0.5, 0.9),
                secondary: Color::from_rgb(0.6, 0.4, 0.9),
                secondary_hover: Color::from_rgb(0.7, 0.5, 1.0),
                background: Color::from_rgb(0.08, 0.08, 0.1),
                surface: Color::from_rgb(0.12, 0.12, 0.14),
                surface_hover: Color::from_rgb(0.16, 0.16, 0.18),
                surface_elevated: Color::from_rgb(0.18, 0.18, 0.2),
                text_primary: Color::from_rgb(0.95, 0.95, 0.95),
                text_secondary: Color::from_rgb(0.7, 0.7, 0.7),
                text_disabled: Color::from_rgb(0.5, 0.5, 0.5),
                success: Color::from_rgb(0.3, 0.8, 0.5),
                warning: Color::from_rgb(1.0, 0.75, 0.2),
                error: Color::from_rgb(1.0, 0.35, 0.35),
                info: Color::from_rgb(0.4, 0.7, 1.0),
                border: Color::from_rgb(0.25, 0.25, 0.28),
                border_focus: Color::from_rgb(0.4, 0.6, 1.0),
                accent: Color::from_rgb(0.4, 0.6, 1.0),
            },
            typography: Typography::default(),
            spacing: Spacing::default(),
            radius: BorderRadius::default(),
            shadows: ShadowConfig::default(),
            animations: AnimationTheme::default(),
        }
    }
    
    /// Create a light theme
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            colors: ColorPalette {
                primary: Color::from_rgb(0.2, 0.45, 0.95),
                primary_hover: Color::from_rgb(0.3, 0.55, 1.0),
                primary_active: Color::from_rgb(0.15, 0.35, 0.85),
                secondary: Color::from_rgb(0.5, 0.3, 0.85),
                secondary_hover: Color::from_rgb(0.6, 0.4, 0.95),
                background: Color::from_rgb(0.97, 0.97, 0.98),
                surface: Color::from_rgb(1.0, 1.0, 1.0),
                surface_hover: Color::from_rgb(0.95, 0.95, 0.96),
                surface_elevated: Color::from_rgb(1.0, 1.0, 1.0),
                text_primary: Color::from_rgb(0.1, 0.1, 0.12),
                text_secondary: Color::from_rgb(0.4, 0.4, 0.45),
                text_disabled: Color::from_rgb(0.6, 0.6, 0.65),
                success: Color::from_rgb(0.15, 0.7, 0.4),
                warning: Color::from_rgb(0.95, 0.65, 0.1),
                error: Color::from_rgb(0.95, 0.25, 0.25),
                info: Color::from_rgb(0.2, 0.55, 0.95),
                border: Color::from_rgb(0.85, 0.85, 0.87),
                border_focus: Color::from_rgb(0.2, 0.45, 0.95),
                accent: Color::from_rgb(0.2, 0.45, 0.95),
            },
            typography: Typography::default(),
            spacing: Spacing::default(),
            radius: BorderRadius::default(),
            shadows: ShadowConfig::default(),
            animations: AnimationTheme::default(),
        }
    }
    
    /// Create focus mode theme (minimal distractions)
    pub fn focus_mode() -> Self {
        let mut theme = Self::dark();
        theme.name = "Focus Mode".to_string();
        theme.colors.surface = Color::from_rgb(0.05, 0.05, 0.06);
        theme.colors.text_secondary = Color::from_rgb(0.5, 0.5, 0.5);
        theme.colors.accent = Color::from_rgb(0.3, 0.5, 0.8);
        theme.animations.duration_normal = 0; // No animations in focus mode
        theme
    }
    
    /// Create a high contrast theme
    pub fn high_contrast() -> Self {
        Self {
            name: "High Contrast".to_string(),
            colors: ColorPalette {
                primary: Color::from_rgb(0.0, 0.8, 1.0),
                primary_hover: Color::from_rgb(0.2, 0.9, 1.0),
                primary_active: Color::from_rgb(0.0, 0.7, 0.9),
                secondary: Color::from_rgb(1.0, 0.8, 0.0),
                secondary_hover: Color::from_rgb(1.0, 0.9, 0.2),
                background: Color::from_rgb(0.0, 0.0, 0.0),
                surface: Color::from_rgb(0.05, 0.05, 0.05),
                surface_hover: Color::from_rgb(0.1, 0.1, 0.1),
                surface_elevated: Color::from_rgb(0.15, 0.15, 0.15),
                text_primary: Color::from_rgb(1.0, 1.0, 1.0),
                text_secondary: Color::from_rgb(0.9, 0.9, 0.9),
                text_disabled: Color::from_rgb(0.6, 0.6, 0.6),
                success: Color::from_rgb(0.0, 1.0, 0.4),
                warning: Color::from_rgb(1.0, 0.9, 0.0),
                error: Color::from_rgb(1.0, 0.2, 0.2),
                info: Color::from_rgb(0.0, 0.8, 1.0),
                border: Color::from_rgb(0.8, 0.8, 0.8),
                border_focus: Color::from_rgb(0.0, 1.0, 1.0),
                accent: Color::from_rgb(0.0, 0.8, 1.0),
            },
            typography: Typography {
                font_sizes: FontSizes {
                    base: 16.0, // Slightly larger for readability
                    ..Default::default()
                },
                ..Default::default()
            },
            spacing: Spacing::default(),
            radius: BorderRadius::default(),
            shadows: ShadowConfig::default(),
            animations: AnimationTheme::default(),
        }
    }
    
    /// Create theme from name
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "light" => Self::light(),
            "dark" => Self::dark(),
            "focus" | "focus_mode" => Self::focus_mode(),
            "high_contrast" | "high-contrast" => Self::high_contrast(),
            _ => Self::dark(),
        }
    }
    
    /// Convert to Iced theme
    pub fn to_iced_theme(&self) -> iced::Theme {
        iced::Theme::custom(
            self.name.clone(),
            iced::theme::Palette {
                background: self.colors.background,
                text: self.colors.text_primary,
                primary: self.colors.primary,
                success: self.colors.success,
                danger: self.colors.error,
            },
        )
    }
    
    /// Adjust for time of day
    pub fn adjust_for_time(&self, hour: u8) -> Self {
        let mut theme = self.clone();
        
        // Warm up colors in the evening
        if hour >= 18 || hour < 6 {
            // Evening/Night: warmer, darker colors
            let warmth = if hour >= 20 || hour < 5 { 0.15 } else { 0.08 };
            
            theme.colors.background = Self::warm_color(theme.colors.background, warmth);
            theme.colors.surface = Self::warm_color(theme.colors.surface, warmth);
            theme.colors.primary = Self::warm_color(theme.colors.primary, warmth * 0.5);
        } else if hour >= 6 && hour < 9 {
            // Morning: slightly cooler, brighter
            theme.colors.surface = Self::adjust_brightness(theme.colors.surface, 0.02);
        }
        
        theme
    }
    
    fn warm_color(color: Color, amount: f32) -> Color {
        Color::from_rgb(
            (color.r + amount).min(1.0),
            (color.g + amount * 0.5).min(1.0),
            color.b,
        )
    }
    
    fn adjust_brightness(color: Color, amount: f32) -> Color {
        Color::from_rgb(
            (color.r + amount).min(1.0),
            (color.g + amount).min(1.0),
            (color.b + amount).min(1.0),
        )
    }
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            font_family: "Inter".to_string(),
            font_sizes: FontSizes::default(),
            font_weights: FontWeights::default(),
            line_heights: LineHeights::default(),
        }
    }
}

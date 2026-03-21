//! Image Suggestions Module
//!
//! Generates prompts and suggestions for image generation.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Image Suggester
pub struct ImageSuggester {
    /// Style templates
    styles: HashMap<String, ImageStyle>,
}

/// Image prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePrompt {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub style: ImageStyle,
    pub aspect_ratio: AspectRatio,
    pub quality_settings: QualitySettings,
}

/// Image style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageStyle {
    pub name: String,
    pub modifiers: Vec<String>,
    pub artists: Vec<String>,
    pub negative_modifiers: Vec<String>,
}

/// Aspect ratio
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AspectRatio {
    Square,      // 1:1
    Landscape,   // 16:9
    Portrait,    // 9:16
    Wide,        // 21:9
}

/// Quality settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySettings {
    pub quality: String,
    pub detail_level: String,
    pub lighting: String,
}

impl ImageSuggester {
    pub fn new() -> Self {
        let mut styles = HashMap::new();
        
        styles.insert(
            "photorealistic".to_string(),
            ImageStyle {
                name: "Photorealistic".to_string(),
                modifiers: vec![
                    "highly detailed".to_string(),
                    "photorealistic".to_string(),
                    "8k resolution".to_string(),
                    "professional photography".to_string(),
                ],
                artists: vec!["Ansel Adams".to_string()],
                negative_modifiers: vec![
                    "cartoon".to_string(),
                    "illustration".to_string(),
                    "painting".to_string(),
                ],
            }
        );
        
        styles.insert(
            "digital_art".to_string(),
            ImageStyle {
                name: "Digital Art".to_string(),
                modifiers: vec![
                    "digital art".to_string(),
                    "concept art".to_string(),
                    "trending on artstation".to_string(),
                    "vibrant colors".to_string(),
                ],
                artists: vec![
                    "Greg Rutkowski".to_string(),
                    "Artgerm".to_string(),
                ],
                negative_modifiers: vec![
                    "blurry".to_string(),
                    "low quality".to_string(),
                ],
            }
        );
        
        styles.insert(
            "anime".to_string(),
            ImageStyle {
                name: "Anime".to_string(),
                modifiers: vec![
                    "anime style".to_string(),
                    "cel shading".to_string(),
                    "vibrant".to_string(),
                    "clean lines".to_string(),
                ],
                artists: vec![
                    "Makoto Shinkai".to_string(),
                    "Studio Ghibli".to_string(),
                ],
                negative_modifiers: vec![
                    "realistic".to_string(),
                    "photo".to_string(),
                ],
            }
        );
        
        styles.insert(
            "oil_painting".to_string(),
            ImageStyle {
                name: "Oil Painting".to_string(),
                modifiers: vec![
                    "oil painting".to_string(),
                    "brushstrokes".to_string(),
                    "classical art".to_string(),
                    "rich texture".to_string(),
                ],
                artists: vec![
                    "Claude Monet".to_string(),
                    "Vincent van Gogh".to_string(),
                ],
                negative_modifiers: vec![
                    "digital".to_string(),
                    "clean lines".to_string(),
                ],
            }
        );
        
        Self { styles }
    }
    
    /// Build prompt for image generation
    pub fn build_prompt(&self, description: &str, style: Option<&ImageStyle>) -> String {
        let style_name = style
            .map(|s| s.name.as_str())
            .unwrap_or("digital art");
        
        format!(
            "Generate {} detailed image prompts for: {}\n\n\
             For each prompt, include:\n\
             1. The main prompt (detailed description)\n\
             2. Art style (e.g., photorealistic, digital art, anime, oil painting)\n\
             3. Lighting (e.g., natural, studio, dramatic, soft)\n\
             4. Composition (e.g., close-up, wide shot, rule of thirds)\n\
             5. Color palette suggestions\n\
             6. Mood/atmosphere\n\
             7. Optional: negative prompts (what to avoid)\n\n\
             Desired style: {}\n\n\
             Prompts:\n",
            3, description, style_name
        )
    }
    
    /// Extract prompts from LLM response
    pub fn extract_prompts(&self, response: &str, num_variations: usize) -> Vec<ImagePrompt> {
        let mut prompts = Vec::new();
        
        // Parse numbered items
        for (i, line) in response.lines().enumerate() {
            let line = line.trim();
            
            if line.starts_with(|c: char| c.is_ascii_digit()) && line.contains('.') {
                // Extract prompt content
                let content = line
                    .split('.')
                    .nth(1)
                    .map(|s| s.trim())
                    .unwrap_or("");
                
                if !content.is_empty() {
                    prompts.push(ImagePrompt {
                        prompt: content.to_string(),
                        negative_prompt: None,
                        style: self.default_style(),
                        aspect_ratio: AspectRatio::Square,
                        quality_settings: QualitySettings::default(),
                    });
                }
            }
        }
        
        // If no numbered prompts found, create from whole text
        if prompts.is_empty() && !response.is_empty() {
            prompts.push(ImagePrompt {
                prompt: response.lines().next().unwrap_or("").to_string(),
                negative_prompt: None,
                style: self.default_style(),
                aspect_ratio: AspectRatio::Square,
                quality_settings: QualitySettings::default(),
            });
        }
        
        prompts.truncate(num_variations);
        prompts
    }
    
    /// Get style by name
    pub fn get_style(&self, name: &str) -> Option<&ImageStyle> {
        self.styles.get(&name.to_lowercase())
    }
    
    /// Default style
    fn default_style(&self) -> ImageStyle {
        ImageStyle {
            name: "Digital Art".to_string(),
            modifiers: vec!["digital art".to_string(), "detailed".to_string()],
            artists: Vec::new(),
            negative_modifiers: Vec::new(),
        }
    }
    
    /// Get available styles
    pub fn available_styles(&self) -> Vec<&str> {
        self.styles.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ImageSuggester {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for QualitySettings {
    fn default() -> Self {
        Self {
            quality: "high".to_string(),
            detail_level: "detailed".to_string(),
            lighting: "natural".to_string(),
        }
    }
}

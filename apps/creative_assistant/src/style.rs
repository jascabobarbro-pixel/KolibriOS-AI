//! Style Analyzer Module
//!
//! Analyzes and adjusts writing style.

use std::collections::HashMap;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use regex::Regex;

/// Style Analyzer
pub struct StyleAnalyzer {
    /// Common word patterns
    patterns: HashMap<String, Regex>,
}

/// Style profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleProfile {
    pub tone: String,
    pub formality: f32,
    pub complexity: f32,
    pub emotional_intensity: f32,
    pub dominant_features: Vec<String>,
}

/// Style analysis result
#[derive(Debug, Clone)]
pub struct StyleAnalysis {
    pub tone: String,
    pub readability_score: f32,
    pub avg_sentence_length: f32,
    pub vocabulary_level: String,
    pub recommendations: Vec<String>,
    pub processing_time_ms: u64,
}

/// Tone adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToneAdjustment {
    pub from_tone: String,
    pub to_tone: String,
    pub adjustments: Vec<TextAdjustment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAdjustment {
    pub original: String,
    pub replacement: String,
    pub reason: String,
}

impl StyleAnalyzer {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Sentence endings
        patterns.insert(
            "sentences".to_string(),
            Regex::new(r"[.!?]+").unwrap()
        );
        
        // Words
        patterns.insert(
            "words".to_string(),
            Regex::new(r"\b\w+\b").unwrap()
        );
        
        // Passive voice indicators
        patterns.insert(
            "passive".to_string(),
            Regex::new(r"(?i)\b(is|are|was|were|been|being)\s+\w+ed\b").unwrap()
        );
        
        // Complex words (3+ syllables estimate)
        patterns.insert(
            "complex_words".to_string(),
            Regex::new(r"\b\w{10,}\b").unwrap()
        );
        
        Self { patterns }
    }
    
    /// Analyze text style
    pub fn analyze(&self, text: &str) -> StyleAnalysis {
        let start = Instant::now();
        
        // Count sentences
        let sentence_count = self.patterns
            .get("sentences")
            .map(|r| r.find_iter(text).count())
            .unwrap_or(1)
            .max(1);
        
        // Count words
        let word_count = self.patterns
            .get("words")
            .map(|r| r.find_iter(text).count())
            .unwrap_or(1)
            .max(1);
        
        // Average sentence length
        let avg_sentence_length = word_count as f32 / sentence_count as f32;
        
        // Count passive voice instances
        let passive_count = self.patterns
            .get("passive")
            .map(|r| r.find_iter(text).count())
            .unwrap_or(0);
        
        // Count complex words
        let complex_count = self.patterns
            .get("complex_words")
            .map(|r| r.find_iter(text).count())
            .unwrap_or(0);
        
        // Calculate readability (simplified Flesch-Kincaid)
        let readability = self.calculate_readability(
            word_count,
            sentence_count,
            complex_count,
        );
        
        // Determine tone
        let tone = self.determine_tone(text, passive_count, avg_sentence_length);
        
        // Determine vocabulary level
        let vocabulary_level = if complex_count as f32 / word_count as f32 > 0.2 {
            "Advanced".to_string()
        } else if complex_count as f32 / word_count as f32 > 0.1 {
            "Intermediate".to_string()
        } else {
            "Basic".to_string()
        };
        
        // Generate recommendations
        let mut recommendations = Vec::new();
        
        if avg_sentence_length > 25.0 {
            recommendations.push("Consider breaking long sentences into shorter ones for clarity".to_string());
        }
        
        if passive_count > sentence_count / 2 {
            recommendations.push("Reduce passive voice usage for more engaging prose".to_string());
        }
        
        if readability < 50.0 {
            recommendations.push("Simplify complex sentences to improve readability".to_string());
        }
        
        if readability > 80.0 && vocabulary_level == "Advanced" {
            recommendations.push("Your writing is clear but could benefit from more sophisticated vocabulary".to_string());
        }
        
        StyleAnalysis {
            tone,
            readability_score: readability,
            avg_sentence_length,
            vocabulary_level,
            recommendations,
            processing_time_ms: start.elapsed().as_millis() as u64,
        }
    }
    
    /// Calculate readability score (0-100)
    fn calculate_readability(&self, words: usize, sentences: usize, complex: usize) -> f32 {
        if words == 0 || sentences == 0 {
            return 50.0;
        }
        
        // Simplified Flesch Reading Ease
        let avg_sentence_length = words as f32 / sentences as f32;
        let avg_syllables = (words as f32 + complex as f32 * 2.0) / words as f32;
        
        let score = 206.835 - (1.015 * avg_sentence_length) - (84.6 * avg_syllables);
        
        score.clamp(0.0, 100.0)
    }
    
    /// Determine tone of text
    fn determine_tone(&self, text: &str, passive_count: usize, avg_length: f32) -> String {
        let lower = text.to_lowercase();
        
        // Emotional indicators
        let emotional_words = [
            "amazing", "terrible", "wonderful", "awful", "incredible",
            "horrible", "fantastic", "disgusting", "love", "hate",
        ];
        
        let emotional_count = emotional_words
            .iter()
            .filter(|w| lower.contains(w.as_str()))
            .count();
        
        // Formal indicators
        let formal_words = [
            "therefore", "consequently", "furthermore", "moreover",
            "thus", "hence", "accordingly", "nevertheless",
        ];
        
        let formal_count = formal_words
            .iter()
            .filter(|w| lower.contains(w.as_str()))
            .count();
        
        // Casual indicators
        let casual_words = [
            "like", "kinda", "sorta", "gonna", "wanna",
            "yeah", "nope", "pretty", "stuff", "things",
        ];
        
        let casual_count = casual_words
            .iter()
            .filter(|w| lower.contains(w.as_str()))
            .count();
        
        // Determine primary tone
        if formal_count > emotional_count && formal_count > casual_count {
            "Formal".to_string()
        } else if emotional_count > casual_count {
            "Emotional".to_string()
        } else if casual_count > 2 {
            "Casual".to_string()
        } else if passive_count > 3 {
            "Passive/Detached".to_string()
        } else if avg_length < 15.0 {
            "Concise/Direct".to_string()
        } else if avg_length > 25.0 {
            "Elaborate/Detailed".to_string()
        } else {
            "Neutral/Professional".to_string()
        }
    }
}

impl Default for StyleAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

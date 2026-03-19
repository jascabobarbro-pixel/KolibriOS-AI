//! AI Bridge - Connects Koli runtime to AI cells

use alloc::string::String;

use super::RuntimeError;

/// AI Bridge
pub struct AiBridge {
    models: alloc::collections::BTreeMap<String, ModelInfo>,
}

/// Model information
pub struct ModelInfo {
    pub name: String,
    pub model_type: ModelType,
    pub loaded: bool,
}

/// Model types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelType {
    Language,
    Vision,
    Audio,
    Multimodal,
}

impl AiBridge {
    /// Create a new AI bridge
    pub fn new() -> Self {
        Self {
            models: alloc::collections::BTreeMap::new(),
        }
    }

    /// Invoke an AI model
    pub fn invoke(&mut self, model: &str, input: &str) -> Result<String, RuntimeError> {
        if let Some(info) = self.models.get(model) {
            if !info.loaded {
                return Err(RuntimeError::AiError(
                    alloc::format!("Model {} is not loaded", model)
                ));
            }

            // In a real implementation, this would call the AI cell
            Ok(String::from("AI response placeholder"))
        } else {
            Err(RuntimeError::AiError(
                alloc::format!("Model {} not found", model)
            ))
        }
    }

    /// Register a model
    pub fn register_model(&mut self, name: &str, model_type: ModelType) {
        self.models.insert(String::from(name), ModelInfo {
            name: String::from(name),
            model_type,
            loaded: true,
        });
    }

    /// Unregister a model
    pub fn unregister_model(&mut self, name: &str) {
        self.models.remove(name);
    }
}

impl Default for AiBridge {
    fn default() -> Self {
        Self::new()
    }
}

//! Gene Trait - Base trait for all kernel genes

use alloc::string::String;

use super::{GeneActivation, GeneDNA, GeneError, GeneRNA, GeneValue};

/// Base trait for kernel genes
pub trait Gene: Send + Sync {
    /// Get the gene's unique identifier
    fn id(&self) -> super::GeneId;

    /// Get the gene's name
    fn name(&self) -> &str;

    /// Get the gene's DNA (configuration)
    fn dna(&self) -> &GeneDNA;

    /// Get mutable access to DNA
    fn dna_mut(&mut self) -> &mut GeneDNA;

    /// Get the gene's RNA (runtime state)
    fn rna(&self) -> &GeneRNA;

    /// Get mutable access to RNA
    fn rna_mut(&mut self) -> &mut GeneRNA;

    /// Check if the gene is active
    fn is_active(&self) -> bool {
        self.rna().activity > self.dna().activation_threshold
    }

    /// Check if the gene is healthy
    fn is_healthy(&self) -> bool {
        self.rna().health > 0.5
    }

    /// Activate the gene with given input
    fn activate(&mut self, input: Option<&GeneValue>) -> Result<GeneActivation, GeneError>;

    /// Deactivate the gene
    fn deactivate(&mut self) {
        self.rna_mut().activity = 0.0;
    }

    /// Update the gene (called periodically)
    fn update(&mut self, _delta_ms: u64) -> Result<(), GeneError> {
        Ok(())
    }

    /// Get a configuration value
    fn get_config(&self, key: &str) -> Option<&GeneValue> {
        self.dna().config.get(key)
    }

    /// Set a configuration value
    fn set_config(&mut self, key: String, value: GeneValue) {
        self.dna_mut().config.insert(key, value);
    }

    /// Record an error
    fn record_error(&mut self, _error: &GeneError) {
        self.rna_mut().error_count += 1;
        self.rna_mut().health = (self.rna().health - 0.1).max(0.0);
    }

    /// Record successful activation
    fn record_success(&mut self) {
        let rna = self.rna_mut();
        rna.activation_count += 1;
        rna.health = (rna.health + 0.01).min(1.0);
    }
}

/// Gene registry for managing all genes
pub struct GeneRegistry {
    genes: alloc::collections::BTreeMap<String, alloc::boxed::Box<dyn Gene>>,
}

impl GeneRegistry {
    /// Create a new gene registry
    pub fn new() -> Self {
        Self {
            genes: alloc::collections::BTreeMap::new(),
        }
    }

    /// Register a gene
    pub fn register<G: Gene + 'static>(&mut self, gene: G) {
        let name = gene.name().to_string();
        self.genes.insert(name, alloc::boxed::Box::new(gene));
    }

    /// Get a gene by name
    pub fn get(&self, name: &str) -> Option<&dyn Gene> {
        self.genes.get(name).map(|g| g.as_ref())
    }

    /// Get a mutable gene by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut dyn Gene> {
        self.genes.get_mut(name).map(|g| g.as_mut())
    }

    /// Activate a gene by name
    pub fn activate(
        &mut self,
        name: &str,
        input: Option<&GeneValue>,
    ) -> Result<GeneActivation, GeneError> {
        if let Some(gene) = self.genes.get_mut(name) {
            gene.activate(input)
        } else {
            Err(GeneError::NotFound(name.to_string()))
        }
    }

    /// Update all genes
    pub fn update_all(&mut self, delta_ms: u64) -> Result<(), GeneError> {
        for gene in self.genes.values_mut() {
            gene.update(delta_ms)?;
        }
        Ok(())
    }

    /// List all gene names
    pub fn list(&self) -> alloc::vec::Vec<&str> {
        self.genes.keys().map(|s| s.as_str()).collect()
    }

    /// Get health status of all genes
    pub fn health_status(&self) -> alloc::collections::BTreeMap<String, f32> {
        self.genes
            .iter()
            .map(|(name, gene)| (name.clone(), gene.rna().health))
            .collect()
    }
}

impl Default for GeneRegistry {
    fn default() -> Self {
        Self::new()
    }
}

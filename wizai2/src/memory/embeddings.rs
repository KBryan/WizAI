use anyhow::Result;
use tracing::warn;

pub struct EmbeddingGenerator {
    // Will use Venice AI when available
}

impl EmbeddingGenerator {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn generate(&self, text: &str) -> Result<Vec<f32>> {
        // Placeholder - Venice AI doesn't have embeddings yet
        // For now, return empty vector
        warn!("Embeddings not available yet");
        Ok(Vec::new())
    }
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (norm_a * norm_b)
}

// compute/mod.rs
pub mod cache;
pub mod embeddings;
pub mod similarity;

// Re-export core functions
pub use similarity::{
    compute_similarity_batch, cosine_similarity_search, create_and_search_memory_index,
    find_similar_documents, similarity_search_filtered, MemorySearchResult,
    SimilarityConfig,
};

pub use embeddings::{embed_document_chunks, embed_query_text};

pub use crate::types::{EmbeddingModel};

pub use cache::{cleanup_cache, clear_cache, get_cache_stats, invalidate_collection_cache};

/// Validate embedding vector
pub fn validate_embedding(embedding: &[f32]) -> Result<(), String> {
    if embedding.is_empty() {
        return Err("Embedding is empty".to_string());
    }

    for (i, &val) in embedding.iter().enumerate() {
        if !val.is_finite() {
            return Err(format!("Invalid value at position {}: {}", i, val));
        }
    }

    Ok(())
}

/// Calculate vector norm
pub fn calculate_norm(embedding: &[f32]) -> Result<f32, String> {
    validate_embedding(embedding)?;

    let norm_squared: f32 = embedding.iter().map(|x| x * x).sum();
    let norm = norm_squared.sqrt();

    if !norm.is_finite() || norm <= 0.0 {
        return Err(format!("Invalid norm: {}", norm));
    }

    Ok(norm)
}

/// Calculate cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32], norm_a: f32, norm_b: f32) -> Result<f64, String> {
    if a.len() != b.len() {
        return Err("Dimension mismatch".to_string());
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let similarity = (dot_product / (norm_a * norm_b)) as f64;

    if !similarity.is_finite() {
        return Err("Invalid similarity result".to_string());
    }

    Ok(similarity)
}

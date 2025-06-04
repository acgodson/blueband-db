// compute/similarity.rs - OPTIMIZED WITH HIERARCHICAL INDEX
use crate::types::*;
use crate::storage;
use std::collections::HashMap; 
use super::{cosine_similarity, validate_embedding, calculate_norm};
use candid::CandidType;
use serde::{Serialize, Deserialize};

/// Simple, focused configuration for similarity search
#[derive(Clone, Debug)]
pub struct SimilarityConfig {
    pub min_score: Option<f64>,
    pub max_results: u32,
    pub use_approximate: bool,  // NEW: Enable fast approximate search
    pub candidate_factor: f32,  // NEW: How many candidates to consider (2.0 = 2x max_results)
}

impl Default for SimilarityConfig {
    fn default() -> Self {
        Self {
            min_score: None,
            max_results: 10,
            use_approximate: true,   // Default to fast search
            candidate_factor: 3.0,   // Consider 3x candidates for better accuracy
        }
    }
}

/// Hierarchical vector index for fast approximate search
#[derive(Clone, Debug)]
struct VectorIndex {
    centroids: Vec<Vec<f32>>,    
    clusters: Vec<Vec<usize>>,     
    vectors: Vec<Vector>,   
    dimensions: usize,
    cluster_count: usize,
}

impl VectorIndex {
  
  /// Build hierarchical index from vectors
  fn build(vectors: Vec<Vector>, target_clusters: usize) -> Self {
    if vectors.is_empty() {
        return Self {
            centroids: Vec::new(),
            clusters: Vec::new(),
            vectors,
            dimensions: 0,
            cluster_count: 0,
        };
    }

    let dimensions = vectors[0].embedding.len();  // Remove underscore prefix
    let actual_clusters = target_clusters.min(vectors.len());
    
    // Use k-means clustering to create hierarchy
    let (centroids, assignments) = simple_kmeans(&vectors, actual_clusters);
    
    // Group vectors by cluster assignment
    let mut clusters = vec![Vec::new(); actual_clusters];
    for (vector_idx, cluster_idx) in assignments.iter().enumerate() {
        clusters[*cluster_idx].push(vector_idx);
    }

    Self {
        centroids,
        clusters,
        vectors,
        dimensions,
        cluster_count: actual_clusters,
    }
}


    /// Fast approximate search using hierarchical index
    fn search_approximate(&self, query: &[f32], config: &SimilarityConfig) -> Result<Vec<(f64, Vector)>, String> {
        if self.centroids.is_empty() {
            return Ok(Vec::new());
        }

        let query_norm = calculate_norm(query)?;
        let candidate_count = (config.max_results as f32 * config.candidate_factor) as usize;

        // Step 1: Find most promising clusters (coarse search)
        let promising_clusters = self.find_best_clusters(query, query_norm, candidate_count)?;

        // Step 2: Search within promising clusters (fine search)
        let mut candidates = Vec::new();
        for cluster_idx in promising_clusters {
            for &vector_idx in &self.clusters[cluster_idx] {
                let vector = &self.vectors[vector_idx];
                
                if vector.embedding.len() != query.len() {
                    continue;
                }

                if let Ok(similarity) = cosine_similarity(query, &vector.embedding, query_norm, vector.norm) {
                    if let Some(min_score) = config.min_score {
                        if similarity < min_score {
                            continue;
                        }
                    }
                    candidates.push((similarity, vector.clone())); // Clone to avoid lifetime issues
                }
            }
        }

        // Step 3: Sort and return top candidates
        candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        candidates.truncate(config.max_results as usize);

        Ok(candidates)
    }

    /// Find most promising clusters to search
    fn find_best_clusters(&self, query: &[f32], query_norm: f32, candidate_count: usize) -> Result<Vec<usize>, String> {
        let mut cluster_scores = Vec::new();

        // Calculate similarity to each cluster centroid
        for (cluster_idx, centroid) in self.centroids.iter().enumerate() {
            if let Ok(centroid_norm) = calculate_norm(centroid) {
                if let Ok(similarity) = cosine_similarity(query, centroid, query_norm, centroid_norm) {
                    cluster_scores.push((similarity, cluster_idx));
                }
            }
        }

        // Sort clusters by similarity to query
        cluster_scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Select top clusters that contain enough candidates
        let mut selected_clusters = Vec::new();
        let mut total_candidates = 0;

        for (_, cluster_idx) in cluster_scores {
            selected_clusters.push(cluster_idx);
            total_candidates += self.clusters[cluster_idx].len();
            
            // Stop when we have enough candidates
            if total_candidates >= candidate_count {
                break;
            }
        }

        Ok(selected_clusters)
    }

    /// Get total number of vectors
    fn len(&self) -> usize {
        self.vectors.len()
    }
}

/// Simple k-means clustering for building the index
fn simple_kmeans(vectors: &[Vector], k: usize) -> (Vec<Vec<f32>>, Vec<usize>) {
    if vectors.is_empty() || k == 0 {
        return (Vec::new(), Vec::new());
    }

    let dimensions = vectors[0].embedding.len();
    let k = k.min(vectors.len());

    // Initialize centroids using k-means++ method
    let mut centroids = kmeans_plus_plus_init(vectors, k);
    let mut assignments = vec![0; vectors.len()];

    // Run k-means iterations
    for _iteration in 0..10 {  // Max 10 iterations for efficiency
        let mut changed = false;

        // Assign vectors to closest centroids
        for (vector_idx, vector) in vectors.iter().enumerate() {
            let mut best_cluster = 0;
            let mut best_distance = f32::INFINITY;

            for (cluster_idx, centroid) in centroids.iter().enumerate() {
                let distance = euclidean_distance(&vector.embedding, centroid);
                if distance < best_distance {
                    best_distance = distance;
                    best_cluster = cluster_idx;
                }
            }

            if assignments[vector_idx] != best_cluster {
                assignments[vector_idx] = best_cluster;
                changed = true;
            }
        }

        if !changed {
            break; // Converged
        }

        // Update centroids
        centroids = update_centroids(vectors, &assignments, k, dimensions);
    }

    (centroids, assignments)
}

/// Initialize centroids using k-means++ for better clustering
fn kmeans_plus_plus_init(vectors: &[Vector], k: usize) -> Vec<Vec<f32>> {
    if vectors.is_empty() {
        return Vec::new();
    }

    let mut centroids = Vec::new();
    let dimensions = vectors[0].embedding.len();

    // First centroid: random vector
    centroids.push(vectors[0].embedding.clone());

    // Subsequent centroids: prefer vectors far from existing centroids
    for _ in 1..k {
        let mut max_min_distance = 0.0f32;
        let mut best_vector_idx = 0;

        for (vector_idx, vector) in vectors.iter().enumerate() {
            // Find distance to closest existing centroid
            let min_distance = centroids.iter()
                .map(|centroid| euclidean_distance(&vector.embedding, centroid))
                .fold(f32::INFINITY, f32::min);

            if min_distance > max_min_distance {
                max_min_distance = min_distance;
                best_vector_idx = vector_idx;
            }
        }

        centroids.push(vectors[best_vector_idx].embedding.clone());
    }

    centroids
}

/// Update centroids based on current assignments
fn update_centroids(vectors: &[Vector], assignments: &[usize], k: usize, dimensions: usize) -> Vec<Vec<f32>> {
    let mut centroids = vec![vec![0.0; dimensions]; k];
    let mut counts = vec![0; k];

    // Sum up vectors in each cluster
    for (vector_idx, vector) in vectors.iter().enumerate() {
        let cluster_idx = assignments[vector_idx];
        for (dim, &value) in vector.embedding.iter().enumerate() {
            centroids[cluster_idx][dim] += value;
        }
        counts[cluster_idx] += 1;
    }

    // Average to get centroids
    for (cluster_idx, centroid) in centroids.iter_mut().enumerate() {
        if counts[cluster_idx] > 0 {
            let count = counts[cluster_idx] as f32;
            for value in centroid.iter_mut() {
                *value /= count;
            }
        }
    }

    centroids
}

/// Calculate euclidean distance between two vectors
fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter())
        .map(|(&x, &y)| (x - y) * (x - y))
        .sum::<f32>()
        .sqrt()
}

/// Core similarity search - OPTIMIZED with hierarchical index
pub fn cosine_similarity_search(
    query_embedding: &[f32],
    collection_id: &str,
    config: &SimilarityConfig,
) -> Result<Vec<VectorMatch>, String> {
    // Validate query embedding
    validate_embedding(query_embedding)?;

    // Get vectors from storage (with caching)
    let vectors = super::cache::get_cached_vectors(collection_id);
    
    if vectors.is_empty() {
        return Ok(Vec::new());
    }

    let scored_vectors = if config.use_approximate && vectors.len() > 1000 {
        // Use fast approximate search for large collections
        let target_clusters = (vectors.len() / 100).max(10).min(100); // 10-100 clusters
        let index = VectorIndex::build(vectors, target_clusters);
        index.search_approximate(query_embedding, config)?
    } else {
        // Use exact search for small collections or when requested
        exact_similarity_search(query_embedding, &vectors, config)?
    };

    // Build enriched results - each match represents a document chunk
    let mut matches = Vec::new();
    
    for (score, vector) in scored_vectors {
        let vector_match = VectorMatch {
            score,
            document_id: vector.document_id.clone(),
            chunk_id: vector.chunk_id.clone(),
            // Enrich with actual content
            document_title: storage::get_document_title(collection_id, &vector.document_id),
            chunk_text: storage::get_chunk_text(&vector.document_id, &vector.chunk_id),
        };

        matches.push(vector_match);
    }

    Ok(matches)
}

/// Exact similarity search (original algorithm)
fn exact_similarity_search(
    query_embedding: &[f32],
    vectors: &[Vector],
    config: &SimilarityConfig,
) -> Result<Vec<(f64, Vector)>, String> {
    // Calculate query norm once
    let query_norm = calculate_norm(query_embedding)?;

    // Compute similarities and collect results
    let mut scored_vectors = Vec::new();
    
    for vector in vectors {
        // Skip dimension mismatches
        if vector.embedding.len() != query_embedding.len() {
            continue;
        }

        // Calculate cosine similarity
        match cosine_similarity(query_embedding, &vector.embedding, query_norm, vector.norm) {
            Ok(similarity) => {
                // Apply minimum score filter if specified
                if let Some(min_score) = config.min_score {
                    if similarity < min_score {
                        continue;
                    }
                }

                scored_vectors.push((similarity, vector.clone())); // Clone to avoid lifetime issues
            }
            Err(_) => continue, // Skip invalid vectors
        }
    }

    // Sort by similarity score (descending) for top-k results
    scored_vectors.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    // Apply result limit
    scored_vectors.truncate(config.max_results as usize);

    Ok(scored_vectors)
}

/// Find similar documents to a given source document
pub fn find_similar_documents(
    source_document_id: &str,
    collection_id: &str,
    config: &SimilarityConfig,
) -> Result<Vec<VectorMatch>, String> {
    // Get all vectors for the source document
    let source_vectors = storage::get_document_vectors(source_document_id);
    
    if source_vectors.is_empty() {
        return Err(format!("No vectors found for document: {}", source_document_id));
    }

    // Calculate document centroid for better similarity representation
    let centroid_embedding = calculate_document_centroid(&source_vectors)?;
    
    // Perform similarity search using the centroid
    let mut matches = cosine_similarity_search(&centroid_embedding, collection_id, config)?;

    // Remove chunks from the source document itself
    matches.retain(|m| m.document_id != source_document_id);

    Ok(matches)
}

/// Batch similarity search for multiple queries
pub fn compute_similarity_batch(
    query_embeddings: &[Vec<f32>],
    collection_id: &str,
    config: &SimilarityConfig,
) -> Result<Vec<Vec<VectorMatch>>, String> {
    let mut results = Vec::new();

    for query_embedding in query_embeddings {
        let matches = cosine_similarity_search(query_embedding, collection_id, config)?;
        results.push(matches);
    }

    Ok(results)
}

/// Simple metadata filtering by document IDs (OPTIMIZED)
pub fn similarity_search_filtered(
    query_embedding: &[f32],
    collection_id: &str,
    document_filter: Option<&[String]>,
    config: &SimilarityConfig,
) -> Result<Vec<VectorMatch>, String> {
    validate_embedding(query_embedding)?;

    let vectors = super::cache::get_cached_vectors(collection_id);
    
    if vectors.is_empty() {
        return Ok(Vec::new());
    }

    // Pre-filter vectors by document filter
    let filtered_vectors: Vec<&Vector> = if let Some(allowed_docs) = document_filter {
        vectors.iter()
            .filter(|vector| allowed_docs.contains(&vector.document_id))
            .collect()
    } else {
        vectors.iter().collect()
    };

    // Convert to owned vectors for the search
    let owned_vectors: Vec<Vector> = filtered_vectors.into_iter().cloned().collect();

    // Use the same optimized search logic
    let scored_vectors = if config.use_approximate && owned_vectors.len() > 1000 {
        let target_clusters = (owned_vectors.len() / 100).max(10).min(100);
        let index = VectorIndex::build(owned_vectors, target_clusters);
        index.search_approximate(query_embedding, config)?
    } else {
        exact_similarity_search(query_embedding, &owned_vectors, config)?
    };

    // Build enriched results
    let mut matches = Vec::new();
    
    for (score, vector) in scored_vectors {
        let vector_match = VectorMatch {
            score,
            document_id: vector.document_id.clone(),
            chunk_id: vector.chunk_id.clone(),
            document_title: storage::get_document_title(collection_id, &vector.document_id),
            chunk_text: storage::get_chunk_text(&vector.document_id, &vector.chunk_id),
        };

        matches.push(vector_match);
    }

    Ok(matches)
}

/// Calculate document centroid from its vectors for document-level similarity
fn calculate_document_centroid(vectors: &[Vector]) -> Result<Vec<f32>, String> {
    if vectors.is_empty() {
        return Err("Cannot calculate centroid of empty vector set".to_string());
    }

    let dimension = vectors[0].embedding.len();
    
    // Verify all vectors have same dimension
    for vector in vectors {
        if vector.embedding.len() != dimension {
            return Err(format!(
                "Dimension mismatch: expected {}, got {}",
                dimension, vector.embedding.len()
            ));
        }
    }

    // Calculate centroid by averaging all vectors
    let mut centroid = vec![0.0f32; dimension];
    
    for vector in vectors {
        for (i, &val) in vector.embedding.iter().enumerate() {
            centroid[i] += val;
        }
    }

    let count = vectors.len() as f32;
    for val in &mut centroid {
        *val /= count;
    }

    // Validate the resulting centroid
    validate_embedding(&centroid)?;

    Ok(centroid)
}

// =============================================================================
// IN-MEMORY VECTOR INDEX FOR DEMO/TESTING (UNCHANGED)
// =============================================================================

/// In-memory vector item for demo/testing
#[derive(Clone, Debug)]
pub struct MemoryVectorItem {
    pub id: String,
    pub text: String,
    pub embedding: Vec<f32>,
    pub norm: f32,
}

/// In-memory vector index for testing/demo without persistence
pub struct MemoryVectorIndex {
    items: Vec<MemoryVectorItem>,
    metadata: HashMap<String, String>,
}

impl MemoryVectorIndex {
    /// Create new empty in-memory index
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add item to memory index with automatic embedding generation
    pub async fn add_item_with_embedding(
        &mut self,
        id: String,
        text: String,
        model: EmbeddingModel,
        proxy_url: String,
    ) -> Result<(), String> {
        // Generate embedding for the text
        let (embedding, norm) = super::embeddings::embed_query_text(&text, model, proxy_url).await?;
        
        let item = MemoryVectorItem {
            id: id.clone(),
            text,
            embedding,
            norm,
        };
        
        self.items.push(item);
        Ok(())
    }

    /// Add pre-embedded item to memory index
    pub fn add_item(&mut self, item: MemoryVectorItem) {
        self.items.push(item);
    }

    /// Search similar items in memory index
    pub fn search_similar(
        &self,
        query_embedding: &[f32],
        query_norm: f32,
        max_results: usize,
        min_score: Option<f64>,
    ) -> Result<Vec<MemorySearchResult>, String> {
        let mut scored_items = Vec::new();

        for item in &self.items {
            // Check dimension compatibility
            if item.embedding.len() != query_embedding.len() {
                continue;
            }

            // Calculate cosine similarity
            match cosine_similarity(query_embedding, &item.embedding, query_norm, item.norm) {
                Ok(similarity) => {
                    // Apply minimum score filter
                    if let Some(min) = min_score {
                        if similarity < min {
                            continue;
                        }
                    }

                    scored_items.push((similarity, item));
                }
                Err(_) => continue,
            }
        }

        // Sort by similarity (descending)
        scored_items.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        // Limit results
        scored_items.truncate(max_results);

        // Convert to search results
        let results = scored_items
            .into_iter()
            .map(|(score, item)| MemorySearchResult {
                document_id: item.id.clone(),
                chunk_id: "".to_string(),
                score,
                text: item.text.clone(),
            })
            .collect();

        Ok(results)
    }

    /// Search with query text (auto-embedding)
    pub async fn search_with_text(
        &self,
        query_text: &str,
        model: EmbeddingModel,
        proxy_url: String,
        max_results: usize,
        min_score: Option<f64>,
    ) -> Result<Vec<MemorySearchResult>, String> {
        // Generate embedding for query
        let (query_embedding, query_norm) = super::embeddings::embed_query_text(query_text, model, proxy_url).await?;
        
        // Search with embedding
        self.search_similar(&query_embedding, query_norm, max_results, min_score)
    }

    /// Get item count
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
        self.metadata.clear();
    }
}

/// Search result from memory index
#[derive(CandidType, Clone, Debug, Serialize, Deserialize)]
pub struct MemorySearchResult {
    pub document_id: String,
    pub chunk_id: String,
    pub score: f64,
    pub text: String,
}

/// Create and search in-memory vector index with custom data
pub async fn create_and_search_memory_index(
    items: Vec<String>,           // Items to add to index
    query: String,                // What to search for
    model: EmbeddingModel,
    proxy_url: String,
    max_results: usize,
    min_score: Option<f64>,
) -> Result<Vec<MemorySearchResult>, String> {
    let mut index = MemoryVectorIndex::new();

    // Add all items to index
    for (i, item_text) in items.iter().enumerate() {
        let id = format!("item_{}", i);
        index.add_item_with_embedding(id, item_text.clone(), model.clone(), proxy_url.clone()).await?;
    }

    // Search with the query
    index.search_with_text(&query, model, proxy_url, max_results, min_score).await
}

/// Batch add multiple texts to memory index
pub async fn batch_add_to_memory_index(
    index: &mut MemoryVectorIndex,
    texts: Vec<(String, String)>, // (id, text) pairs
    model: EmbeddingModel,
    proxy_url: String,
) -> Result<(), String> {
    for (id, text) in texts {
        index.add_item_with_embedding(id, text, model.clone(), proxy_url.clone()).await?;
    }
    Ok(())
}
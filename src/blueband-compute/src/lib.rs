// src/blueband-compute/src/lib.rs - Updated with fixes
use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::call::call;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use ic_cdk_macros::{update, query};
use serde::Serialize;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_TOP_RESULTS: usize = 5;
const CACHE_TTL_SECS: u64 = 86400; // 24 hours
const SCALE_FACTOR: f64 = 1_000_000.0;
const OFFSET_VALUE: f64 = 10.0;

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct Vector {
    pub id: String,
    pub document_id: String,
    pub chunk_id: String,
    pub embedding: Vec<f32>,
}

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct MetadataFilter {
    pub document_ids: Option<Vec<String>>,
    pub chunk_ids: Option<Vec<String>>,
    pub limit: Option<u64>,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct ScoredMatch {
    pub score: f64,
    pub document_id: String,
    pub chunk_id: String,
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct QueryResult {
    pub matches: Vec<ScoredMatch>,
}

#[derive(CandidType, Deserialize, Serialize)]
pub enum EmbeddingResult {
    Scaled(ScaledEmbedding),
    Float(FloatEmbedding),
}

// FIXED: Updated CacheStats to use u64 consistently (aligns with Motoko Nat64)
#[derive(CandidType, Deserialize, Serialize)]
pub struct CacheStats {
    pub cache_size: u64,
    pub hits: u64,
    pub misses: u64,
    pub memory_usage: u64,
}

#[derive(Default, Clone)]
struct CacheEntry {
    vector_index: Vec<(Vector, Vec<f64>)>, // normalized vectors
    timestamp: u64,
    hits: u64,
}

// FIXED: Updated ScaledEmbedding to use u64 consistently (aligns with Motoko Nat64)
#[derive(CandidType, Deserialize, Serialize)]
pub struct ScaledEmbedding {
    pub values: Vec<Vec<u64>>, // Consistent with Motoko Nat64
    pub norms: Vec<u64>,       // Consistent with Motoko Nat64
}

#[derive(CandidType, Deserialize, Serialize)]
pub struct FloatEmbedding {
    pub embeddings: Vec<Vec<f64>>,
    pub norm_values: Vec<f64>,
}

thread_local! {
    static INDEX_CACHE: std::cell::RefCell<HashMap<String, CacheEntry>> = Default::default();
    static CACHE_STATS: std::cell::RefCell<CacheStats> = std::cell::RefCell::new(CacheStats {
        cache_size: 0,
        hits: 0,
        misses: 0,
        memory_usage: 0,
    });
}

#[update]
pub async fn query_text(
    query: String,
    storage_principal: String,
    filter: Option<MetadataFilter>
) -> Result<QueryResult, String> {
    // FIXED: Enhanced error handling with validation
    if query.trim().is_empty() {
        return Err("Query cannot be empty".to_string());
    }

    if storage_principal.trim().is_empty() {
        return Err("Storage principal cannot be empty".to_string());
    }

    // Check cache first
    let cache_hit = INDEX_CACHE.with(|cache| {
        if let Some(entry) = cache.borrow().get(&storage_principal) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if now - entry.timestamp < CACHE_TTL_SECS {
                CACHE_STATS.with(|stats| {
                    let mut stats = stats.borrow_mut();
                    stats.hits += 1;
                });
                return true;
            }
        }
        CACHE_STATS.with(|stats| {
            let mut stats = stats.borrow_mut();
            stats.misses += 1;
        });
        false
    });

    if !cache_hit {
        // FIXED: Enhanced error handling in cache population
        if let Err(e) = populate_cache(storage_principal.clone(), filter.clone()).await {
            return Err(format!("Failed to populate cache: {}", e));
        }
    }

    // Generate embedding for the query
    let default_proxy_url = "https://us-central1-blueband-db-442d8.cloudfunctions.net/proxy".to_string();
    let embeddings = fetch_embeddings(vec![query.clone()], default_proxy_url, false).await?;
    
    let (query_embedding, _query_norm) = match embeddings {
        EmbeddingResult::Float(ref float_emb) => {
            if float_emb.embeddings.is_empty() {
                return Err("Failed to generate embedding for query".to_string());
            }
            // FIXED: Validate embedding before using
            let embedding = &float_emb.embeddings[0];
            if embedding.is_empty() {
                return Err("Generated embedding is empty".to_string());
            }
            (embedding, float_emb.norm_values[0])
        },
        EmbeddingResult::Scaled(_) => {
            return Err("Scaled embeddings are not supported for query text".to_string());
        }
    };

    // Get vectors from cache and compute similarity
    let results = INDEX_CACHE.with(|cache| {
        let cache = cache.borrow();
        let entry = cache.get(&storage_principal).unwrap(); // Safe because we just populated it if needed
        
        let mut results: Vec<ScoredMatch> = entry.vector_index.iter()
            .filter_map(|(vector, normalized_vec)| {
                // FIXED: Enhanced validation
                if normalized_vec.len() != query_embedding.len() {
                    eprintln!("Warning: Vector dimension mismatch for vector {}: {} vs {}", 
                             vector.id, normalized_vec.len(), query_embedding.len());
                    return None;
                }
                
                let dot: f64 = normalized_vec.iter()
                    .zip(query_embedding.iter())
                    .map(|(a, b)| a * b)
                    .sum();
                
                // FIXED: Validate similarity score
                if !dot.is_finite() {
                    eprintln!("Warning: Invalid similarity score for vector {}", vector.id);
                    return None;
                }
                
                Some(ScoredMatch {
                    score: dot,
                    document_id: vector.document_id.clone(),
                    chunk_id: vector.chunk_id.clone(),
                })
            })
            .collect();

        // Sort by score in descending order
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Apply limit if specified in filter
        if let Some(ref f) = filter {
            if let Some(limit) = f.limit {
                results.truncate(limit as usize);
            } else {
                results.truncate(MAX_TOP_RESULTS);
            }
        } else {
            results.truncate(MAX_TOP_RESULTS);
        }

        results
    });

    // Update hit count for this cache entry
    INDEX_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(entry) = cache.get_mut(&storage_principal) {
            entry.hits += 1;
        }
    });

    Ok(QueryResult { matches: results })
}

#[update]
pub async fn generate_embeddings(texts: Vec<String>, proxy_url: String, use_scaled: bool) -> Result<EmbeddingResult, String> {
    // FIXED: Enhanced input validation
    if texts.is_empty() {
        return Err("Cannot generate embeddings for empty text list".to_string());
    }

    if proxy_url.trim().is_empty() {
        return Err("Proxy URL cannot be empty".to_string());
    }

    // FIXED: Validate individual texts
    for (i, text) in texts.iter().enumerate() {
        if text.trim().is_empty() {
            return Err(format!("Text at index {} is empty", i));
        }
        if text.len() > 100000 { // Reasonable limit
            return Err(format!("Text at index {} is too long ({} chars)", i, text.len()));
        }
    }

    fetch_embeddings(texts, proxy_url, use_scaled).await
}

#[update]
pub fn invalidate_cache(storage_principal: String) {
    // FIXED: Add validation
    if storage_principal.trim().is_empty() {
        return; // Silent fail for empty principal
    }

    INDEX_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.remove(&storage_principal).is_some() {
            println!("Cache invalidated for principal: {}", storage_principal);
        }
    });
}

#[query]
pub fn get_cache_stats() -> CacheStats {
    CACHE_STATS.with(|stats| {
        let cache_size = INDEX_CACHE.with(|cache| {
            cache.borrow().len() as u64
        });
        
        let stats = stats.borrow();
        CacheStats {
            cache_size,
            hits: stats.hits,
            misses: stats.misses,
            memory_usage: stats.memory_usage,
        }
    })
}

// FIXED: Enhanced populate_cache with better error handling
async fn populate_cache(principal: String, filter: Option<MetadataFilter>) -> Result<(), String> {
    let principal_id = Principal::from_text(&principal)
        .map_err(|e| format!("Invalid principal '{}': {}", principal, e))?;
    
    // FIXED: Call the correct getVectors method that now exists in Motoko bucket
    let call_result: Result<(Vec<Vector>,), _> = call(
        principal_id, 
        "getVectors", 
        (filter,)
    ).await;
    
    match call_result {
        Ok((vectors,)) => {
            // FIXED: Enhanced validation of returned vectors
            if vectors.is_empty() {
                println!("Warning: No vectors returned from storage canister {}", principal);
                // Don't return error - empty is valid
            }

            let mut vector_index = Vec::new();
            let mut invalid_count = 0;
            
            for vector in vectors.iter() {
                // FIXED: Validate vector before processing
                if vector.embedding.is_empty() {
                    invalid_count += 1;
                    continue;
                }

                // Convert f32 embedding to f64 for better precision
                let normed: Vec<f64> = vector.embedding.iter().map(|&x| x as f64).collect();
                let norm = (normed.iter().map(|x| x * x).sum::<f64>()).sqrt();
                
                // FIXED: Validate norm
                if norm <= 0.0 || !norm.is_finite() {
                    invalid_count += 1;
                    eprintln!("Warning: Invalid norm {} for vector {}", norm, vector.id);
                    continue;
                }

                // Normalize the vector for faster cosine similarity calculation
                let normalized: Vec<f64> = normed.iter().map(|x| x / norm).collect();
                
                // FIXED: Validate normalized vector
                if normalized.iter().any(|&x| !x.is_finite()) {
                    invalid_count += 1;
                    eprintln!("Warning: Invalid normalized values for vector {}", vector.id);
                    continue;
                }

                vector_index.push((vector.clone(), normalized));
            }
            
            if invalid_count > 0 {
                println!("Warning: Skipped {} invalid vectors out of {}", invalid_count, vectors.len());
            }
            
            // Update the cache
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            
            INDEX_CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();
                cache.insert(principal.clone(), CacheEntry {
                    vector_index,
                    timestamp: now,
                    hits: 0,
                });
            });
            
            println!("Successfully cached {} valid vectors for principal {}", 
                    vectors.len() - invalid_count, principal);
            Ok(())
        },
        Err(e) => {
            let error_msg = format!("Call to Motoko canister {} failed: {:?}", principal, e);
            eprintln!("{}", error_msg);
            Err(error_msg)
        }
    }
}

// FIXED: Enhanced fetch_embeddings with comprehensive error handling
// Complete fetch_embeddings function with all fixes
async fn fetch_embeddings(texts: Vec<String>, proxy_url: String, use_scaled: bool) -> Result<EmbeddingResult, String> {
    if texts.is_empty() {
        return Ok(EmbeddingResult::Float(FloatEmbedding {
            embeddings: vec![],
            norm_values: vec![],
        }));
    }

    // FIXED: Validate proxy URL format
    if !proxy_url.starts_with("http://") && !proxy_url.starts_with("https://") {
        return Err(format!("Invalid proxy URL format: {}", proxy_url));
    }

    // Create the request body
    let request_body = match serde_json::to_string(&serde_json::json!({
        "input": texts,
        "model": "text-embedding-ada-002"
    })) {
        Ok(body) => body,
        Err(e) => return Err(format!("Failed to serialize request body: {}", e)),
    };

    // Set up the HTTP request
    let request_headers = vec![
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: proxy_url.clone(),
        method: HttpMethod::POST,
        body: Some(request_body.into_bytes()),
        max_response_bytes: Some(2_000_000), // 2MB limit
        transform: None,
        headers: request_headers,
    };

    // Make the HTTP request
    let cycles = 20_000_000_000u128; // 20B cycles for HTTP outcalls
    match http_request(request, cycles).await {
        Ok((response,)) => {
            if response.status < 200u16 || response.status >= 300u16 {
                return Err(format!("HTTP request failed with status {}", response.status));
            }

            let body = String::from_utf8(response.body)
                .map_err(|e| format!("Failed to decode response as UTF-8: {}", e))?;
            
            if body.trim().is_empty() {
                return Err("Received empty response body".to_string());
            }

            // Parse the JSON response
            let raw: serde_json::Value = serde_json::from_str(&body)
                .map_err(|e| format!("Failed to parse JSON response: {}", e))?;
            
            // FIXED: Enhanced JSON validation
            let data = raw["data"].as_array()
                .ok_or("Missing or invalid 'data' field in response")?;

            if data.len() != texts.len() {
                return Err(format!("Response contains {} embeddings but expected {}", 
                                 data.len(), texts.len()));
            }

            // Process the embeddings based on use_scaled flag
            if use_scaled {
                let mut scaled_values = Vec::new();
                let mut scaled_norms = Vec::new();
                
                for (i, item) in data.iter().enumerate() {
                    let emb_array = item["embedding"].as_array()
                        .ok_or_else(|| format!("Missing embedding for item {}", i))?;
                    
                    let embedding: Vec<f64> = emb_array
                        .iter()
                        .enumerate()
                        .map(|(j, v)| {
                            v.as_f64().ok_or_else(|| format!("Invalid float at position {} in embedding {}", j, i))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    
                    // FIXED: Validate embedding values
                    if embedding.is_empty() {
                        return Err(format!("Empty embedding for item {}", i));
                    }
                    
                    for (j, &val) in embedding.iter().enumerate() {
                        if !val.is_finite() {
                            return Err(format!("Invalid embedding value at position {} in item {}: {}", j, i, val));
                        }
                    }
                    
                    // Calculate norm
                    let norm = (embedding.iter().map(|x| x * x).sum::<f64>()).sqrt();
                    
                    // FIXED: Validate norm
                    if !norm.is_finite() || norm <= 0.0 {
                        return Err(format!("Invalid norm for item {}: {}", i, norm));
                    }
                    
                    // FIXED: Scale the values using u64 (consistent with Motoko Nat64)
                    let scaled_embedding: Vec<u64> = embedding.iter()
                        .map(|&x| {
                            let scaled = (x + OFFSET_VALUE) * SCALE_FACTOR;
                            // FIXED: Validate scaled value before conversion
                            if !scaled.is_finite() || scaled < 0.0 || scaled > u64::MAX as f64 {
                                0u64 // Fallback to 0 for invalid values
                            } else {
                                scaled as u64
                            }
                        })
                        .collect();
                    
                    let scaled_norm = {
                        let scaled = (norm + OFFSET_VALUE) * SCALE_FACTOR;
                        if !scaled.is_finite() || scaled < 0.0 || scaled > u64::MAX as f64 {
                            0u64 // Fallback to 0 for invalid values
                        } else {
                            scaled as u64
                        }
                    };
                    
                    scaled_values.push(scaled_embedding);
                    scaled_norms.push(scaled_norm);
                }

                Ok(EmbeddingResult::Scaled(ScaledEmbedding {
                    values: scaled_values,
                    norms: scaled_norms,
                }))
            } else {
                let mut embeddings = Vec::new();
                let mut norm_values = Vec::new();
                
                for (i, item) in data.iter().enumerate() {
                    let emb_array = item["embedding"].as_array()
                        .ok_or_else(|| format!("Missing embedding for item {}", i))?;
                    
                    let embedding: Vec<f64> = emb_array
                        .iter()
                        .enumerate()
                        .map(|(j, v)| {
                            v.as_f64().ok_or_else(|| format!("Invalid float at position {} in embedding {}", j, i))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    
                    // FIXED: Validate embedding values
                    if embedding.is_empty() {
                        return Err(format!("Empty embedding for item {}", i));
                    }
                    
                    for (j, &val) in embedding.iter().enumerate() {
                        if !val.is_finite() {
                            return Err(format!("Invalid embedding value at position {} in item {}: {}", j, i, val));
                        }
                    }
                    
                    // Calculate norm
                    let norm = (embedding.iter().map(|x| x * x).sum::<f64>()).sqrt();
                    
                    // FIXED: Validate norm
                    if !norm.is_finite() || norm <= 0.0 {
                        return Err(format!("Invalid norm for item {}: {}", i, norm));
                    }
                    
                    embeddings.push(embedding);
                    norm_values.push(norm);
                }

                Ok(EmbeddingResult::Float(FloatEmbedding {
                    embeddings,
                    norm_values,
                }))
            }
        }
        Err((r, m)) => {
            let error_msg = format!("HTTP request failed: {:?} - {}", r, m);
            eprintln!("{}", error_msg);
            Err(error_msg)
        }
    }
}

// Additional helper function for validation
fn validate_embedding_vector(embedding: &[f64]) -> Result<(), String> {
    if embedding.is_empty() {
        return Err("Embedding vector is empty".to_string());
    }
    
    for (i, &val) in embedding.iter().enumerate() {
        if !val.is_finite() {
            return Err(format!("Invalid value at position {}: {}", i, val));
        }
    }
    
    Ok(())
}

// Additional helper function for calculating and validating norm
fn calculate_and_validate_norm(embedding: &[f64]) -> Result<f64, String> {
    if embedding.is_empty() {
        return Err("Cannot calculate norm of empty vector".to_string());
    }
    
    let norm_squared: f64 = embedding.iter().map(|x| x * x).sum();
    
    if !norm_squared.is_finite() || norm_squared < 0.0 {
        return Err(format!("Invalid norm squared value: {}", norm_squared));
    }
    
    let norm = norm_squared.sqrt();
    
    if !norm.is_finite() || norm <= 0.0 {
        return Err(format!("Invalid norm value: {}", norm));
    }
    
    Ok(norm)
}

// FIXED: Enhanced scaling function with proper validation
fn scale_value_to_u64(value: f64) -> Result<u64, String> {
    let scaled = (value + OFFSET_VALUE) * SCALE_FACTOR;
    
    if !scaled.is_finite() {
        return Err(format!("Scaled value is not finite: {}", scaled));
    }
    
    if scaled < 0.0 {
        return Err(format!("Scaled value is negative: {}", scaled));
    }
    
    if scaled > u64::MAX as f64 {
        return Err(format!("Scaled value exceeds u64 max: {}", scaled));
    }
    
    Ok(scaled as u64)
}

// FIXED: Enhanced unscaling function for debugging/validation
fn unscale_u64_to_float(value: u64) -> f64 {
    (value as f64) / SCALE_FACTOR - OFFSET_VALUE
}

// Add to your Rust canister
#[update]
pub fn wallet_receive() -> u64 {
    let available = ic_cdk::api::call::msg_cycles_available128();
    let accepted = ic_cdk::api::call::msg_cycles_accept128(available);
    accepted as u64
} 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_unscale_roundtrip() {
        let original = 0.5;
        let scaled = scale_value_to_u64(original).unwrap();
        let unscaled = unscale_u64_to_float(scaled);
        
        // Should be approximately equal (within floating point precision)
        assert!((original - unscaled).abs() < 1e-6);
    }

    #[test]
    fn test_norm_validation() {
        let valid_embedding = vec![0.1, 0.2, 0.3];
        assert!(calculate_and_validate_norm(&valid_embedding).is_ok());
        
        let empty_embedding = vec![];
        assert!(calculate_and_validate_norm(&empty_embedding).is_err());
        
        let invalid_embedding = vec![f64::NAN, 0.2, 0.3];
        assert!(calculate_and_validate_norm(&invalid_embedding).is_err());
    }

    #[test]
    fn test_embedding_validation() {
        let valid_embedding = vec![0.1, 0.2, 0.3];
        assert!(validate_embedding_vector(&valid_embedding).is_ok());
        
        let empty_embedding = vec![];
        assert!(validate_embedding_vector(&empty_embedding).is_err());
        
        let invalid_embedding = vec![f64::INFINITY, 0.2, 0.3];
        assert!(validate_embedding_vector(&invalid_embedding).is_err());
    }
}
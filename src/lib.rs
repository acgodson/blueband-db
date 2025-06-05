// lib.rs - Complete Vector Database API Layer
use ic_cdk::api::caller;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use candid::CandidType;
use candid::export_service;
use serde::{Serialize, Deserialize};
use crate::storage::collections;
use crate::storage::documents;
use crate::storage::vectors;
use crate::compute::{
    cosine_similarity_search, similarity_search_filtered,
    MemorySearchResult, SimilarityConfig,
};
use crate::types::{
    SearchRequest, SearchResponse,
};
use crate::compute::EmbeddingModel;
use ic_cdk::api::management_canister::http_request::{TransformArgs, HttpResponse};


mod compute;
mod storage;
mod types;

pub use types::*;



// =============================================================================
// CANISTER LIFECYCLE
// =============================================================================

#[init]
fn init() {
    ic_cdk::println!("Vector Database canister initialized");
}

#[pre_upgrade]
fn pre_upgrade() {
    ic_cdk::println!("Starting canister upgrade...");
}

#[post_upgrade]
fn post_upgrade() {
    ic_cdk::println!("Canister upgrade completed");
}

// =============================================================================
// COLLECTION MANAGEMENT
// =============================================================================

#[update]
fn create_collection(request: CreateCollectionRequest) -> Result<Collection, String> {
    let caller = caller().to_string();
    storage::create_collection(request, caller)
}

#[query]
fn get_collection(collection_id: String) -> Option<Collection> {
    storage::get_collection(&collection_id)
}

#[query]
fn list_collections() -> Vec<Collection> {
    storage::list_collections()
}

#[query]
fn get_collection_with_stats(collection_id: String) -> Option<CollectionWithStats> {
    storage::get_collection_with_stats(&collection_id)
}

#[query]
fn list_collections_with_stats() -> Vec<CollectionWithStats> {
    storage::list_collections_with_stats()
}

#[update]
fn update_collection_settings(collection_id: String, settings: CollectionSettings) -> Result<(), String> {
    let caller = ic_cdk::caller().to_string();
    collections::update_collection_settings(&collection_id, settings, &caller)
}

#[update]
fn update_collection_metadata(
    collection_id: String,
    name: Option<String>,
    description: Option<String>,
) -> Result<(), String> {
    let caller = ic_cdk::caller().to_string();
    collections::update_collection_metadata(&collection_id, name, description, &caller)
}

#[update]
fn delete_collection(collection_id: String) -> Result<(), String> {
    let caller = ic_cdk::caller().to_string();
    collections::delete_collection(&collection_id, &caller)
}

#[update]
fn add_collection_admin(collection_id: String, new_admin: String) -> Result<(), String> {
    let caller = ic_cdk::caller().to_string();
    collections::add_collection_admin(&collection_id, &new_admin, &caller)
}

#[update]
fn remove_collection_admin(collection_id: String, admin_to_remove: String) -> Result<(), String> {
    let caller = ic_cdk::caller().to_string();
    collections::remove_collection_admin(&collection_id, &admin_to_remove, &caller)
}

#[update]
fn transfer_genesis_admin(collection_id: String, new_genesis_admin: String) -> Result<(), String> {
    let caller = ic_cdk::caller().to_string();
    collections::transfer_genesis_admin(&collection_id, &new_genesis_admin, &caller)
}

#[query]
fn is_collection_admin(collection_id: String, principal: String) -> bool {
    storage::is_collection_admin(&collection_id, &principal)
}

#[query]
fn get_my_admin_level(collection_id: String) -> String {
    let caller = ic_cdk::caller().to_string();
    match collections::get_admin_level(&collection_id, &caller) {
        collections::AdminLevel::Genesis => "genesis".to_string(),
        collections::AdminLevel::Regular => "regular".to_string(),
        collections::AdminLevel::None => "none".to_string(),
    }
}

#[query]
fn list_collection_admins(collection_id: String) -> Result<Vec<String>, String> {
    let caller = ic_cdk::caller().to_string();
    collections::require_admin_access(&collection_id, &caller)?;
    Ok(collections::get_collection_admins(&collection_id))
}

#[query]
fn get_genesis_admin(collection_id: String) -> Option<String> {
    collections::get_genesis_admin(&collection_id)
}

// =============================================================================
// DOCUMENT MANAGEMENT
// =============================================================================

#[update]
async fn add_document(request: AddDocumentRequest) -> Result<DocumentMetadata, String> {
    // Verify caller has admin access
    let caller = caller().to_string();
    if !storage::is_collection_admin(&request.collection_id, &caller) {
        return Err("Only collection admins can add documents".to_string());
    }

    // Add document to storage
    let document = storage::add_document(request)?;
    
    // Invalidate cache since we're adding new content
    compute::invalidate_collection_cache(&document.collection_id);
    
    Ok(document)
}

#[update]
async fn add_document_and_embed(
    request: AddDocumentRequest,
    proxy_url: String,
) -> Result<DocumentMetadata, String> {
    // Verify caller has admin access
    let caller = caller().to_string();
    if !storage::is_collection_admin(&request.collection_id, &caller) {
        return Err("Only collection admins can add documents".to_string());
    }

    // Get collection settings for embedding
    let collection = storage::get_collection(&request.collection_id)
        .ok_or_else(|| format!("Collection '{}' not found", request.collection_id))?;

    // Step 1: Add document to storage
    let document = storage::add_document(request)?;
    
    // Step 2: Generate embeddings for document chunks
    let chunks = storage::get_document_chunks(&document.id);
    
    match compute::embed_document_chunks(&chunks, &collection.settings, proxy_url).await {
        Ok(vectors) => {
            // Step 3: Store vectors in stable memory
            match storage::store_vectors_batch(vectors) {
                Ok(_) => {
                    // Mark document as embedded
                    storage::mark_document_embedded(&document.collection_id, &document.id)?;
                    
                    // Invalidate cache to include new vectors
                    compute::invalidate_collection_cache(&document.collection_id);
                    
                    Ok(document)
                }
                Err(e) => {
                    // Compensation: Remove document if vector storage fails
                    let _ = storage::delete_document(&document.collection_id, &document.id);
                    Err(format!("Failed to store vectors: {}", e))
                }
            }
        }
        Err(e) => {
            // Compensation: Remove document if embedding fails
            let _ = storage::delete_document(&document.collection_id, &document.id);
            Err(format!("Failed to generate embeddings: {}", e))
        }
    }
}

#[query]
fn get_document(collection_id: String, document_id: String) -> Option<DocumentMetadata> {
    storage::get_document(&collection_id, &document_id)
}

#[query]
fn list_documents(collection_id: String) -> Vec<DocumentMetadata> {
    storage::list_documents(&collection_id)
}

#[query]
fn get_document_content(collection_id: String, document_id: String) -> Option<String> {
    storage::get_document_content(&collection_id, &document_id)
}

#[update]
fn delete_document(collection_id: String, document_id: String) -> Result<(), String> {
    documents::delete_document(&collection_id, &document_id)
}

// =============================================================================
// VECTOR SEARCH & SIMILARITY
// =============================================================================

#[update]
pub async fn search(request: SearchRequest) -> Result<Vec<MemorySearchResult>, String> {
    let collection = storage::get_collection(&request.collection_id)
        .ok_or_else(|| format!("Collection '{}' not found", request.collection_id))?;
    let proxy_url = collection.settings.proxy_url;
    let model = parse_embedding_model(&collection.settings.embedding_model)?;
    let (query_embedding, _) = compute::embed_query_text(&request.query, model, proxy_url).await?;
    
    // Use request.use_approximate or default to true for backward compatibility
    let matches = cosine_similarity_search(
        &query_embedding,
        &request.collection_id,
        &SimilarityConfig {
            min_score: request.min_score,
            max_results: request.limit.unwrap_or(10),
            use_approximate: request.use_approximate.unwrap_or(true),    // Use request parameter
            candidate_factor: 3.0,    // Search 3x more candidates for accuracy
        }
    )?;
    
    Ok(matches.into_iter().map(|m| MemorySearchResult {
        document_id: m.document_id,
        chunk_id: m.chunk_id,
        score: m.score,
        text: m.chunk_text.unwrap_or_default(),
    }).collect())
}


#[update]
pub async fn search_filtered(request: SearchRequest) -> Result<Vec<MemorySearchResult>, String> {
    let collection = storage::get_collection(&request.collection_id)
        .ok_or_else(|| format!("Collection '{}' not found", request.collection_id))?;
    let proxy_url = collection.settings.proxy_url.clone();
    let model = parse_embedding_model(&collection.settings.embedding_model)?;
    
    let (query_embedding, _) = compute::embed_query_text(&request.query, model, proxy_url).await?;
    
    // FIXED: Include new fields in SimilarityConfig
    let matches = similarity_search_filtered(
        &query_embedding,
        &request.collection_id,
        request.filter.as_ref().map(|f| vec![f.clone()]).as_deref(),
        &SimilarityConfig {
            min_score: request.min_score,
            max_results: request.limit.unwrap_or(10),
            use_approximate: true,    // Enable fast search by default
            candidate_factor: 3.0,    // Search 3x more candidates for accuracy
        }
    )?;
    
    Ok(matches.into_iter().map(|m| MemorySearchResult {
        document_id: m.document_id,
        chunk_id: m.chunk_id,
        score: m.score,
        text: m.chunk_text.unwrap_or_default(),
    }).collect())
}



#[query]
fn find_similar_documents(
    source_document_id: String,
    collection_id: String,
    limit: Option<u32>,
    min_score: Option<f64>,
) -> Result<Vec<VectorMatch>, String> {
    // FIXED: Include new fields in SimilarityConfig
    let config = compute::SimilarityConfig {
        min_score,
        max_results: limit.unwrap_or(10),
        use_approximate: true,    // Enable fast search by default
        candidate_factor: 3.0,    // Search 3x more candidates for accuracy
    };
    
    compute::find_similar_documents(&source_document_id, &collection_id, &config)
}



#[update]
async fn batch_similarity_search(
    queries: Vec<String>,
    collection_id: String,
    limit: Option<u32>,
    min_score: Option<f64>,
) -> Result<Vec<Vec<VectorMatch>>, String> {
    let collection = storage::get_collection(&collection_id)
        .ok_or_else(|| format!("Collection '{}' not found", collection_id))?;

    let model = parse_embedding_model(&collection.settings.embedding_model)?;
    let proxy_url = collection.settings.proxy_url;
    
    // Generate embeddings for all queries
    let mut query_embeddings = Vec::new();
    for query in queries {
        let (embedding, _) = compute::embed_query_text(&query, model.clone(), proxy_url.clone()).await?;
        query_embeddings.push(embedding);
    }
    
    // FIXED: Include new fields in SimilarityConfig
    let config = compute::SimilarityConfig {
        min_score,
        max_results: limit.unwrap_or(10),
        use_approximate: true,    // Enable fast search by default
        candidate_factor: 3.0,    // Search 3x more candidates for accuracy
    };
    
    compute::compute_similarity_batch(&query_embeddings, &collection_id, &config)
}

// =============================================================================
// IN-MEMORY DEMO FUNCTIONS
// =============================================================================

#[update]
async fn demo_vector_similarity(
    items: Vec<String>,
    query: String,
    proxy_url: String,
    max_results: Option<u32>,
    min_score: Option<f64>,
) -> Result<Vec<compute::MemorySearchResult>, String> {
    compute::create_and_search_memory_index(
        items,
        query,
        compute::EmbeddingModel::OpenAIAda002,
        proxy_url,
        max_results.unwrap_or(5) as usize,
        min_score,
    ).await
}

// =============================================================================
// EMBEDDING MANAGEMENT
// =============================================================================

#[update]
async fn embed_existing_document(
    collection_id: String,
    document_id: String,
) -> Result<u32, String> {
    let caller = caller().to_string();
    if !storage::is_collection_admin(&collection_id, &caller) {
        return Err("Only collection admins can embed documents".to_string());
    }

    let collection = storage::get_collection(&collection_id)
        .ok_or_else(|| format!("Collection '{}' not found", collection_id))?;

    let document = storage::get_document(&collection_id, &document_id)
        .ok_or_else(|| format!("Document '{}' not found", document_id))?;

    if document.is_embedded {
        return Err("Document is already embedded".to_string());
    }

    let chunks = storage::get_document_chunks(&document_id);
    if chunks.is_empty() {
        return Err("No chunks found for document".to_string());
    }

    let proxy_url = collection.settings.proxy_url.clone();
    let vectors = compute::embed_document_chunks(&chunks, &collection.settings, proxy_url).await?;
    let vector_count = vectors.len() as u32;
    
    storage::store_vectors_batch(vectors)?;
    storage::mark_document_embedded(&collection_id, &document_id)?;
    
    // Invalidate cache to include new vectors
    compute::invalidate_collection_cache(&collection_id);
    
    Ok(vector_count)
}

#[update]
async fn bulk_embed_collection(collection_id: String) -> Result<BulkEmbedResult, String> {
    let caller = caller().to_string();
    if !storage::is_collection_admin(&collection_id, &caller) {
        return Err("Only collection admins can bulk embed collections".to_string());
    }

    let documents = storage::list_documents(&collection_id);
    let mut result = BulkEmbedResult {
        embedded: 0,
        skipped: 0,
        failed: 0,
        errors: Vec::new(),
    };

    for document in documents {
        if document.is_embedded {
            result.skipped += 1;
            continue;
        }

        match embed_existing_document(collection_id.clone(), document.id.clone()).await {
            Ok(_) => result.embedded += 1,
            Err(e) => {
                result.failed += 1;
                result.errors.push(format!("Document {}: {}", document.id, e));
            }
        }
    }

    Ok(result)
}

// =============================================================================
// SYSTEM FUNCTIONS
// =============================================================================

#[query]
fn get_storage_stats() -> (u64, u64, u64, u64) {
    storage::get_storage_stats()
}

#[query]
fn get_memory_stats() -> storage::MemoryStats {
    storage::get_memory_stats()
}

#[update]
fn clear_cache() {
    compute::clear_cache();
}


#[query]
fn get_cache_stats() -> CacheStats {
    compute::get_cache_stats()
}

#[update]
fn cleanup_cache() -> u32 {
    compute::cleanup_cache()
}
#[update]
fn invalidate_collection_cache(collection_id: String) {
    compute::invalidate_collection_cache(&collection_id);
}

#[update]
fn set_config(key: String, value: String) -> Result<(), String> {
    storage::set_config(&key, value)
}

#[query]
fn get_config(key: String) -> Option<String> {
    storage::get_config(&key)
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn parse_embedding_model(model_str: &str) -> Result<compute::EmbeddingModel, String> {
    match model_str {
        "text-embedding-ada-002" => Ok(compute::EmbeddingModel::OpenAIAda002),
        "text-embedding-3-small" => Ok(compute::EmbeddingModel::OpenAISmall),
        "text-embedding-3-large" => Ok(compute::EmbeddingModel::OpenAILarge),
        custom => Ok(compute::EmbeddingModel::Custom(custom.to_string())),
    }
}

// =============================================================================
// HELPER TYPES
// =============================================================================

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct BulkEmbedResult {
    pub embedded: u32,
    pub skipped: u32,
    pub failed: u32,
    pub errors: Vec<String>,
}

// =============================================================================
// CYCLE MANAGEMENT
// =============================================================================

#[update]
fn wallet_receive() -> u64 {
    let available = ic_cdk::api::call::msg_cycles_available128();
    let accepted = ic_cdk::api::call::msg_cycles_accept128(available);
    accepted as u64
}

#[query]
fn get_canister_cycles() -> u64 {
    ic_cdk::api::canister_balance128() as u64
}

// =============================================================================
// VECTOR MANAGEMENT
// =============================================================================

#[query]
fn get_vector(vector_id: String) -> Option<Vector> {
    vectors::get_vector(&vector_id)
}

#[query]
fn get_vector_count(collection_id: String) -> u64 {
    vectors::get_vector_count(&collection_id)
}

#[query]
fn get_collection_embedding_dimensions(collection_id: String) -> Option<u32> {
    vectors::get_collection_embedding_dimensions(&collection_id)
}

#[update]
fn delete_vector(vector_id: String) -> Result<(), String> {
    vectors::delete_vector(&vector_id)
}

#[update]
fn delete_document_vectors(collection_id: String, document_id: String) -> Result<(), String> {
    vectors::delete_document_vectors(&document_id)
}

#[update]
fn clear_collection_vectors(collection_id: String) -> Result<(), String> {
    vectors::cleanup_collection_index(&collection_id);
    Ok(())
}

#[update]
fn validate_collection_vectors(collection_id: String) -> Vec<String> {
    vectors::validate_vectors()
}

// =============================================================================
// DOCUMENT MANAGEMENT
// =============================================================================

#[query]
fn get_document_chunks(document_id: String) -> Vec<SemanticChunk> {
    documents::get_document_chunks_by_document_id(&document_id)
}

#[update]
fn store_document_chunks(document_id: String, chunks: Vec<SemanticChunk>) -> Result<(), String> {
    documents::store_document_chunks(&document_id, chunks);
    Ok(())
}

#[query]
fn get_document_count(collection_id: String) -> u64 {
    documents::get_document_count(&collection_id)
}

#[query]
fn get_collection_documents(collection_id: String) -> Vec<DocumentMetadata> {
    documents::get_collection_documents(&collection_id)
}

#[update]
fn delete_collection_documents(collection_id: String) -> Result<(), String> {
    documents::delete_collection_documents(&collection_id)
}





#[cfg(test)]
mod export {
    use super::*;
    use candid::export_service;

    #[test]
    fn export_candid_interface() {
        export_service!();
        std::fs::write("blueband_rust.did", __export_service()).expect("Write failed");
    }
}

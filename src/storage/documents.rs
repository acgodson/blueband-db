// storage/documents.rs
use ic_stable_structures::{StableBTreeMap, Storable};
use std::cell::RefCell;


use super::memory::{
    get_memory, CHUNKS_MEMORY_ID, DOCUMENTS_MEMORY_ID, DOCUMENT_INDEX_MEMORY_ID,
    MemoryType,
};
use crate::types::*;

// =============================================================================
// GLOBAL STORAGE
// =============================================================================

thread_local! {
    // Documents: collection_id::document_id -> DocumentMetadata
    static DOCUMENTS: RefCell<StableBTreeMap<String, DocumentMetadata, MemoryType>> = RefCell::new(
        StableBTreeMap::init(get_memory(DOCUMENTS_MEMORY_ID))
    );

    // Document Chunks: document_id -> Vec<SemanticChunk> (O(1) lookup, small Vec scan)
    static DOCUMENT_CHUNKS: RefCell<StableBTreeMap<String, ChunkList, MemoryType>> = RefCell::new(
        StableBTreeMap::init(get_memory(CHUNKS_MEMORY_ID))
    );

    // Document Index: collection_id -> Vec<document_id> (for O(1) collection lookups)
    static DOCUMENT_INDEX: RefCell<StableBTreeMap<String, StringList, MemoryType>> = RefCell::new(
        StableBTreeMap::init(get_memory(DOCUMENT_INDEX_MEMORY_ID))
    );
}

// =============================================================================
// DOCUMENT OPERATIONS
// =============================================================================

pub fn add_document(request: AddDocumentRequest) -> Result<DocumentMetadata, String> {
    validate_document_content(&request.content)?;


    let collection = super::collections::get_collection(&request.collection_id)
        .ok_or_else(|| format!("Collection '{}' not found", request.collection_id))?;

    if let Some(max_docs) = collection.settings.max_documents {
        let current_count = count_collection_documents(&request.collection_id);
        if current_count >= max_docs as u64 {
            return Err(format!(
                "Collection has reached maximum of {} documents",
                max_docs
            ));
        }
    }

    let document_id = generate_id("doc", &request.title);
    let storage_key = format!("{}::{}", request.collection_id, document_id);

    let checksum = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(request.content.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    let chunks = create_semantic_chunks(&request.content, &document_id, &collection.settings);

    let document = DocumentMetadata {
        id: document_id.clone(),
        collection_id: request.collection_id.clone(),
        title: request.title,
        content_type: request.content_type.unwrap_or_default(),
        source_url: request.source_url,
        timestamp: current_time(),
        total_chunks: chunks.len() as u32,
        size: request.content.len() as u64,
        is_embedded: false,
        checksum,
        author: request.author,
        tags: request.tags,
    };

    // Store document metadata
    DOCUMENTS.with(|d| d.borrow_mut().insert(storage_key, document.clone()));

    // Store all chunks for this document
    DOCUMENT_CHUNKS.with(|c| c.borrow_mut().insert(document_id.clone(), ChunkList(chunks)));

    // Update document index for collection lookups
    add_to_document_index(&request.collection_id, &document_id);

    Ok(document)
}

pub fn get_document(collection_id: &str, document_id: &str) -> Option<DocumentMetadata> {
    DOCUMENT_INDEX.with(|idx| {
        if let Some(doc_ids) = idx.borrow().get(&collection_id.to_string()) {
            if doc_ids.0.contains(&document_id.to_string()) {
                let storage_key = format!("{}::{}", collection_id, document_id);
                DOCUMENTS.with(|d| d.borrow().get(&storage_key))
            } else {
                None
            }
        } else {
            None
        }
    })
}

pub fn list_documents(collection_id: &str) -> Vec<DocumentMetadata> {
    // O(1) index lookup + O(k) document retrieval where k = docs in collection
    DOCUMENT_INDEX.with(|idx| {
        if let Some(doc_ids) = idx.borrow().get(&collection_id.to_string()) {
            doc_ids.0
                .iter()
                .filter_map(|doc_id| get_document(collection_id, doc_id))
                .collect()
        } else {
            Vec::new()
        }
    })
}

pub fn delete_document(collection_id: &str, document_id: &str) -> Result<(), String> {
    // Delete document chunks
    DOCUMENT_CHUNKS.with(|c| c.borrow_mut().remove(&document_id.to_string()));
    
    // Remove from collection index
    DOCUMENT_INDEX.with(|idx| {
        let mut index = idx.borrow_mut();
        if let Some(mut doc_ids) = index.get(&collection_id.to_string()) {
            doc_ids.0.retain(|id| id != document_id);
            index.insert(collection_id.to_string(), doc_ids);
            Ok(())
        } else {
            Err("Collection not found".to_string())
        }
    })
}

pub fn mark_document_embedded(collection_id: &str, document_id: &str) -> Result<(), String> {
    let storage_key = format!("{}::{}", collection_id, document_id);
    DOCUMENTS.with(|d| {
        let mut docs = d.borrow_mut();
        if let Some(mut doc) = docs.get(&storage_key) {
            doc.is_embedded = true;
            docs.insert(storage_key, doc);
            Ok(())
        } else {
            Err(format!(
                "Document '{}' not found in collection '{}'",
                document_id, collection_id
            ))
        }
    })
}

// =============================================================================
// DOCUMENT INDEX OPERATIONS (for O(1) performance)
// =============================================================================

pub fn init_collection_document_index(collection_id: &str) {
    DOCUMENT_INDEX.with(|idx| {
        idx.borrow_mut()
            .insert(collection_id.to_string(), StringList::new())
    });
}

pub fn cleanup_collection_document_index(collection_id: &str) {
    DOCUMENT_INDEX.with(|idx| idx.borrow_mut().remove(&collection_id.to_string()));
}

fn add_to_document_index(collection_id: &str, document_id: &str) {
    DOCUMENT_INDEX.with(|idx| {
        let mut index = idx.borrow_mut();
        let mut doc_ids = index.get(&collection_id.to_string()).unwrap_or_default();
        doc_ids.0.push(document_id.to_string());
        index.insert(collection_id.to_string(), doc_ids);
    });
}

fn remove_from_document_index(collection_id: &str, document_id: &str) {
    DOCUMENT_INDEX.with(|idx| {
        let mut index = idx.borrow_mut();
        if let Some(mut doc_ids) = index.get(&collection_id.to_string()) {
            doc_ids.0.retain(|id| id != document_id);
            index.insert(collection_id.to_string(), doc_ids);
        }
    });
}

// =============================================================================
// CHUNK OPERATIONS (Vector Database Optimized)
// =============================================================================

pub fn get_document_chunks(document_id: &str) -> Vec<SemanticChunk> {
    DOCUMENT_CHUNKS.with(|c| {
        c.borrow()
            .get(&document_id.to_string())
            .map(|chunks| chunks.0)
            .unwrap_or_default()
    })
}

pub fn get_chunk(document_id: &str, chunk_id: &str) -> Option<SemanticChunk> {
    // O(1) document lookup + O(k) chunk scan where k = ~20-50 chunks
    DOCUMENT_CHUNKS.with(|c| {
        if let Some(chunks) = c.borrow().get(&document_id.to_string()) {
            chunks.0.iter().find(|chunk| chunk.id == chunk_id).cloned()
        } else {
            None
        }
    })
}

pub fn get_chunk_text(document_id: &str, chunk_id: &str) -> Option<String> {
    // Optimized for vector search results - get specific chunk text
    get_chunk(document_id, chunk_id).map(|chunk| chunk.text)
}

pub fn get_document_content(collection_id: &str, document_id: &str) -> Option<String> {
    DOCUMENT_CHUNKS.with(|c| {
        c.borrow()
            .get(&document_id.to_string())
            .and_then(|chunks| chunks.0.first().map(|chunk| chunk.text.clone()))
    })
}

// =============================================================================
// UTILITY FUNCTIONS (Supporting computed stats)
// =============================================================================

fn create_semantic_chunks(
    content: &str,
    document_id: &str,
    settings: &CollectionSettings,
) -> Vec<SemanticChunk> {
    let chunk_size = settings.chunk_size as usize;
    let overlap = settings.chunk_overlap as usize;
    let mut chunks = Vec::new();
    let mut position = 0u32;
    let mut start = 0;

    while start < content.len() {
        let end = (start + chunk_size).min(content.len());
        let chunk_text = content[start..end].to_string();

        if !chunk_text.trim().is_empty() {
            let chunk_id = format!("chunk_{}", position);
            chunks.push(SemanticChunk {
                id: chunk_id,
                document_id: document_id.to_string(),
                text: chunk_text.clone(),
                position,
                char_start: start as u64,
                char_end: end as u64,
                token_count: Some(estimate_tokens(&chunk_text)),
            });
            position += 1;
        }

        if end >= content.len() {
            break;
        }

        start = end - overlap.min(end);
    }

    // Return chunks already sorted by position
    chunks
}

// Functions to support computed collection stats
pub fn count_documents() -> u64 {
    DOCUMENTS.with(|d| d.borrow().len())
}

pub fn count_chunks() -> u64 {
    DOCUMENT_CHUNKS.with(|c| {
        c.borrow()
            .iter()
            .map(|(_, chunks)| chunks.0.len() as u64)
            .sum()
    })
}

pub fn count_collection_documents(collection_id: &str) -> u64 {
    // O(1) index lookup instead of O(n) scan
    DOCUMENT_INDEX.with(|idx| {
        idx.borrow()
            .get(&collection_id.to_string())
            .map(|doc_ids| doc_ids.0.len() as u64)
            .unwrap_or(0)
    })
}

pub fn calculate_collection_size(collection_id: &str) -> u64 {
    // O(k) where k = docs in collection, not O(n) where n = all docs
    DOCUMENT_INDEX.with(|idx| {
        if let Some(doc_ids) = idx.borrow().get(&collection_id.to_string()) {
            doc_ids.0
                .iter()
                .filter_map(|doc_id| get_document(collection_id, doc_id))
                .map(|doc| doc.size)
                .sum()
        } else {
            0
        }
    })
}

pub fn get_last_document_update(collection_id: &str) -> Option<u64> {
    // O(k) where k = docs in collection, not O(n) where n = all docs
    DOCUMENT_INDEX.with(|idx| {
        if let Some(doc_ids) = idx.borrow().get(&collection_id.to_string()) {
            doc_ids.0
                .iter()
                .filter_map(|doc_id| get_document(collection_id, doc_id))
                .map(|doc| doc.timestamp)
                .max()
        } else {
            None
        }
    })
}

pub fn document_exists(collection_id: &str, document_id: &str) -> bool {
    let storage_key = format!("{}::{}", collection_id, document_id);
    DOCUMENTS.with(|d| d.borrow().contains_key(&storage_key))
}

pub fn get_document_title(collection_id: &str, document_id: &str) -> Option<String> {
    get_document(collection_id, document_id).map(|doc| doc.title)
}

pub fn find_document_by_title(collection_id: &str, title: &str) -> Option<DocumentMetadata> {
    list_documents(collection_id)
        .into_iter()
        .find(|doc| doc.title == title)
}

// Simple token estimation
fn estimate_tokens(text: &str) -> u32 {
    // Rough estimation: ~4 characters per token for English
    (text.len() as f32 / 4.0).ceil() as u32
}

pub fn get_document_count(collection_id: &str) -> u64 {
    DOCUMENT_INDEX.with(|idx| {
        idx.borrow()
            .get(&collection_id.to_string())
            .map(|doc_ids| doc_ids.0.len() as u64)
            .unwrap_or(0)
    })
}

pub fn get_collection_documents(collection_id: &str) -> Vec<DocumentMetadata> {
    DOCUMENT_INDEX.with(|idx| {
        if let Some(doc_ids) = idx.borrow().get(&collection_id.to_string()) {
            DOCUMENTS.with(|d| {
                let documents = d.borrow();
                doc_ids.0
                    .iter()
                    .filter_map(|id| documents.get(&id.to_string()))
                    .collect()
            })
        } else {
            Vec::new()
        }
    })
}

pub fn delete_collection_documents(collection_id: &str) -> Result<(), String> {
    DOCUMENT_INDEX.with(|idx| {
        if let Some(doc_ids) = idx.borrow().get(&collection_id.to_string()) {
            DOCUMENTS.with(|d| {
                let mut documents = d.borrow_mut();
                for doc_id in doc_ids.0.iter() {
                    documents.remove(&doc_id.to_string());
                }
            });
            DOCUMENT_CHUNKS.with(|c| {
                let mut chunks = c.borrow_mut();
                for doc_id in doc_ids.0.iter() {
                    chunks.remove(&doc_id.to_string());
                }
            });
        }
    });
    DOCUMENT_INDEX.with(|idx| idx.borrow_mut().remove(&collection_id.to_string()));
    Ok(())
}

pub fn add_document_to_collection(collection_id: &str, document_id: &str) {
    DOCUMENT_INDEX.with(|idx| {
        let mut index = idx.borrow_mut();
        let mut doc_ids = index.get(&collection_id.to_string()).unwrap_or_default();
        doc_ids.0.push(document_id.to_string());
        index.insert(collection_id.to_string(), doc_ids);
    });
}

pub fn remove_document_from_collection(collection_id: &str, document_id: &str) {
    DOCUMENT_INDEX.with(|idx| {
        let mut index = idx.borrow_mut();
        if let Some(mut doc_ids) = index.get(&collection_id.to_string()) {
            doc_ids.0.retain(|id| id != document_id);
            index.insert(collection_id.to_string(), doc_ids);
        }
    });
}

pub fn get_document_chunks_by_document_id(document_id: &str) -> Vec<SemanticChunk> {
    DOCUMENT_CHUNKS.with(|c| {
        c.borrow()
            .get(&document_id.to_string())
            .map(|chunks| chunks.0)
            .unwrap_or_default()
    })
}

pub fn get_total_chunks() -> u64 {
    DOCUMENT_CHUNKS.with(|c| {
        c.borrow()
            .iter()
            .map(|(_, chunks)| chunks.0.len() as u64)
            .sum()
    })
}

pub fn store_document_chunks(document_id: &str, chunks: Vec<SemanticChunk>) {
    DOCUMENT_CHUNKS.with(|c| c.borrow_mut().insert(document_id.to_string(), ChunkList(chunks)));
}

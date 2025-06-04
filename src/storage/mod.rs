pub mod collections;
pub mod documents;
pub mod memory;
pub mod vectors;

// Re-export commonly used functions for easy access from lib.rs
pub use collections::{
    create_collection, get_collection, is_collection_admin, list_collections,
    get_collection_with_stats, list_collections_with_stats,
};

pub use documents::{
    add_document, get_chunk_text, get_document, get_document_chunks, get_document_content,
    get_document_title, list_documents, mark_document_embedded, delete_document,
};

pub use vectors::{
    delete_vector, get_collection_vectors, get_document_vectors, get_vector,
    get_vectors_for_similarity, store_vector, store_vectors_batch,
};

pub use memory::{get_memory_stats, MemoryStats};

// Configuration storage
use ic_stable_structures::StableBTreeMap;
use memory::{get_memory, CONFIG_MEMORY_ID};
use ic_stable_structures::Memory;
use std::cell::RefCell;
use crate::storage::memory::MemoryType;

thread_local! {
    static CONFIG: RefCell<StableBTreeMap<String, String, MemoryType>> = RefCell::new(
        StableBTreeMap::init(get_memory(CONFIG_MEMORY_ID))
    );
}

/// Set configuration value
pub fn set_config(key: &str, value: String) -> Result<(), String> {
    CONFIG.with(|c| {
        c.borrow_mut().insert(key.to_string(), value);
        Ok(())
    })
}

/// Get configuration value
pub fn get_config(key: &str) -> Option<String> {
    CONFIG.with(|c| c.borrow().get(&key.to_string()))
}

/// Get all storage statistics
pub fn get_storage_stats() -> (u64, u64, u64, u64) {
    let collections_count = collections::count_collections();
    let documents_count = documents::count_documents();
    let chunks_count = documents::count_chunks();
    let vectors_count = vectors::count_vectors();

    (
        collections_count,
        documents_count,
        chunks_count,
        vectors_count,
    )
}

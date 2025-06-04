// storage/vectors.rs
use ic_stable_structures::{StableBTreeMap, Storable};
use std::cell::RefCell;

use super::memory::{get_memory, MemoryType, VECTORS_MEMORY_ID, VECTOR_INDEX_MEMORY_ID};
use crate::types::current_time;
use crate::types::*;

// =============================================================================
// GLOBAL STORAGE
// =============================================================================

thread_local! {
    // Vectors: vector_id -> Vector
    static VECTORS: RefCell<StableBTreeMap<String, Vector, MemoryType>> = RefCell::new(
        StableBTreeMap::init(get_memory(VECTORS_MEMORY_ID))
    );

    // Vector Index: collection_id -> Vec<VectorId> (for O(1) collection lookups)
    static VECTOR_INDEX: RefCell<StableBTreeMap<String, StringList, MemoryType>> = RefCell::new(
        StableBTreeMap::init(get_memory(VECTOR_INDEX_MEMORY_ID))
    );
}

// =============================================================================
// VECTOR STORAGE OPERATIONS
// =============================================================================

/// Stores a vector with validation - pure storage operation
pub fn store_vector(vector: Vector) -> Result<(), String> {
    // Basic validation first
    if vector.embedding.is_empty() {
        return Err("Vector embedding cannot be empty".to_string());
    }

    if vector.norm <= 0.0 || !vector.norm.is_finite() {
        return Err("Vector norm must be positive and finite".to_string());
    }

    let collection_id = extract_collection_id_from_document_id(&vector.document_id)?;

    // Verify dependencies exist BEFORE any writes
    if !super::collections::collection_exists(&collection_id) {
        return Err(format!("Collection '{}' not found", collection_id));
    }
    if !super::documents::document_exists(&collection_id, &vector.document_id) {
        return Err(format!("Document '{}' not found", vector.document_id));
    }

    // SINGLE ATOMIC SCOPE: Both operations happen in one borrow
    VECTORS.with(|v| {
        VECTOR_INDEX.with(|vi| {
            let mut vectors = v.borrow_mut();
            let mut index = vi.borrow_mut();

            // Check if vector already exists
            let vector_exists = vectors.contains_key(&vector.id);

            // Prepare index update
            let mut vector_ids = index.get(&collection_id.to_string()).unwrap_or_default();

            // ATOMIC: Both writes happen in sequence with no opportunity for failure between them
            vectors.insert(vector.id.clone(), vector.clone());

            if !vector_exists {
                vector_ids.0.push(vector.id.clone());
                index.insert(collection_id.to_string(), vector_ids);
            }

            Ok(())
        })
    })
}

/// Retrieves a vector by ID
pub fn get_vector(vector_id: &str) -> Option<Vector> {
    VECTORS.with(|v| v.borrow().get(&vector_id.to_string()))
}

/// Deletes a vector by ID
pub fn delete_vector(vector_id: &str) -> Result<(), String> {
    // Get vector info first (outside the atomic section)
    let vector = VECTORS
        .with(|v| v.borrow().get(&vector_id.to_string()))
        .ok_or_else(|| format!("Vector '{}' not found", vector_id))?;

    let collection_id = extract_collection_id_from_document_id(&vector.document_id)?;

    // SINGLE ATOMIC SCOPE: Both deletions happen together
    VECTORS.with(|v| {
        VECTOR_INDEX.with(|vi| {
            let mut vectors = v.borrow_mut();
            let mut index = vi.borrow_mut();

            // ATOMIC: Both removes happen in sequence
            if let Some(_removed_vector) = vectors.remove(&vector_id.to_string()) {
                // Only update index if vector was actually removed
                if let Some(mut vector_ids) = index.get(&collection_id.to_string()) {
                    vector_ids.0.retain(|id| id != vector_id);
                    index.insert(collection_id.to_string(), vector_ids);
                }
                Ok(())
            } else {
                Err(format!("Failed to remove vector '{}'", vector_id))
            }
        })
    })
}

/// Gets all vectors for a collection (used by compute layer)
pub fn get_collection_vectors(collection_id: &str) -> Vec<Vector> {
    VECTOR_INDEX.with(|vi| {
        if let Some(vector_ids) = vi.borrow().get(&collection_id.to_string()) {
            VECTORS.with(|v| {
                let vectors = v.borrow();
                vector_ids
                    .0
                    .iter()
                    .filter_map(|id| vectors.get(&id.to_string()))
                    .collect()
            })
        } else {
            Vec::new()
        }
    })
}

/// Gets vectors for a collection with limit (optimization for compute layer)
pub fn get_vectors_for_similarity(collection_id: &str, limit: Option<u32>) -> Vec<Vector> {
    let mut vectors = get_collection_vectors(collection_id);

    if let Some(limit) = limit {
        vectors.truncate(limit as usize);
    }

    vectors
}

/// Deletes all vectors associated with a document
pub fn delete_document_vectors(document_id: &str) -> Result<(), String> {
    // Get all vector IDs for this document first
    let vector_ids_to_delete: Vec<String> = VECTORS.with(|v| {
        v.borrow()
            .iter()
            .filter_map(|(vector_id, vector)| {
                if vector.document_id == document_id {
                    Some(vector_id.clone())
                } else {
                    None
                }
            })
            .collect()
    });

    if vector_ids_to_delete.is_empty() {
        return Ok(()); // Nothing to delete
    }

    // Get collection ID from first vector
    let collection_id = if let Some(first_vector_id) = vector_ids_to_delete.first() {
        let vector = VECTORS.with(|v| v.borrow().get(first_vector_id));
        if let Some(vector) = vector {
            extract_collection_id_from_document_id(&vector.document_id)?
        } else {
            return Err("Vector not found".to_string());
        }
    } else {
        return Ok(());
    };

    // ATOMIC: Delete all vectors and update index in single operation
    VECTORS.with(|v| {
        VECTOR_INDEX.with(|vi| {
            let mut vectors = v.borrow_mut();
            let mut index = vi.borrow_mut();
            let mut vector_ids = index.get(&collection_id).unwrap_or_default();

            // Remove all vectors and their index entries atomically
            for vector_id in &vector_ids_to_delete {
                vectors.remove(vector_id);
                vector_ids.0.retain(|id| id != vector_id);
            }

            // Update index once
            index.insert(collection_id, vector_ids);

            Ok(())
        })
    })
}

/// Gets all vectors for a specific document
pub fn get_document_vectors(document_id: &str) -> Vec<Vector> {
    VECTORS.with(|v| {
        v.borrow()
            .iter()
            .filter_map(|(_, vector)| {
                if vector.document_id == document_id {
                    Some(vector)
                } else {
                    None
                }
            })
            .collect()
    })
}

/// Gets a specific chunk vector
pub fn get_chunk_vector(document_id: &str, chunk_id: &str) -> Option<Vector> {
    VECTORS.with(|v| {
        v.borrow().iter().find_map(|(_, vector)| {
            if vector.document_id == document_id && vector.chunk_id == chunk_id {
                Some(vector)
            } else {
                None
            }
        })
    })
}

// =============================================================================
// VECTOR INDEX OPERATIONS (Performance Optimization)
// =============================================================================

/// Initializes empty vector index for a collection
pub fn init_collection_index(collection_id: &str) {
    VECTOR_INDEX.with(|vi| {
        vi.borrow_mut()
            .insert(collection_id.to_string(), StringList::new())
    });
}

/// Cleans up vector index when collection is deleted
pub fn cleanup_collection_index(collection_id: &str) {
    VECTOR_INDEX.with(|vi| vi.borrow_mut().remove(&collection_id.to_string()));
}

/// Adds vector to collection index
pub fn add_vector_to_collection(collection_id: &str, vector_id: &str) {
    VECTOR_INDEX.with(|idx| {
        let mut index = idx.borrow_mut();
        let mut vector_ids = index.get(&collection_id.to_string()).unwrap_or_default();
        vector_ids.0.push(vector_id.to_string());
        index.insert(collection_id.to_string(), vector_ids);
    });
}

/// Removes vector from collection index
pub fn remove_vector_from_collection(collection_id: &str, vector_id: &str) {
    VECTOR_INDEX.with(|idx| {
        let mut index = idx.borrow_mut();
        if let Some(mut vector_ids) = index.get(&collection_id.to_string()) {
            vector_ids.0.retain(|id| id != vector_id);
            index.insert(collection_id.to_string(), vector_ids);
        }
    });
}

/// Clears all vectors and vector index
pub fn clear_vectors() {
    VECTORS.with(|v| v.borrow_mut().clear_new());
    VECTOR_INDEX.with(|idx| idx.borrow_mut().clear_new());
}

/// Validates vector index integrity (returns issues found)
pub fn validate_vectors() -> Vec<String> {
    let mut issues = Vec::new();

    VECTOR_INDEX.with(|idx| {
        for (collection_id, vector_ids) in idx.borrow().iter() {
            VECTORS.with(|v| {
                let vectors = v.borrow();
                for vector_id in vector_ids.0.iter() {
                    if !vectors.contains_key(&vector_id.to_string()) {
                        issues.push(format!(
                            "Vector {} in collection {} not found in vectors map",
                            vector_id, collection_id
                        ));
                    }
                }
            });
        }
    });

    issues
}

// =============================================================================
// BATCH STORAGE OPERATIONS
// =============================================================================

/// Stores multiple vectors in batch
pub fn store_vectors_batch(vectors: Vec<Vector>) -> Result<u32, String> {
    if vectors.is_empty() {
        return Ok(0);
    }

    // Validate ALL vectors before storing ANY (fail-fast)
    for (i, vector) in vectors.iter().enumerate() {
        if vector.embedding.is_empty() {
            return Err(format!("Vector at index {} has empty embedding", i));
        }
        if vector.norm <= 0.0 || !vector.norm.is_finite() {
            return Err(format!(
                "Vector at index {} has invalid norm: {}",
                i, vector.norm
            ));
        }
    }

    // Group vectors by collection for efficient batch processing
    let mut vectors_by_collection: std::collections::HashMap<String, Vec<Vector>> =
        std::collections::HashMap::new();

    for vector in vectors {
        let collection_id = extract_collection_id_from_document_id(&vector.document_id)?;
        vectors_by_collection
            .entry(collection_id)
            .or_default()
            .push(vector);
    }

    let mut total_stored = 0u32;

    // Process each collection's vectors in a single atomic operation
    for (collection_id, collection_vectors) in vectors_by_collection {
        // Verify collection exists
        if !super::collections::collection_exists(&collection_id) {
            return Err(format!("Collection '{}' not found", collection_id));
        }

        // Atomic batch insert for this collection
        let stored_count: Result<u32, String> = VECTORS.with(|v| {
            VECTOR_INDEX.with(|vi| {
                let mut vectors_map = v.borrow_mut();
                let mut index = vi.borrow_mut();
                let mut vector_ids = index.get(&collection_id).unwrap_or_default();

                let mut count = 0u32;

                // All vectors for this collection in one atomic operation
                for vector in collection_vectors {
                    let vector_exists = vectors_map.contains_key(&vector.id);

                    // Store vector
                    vectors_map.insert(vector.id.clone(), vector.clone());

                    // Update index if new vector
                    if !vector_exists {
                        vector_ids.0.push(vector.id);
                    }

                    count += 1;
                }

                // Update index once for all vectors in this collection
                index.insert(collection_id, vector_ids);

                Ok(count)
            })
        });

        total_stored += stored_count?;
    }

    Ok(total_stored)
}

/// Deletes multiple vectors in batch
pub fn delete_vectors_batch(vector_ids: Vec<String>) -> Result<u32, String> {
    let mut deleted_count = 0u32;
    let mut errors = Vec::new();

    for vector_id in vector_ids {
        match delete_vector(&vector_id) {
            Ok(()) => deleted_count += 1,
            Err(e) => errors.push(e),
        }
    }

    if !errors.is_empty() {
        return Err(format!(
            "Failed to delete {} vectors: {}",
            errors.len(),
            errors.join("; ")
        ));
    }

    Ok(deleted_count)
}

// =============================================================================
// STORAGE FILTERING (For Compute Layer)
// =============================================================================

/// Finds vectors by embedding model (storage query)
pub fn find_vectors_by_model(collection_id: &str, model: &str) -> Vec<Vector> {
    get_collection_vectors(collection_id)
        .into_iter()
        .filter(|vector| vector.model == model)
        .collect()
}

/// Finds vectors within a date range (storage query)
pub fn find_vectors_by_date_range(
    collection_id: &str,
    start_time: u64,
    end_time: u64,
) -> Vec<Vector> {
    get_collection_vectors(collection_id)
        .into_iter()
        .filter(|vector| vector.created_at >= start_time && vector.created_at <= end_time)
        .collect()
}

// =============================================================================
// STORAGE METRICS (Supporting Collection Stats)
// =============================================================================

/// Counts total vectors across all collections
pub fn count_vectors() -> u64 {
    VECTORS.with(|v| v.borrow().len())
}

/// Counts vectors in a specific collection (O(1) lookup)
pub fn get_vector_count(collection_id: &str) -> u64 {
    VECTOR_INDEX.with(|vi| {
        vi.borrow()
            .get(&collection_id.to_string())
            .map(|vector_ids| vector_ids.0.len() as u64)
            .unwrap_or(0)
    })
}

/// Gets the timestamp of the most recent vector in a collection
pub fn get_last_vector_update(collection_id: &str) -> Option<u64> {
    get_collection_vectors(collection_id)
        .iter()
        .map(|vector| vector.created_at)
        .max()
}

/// Checks if a vector exists by ID
pub fn vector_exists(vector_id: &str) -> bool {
    VECTORS.with(|v| v.borrow().contains_key(&vector_id.to_string()))
}

/// Gets embedding dimensions for a collection (assumes consistent dimensions)
pub fn get_collection_embedding_dimensions(collection_id: &str) -> Option<u32> {
    get_collection_vectors(collection_id)
        .first()
        .map(|vector| vector.embedding.len() as u32)
}

// =============================================================================
// STORAGE UTILITY FUNCTIONS
// =============================================================================

/// Extracts collection_id from document_id by checking all collections
fn extract_collection_id_from_document_id(document_id: &str) -> Result<String, String> {
    // Check all collections to find where this document exists
    for collection in super::collections::list_collections() {
        if super::documents::document_exists(&collection.id, document_id) {
            return Ok(collection.id);
        }
    }

    Err(format!(
        "Could not determine collection for document '{}'",
        document_id
    ))
}

// Rebuilds vector index from scratch
// pub fn rebuild_vector_index() -> Result<(), String> {
//     // Clear existing index
//     VECTOR_INDEX.with(|vi| vi.borrow_mut().clear_new());

//     // Initialize empty indexes for all collections
//     for collection in super::collections::list_collections() {
//         init_collection_index(&collection.id);
//     }

//     // Rebuild from vectors
//     VECTORS.with(|v| {
//         for (vector_id, vector) in v.borrow().iter() {
//             let vector_id = vector_id.to_string();
//             if let Ok(collection_id) = extract_collection_id_from_document_id(&vector.document_id) {
//                 add_vector_to_collection(&collection_id, &vector_id);
//             }
//         }
//     });

//     Ok(())
// }

// storage/vectors.rs
use super::memory::{get_memory, MemoryType, VECTORS_MEMORY_ID, VECTOR_INDEX_MEMORY_ID};
use ic_stable_structures::StableBTreeMap;
use std::cell::RefCell;

use crate::types::*;

// =============================================================================
// GLOBAL STORAGE
// =============================================================================

thread_local! {
    // Vectors: vector_id -> Vector
    static VECTORS: RefCell<StableBTreeMap<String, Vector, MemoryType>> = RefCell::new(
        StableBTreeMap::init(get_memory(VECTORS_MEMORY_ID))
    );

    // Vector Index: collection_id -> Vec<VectorId>
    static VECTOR_INDEX: RefCell<StableBTreeMap<String, StringList, MemoryType>> = RefCell::new(
        StableBTreeMap::init(get_memory(VECTOR_INDEX_MEMORY_ID))
    );
}

// =============================================================================
// VECTOR STORAGE OPERATIONS
// =============================================================================

pub fn get_vector(vector_id: &str) -> Option<Vector> {
    VECTORS.with(|v| v.borrow().get(&vector_id.to_string()))
}

pub fn delete_vector(vector_id: &str) -> Result<(), String> {
    let vector = VECTORS
        .with(|v| v.borrow().get(&vector_id.to_string()))
        .ok_or_else(|| format!("Vector '{}' not found", vector_id))?;

    let collection_id = extract_collection_id_from_document_id(&vector.document_id)?;

    VECTORS.with(|v| {
        VECTOR_INDEX.with(|vi| {
            let mut vectors = v.borrow_mut();
            let mut index = vi.borrow_mut();

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

pub fn delete_document_vectors(document_id: &str) -> Result<(), String> {
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

    VECTORS.with(|v| {
        VECTOR_INDEX.with(|vi| {
            let mut vectors = v.borrow_mut();
            let mut index = vi.borrow_mut();
            let mut vector_ids = index.get(&collection_id).unwrap_or_default();

            for vector_id in &vector_ids_to_delete {
                vectors.remove(vector_id);
                vector_ids.0.retain(|id| id != vector_id);
            }

            index.insert(collection_id, vector_ids);

            Ok(())
        })
    })
}

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

/// Clears all vectors and vector index
pub fn clear_vectors() {
    VECTORS.with(|v| v.borrow_mut().clear_new());
    VECTOR_INDEX.with(|idx| idx.borrow_mut().clear_new());
}

/// Validates and optionally repairs vector index integrity for a specific collection
/// Returns a report of issues found and any repairs made
pub fn validate_vectors(collection_id: &str, should_repair: bool) -> Vec<String> {
    let mut report = Vec::new();
    
    // Verify collection exists
    if !super::collections::collection_exists(collection_id) {
        report.push(format!("Collection '{}' not found", collection_id));
        return report;
    }

    // Get all vectors for this collection
    let vectors = get_collection_vectors(collection_id);
    let mut valid_vector_ids = Vec::new();
    let mut invalid_vectors = Vec::new();

    // Validate each vector
    for vector in vectors {
        let mut is_valid = true;
        let mut issues = Vec::new();

        // Check if document exists
        if !super::documents::document_exists(collection_id, &vector.document_id) {
            issues.push(format!("Document '{}' not found", vector.document_id));
            is_valid = false;
        }

        // Validate vector data
        if vector.embedding.is_empty() {
            issues.push("Empty embedding".to_string());
            is_valid = false;
        }
        if vector.norm <= 0.0 || !vector.norm.is_finite() {
            issues.push(format!("Invalid norm: {}", vector.norm));
            is_valid = false;
        }

        if is_valid {
            valid_vector_ids.push(vector.id.clone());
        } else {
            invalid_vectors.push((vector.id.clone(), issues));
        }
    }

    // Report issues found
    if !invalid_vectors.is_empty() {
        report.push(format!("Found {} invalid vectors:", invalid_vectors.len()));
        for (vector_id, issues) in invalid_vectors {
            report.push(format!("  Vector {}: {}", vector_id, issues.join(", ")));
        }

        // Only repair if requested and there are issues
        if should_repair {
            let valid_count = valid_vector_ids.len();
            // Rebuild index with only valid vectors
            VECTOR_INDEX.with(|vi| {
                let mut index = vi.borrow_mut();
                index.insert(collection_id.to_string(), StringList(valid_vector_ids));
            });
            report.push(format!("Index repaired: {} valid vectors retained", valid_count));
        }
    } else {
        report.push("No issues found".to_string());
    }

    report
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

    // Group vectors by collection
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

        let stored_count: Result<u32, String> = VECTORS.with(|v| {
            VECTOR_INDEX.with(|vi| {
                let mut vectors_map = v.borrow_mut();
                let mut index = vi.borrow_mut();
                let mut vector_ids = index.get(&collection_id).unwrap_or_default();

                let mut count = 0u32;

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


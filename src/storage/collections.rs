// storage/collections.rs
use crate::storage::memory::MemoryType;
use ic_stable_structures::StableBTreeMap;
use std::cell::RefCell;

use super::memory::{get_memory, COLLECTIONS_MEMORY_ID};
use crate::types::*;

// =============================================================================
// GLOBAL STORAGE
// =============================================================================

thread_local! {
    // Collections: collection_id -> Collection
    static COLLECTIONS: RefCell<StableBTreeMap<String, Collection, MemoryType>> = RefCell::new(
        StableBTreeMap::init(get_memory(COLLECTIONS_MEMORY_ID))
    );
}

// =============================================================================
// COLLECTION OPERATIONS
// =============================================================================

pub fn create_collection(
    request: CreateCollectionRequest,
    creator: String,
) -> Result<Collection, String> {
    validate_collection_id(&request.id)?;

    if get_collection(&request.id).is_some() {
        return Err(format!("Collection '{}' already exists", request.id));
    }

    let collection = Collection {
        id: request.id.clone(),
        name: request.name,
        description: request.description,
        created_at: current_time(),
        updated_at: current_time(),
        genesis_admin: creator.clone(),
        admins: vec![creator],
        settings: request.settings.unwrap_or_default(),
    };

    COLLECTIONS.with(|c| {
        c.borrow_mut()
            .insert(request.id.clone(), collection.clone())
    });

    // Initialize empty vector index for this collection in vectors module
    super::vectors::init_collection_index(&request.id);

    // Initialize empty document index for this collection
    super::documents::init_collection_document_index(&request.id);

    Ok(collection)
}

pub fn get_collection(collection_id: &str) -> Option<Collection> {
    COLLECTIONS.with(|c| c.borrow().get(&collection_id.to_string()))
}

pub fn list_collections() -> Vec<Collection> {
    COLLECTIONS.with(|c| {
        c.borrow()
            .iter()
            .map(|(_, collection)| collection)
            .collect()
    })
}

pub fn get_collection_with_stats(collection_id: &str) -> Option<CollectionWithStats> {
    let collection = get_collection(collection_id)?;

    // Compute stats on-demand from actual data
    let stats = CollectionStats {
        document_count: super::documents::count_collection_documents(collection_id) as u32,
        vector_count: super::vectors::get_collection_vectors(collection_id).len() as u32,
        created_at: collection.created_at,
        updated_at: collection.updated_at,
    };

    Some(CollectionWithStats { collection, stats })
}

pub fn list_collections_with_stats() -> Vec<CollectionWithStats> {
    list_collections()
        .into_iter()
        .filter_map(|collection| get_collection_with_stats(&collection.id))
        .collect()
}

// =============================================================================
// ADMIN MANAGEMENT (Fixed to match Motoko logic)
// =============================================================================

pub fn is_collection_admin(collection_id: &str, caller: &str) -> bool {
    COLLECTIONS.with(|c| {
        if let Some(collection) = c.borrow().get(&collection_id.to_string()) {
            collection.genesis_admin == caller || collection.admins.contains(&caller.to_string())
        } else {
            false
        }
    })
}

pub fn add_collection_admin(
    collection_id: &str,
    new_admin: &str,
    caller: &str,
) -> Result<(), String> {
    COLLECTIONS.with(|c| {
        if let Some(mut collection) = c.borrow().get(&collection_id.to_string()) {
            if collection.genesis_admin != caller {
                return Err("Only the genesis admin can add new admins".to_string());
            }

            if collection.admins.contains(&new_admin.to_string()) {
                return Err("User is already an admin".to_string());
            }

            collection.admins.push(new_admin.to_string());
            collection.updated_at = current_time();

            c.borrow_mut().insert(collection_id.to_string(), collection);
            Ok(())
        } else {
            Err("Collection not found".to_string())
        }
    })
}

pub fn remove_collection_admin(
    collection_id: &str,
    admin_to_remove: &str,
    caller: &str,
) -> Result<(), String> {
    COLLECTIONS.with(|c| {
        if let Some(mut collection) = c.borrow().get(&collection_id.to_string()) {
            if collection.genesis_admin != caller {
                return Err("Only the genesis admin can remove admins".to_string());
            }

            if admin_to_remove == collection.genesis_admin {
                return Err("Cannot remove the genesis admin".to_string());
            }

            if !collection.admins.contains(&admin_to_remove.to_string()) {
                return Err("User is not an admin".to_string());
            }

            collection.admins.retain(|admin| admin != admin_to_remove);
            collection.updated_at = current_time();

            c.borrow_mut().insert(collection_id.to_string(), collection);
            Ok(())
        } else {
            Err("Collection not found".to_string())
        }
    })
}

pub fn transfer_genesis_admin(
    collection_id: &str,
    new_genesis_admin: &str,
    caller: &str,
) -> Result<(), String> {
    COLLECTIONS.with(|c| {
        if let Some(mut collection) = c.borrow().get(&collection_id.to_string()) {
            if collection.genesis_admin != caller {
                return Err("Only the current genesis admin can transfer ownership".to_string());
            }

            if !collection.admins.contains(&new_genesis_admin.to_string()) {
                return Err("New genesis admin must be an existing admin".to_string());
            }

            collection.genesis_admin = new_genesis_admin.to_string();
            collection.updated_at = current_time();

            c.borrow_mut().insert(collection_id.to_string(), collection);
            Ok(())
        } else {
            Err("Collection not found".to_string())
        }
    })
}

pub fn update_collection_settings(
    collection_id: &str,
    settings: CollectionSettings,
    caller: &str,
) -> Result<(), String> {
    // Any admin can update settings (unchanged behavior)
    require_admin_access(collection_id, caller)?;

    COLLECTIONS.with(|c| {
        let mut collections = c.borrow_mut();
        if let Some(mut collection) = collections.get(&collection_id.to_string()) {
            collection.settings = settings;
            collection.updated_at = current_time();
            collections.insert(collection_id.to_string(), collection);
            Ok(())
        } else {
            Err(format!("Collection '{}' not found", collection_id))
        }
    })
}

pub fn update_collection_metadata(
    collection_id: &str,
    name: Option<String>,
    description: Option<String>,
    caller: &str,
) -> Result<(), String> {
    require_admin_access(collection_id, caller)?;

    COLLECTIONS.with(|c| {
        let mut collections = c.borrow_mut();
        if let Some(mut collection) = collections.get(&collection_id.to_string()) {
            if let Some(new_name) = name {
                if new_name.trim().is_empty() || new_name.len() > 100 {
                    return Err("Collection name must be 1-100 characters".to_string());
                }
                collection.name = new_name;
            }

            if let Some(new_description) = description {
                if new_description.len() > 500 {
                    return Err("Collection description must be 500 characters or less".to_string());
                }
                collection.description = Some(new_description);
            }

            collection.updated_at = current_time();
            collections.insert(collection_id.to_string(), collection);
            Ok(())
        } else {
            Err(format!("Collection '{}' not found", collection_id))
        }
    })
}

pub fn delete_collection(collection_id: &str, caller: &str) -> Result<(), String> {
    require_genesis_admin(collection_id, caller)?;

    COLLECTIONS.with(|c| {
        if c.borrow_mut().remove(&collection_id.to_string()).is_some() {
            // Clean up associated indexes
            super::vectors::cleanup_collection_index(collection_id);
            super::documents::cleanup_collection_document_index(collection_id);
            Ok(())
        } else {
            Err("Collection not found".to_string())
        }
    })
}
// =============================================================================
// UTILITY FUNCTIONS (Simplified - no bloat)
// =============================================================================

pub fn count_collections() -> u64 {
    COLLECTIONS.with(|c| c.borrow().len())
}

pub fn collection_exists(collection_id: &str) -> bool {
    COLLECTIONS.with(|c| c.borrow().contains_key(&collection_id.to_string()))
}


pub fn get_collection_admins(collection_id: &str) -> Vec<String> {
    COLLECTIONS.with(|c| {
        c.borrow()
            .get(&collection_id.to_string())
            .map(|collection| collection.admins.clone())
            .unwrap_or_default()
    })
}

pub fn get_genesis_admin(collection_id: &str) -> Option<String> {
    COLLECTIONS.with(|c| {
        c.borrow()
            .get(&collection_id.to_string())
            .map(|collection| collection.genesis_admin.clone())
    })
}

pub fn get_admin_level(collection_id: &str, caller: &str) -> AdminLevel {
    COLLECTIONS.with(|c| {
        if let Some(collection) = c.borrow().get(&collection_id.to_string()) {
            if collection.genesis_admin == caller {
                AdminLevel::Genesis
            } else if collection.admins.contains(&caller.to_string()) {
                AdminLevel::Regular
            } else {
                AdminLevel::None
            }
        } else {
            AdminLevel::None
        }
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdminLevel {
    Genesis,
    Regular,
    None,
}

pub fn require_admin_access(collection_id: &str, caller: &str) -> Result<AdminLevel, String> {
    let level = get_admin_level(collection_id, caller);
    match level {
        AdminLevel::Genesis | AdminLevel::Regular => Ok(level),
        AdminLevel::None => Err("Admin access required".to_string()),
    }
}

pub fn require_genesis_admin(collection_id: &str, caller: &str) -> Result<(), String> {
    let level = get_admin_level(collection_id, caller);
    match level {
        AdminLevel::Genesis => Ok(()),
        AdminLevel::Regular => Err("Genesis admin access required".to_string()),
        AdminLevel::None => Err("Admin access required".to_string()),
    }
}

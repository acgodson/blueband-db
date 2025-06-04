// compute/cache.rs - FIXED: Bounded LRU Cache with Memory Limits
use crate::types::*;
use std::cell::RefCell;
use std::collections::HashMap;

/// Cache entry with vectors and access tracking
#[derive(Clone, Debug)]
struct CacheEntry {
    vectors: Vec<Vector>,
    timestamp: u64,
    last_accessed: u64,
    memory_size: usize, // Track memory usage of this entry
}

/// LRU cache with memory bounds
#[derive(Debug)]
struct BoundedCache {
    entries: HashMap<String, CacheEntry>,
    access_order: Vec<String>, // LRU tracking (most recent at end)
    total_memory: usize,       // Current memory usage
    max_memory: usize,         // Memory limit
    max_entries: usize,        // Entry count limit
}

impl BoundedCache {
    fn new(max_memory: usize, max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            access_order: Vec::new(),
            total_memory: 0,
            max_memory,
            max_entries,
        }
    }

    fn get(&mut self, collection_id: &str) -> Option<Vec<Vector>> {
        let now = current_time();

        // First check if entry exists and is valid
        let is_valid = if let Some(entry) = self.entries.get(collection_id) {
            now - entry.timestamp < CACHE_TTL
        } else {
            false
        };

        if is_valid {
            // Update access time
            if let Some(entry) = self.entries.get_mut(collection_id) {
                entry.last_accessed = now;
                let vectors = entry.vectors.clone();

                // Update LRU order (safe now that we're done with the entry)
                self.update_lru_order(collection_id);
                return Some(vectors);
            }
        } else if self.entries.contains_key(collection_id) {
            // Entry expired, remove it
            self.remove_entry(collection_id);
        }

        None
    }

    fn insert(&mut self, collection_id: String, vectors: Vec<Vector>) {
        let now = current_time();
        let memory_size = estimate_vectors_memory_size(&vectors);

        // Remove existing entry if present
        if self.entries.contains_key(&collection_id) {
            self.remove_entry(&collection_id);
        }

        // Ensure we have space for the new entry
        self.make_space_for(memory_size, 1);

        let entry = CacheEntry {
            vectors,
            timestamp: now,
            last_accessed: now,
            memory_size,
        };

        // Insert new entry
        self.entries.insert(collection_id.clone(), entry);
        self.access_order.push(collection_id);
        self.total_memory += memory_size;
    }

    fn remove(&mut self, collection_id: &str) {
        self.remove_entry(collection_id);
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        self.total_memory = 0;
    }

    fn remove_entry(&mut self, collection_id: &str) {
        if let Some(entry) = self.entries.remove(collection_id) {
            self.total_memory = self.total_memory.saturating_sub(entry.memory_size);
            self.access_order.retain(|id| id != collection_id);
        }
    }

    fn update_lru_order(&mut self, collection_id: &str) {
        // Move to end (most recently used)
        if let Some(pos) = self.access_order.iter().position(|id| id == collection_id) {
            let id = self.access_order.remove(pos);
            self.access_order.push(id);
        }
    }

    fn make_space_for(&mut self, needed_memory: usize, needed_entries: usize) {
        // Remove expired entries first
        self.remove_expired_entries();

        // If still need space, remove LRU entries
        while (self.total_memory + needed_memory > self.max_memory
            || self.entries.len() + needed_entries > self.max_entries)
            && !self.access_order.is_empty()
        {
            let lru_id = self.access_order.remove(0); // Remove least recently used
            self.entries.remove(&lru_id);
        }

        // Recalculate total memory after cleanup
        self.recalculate_memory();
    }

    fn remove_expired_entries(&mut self) {
        let now = current_time();
        let expired_ids: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, entry)| now - entry.timestamp >= CACHE_TTL)
            .map(|(id, _)| id.clone())
            .collect();

        for id in expired_ids {
            self.remove_entry(&id);
        }
    }

    fn recalculate_memory(&mut self) {
        self.total_memory = self.entries.values().map(|entry| entry.memory_size).sum();
    }

    fn get_stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.entries.len(),
            total_memory_bytes: self.total_memory,
            max_memory_bytes: self.max_memory,
            max_entries: self.max_entries,
            memory_usage_percent: (self.total_memory as f64 / self.max_memory as f64 * 100.0)
                as u32,
        }
    }
}

thread_local! {
    static CACHE: RefCell<BoundedCache> = RefCell::new(
        BoundedCache::new(
            100 * 1024 * 1024,  // 100MB memory limit
            1000,                // 1000 collections max
        )
    );
}

/// Cache TTL in nanoseconds (24 hours)
const CACHE_TTL: u64 = 24 * 60 * 60 * 1_000_000_000;

/// Get vectors from cache or storage (BOUNDED)
pub fn get_cached_vectors(collection_id: &str) -> Vec<Vector> {
    let cache_hit = CACHE.with(|cache| cache.borrow_mut().get(collection_id));

    match cache_hit {
        Some(vectors) => vectors,
        None => {
            // Load from storage and cache
            let vectors = crate::storage::get_collection_vectors(collection_id);

            if !vectors.is_empty() {
                CACHE.with(|cache| {
                    cache
                        .borrow_mut()
                        .insert(collection_id.to_string(), vectors.clone());
                });
            }

            vectors
        }
    }
}

/// Invalidate cache for a collection when data changes
pub fn invalidate_collection_cache(collection_id: &str) {
    CACHE.with(|cache| {
        cache.borrow_mut().remove(collection_id);
    });
}

/// Clear all cache
pub fn clear_cache() {
    CACHE.with(|cache| {
        cache.borrow_mut().clear();
    });
}

/// Get cache statistics for monitoring
pub fn get_cache_stats() -> CacheStats {
    CACHE.with(|cache| cache.borrow().get_stats())
}

/// Force cache cleanup (remove expired entries)
pub fn cleanup_cache() -> u32 {
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let before_count = cache.entries.len();
        cache.remove_expired_entries();
        let after_count = cache.entries.len();
        (before_count - after_count) as u32
    })
}

/// Estimate memory usage of vectors for cache management
fn estimate_vectors_memory_size(vectors: &[Vector]) -> usize {
    let mut total_size = 0;

    for vector in vectors {
        // Vector struct overhead
        total_size += std::mem::size_of::<Vector>();

        // Embedding data (f32 slice)
        total_size += vector.embedding.len() * std::mem::size_of::<f32>();

        // String fields (IDs, model name)
        total_size += vector.id.len();
        total_size += vector.document_id.len();
        total_size += vector.chunk_id.len();
        total_size += vector.model.len();
    }

    total_size
}

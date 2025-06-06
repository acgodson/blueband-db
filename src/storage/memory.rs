// storage/memory.rs
use candid::CandidType;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, Memory as IcMemory};
use std::cell::RefCell;

pub type MemoryType = VirtualMemory<DefaultMemoryImpl>;
pub type MemManager = MemoryManager<DefaultMemoryImpl>;

// Memory layout for stable structures
// Each stable structure gets its own isolated memory space
pub const COLLECTIONS_MEMORY_ID: MemoryId = MemoryId::new(0);
pub const DOCUMENTS_MEMORY_ID: MemoryId = MemoryId::new(1);
pub const CHUNKS_MEMORY_ID: MemoryId = MemoryId::new(2);
pub const VECTORS_MEMORY_ID: MemoryId = MemoryId::new(3);
pub const VECTOR_INDEX_MEMORY_ID: MemoryId = MemoryId::new(4);
pub const CONFIG_MEMORY_ID: MemoryId = MemoryId::new(5);
pub const DOCUMENT_INDEX_MEMORY_ID: MemoryId = MemoryId::new(6); 

// Performance tracking
// pub const METRICS_MEMORY_ID: MemoryId = MemoryId::new(10);

// pub const GOVERNANCE_MEMORY_ID: MemoryId = MemoryId::new(20);
// pub const PROPOSALS_MEMORY_ID: MemoryId = MemoryId::new(21);
// pub const VOTES_MEMORY_ID: MemoryId = MemoryId::new(22);

// Global memory manager instance
thread_local! {
    static MEMORY_MANAGER: RefCell<MemManager> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );
}

/// Get a virtual memory instance for the given memory ID
pub fn get_memory(memory_id: MemoryId) -> MemoryType {
    MEMORY_MANAGER.with(|m| m.borrow().get(memory_id))
}

/// Get memory statistics for monitoring
pub fn get_memory_stats() -> MemoryStats {
    MEMORY_MANAGER.with(|m| {
        let manager = m.borrow();
        let base_memory = manager.get(MemoryId::new(0));
        let total_pages = base_memory.size();
        let used_pages = estimate_used_pages();
        let available_pages = total_pages.saturating_sub(used_pages);

        MemoryStats {
            total_pages,
            used_pages,
            total_bytes: total_pages * 65536, // 64KB per page
            available_bytes: available_pages * 65536,
        }
    })
}

fn estimate_used_pages() -> u64 {
    MEMORY_MANAGER.with(|m| {
        let manager = m.borrow();
        let mut total = 0u64;

        // Check key memory spaces
        for id in [
            COLLECTIONS_MEMORY_ID,
            DOCUMENTS_MEMORY_ID,
            CHUNKS_MEMORY_ID,
            VECTORS_MEMORY_ID,
            VECTOR_INDEX_MEMORY_ID,
            DOCUMENT_INDEX_MEMORY_ID, 
        ] {
            total += manager.get(id).size();
        }

        total
    })
}

#[derive(CandidType, Debug, Clone)]
pub struct MemoryStats {
    pub total_pages: u64,
    pub used_pages: u64,
    pub total_bytes: u64,
    pub available_bytes: u64,
}

// impl MemoryStats {
//     pub fn usage_percentage(&self) -> f64 {
//         if self.total_pages == 0 {
//             0.0
//         } else {
//             (self.used_pages as f64 / self.total_pages as f64) * 100.0
//         }
//     }

//     pub fn available_gb(&self) -> f64 {
//         self.available_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
//     }

//     pub fn used_gb(&self) -> f64 {
//         (self.used_pages * 65536) as f64 / (1024.0 * 1024.0 * 1024.0)
//     }
// }

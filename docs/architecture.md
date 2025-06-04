# Architecture

Blueband DB is built with a modular, layered architecture specifically designed for the Internet Computer Protocol (ICP). This document details the system's architecture and design decisions.

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    API Layer (lib.rs)                      │
├─────────────────────────────────────────────────────────────┤
│  Compute Layer     │  Storage Layer      │  Types Layer    │
│  - Embeddings      │  - Collections      │  - Definitions  │
│  - Similarity      │  - Documents        │  - Validation   │
│  - Cache           │  - Vectors          │  - Utilities    │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. API Layer (`lib.rs`)

The API layer provides the public interface for interacting with Blueband DB. It handles:
- Request validation
- Permission checking
- Error handling
- Response formatting

### 2. Compute Layer

Responsible for all computational operations:

#### Embeddings Module
- HTTP outcall management for external embedding services
- Cycle calculation and management
- Response transformation and validation

#### Similarity Module
- Vector similarity calculations
- Algorithm selection (exact vs approximate)
- Search result ranking

#### Cache Module
- LRU cache implementation
- Memory tracking
- Cache invalidation
- TTL management

### 3. Storage Layer

Manages all persistent data:

#### Collections
- Collection metadata storage
- Admin permission management
- Settings and configuration

#### Documents
- Document content storage
- Chunk management
- Metadata indexing

#### Vectors
- Vector embedding storage
- Index management
- Vector validation

### 4. Types Layer

Provides type definitions and validation:

#### Definitions
- Core data structures
- API types
- Configuration types

#### Validation
- Input validation
- Type checking
- Error types

#### Utilities
- Helper functions
- Common operations
- Type conversions

## Memory Management

### Stable Structure Layout

Blueband DB uses isolated memory spaces to prevent data corruption and enable efficient access:

```
COLLECTIONS_MEMORY_ID: 0     // Collection metadata
DOCUMENTS_MEMORY_ID: 1       // Document metadata  
CHUNKS_MEMORY_ID: 2          // Semantic chunks
VECTORS_MEMORY_ID: 3         // Vector embeddings
VECTOR_INDEX_MEMORY_ID: 4    // Collection→Vector mapping
DOCUMENT_INDEX_MEMORY_ID: 6  // Collection→Document mapping
```

### Memory Optimization

1. **Isolated Memory Spaces**
   - Each data type gets its own virtual memory space
   - Prevents memory fragmentation
   - Enables O(1) collection-based lookups

2. **Bounded Resource Management**
   ```rust
   const MAX_DOCUMENT_SIZE: usize = 10 * 1024 * 1024;  // 10MB
   const MAX_BATCH_SIZE: usize = 50;
   const MAX_EMBEDDING_DIMENSIONS: usize = 65_536;
   const CACHE_MEMORY_LIMIT: usize = 100 * 1024 * 1024;  // 100MB
   const CACHE_TTL: u64 = 86_400;  // 24 hours
   ```

3. **Cache Strategy**
   - Bounded LRU cache
   - Memory tracking per entry
   - Intelligent invalidation
   - TTL-based expiration

## Data Flow

### 1. Document Ingestion

```
User Input → Validation → Chunking → Storage → [Optional] Embedding → Indexing
```

Detailed flow:
```rust
add_document_and_embed() {
    1. Validate: Admin permissions, content size, collection exists
    2. Chunk: Split content using collection's chunk_size/overlap settings  
    3. Store: Document metadata + chunks in separate stable structures
    4. Embed: HTTP outcall to OpenAI/custom API for vector generation
    5. Index: Store vectors + update collection→vector index
    6. Cache: Invalidate collection cache to include new vectors
}
```

### 2. Search Query Flow

```
Query Text → Embedding → Vector Search → Enrichment → Ranking → Results
```

Detailed flow:
```rust
search() {
    1. Embed Query: HTTP outcall to convert text → embedding vector
    2. Load Vectors: Get from cache OR storage + cache for next time
    3. Choose Algorithm: 
       - Small collections (<1000): Exact cosine similarity
       - Large collections: Hierarchical approximate search (k-means)
    4. Compute Similarities: Dot product / (norm_a * norm_b)
    5. Filter & Rank: Apply min_score threshold, sort by similarity
    6. Enrich Results: Add document titles and chunk text
    7. Return: Top-k ranked matches with metadata
}
```

## Security Model

### Three-Tier Admin System

```rust
Genesis Admin:
    - Original collection creator
    - Can add/remove admins
    - Can delete collection
    - Can transfer ownership

Regular Admin:  
    - Can manage documents/vectors
    - Can update collection settings
    - Cannot manage other admins

No Access:
    - Cannot perform any operations
```

### Permission Inheritance

- Collections are completely isolated
- Admins of one collection have no access to others
- Permissions are checked before any operation
- Fail-fast validation prevents unauthorized access

## Scalability Design

### Horizontal Scaling

1. **Collection-Based Partitioning**
   - Each collection is independent
   - Can be moved to separate canisters
   - Load balancing via collection distribution

2. **Memory-Efficient Operations**
   - Streaming chunk processing
   - Bounded batch operations
   - Cache-aware vector loading

### Vertical Scaling

1. **Hierarchical Indexing**
   - O(1) collection filtering
   - Efficient bulk operations
   - Natural data isolation

2. **Adaptive Algorithms**
   - Exact search for small collections
   - Approximate search for large collections
   - Automatic algorithm selection

## Error Handling

### Philosophy

**Fail fast, fail clearly, fail safely.**

```rust
// Validation Errors
if !is_valid_input(input) {
    return Err(Error::InvalidInput);
}

// Resource Errors
if !has_sufficient_cycles() {
    return Err(Error::InsufficientCycles);
}

// External Errors
match external_call() {
    Ok(result) => process_result(result),
    Err(e) => handle_external_error(e),
}

// Storage Errors
match stable_storage.get() {
    Ok(data) => process_data(data),
    Err(e) => handle_storage_error(e),
}
```

### Error Categories

1. **Validation Errors**
   - Return immediately with specific message
   - No state changes
   - Clear error codes

2. **Resource Errors**
   - Check availability before operations
   - Cycle management
   - Memory limits

3. **External Errors**
   - HTTP outcall failures
   - API rate limits
   - Network issues

4. **Storage Errors**
   - Stable storage failures
   - Index corruption
   - Data consistency

## Performance Optimization

### 1. Caching Strategy

```rust
CacheStrategy {
    Memory Limit: 100MB
    Entry Limit: 1000 collections  
    TTL: 24 hours
    Eviction: LRU + expired entry cleanup
    
    get() → update_access_time() → move_to_lru_end()
    insert() → evict_if_needed() → store_with_metadata()
}
```

### 2. Search Optimization

```rust
// Small collections: Exact search
if vectors.len() < 1000 {
    exact_similarity_search(query, vectors)
} else {
    // Large collections: Approximate search
    hierarchical_approximate_search(query, vectors)
}
```

### 3. Memory Management

```rust
// Bounded operations
const MAX_BATCH_SIZE: usize = 50;
const MAX_DOCUMENT_SIZE: usize = 10 * 1024 * 1024;

// Memory tracking
struct MemoryTracker {
    current_usage: usize,
    max_usage: usize,
    entries: HashMap<String, usize>,
}
```

## Technical Innovations

1. **Stable Structure Composition**
   - Multiple coordinated stable structures
   - Efficient partial updates
   - Better memory management

2. **Adaptive Search Algorithms**
   - Automatic algorithm selection
   - Optimized for both accuracy and performance
   - Collection-size aware

3. **Cycle-Aware External Calls**
   - Dynamic cycle calculation
   - Buffers for network variability
   - Prevents failed operations

4. **Semantic Chunking**
   - Configurable chunk size/overlap
   - Context preservation
   - Search granularity

5. **Index Invalidation Strategy**
   - Selective cache invalidation
   - Performance optimization
   - Consistency maintenance

## Future Considerations

1. **Planned Improvements**
   - Sharding support
   - Additional embedding models
   - Enhanced monitoring
   - Backup/restore functionality

2. **Research Areas**
   - Alternative search algorithms
   - Compression techniques
   - Distributed caching
   - Cross-canister operations

3. **Integration Opportunities**
   - Additional AI models
   - Monitoring tools
   - Backup solutions
   - Analytics platforms 
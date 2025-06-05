# Vector Search

Blueband uses hierarchical vector indexing and cosine similarity to enable smooth semantic search across document collections. 

## Search Algorithm

The search process involves several key steps:

1. **Query Processing**
   - Text query is converted to vector embedding using the collection's configured model
   - Embedding is validated for proper dimensions and values
   - Vector norm is pre-computed

2. **Search Method Selection**
   - For collections > 1000 vectors:
     - If `use_approximate` is true (default): Uses fast approximate search with k-means clustering
     - If `use_approximate` is false: Uses exact cosine similarity search (slower but more accurate)
   - For smaller collections: Always uses exact cosine similarity search
   - The choice between approximate and exact search is configurable per request

3. **Similarity Computation**
   - Cosine similarity between query and document vectors
   - Score range: 0.0 (dissimilar) to 1.0 (identical)

## Search Configuration

```typescript
interface SimilarityConfig {
    min_score?: number;        // Minimum similarity threshold (0-1)
    max_results: number;       // Maximum results to return
    use_approximate: boolean;  // Use fast approximate search
    candidate_factor: number;  // Candidate multiplier for accuracy
}
```

## Search Methods

| Method                    | Description                    | Algorithm             | Use Case               |
| ------------------------- | ------------------------------ | --------------------- | ---------------------- |
| `search`                  | Standard semantic search       | Hierarchical + Cosine | General purpose search |
| `search_filtered`         | Search with document filtering | Hierarchical + Cosine | Targeted search        |
| `find_similar_documents`  | Document similarity            | Centroid-based        | Related content        |
| `batch_similarity_search` | Multiple queries               | Parallel search       | Bulk operations        |

## Search Parameters

```typescript
interface SearchRequest {
    collection_id: string;     // Target collection
    query: string;            // Search text
    limit?: number;           // Max results (default: 10)
    min_score?: number;       // Min similarity (0-1)
    filter?: string[];        // Document ID filter
    use_approximate?: boolean; // Whether to use fast approximate search (default: true)
}
```

## Search Results

```typescript
interface SearchResult {
    document_id: string;      // Source document
    chunk_id: string;         // Matching chunk
    text: string;            // Chunk content
    score: number;           // Similarity score (0-1)
    metadata?: Record<string, string>;  // Optional metadata
}
```

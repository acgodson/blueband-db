# Vector Search

Vector search functions enable semantic similarity search across document collections using vector embeddings.

## Search

Performs semantic similarity search across a collection using a text query.

```typescript
search(params: {
    collection_id: string;
    query: string;
    limit?: number;           // Maximum number of results
    min_score?: number;       // Minimum similarity score (0-1)
    metadata_filter?: Record<string, string>;  // Optional metadata filtering
}): Promise<{
    matches: Array<{
        document_id: string;
        chunk_id: string;
        text: string;
        score: number;
        metadata?: Record<string, string>;
    }>;
    total_found: number;
    query_time_ms: number;
}>;
```

**Example:**
```typescript
const results = await actor.search({
    collection_id: "research_papers",
    query: "What are the latest developments in quantum computing?",
    limit: 5,
    min_score: 0.7,
    metadata_filter: {
        year: "2023",
        category: "quantum"
    }
});
```

## Search Filtered

Semantic search with document ID filtering capabilities.

```typescript
search_filtered(request: {
    collection_id: string;
    query: string;
    limit?: number;
    min_score?: number;
    filter?: Record<string, string>;  // Metadata filters
}): Promise<Array<{
    document_id: string;
    chunk_id: string;
    score: number;
    text: string;
}>>;
```

## Find Similar Documents

Finds documents similar to a given document.

```typescript
find_similar_documents(
    document_id: string,
    collection_id: string,
    limit?: number,
    min_score?: number
): Promise<{
    matches: Array<{
        document_id: string;
        chunk_id: string;
        text: string;
        score: number;
        metadata?: Record<string, string>;
    }>;
    total_found: number;
    query_time_ms: number;
}>;
```

## Batch Similarity Search

Performs similarity search across multiple collections simultaneously.

```typescript
batch_similarity_search(
    queries: Array<string>,
    collection_id: string,
    limit?: number,
    min_score?: number
): Promise<Array<{
    query: string;
    matches: Array<{
        document_id: string;
        chunk_id: string;
        text: string;
        score: number;
        metadata?: Record<string, string>;
    }>;
    total_found: number;
    query_time_ms: number;
}>>;
```

## Demo Vector Similarity

A simplified search function for testing and demonstration purposes.

```typescript
demo_vector_similarity(
    documents: Array<string>,
    query: string,
    proxy_url: string,
    limit?: number,
    min_score?: number
): Promise<{
    matches: Array<{
        document_id: string;
        chunk_id: string;
        text: string;
        score: number;
    }>;
    total_found: number;
    query_time_ms: number;
}>;
```

## Error Handling

All vector search functions can return the following errors:

```typescript
type SearchError = {
    NotFound: string;           // Collection or document not found
    NotAuthorized: string;      // Caller lacks required permissions
    InvalidInput: string;       // Invalid parameters provided
    EmbeddingError: string;     // Failed to generate embeddings
    InvalidProxy: string;       // Invalid proxy URL
    SearchError: string;        // General search error
};
```

## Best Practices

1. **Query Optimization**
   - Keep queries concise and focused
   - Use appropriate minimum score thresholds
   - Consider using metadata filters for better results

2. **Performance Considerations**
   - Use appropriate limit values
   - Consider batch operations for multiple queries
   - Monitor query times for optimization

3. **Result Handling**
   - Always check total_found count
   - Consider score thresholds for quality
   - Use metadata for additional filtering 
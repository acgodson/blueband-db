# Vector Search API

Core functions for semantic search and similarity operations.

## Search

Performs semantic similarity search using query text and returns ranked results.

```typescript
search(request: {
    collection_id: string;
    query: string;
    limit?: number;        // Default: 10
    min_score?: number;    // Default: 0.7
}): Promise<Array<{
    document_id: string;
    chunk_id: string;
    score: number;
    text: string;
}>>;
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

Finds documents similar to a given source document using centroid.

```typescript
find_similar_documents(
    source_document_id: string,
    collection_id: string,
    limit?: number,        // Default: 10
    min_score?: number     // Default: 0.7
): Promise<Array<{
    document_id: string;
    chunk_id: string;
    score: number;
    text: string;
}>>;
```

## Batch Similarity Search

Processes multiple search queries in a single call.

```typescript
batch_similarity_search(
    queries: Array<string>,
    collection_id: string,
    limit?: number,
    min_score?: number
): Promise<Array<Array<{
    document_id: string;
    chunk_id: string;
    score: number;
    text: string;
}>>>;
```

## Demo Vector Similarity

Creates temporary in-memory index for testing without persistence.

```typescript
demo_vector_similarity(
    items: Array<string>,
    query: string,
    proxy_url: string,
    max_results?: number,  // Default: 5
    min_score?: number     // Default: 0.7
): Promise<Array<{
    document_id: string;
    chunk_id: string;
    score: number;
    text: string;
}>>;
```

## Error Handling

```typescript
type SearchError = {
    NotFound: string;           // Collection or document not found
    NotAuthorized: string;      // Caller lacks required permissions
    InvalidInput: string;       // Invalid parameters
    EmbeddingError: string;     // Failed to generate embeddings
    InvalidVector: string;      // Invalid vector dimensions
};
``` 
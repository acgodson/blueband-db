# Document Management API

Document management functions handle the lifecycle of documents in a collection, including adding, retrieving, updating, and deleting documents and their chunks.

## Add Document

Adds a document to a collection without generating embeddings. The document will be chunked according to collection settings.

```typescript
add_document(params: {
    collection_id: string;
    document_id: string;
    content: string;
    metadata?: Record<string, string>;  // Optional key-value pairs
}): Promise<{
    document_id: string;
    chunk_count: number;
    created_at: bigint;
}>;
```

**Example:**
```typescript
const result = await actor.add_document({
    collection_id: "col_123",
    document_id: "doc_456",
    content: "This is a sample document that will be chunked automatically.",
    metadata: {
        author: "John Doe",
        category: "example"
    }
});
```

## Add Document and Embed

Adds a document, chunks it, and generates embeddings in one operation.

```typescript
add_document_and_embed(
    params: {
        collection_id: string;
        document_id: string;
        content: string;
        metadata?: Record<string, string>;
    },
    proxy_url: string  // OpenAI embedding proxy URL
): Promise<{
    document_id: string;
    chunk_count: number;
    vector_count: number;
    created_at: bigint;
}>;
```

**Example:**
```typescript
const result = await actor.add_document_and_embed({
    collection_id: "col_123",
    document_id: "doc_456",
    content: "This document will be chunked and embedded immediately.",
    metadata: {
        source: "web",
        language: "en"
    }
}, "https://your-proxy-url");
```

## Get Document

Retrieves document metadata and chunk information.

```typescript
get_document(
    collection_id: string,
    document_id: string
): Promise<{
    document_id: string;
    metadata: Record<string, string>;
    chunk_count: number;
    vector_count: number;
    created_at: bigint;
    updated_at: bigint;
}>;
```

## Get Document Content

Retrieves the full document content.

```typescript
get_document_content(
    collection_id: string,
    document_id: string
): Promise<string>;
```

## Get Document Chunks

Retrieves all chunks of a document with their metadata.

```typescript
get_document_chunks(
    collection_id: string,
    document_id: string
): Promise<Array<{
    chunk_id: string;
    content: string;
    metadata: Record<string, string>;
    has_embedding: boolean;
    created_at: bigint;
}>>;
```

## Update Document

Updates document content and/or metadata. This will re-chunk the document.

```typescript
update_document(params: {
    collection_id: string;
    document_id: string;
    content?: string;
    metadata?: Record<string, string>;
}): Promise<{
    document_id: string;
    chunk_count: number;
    updated_at: bigint;
}>;
```

**Example:**
```typescript
await actor.update_document({
    collection_id: "col_123",
    document_id: "doc_456",
    content: "Updated document content",
    metadata: {
        status: "revised",
        updated_by: "Jane Smith"
    }
});
```

## Update Document Metadata

Updates only the document metadata without re-chunking.

```typescript
update_document_metadata(
    collection_id: string,
    document_id: string,
    metadata: Record<string, string>
): Promise<void>;
```

## Delete Document

Removes a document and all its chunks and vectors.

```typescript
delete_document(
    collection_id: string,
    document_id: string
): Promise<void>;
```

## Embed Existing Document

Generates embeddings for an existing document's chunks.

```typescript
embed_existing_document(
    collection_id: string,
    document_id: string,
    proxy_url: string  // OpenAI embedding proxy URL
): Promise<{
    document_id: string;
    vector_count: number;
    updated_at: bigint;
}>;
```

## Document Operations

### List Documents

Returns all documents in a collection with their metadata.

```typescript
list_documents(
    collection_id: string
): Promise<Array<{
    document_id: string;
    metadata: Record<string, string>;
    chunk_count: number;
    vector_count: number;
    created_at: bigint;
    updated_at: bigint;
}>>;
```

### Get Document Stats

Returns document statistics including chunk and vector counts.

```typescript
get_document_stats(
    collection_id: string,
    document_id: string
): Promise<{
    chunk_count: number;
    vector_count: number;
    created_at: bigint;
    updated_at: bigint;
    last_embedded_at?: bigint;
}>;
```

### Batch Operations

```typescript
// Add multiple documents
add_documents(
    collection_id: string,
    documents: Array<{
        document_id: string;
        content: string;
        metadata?: Record<string, string>;
    }>
): Promise<Array<{
    document_id: string;
    chunk_count: number;
    created_at: bigint;
}>>;

// Delete multiple documents
delete_documents(
    collection_id: string,
    document_ids: Array<string>
): Promise<void>;
```

## Error Handling

All document management functions can return the following errors:

```typescript
type DocumentError = {
    NotFound: string;           // Document not found
    NotAuthorized: string;      // Caller lacks required permissions
    InvalidInput: string;       // Invalid parameters provided
    AlreadyExists: string;      // Document with same ID exists
    EmbeddingError: string;     // Failed to generate embeddings
    InvalidMetadata: string;    // Invalid metadata format
    CollectionNotFound: string; // Collection does not exist
    InvalidProxy: string;       // Invalid proxy URL
};
```

## Best Practices

1. **Document IDs**
   - Use unique, descriptive IDs
   - Consider using UUIDs for large collections
   - Avoid special characters in IDs

2. **Metadata Usage**
   - Keep metadata keys consistent
   - Use meaningful key names
   - Consider metadata size limits

3. **Content Management**
   - Monitor document size
   - Consider chunk size impact
   - Plan for embedding generation

4. **Batch Operations**
   - Use batch operations for multiple documents
   - Monitor memory usage during batches
   - Handle partial failures appropriately

5. **Embedding Generation**
   - Use proxy URLs for embedding generation
   - Monitor embedding costs
   - Consider batching embedding requests 
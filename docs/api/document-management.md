# Document

A document in Blueband is a text-based content unit stored in a collection. Each document is automatically split into semantic chunks for efficient vector search. 

## Document Structure

```typescript
interface DocumentMetadata {
    id: string;              // Unique document identifier
    collection_id: string;   // Parent collection
    title: string;           // Document title
    content_type: ContentType; // PlainText, Markdown, Html, Pdf, or Other
    source_url?: string;     // Optional source URL
    timestamp: bigint;       // Creation timestamp
    total_chunks: number;    // Number of semantic chunks
    size: bigint;           // Content size in bytes
    is_embedded: boolean;    // Whether vectors are generated
    checksum: string;        // Content hash for integrity
}

type ContentType = 
    | "PlainText"
    | "Markdown"
    | "Html"
    | "Pdf"
    | { Other: string };
```

## Document Operations

| Method                    | Description                    | Parameters                                                                 | Returns                       |
| ------------------------- | ------------------------------ | -------------------------------------------------------------------------- | ----------------------------- |
| `add_document`            | Add document without embedding | `{ collection_id, title, content, content_type?, source_url? }`            | `Promise<DocumentMetadata>`   |
| `add_document_and_embed`  | Add and embed in one step      | `{ collection_id, title, content, content_type?, source_url? }, proxy_url` | `Promise<DocumentMetadata>`   |
| `get_document`            | Get document metadata          | `collection_id: string, document_id: string`                               | `Promise<DocumentMetadata>`   |
| `get_document_content`    | Get raw content                | `collection_id: string, document_id: string`                               | `Promise<string>`             |
| `get_document_chunks`     | Get all chunks                 | `document_id: string`                                                      | `Promise<SemanticChunk[]>`    |
| `list_documents`          | List collection documents      | `collection_id: string`                                                    | `Promise<DocumentMetadata[]>` |
| `delete_document`         | Remove document and data       | `collection_id: string, document_id: string`                               | `Promise<void>`               |
| `embed_existing_document` | Generate vectors               | `collection_id: string, document_id: string`                               | `Promise<number>`             |

## Chunk Structure

```typescript
interface SemanticChunk {
    id: string;           // Unique chunk identifier
    document_id: string;  // Parent document
    text: string;         // Chunk content
    position: number;     // Order in document
    char_start: bigint;   // Start position
    char_end: bigint;     // End position
    token_count?: number; // Estimated tokens
}
```

## Error Handling

```typescript
type DocumentError = {
    NotFound: string;           // Document not found
    NotAuthorized: string;      // Caller lacks permissions
    InvalidInput: string;       // Invalid parameters
    AlreadyExists: string;      // Duplicate document
    EmbeddingError: string;     // Vector generation failed
    CollectionNotFound: string; // Parent collection missing
    InvalidProxy: string;       // Invalid embedding proxy
};
```

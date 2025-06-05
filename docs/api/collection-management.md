# Collection

A collection is a logical container for documents in Blueband, similar to a database in traditional systems. Each collection has its own settings for document chunking and vector embeddings, and maintains its own access control list. Collections are stored in stable memory with separate indexes for documents and vectors.

## Create Collection

Creates a new collection with specified settings. The caller becomes the genesis admin of the collection.

```typescript
import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { idlFactory } from "./declarations/blueband_rust.did.js";

interface CreateCollectionRequest {
    id: string;              // Unique identifier (1-64 chars, alphanumeric + _-)
    name: string;            // Display name (1-100 chars)
    description?: string;    // Optional description (max 500 chars)
    settings?: {            // Optional collection settings
        embedding_model: string;    // e.g. "text-embedding-3-small"
        proxy_url: string;         // OpenAI proxy URL
        chunk_size: number;        // Characters per chunk (default: 512)
        chunk_overlap: number;     // Overlap between chunks (default: 64)
        max_documents?: number;    // Optional document limit
        auto_embed: boolean;       // Auto-generate embeddings (default: true)
    };
}

interface Collection {
    id: string;
    name: string;
    description?: string;
    created_at: bigint;
    updated_at: bigint;
    genesis_admin: string;    // Principal ID of creator
    admins: string[];         // List of admin principals
    settings: CollectionSettings;
}

// Example usage
const agent = new HttpAgent({ host: "http://127.0.0.1:4943" });
const actor = Actor.createActor(idlFactory, {
    agent,
    canisterId: Principal.fromText("your-canister-id")
});

const collection = await actor.create_collection({
    id: "research_papers",
    name: "Research Papers",
    description: "Collection of academic research papers",
    settings: {
        embedding_model: "text-embedding-3-small",
        proxy_url: "https://api.openai.com/v1",
        chunk_size: 1000,
        chunk_overlap: 200,
        auto_embed: true
    }
});
```

## Collection Management Methods

| Method                        | Description                     | Parameters                                                   | Returns                          |
| ----------------------------- | ------------------------------- | ------------------------------------------------------------ | -------------------------------- |
| `get_collection`              | Get collection metadata         | `collection_id: string`                                      | `Promise<Collection>`            |
| `list_collections`            | List all collections            | None                                                         | `Promise<Collection[]>`          |
| `get_collection_with_stats`   | Get collection with usage stats | `collection_id: string`                                      | `Promise<CollectionWithStats>`   |
| `list_collections_with_stats` | List all collections with stats | None                                                         | `Promise<CollectionWithStats[]>` |
| `update_collection_metadata`  | Update name/description         | `collection_id: string, name?: string, description?: string` | `Promise<void>`                  |
| `update_collection_settings`  | Update collection settings      | `collection_id: string, settings: CollectionSettings`        | `Promise<void>`                  |
| `delete_collection`           | Delete collection and all data  | `collection_id: string`                                      | `Promise<void>`                  |

## Access Control

The collection access control system has two levels:
- **Genesis Admin**: The original creator who can manage admins and all operations
- **Regular Admin**: Can manage content but not admins

| Method                    | Description                 | Parameters                                 | Returns                                   |
| ------------------------- | --------------------------- | ------------------------------------------ | ----------------------------------------- |
| `add_collection_admin`    | Add admin (genesis only)    | `collection_id: string, principal: string` | `Promise<void>`                           |
| `remove_collection_admin` | Remove admin (genesis only) | `collection_id: string, principal: string` | `Promise<void>`                           |
| `transfer_genesis_admin`  | Transfer ownership          | `collection_id: string, new_admin: string` | `Promise<void>`                           |
| `is_collection_admin`     | Check admin status          | `collection_id: string, principal: string` | `Promise<boolean>`                        |
| `get_my_admin_level`      | Get caller's level          | `collection_id: string`                    | `Promise<"genesis" \| "admin" \| "none">` |
| `list_collection_admins`  | List all admins             | `collection_id: string`                    | `Promise<string[]>`                       |
| `get_genesis_admin`       | Get original creator        | `collection_id: string`                    | `Promise<string>`                         |

## Error Handling

All collection management functions can return these errors:

```typescript
type CollectionError = {
    NotFound: string;           // Collection not found
    NotAuthorized: string;      // Caller lacks required permissions
    InvalidInput: string;       // Invalid parameters provided
    AlreadyExists: string;      // Collection with same ID exists
    InvalidAdmin: string;       // Invalid admin principal
    NotAnAdmin: string;         // Target is not an admin
    IsGenesisAdmin: string;     // Cannot remove genesis admin
    InvalidTransfer: string;    // Invalid genesis admin transfer
};
```

### Best Practices

-  **Collection IDs**
   - Use descriptive, unique IDs (1-64 chars)
   - Only alphanumeric, underscore, and hyphen allowed
   - Avoid reserved prefixes (`__`, `admin`, `system`)

<!-- 2. **Chunk Settings**
   - Small (100-200): Good for short texts
   - Medium (200-500): Balanced for most content
   - Large (500-1000): Good for long documents -->

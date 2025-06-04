# Collection Management API

Collection management functions allow you to create, update, and manage vector collections with their associated permissions and settings.

## Create Collection

Creates a new vector collection with admin permissions and configurable settings.

```typescript
create_collection(config: {
    name: string;
    description: string;
    embedding_model: "openai-ada-002" | "openai-3-small" | "openai-3-large";
    chunk_size: "small" | "medium" | "large";
    chunk_overlap: number;
}): Promise<{
    id: string;
    name: string;
    description: string;
    embedding_model: string;
    chunk_size: string;
    chunk_overlap: number;
    created_at: bigint;
    genesis_admin: string;
}>;
```

**Example:**
```typescript
const collection = await actor.create_collection({
    name: "my_collection",
    description: "Collection for product descriptions",
    embedding_model: "openai-ada-002",
    chunk_size: "medium",
    chunk_overlap: 100
});
```

## Get Collection

Retrieves collection metadata by ID.

```typescript
get_collection(collection_id: string): Promise<{
    id: string;
    name: string;
    description: string;
    embedding_model: string;
    chunk_size: string;
    chunk_overlap: number;
    created_at: bigint;
    genesis_admin: string;
}>;
```

## List Collections

Returns all collections without computed statistics.

```typescript
list_collections(): Promise<Array<{
    id: string;
    name: string;
    description: string;
    embedding_model: string;
    chunk_size: string;
    chunk_overlap: number;
    created_at: bigint;
    genesis_admin: string;
}>>;
```

## Get Collection with Stats

Returns collection with real-time document/vector counts.

```typescript
get_collection_with_stats(collection_id: string): Promise<{
    collection: {
        id: string;
        name: string;
        description: string;
        embedding_model: string;
        chunk_size: string;
        chunk_overlap: number;
        created_at: bigint;
        genesis_admin: string;
    };
    stats: {
        document_count: number;
        vector_count: number;
        chunk_count: number;
        last_updated: bigint;
    };
}>;
```

## List Collections with Stats

Returns all collections with computed statistics.

```typescript
list_collections_with_stats(): Promise<Array<{
    collection: {
        id: string;
        name: string;
        description: string;
        embedding_model: string;
        chunk_size: string;
        chunk_overlap: number;
        created_at: bigint;
        genesis_admin: string;
    };
    stats: {
        document_count: number;
        vector_count: number;
        chunk_count: number;
        last_updated: bigint;
    };
}>>;
```

## Update Collection Settings

Modifies embedding model, chunk size, and other collection parameters.

```typescript
update_collection_settings(
    collection_id: string,
    settings: {
        embedding_model?: "openai-ada-002" | "openai-3-small" | "openai-3-large";
        chunk_size?: "small" | "medium" | "large";
        chunk_overlap?: number;
    }
): Promise<void>;
```

**Example:**
```typescript
await actor.update_collection_settings(collection.id, {
    chunk_size: "large",
    chunk_overlap: 200
});
```

## Update Collection Metadata

Changes collection name and description.

```typescript
update_collection_metadata(
    collection_id: string,
    metadata: {
        name?: string;
        description?: string;
    }
): Promise<void>;
```

## Delete Collection

Removes collection and all associated data. Only the genesis admin can delete a collection.

```typescript
delete_collection(collection_id: string): Promise<void>;
```

## Admin Management

### Add Collection Admin

Grants admin permissions to a user. Only the genesis admin can add other admins.

```typescript
add_collection_admin(
    collection_id: string,
    principal: string
): Promise<void>;
```

### Remove Collection Admin

Revokes admin permissions from a user. Only the genesis admin can remove admins.

```typescript
remove_collection_admin(
    collection_id: string,
    principal: string
): Promise<void>;
```

### Transfer Genesis Admin

Transfers ownership to another existing admin.

```typescript
transfer_genesis_admin(
    collection_id: string,
    new_admin: string
): Promise<void>;
```

### Check Admin Status

```typescript
// Check if user has admin access
is_collection_admin(
    collection_id: string,
    principal: string
): Promise<boolean>;

// Get caller's permission level
get_my_admin_level(collection_id: string): Promise<"genesis" | "admin" | "none">;

// List all collection administrators
list_collection_admins(collection_id: string): Promise<Array<string>>;

// Get the original collection creator/owner
get_genesis_admin(collection_id: string): Promise<string>;
```

## Error Handling

All collection management functions can return the following errors:

```typescript
type CollectionError = {
    NotFound: string;           // Collection not found
    NotAuthorized: string;      // Caller lacks required permissions
    InvalidInput: string;       // Invalid parameters provided
    AlreadyExists: string;      // Collection with same name exists
    InvalidAdmin: string;       // Invalid admin principal
    NotAnAdmin: string;         // Target is not an admin
    IsGenesisAdmin: string;     // Cannot remove genesis admin
    InvalidTransfer: string;    // Invalid genesis admin transfer
};
```

## Best Practices

1. **Collection Naming**
   - Use descriptive names
   - Avoid special characters
   - Keep names unique within your application

2. **Chunk Size Selection**
   - Small: 100-200 characters (good for short texts)
   - Medium: 200-500 characters (balanced)
   - Large: 500-1000 characters (good for long documents)

3. **Admin Management**
   - Keep genesis admin secure
   - Regularly audit admin list
   - Use principal IDs for admin management

4. **Settings Updates**
   - Test new settings on a small collection first
   - Consider impact on existing embeddings
   - Update settings during low-usage periods 
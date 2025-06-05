# Collection Management

Collection management functions handle the lifecycle of collections, including creation, configuration, and access control.

## Create Collection

Creates a new collection with specified settings.

```typescript
create_collection(params: {
    id: string;
    name: string;
    description?: string;
    settings?: CollectionSettings;
}): Promise<Collection>;
```

**Example:**
```typescript
const collection = await actor.create_collection({
    id: "research_papers",
    name: "Research Papers",
    description: "Collection of academic research papers",
    settings: {
        embedding_model: "text-embedding-3-small",
        chunk_size: 1000,
        chunk_overlap: 200,
        auto_embed: true
    }
});
```

## Collection Management Methods

### Get Collection

Retrieves collection metadata and settings.

```typescript
get_collection(collection_id: string): Promise<Collection>;
```

### List Collections

Returns all collections with their metadata.

```typescript
list_collections(): Promise<Array<Collection>>;
```

### Get Collection with Stats

Returns collection metadata along with usage statistics.

```typescript
get_collection_with_stats(collection_id: string): Promise<CollectionWithStats>;
```

### List Collections with Stats

Returns all collections with their usage statistics.

```typescript
list_collections_with_stats(): Promise<Array<CollectionWithStats>>;
```

### Update Collection

Updates collection metadata and settings.

```typescript
update_collection_metadata(
    collection_id: string,
    name?: string,
    description?: string
): Promise<void>;

update_collection_settings(
    collection_id: string,
    settings: CollectionSettings
): Promise<void>;
```

### Delete Collection

Removes a collection and all its documents and vectors.

```typescript
delete_collection(collection_id: string): Promise<void>;
```

## Access Control Methods

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
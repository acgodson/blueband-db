# Blueband DB

An on-chain vector database built on the ICP for semantic search, similarity matching, and document retrieval

- **`Semantic Document Processing`**: Intelligent text chunking with configurable overlap and embedding generation
- **`Hierarchical Vector Indexing`**: K-means clustering for sub-linear search performance on large datasets
- **`Embedding Support`**: OpenAI embeddings (Ada-002, 3-small, 3-large)
- **`Collection-Based Architecture`**: Isolated namespaces with granular access controls and admin hierarchies
- **`Adaptive Search Algorithm`**: Automatic switching between exact and approximate similarity search based on dataset size
- **`Memory-Optimized Caching`**: LRU cache with bounded memory



## Installation

### Add to your dfx.json (Recommended)

Add Blueband to your project's `dfx.json` like Internet Identity:

```json
{
  "canisters": {
    "blueband_rust": {
      "type": "custom",
      "candid": "https://github.com/acgodson/blueband-db/releases/latest/download/blueband_rust.did",
      "wasm": "https://github.com/acgodson/blueband-db/releases/latest/download/blueband_rust.wasm.gz"
    },
    "frontend": {
      "source": ["src/frontend/dist"],
      "type": "assets",
      "dependencies": ["blueband_rust"]
    }
  }
}
```

Then deploy:

```bash
dfx start --background
dfx canister create blueband_rust
dfx canister install blueband_rust
```

### Manual Installation

```bash
# Download and deploy directly
wget https://github.com/acgodson/blueband-db/releases/download/v0.1.9/blueband_rust.wasm.gz
gunzip blueband_rust.wasm.gz
dfx canister install --mode install --wasm blueband_rust.wasm <canister-id>
```


### Usage

```typescript

// Example: Simple semantic search

const docs = [
  "Pizza is a delicious Italian food with cheese and tomatoes",
  "Soccer is the most popular sport in the world", 
  "JavaScript is a programming language for web development",
];

const query = "Which sport is most popular?";

const results = await actor.demo_vector_similarity(
  docs,
  query,
  "<openai-embedding-proxy-url>",
  [1], // Only return top result
  []
);

console.log(results);
// Returns: [{ score: 0.91, text: "Soccer is the most popular sport in the world", ... }]
```

## Operations

### Collection Management
```typescript
// Create isolated space
await actor.create_collection(config);

// Manage access controls
await actor.add_collection_admin(collection_id, principal);
await actor.transfer_genesis_admin(collection_id, new_admin);
```

### Document Operations
```typescript
// Store document only
await actor.add_document(documentRequest);

// Add with automatic embedding
await actor.add_document_and_embed(documentRequest, proxy_url);

// Retrieve with metadata
const doc = await actor.get_document(collection_id, document_id);
```

### Vector Search
```typescript
// Semantic similarity search
const matches = await actor.search(query_config);

// Document-to-document similarity  
const similar = await actor.find_similar_documents(source_id, collection_id, config);
```

## Use Cases

- **`Enterprise Search`**: Intelligent document retrieval across knowledge bases
- **`Recommendation Systems`**: Content similarity for personalized suggestions  
- **`Research Platforms`**: Semantic paper discovery and citation analysis


## Documentation

- [API Reference](docs/api.md) - Operations documentation



## License

-  [MIT](LICENSE)


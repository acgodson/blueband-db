# Blueband DB

A vector database for the Internet Computer Protocol (ICP) that enables semantic search and similarity operations.

- Document storage with automatic chunking
- Vector embeddings using OpenAI models
- Semantic search across documents
- Collection-based organization
- Admin-based access control

## Quick Start

1. Add to your `dfx.json`:
```json
{
  "canisters": {
    "blueband_rust": {
      "type": "custom",
      "candid": "https://github.com/acgodson/blueband-db/releases/latest/download/blueband_rust.did",
      "wasm": "https://github.com/acgodson/blueband-db/releases/latest/download/blueband_rust.wasm.gz"
    }
  }
}
```

2. Deploy:
```bash
dfx start --background
dfx canister create blueband_rust
dfx canister install blueband_rust
```

3. Use in your frontend:
```typescript
import { Actor, HttpAgent } from "@dfinity/agent";
import { idlFactory } from "./declarations/blueband_rust.did.js";
```

## Key Features

| Category | Features |
|----------|----------|
| **Document Management** | • Add, update, and delete documents<br>• Automatic text chunking<br>• Metadata support<br>• Batch operations |
| **Vector Search** | • Text-based semantic search<br>• Vector similarity search<br>• Document similarity search<br>• Metadata filtering |
| **Collection Management** | • Create and manage collections<br>• Admin-based access control<br>• Configurable chunking<br>• Multiple embedding models |

## Documentation

- [API Reference](api/collection-management.md)
- [Examples](examples/basic-usage.md)

## Support

- [GitHub Issues](https://github.com/acgodson/blueband-db/issues)
- [Discord Community](https://discord.gg/your-discord) 
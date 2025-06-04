# Blueband DB

A vectra-inspired vector db on ICP. Blueband handles the complete document lifecycle - from text chunking and embedding to vector storage and similarity search - while leveraging ICP's stable memory for persistence. Its adaptive search algorithms and collection-based design make it particularly effective for applications requiring semantic understanding and scalable vector operations.


## 🚀 Quick Start

```typescript
import { Actor, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import { Ed25519KeyIdentity } from "@dfinity/identity";
import { idlFactory } from "./declarations/blueband_rust.did.js";


const identity = Ed25519KeyIdentity.generate();
const agent = new HttpAgent({
  host: "http://127.0.0.1:4943", 
  identity,
});

const actor = Actor.createActor(idlFactory, {
  agent,
  canisterId: Principal.fromText("canister-id"),
});

// Example: Simple semantic search
const docs = [
  "Pizza is a delicious Italian food with cheese and tomatoes",
  "Soccer is the most popular sport in the world",
  "JavaScript is a programming language for web development",
];

const query =  "Which sport is most popular?";

// Search with 91%+ accuracy
const results = await actor.demo_vector_similarity(
  docs,
  query,
  "openai-embedding-proxy-url",
  [1], // Return top result
  []
);

// Results:
// ✅ [91.1%] Soccer is the most popular sport in the world
```

## 🔧 Installing pre-built cannister

```bash
wget http://github.com/acgodson/blueband/
```

## 📚 Documentation

For detailed API documentation and advanced usage, see our [technical documentation](docs.md).

## 🤝 Contributing

We welcome contributions! Please see our contributing guidelines for more details.

## 📄 License

MIT License - see LICENSE file for details

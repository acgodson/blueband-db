# 🌀 Blueband: Vector db on ICP

<div class="hero">
  <p class="description">
    Store collections of documents in semantic chunks and adaptive vector indexing for efficient similarity search and retrieval.
  </p>
</div>

<div class="features">
  <div class="feature">
    <h2>📚 Document Management</h2>
    <ul>
      <li>Add, update, and delete documents</li>
      <li>Automatic text chunking</li>
      <li>Metadata support</li>
      <li>Batch operations</li>
    </ul>
  </div>

  <div class="feature">
    <h2>🔍 Vector Search</h2>
    <ul>
      <li>Text-based semantic search</li>
      <li>Vector similarity search</li>
      <li>Document similarity search</li>
      <li>Metadata filtering</li>
    </ul>
  </div>

  <div class="feature">
    <h2>⚙️ Collection Management</h2>
    <ul>
      <li>Create and manage collections</li>
      <li>Admin-based access control</li>
      <li>Configurable chunking</li>
      <li>Multiple embedding models</li>
      <li>Type-safe canister interface</li>
    </ul>
  </div>
</div>

## Quick Start

<div class="custom-block tip">
  <p class="custom-block-title">Getting Started</p>
  <p>Follow these simple steps to integrate Blueband into your ICP project.</p>
</div>

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
dfx deploy
```

3. Use in your frontend:
```typescript
import { Actor, HttpAgent } from "@dfinity/agent";
import { idlFactory } from "./declarations/blueband_rust.did.js";
```

## Documentation

<div class="custom-block links">
  <a href="#/api/collection-management" class="nav-link">
    <span class="icon">📖</span>
    <span class="text">Backend Canister Reference</span>
  </a>
</div>

## Support

<div class="custom-block links">
  <a href="https://github.com/acgodson/blueband-db/issues" class="nav-link" target="_blank">
    <span class="icon">💬</span>
    <span class="text">GitHub Issues</span>
  </a>
</div>

<style>
.hero {
  text-align: center;
  margin: 2rem 0 4rem;
}

.description {
  font-size: 1.4rem;
  color: #2c3e50;
  max-width: 800px;
  margin: 0 auto;
  line-height: 1.6;
}

.features {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 2rem;
  margin: 3rem 0;
}

.feature {
  background: #f8f9fa;
  border-radius: 12px;
  padding: 1.5rem;
  transition: transform 0.3s ease;
  border: 1px solid #e9ecef;
}

.feature:hover {
  transform: translateY(-5px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.feature h2 {
  color: #42b983;
  margin-top: 0;
  font-size: 1.4rem;
  border-bottom: 2px solid #42b983;
  padding-bottom: 0.5rem;
}

.feature ul {
  list-style: none;
  padding-left: 0;
  margin: 1rem 0 0;
}

.feature li {
  margin: 0.8rem 0;
  padding-left: 1.5rem;
  position: relative;
}

.feature li::before {
  content: "→";
  position: absolute;
  left: 0;
  color: #42b983;
}

.custom-block {
  margin: 2rem 0;
  padding: 1.5rem;
  border-radius: 8px;
}

.custom-block.tip {
  background: #e6f7ff;
  border-left: 4px solid #1890ff;
}

.custom-block-title {
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: #1890ff;
}

.custom-block.links {
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
}

.nav-link {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1.5rem;
  background: #f8f9fa;
  border: 1px solid #e9ecef;
  border-radius: 8px;
  color: #2c3e50;
  text-decoration: none;
  transition: all 0.3s ease;
}

.nav-link:hover {
  background: #42b983;
  color: white;
  border-color: #42b983;
  transform: translateY(-2px);
}

.nav-link .icon {
  font-size: 1.2rem;
}

@media (max-width: 768px) {
  .description {
    font-size: 1.2rem;
    padding: 0 1rem;
  }
  
  .features {
    grid-template-columns: 1fr;
  }
  
  .custom-block.links {
    flex-direction: column;
  }
  
  .nav-link {
    width: 100%;
    justify-content: center;
  }
}
</style>

# Blueband db

Blueband is a vector database on ICP, building from the core of [Vectra](https://github.com/Stevenic/vectra/tree/main) and its local db principles.

Blueband persists data like a traditional db and saves the embeddings on ICP's stable memory. (Unlike Vectra, which saves to local disk)

This setup can be ideal for use cases involving small, mostly static datasets where quick quick comparison is needed.

- `Loading Data into Memory`: The index, which contains metadata and vectors,is loaded from a persistent storage (a collection's canister) into the system’s memory

- `Querying`: Once in memory (initialized), the index can be queried to calaculate and rank the scores between saved vectors, and external prompts.



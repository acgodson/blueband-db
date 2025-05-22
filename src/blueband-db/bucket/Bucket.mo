//bucket/Bucket.mo
import Cycles "mo:base/ExperimentalCycles";
import Debug "mo:base/Debug";
import Nat "mo:base/Nat";
import Nat64 "mo:base/Nat64";
import Prim "mo:prim";
import Array "mo:base/Array";
import Buffer "mo:base/Buffer";

import Map "mo:map/Map";
import { thash } "mo:map/Map";
import Types "../Types";

shared ({ caller }) actor class Bucket() {
    type DocumentId = Types.DocumentId;
    type ChunkId = Types.ChunkId;
    type VectorId = Types.VectorId;
    type DocumentMetadata = Types.DocumentMetadata;
    type SemanticChunk = Types.SemanticChunk;
    type Vector = Types.Vector;
    type BucketState = Types.BucketState;

    // Constants
    let MAX_CYCLES = 100_000_000_000_000; // Cap for cycle balance

    // State
    stable var state = Types.emptyBucket();

    // Canister size for monitoring
    public query func getSize() : async Nat {
        Debug.print("canister balance: " # Nat.toText(Cycles.balance()));
        Prim.rts_memory_size();
    };

    // Document operations
    public shared func insertDocument(metadata : DocumentMetadata) : async ?DocumentId {
        switch (Map.get(state.documents, thash, metadata.id)) {
            case (?_) { return null }; // Document ID already exists
            case null {
                ignore Map.put(state.documents, thash, metadata.id, metadata);
                ?metadata.id;
            };
        };
    };

    public shared func insertChunk(documentId : DocumentId, chunkId : ChunkId, chunk : SemanticChunk) : async ?() {
        // Verify document exists
        switch (Map.get(state.documents, thash, documentId)) {
            case null { return null };
            case (?_) {
                // Store chunk
                ignore Map.put(state.chunks, thash, chunkId, chunk);
                // Store chunk to document mapping
                ignore Map.put(state.chunk_to_document, thash, chunkId, documentId);
                ?();
            };
        };
    };

    public shared func insertVector(vector : Vector) : async ?() {
        // Verify document exists
        switch (Map.get(state.documents, thash, vector.document_id)) {
            case null { return null };
            case (?_) {
                // Store vector
                ignore Map.put(state.vectors, thash, vector.id, vector);
                // Store vector to document mapping
                ignore Map.put(state.vector_to_document, thash, vector.id, vector.document_id);
                ?();
            };
        };
    };

    public shared func updateDocumentStatus(documentId : DocumentId, isEmbedded : Bool) : async ?() {
        switch (Map.get(state.documents, thash, documentId)) {
            case null { return null };
            case (?doc) {
                let updatedDoc = {
                    id = doc.id;
                    title = doc.title;
                    content_type = doc.content_type;
                    source_url = doc.source_url;
                    timestamp = doc.timestamp;
                    total_chunks = doc.total_chunks;
                    size = doc.size;
                    is_embedded = isEmbedded;
                };
                ignore Map.put(state.documents, thash, documentId, updatedDoc);
                ?();
            };
        };
    };

    // Query operations
    public query func getDocument(documentId : DocumentId) : async ?DocumentMetadata {
        Map.get(state.documents, thash, documentId);
    };

    public query func getChunk(chunkId : ChunkId) : async ?SemanticChunk {
        Map.get(state.chunks, thash, chunkId);
    };

    public query func getDocumentChunks(documentId : DocumentId) : async [SemanticChunk] {
        let results = Buffer.Buffer<SemanticChunk>(0);

        for ((id, chunk) in Map.entries(state.chunks)) {
            switch (Map.get(state.chunk_to_document, thash, id)) {
                case (?docId) {
                    if (docId == documentId) {
                        results.add(chunk);
                    };
                };
                case null {};
            };
        };

        // Sort by position before returning
        let sorted = Array.sort<SemanticChunk>(
            Buffer.toArray(results),
            func(a, b) { Nat.compare(a.position, b.position) },
        );

        sorted;
    };

    // public query func getVectors(
    //     filter : ?{
    //         document_id : ?DocumentId;
    //         chunk_id : ?ChunkId;
    //         limit : ?Nat;
    //     }
    // ) : async [Vector] {
    //     let results = Buffer.Buffer<Vector>(0);
    //     let limit_value = switch (filter) {
    //         case (?{ limit = ?l }) { l };
    //         case _ { 1000 }; // Default limit
    //     };

    //     label filtering for ((id, vector) in Map.entries(state.vectors)) {
    //         // Apply document filter if provided
    //         switch (filter) {
    //             case (?{ document_id = ?doc_id }) {
    //                 if (vector.document_id != doc_id) continue filtering;
    //             };
    //             case _ {};
    //         };

    //         // Apply chunk filter if provided
    //         switch (filter) {
    //             case (?{ chunk_id = ?chunk_id }) {
    //                 if (vector.chunk_id != chunk_id) continue filtering;
    //             };
    //             case _ {};
    //         };

    //         results.add(vector);

    //         // Apply limit
    //         if (results.size() >= limit_value) {
    //             break filtering;
    //         };
    //     };

    //     Buffer.toArray(results);
    // };

    public shared func getVectors(
        filter : ?{
            document_id : ?DocumentId;
            chunk_id : ?ChunkId;
            limit : ?Nat;
        }
    ) : async [Vector] {
        let results = Buffer.Buffer<Vector>(0);
        let limit_value = switch (filter) {
            case (?{ limit = ?l }) { l };
            case _ { 1000 }; // Default limit
        };

        label filtering for ((id, vector) in Map.entries(state.vectors)) {
            // Apply document filter if provided
            switch (filter) {
                case (?{ document_id = ?doc_id }) {
                    if (vector.document_id != doc_id) continue filtering;
                };
                case _ {};
            };

            // Apply chunk filter if provided
            switch (filter) {
                case (?{ chunk_id = ?chunk_id }) {
                    if (vector.chunk_id != chunk_id) continue filtering;
                };
                case _ {};
            };

            results.add(vector);

            // Apply limit
            if (results.size() >= limit_value) {
                break filtering;
            };
        };

        Buffer.toArray(results);
    };

    public query func listDocuments() : async [DocumentMetadata] {
        // Fix: Map.toArray expects 2 type arguments, not 3
        let docs = Buffer.Buffer<DocumentMetadata>(0);
        for ((_, doc) in Map.entries(state.documents)) {
            docs.add(doc);
        };
        Buffer.toArray(docs);
    };

    public shared func deleteVector(vectorId : VectorId) : async ?() {
        switch (Map.get(state.vectors, thash, vectorId)) {
            case null { return null };
            case (?vector) {
                // Verify vector belongs to the document
                switch (Map.get(state.vector_to_document, thash, vectorId)) {
                    case null { return null };
                    case (?docId) {
                        if (docId != vector.document_id) {
                            Debug.print("Vector ownership mismatch");
                            return null;
                        };
                        Map.delete(state.vectors, thash, vectorId);
                        Map.delete(state.vector_to_document, thash, vectorId);
                        ?();
                    };
                };
            };
        };
    };

    // Cycle management
    public func wallet_receive() : async { accepted : Nat64 } {
        let available = Cycles.available();
        let acceptable = Nat.min(available, MAX_CYCLES);
        let accepted = Cycles.accept<system>(acceptable);
        { accepted = Nat64.fromNat(accepted) };
    };
};

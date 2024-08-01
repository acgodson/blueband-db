import Cycles "mo:base/ExperimentalCycles";

import Nat "mo:base/Nat";
import Debug "mo:base/Debug";
import Blob "mo:base/Blob";
import Text "mo:base/Text";
import Nat64 "mo:base/Nat64";
import Buffer "mo:base/Buffer";
import Prim "mo:prim";

import Map "mo:map/Map";
import { thash; nhash } "mo:map/Map";
import BucketTypes "./Types";
import Utils "../Utils";

shared ({ caller }) actor class Bucket() {

    type DocumentId = BucketTypes.DocumentId;
    type VectorId = BucketTypes.VectorId;
    type ChunkId = BucketTypes.ChunkId;
    type State = BucketTypes.CanisterState;

    type DocumentMetadata = BucketTypes.DocumentMetadata;
    type Vector = BucketTypes.Vector;

    // test constants
    let MAX_DOCUMENT_SIZE = 20_000_000_000_000;

    stable var state = BucketTypes.empty();
    stable var chunkCounter : Nat = 0;
    private var _vectorIdToDocId : Map.Map<Text, Text> = Map.new<Text, Text>();

    //current size of cannister
    public func getSize() : async Nat {
        Debug.print("canister balance: " # Nat.toText(Cycles.balance()));
        Prim.rts_memory_size();
    };

    //Used when adding next recipe
    public func lastCounter() : async Nat {
        chunkCounter;
    };
    // increment number
    private func inc() { chunkCounter += 1 };

    //  save Recipe info/metadata
    private func _insertMetadata(documentId : DocumentId, metadata : DocumentMetadata) : ?DocumentId {
        switch (Map.get(state.metadatas, thash, documentId)) {
            case (?_) { /* error -- ID already taken. */ return null };
            case null {
                Debug.print("new recipe id is..." # debug_show (documentId));
                let _metadata = {
                    id = documentId;
                    name = metadata.name;
                    chunkStartId = metadata.chunkStartId;
                    chunkEndId = metadata.chunkEndId;
                    chunkCount = metadata.chunkCount;
                    size = metadata.size;
                    isEmbedded = metadata.isEmbedded;
                };
                ignore Map.put(state.metadatas, thash, documentId, _metadata);
                ?documentId;
            };
        };
    };

    // update catalog: which is amapping of each new embeddings to vector-id generated from an embedding to a corresponding recipe-id
    private func startUpdate(documentId : DocumentId, vectorId : VectorId) {
        switch (Map.get(state.metadatas, thash, documentId)) {
            case null {};
            case (?_) {
                let updatedVectorIdToDocId = Map.clone(_vectorIdToDocId);
                ignore Map.put(updatedVectorIdToDocId, thash, vectorId, documentId);
                _vectorIdToDocId := updatedVectorIdToDocId;
            };
        };
    };

    // finish embedding
    public func endUpdate(documentId : DocumentId) : async () {
        switch (Map.get(state.metadatas, thash, documentId)) {
            case (?doc) {
                let metadata = {
                    id = doc.id;
                    name = doc.name;
                    chunkStartId = doc.chunkStartId;
                    chunkEndId = doc.chunkEndId;
                    chunkCount = doc.chunkCount;
                    size = doc.size;
                    isEmbedded = true;
                };
                ignore Map.put(state.metadatas, thash, doc.id, metadata);
            };
            case null {};
        };
    };

    // instantiate a new document metadata and returns an id
    public func insertMetadata(metadata : DocumentMetadata) : async ?DocumentId {
        do ? {
            let documentId = await Utils.generateRandomID(metadata.name);
            _insertMetadata(documentId, metadata)!;
        };
    };

    // generate chunkID- updating global counter
    func chunkId() : ChunkId {
        inc();
        chunkCounter;
    };

    // save recipe raw data
    public func insertChunk(documentId : DocumentId, chunkData : Blob) : async ?() {
        do ? {
            let id = chunkCounter;
            ignore Map.put(
                state.chunks,
                nhash,
                id,
                chunkData,
            );
            Debug.print("new chunk added with id" # debug_show (id) # " from recipe with id: " # debug_show (documentId) # " and " # debug_show (chunkCounter) # "  and chunk size..." # debug_show (Blob.toArray(chunkData).size()));
            ignore chunkId();
        };
    };

    // store recipe embeddings
    public func addVector(data : Vector) : async ?() {
        let existingVector = Map.get(state.vectors, thash, data.id);
        switch (existingVector) {
            case (null) {

                let vectorData = {
                    id = data.id;
                    documentId = data.documentId;
                    startPos = data.startPos;
                    endPos = data.endPos;
                    vector = data.vector;
                };
                do ? {
                    ignore Map.put(
                        state.vectors,
                        thash,
                        data.id,
                        vectorData,
                    );
                    startUpdate(data.documentId, data.id);
                };
            };
            case (?_) {
                // Vector already exists
                return null;
            };
        };
    };

    // List  infos/metadata of all recipes
    public query func listDocumentMetadata() : async [DocumentMetadata] {
        let b = Buffer.Buffer<DocumentMetadata>(0);
        let _ = do ? {
            for (
                (f, _) in Map.entries(state.metadatas)
            ) {
                b.add(_getMetadata(f)!);
            };
        };
        Buffer.toArray(b);
    };

    // return all recipes embeddings
    public query func listVectors() : async { items : [Vector] } {
        let transformedVectorDataList = Map.toArrayMap<Text, Vector, Vector>(
            state.vectors,
            func(_, d) {
                ?{
                    id = d.id;
                    documentId = d.documentId;
                    startPos = d.startPos;
                    endPos = d.endPos;
                    vector = d.vector;
                };
            },
        );
        { items = transformedVectorDataList };
    };

    func _getMetadata(documentId : DocumentId) : ?DocumentMetadata {
        do ? {
            let v = Map.get(state.metadatas, thash, documentId)!;
            {
                id = v.id;
                name = v.name;
                size = v.size;
                chunkEndId = v.chunkEndId;
                chunkStartId = v.chunkStartId;
                chunkCount = v.chunkCount;
                isEmbedded = v.isEmbedded;
            };
        };
    };

    public query func getMetadata(documentId : DocumentId) : async ?DocumentMetadata {
        do ? {
            _getMetadata(documentId)!;
        };
    };

    public query func getChunk(chunkId : Nat) : async ?Text {
        // TODO: retrieve all existing chunks of a recipe ID
        do ? {
            let blob = Map.get(state.chunks, nhash, chunkId);
            return Text.decodeUtf8(blob!);
        };
    };

    public query func vectorIdToRecipeId(vectorId : Text) : async ?Text {
        do ? {
            let v = Map.get(_vectorIdToDocId, thash, vectorId)!;
            return ?v;
        };
    };

    public func wallet_receive() : async { accepted : Nat64 } {
        let available = Cycles.available();
        let accepted = Cycles.accept<system>(Nat.min(available, MAX_DOCUMENT_SIZE));
        { accepted = Nat64.fromNat(accepted) };
    };

    public func wallet_balance() : async Nat {
        return Cycles.balance();
    };

};

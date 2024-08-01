import Principal "mo:base/Principal";
import Cycles "mo:base/ExperimentalCycles";

import Text "mo:base/Text";
import Nat "mo:base/Nat";
import Debug "mo:base/Debug";
import Error "mo:base/Error";
import Array "mo:base/Array";
import Iter "mo:base/Iter";
import Int "mo:base/Int";
import Map "mo:map/Map";
import { thash } "mo:map/Map";

import Buckets "./Bucket";
import Types "../Types";

import StorageTypes "./Types";


import { sliceText } "../Utils";

module BucketProvider {

    type CollectionId = Types.CollectionId;
    public type Collection = Types.Collection;

    type CanisterState = Map.Map<CollectionId, Collection>;

    type Bucket = Buckets.Bucket;

    type TextChunk = Types.TextChunk;

    type DocumentId = StorageTypes.DocumentId;
    type VectorId = StorageTypes.VectorId;
    type ChunkId = StorageTypes.ChunkId;

    type Vector = StorageTypes.Vector;

    public type VectorStore = Types.VectorStore;

    public type MetadataList = Types.MetadataList;

    public type DocumentMetadata = StorageTypes.DocumentMetadata;

    // TODO: allow users to set threshold and defaultCycles
    public let threshold = 2147483648; // ~2GB
    public let defaultCycleShare = 500_000_000_000;

    public func addDcoument(
        collectionId : CollectionId,
        db : CanisterState,
        title : Text,
        content : Text,
    ) : async { collection : ?Principal; documentId : ?Text } {
        //TODO: Check if splitting a recipe_raw_data into chunks for a single canister improves performance
        let chunkSize = 1024;
        let textSize = Text.size(content);

        // Ensure proper handling of Nat and Int conversion to avoid traps
        let _chunkCount = if (textSize == 0) 0 else {
            let textSizeInt = textSize;
            let chunkSizeInt = chunkSize;
            let sum = Nat.add(textSizeInt, chunkSizeInt);
            let chunkCountInt = Nat.div(Nat.sub(sum, 1), chunkSizeInt);
            chunkCountInt;
        };

        // retrieve existing bucket or create new bucket
        let bucket = await getBucket(collectionId, db);

        // increment the chunkCounter in a loop of the total chunkcount
        let lastCounter = await bucket.lastCounter();
        let _chunkStartId = lastCounter;
        let _chunkEndId = Nat.sub(Nat.add(_chunkStartId, _chunkCount), 1);
        // FileInfo
        let _metadata : DocumentMetadata = {
            id = "";
            name = title;
            chunkStartId = _chunkStartId;
            chunkEndId = _chunkEndId;
            chunkCount = _chunkCount;
            size = textSize;
            isEmbedded = false;
        };

        //store recipe metadata
        let documentResult = await bucket.insertMetadata(_metadata);
        switch (documentResult) {
            case (null) {
                throw Error.reject("Failed to store recipe metadata");
            };
            case (?docId) {
                var currentChunkId = _chunkStartId;
                while (Nat.lessOrEqual(currentChunkId, _chunkEndId)) {
                    let start = Nat.mul(Nat.sub(currentChunkId, _chunkStartId), chunkSize);
                    let end = Nat.min(Nat.add(start, chunkSize), textSize);
                    let chunkData = sliceText(content, start, end);
                    let chunkBlob = Text.encodeUtf8(chunkData);
                    let _ = await bucket.insertChunk(docId, chunkBlob);
                    currentChunkId := Nat.add(currentChunkId, 1);
                };
                let canisterID = Principal.fromActor(bucket);
                return { collection = ?canisterID; documentId = ?docId };
            };
        };
    };

    //TODO: remeber to add norm later
    public func addVector(
        collectionId : CollectionId,
        db : CanisterState,
        documentId : Text,
        vectorId : Text,
        start : Nat,
        end : Nat,
        vector : [Float],
    ) : async Text {

        let vectorData = {
            id = vectorId;
            documentId = documentId;
            startPos = start;
            endPos = end;
            vector = vector;
        };
        let bucket : Bucket = await getBucket(collectionId, db);
        //TODO: verify that document metadata still exisits
        let _ = await bucket.addVector(vectorData);
        vectorId;
    };

    public func endVectorUpdate(
        collectionId : CollectionId,
        db : CanisterState,
        documentId : Text,
    ) : async () {
        let maybeBucket = await getBucket(collectionId, db);
        switch (maybeBucket) {
            case (bucket) {
                let maybeMetadata = await bucket.getMetadata(documentId);
                switch (maybeMetadata) {
                    case (?m) {
                        if (m.isEmbedded) {
                            throw Error.reject("Update already ended");
                        } else {
                            let result = await bucket.endUpdate(documentId);
                            result;
                        };
                    };
                    case (null) {
                        throw Error.reject("Recipe does not exist, may have been deleted");
                    };

                };

                let result = await bucket.endUpdate(documentId);
                result;
            };
        };

    };

    public func listVectors(
        collectionId : CollectionId,
        db : CanisterState,
    ) : async ?{
        items : [Vector];
    } {
        let maybeBucket = await getBucket(collectionId, db);
        switch (maybeBucket) {
            case (bucket) {
                let vectorList = await bucket.listVectors();
                return ?vectorList;
            };
        };
    };

    //Retrieve Complete raw Recipe
    //WARNING: Implement Pagination
    public func getChunks(
        db : CanisterState,
        collectionId : CollectionId,
        documentId : DocumentId,
    ) : async ?Text {

        let result = await listDocumentMetadata(
            db,
            collectionId,
        );
        switch (result) {
            case (?metadataList) {
                let filter = Array.find(
                    metadataList,
                    func(info : DocumentMetadata) : Bool {
                        info.id == documentId;
                    },
                );
                switch (filter) {
                    case (?metadata) {
                        var text = "";
                        var _start = Int.abs(metadata.chunkStartId);
                        var end = Int.abs(metadata.chunkEndId);

                        for (chunkNum in Iter.range(_start, end)) {
                            let chunk = await getChunk(db, collectionId, chunkNum);
                            switch (chunk) {
                                case (?chunkText) {
                                    text := text # chunkText;
                                };
                                case null {};
                            };
                        };
                        return ?text;
                    };
                    case null { return null };
                };
            };
            case null { return null };
        };
    };

    public func getChunk(
        db : CanisterState,
        collectionId : CollectionId,
        chunkId : Nat,
    ) : async ?Text {
        let maybeBucket = await getBucket(collectionId, db);
        switch (maybeBucket) {
            case (bucket) {
                return await bucket.getChunk(chunkId);
            };
        };
    };

    // returns collection Principal
    public func getBucketPrincipal(collectionId : CollectionId, db : CanisterState) : async ?Principal {
        let bucket : Bucket = await getBucket(collectionId, db);
        ?Principal.fromActor(bucket);
    };

    public func getBucket(collectionId : CollectionId, db : CanisterState) : async Bucket {
        let existingBucket = Map.get(db, thash, collectionId);
        switch (existingBucket) {
            case (?v) { v.bucket };
            case (null) {
                /* bucket not found */
                Cycles.add<system>(defaultCycleShare);
                let newBucket = await Buckets.Bucket();
                let size = await newBucket.getSize();

                Debug.print("new collection principal is " # debug_show (Principal.toText(Principal.fromActor(newBucket))));
                Debug.print("the initial size is " # debug_show (size));
                var v : Collection = {
                    bucket = newBucket;
                    var size = size;
                    var cycle_balance = defaultCycleShare;
                };
                ignore Map.put(db, thash, collectionId, v);
                newBucket;
            };
        };
    };

    public func listDocumentMetadata(
        db : CanisterState,
        collectionId : CollectionId,
    ) : async ?[DocumentMetadata] {
        let maybeBucket = await getBucket(collectionId, db);
        switch (maybeBucket) {
            case (bucket) {
                let metadataList = await bucket.listDocumentMetadata();
                return ?metadataList;
            };
        };
    };

    public func getMetadata(
        db : CanisterState,
        collectionId : CollectionId,
        documentId : DocumentId,
    ) : async ?DocumentMetadata {
        let maybeBucket = await getBucket(collectionId, db);
        switch (maybeBucket) {
            case (bucket) {
                let fileInfoList = await bucket.getMetadata(documentId);
                return fileInfoList;
            };
        };
    };

    // get document Id of a particular vector
    public func getDocumentIdByVectorId(db : CanisterState, collectionId : CollectionId, vectorId : VectorId) : async ?Text {
        let maybeBucket = await getBucket(collectionId, db);
        switch (maybeBucket) {
            case (bucket) {
                return await bucket.vectorIdToRecipeId(vectorId);
            };
        };
    };

    // this assumes title is unique - TODO: return array of matching ids instead just a single
    public func titleToDocumentID(db : CanisterState, collectionId : CollectionId, title : Text) : async ?Text {
        let maybeBucket = await getBucket(collectionId, db);
        switch (maybeBucket) {
            case (bucket) {
                var id : ?Text = null;
                let metadataList = await bucket.listDocumentMetadata();
                for (metadata in metadataList.vals()) {
                    if (metadata.name == title) {
                        id := ?metadata.id;
                    };
                };
                id;
            };
        };
    };

    public func documentIdToTitle(db : CanisterState, collectionId : CollectionId, documentId : DocumentId) : async ?Text {
        let maybeBucket = await getBucket(collectionId, db);
        switch (maybeBucket) {
            case (bucket) {
                var id : ?Text = null;
                let metadataList = await bucket.listDocumentMetadata();
                for (metadata in metadataList.vals()) {
                    if (metadata.id == documentId) {
                        return ?metadata.name;
                    };
                };
                id;
            };
        };

    };
};

//bucket/bucketprovider.mo
import Principal "mo:base/Principal";
import Cycles "mo:base/ExperimentalCycles";
import Text "mo:base/Text";
import Nat "mo:base/Nat";
import Nat64 "mo:base/Nat64";
import Debug "mo:base/Debug";
import Error "mo:base/Error";
import Time "mo:base/Time";
import Buffer "mo:base/Buffer";

import Map "mo:map/Map";
import { thash; phash } "mo:map/Map";

import Bucket "./Bucket";
import Types "../Types";
import Utils "../Utils";

module BucketProvider {
    type CollectionId = Types.CollectionId;
    type DocumentId = Types.DocumentId;
    type ChunkId = Types.ChunkId;
    type VectorId = Types.VectorId;
    type Role = Types.Role;

    type Collection = Types.Collection;
    type SemanticChunk = Types.SemanticChunk;
    type DocumentMetadata = Types.DocumentMetadata;
    type Vector = Types.Vector;

    type Database = Types.Database;

    // Define the expected bucket interface
    public type BucketActor = actor {
        // Core CRUD operations
        insertDocument : shared (metadata : DocumentMetadata) -> async ?DocumentId;
        insertChunk : shared (documentId : DocumentId, chunkId : ChunkId, chunk : SemanticChunk) -> async ?();
        insertVector : shared (vector : Vector) -> async ?();
        updateDocumentStatus : shared (documentId : DocumentId, isEmbedded : Bool) -> async ?();
        deleteVector : shared (vectorId : VectorId) -> async ?();

        // Query operations
        getDocument : shared (documentId : DocumentId) -> async ?DocumentMetadata;
        getChunk : shared (chunkId : ChunkId) -> async ?SemanticChunk;
        getDocumentChunks : shared (documentId : DocumentId) -> async [SemanticChunk];
        getVectors : shared (
            filter : ?{
                document_id : ?DocumentId;
                chunk_id : ?ChunkId;
                limit : ?Nat;
            }
        ) -> async [Vector];

        listDocuments : shared () -> async [DocumentMetadata];

        // Utility operations
        getSize : shared () -> async Nat;
        wallet_receive : shared () -> async { accepted : Nat64 };
    };

    // Constants
    public let DEFAULT_CYCLES = 100_000_000_000; // 100B cycles per bucket
    public let MAX_DOCUMENT_SIZE = 20_000_000; // Characters
    public let RECOMMENDED_CHUNK_SIZE = 500; // Characters per semantic chunk
    public let DEFAULT_EXPIRY_TIME = 604_800_000_000_000; // 7 days in nanoseconds (pre-computed)

    // Admin validation functions
    public func isAdmin(principal : Principal, collection : Collection) : Bool {
        switch (Map.get(collection.admins, phash, principal)) {
            case (?_) { true };
            case null { false };
        };
    };

    public func isGenesisAdmin(principal : Principal, collection : Collection) : Bool {
        switch (Map.get(collection.admins, phash, principal)) {
            case (?#GenesisAdmin) { true };
            case (_) { false };
        };
    };

    // Create a new collection with admin and token configuration
    public func createCollection(
        collectionId : CollectionId,
        creator : Principal,
        token_canister : ?Principal,
        voting_threshold : ?Nat,
        quorum_threshold : ?Nat,
        expiry_time : ?Nat,
        db : Map.Map<CollectionId, Collection>,
    ) : async ?Principal {
        // Check if collection already exists
        switch (Map.get(db, thash, collectionId)) {
            case (?_) {
                return null; // Collection already exists
            };
            case null {
                // Create new bucket with proper cycle transfer
                Debug.print("Current canister balance: " # Nat.toText(Cycles.balance()));
                
                // Add cycles for bucket creation
                Cycles.add<system>(DEFAULT_CYCLES);
                
                // Create the bucket first
                let newBucket = await Bucket.Bucket();
                
                // Now transfer cycles to the new bucket
                let accepted = await newBucket.wallet_receive();
                Debug.print("Transferred " # Nat64.toText(accepted.accepted) # " cycles to new bucket");
                
                let size = await newBucket.getSize();
                Debug.print("New collection created: " # collectionId);
                Debug.print("Initial size: " # Nat.toText(size));
                Debug.print("New canister balance: " # Nat.toText(Cycles.balance()));

                // Handle the type mismatch using a type cast
                let castedBucket : BucketActor = actor (Principal.toText(Principal.fromActor(newBucket)));

                // Setup admin roles
                let admins = Map.new<Principal, Types.Role>();
                ignore Map.put(admins, phash, creator, #GenesisAdmin);

                // Set default values if not provided
                let default_expiry = 7 * 24 * 60 * 60 * 1_000_000_000; // 7 days
                let default_voting_threshold = 51; // 51%
                let default_quorum_threshold = 10; // 10%

                let actual_expiry = switch (expiry_time) {
                    case (?exp) { exp };
                    case null { default_expiry };
                };

                let actual_voting_threshold = switch (voting_threshold) {
                    case (?vt) { vt };
                    case null { default_voting_threshold };
                };

                let actual_quorum_threshold = switch (quorum_threshold) {
                    case (?qt) { qt };
                    case null { default_quorum_threshold };
                };

                // Create collection with admin and governance configuration
                let collection : Collection = {
                    bucket = castedBucket;
                    var size = size;
                    var cycle_balance = DEFAULT_CYCLES - Nat64.toNat(accepted.accepted);
                    admins = admins;
                    token_canister = token_canister;
                    voting_threshold = actual_voting_threshold;
                    quorum_threshold = actual_quorum_threshold;
                    expiry_time = actual_expiry;
                };

                ignore Map.put(db, thash, collectionId, collection);
                ?Principal.fromActor(castedBucket);
            };
        };
    };

    // Add a new admin to a collection (Genesis Admin only)
    public func addAdmin(
        caller : Principal,
        newAdmin : Principal,
        collectionId : CollectionId,
        db : Map.Map<CollectionId, Collection>,
    ) : async Bool {
        switch (Map.get(db, thash, collectionId)) {
            case null {
                return false; // Collection doesn't exist
            };
            case (?collection) {
                // Check if caller is Genesis Admin
                if (not isGenesisAdmin(caller, collection)) {
                    return false; // Not authorized
                };

                // Add the new admin
                ignore Map.put(collection.admins, phash, newAdmin, #Admin);
                return true;
            };
        };
    };

    // Remove an admin from a collection (Genesis Admin only)
    public func removeAdmin(
        caller : Principal,
        adminToRemove : Principal,
        collectionId : CollectionId,
        db : Map.Map<CollectionId, Collection>,
    ) : async Bool {
        switch (Map.get(db, thash, collectionId)) {
            case null {
                return false; // Collection doesn't exist
            };
            case (?collection) {
                // Check if caller is Genesis Admin
                if (not isGenesisAdmin(caller, collection)) {
                    return false; // Not authorized
                };

                // Check if target is not Genesis Admin
                switch (Map.get(collection.admins, phash, adminToRemove)) {
                    case (?#GenesisAdmin) {
                        return false; // Cannot remove Genesis Admin
                    };
                    case (?#Admin) {
                        // Remove the admin
                        ignore Map.remove(collection.admins, phash, adminToRemove);
                        return true;
                    };
                    case null {
                        return false; // Not an admin
                    };
                };
            };
        };
    };

    // Helper function to split text into semantic chunks
    private func createSemanticChunks(text : Text, documentId : DocumentId) : [SemanticChunk] {
        // Simple paragraph-based chunking for now
        // We'll use double newlines as paragraph separators
        let paragraphs = Text.split(text, #text("\n\n"));
        let chunks = Buffer.Buffer<SemanticChunk>(10);

        var position = 0;
        for (paragraph in paragraphs) {
            let trimmed = Text.trim(paragraph, #char(' '));

            if (trimmed.size() > 0) {
                // Create chunk ID combining document ID and position
                let chunkId = documentId # "_chunk_" # Nat.toText(position);

                chunks.add({
                    id = chunkId;
                    text = trimmed;
                    position = position;
                });

                position += 1;
            };
        };

        Buffer.toArray(chunks);
    };

    // Get or create a bucket for a collection
    public func getBucket(collectionId : CollectionId, db : Map.Map<CollectionId, Collection>) : async ?BucketActor {
        switch (Map.get(db, thash, collectionId)) {
            case (?collection) {
                ?collection.bucket;
            };
            case null {
                // Collection doesn't exist, return null
                // Users must explicitly create collections using createCollection
                null;
            };
        };
    };

    // Add a document with semantic chunking - now with admin check
    public func addDocument(
        caller : Principal,
        collectionId : CollectionId,
        db : Map.Map<CollectionId, Collection>,
        title : Text,
        content : Text,
    ) : async ?{
        collection : ?Principal;
        documentId : ?DocumentId;
    } {
        // Get the collection
        switch (Map.get(db, thash, collectionId)) {
            case null {
                return null; // Collection doesn't exist
            };
            case (?collection) {
                // Check if caller is an admin
                if (not isAdmin(caller, collection)) {
                    return null; // Not authorized
                };

                // Proceed with document addition
                // Validate content size
                if (content.size() > MAX_DOCUMENT_SIZE) {
                    throw Error.reject("Document exceeds maximum size");
                };

                // Generate document ID
                let documentId = await Utils.generateRandomID(title);

                // Get bucket for collection
                let bucket = collection.bucket;

                // Create document metadata
                let documentMetadata : DocumentMetadata = {
                    id = documentId;
                    title = title;
                    content_type = #PlainText;
                    source_url = null;
                    timestamp = Time.now();
                    total_chunks = 0; // Will update after chunking
                    size = content.size();
                    is_embedded = false;
                };

                // Add document to bucket
                let docResult = await bucket.insertDocument(documentMetadata);

                switch (docResult) {
                    case null {
                        throw Error.reject("Failed to store document metadata");
                    };
                    case (?_) {
                        // Create semantic chunks
                        let chunks = createSemanticChunks(content, documentId);

                        // Store each chunk
                        for (chunk in chunks.vals()) {
                            let _ = await bucket.insertChunk(documentId, chunk.id, chunk);
                        };

                        // Update document with chunk count
                        let updatedMetadata : DocumentMetadata = {
                            id = documentId;
                            title = title;
                            content_type = #PlainText;
                            source_url = null;
                            timestamp = Time.now();
                            total_chunks = chunks.size();
                            size = content.size();
                            is_embedded = false;
                        };

                        ignore await bucket.insertDocument(updatedMetadata);

                        let canisterID = Principal.fromActor(bucket);
                        return ?{
                            collection = ?canisterID;
                            documentId = ?documentId;
                        };
                    };
                };
            };
        };
    };

    // Add a vector for a specific chunk - now checking token configuration
    public func addVector(
        caller : Principal,
        collectionId : CollectionId,
        db : Map.Map<CollectionId, Collection>,
        documentId : DocumentId,
        chunkId : ChunkId,
        vectorId : VectorId,
        embedding : [Float],
        norm : Float,
    ) : async ?VectorId {
        // Get the collection
        switch (Map.get(db, thash, collectionId)) {
            case null {
                return null; // Collection doesn't exist
            };
            case (?collection) {
                // Check if token is configured
                switch (collection.token_canister) {
                    case (?_) {
                        // If token is configured, direct embedding is not allowed
                        // This would be handled through proposals instead
                        return null;
                    };
                    case null {
                        // No token configured, check if caller is admin
                        if (not isAdmin(caller, collection)) {
                            return null; // Not authorized
                        };

                        // Proceed with vector addition
                        let bucket = collection.bucket;

                        // Verify document exists
                        let documentOpt = await bucket.getDocument(documentId);
                        switch (documentOpt) {
                            case null {
                                return null;
                            };
                            case (?_) {
                                // Verify chunk exists
                                let chunkOpt = await bucket.getChunk(chunkId);
                                switch (chunkOpt) {
                                    case null {
                                        return null;
                                    };
                                    case (?_) {
                                        // Create vector
                                        let vector : Vector = {
                                            id = vectorId;
                                            document_id = documentId;
                                            chunk_id = chunkId;
                                            embedding = embedding;
                                            norm = norm;
                                        };

                                        // Store vector
                                        let result = await bucket.insertVector(vector);

                                        switch (result) {
                                            case null {
                                                return null;
                                            };
                                            case (?_) {
                                                return ?vectorId;
                                            };
                                        };
                                    };
                                };
                            };
                        };
                    };
                };
            };
        };
    };

    // Mark document as fully embedded - now with token check
    public func endVectorUpdate(
        caller : Principal,
        collectionId : CollectionId,
        db : Map.Map<CollectionId, Collection>,
        documentId : DocumentId,
    ) : async Bool {
        // Get the collection
        switch (Map.get(db, thash, collectionId)) {
            case null {
                return false; // Collection doesn't exist
            };
            case (?collection) {
                // Check if token is configured
                switch (collection.token_canister) {
                    case (?_) {
                        // If token is configured, direct completion is not allowed
                        // This would be handled through proposals instead
                        return false;
                    };
                    case null {
                        // No token configured, check if caller is admin
                        if (not isAdmin(caller, collection)) {
                            return false; // Not authorized
                        };

                        // Proceed with completion
                        let bucket = collection.bucket;
                        let result = await bucket.updateDocumentStatus(documentId, true);
                        switch (result) {
                            case null { false };
                            case (?_) { true };
                        };
                    };
                };
            };
        };
    };

    // The rest of the original functions without administrative changes

    // Get all vectors for a collection
    public func listVectors(
        collectionId : CollectionId,
        db : Map.Map<CollectionId, Collection>,
    ) : async ?{ items : [Vector] } {
        let bucketOpt = await getBucket(collectionId, db);

        switch (bucketOpt) {
            case (null) {
                return null; // Collection doesn't exist
            };
            case (?bucket) {
                // Get all vectors (no filter)
                let vectors = await bucket.getVectors(null);
                return ?{ items = vectors };
            };
        };
    };

    // Get all document metadata for a collection
    public func listDocumentMetadata(
        db : Map.Map<CollectionId, Collection>,
        collectionId : CollectionId,
    ) : async ?[DocumentMetadata] {
        let bucketOpt = await getBucket(collectionId, db);
        switch (bucketOpt) {
            case (null) {
                return null;
            };
            case (?bucket) {
                let documents = await bucket.listDocuments();
                return ?documents;
            };
        };

    };

    // Get a specific document's metadata
    public func getMetadata(
        db : Map.Map<CollectionId, Collection>,
        collectionId : CollectionId,
        documentId : DocumentId,
    ) : async ?DocumentMetadata {
        let bucketOpt = await getBucket(collectionId, db);
        switch (bucketOpt) {
            case (null) {
                return null;
            };
            case (?bucket) {
                await bucket.getDocument(documentId);
            };
        };

    };

    // Get all chunks for a document
    public func getChunks(
        db : Map.Map<CollectionId, Collection>,
        collectionId : CollectionId,
        documentId : DocumentId,
    ) : async ?Text {
        let bucketOpt = await getBucket(collectionId, db);
        switch (bucketOpt) {
            case (null) {
                return null;
            };
            case (?bucket) {
                // Get all chunks for document
                let chunks = await bucket.getDocumentChunks(documentId);

                if (chunks.size() == 0) {
                    return null;
                };

                // Combine chunks into original text
                var fullText = "";
                for (chunk in chunks.vals()) {
                    fullText := fullText # chunk.text # "\n\n";
                };

                ?fullText;
            };
        };

    };

    // Get the document ID for a specific vector
    public func getDocumentIdByVectorId(
        db : Map.Map<CollectionId, Collection>,
        collectionId : CollectionId,
        vectorId : VectorId,
    ) : async ?DocumentId {

        let bucketOpt = await getBucket(collectionId, db);
        switch (bucketOpt) {
            case (null) {
                return null;
            };
            case (?bucket) {
                let vectors = await bucket.getVectors(
                    ?{
                        document_id = null;
                        chunk_id = null;
                        limit = ?1;
                    }
                );
                for (vector in vectors.vals()) {
                    if (vector.id == vectorId) {
                        return ?vector.document_id;
                    };
                };

                null;
            };
        };

    };

    // Get a document title by ID
    public func documentIdToTitle(
        db : Map.Map<CollectionId, Collection>,
        collectionId : CollectionId,
        documentId : DocumentId,
    ) : async ?Text {

        let bucketOpt = await getBucket(collectionId, db);
        switch (bucketOpt) {
            case (null) {
                return null;
            };
            case (?bucket) {
                let documentOpt = await bucket.getDocument(documentId);
                switch (documentOpt) {
                    case (?doc) { ?doc.title };
                    case null { null };
                };
            };
        };

    };

    // Find a document ID by title
    public func titleToDocumentID(
        db : Map.Map<CollectionId, Collection>,
        collectionId : CollectionId,
        title : Text,
    ) : async ?DocumentId {

        let bucketOpt = await getBucket(collectionId, db);
        switch (bucketOpt) {
            case (null) {
                return null;
            };
            case (?bucket) {

                let documents = await bucket.listDocuments();

                for (doc in documents.vals()) {
                    if (doc.title == title) {
                        return ?doc.id;
                    };
                };

                null;
            };
        };

    };

    public func getBucketPrincipal(collectionId : CollectionId, db : Map.Map<CollectionId, Collection>) : async ?Principal {
        switch (Map.get(db, thash, collectionId)) {
            case (?collection) {
                ?Principal.fromActor(collection.bucket);
            };
            case null {
                null;
            };
        };
    };
};

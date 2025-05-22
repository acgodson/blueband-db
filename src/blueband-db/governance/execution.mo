//governance/execution.mo
import Debug "mo:base/Debug";
import Time "mo:base/Time";
import Int "mo:base/Int";
import Nat "mo:base/Nat";
import Array "mo:base/Array";
import Iter "mo:base/Iter";
import Float "mo:base/Float";
import Principal "mo:base/Principal";
import Buffer "mo:base/Buffer";

import Map "mo:map/Map";
import { thash } "mo:map/Map";

import Types "../Types";
import Proposals "./proposals";
import Embeddings "../embeddings";
import Storage "../bucket/BucketProvider";
import TokenInterface "./TokenInterface";
import ComputeProvider "../ComputeProvider";
import Error "mo:base/Error";
import Int64 "mo:base/Int64";

module {
    type ProposalId = Types.ProposalId;
    type DocumentProposal = Types.DocumentProposal;
    type DocumentId = Types.DocumentId;
    type ChunkId = Types.ChunkId;
    type VectorId = Types.VectorId;
    type SemanticChunk = Types.SemanticChunk;
    type CollectionId = Types.CollectionId;
    type Collection = Types.Collection;
    type Bucket = Storage.BucketActor;

    // Constants for scaling
    private let SCALE_FACTOR : Float = 1000000.0;
    private let OFFSET_VALUE : Float = 10.0;

    // Flag to determine whether to use scaled embeddings
    private let use_scaled : Bool = true; // Set to true for backward compatibility

    // Get the compute canister actor reference
    private func getComputeCanister(compute_canister_id : Principal) : ComputeProvider.ComputeProvider {
        actor (Principal.toText(compute_canister_id));
    };

    // Helper function for rolling back stored vectors
    private func rollbackVectors(
        bucket : Bucket,
        storedVectors : Buffer.Buffer<Text>,
    ) : async () {
        Debug.print("Rolling back stored vectors...");
        for (vectorId in storedVectors.vals()) {
            Debug.print("Deleting vector: " # vectorId);
            ignore await bucket.deleteVector(vectorId);
        };
    };

    // Process a batch of chunks using the compute canister
    private func processBatch(
        chunks : [SemanticChunk],
        texts : [Text],
        startIndex : Nat,
        documentId : Text,
        bucket : Bucket,
        storedVectors : Buffer.Buffer<Text>,
        compute_canister_id : Principal,
        embeddingAPI : Text,
    ) : async Bool {
        try {
            let compute_canister = getComputeCanister(compute_canister_id);
            let result = await compute_canister.generate_embeddings(
                texts,
                embeddingAPI,
                use_scaled,
            );

            switch (result) {
                case (#Ok(embeddingResult)) {
                    // FIXED: Add comprehensive validation before processing
                    let isValidResult = switch (embeddingResult) {
                        case (#Scaled(scaled)) {
                            scaled.values.size() == texts.size() and scaled.norms.size() == texts.size();
                        };
                        case (#Float(floatEmb)) {
                            floatEmb.embeddings.size() == texts.size() and floatEmb.norm_values.size() == texts.size();
                        };
                    };

                    if (not isValidResult) {
                        Debug.print("ERROR: Embedding result validation failed - size mismatch");
                        return false;
                    };

                    // Process embeddings with enhanced error checking
                    switch (embeddingResult) {
                        case (#Scaled(scaled)) {
                            // FIXED: Enhanced error handling for scaled processing
                            if (scaled.values.size() != texts.size() or scaled.norms.size() != texts.size()) {
                                Debug.print("ERROR: Scaled embedding size mismatch");
                                return false;
                            };

                            var success = true;
                            var i = 0;

                            while (i < texts.size() and success) {
                                let chunkIndex = startIndex + i;

                                // FIXED: Add bounds checking
                                if (chunkIndex >= chunks.size()) {
                                    Debug.print("ERROR: Chunk index out of bounds: " # Nat.toText(chunkIndex));
                                    success := false;
                                } else {
                                    let chunk = chunks[chunkIndex];
                                    let scaledValues = scaled.values[i];
                                    let scaledNorm = scaled.norms[i];

                                    // FIXED: Add validation for scaled values
                                    if (scaledValues.size() == 0) {
                                        Debug.print("ERROR: Empty embedding vector for chunk " # Nat.toText(chunkIndex));
                                        success := false;
                                    } else {
                                        // Convert scaled integers back to floats with validation
                                        let embedding = Array.map<Nat64, Float>(
                                            scaledValues,
                                            func(n : Nat64) : Float {
                                                Float.fromInt64(Int64.fromNat64(n)) / SCALE_FACTOR - OFFSET_VALUE;
                                            },
                                        );

                                        let norm = Float.fromInt64(Int64.fromNat64(scaledNorm)) / SCALE_FACTOR - OFFSET_VALUE;

                                        // FIXED: Validate converted values
                                        if (norm <= 0.0) {
                                            Debug.print("ERROR: Invalid norm value for chunk " # Nat.toText(chunkIndex));
                                            success := false;
                                        } else {
                                            let vectorId = documentId # "_vector_" # Nat.toText(chunkIndex);

                                            let vectorResult = await bucket.insertVector({
                                                id = vectorId;
                                                document_id = documentId;
                                                chunk_id = chunk.id;
                                                embedding = embedding;
                                                norm = norm;
                                            });

                                            switch (vectorResult) {
                                                case (?_) {
                                                    storedVectors.add(vectorId);
                                                    i += 1;
                                                };
                                                case null {
                                                    Debug.print("ERROR: Failed to store vector for chunk " # Nat.toText(chunkIndex));
                                                    success := false;
                                                };
                                            };
                                        };
                                    };
                                };
                            };

                            return success;
                        };

                        case (#Float(floatEmb)) {
                            // FIXED: Enhanced error handling for float processing
                            if (floatEmb.embeddings.size() != texts.size() or floatEmb.norm_values.size() != texts.size()) {
                                Debug.print("ERROR: Float embedding size mismatch");
                                return false;
                            };

                            var success = true;
                            var i = 0;

                            while (i < texts.size() and success) {
                                let chunkIndex = startIndex + i;

                                // FIXED: Add bounds checking
                                if (chunkIndex >= chunks.size()) {
                                    Debug.print("ERROR: Chunk index out of bounds: " # Nat.toText(chunkIndex));
                                    success := false;
                                } else {
                                    let chunk = chunks[chunkIndex];
                                    let embedding = floatEmb.embeddings[i];
                                    let norm = floatEmb.norm_values[i];

                                    // FIXED: Validate embedding values
                                    if (embedding.size() == 0) {
                                        Debug.print("ERROR: Empty embedding vector for chunk " # Nat.toText(chunkIndex));
                                        success := false;
                                    } else if (norm <= 0.0) {
                                        Debug.print("ERROR: Invalid norm value for chunk " # Nat.toText(chunkIndex));
                                        success := false;
                                    } else {
                                        let vectorId = documentId # "_vector_" # Nat.toText(chunkIndex);

                                        let vectorResult = await bucket.insertVector({
                                            id = vectorId;
                                            document_id = documentId;
                                            chunk_id = chunk.id;
                                            embedding = embedding;
                                            norm = norm;
                                        });

                                        switch (vectorResult) {
                                            case (?_) {
                                                storedVectors.add(vectorId);
                                                i += 1;
                                            };
                                            case null {
                                                Debug.print("ERROR: Failed to store vector for chunk " # Nat.toText(chunkIndex));
                                                success := false;
                                            };
                                        };
                                    };
                                };
                            };

                            return success;
                        };
                    };
                };
                case (#Err(error)) {
                    // FIXED: Enhanced error logging with context
                    Debug.print("ERROR: Compute canister returned error for batch starting at " # Nat.toText(startIndex) # ": " # error);
                    return false;
                };
            };
        } catch (error) {
            // FIXED: Enhanced error logging with context
            Debug.print("ERROR: Exception calling compute canister for batch starting at " # Nat.toText(startIndex) # ": " # Error.message(error));
            return false;
        };
    };

    public func createEmbeddingProposal(
        caller : Principal,
        collectionId : Text,
        documentId : Text,
        description : ?Text,
        collections : Map.Map<CollectionId, Collection>,
        proposals : Map.Map<ProposalId, DocumentProposal>,
    ) : async ?Types.ProposalId {
        switch (Map.get(collections, thash, collectionId)) {
            case null { return null };
            case (?collection) {
                let isAdmin = Storage.isAdmin(caller, collection);
                switch (collection.token_canister) {
                    case null {
                        if (not isAdmin) return null;
                    };
                    case (?tokenCanister) {
                        if (not isAdmin) {
                            let votingPower = await TokenInterface.getVotingPower(caller, ?tokenCanister);
                            if (votingPower == 0) return null;
                        };
                    };
                };

                let bucket = collection.bucket;
                let documentOpt = await bucket.getDocument(documentId);
                switch (documentOpt) {
                    case null { return null };
                    case (?doc) {
                        if (doc.is_embedded) return null;
                        if (Proposals.hasActiveProposal(collectionId, documentId, proposals)) return null;

                        let proposalId = Proposals.createProposal(
                            collectionId,
                            documentId,
                            caller,
                            description,
                            proposals,
                        );
                        return ?proposalId;
                    };
                };
            };
        };
    };

    public func voteOnProposal(
        caller : Principal,
        proposalId : Text,
        inFavor : Bool,
        collections : Map.Map<CollectionId, Collection>,
        proposals : Map.Map<ProposalId, DocumentProposal>,
    ) : async Bool {
        switch (Proposals.getProposal(proposalId, proposals)) {
            case null { return false };
            case (?proposal) {
                switch (Map.get(collections, thash, proposal.collectionId)) {
                    case null { return false };
                    case (?collection) {
                        let votingPower = await TokenInterface.getVotingPower(caller, collection.token_canister);
                        if (votingPower == 0) return false;

                        let result = await Proposals.vote(
                            proposalId,
                            caller,
                            inFavor,
                            votingPower,
                            collection,
                            proposals,
                        );

                        return switch (result) {
                            case (#ok(_)) true;
                            case (#err(_)) false;
                        };
                    };
                };
            };
        };
    };

    public func executeProposal(
        proposalId : Text,
        proposals : Map.Map<ProposalId, DocumentProposal>,
        collections : Map.Map<CollectionId, Collection>,
        embeddingAPI : Text,
        transform : shared query Embeddings.TransformArgs -> async Embeddings.HttpResponsePayload,
        caller : Principal,
        compute_canister_id : Principal,
    ) : async Bool {
        let startTime = Time.now();
        Debug.print("Executing proposal: " # proposalId);

        var storedVectors = Buffer.Buffer<Text>(0);

        switch (Proposals.getProposal(proposalId, proposals)) {
            case null {
                Debug.print("Proposal not found: " # proposalId);
                return false;
            };
            case (?proposal) {
                if (proposal.status != #Approved) {
                    Debug.print("Proposal not approved or already executed: " # proposalId);
                    return false;
                };

                let collection = switch (Map.get(collections, thash, proposal.collectionId)) {
                    case null {
                        Debug.print("Collection not found: " # proposal.collectionId);
                        return false;
                    };
                    case (?c) c;
                };

                let bucket = collection.bucket;

                let doc = switch (await bucket.getDocument(proposal.documentId)) {
                    case null {
                        Debug.print("Document not found: " # proposal.documentId);
                        return false;
                    };
                    case (?d) d;
                };

                if (doc.is_embedded) {
                    Debug.print("Document already embedded: " # proposal.documentId);
                    return true;
                };

                let chunks = await bucket.getDocumentChunks(proposal.documentId);
                if (chunks.size() == 0) {
                    Debug.print("No chunks found for document: " # proposal.documentId);
                    return false;
                };

                let texts = Array.map<SemanticChunk, Text>(
                    chunks,
                    func(chunk) = chunk.text,
                );

                Debug.print("Generating embeddings for " # Nat.toText(texts.size()) # " chunks");

                // Calculate dynamic batch size based on document size
                let MAX_BATCH_SIZE = 10;
                let totalSize = Array.foldLeft<Text, Nat>(
                    texts,
                    0,
                    func(acc, text) { acc + text.size() },
                );
                let avgChunkSize = totalSize / texts.size();

                // Adjust batch size based on average chunk size
                // Smaller chunks can be processed in larger batches
                let batchSize = if (avgChunkSize > 1000) {
                    // Large chunks - process fewer at a time
                    Nat.min(3, MAX_BATCH_SIZE);
                } else if (avgChunkSize > 500) {
                    // Medium chunks
                    Nat.min(5, MAX_BATCH_SIZE);
                } else {
                    // Small chunks - can process more
                    MAX_BATCH_SIZE;
                };

                Debug.print("Using batch size: " # Nat.toText(batchSize) # " (avg chunk size: " # Nat.toText(avgChunkSize) # ")");
                var batchStart = 0;
                var success = true;

                while (batchStart < texts.size() and success) {
                    let batchEnd = Nat.min(batchStart + batchSize - 1, texts.size() - 1);
                    Debug.print("Processing batch " # Nat.toText(batchStart) # " to " # Nat.toText(batchEnd));

                    let batchTexts = Array.tabulate<Text>(
                        batchEnd - batchStart + 1,
                        func(i) = texts[batchStart + i],
                    );

                    success := await processBatch(
                        chunks,
                        batchTexts,
                        batchStart,
                        proposal.documentId,
                        bucket,
                        storedVectors,
                        compute_canister_id,
                        embeddingAPI,
                    );

                    if (success) {
                        batchStart := batchEnd + 1;
                    };
                };

                if (success) {
                    let result = await bucket.updateDocumentStatus(proposal.documentId, true);
                    switch (result) {
                        case null {
                            Debug.print("Failed to update document status");
                            await rollbackVectors(bucket, storedVectors);
                            return false;
                        };
                        case (?_) {
                            let updatedProposal : DocumentProposal = {
                                id = proposal.id;
                                collectionId = proposal.collectionId;
                                documentId = proposal.documentId;
                                proposer = proposal.proposer;
                                description = proposal.description;
                                votesFor = proposal.votesFor;
                                votesAgainst = proposal.votesAgainst;
                                voters = proposal.voters;
                                status = #Executed;
                                created = proposal.created;
                                expires = proposal.expires;
                            };

                            ignore Map.put(proposals, thash, proposalId, updatedProposal);
                            let duration = (Time.now() - startTime) / 1_000_000_000;
                            Debug.print("Proposal execution completed in " # Int.toText(duration) # " seconds");
                            return true;
                        };
                    };
                } else {
                    Debug.print("Embedding failed, rolling back...");
                    await rollbackVectors(bucket, storedVectors);
                    return false;
                };
            };
        };
    };
};

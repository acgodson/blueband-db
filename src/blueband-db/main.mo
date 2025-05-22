//main.mo
import Debug "mo:base/Debug";
import Cycles "mo:base/ExperimentalCycles";
import Principal "mo:base/Principal";
import Text "mo:base/Text";
import Map "mo:map/Map";
import Nat "mo:base/Nat";
import Float "mo:base/Float";
import Nat64 "mo:base/Nat64";
import Buffer "mo:base/Buffer";
import Error "mo:base/Error";
import { thash } "mo:map/Map";

import Storage "./bucket/BucketProvider";
import Embeddings "./embeddings";
import Types "./Types";
import BluebandProvider "./BluebandProvider";
import Proposals "./governance/proposals";
import CollectionManager "./collection/collectionManager";
import Execution "./governance/execution";
import ComputeProvider "./ComputeProvider";

shared ({ caller }) actor class Blueband(compute_canister_id : Principal) : async BluebandProvider.BluebandProvider {
    type EmbeddingsResponse = Embeddings.EmbeddingsResponse;
    type Transform = Embeddings.TransformArgs -> async Embeddings.HttpResponsePayload;

    // State
    stable var state : Types.Database = Types.emptyDatabase();

    // Proposal state
    stable var proposals = Map.new<Types.ProposalId, Types.DocumentProposal>();

    private let compute_canister : ComputeProvider.ComputeProvider = actor (Principal.toText(compute_canister_id));
    private let PROXY_ENDPOINT = "https://us-central1-blueband-db-442d8.cloudfunctions.net/proxy";

    private func getOpenAIApi() : Embeddings.Embeddings {
        Embeddings.Embeddings(compute_canister);
    };

    //////////////////////////
    // Collection Management
    //////////////////////////

    // Create a new collection with admin privileges
    public shared ({ caller }) func createCollection(
        collectionId : Text,
        token_canister : ?Principal,
        voting_threshold : ?Nat,
        quorum_threshold : ?Nat,
        expiry_time : ?Nat,
    ) : async ?Principal {
        await CollectionManager.createCollection(
            collectionId,
            caller,
            token_canister,
            voting_threshold,
            quorum_threshold,
            expiry_time,
            state.collections,
        );
    };

    //////////////////////////
    // Admin Management
    //////////////////////////

    // Add a new admin to a collection
    public shared ({ caller }) func addCollectionAdmin(
        collectionId : Text,
        newAdmin : Principal,
    ) : async Bool {
        await CollectionManager.addCollectionAdmin(
            caller,
            collectionId,
            newAdmin,
            state.collections,
        );
    };

    // Remove an admin from a collection
    public shared ({ caller }) func removeCollectionAdmin(
        collectionId : Text,
        adminToRemove : Principal,
    ) : async Bool {
        await CollectionManager.removeCollectionAdmin(
            caller,
            collectionId,
            adminToRemove,
            state.collections,
        );
    };

    // Check if a principal is an admin for a collection
    public query ({ caller }) func isCollectionAdmin(
        collectionId : Text
    ) : async Bool {
        CollectionManager.isCollectionAdmin(
            caller,
            collectionId,
            state.collections,
        );
    };

    //////////////////////////
    // Storage Operations
    //////////////////////////

    // Add a document to a collection
    public shared ({ caller }) func addDocument(
        collectionId : Text,
        title : Text,
        content : Text,
    ) : async ?{
        collection : ?Principal;
        documentId : ?Types.DocumentId;
    } {
        await Storage.addDocument(
            caller,
            collectionId,
            state.collections,
            title,
            content,
        );
    };

    // Get all chunks for a document
    public shared func getDocumentContent(
        collectionId : Text,
        documentId : Text,
    ) : async ?Text {
        await Storage.getChunks(state.collections, collectionId, documentId);
    };

    // Get semantic chunks individually (for frontend display or processing)
    public shared func getDocumentChunks(
        collectionId : Text,
        documentId : Text,
    ) : async ?[Types.SemanticChunk] {
        let bucketOpt = await Storage.getBucket(collectionId, state.collections);
        switch (bucketOpt) {
            case null { return null };
            case (?bucket) {
                let chunks = await bucket.getDocumentChunks(documentId);

                if (chunks.size() == 0) {
                    return null;
                };

                ?chunks;
            };
        };
    };

    // Add a vector for a specific chunk (admin only, if no token)
    public shared ({ caller }) func addVector(
        collectionId : Text,
        documentId : Text,
        chunkId : Text,
        vectorId : Text,
        embedding : [Float],
        norm : Float,
    ) : async ?Text {
        await Storage.addVector(
            caller,
            collectionId,
            state.collections,
            documentId,
            chunkId,
            vectorId,
            embedding,
            norm,
        );
    };

    // Mark document as fully embedded (admin only, if no token)
    public shared ({ caller }) func completeEmbedding(
        collectionId : Text,
        documentId : Text,
    ) : async Bool {
        await Storage.endVectorUpdate(
            caller,
            collectionId,
            state.collections,
            documentId,
        );
    };

    //////////////////////////
    // Governance Functions
    //////////////////////////

    // Create a proposal to embed a document
    public shared ({ caller }) func createEmbeddingProposal(
        collectionId : Text,
        documentId : Text,
        description : ?Text,
    ) : async ?Types.ProposalId {
        await Execution.createEmbeddingProposal(
            caller,
            collectionId,
            documentId,
            description,
            state.collections,
            proposals,
        );
    };

    // Vote on a proposal
    public shared ({ caller }) func voteOnProposal(
        proposalId : Text,
        inFavor : Bool,
    ) : async Bool {
        let result = await Execution.voteOnProposal(
            caller,
            proposalId,
            inFavor,
            state.collections,
            proposals,
        );

        // If successful and proposal is approved, execute it
        if (result) {
            switch (Proposals.getProposal(proposalId, proposals)) {
                case (?proposal) {
                    if (proposal.status == #Approved) {
                        // Process approved proposal in the background
                        ignore executeProposal(proposalId);
                    };
                };
                case (null) {};
            };
        };

        return result;
    };

    // Execute an approved proposal manually
    public shared ({ caller }) func executeProposal(proposalId : Text) : async Bool {
        switch (Proposals.getProposal(proposalId, proposals)) {
            case (?proposal) {
                if (proposal.status != #Approved) {
                    return false;
                };
                await Execution.executeProposal(
                    proposalId,
                    proposals,
                    state.collections,
                    PROXY_ENDPOINT,
                    transform,
                    caller,
                    compute_canister_id,
                );
            };
            case null { false };
        };
    };

    // Query proposal details
    public query func getProposal(proposalId : Text) : async ?Types.DocumentProposal {
        Proposals.getProposal(proposalId, proposals);
    };

    // List proposals for a collection
    public query func listProposals(
        collectionId : Text,
        status : ?Types.ProposalStatus,
    ) : async [Types.DocumentProposal] {
        Proposals.listProposals(collectionId, status, proposals);
    };

    //////////////////////////
    // Query Operations
    //////////////////////////

    // Get all vectors for a collection
    public shared func getVectorIndex(
        collectionId : Text
    ) : async ?{ items : [Types.Vector] } {
        await Storage.listVectors(collectionId, state.collections);
    };

    // Get all document metadata for a collection
    public shared func getDocumentList(
        collectionId : Text
    ) : async ?[Types.DocumentMetadata] {
        await Storage.listDocumentMetadata(state.collections, collectionId);
    };

    // Get a specific document's metadata
    public shared func getDocumentMetadata(
        collectionId : Text,
        documentId : Text,
    ) : async ?Types.DocumentMetadata {
        await Storage.getMetadata(state.collections, collectionId, documentId);
    };

    // Get the collection canister principal
    public shared func getCollectionPrincipal(
        collectionId : Text
    ) : async ?Principal {
        await CollectionManager.getCollectionPrincipal(
            collectionId,
            state.collections,
        );
    };

    //////////////////////////
    // Query Utilities
    //////////////////////////

    // Add to main.mo
    public shared func queryVectorSimilarity(
        collectionId : Text,
        queryText : Text,
        limit : ?Nat,
    ) : async ?{
        matches : [{
            score : Float;
            document_id : Text;
            chunk_id : Text;
            document_title : ?Text;
            chunk_text : ?Text;
        }];
    } {
        switch (Map.get(state.collections, thash, collectionId)) {
            case null { return null }; // Collection doesn't exist
            case (?collection) {
                try {
                    // Get the bucket principal to pass to Rust canister
                    let bucketPrincipal = Principal.fromActor(collection.bucket);

                    // Call Rust canister for similarity search
                    let queryResult = await compute_canister.query_text(
                        queryText,
                        Principal.toText(bucketPrincipal),
                        ?{
                            document_ids = null;
                            chunk_ids = null;
                            limit = switch (limit) {
                                case (?l) { ?Nat64.fromNat(l) };
                                case null { ?5 }; // Default limit
                            };
                        },
                    );

                    switch (queryResult) {
                        case (#Ok(result)) {
                            // Enrich results with document titles and chunk text
                            let enrichedMatches = Buffer.Buffer<{ score : Float; document_id : Text; chunk_id : Text; document_title : ?Text; chunk_text : ?Text }>(result.matches.size());

                            for (match in result.matches.vals()) {
                                // Get document title
                                let titleOpt = await Storage.documentIdToTitle(
                                    state.collections,
                                    collectionId,
                                    match.document_id,
                                );

                                // Get chunk text
                                let chunkOpt = await collection.bucket.getChunk(match.chunk_id);
                                let chunkText = switch (chunkOpt) {
                                    case (?chunk) { ?chunk.text };
                                    case null { null };
                                };

                                enrichedMatches.add({
                                    score = match.score;
                                    document_id = match.document_id;
                                    chunk_id = match.chunk_id;
                                    document_title = titleOpt;
                                    chunk_text = chunkText;
                                });
                            };

                            ?{ matches = Buffer.toArray(enrichedMatches) };
                        };
                        case (#Err(error)) {
                            Debug.print("Query error: " # error);
                            null;
                        };
                    };
                } catch (error) {
                    Debug.print("Query failed: " # Error.message(error));
                    null;
                };
            };
        };
    };

    // Get the document ID for a specific vector
    public shared func getDocumentIdFromVector(
        collectionId : Text,
        vectorId : Text,
    ) : async ?Types.DocumentId {
        await Storage.getDocumentIdByVectorId(state.collections, collectionId, vectorId);
    };

    // Get a document title by ID
    public shared func getDocumentTitle(
        collectionId : Text,
        documentId : Text,
    ) : async ?Text {
        await Storage.documentIdToTitle(state.collections, collectionId, documentId);
    };

    // Find a document ID by title
    public shared func findDocumentByTitle(
        collectionId : Text,
        title : Text,
    ) : async ?Text {
        await Storage.titleToDocumentID(state.collections, collectionId, title);
    };

    //////////////////////////
    // Embedding Operations
    //////////////////////////

    // Generate embeddings for texts using OpenAI
    public shared func generateEmbeddings(texts : [Text]) : async EmbeddingsResponse {
        try {
            let result = await compute_canister.generate_embeddings(texts, PROXY_ENDPOINT, true);

            switch (result) {
                case (#Ok(embeddings)) {
                    // Format the embedding result to match the expected response format
                    let embeddings_api = getOpenAIApi();
                    let raw_response = embeddings_api.formatEmbeddingsResponse(embeddings);
                    #success({ raw_response = raw_response });
                };
                case (#Err(error)) {
                    if (Text.contains(error, #text "rate limit")) {
                        #rate_limited("The embeddings API returned a rate limit error: " # error);
                    } else {
                        #error("The embeddings API returned an error: " # error);
                    };
                };
            };
        } catch (error) {
            #error("Failed to call compute canister: " # Error.message(error));
        };
    };

    // HTTP transform function
    public shared query func transform(
        args : Embeddings.TransformArgs
    ) : async Embeddings.HttpResponsePayload {
        {
            status = args.response.status;
            body = args.response.body;
            headers = [
                {
                    name = "Content-Security-Policy";
                    value = "default-src 'self'";
                },
                { name = "Referrer-Policy"; value = "strict-origin" },
                { name = "Permissions-Policy"; value = "geolocation=(self)" },
                {
                    name = "Strict-Transport-Security";
                    value = "max-age=63072000";
                },
                { name = "X-Frame-Options"; value = "DENY" },
                { name = "X-Content-Type-Options"; value = "nosniff" },
            ];
        };
    };

    //////////////////////////
    // Cycle Management
    //////////////////////////

    public shared ({ caller }) func wallet_receive() : async () {
        ignore Cycles.accept<system>(Cycles.available());
        Debug.print("Cycles deposited by " # Principal.toText(caller));
    };

    // Add to main.mo initialization or separate function
    public shared ({ caller = _ }) func donateToComputeCanister() : async Bool {
        let available = Cycles.available(); // Caller's cycles sent with the call
        let accepted = Cycles.accept<system>(available); // Accept caller's cycles
        Cycles.add<system>(accepted); // Forward to compute canister
        await compute_canister.wallet_receive();
        true;
    };

    //////////////////////////
    // System Functions
    //////////////////////////

    // Process expired proposals periodically
    system func timer(setGlobalTimer : Nat64 -> ()) : async () {
        // Process expired proposals
        Proposals.processExpiredProposals(proposals);

        // Set timer to run again in 1 hour
        setGlobalTimer(3_600_000_000_000); // 1 hour in nanoseconds
    };
};

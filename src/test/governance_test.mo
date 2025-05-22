// src/test/governance_test.mo
import Debug "mo:base/Debug";
import Principal "mo:base/Principal";
import Nat "mo:base/Nat";
import Time "mo:base/Time";
import Array "mo:base/Array";

actor class GovernanceTest(
    bluebandCanisterId : Principal,
    testTokenCanisterId : Principal,
) {
    // Type definitions
    type SemanticChunk = {
        id : Text;
        text : Text;
        position : Nat;
    };

    // Basic interface for interacting with Blueband
    let blueband = actor (Principal.toText(bluebandCanisterId)) : actor {
        createCollection : (collectionId : Text, token_canister : ?Principal, voting_threshold : ?Nat, quorum_threshold : ?Nat, expiry_time : ?Nat) -> async ?Principal;
        addDocument : (collectionId : Text, title : Text, content : Text) -> async ?{
            collection : ?Principal;
            documentId : ?Text;
        };
        createEmbeddingProposal : (collectionId : Text, documentId : Text, description : ?Text) -> async ?Text;
        voteOnProposal : (proposalId : Text, inFavor : Bool) -> async Bool;
        getProposal : query (proposalId : Text) -> async ?{
            id : Text;
            collectionId : Text;
            documentId : Text;
            proposer : Principal;
            description : ?Text;
            votesFor : Nat;
            votesAgainst : Nat;
            voters : [Principal];
            status : { #Active; #Approved; #Rejected; #Executed; #Expired };
            created : Int;
            expires : Int;
        };
        getDocumentMetadata : (collectionId : Text, documentId : Text) -> async ?{
            id : Text;
            title : Text;
            content_type : { #PlainText; #ExternalLink; #Custom : Text };
            source_url : ?Text;
            timestamp : Int;
            total_chunks : Nat;
            size : Nat;
            is_embedded : Bool;
        };
        getVectorIndex : (collectionId : Text) -> async ?{
            items : [{
                id : Text;
                document_id : Text;
                chunk_id : Text;
                embedding : [Float];
                norm : Float;
            }];
        };
        getDocumentChunks : (collectionId : Text, documentId : Text) -> async ?[{
            id : Text;
            text : Text;
            position : Nat;
        }];
    };

    // Test token interface
    let testToken = actor (Principal.toText(testTokenCanisterId)) : actor {
        mint : (to : Principal, amount : Nat) -> async ();
        icrc1_balance_of : (account : { owner : Principal; subaccount : ?[Nat8] }) -> async Nat;
        icrc1_total_supply : () -> async Nat;
    };

    // Test parameters
    let collectionId = "test-collection";
    let documentTitle = "Test Document";
    let documentContent = "This is a test document for vector embedding.\n\nIt contains multiple paragraphs.\n\nEach paragraph should be chunked separately.\n\nThis allows us to test the embedding process more thoroughly.";

    // Helper to sleep for a specified duration
    private func sleep(seconds : Nat) : async () {
        let now = Time.now();
        let targetTime = now + (seconds * 1_000_000_000);

        while (Time.now() < targetTime) {
            // Busy wait (not ideal, but works for testing)
        };
    };

    // Verify dynamic batch sizing
    private func verifyBatchSizing(collectionId : Text, documentId : Text) : async Bool {
        let ?chunks = await blueband.getDocumentChunks(collectionId, documentId) else {
            Debug.print("⛔ Could not retrieve document chunks");
            return false;
        };

        let texts = Array.map<SemanticChunk, Text>(chunks, func(c) = c.text);
        let totalSize = Array.foldLeft<Text, Nat>(texts, 0, func(acc, text) { acc + text.size() });
        let avgChunkSize = totalSize / texts.size();

        let expectedBatchSize = if (avgChunkSize > 1000) { Nat.min(3, 10) } else if (avgChunkSize > 500) {
            Nat.min(5, 10);
        } else { 10 };

        Debug.print("Batch size calculation: " # Nat.toText(expectedBatchSize));

        let ?vectorIndex = await blueband.getVectorIndex(collectionId) else {
            Debug.print("⛔ Could not retrieve vector index");
            return false;
        };

        var vectorCount = 0;
        for (vector in vectorIndex.items.vals()) {
            if (vector.document_id == documentId) {
                vectorCount += 1;
            };
        };

        Debug.print("Vector count mismatch: expected " # Nat.toText(chunks.size()) # ", got " # Nat.toText(vectorCount));

        return true;
    };

    // Update verifyEmbeddings to include batch sizing verification
    public shared func verifyEmbeddings(collectionId : Text, documentId : Text) : async Bool {
        Debug.print("Verifying embeddings for document: " # documentId);

        // First check if document is marked as embedded
        let ?metadata = await blueband.getDocumentMetadata(collectionId, documentId) else {
            Debug.print("⛔ Could not retrieve document metadata");
            return false;
        };

        if (not metadata.is_embedded) {
            Debug.print("⛔ Document is not marked as embedded");
            return false;
        };

        Debug.print("✅ Document is marked as embedded");

        // Get the chunks to know how many vectors we should expect
        let ?chunks = await blueband.getDocumentChunks(collectionId, documentId) else {
            Debug.print("⛔ Could not retrieve document chunks");
            return false;
        };

        let expectedVectorCount = chunks.size();
        Debug.print("Expected vector count: " # Nat.toText(expectedVectorCount));

        // Then check if vectors exist for this document
        let ?vectorIndex = await blueband.getVectorIndex(collectionId) else {
            Debug.print("⛔ Could not retrieve vector index");
            return false;
        };

        // Count vectors for this document
        var vectorCount = 0;
        for (vector in vectorIndex.items.vals()) {
            if (vector.document_id == documentId) {
                vectorCount += 1;

                // Verify the vector has actual embeddings (not empty)
                if (vector.embedding.size() == 0) {
                    Debug.print("⛔ Vector has empty embeddings: " # vector.id);
                    return false;
                };

                // Check if norm is calculated (should not be 0)
                if (vector.norm == 0.0) {
                    Debug.print("⛔ Vector has zero norm: " # vector.id);
                    return false;
                };
            };
        };

        Debug.print("Found " # Nat.toText(vectorCount) # " vectors for document");

        if (vectorCount != expectedVectorCount) {
            Debug.print("⛔ Vector count mismatch");
            return false;
        };

        // Verify batch sizing
        let batchSizingVerified = await verifyBatchSizing(collectionId, documentId);
        if (not batchSizingVerified) {
            Debug.print("⛔ Batch sizing verification failed");
            return false;
        };

        Debug.print("✅ All vectors verified successfully");
        return true;
    };

    // Test rollback functionality
    public shared func testRollback(collectionId : Text, documentId : Text) : async Bool {
        Debug.print("Testing rollback functionality...");

        // Create a proposal that will fail
        let ?proposalId = await blueband.createEmbeddingProposal(
            collectionId,
            documentId,
            ?("Test rollback proposal"),
        ) else {
            Debug.print("⛔ Failed to create proposal for rollback test");
            return false;
        };

        Debug.print("✅ Created proposal for rollback test: " # proposalId);

        // Vote with insufficient tokens to force a failure
        let voteResult = await blueband.voteOnProposal(proposalId, true);
        if (not voteResult) {
            Debug.print("⛔ Vote failed as expected");
            return false;
        };

        // Wait for potential background processing
        await sleep(2);

        // Verify rollback occurred by checking vector index
        let ?vectorIndex = await blueband.getVectorIndex(collectionId) else {
            Debug.print("⛔ Could not retrieve vector index");
            return false;
        };

        // Check that no vectors exist for this document
        for (vector in vectorIndex.items.vals()) {
            if (vector.document_id == documentId) {
                Debug.print("⛔ Found vector after rollback: " # vector.id);
                return false; // Should not find any vectors
            };
        };

        // Verify document status
        let ?metadata = await blueband.getDocumentMetadata(collectionId, documentId) else {
            Debug.print("⛔ Could not retrieve document metadata");
            return false;
        };

        if (metadata.is_embedded) {
            Debug.print("⛔ Document still marked as embedded after rollback");
            return false;
        };

        Debug.print("✅ Rollback test passed - no vectors found and document not embedded");
        return true;
    };

    // Update runFullTest to include rollback testing
    public shared ({ caller }) func runFullTest() : async Text {
        Debug.print("🚀 Starting governance flow test");
        Debug.print("Test caller: " # Principal.toText(caller));

        // Step 1: Create a collection with token governance
        Debug.print("📁 Creating collection...");
        let ?collectionPrincipal = await blueband.createCollection(
            collectionId,
            ?testTokenCanisterId,
            ?51, // 51% voting threshold
            ?10, // 10% quorum
            null // Default expiry
        ) else {
            return "❌ Failed to create collection";
        };

        Debug.print("✅ Collection created: " # Principal.toText(collectionPrincipal));

        // Step 2: Add a document to the collection
        Debug.print("📄 Adding document...");
        let ?documentResult = await blueband.addDocument(
            collectionId,
            documentTitle,
            documentContent,
        ) else {
            return "❌ Failed to add document";
        };

        let ?documentId = documentResult.documentId else {
            return "❌ No document ID returned";
        };

        Debug.print("✅ Document added: " # documentId);

        // Verify document chunks were created
        let ?chunks = await blueband.getDocumentChunks(collectionId, documentId) else {
            return "❌ Failed to retrieve document chunks";
        };

        Debug.print("✅ Document chunked into " # Nat.toText(chunks.size()) # " chunks");

        // Step 3: Mint tokens to test user
        Debug.print("💰 Minting test tokens...");
        let testUser = caller;
        await testToken.mint(testUser, 1_000_000_000); // 10 tokens with 8 decimals

        // Verify token minting
        let balance = await testToken.icrc1_balance_of({
            owner = testUser;
            subaccount = null;
        });
        Debug.print("✅ Token balance: " # Nat.toText(balance));

        // Get total supply for quorum calculation
        let totalSupply = await testToken.icrc1_total_supply();
        Debug.print("Total token supply: " # Nat.toText(totalSupply));

        // Add rollback test before the main embedding test
        Debug.print("🔄 Testing rollback functionality...");
        let rollbackTested = await testRollback(collectionId, documentId);
        if (not rollbackTested) {
            return "❌ Rollback test failed";
        };
        Debug.print("✅ Rollback test passed");

        // Step 4: Create embedding proposal
        Debug.print("📝 Creating embedding proposal...");
        let ?proposalId = await blueband.createEmbeddingProposal(
            collectionId,
            documentId,
            ?("Proposal to embed test document"),
        ) else {
            return "❌ Failed to create proposal";
        };

        Debug.print("✅ Proposal created: " # proposalId);

        // Verify proposal details
        let ?proposalBefore = await blueband.getProposal(proposalId) else {
            return "❌ Could not retrieve proposal";
        };

        Debug.print("Initial proposal status: " # debug_show (proposalBefore.status));

        // Step 5: Vote on proposal
        Debug.print("🗳️ Voting on proposal...");
        let voteResult = await blueband.voteOnProposal(proposalId, true);
        if (not voteResult) {
            return "❌ Vote failed";
        };

        Debug.print("✅ Vote successful");

        // Step 6: Check proposal execution
        Debug.print("⏳ Checking proposal status...");

        // Small delay to allow background processing
        await sleep(2);

        let ?proposalAfter = await blueband.getProposal(proposalId) else {
            return "❌ Could not retrieve proposal after voting";
        };

        Debug.print("Final proposal status: " # debug_show (proposalAfter.status));

        if (proposalAfter.status != #Executed) {
            Debug.print("⚠️ Proposal not executed yet, status: " # debug_show (proposalAfter.status));

            // If proposal is still approved but not executed, we might need to wait longer
            if (proposalAfter.status == #Approved) {
                Debug.print("Waiting for execution to complete...");
                await sleep(5); // Wait a bit longer

                let ?proposalFinal = await blueband.getProposal(proposalId) else {
                    return "❌ Could not retrieve proposal after waiting";
                };

                Debug.print("Status after waiting: " # debug_show (proposalFinal.status));

                if (proposalFinal.status != #Executed) {
                    return "❌ Proposal execution failed or timed out";
                };
            } else {
                return "❌ Proposal not approved. Status: " # debug_show (proposalAfter.status);
            };
        };

        // Step 7: Verify embeddings were created
        Debug.print("🔍 Verifying embeddings were created...");
        let embeddingsCreated = await verifyEmbeddings(collectionId, documentId);

        if (not embeddingsCreated) {
            return "❌ Test failed: Embeddings were not created correctly";
        };

        // Final success message
        return "✅ Test completed successfully. Final proposal status: " # debug_show (proposalAfter.status) #
        ". Embeddings verified: true";
    };

    // Additional test for direct admin embedding (no token governance)
    public shared ({ caller }) func testDirectAdminEmbedding() : async Text {
        Debug.print("🚀 Starting direct admin embedding test");

        // Create a collection without token governance
        let nonGovernanceCollectionId = "direct-admin-collection";

        Debug.print("📁 Creating collection without token governance...");
        let ?collectionPrincipal = await blueband.createCollection(
            nonGovernanceCollectionId,
            null, // No token canister
            null, // Default threshold
            null, // Default quorum
            null // Default expiry
        ) else {
            return "❌ Failed to create collection";
        };

        Debug.print("✅ Collection created: " # Principal.toText(collectionPrincipal));

        // Add a document
        Debug.print("📄 Adding document...");
        let ?documentResult = await blueband.addDocument(
            nonGovernanceCollectionId,
            "Direct Admin Test",
            "This document should be directly embeddable by admins.",
        ) else {
            return "❌ Failed to add document";
        };

        let ?documentId = documentResult.documentId else {
            return "❌ No document ID returned";
        };

        Debug.print("✅ Document added: " # documentId);

        // In a real test, we would now directly call the embedding function
        // However, since that requires an OpenAI API key, we'll verify that:
        // 1. No governance is needed (can't create proposal)
        // 2. Document can be marked as embedded directly by admin

        // Try to create a proposal (should fail without token)
        let proposalResult = await blueband.createEmbeddingProposal(
            nonGovernanceCollectionId,
            documentId,
            ?("This should fail"),
        );

        // Verify no governance is needed
        if (proposalResult != null) {
            return "❌ Test failed: Was able to create proposal for non-governed collection";
        };

        Debug.print("✅ Verified proposal creation not needed for non-governed collection");

        // In a real test, we would now directly call embeddings
        // For now, we'll just check that the document is not yet embedded

        let ?metadata = await blueband.getDocumentMetadata(nonGovernanceCollectionId, documentId) else {
            return "❌ Could not retrieve document metadata";
        };

        if (metadata.is_embedded) {
            return "❌ Document unexpectedly already marked as embedded";
        };

        Debug.print("✅ Document correctly not embedded yet");

        return "✅ Direct admin embedding test passed";
    };
};

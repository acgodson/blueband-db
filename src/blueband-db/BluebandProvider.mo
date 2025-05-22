//bluebandprovider.mo
import Types "./Types";
import Principal "mo:base/Principal";

module {
    type CollectionId = Types.CollectionId;
    type DocumentId = Types.DocumentId;
    type ChunkId = Types.ChunkId;
    type VectorId = Types.VectorId;
    type Vector = Types.Vector;
    type DocumentMetadata = Types.DocumentMetadata;
    type SemanticChunk = Types.SemanticChunk;
    type ProposalId = Types.ProposalId;
    type ProposalStatus = Types.ProposalStatus;
    type DocumentProposal = Types.DocumentProposal;
    type EmbeddingsResponse = {
        #success : {
            raw_response : Text;
        };
        #rate_limited : Text;
        #error : Text;
    };
    type TransformArgs = {
        response : {
            status : Nat;
            headers : [{ name : Text; value : Text }];
            body : [Nat8];
        };
        context : Blob;
    };
    type HttpResponsePayload = {
        status : Nat;
        headers : [{ name : Text; value : Text }];
        body : [Nat8];
    };

    // Define the public interface for the Blueband service
    public type BluebandProvider = actor {
        // Collection Management
        createCollection : (
            collectionId : Text,
            token_canister : ?Principal,
            voting_threshold : ?Nat,
            quorum_threshold : ?Nat,
            expiry_time : ?Nat,
        ) -> async ?Principal;

        // Admin Management
        addCollectionAdmin : (collectionId : Text, newAdmin : Principal) -> async Bool;
        removeCollectionAdmin : (collectionId : Text, adminToRemove : Principal) -> async Bool;
        isCollectionAdmin : query (collectionId : Text) -> async Bool;

        // Document Management
        addDocument : (collectionId : Text, title : Text, content : Text) -> async ?{
            collection : ?Principal;
            documentId : ?DocumentId;
        };
        getDocumentContent : (collectionId : Text, documentId : Text) -> async ?Text;
        getDocumentChunks : (collectionId : Text, documentId : Text) -> async ?[SemanticChunk];

        // Vector Operations
        addVector : (
            collectionId : Text,
            documentId : Text,
            chunkId : Text,
            vectorId : Text,
            embedding : [Float],
            norm : Float,
        ) -> async ?Text;
        completeEmbedding : (collectionId : Text, documentId : Text) -> async Bool;

        // Governance
        createEmbeddingProposal : (collectionId : Text, documentId : Text, description : ?Text) -> async ?ProposalId;
        voteOnProposal : (proposalId : Text, inFavor : Bool) -> async Bool;
        getProposal : query (proposalId : Text) -> async ?DocumentProposal;
        listProposals : query (collectionId : Text, status : ?ProposalStatus) -> async [DocumentProposal];

        // Query Operations
        getVectorIndex : (collectionId : Text) -> async ?{ items : [Vector] };
        getDocumentList : (collectionId : Text) -> async ?[DocumentMetadata];
        getDocumentMetadata : (collectionId : Text, documentId : Text) -> async ?DocumentMetadata;
        getCollectionPrincipal : (collectionId : Text) -> async ?Principal;
        queryVectorSimilarity : (
            collectionId : Text,
            queryText : Text,
            limit : ?Nat,
        ) -> async ?{
            matches : [{
                score : Float;
                document_id : Text;
                chunk_id : Text;
                document_title : ?Text;
                chunk_text : ?Text;
            }];
        };

        // Utility Functions
        getDocumentIdFromVector : (collectionId : Text, vectorId : Text) -> async ?Text;
        getDocumentTitle : (collectionId : Text, documentId : Text) -> async ?Text;
        findDocumentByTitle : (collectionId : Text, title : Text) -> async ?Text;

        // Embedding Operations
        generateEmbeddings : (texts : [Text]) -> async EmbeddingsResponse;
        transform : query (args : TransformArgs) -> async HttpResponsePayload;

        // System Operations
        wallet_receive : () -> async ();
    };
};

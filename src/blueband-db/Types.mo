//Types.mo
import Map "mo:map/Map";
import Text "mo:base/Text";
import Principal "mo:base/Principal";

module {
    // Collection Admins
    public type Role = {
        #GenesisAdmin;
        #Admin;
    };

    // Core identifiers
    public type CollectionId = Text;
    public type DocumentId = Text;
    public type ChunkId = Text;
    public type VectorId = Text;

    // Semantic chunk that generated the embedding
    public type SemanticChunk = {
        id : ChunkId;
        text : Text; // The actual text content that generated the embedding
        position : Nat; // Position in the original document
    };

    // Document source information
    public type DocumentMetadata = {
        id : DocumentId;
        title : Text;
        content_type : {
            #PlainText;
            #ExternalLink;
            #Custom : Text;
        };
        source_url : ?Text; // Optional reference to external source
        timestamp : Int; // Creation timestamp
        total_chunks : Nat; // Number of semantic chunks in this document
        size : Nat; // Total document size in bytes/chars
        is_embedded : Bool; // Whether vectors have been generated
    };

    // Vector with clear connection to semantic chunk
    public type Vector = {
        id : VectorId;
        document_id : DocumentId;
        chunk_id : ChunkId; // Directly links to the chunk that generated this vector
        embedding : [Float]; // The vector embedding
        norm : Float; // Pre-calculated vector norm for efficiency
    };

    // Collection information
    public type Collection = {
        bucket : actor {
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
        var size : Nat;
        var cycle_balance : Nat;
        admins : Map.Map<Principal, Role>;
        token_canister : ?Principal;
        voting_threshold : Nat; // Percentage (0-100)
        quorum_threshold : Nat; // Percentage (0-100)
        expiry_time : Nat; // Document expiry in nanoseconds
    };

    // Canister state types
    public type BucketState = {
        documents : Map.Map<DocumentId, DocumentMetadata>;
        chunks : Map.Map<ChunkId, SemanticChunk>;
        vectors : Map.Map<VectorId, Vector>;
        vector_to_document : Map.Map<VectorId, DocumentId>; // For faster lookups
        chunk_to_document : Map.Map<ChunkId, DocumentId>; // For faster lookups
    };

    public type Database = {
        collections : Map.Map<CollectionId, Collection>;
    };

    // Proposal types for governance
    public type ProposalId = Text;

    public type ProposalStatus = {
        #Active;
        #Approved;
        #Rejected;
        #Executed;
        #Expired;
    };

    public type DocumentProposal = {
        id : ProposalId;
        collectionId : CollectionId;
        documentId : DocumentId;
        proposer : Principal;
        description : ?Text;
        votesFor : Nat;
        votesAgainst : Nat;
        voters : [Principal];
        status : ProposalStatus;
        created : Int;
        expires : Int;
    };

    public type CollectionConfig = {
        token_canister : ?Principal;
        voting_threshold : Nat; // Percentage (0-100)
        quorum_threshold : Nat; // Percentage (0-100)
        expiry_time : Nat; // Document expiry in nanoseconds
    };

    public type TransferError = {
        #BadFee : { expected_fee : Nat };
        #BadBurn : { min_burn_amount : Nat };
        #InsufficientFunds : { balance : Nat };
        #TooOld;
        #CreatedInFuture : { ledger_time : Nat64 };
        #Duplicate : { duplicate_of : Nat };
        #TemporarilyUnavailable;
        #GenericError : { error_code : Nat; message : Text };
    };

    // Empty state initializers
    public func emptyBucket() : BucketState {
        {
            documents = Map.new<DocumentId, DocumentMetadata>();
            chunks = Map.new<ChunkId, SemanticChunk>();
            vectors = Map.new<VectorId, Vector>();
            vector_to_document = Map.new<VectorId, DocumentId>();
            chunk_to_document = Map.new<ChunkId, DocumentId>();
        };
    };

    public func emptyDatabase() : Database {
        {
            collections = Map.new<CollectionId, Collection>();
        };
    };
};

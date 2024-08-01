import Map "mo:map/Map";
module {
    public type DocumentId = Text;
    public type VectorId = Text;
    public type ChunkId = Nat;

    public type DocumentMetadata = {
        id : DocumentId;
        name : Text;
        chunkStartId : ChunkId;
        chunkEndId : ChunkId;
        chunkCount : Nat;
        size : Nat;
        isEmbedded : Bool;
    };

    public type Vector = {
        id : VectorId;
        documentId : DocumentId;
        startPos : Nat;
        endPos : Nat;
        vector : [Float];
    };

    public type CanisterState = {
        metadatas : Map.Map<DocumentId, DocumentMetadata>;
        chunks : Map.Map<ChunkId, Blob>;
        vectors : Map.Map<VectorId, Vector>;
    };

    public func empty() : CanisterState {
        {
            metadatas = Map.new<DocumentId, DocumentMetadata>();
            chunks = Map.new<ChunkId, Blob>();
            vectors = Map.new<VectorId, Vector>();
        };
    };

};

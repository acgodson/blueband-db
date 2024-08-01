import Buckets "./bucket/Bucket";
import Map "mo:map/Map";

module BlueBandTypes {

    public type Bucket = Buckets.Bucket;

    public type DocumentId = Text;
    public type VectorId = Text;
    public type ChunkId = Nat;

    public type CollectionId = Text;
    public type VectorStore = [Vector];
    public type MetadataList = [DocumentMetadata];

    public type TextChunk = {
        text : Text;
        startPos : Nat;
        endPos : Nat;
    };

    public type Collection = {
        bucket : Bucket;
        var size : Nat;
        var cycle_balance : Nat;
    };

    public type Vector = {
        id : VectorId;
        documentId : DocumentId;
        startPos : Nat;
        endPos : Nat;
        vector : [Float];
    };

    public type DocumentMetadata = {
        id : DocumentId;
        name : Text;
        chunkStartId : ChunkId;
        chunkEndId : ChunkId;
        chunkCount : Nat;
        size : Nat;
        isEmbedded : Bool;
    };

    public type Database = {
        collections : Map.Map<CollectionId, Collection>;
        //TODO: users
    };

    public func empty() : Database {
        {
            collections = Map.new<CollectionId, Collection>();
        };
    };

};

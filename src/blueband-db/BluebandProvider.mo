import Buckets "./bucket/Bucket";

module BluebandProvider {

    public type Bucket = Buckets.Bucket;
    public type CollectionId = Text;
    public type DocumentId = Text;
    public type VectorId = Text;
    public type Title = Text;
    public type Content = Text;

    public type DocumentMetadata = {
        id : DocumentId;
        title : Title;
        chunkCount : Nat;
        totalSize : Nat;
        isEmbedded : Bool;
    };

    public type Vector = {
        id : VectorId;
        documentId : DocumentId;
        start : Nat;
        end : Nat;
        embedding : [Float];
    };

    public type Collection = {
        bucket : Bucket;
        var size : Nat;
        var cycle_balance : Nat;
    };

    public type VectorStore = [Vector];
    public type MetadataList = [DocumentMetadata];

    public type BluebandProvider = actor {

        index : (collectionId : CollectionId) -> async ?{
            items : VectorStore;
        };

        addDocument : (collectionId : CollectionId, title : Title, content : Content) -> async ?({
            collection : ?Principal;
            documentId : ?DocumentId;
        });

        putVector : (
            collectionId : CollectionId,
            documentId : DocumentId,
            vectorId : VectorId,
            start : Nat,
            end : Nat,
            vector : [Float],
        ) -> async Text;

        endUpdate : (collectionId : CollectionId, documentId : DocumentId) -> async ();

        getCollectionPrincipal : (collectionId : CollectionId) -> async ?Principal;

        getMetadataList : (collectionId : CollectionId) -> async ?MetadataList;

        getChunks : (collectionId : CollectionId, documentId : DocumentId) -> async ?Text;

        getMetadata : (collectionId : CollectionId, documentId : DocumentId) -> async ?DocumentMetadata;

        getDocumentId : (collectionId : CollectionId, vectorId : VectorId) -> async ?DocumentId;

        documentIDToTitle : (collectionId : CollectionId, documentId : DocumentId) -> async ?Title;

        titleToDocumentID : (collectionId : CollectionId, title : Title) -> async ?DocumentId;

        wallet_receive : () -> async ();
    };
};

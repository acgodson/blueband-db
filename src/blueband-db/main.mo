import Debug "mo:base/Debug";
import Cycles "mo:base/ExperimentalCycles";
import Principal "mo:base/Principal";

import Storage "./bucket/BucketProvider";
import Types "./Types";

shared ({ caller }) actor class Blueband() {

    public type State = Types.Database;

    type Collection = Storage.Collection;
    type VectorStore = Storage.VectorStore;
    type MetadataList = Storage.MetadataList;
    type DocumentMetadata = Storage.DocumentMetadata;

    stable var state = Types.empty();

    //////////////////////////
    // Storage
    //////////////////////////
    // if collectionId exists it will add it to the collection or create a new collection
    public func addDocument(collectionId : Text, title : Text, content : Text) : async ?({
        collection : ?Principal;
        documentId : ?Text;
    }) {
        let result = await Storage.addDcoument(collectionId, state.collections, title, content);
        var response : ?{ collection : ?Principal; documentId : ?Text } = null;
        switch (result) {
            case (b : { collection : ?Principal; documentId : ?Text }) {
                response := ?b;
            };
        };
        return response;
    };

    public func putVector(
        collectionId : Text,
        doc_id : Text,
        vector_id : Text,
        start : Nat,
        end : Nat,
        vector : [Float],
    ) : async Text {
        let result = await Storage.addVector(
            collectionId,
            state.collections,
            doc_id,
            vector_id,
            start,
            end,
            vector,
        );
        result;
    };

    public func endUpdate(collectionId : Text, documentId : Text) : async () {
        let result = await Storage.endVectorUpdate(collectionId, state.collections, documentId);
        result;
    };

    public func getCollectionPrincipal(collectionId : Text) : async ?Principal {
        let result = await Storage.getBucketPrincipal(collectionId, state.collections);
        result;
    };

    public func getIndex(collectionId : Text) : async ?{ items : VectorStore } {
        return await Storage.listVectors(collectionId, state.collections);
    };

    public func getMetadataList(collectionId : Text) : async ?MetadataList {
        return await Storage.listDocumentMetadata(state.collections, collectionId);
    };

    public func getChunks(collectionId : Text, documentId : Text) : async ?Text {
        return await Storage.getChunks(state.collections, collectionId, documentId);
    };

    public func getMetadata(collectionId : Text, documentId : Text) : async ?DocumentMetadata {
        return await Storage.getMetadata(state.collections, collectionId, documentId);
    };

    //////////////////////////
    //Query Utils
    //////////////////////////

    public shared func getDocumentId(collectionId : Text, vectorId : Text) : async ?Text {
        return await Storage.getDocumentIdByVectorId(state.collections, collectionId, vectorId);
    };

    public shared func documentIDToTitle(collectionId : Text, documentId : Text) : async ?Text {
        return await Storage.titleToDocumentID(state.collections, collectionId, documentId);
    };

    public shared func titleToDocumentID(collectionId : Text, title : Text) : async ?Text {
        return await Storage.documentIdToTitle(state.collections, collectionId, title);
    };

    // Add Cycles Functions
    public shared ({ caller = caller }) func wallet_receive() : async () {
        ignore Cycles.accept<system>(Cycles.available());
        Debug.print("intital cycles deposited by " # debug_show (caller));
    };

};

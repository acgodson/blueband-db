//collection/CollectionManager.mo
import Principal "mo:base/Principal";
import Text "mo:base/Text";
import Nat "mo:base/Nat";

import Map "mo:map/Map";
import { thash } "mo:map/Map";

import Types "../Types";
import Storage "../bucket/BucketProvider";

module {
    type CollectionId = Types.CollectionId;

    // Create a new collection with admin privileges
    public func createCollection(
        collectionId : Text,
        caller : Principal,
        token_canister : ?Principal,
        voting_threshold : ?Nat,
        quorum_threshold : ?Nat,
        expiry_time : ?Nat,
        collections : Map.Map<CollectionId, Types.Collection>,
    ) : async ?Principal {
        await Storage.createCollection(
            collectionId,
            caller, // Creator becomes Genesis Admin
            token_canister,
            voting_threshold,
            quorum_threshold,
            expiry_time,
            collections,
        );
    };

    // Add a new admin to a collection
    public func addCollectionAdmin(
        caller : Principal,
        collectionId : Text,
        newAdmin : Principal,
        collections : Map.Map<CollectionId, Types.Collection>,
    ) : async Bool {
        await Storage.addAdmin(
            caller,
            newAdmin,
            collectionId,
            collections,
        );
    };

    // Remove an admin from a collection
    public func removeCollectionAdmin(
        caller : Principal,
        collectionId : Text,
        adminToRemove : Principal,
        collections : Map.Map<CollectionId, Types.Collection>,
    ) : async Bool {
        await Storage.removeAdmin(
            caller,
            adminToRemove,
            collectionId,
            collections,
        );
    };

    // Check if a principal is an admin for a collection
    public func isCollectionAdmin(
        caller : Principal,
        collectionId : Text,
        collections : Map.Map<CollectionId, Types.Collection>,
    ) : Bool {
        switch (Map.get(collections, thash, collectionId)) {
            case null { false };
            case (?collection) {
                Storage.isAdmin(caller, collection);
            };
        };
    };

    // Get the collection canister principal
    public func getCollectionPrincipal(
        collectionId : Text,
        collections : Map.Map<CollectionId, Types.Collection>,
    ) : async ?Principal {
        await Storage.getBucketPrincipal(collectionId, collections);
    };
};

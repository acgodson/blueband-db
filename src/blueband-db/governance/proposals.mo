//governance/proposals.mo
import Principal "mo:base/Principal";
import Text "mo:base/Text";
import Nat "mo:base/Nat";
import Int "mo:base/Int";
import Time "mo:base/Time";
import Array "mo:base/Array";
import Buffer "mo:base/Buffer";
import Result "mo:base/Result";

import Map "mo:map/Map";
import { thash } "mo:map/Map";

import Types "../Types";
import TokenInterface "./TokenInterface";

module {
    type DocumentProposal = Types.DocumentProposal;
    type ProposalId = Types.ProposalId;
    type ProposalStatus = Types.ProposalStatus;
    type CollectionId = Types.CollectionId;
    type DocumentId = Types.DocumentId;
    type Collection = Types.Collection;

    // Constants
    let DEFAULT_PROPOSAL_EXPIRY : Nat = 604_800_000_000_000; // 7 days in nanoseconds

    // Generate a unique proposal ID
    public func generateProposalId(collectionId : CollectionId, documentId : DocumentId) : ProposalId {
        collectionId # "_" # documentId # "_" # Int.toText(Time.now());
    };

    // Create a new embedding proposal
    public func createProposal(
        collectionId : CollectionId,
        documentId : DocumentId,
        proposer : Principal,
        description : ?Text,
        proposals : Map.Map<ProposalId, DocumentProposal>,
    ) : ProposalId {
        let proposalId = generateProposalId(collectionId, documentId);

        let proposal : DocumentProposal = {
            id = proposalId;
            collectionId = collectionId;
            documentId = documentId;
            proposer = proposer;
            description = description;
            votesFor = 0;
            votesAgainst = 0;
            voters = [];
            status = #Active;
            created = Time.now();
            expires = Time.now() + DEFAULT_PROPOSAL_EXPIRY;
        };

        ignore Map.put(proposals, thash, proposalId, proposal);
        proposalId;
    };

    // Get a proposal by ID
    public func getProposal(
        proposalId : ProposalId,
        proposals : Map.Map<ProposalId, DocumentProposal>,
    ) : ?DocumentProposal {
        Map.get(proposals, thash, proposalId);
    };

    // List proposals for a collection, optionally filtered by status
    public func listProposals(
        collectionId : CollectionId,
        status : ?ProposalStatus,
        proposals : Map.Map<ProposalId, DocumentProposal>,
    ) : [DocumentProposal] {
        let results = Buffer.Buffer<DocumentProposal>(10);

        for ((_, proposal) in Map.entries(proposals)) {
            if (proposal.collectionId == collectionId) {
                switch (status) {
                    case (?s) {
                        if (proposal.status == s) {
                            results.add(proposal);
                        };
                    };
                    case (null) {
                        results.add(proposal);
                    };
                };
            };
        };

        Buffer.toArray(results);
    };

    // Check if a proposal exists for a document
    public func hasActiveProposal(
        collectionId : CollectionId,
        documentId : DocumentId,
        proposals : Map.Map<ProposalId, DocumentProposal>,
    ) : Bool {
        for ((_, proposal) in Map.entries(proposals)) {
            if (
                proposal.collectionId == collectionId and
                proposal.documentId == documentId and
                proposal.status == #Active
            ) {
                return true;
            };
        };

        false;
    };

    // Vote on a proposal
    public func vote(
        proposalId : ProposalId,
        voter : Principal,
        inFavor : Bool,
        votingPower : Nat,
        collection : Collection,
        proposals : Map.Map<ProposalId, DocumentProposal>,
    ) : async Result.Result<ProposalStatus, Text> {
        switch (Map.get(proposals, thash, proposalId)) {
            case (null) {
                #err("Proposal not found");
            };
            case (?proposal) {
                // Check if proposal is still active
                if (proposal.status != #Active) {
                    return #err("Proposal is no longer active");
                };

                // Check if already voted
                if (Array.find<Principal>(proposal.voters, func(p) { p == voter }) != null) {
                    return #err("Already voted on this proposal");
                };

                // Update votes
                let voters = Array.append<Principal>(proposal.voters, [voter]);

                let updatedVotesFor = if (inFavor) {
                    proposal.votesFor + votingPower;
                } else { proposal.votesFor };
                let updatedVotesAgainst = if (inFavor) { proposal.votesAgainst } else {
                    proposal.votesAgainst + votingPower;
                };
                let totalVotes = updatedVotesFor + updatedVotesAgainst;

                // Check for expiration
                let now = Time.now();
                if (now > proposal.expires) {
                    let updatedProposal = {
                        id = proposal.id;
                        collectionId = proposal.collectionId;
                        documentId = proposal.documentId;
                        proposer = proposal.proposer;
                        description = proposal.description;
                        votesFor = updatedVotesFor;
                        votesAgainst = updatedVotesAgainst;
                        voters = voters;
                        status = #Expired;
                        created = proposal.created;
                        expires = proposal.expires;
                    };

                    ignore Map.put(proposals, thash, proposalId, updatedProposal);
                    return #ok(#Expired);
                };

                // Check if quorum and threshold reached
                let hasQuorum = await TokenInterface.hasReachedQuorum(
                    totalVotes,
                    collection.quorum_threshold,
                    collection.token_canister,
                );

                let hasThreshold = TokenInterface.hasReachedThreshold(
                    updatedVotesFor,
                    totalVotes,
                    collection.voting_threshold,
                );

                // Auto-approve if both quorum and threshold met
                let newStatus = if (hasQuorum and hasThreshold) {
                    #Approved;
                } else {
                    // Check if mathematically impossible to pass
                    let impossible = await TokenInterface.isImpossibleToPass(
                        updatedVotesFor,
                        updatedVotesAgainst,
                        collection.voting_threshold,
                        collection.token_canister,
                    );

                    if (impossible) {
                        #Rejected;
                    } else {
                        #Active;
                    };
                };

                // Update proposal
                let updatedProposal : DocumentProposal = {
                    id = proposal.id;
                    collectionId = proposal.collectionId;
                    documentId = proposal.documentId;
                    proposer = proposal.proposer;
                    description = proposal.description;
                    votesFor = updatedVotesFor;
                    votesAgainst = updatedVotesAgainst;
                    voters = voters;
                    status = newStatus;
                    created = proposal.created;
                    expires = proposal.expires;
                };

                ignore Map.put(proposals, thash, proposalId, updatedProposal);
                #ok(newStatus);
            };
        };
    };

    // Mark a proposal as executed
    public func markExecuted(
        proposalId : ProposalId,
        proposals : Map.Map<ProposalId, DocumentProposal>,
    ) : Bool {
        switch (Map.get(proposals, thash, proposalId)) {
            case (null) {
                false;
            };
            case (?proposal) {
                if (proposal.status != #Approved) {
                    return false;
                };

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
                true;
            };
        };
    };

    // Process expired proposals (could be called periodically)
    public func processExpiredProposals(
        proposals : Map.Map<ProposalId, DocumentProposal>
    ) {
        let now = Time.now();

        for ((id, proposal) in Map.entries(proposals)) {
            if (proposal.status == #Active and now > proposal.expires) {
                let updatedProposal : DocumentProposal = {
                    id = proposal.id;
                    collectionId = proposal.collectionId;
                    documentId = proposal.documentId;
                    proposer = proposal.proposer;
                    description = proposal.description;
                    votesFor = proposal.votesFor;
                    votesAgainst = proposal.votesAgainst;
                    voters = proposal.voters;
                    status = #Expired;
                    created = proposal.created;
                    expires = proposal.expires;
                };

                ignore Map.put(proposals, thash, id, updatedProposal);
            };
        };
    };
};

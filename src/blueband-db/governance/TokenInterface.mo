// governance/TokenInterface.mo
import Principal "mo:base/Principal";
import Debug "mo:base/Debug";
import Error "mo:base/Error";
import Nat "mo:base/Nat";
import Nat8 "mo:base/Nat8";
import Array "mo:base/Array";
import Result "mo:base/Result";
import Text "mo:base/Text";
import Iter "mo:base/Iter";
import ICRC1 "mo:icrc1-types";

module {

    // Get the voting power (token balance) for a principal
    public func getVotingPower(
        principal : Principal,
        tokenCanister : ?Principal,
    ) : async Nat {
        switch (tokenCanister) {
            case (null) {
                return 0; // No token canister configured
            };
            case (?canister) {
                try {
                    let token : ICRC1.Service = actor (Principal.toText(canister));

                    let balance = await token.icrc1_balance_of({
                        owner = principal;
                        subaccount = null;
                    });
                    return balance;
                } catch (err) {
                    Debug.print("Error getting token balance: " # Error.message(err));
                    return 0;
                };
            };
        };
    };

    // Get total supply to calculate quorum requirements
    public func getTotalSupply(
        tokenCanister : ?Principal
    ) : async Nat {
        switch (tokenCanister) {
            case (null) {
                return 0; // No token canister configured
            };
            case (?canister) {
                try {
                    let token : ICRC1.Service = actor (Principal.toText(canister));
                    let totalSupply = await token.icrc1_total_supply();
                    return totalSupply;
                } catch (err) {
                    Debug.print("Error getting total supply: " # Error.message(err));
                    return 0;
                };
            };
        };
    };

    // Check if proposal has reached quorum threshold
    public func hasReachedQuorum(
        totalVotes : Nat,
        quorumThreshold : Nat, // Percentage (0-100)
        tokenCanister : ?Principal,
    ) : async Bool {
        let totalSupply = await getTotalSupply(tokenCanister);

        if (totalSupply == 0) {
            return false;
        };

        // Calculate percentage of total supply that has voted
        let participationPercentage = (totalVotes * 100) / totalSupply;
        return participationPercentage >= quorumThreshold;
    };

    // Check if proposal has reached approval threshold based on votes cast
    public func hasReachedThreshold(
        votesFor : Nat,
        totalVotes : Nat,
        thresholdPercentage : Nat // Percentage (0-100)
    ) : Bool {
        if (totalVotes == 0) {
            return false;
        };

        let approvalPercentage = (votesFor * 100) / totalVotes;
        return approvalPercentage >= thresholdPercentage;
    };

    // Check if it's mathematically impossible for proposal to pass
    public func isImpossibleToPass(
        votesFor : Nat,
        votesAgainst : Nat,
        thresholdPercentage : Nat,
        tokenCanister : ?Principal,
    ) : async Bool {
        let totalVoted = votesFor + votesAgainst;
        let totalSupply = await getTotalSupply(tokenCanister);

        if (totalSupply == 0 or totalVoted == 0) {
            return false;
        };

        // Calculate maximum possible votes for
        let remainingVotes = if (totalVoted >= totalSupply) {
            0;
        } else {
            Nat.sub(totalSupply, totalVoted);
        };
        let maxPossibleVotesFor = votesFor + remainingVotes;

        // Calculate maximum possible percentage
        let maxPossiblePercentage = (maxPossibleVotesFor * 100) / (maxPossibleVotesFor + votesAgainst);

        return maxPossiblePercentage < thresholdPercentage;
    };

    // Verify token implements necessary ICRC-1 functions
    public func verifyTokenSupport(
        tokenCanister : Principal
    ) : async Bool {
        try {
            let token : ICRC1.Service = actor (Principal.toText(tokenCanister));

            // Try basic ICRC-1 functions
            let _ = await token.icrc1_name();
            let _ = await token.icrc1_symbol();
            let _ = await token.icrc1_decimals();
            let _ = await token.icrc1_total_supply();
            let _ = await token.icrc1_balance_of({
                owner = Principal.fromText("aaaaa-aa");
                subaccount = null;
            });

            return true;
        } catch (err) {
            Debug.print("Token verification failed: " # Error.message(err));
            return false;
        };
    };

    // Get basic token metadata (useful for UI display)
    public func getTokenMetadata(
        tokenCanister : Principal
    ) : async Result.Result<{ name : Text; symbol : Text; decimals : Nat8 }, Text> {
        try {
            let token : ICRC1.Service = actor (Principal.toText(tokenCanister));

            let name = await token.icrc1_name();
            let symbol = await token.icrc1_symbol();
            let decimals = await token.icrc1_decimals();

            #ok({
                name = name;
                symbol = symbol;
                decimals = decimals;
            });
        } catch (err) {
            #err("Error getting token metadata: " # Error.message(err));
        };
    };

    // Format token amount with proper decimal places for display
    public func formatTokenAmount(
        amount : Nat,
        tokenCanister : Principal,
    ) : async Text {
        try {
            let token : ICRC1.Service = actor (Principal.toText(tokenCanister));
            let decimals = await token.icrc1_decimals();

            if (decimals == 0) {
                return Nat.toText(amount);
            };

            let divisor = 10 ** Nat8.toNat(decimals);
            let wholePart = amount / divisor;
            let fractionalPart = amount % divisor;

            // Format with proper decimal places
            let fractionalStr = Nat.toText(fractionalPart);
            let padLength = if (Nat8.toNat(decimals) > fractionalStr.size()) {
                Nat.sub(Nat8.toNat(decimals), fractionalStr.size());
            } else {
                0;
            };
            let zeros = Array.tabulate<Text>(
                padLength,
                func(_ : Nat) : Text { "0" },
            );

            let padded = Text.join("", Iter.fromArray(zeros)) # fractionalStr;

            return Nat.toText(wholePart) # "." # padded;
        } catch (_) {
            return Nat.toText(amount);
        };
    };
};

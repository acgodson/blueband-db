import Nat8 "mo:base/Nat8";
import Principal "mo:base/Principal";
import HashMap "mo:base/HashMap";
import Iter "mo:base/Iter";
import ICRC1 "mo:icrc1-types";

actor class TestToken() : async ICRC1.Service {
    private stable var balances : [(Principal, Nat)] = [];
    private stable var totalSupplyVar : Nat = 0;
    private let FEE : Nat = 0;

    private let balanceMap = HashMap.fromIter<Principal, Nat>(
        balances.vals(),
        10,
        Principal.equal,
        Principal.hash,
    );

    // ICRC-1 standard methods
    public query func icrc1_name() : async Text { "Test Token" };
    public query func icrc1_symbol() : async Text { "TEST" };
    public query func icrc1_decimals() : async Nat8 { 8 };
    public query func icrc1_fee() : async Nat { FEE };
    public query func icrc1_total_supply() : async Nat { totalSupplyVar };
    public query func icrc1_metadata() : async ICRC1.Metadata {
        [
            ("icrc1:fee", #Nat(FEE)),
            ("icrc1:name", #Text("Test Token")),
            ("icrc1:symbol", #Text("TEST")),
            ("icrc1:decimals", #Nat(8)),
        ]
    };
    public query func icrc1_minting_account() : async ?ICRC1.Account { null };
    public query func icrc1_supported_standards() : async ICRC1.SupportedStandards {
        [
            { name = "ICRC-1"; url = "https://github.com/dfinity/ICRC-1" },
            { name = "ICRC-2"; url = "https://github.com/dfinity/ICRC-2" },
        ]
    };

    public query func icrc1_balance_of(account : ICRC1.Account) : async Nat {
        switch (balanceMap.get(account.owner)) {
            case (?balance) { balance };
            case null { 0 };
        }
    };

    public shared ({ caller }) func icrc1_transfer(args : ICRC1.TransferArgs) : async ICRC1.TransferResult {
        let from = { owner = caller; subaccount = args.from_subaccount };
        let to = args.to;
        let amount = args.amount;

        switch (balanceMap.get(from.owner)) {
            case (?fromBalance) {
                if (fromBalance < amount + FEE) {
                    return #Err(#InsufficientFunds { balance = fromBalance });
                };

                balanceMap.put(from.owner, fromBalance - amount - FEE);

                let toBalance = switch (balanceMap.get(to.owner)) {
                    case (?balance) { balance };
                    case null { 0 };
                };

                balanceMap.put(to.owner, toBalance + amount);
                #Ok(0)
            };
            case null {
                #Err(#InsufficientFunds { balance = 0 })
            };
        }
    };


    // ICRC-2 standard method
    public shared ({ caller }) func icrc2_transfer(
        args : {
            from_subaccount : ?[Nat8];
            to : { owner : Principal; subaccount : ?[Nat8] };
            amount : Nat;
            fee : ?Nat;
            memo : ?[Nat8];
            created_at_time : ?Nat64;
        }
    ) : async {
        #Ok : Nat;
        #Err : {
            #BadFee : { expected_fee : Nat };
            #BadBurn : { min_burn_amount : Nat };
            #InsufficientFunds : { balance : Nat };
            #TooOld;
            #CreatedInFuture : { ledger_time : Nat64 };
            #Duplicate : { duplicate_of : Nat };
            #TemporarilyUnavailable;
            #GenericError : { error_code : Nat; message : Text };
        };
    } {
        let sender = caller;

        switch (balanceMap.get(sender)) {
            case (?fromBalance) {
                if (fromBalance < args.amount) {
                    return #Err(#InsufficientFunds { balance = fromBalance });
                };

                balanceMap.put(sender, fromBalance - args.amount);

                let toBalance = switch (balanceMap.get(args.to.owner)) {
                    case (?balance) { balance };
                    case null { 0 };
                };

                balanceMap.put(args.to.owner, toBalance + args.amount);

                #Ok(0) // Successful transfer
            };
            case null {
                #Err(#InsufficientFunds { balance = 0 });
            };
        };
    };

    // Test helper functions
    public shared func mint(to : Principal, amount : Nat) : async () {
        let balance = switch (balanceMap.get(to)) {
            case (?b) { b };
            case null { 0 };
        };
        balanceMap.put(to, balance + amount);
        totalSupplyVar += amount;
    };

    // System functions for stable storage
    system func preupgrade() {
        balances := Iter.toArray(balanceMap.entries());
    };

    system func postupgrade() {
        balances := [];
    };
};

import Types "./Types";

module {
    public type Vector = Types.Vector;

    public type MetadataFilter = {
        document_ids : ?[Text];
        chunk_ids : ?[Text];
        limit : ?Nat64;
    };

    public type ScoredMatch = {
        score : Float;
        document_id : Text;
        chunk_id : Text;
    };

    public type QueryResult = {
        matches : [ScoredMatch];
    };

    // FIXED: Use Nat64 to align with Rust u64
    public type ScaledEmbedding = {
        values : [[Nat64]]; // Changed from [[Nat]] to [[Nat64]]
        norms : [Nat64]; // Changed from [Nat] to [Nat64]
    };

    public type FloatEmbedding = {
        embeddings : [[Float]];
        norm_values : [Float];
    };

    public type EmbeddingResult = {
        #Scaled : ScaledEmbedding;
        #Float : FloatEmbedding;
    };

    public type ComputeProvider = actor {
        generate_embeddings : ([Text], Text, Bool) -> async {
            #Ok : EmbeddingResult;
            #Err : Text;
        };
        query_text : (Text, Text, ?MetadataFilter) -> async {
            #Ok : QueryResult;
            #Err : Text;
        };
        invalidate_cache : (Text) -> async ();
        get_cache_stats : () -> async {
            cache_size : Nat64; // Changed from Nat to Nat64
            hits : Nat64; // Changed from Nat to Nat64
            misses : Nat64; // Changed from Nat to Nat64
            memory_usage : Nat64; // Changed from Nat to Nat64
        };
        wallet_receive : () -> async ();
    };
};

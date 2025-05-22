//embeddings.mo
import Debug "mo:base/Debug";
import Error "mo:base/Error";
import Nat "mo:base/Nat";
import Text "mo:base/Text";
import Time "mo:base/Time";
import Int "mo:base/Int";
import Blob "mo:base/Blob";
import Iter "mo:base/Iter";
import Nat8 "mo:base/Nat8";
import Nat64 "mo:base/Nat64";
import Float "mo:base/Float";
import Int64 "mo:base/Int64";
import ComputeProvider "./ComputeProvider";

module {

    private let SCALE_FACTOR : Float = 1000000.0;
    private let OFFSET_VALUE : Float = 10.0;
    // Response types - simplified to keep raw text
    public type EmbeddingsResponse = {
        #success : {
            raw_response : Text; // Keep the entire response as raw text
        };
        #rate_limited : Text;
        #error : Text;
    };

    // Request types
    private type CreateEmbeddingRequest = {
        input : [Text];
        model : Text;
    };

    public type HttpHeader = {
        name : Text;
        value : Text;
    };

    public type HttpResponsePayload = {
        status : Nat;
        headers : [HttpHeader];
        body : [Nat8];
    };

    public type Request = {
        url : Text;
        max_response_bytes : ?Nat64;
        headers : [HttpHeader];
        body : ?[Nat8];
        method : HttpMethod;
        transform : ?TransformContext;
    };

    public type HttpMethod = {
        #get;
        #post;
        #head;
    };

    public type TransformArgs = {
        response : HttpResponsePayload;
        context : Blob;
    };

    public type TransformContext = {
        function : shared query TransformArgs -> async HttpResponsePayload;
        context : Blob;
    };

    type ICProvider = actor {
        http_request : Request -> async HttpResponsePayload;
    };

    // Default cycle configuration values as constants (static)
    private let DEFAULT_BASE_COST : Nat = 3_000_000_000; // 3B cycles base cost
    private let DEFAULT_COST_PER_BYTE : Nat = 400_000; // 400K cycles per byte
    private let DEFAULT_BUFFER_PERCENTAGE : Nat = 20; // 20% buffer

    public class Embeddings(compute_canister : ComputeProvider.ComputeProvider) {
        // Calculate cycles needed for the compute request
        private func calculateCycles(texts : [Text]) : Nat {
            let textCost = texts.size() * DEFAULT_COST_PER_BYTE;
            let subtotal = DEFAULT_BASE_COST + textCost;
            let buffer = (subtotal * DEFAULT_BUFFER_PERCENTAGE) / 100;
            subtotal + buffer;
        };

        // Main embeddings creation function
        public func createEmbeddings(inputs : [Text], proxy_url : Text) : async EmbeddingsResponse {
            let startTime = Time.now();
            Debug.print("Starting embedding generation for " # Nat.toText(inputs.size()) # " inputs");

            try {
                // Calculate and add cycles
                let requiredCycles = calculateCycles(inputs);
                let result = await (with cycles = requiredCycles) compute_canister.generate_embeddings(inputs, proxy_url, true);
                Debug.print("Response received in " # Int.toText((Time.now() - startTime) / 1000000) # " ms");

                switch (result) {
                    case (#Ok(embeddings)) {
                        Debug.print("Successfully received embeddings response");
                        let raw_response = formatEmbeddingsResponse(embeddings);
                        #success({ raw_response = raw_response });
                    };
                    case (#Err(error)) {
                        if (Text.contains(error, #text "rate limit")) {
                            Debug.print("Rate limit error encountered");
                            #rate_limited("The embeddings API returned a rate limit error: " # error);
                        } else {
                            Debug.print("API error: " # error);
                            #error("The embeddings API returned an error: " # error);
                        };
                    };
                };
            } catch (error) {
                Debug.print("Request failed: " # Error.message(error));
                #error("Request failed: " # Error.message(error));
            };
        };

        // Format embeddings response from compute canister
        // The compute canister returns either:
        // 1. Scaled integers (when use_scaled=true):
        //    - Values are scaled by SCALE_FACTOR and offset by OFFSET_VALUE
        //    - Format: [scaled_int1,scaled_int2,...],scaled_norm
        // 2. Raw floats (when use_scaled=false):
        //    - Values are direct float embeddings
        //    - Format: [float1,float2,...],norm
        public func formatEmbeddingsResponse(result : ComputeProvider.EmbeddingResult) : Text {
            switch (result) {
                case (#Scaled(scaled)) {
                    var response = "";
                    for (i in Iter.range(0, scaled.values.size() - 1)) {
                        if (i > 0) {
                            response #= "|";
                        };
                        let embedding = scaled.values[i];
                        let norm = scaled.norms[i];
                        response #= "[";
                        for (j in Iter.range(0, embedding.size() - 1)) {
                            if (j > 0) {
                                response #= ",";
                            };
                            // FIXED: Convert Nat64 to Float properly
                            // embedding[j] is now Nat64, so we convert via Int64
                            let floatValue = Float.fromInt64(Int64.fromNat64(embedding[j])) / SCALE_FACTOR - OFFSET_VALUE;
                            response #= Float.toText(floatValue);
                        };
                        // FIXED: Convert scaled norm (Nat64) back to float
                        let floatNorm = Float.fromInt64(Int64.fromNat64(norm)) / SCALE_FACTOR - OFFSET_VALUE;
                        response #= "]," # Float.toText(floatNorm);
                    };
                    response;
                };
                case (#Float(floatEmb)) {
                    var response = "";
                    for (i in Iter.range(0, floatEmb.embeddings.size() - 1)) {
                        if (i > 0) {
                            response #= "|";
                        };
                        let embedding = floatEmb.embeddings[i];
                        let norm = floatEmb.norm_values[i];
                        response #= "[";
                        for (j in Iter.range(0, embedding.size() - 1)) {
                            if (j > 0) {
                                response #= ",";
                            };
                            response #= Float.toText(embedding[j]);
                        };
                        response #= "]," # Float.toText(norm);
                    };
                    response;
                };
            };
        };

        public func validateEmbeddingResult(result : ComputeProvider.EmbeddingResult) : Bool {
            switch (result) {
                case (#Scaled(scaled)) {
                    // Check that values and norms arrays have same length
                    if (scaled.values.size() != scaled.norms.size()) {
                        return false;
                    };
                    // Check that no embedding array is empty
                    for (embedding in scaled.values.vals()) {
                        if (embedding.size() == 0) {
                            return false;
                        };
                    };
                    true;
                };
                case (#Float(floatEmb)) {
                    // Check that embeddings and norm_values arrays have same length
                    if (floatEmb.embeddings.size() != floatEmb.norm_values.size()) {
                        return false;
                    };
                    // Check that no embedding array is empty and no NaN values
                    for (embedding in floatEmb.embeddings.vals()) {
                        if (embedding.size() == 0) {
                            return false;
                        };
                        for (value in embedding.vals()) {
                            if (Float.isNaN(value)) {
                                return false;
                            };
                        };
                    };
                    // Check norm values for NaN
                    for (norm in floatEmb.norm_values.vals()) {
                        if (Float.isNaN(norm) or norm <= 0.0) {
                            return false;
                        };
                    };
                    true;
                };
            };
        };
    };

};

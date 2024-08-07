import Debug "mo:base/Debug";
import Error "mo:base/Error";
import Nat "mo:base/Nat";
import Text "mo:base/Text";
import Time "mo:base/Time";

import Int "mo:base/Int";
import Result "mo:base/Result";
import Blob "mo:base/Blob";
import Iter "mo:base/Iter";
import Cycles "mo:base/ExperimentalCycles";
import Nat8 "mo:base/Nat8";
import Nat32 "mo:base/Nat32";
// import { generateRandomID } "./Utils"

module {
    public type EmbeddingsResponse = {
        #success : Text;
        #rate_limited : Text;
        #error : Text;
    };

    private type CreateEmbeddingRequest = {
        input : [Text];
        model : Text;
    };

    public type HttpHeader = {
        name : Text;
        value : Text;
    };

    public type Request = {
        url : Text;
        max_response_bytes : ?Nat64;
        headers : [HttpHeader];
        body : ?[Nat8];
        method : HttpMethod;
        transform : ?TransformContext;
    };

    public type HttpResponsePayload = {
        status : Nat;
        headers : [HttpHeader];
        body : [Nat8];
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

    type EmbeddingData = {
        embedding : [Text];
        index : Nat;
    };

    type ICProvider = actor {
        http_request : Request -> async HttpResponsePayload;
    };

    private type CreateEmbeddingResponse = Text;

    public class OpenAIEmbeddings(apiKey : Text, model : Text) {
        private let API_KEY = apiKey;
        private let MODEL = model;
        private let ENDPOINT = "https://main.d3io3l9ylcsplj.amplifyapp.com/api/proxy";
        public type Transform = shared query TransformArgs -> async HttpResponsePayload;

        public func createEmbeddings(inputs : [Text], transform : Transform) : async EmbeddingsResponse {
            let startTime = Time.now();

            let request : CreateEmbeddingRequest = {
                input = inputs;
                model = MODEL;
            };

            let response = await _createEmbeddingRequest(request, transform, API_KEY, ENDPOINT);
            Debug.print("Response received in " # Int.toText((Time.now() - startTime) / 1000000) # " ms");
            switch (response) {
                case (#ok(result)) {
                    #success(result);
                };
                case (#err(error)) {
                    if (Text.contains(error, #text "rate limit")) {
                        #rate_limited("The embeddings API returned a rate limit error.");
                    } else {
                        #error("The embeddings API returned an error: " # error);
                    };
                };
            };
        };

        private func _createEmbeddingRequest(request : CreateEmbeddingRequest, transform : Transform, api_key : Text, end_point : Text) : async Result.Result<CreateEmbeddingResponse, Text> {
            let requestBody = createRequestBody(request);

            let request_body_as_Blob : Blob = Text.encodeUtf8(requestBody);
            let request_body_as_nat8 : [Nat8] = Blob.toArray(request_body_as_Blob);

            // 2.2.1 Transform context
            let transform_context : TransformContext = {
                function = transform;
                context = Blob.fromArray([]);
            };

            let key = generateIdempotencyKey(request);

            Debug.print("impotency key" # key);
            let request_headers = [
                { name = "Content-Type"; value = "application/json" },
                { name = "Authorization"; value = "Bearer " # api_key },
                {
                    name = "idempotency-key";
                    value = key;
                },

            ];

            let http_request : Request = {
                url = end_point;
                max_response_bytes = null;
                headers = request_headers;
                body = ?request_body_as_nat8;
                method = #post;
                transform = ?transform_context;
            };

            try {

                //3. ADD CYCLES TO PAY FOR HTTP REQUEST
                Cycles.add<system>(21_850_258_000);
                let ic : ICProvider = actor ("aaaaa-aa");
                let httpResponse = await ic.http_request(http_request);
                if (httpResponse.status >= 200 and httpResponse.status < 300) {
                    // Here you would parse the response body
                    Debug.print("Embedding response arrived");
                    // Parse the response body
                    let responseBody : Text = switch (Text.decodeUtf8(Blob.fromArray(httpResponse.body))) {
                        case (null) { "" };
                        case (?y) { y };
                    };
                    Debug.print("this response" # responseBody);
                    // let x = parseEmbeddings(responseBody);
                    #ok(responseBody);
                } else if (httpResponse.status == 429) {
                    #err("Rate limit exceeded");
                } else {
                    #err("HTTP error: " # Nat.toText(httpResponse.status));
                };
            } catch (error) {
                #err("Request failed: " # Error.message(error));
            };
        };
    };

    private func createRequestBody(request : CreateEmbeddingRequest) : Text {
        var inputArray = "[\"" # request.input[0] # "\"";
        for (i in Iter.range(1, request.input.size() - 1)) {
            inputArray #= ",\"" # request.input[i] # "\"";
        };
        inputArray #= "]";

        "{\"input\":" # inputArray # ",\"model\":\"" # request.model # "\"}";
    };
    private func generateIdempotencyKey(request : CreateEmbeddingRequest) : Text {
        let joined = Text.join(", ", request.input.vals());
        let inputHash = Text.hash(joined);
        Nat32.toText(inputHash);
    };

};

import Nat64 "mo:base/Nat64";
import Float "mo:base/Float";
import Int64 "mo:base/Int64";
import Array "mo:base/Array";

module TypeConversion {

    // Convert Rust u64 to Motoko Nat64
    public func u64ToNat64(value : Nat64) : Nat64 {
        value; // Direct mapping
    };

    // Convert Motoko Nat64 to scaled Float
    public func scaledNat64ToFloat(value : Nat64, scale : Float, offset : Float) : Float {
        Float.fromInt64(Int64.fromNat64(value)) / scale - offset;
    };

    // Convert Float to scaled Nat64
    public func floatToScaledNat64(value : Float, scale : Float, offset : Float) : Nat64 {
        Int64.toNat64(Float.toInt64((value + offset) * scale));
    };

    // Validate embedding array
    public func isValidEmbedding(embedding : [Float]) : Bool {
        embedding.size() > 0 and Array.foldLeft<Float, Bool>(
            embedding,
            true,
            func(acc, val) {
                acc and not Float.isNaN(val)
            },
        );
    };

    // Validate norm value
    public func isValidNorm(norm : Float) : Bool {
        norm > 0.0 and not Float.isNaN(norm);
    };
};

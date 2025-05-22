import Random "mo:base/Random";
import Nat8 "mo:base/Nat8";
import Nat "mo:base/Nat";
import Text "mo:base/Text";
import Nat32 "mo:base/Nat32";
import Char "mo:base/Char";

module {

    public func generateRandomID(name : Text) : async Text {
        let entropy = await Random.blob();
        var f = Random.Finite(entropy);
        let count : Nat = 8; // Generate 8 random bytes
        var id = "";
        var i = 0;
        label l loop {
            if (i >= count) break l;
            let b = f.byte();
            switch (b) {
                case (?byte) {
                    // Convert byte to hex string (0-9, a-f)
                    let hex = toHex(Nat32.fromNat(Nat8.toNat(byte)));
                    id := id # hex;
                    i += 1;
                };
                case null {
                    let entropy = await Random.blob();
                    f := Random.Finite(entropy);
                };
            };
        };
        // Prefix with first 4 chars of name (sanitized) and underscore
        let prefix = sliceText(name, 0, Nat.min(4, name.size()));
        prefix # "_" # id;
    };

    public func sliceText(text : Text, start : Nat, end : Nat) : Text {
        var slicedText = "";
        var i = start;
        while (Nat.less(i, end)) {
            let char = Text.fromChar(Text.toArray(text)[i]);
            slicedText := Text.concat(slicedText, char);
            i := Nat.add(i, 1);
        };
        slicedText;
    };

    public func toHex(combinedHash : Nat32) : Text {
        let hex : [Char] = [
            '0',
            '1',
            '2',
            '3',
            '4',
            '5',
            '6',
            '7',
            '8',
            '9',
            'a',
            'b',
            'c',
            'd',
            'e',
            'f',
        ];
        let c0 = hex[Nat32.toNat(combinedHash / 16)];
        let c1 = hex[Nat32.toNat(combinedHash % 16)];
        Char.toText(c0) # Char.toText(c1);
    };

};

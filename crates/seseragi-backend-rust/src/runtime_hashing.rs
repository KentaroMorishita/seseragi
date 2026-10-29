// Public Hash values contain no process-local seed. String/Char fold Unicode
// scalar values with the same wrapping 32-bit FNV steps as the TS runtime.
fn native_hash_scalars(value: impl Iterator<Item = char>) -> i64 {
    let mut state = 0x811c9dc5_u32;
    for scalar in value {
        state = (state ^ scalar as u32).wrapping_mul(0x01000193);
    }
    state as i32 as i64
}

fn native_hash_dictionary(identity: &'static str) -> V {
    cached(identity, || {
        let code: fn(&[V], V) -> V = match identity {
            "std/int::Hash" => |_, value| match value {
                V::Int(value) => checked_int(value),
                _ => panic!("Rust ABI: Hash<Int> requires Int"),
            },
            "std/bool::Hash" => |_, value| match value {
                V::Bool(value) => V::Int(i64::from(value)),
                _ => panic!("Rust ABI: Hash<Bool> requires Bool"),
            },
            "std/char::Hash" => |_, value| match value {
                V::Char(value) => V::Int(native_hash_scalars(std::iter::once(value))),
                _ => panic!("Rust ABI: Hash<Char> requires Char"),
            },
            "std/string::Hash" => |_, value| match value {
                V::Text(value) => V::Int(native_hash_scalars(value.get().chars())),
                _ => panic!("Rust ABI: Hash<String> requires String"),
            },
            "std/unit::Hash" => |_, value| match value {
                V::Unit => V::Int(0),
                _ => panic!("Rust ABI: Hash<Unit> requires Unit"),
            },
            _ => panic!("Rust ABI: unsupported primitive Hash dictionary"),
        };
        V::dict(vec![("hash", V::fun(vec![], code))])
    })
}

#[cfg(test)]
mod hashing_tests {
    use super::*;

    #[test]
    fn public_hashes_preserve_safe_ints_sign_and_unicode_scalar_sequences() {
        let int = native_hash_dictionary("std/int::Hash").field("hash");
        for value in [
            -9007199254740991,
            -4294967296,
            -1,
            0,
            1,
            4294967296,
            9007199254740991,
        ] {
            assert!(matches!(int.apply(V::Int(value)), V::Int(actual) if actual == value));
        }
        for (text, expected) in [("", -2128831035), ("a", -468965076), ("雫🙂", -128254984)] {
            assert_eq!(native_hash_scalars(text.chars()), expected);
        }
        let char_hash = native_hash_dictionary("std/char::Hash").field("hash");
        let text_hash = native_hash_dictionary("std/string::Hash").field("hash");
        for code in [0, 0x7f, 0x80, 0xffff, 0x10000, 0x1f642, 0x10ffff] {
            let scalar = char::from_u32(code).unwrap();
            assert_eq!(
                char_hash.apply(V::Char(scalar)).render(),
                text_hash.apply(V::text(&scalar.to_string())).render()
            );
        }
        drop((int, char_hash, text_hash));
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}

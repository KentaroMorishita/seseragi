fn native_bytes_codec_offset(tag: &'static str, offset: usize) -> V {
    native_left(V::adt(
        tag,
        V::Record(Shared::new(BTreeMap::from([(
            "offset".into(),
            V::Int(i64::try_from(offset).expect("Bytes codec offset fits Int")),
        )]))),
    ))
}

fn native_base64_encode(bytes: &[u8], url_safe: bool) -> V {
    let alphabet = if url_safe {
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"
    } else {
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
    };
    let mut output = String::new();
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(alphabet[(first >> 2) as usize] as char);
        output.push(alphabet[(((first & 3) << 4) | (second >> 4)) as usize] as char);
        if chunk.len() > 1 {
            output.push(alphabet[(((second & 15) << 2) | (third >> 6)) as usize] as char);
        } else if !url_safe {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(alphabet[(third & 63) as usize] as char);
        } else if !url_safe {
            output.push('=');
        }
    }
    V::text(&output)
}

fn native_base64_value(code: u8, url_safe: bool) -> Option<u8> {
    Some(match code {
        b'A'..=b'Z' => code - b'A',
        b'a'..=b'z' => code - b'a' + 26,
        b'0'..=b'9' => code - b'0' + 52,
        b'-' if url_safe => 62,
        b'_' if url_safe => 63,
        b'+' if !url_safe => 62,
        b'/' if !url_safe => 63,
        _ => return None,
    })
}

fn native_base64_decode(text: &str, url_safe: bool) -> V {
    let invalid_length = if url_safe {
        text.len() % 4 == 1
    } else {
        text.len() % 4 != 0
    };
    if invalid_length {
        return native_left(V::adt(
            "std/bytes/base64::InvalidBase64Length",
            V::Int(i64::try_from(text.len()).expect("Base64 length fits Int")),
        ));
    }
    let mut sextets = Vec::new();
    let mut first_padding = None;
    for (offset, code) in text.bytes().enumerate() {
        if code == b'=' {
            if url_safe {
                return native_bytes_codec_offset("std/bytes/base64::InvalidBase64Padding", offset);
            }
            first_padding.get_or_insert(offset);
            continue;
        }
        let Some(value) = native_base64_value(code, url_safe) else {
            return native_bytes_codec_offset("std/bytes/base64::InvalidBase64Digit", offset);
        };
        if let Some(offset) = first_padding {
            return native_bytes_codec_offset("std/bytes/base64::InvalidBase64Padding", offset);
        }
        sextets.push(value);
    }
    let remainder = sextets.len() % 4;
    if !url_safe {
        let padding = text.len() - sextets.len();
        let expected = match remainder {
            2 => 2,
            3 => 1,
            _ => 0,
        };
        if remainder == 1 || padding > 2 || padding != expected {
            return native_bytes_codec_offset(
                "std/bytes/base64::InvalidBase64Padding",
                first_padding.unwrap_or(text.len()),
            );
        }
    }
    if (remainder == 2 && sextets.last().unwrap() & 15 != 0)
        || (remainder == 3 && sextets.last().unwrap() & 3 != 0)
    {
        return native_bytes_codec_offset(
            "std/bytes/base64::NonCanonicalBase64Bits",
            sextets.len() - 1,
        );
    }
    let mut output = Vec::new();
    let mut bits = 0;
    let mut buffer = 0u32;
    for sextet in sextets {
        buffer = (buffer << 6) | u32::from(sextet);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push(((buffer >> bits) & 255) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    native_right(V::Bytes(Shared::new(output)))
}

fn native_hex_encode(bytes: &[u8]) -> V {
    let digits = b"0123456789abcdef";
    let mut output = String::new();
    for &byte in bytes {
        output.push(digits[(byte >> 4) as usize] as char);
        output.push(digits[(byte & 15) as usize] as char);
    }
    V::text(&output)
}

fn native_hex_value(code: u8) -> Option<u8> {
    match code {
        b'0'..=b'9' => Some(code - b'0'),
        b'A'..=b'F' => Some(code - b'A' + 10),
        b'a'..=b'f' => Some(code - b'a' + 10),
        _ => None,
    }
}

fn native_hex_decode(text: &str) -> V {
    if text.len() % 2 != 0 {
        return native_left(V::adt(
            "std/bytes/hex::OddHexLength",
            V::Int(i64::try_from(text.len()).expect("Hex length fits Int")),
        ));
    }
    let mut output = Vec::with_capacity(text.len() / 2);
    for (index, pair) in text.as_bytes().chunks_exact(2).enumerate() {
        let Some(high) = native_hex_value(pair[0]) else {
            return native_bytes_codec_offset("std/bytes/hex::InvalidHexDigit", index * 2);
        };
        let Some(low) = native_hex_value(pair[1]) else {
            return native_bytes_codec_offset("std/bytes/hex::InvalidHexDigit", index * 2 + 1);
        };
        output.push((high << 4) | low);
    }
    native_right(V::Bytes(Shared::new(output)))
}

#[cfg(test)]
mod bytes_codec_tests {
    use super::*;

    fn decode(name: &str, text: &str) -> V {
        native_bytes_operation(name, vec![V::text(text)])
    }

    #[test]
    fn codecs_match_canonical_vectors_and_every_byte_with_all_tail_lengths() {
        clear_dictionaries();
        for (plain, hex, base64, url) in [
            ("", "", "", ""),
            ("f", "66", "Zg==", "Zg"),
            ("fo", "666f", "Zm8=", "Zm8"),
            ("foo", "666f6f", "Zm9v", "Zm9v"),
            ("foob", "666f6f62", "Zm9vYg==", "Zm9vYg"),
            ("fooba", "666f6f6261", "Zm9vYmE=", "Zm9vYmE"),
            ("foobar", "666f6f626172", "Zm9vYmFy", "Zm9vYmFy"),
        ] {
            for (module, encode, decode, expected) in [
                ("hex", "encode", "decode", hex),
                ("base64", "encode", "decode", base64),
                ("base64", "encodeUrl", "decodeUrl", url),
            ] {
                let bytes = V::Bytes(Shared::new(plain.as_bytes().to_vec()));
                let encoded =
                    native_bytes_operation(&format!("std/bytes/{module}::{encode}"), vec![bytes]);
                assert_eq!(native_text(&encoded), expected);
                let decoded = self::decode(&format!("std/bytes/{module}::{decode}"), expected);
                assert!(decoded.tagged("std/prelude::Right"));
                assert_eq!(native_bytes(&decoded.payload()), plain.as_bytes());
            }
        }
        for length in 0..=258 {
            let input: Vec<u8> = (0..length)
                .map(|index| (index * 67 + length) as u8)
                .collect();
            let bytes = V::Bytes(Shared::new(input.clone()));
            for (module, encode, decode) in [
                ("hex", "encode", "decode"),
                ("base64", "encode", "decode"),
                ("base64", "encodeUrl", "decodeUrl"),
            ] {
                let encoded = native_bytes_operation(
                    &format!("std/bytes/{module}::{encode}"),
                    vec![bytes.clone()],
                );
                let decoded = self::decode(
                    &format!("std/bytes/{module}::{decode}"),
                    native_text(&encoded),
                );
                assert!(decoded.tagged("std/prelude::Right"));
                assert_eq!(native_bytes(&decoded.payload()), input);
                assert_eq!(native_bytes(&bytes), input);
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn decoders_preserve_error_variants_payloads_and_validation_order() {
        clear_dictionaries();
        for (operation, text, tag, value) in [
            ("hex::decode", "é0", "OddHexLength", 3),
            ("hex::decode", "0é0", "InvalidHexDigit", 1),
            ("hex::decode", "00🙂", "InvalidHexDigit", 2),
            ("hex::decode", "00xz", "InvalidHexDigit", 2),
            ("base64::decode", "éAA=", "InvalidBase64Length", 5),
            ("base64::decode", "AAé", "InvalidBase64Digit", 2),
            ("base64::decode", "=AA?", "InvalidBase64Padding", 0),
            ("base64::decode", "AA=?", "InvalidBase64Digit", 3),
            ("base64::decode", "====", "InvalidBase64Padding", 0),
            ("base64::decode", "AB==", "NonCanonicalBase64Bits", 1),
            ("base64::decode", "AAB=", "NonCanonicalBase64Bits", 2),
            ("base64::decodeUrl", "AA===", "InvalidBase64Length", 5),
            ("base64::decodeUrl", "AA=", "InvalidBase64Padding", 2),
            ("base64::decodeUrl", "?=", "InvalidBase64Digit", 0),
            ("base64::decodeUrl", "=é", "InvalidBase64Padding", 0),
            ("base64::decodeUrl", "AB", "NonCanonicalBase64Bits", 1),
            ("base64::decodeUrl", "AAB", "NonCanonicalBase64Bits", 2),
        ] {
            let result = decode(&format!("std/bytes/{operation}"), text);
            assert!(result.tagged("std/prelude::Left"), "{operation}/{text}");
            let error = result.payload();
            let module = operation.split("::").next().unwrap();
            assert!(
                error.tagged(&format!("std/bytes/{module}::{tag}")),
                "{operation}/{text}"
            );
            let payload = error.payload();
            let actual = if tag.ends_with("Length") {
                payload
            } else {
                payload.field("offset")
            };
            assert!(
                matches!(actual, V::Int(actual) if actual == value),
                "{operation}/{text}"
            );
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}

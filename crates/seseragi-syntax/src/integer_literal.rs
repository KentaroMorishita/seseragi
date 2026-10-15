//! Decode the language's unsigned Int token, independently of host parsers.
//! Unary minus is an expression operator, not part of this token.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegerLiteralError {
    InvalidSpelling,
    OutsideSafeRange,
}

pub fn decode_integer_literal(raw: &str) -> Result<i64, IntegerLiteralError> {
    use IntegerLiteralError::{InvalidSpelling, OutsideSafeRange};
    let (radix, digits) = match raw.get(..2) {
        Some("0b" | "0B") => (2, &raw[2..]),
        Some("0o" | "0O") => (8, &raw[2..]),
        Some("0x" | "0X") => (16, &raw[2..]),
        _ => (10, raw),
    };
    if digits.is_empty() || (radix == 10 && digits.starts_with('0') && digits.len() > 1) {
        return Err(InvalidSpelling);
    }
    let mut value = Some(0_i64);
    let mut previous_was_digit = false;
    for byte in digits.bytes() {
        if byte == b'_' && previous_was_digit {
            previous_was_digit = false;
            continue;
        }
        let digit = match byte {
            b'0'..=b'9' => i64::from(byte - b'0'),
            b'a'..=b'f' => i64::from(byte - b'a' + 10),
            b'A'..=b'F' => i64::from(byte - b'A' + 10),
            _ => return Err(InvalidSpelling),
        };
        if digit >= radix {
            return Err(InvalidSpelling);
        }
        previous_was_digit = true;
        // Continue validating the spelling even after a value exceeds the limit.
        value = value
            .and_then(|value| value.checked_mul(radix))
            .and_then(|value| value.checked_add(digit))
            .filter(|value| *value <= 9_007_199_254_740_991);
    }
    if !previous_was_digit {
        return Err(InvalidSpelling);
    }
    value.ok_or(OutsideSafeRange)
}

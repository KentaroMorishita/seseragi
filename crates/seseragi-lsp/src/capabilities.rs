use seseragi_source::PositionEncoding;

pub fn negotiate_position_encoding(supported: &[String]) -> PositionEncoding {
    if supported.iter().any(|encoding| encoding == "utf-8") {
        PositionEncoding::Utf8
    } else if supported.iter().any(|encoding| encoding == "utf-16") || supported.is_empty() {
        PositionEncoding::Utf16
    } else if supported.iter().any(|encoding| encoding == "utf-32") {
        PositionEncoding::Utf32
    } else {
        PositionEncoding::Utf16
    }
}

pub fn position_encoding_name(encoding: PositionEncoding) -> &'static str {
    match encoding {
        PositionEncoding::Utf8 => "utf-8",
        PositionEncoding::Utf16 => "utf-16",
        PositionEncoding::Utf32 => "utf-32",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_utf16_when_the_client_does_not_negotiate() {
        assert_eq!(negotiate_position_encoding(&[]), PositionEncoding::Utf16);
    }

    #[test]
    fn prefers_the_native_utf8_range_encoding_when_available() {
        assert_eq!(
            negotiate_position_encoding(&["utf-16".to_owned(), "utf-8".to_owned()]),
            PositionEncoding::Utf8
        );
    }
}

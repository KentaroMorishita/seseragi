fn native_stdin_failure(tag: &'static str, field: Option<(&str, u64)>) -> NativeExit {
    let payload = match field {
        Some((name, value)) => V::Record(Shared::new(BTreeMap::from([(
            name.into(),
            V::Int(value as i64),
        )]))),
        None => V::Unit,
    };
    NativeExit::Failure(V::adt(tag, payload))
}
fn native_read_line(reader: &mut impl std::io::BufRead, offset: &mut u64) -> NativeOutcome {
    const LIMIT: usize = 1024 * 1024;
    const MAX_POSITION: u64 = 9_007_199_254_740_991;
    if *offset > MAX_POSITION {
        return Err(native_stdin_failure(
            "std/stdin::StdinPositionOverflow",
            None,
        ));
    }
    let start = *offset;
    let mut bytes = Vec::new();
    let mut too_long = false;
    let terminated = loop {
        let available = reader
            .fill_buf()
            .map_err(|_| native_stdin_failure("std/stdin::StdinReadFailure", None))?;
        if available.is_empty() {
            break false;
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let count = newline.map_or(available.len(), |index| index + 1);
        let segment = &available[..newline.unwrap_or(count)];
        if !too_long {
            if bytes.len() + segment.len() <= LIMIT + 1 {
                bytes.extend_from_slice(segment);
            } else {
                too_long = true;
                bytes.clear();
            }
        }
        reader.consume(count);
        if *offset > MAX_POSITION - count as u64 {
            *offset = MAX_POSITION + 1;
            return Err(native_stdin_failure(
                "std/stdin::StdinPositionOverflow",
                None,
            ));
        }
        *offset += count as u64;
        if newline.is_some() {
            break true;
        }
    };
    if !terminated && bytes.is_empty() && !too_long {
        return Ok(V::adt("std/prelude::Nothing", V::Unit));
    }
    if terminated && bytes.last() == Some(&b'\r') {
        bytes.pop();
    }
    if too_long || bytes.len() > LIMIT {
        return Err(native_stdin_failure(
            "std/stdin::StdinLineTooLong",
            Some(("limitBytes", LIMIT as u64)),
        ));
    }
    let text = std::str::from_utf8(&bytes).map_err(|error| {
        native_stdin_failure(
            "std/stdin::InvalidStdinUtf8",
            Some(("offset", start + error.valid_up_to() as u64)),
        )
    })?;
    Ok(V::adt(
        "std/prelude::Just",
        V::text(text.strip_prefix('\u{feff}').unwrap_or(text)),
    ))
}

#[cfg(test)]
mod stdin_tests {
    use super::*;

    fn line(reader: &mut impl std::io::BufRead, offset: &mut u64) -> V {
        native_read_line(reader, offset).unwrap_or_else(|_| panic!("readLine failed"))
    }

    #[test]
    fn read_line_preserves_empty_lines_crlf_final_unterminated_line_and_repeated_eof() {
        let mut input = std::io::Cursor::new("雪🙂\r\n\nlast\r".as_bytes());
        let mut offset = 0;
        for expected in [Some("雪🙂"), Some(""), Some("last\r"), None, None] {
            let result = line(&mut input, &mut offset);
            if let Some(expected) = expected {
                assert!(result.tagged("std/prelude::Just"));
                assert_eq!(native_text(&result.payload()), expected);
            } else {
                assert!(result.tagged("std/prelude::Nothing"));
            }
        }
        assert_eq!(offset, input.get_ref().len() as u64);
        assert!(line(&mut std::io::Cursor::new([]), &mut 0).tagged("std/prelude::Nothing"));
    }

    #[test]
    fn read_line_limits_memory_drains_failed_line_and_reports_absolute_utf8_offset() {
        let mut data = vec![b'x'; 1024 * 1024 + 2];
        data.extend_from_slice(b"\nvalid\n\xff\nlast");
        let mut input = std::io::Cursor::new(data);
        let mut offset = 0;
        let Err(NativeExit::Failure(error)) = native_read_line(&mut input, &mut offset) else {
            panic!()
        };
        assert!(error.tagged("std/stdin::StdinLineTooLong"));
        assert_eq!(error.payload().field("limitBytes").render(), "1048576");
        assert_eq!(
            native_text(&line(&mut input, &mut offset).payload()),
            "valid"
        );
        let start = offset;
        let Err(NativeExit::Failure(error)) = native_read_line(&mut input, &mut offset) else {
            panic!()
        };
        assert!(error.tagged("std/stdin::InvalidStdinUtf8"));
        assert_eq!(error.payload().field("offset").render(), start.to_string());
        assert_eq!(
            native_text(&line(&mut input, &mut offset).payload()),
            "last"
        );
        for suffix in [b"\n".as_slice(), b"\r\n".as_slice(), b"".as_slice()] {
            let mut bytes = vec![b'x'; 1024 * 1024];
            bytes.extend_from_slice(suffix);
            assert_eq!(
                native_text(&line(&mut std::io::Cursor::new(bytes), &mut 0).payload()).len(),
                1024 * 1024
            );
        }
    }
}

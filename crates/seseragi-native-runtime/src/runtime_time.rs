const NATIVE_TIME_NANOS_PER_SECOND: i128 = 1_000_000_000;
const NATIVE_TIME_NANOS_PER_DAY: i128 = 86_400 * NATIVE_TIME_NANOS_PER_SECOND;
const NATIVE_TIME_MAX_YEAR: i128 = 9_007_199_254_740_991;

fn native_time_value(owner: &'static str, parts: Vec<V>) -> V {
    V::adt(owner, V::Tuple(Shared::new(parts)))
}

fn native_time_payload<'a>(value: &'a V, owner: &str) -> &'a V {
    let V::Adt(value) = value else {
        panic!("Rust ABI: expected {owner}")
    };
    assert_eq!(value.get().0, owner, "Rust ABI: calendar value owner");
    &value.get().1
}

fn native_time_parts<'a, const N: usize>(value: &'a V, owner: &str) -> &'a [V; N] {
    let V::Tuple(parts) = native_time_payload(value, owner) else {
        panic!("Rust ABI: calendar payload")
    };
    parts
        .get()
        .as_slice()
        .try_into()
        .expect("Rust ABI: calendar field count")
}

fn native_time_numbers<const N: usize>(value: &V, owner: &str) -> [i64; N] {
    native_time_parts::<N>(value, owner).each_ref().map(|part| {
        let V::Int(value) = part else {
            panic!("Rust ABI: calendar Int field")
        };
        checked_int_value(*value)
    })
}

fn native_time_error(tag: &'static str, fields: impl IntoIterator<Item = (&'static str, V)>) -> V {
    native_left(V::adt(
        tag,
        V::Record(Shared::new(
            fields
                .into_iter()
                .map(|(key, value)| (key.into(), value))
                .collect(),
        )),
    ))
}

fn native_time_invalid_date(year: V, month: i64, day: i64) -> V {
    native_time_error(
        "std/time::InvalidDate",
        [
            ("year", year),
            ("month", V::Int(month)),
            ("day", V::Int(day)),
        ],
    )
}

fn native_time_valid_date(year: i64, month: i64, day: i64) -> bool {
    let leap = year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0);
    let length = match month {
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => return false,
    };
    (1..=length).contains(&day)
}

fn native_time_date(year: i64, month: i64, day: i64) -> V {
    for value in [year, month, day] {
        checked_int_value(value);
    }
    if native_time_valid_date(year, month, day) {
        native_right(native_time_value(
            "std/time::LocalDate",
            vec![V::Int(year), V::Int(month), V::Int(day)],
        ))
    } else {
        native_time_invalid_date(V::Int(year), month, day)
    }
}

fn native_time_clock(hour: i64, minute: i64, second: i64, nanosecond: i64) -> V {
    for value in [hour, minute, second, nanosecond] {
        checked_int_value(value);
    }
    if (0..24).contains(&hour)
        && (0..60).contains(&minute)
        && (0..60).contains(&second)
        && (0..1_000_000_000).contains(&nanosecond)
    {
        native_right(native_time_value(
            "std/time::LocalTime",
            vec![
                V::Int(hour),
                V::Int(minute),
                V::Int(second),
                V::Int(nanosecond),
            ],
        ))
    } else {
        native_time_error(
            "std/time::InvalidTime",
            [
                ("hour", V::Int(hour)),
                ("minute", V::Int(minute)),
                ("second", V::Int(second)),
                ("nanosecond", V::Int(nanosecond)),
            ],
        )
    }
}

fn native_time_local(date: &V, time: &V) -> V {
    native_time_numbers::<3>(date, "std/time::LocalDate");
    native_time_numbers::<4>(time, "std/time::LocalTime");
    native_time_value("std/time::LocalDateTime", vec![date.clone(), time.clone()])
}

fn native_time_offset(seconds: i64) -> V {
    checked_int_value(seconds);
    if (-64_800..=64_800).contains(&seconds) {
        native_right(V::adt("std/time::UtcOffset", V::Int(seconds)))
    } else {
        native_left(V::adt("std/time::InvalidUtcOffsetSeconds", V::Int(seconds)))
    }
}

fn native_time_offset_seconds(offset: &V) -> i64 {
    let V::Int(seconds) = native_time_payload(offset, "std/time::UtcOffset") else {
        panic!("Rust ABI: UtcOffset payload")
    };
    assert!(
        (-64_800..=64_800).contains(seconds),
        "Rust ABI: UtcOffset range"
    );
    *seconds
}

fn native_time_instant(value: &V) -> &NativeBigInt {
    let V::BigInt(nanoseconds) = native_time_payload(value, "std/time::Instant") else {
        panic!("Rust ABI: Instant payload")
    };
    nanoseconds.get()
}

fn native_time_at_offset(offset: &V, instant: &V) -> V {
    native_time_offset_seconds(offset);
    native_time_instant(instant);
    native_time_value(
        "std/time::OffsetDateTime",
        vec![instant.clone(), offset.clone()],
    )
}

fn native_time_text_failure_bytes(offset: usize) -> V {
    native_time_error(
        "std/time::InvalidDateTimeText",
        [("offset", V::Int(offset as i64))],
    )
}

fn native_time_text_failure(text: &str, utf16_offset: usize) -> V {
    let (mut units, mut bytes) = (0, 0);
    for character in text.chars() {
        if units >= utf16_offset {
            break;
        }
        if units + character.len_utf16() > utf16_offset {
            // TextEncoder replaces an isolated UTF-16 surrogate with U+FFFD.
            bytes += 3;
            break;
        }
        units += character.len_utf16();
        bytes += character.len_utf8();
    }
    native_time_text_failure_bytes(bytes)
}

fn native_time_shape_failure(text: &str, positions: &[usize], separator: u16) -> V {
    for &index in positions {
        if text.encode_utf16().nth(index) != Some(separator) {
            return native_time_text_failure(text, index);
        }
    }
    native_time_text_failure_bytes(text.len())
}

fn native_time_parse_date(text: &str) -> V {
    let bytes = text.as_bytes();
    let signed = matches!(bytes.first(), Some(b'+' | b'-'));
    let start = usize::from(signed);
    let digits = bytes[start..]
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    let year_end = start + digits;
    if (if signed { digits < 6 } else { digits != 4 })
        || bytes.len() != year_end + 6
        || bytes.get(year_end) != Some(&b'-')
        || bytes.get(year_end + 3) != Some(&b'-')
        || !bytes[year_end + 1..year_end + 3]
            .iter()
            .all(u8::is_ascii_digit)
        || !bytes[year_end + 4..].iter().all(u8::is_ascii_digit)
    {
        // Date shape errors inspect existing positions, even for signed years.
        let offset = text.encode_utf16().enumerate().find_map(|(index, unit)| {
            ((index == 4 || index == 7) && unit != u16::from(b'-')).then_some(index)
        });
        return native_time_text_failure(
            text,
            offset.unwrap_or_else(|| text.encode_utf16().count()),
        );
    }
    let year = text[..year_end].parse::<f64>().expect("ASCII year");
    let month = text[year_end + 1..year_end + 3].parse::<i64>().unwrap();
    let day = text[year_end + 4..].parse::<i64>().unwrap();
    if !year.is_finite() || year.abs() > NATIVE_TIME_MAX_YEAR as f64 {
        return native_time_invalid_date(V::Float(year), month, day);
    }
    native_time_date(year as i64, month, day)
}

fn native_time_parse_clock(text: &str) -> V {
    let bytes = text.as_bytes();
    let base_valid = bytes.len() >= 8
        && bytes[2] == b':'
        && bytes[5] == b':'
        && [0, 1, 3, 4, 6, 7]
            .iter()
            .all(|&index| bytes[index].is_ascii_digit());
    let fraction_valid = bytes.len() == 8
        || ((10..=18).contains(&bytes.len())
            && bytes[8] == b'.'
            && bytes[9..].iter().all(u8::is_ascii_digit));
    if !base_valid || !fraction_valid {
        return native_time_shape_failure(text, &[2, 5], u16::from(b':'));
    }
    let nanosecond = if bytes.len() == 8 {
        0
    } else {
        text[9..].parse::<i64>().unwrap() * 10_i64.pow((18 - bytes.len()) as u32)
    };
    native_time_clock(
        text[..2].parse().unwrap(),
        text[3..5].parse().unwrap(),
        text[6..8].parse().unwrap(),
        nanosecond,
    )
}

fn native_time_parse_local(text: &str) -> V {
    let Some(separator) = text.find('T') else {
        return native_time_text_failure_bytes(text.len());
    };
    if separator == 0 || Some(separator) != text.rfind('T') {
        return native_time_text_failure_bytes(separator);
    }
    let date = native_time_parse_date(&text[..separator]);
    if date.tagged("std/prelude::Left") {
        return date;
    }
    let time = native_time_parse_clock(&text[separator + 1..]);
    if time.tagged("std/prelude::Left") {
        let error = time.payload();
        if error.tagged("std/time::InvalidDateTimeText") {
            let V::Int(offset) = error.payload().field("offset") else {
                unreachable!()
            };
            return native_time_text_failure_bytes(separator + 1 + offset as usize);
        }
        return time;
    }
    native_right(native_time_local(&date.payload(), &time.payload()))
}

fn native_time_parse_offset(text: &str) -> V {
    let bytes = text.as_bytes();
    let (start, seconds, invalid_minute) = if bytes.last() == Some(&b'Z') {
        (bytes.len() - 1, 0, false)
    } else if bytes.len() >= 6 {
        let start = bytes.len() - 6;
        let suffix = &bytes[start..];
        if !matches!(suffix[0], b'+' | b'-')
            || suffix[3] != b':'
            || ![1, 2, 4, 5].iter().all(|&i| suffix[i].is_ascii_digit())
        {
            return native_time_text_failure_bytes(bytes.len());
        }
        let hour = i64::from(suffix[1] - b'0') * 10 + i64::from(suffix[2] - b'0');
        let minute = i64::from(suffix[4] - b'0') * 10 + i64::from(suffix[5] - b'0');
        (
            start,
            (if suffix[0] == b'-' { -1 } else { 1 }) * (hour * 3600 + minute * 60),
            minute > 59,
        )
    } else {
        return native_time_text_failure_bytes(bytes.len());
    };
    let local = native_time_parse_local(&text[..start]);
    if local.tagged("std/prelude::Left") {
        return local;
    }
    if invalid_minute {
        return native_time_text_failure_bytes(start + 4);
    }
    let offset = native_time_offset(seconds);
    if offset.tagged("std/prelude::Left") {
        return offset;
    }
    let nanos = native_time_epoch_nanoseconds(&local.payload())
        - i128::from(seconds) * NATIVE_TIME_NANOS_PER_SECOND;
    let instant = V::adt(
        "std/time::Instant",
        native_big_int_parse(&nanos.to_string()).payload(),
    );
    native_right(native_time_at_offset(&offset.payload(), &instant))
}

fn native_time_epoch_nanoseconds(local: &V) -> i128 {
    let [date, time] = native_time_parts::<2>(local, "std/time::LocalDateTime");
    let [year, month, day] = native_time_numbers::<3>(date, "std/time::LocalDate").map(i128::from);
    let [hour, minute, second, nanosecond] =
        native_time_numbers::<4>(time, "std/time::LocalTime").map(i128::from);
    let adjusted_year = year - i128::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let adjusted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * adjusted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146097 + day_of_era - 719468;
    days * NATIVE_TIME_NANOS_PER_DAY
        + (hour * 3600 + minute * 60 + second) * NATIVE_TIME_NANOS_PER_SECOND
        + nanosecond
}

fn native_time_local_from_nanoseconds(nanos: i128) -> V {
    let days = nanos.div_euclid(NATIVE_TIME_NANOS_PER_DAY);
    let within_day = nanos.rem_euclid(NATIVE_TIME_NANOS_PER_DAY);
    let shifted = days + 719468;
    let era = shifted.div_euclid(146097);
    let day_of_era = shifted - era * 146097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year_of_era + era * 400 + i128::from(month <= 2);
    assert!(
        year.abs() <= NATIVE_TIME_MAX_YEAR,
        "Instant calendar year is outside the Int range"
    );
    let seconds = within_day / NATIVE_TIME_NANOS_PER_SECOND;
    native_time_local(
        &native_time_date(year as i64, month as i64, day as i64).payload(),
        &native_time_clock(
            (seconds / 3600) as i64,
            (seconds % 3600 / 60) as i64,
            (seconds % 60) as i64,
            (within_day % NATIVE_TIME_NANOS_PER_SECOND) as i64,
        )
        .payload(),
    )
}

fn native_time_offset_local(value: &V) -> V {
    let [instant, offset] = native_time_parts::<2>(value, "std/time::OffsetDateTime");
    let nanos = native_time_instant(instant)
        .decimal()
        .parse::<i128>()
        .expect("Instant calendar year is outside the Int range");
    let nanos = nanos
        .checked_add(i128::from(native_time_offset_seconds(offset)) * NATIVE_TIME_NANOS_PER_SECOND)
        .expect("Instant calendar year is outside the Int range");
    native_time_local_from_nanoseconds(nanos)
}

fn native_time_format_date(value: &V) -> String {
    let [year, month, day] = native_time_numbers::<3>(value, "std/time::LocalDate");
    let year = if (0..=9999).contains(&year) {
        format!("{year:04}")
    } else {
        format!("{}{:06}", if year < 0 { '-' } else { '+' }, year.abs())
    };
    format!("{year}-{month:02}-{day:02}")
}

fn native_time_format_clock(value: &V) -> String {
    let [hour, minute, second, nanosecond] = native_time_numbers::<4>(value, "std/time::LocalTime");
    let fraction = if nanosecond == 0 {
        String::new()
    } else {
        format!(".{nanosecond:09}").trim_end_matches('0').to_owned()
    };
    format!("{hour:02}:{minute:02}:{second:02}{fraction}")
}

fn native_time_format_local(value: &V) -> String {
    let [date, time] = native_time_parts::<2>(value, "std/time::LocalDateTime");
    format!(
        "{}T{}",
        native_time_format_date(date),
        native_time_format_clock(time)
    )
}

fn native_time_format_offset(value: &V) -> String {
    let [_, offset] = native_time_parts::<2>(value, "std/time::OffsetDateTime");
    let seconds = native_time_offset_seconds(offset);
    let suffix = if seconds == 0 {
        "Z".into()
    } else {
        format!(
            "{}{:02}:{:02}",
            if seconds < 0 { '-' } else { '+' },
            seconds.abs() / 3600,
            seconds.abs() % 3600 / 60
        )
    };
    format!(
        "{}{suffix}",
        native_time_format_local(&native_time_offset_local(value))
    )
}

fn native_time_error_year(value: &V) -> String {
    // Invalid parsed years retain Number's rounding, exponent and Infinity.
    if let V::Float(year) = value {
        native_number_text(*year)
    } else {
        value.render()
    }
}

pub fn native_time_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/time::localDate", [V::Int(year), V::Int(month), V::Int(day)]) => {
            native_time_date(*year, *month, *day)
        }
        (
            "std/time::localTime",
            [V::Int(hour), V::Int(minute), V::Int(second), V::Int(nanosecond)],
        ) => native_time_clock(*hour, *minute, *second, *nanosecond),
        ("std/time::localDateTime", [date, time]) => native_time_local(date, time),
        ("std/time::utcOffset", [V::Int(seconds)]) => native_time_offset(*seconds),
        ("std/time::parseLocalDate", [text]) => native_time_parse_date(native_text(text)),
        ("std/time::parseLocalTime", [text]) => native_time_parse_clock(native_text(text)),
        ("std/time::parseLocalDateTime", [text]) => native_time_parse_local(native_text(text)),
        ("std/time::parseOffsetDateTime", [text]) => native_time_parse_offset(native_text(text)),
        ("std/time::formatLocalDate", [value]) => V::text(&native_time_format_date(value)),
        ("std/time::formatLocalTime", [value]) => V::text(&native_time_format_clock(value)),
        ("std/time::formatLocalDateTime", [value]) => V::text(&native_time_format_local(value)),
        ("std/time::formatOffsetDateTime", [value]) => V::text(&native_time_format_offset(value)),
        ("std/time::atOffset", [offset, instant]) => native_time_at_offset(offset, instant),
        ("std/time::offsetInstant", [value]) => {
            native_time_parts::<2>(value, "std/time::OffsetDateTime")[0].clone()
        }
        ("std/time::offsetLocalDateTime", [value]) => native_time_offset_local(value),
        _ => panic!("Rust ABI: unadmitted native calendar operation"),
    }
}

#[cfg(test)]
mod time_calendar_tests {
    use super::*;

    fn error_text(result: &V) -> String {
        assert!(result.tagged("std/prelude::Left"));
        native_standard_error_text("std/time::DateTimeError", &result.payload(), false, None)
    }

    #[test]
    fn invalid_parsed_year_fields_keep_int_show_and_debug_number_spelling() {
        for (text, expected) in [
            ("+9007199254740993-01-01".to_owned(), "9007199254740992"),
            ("+9999999999999999999999999-01-01".to_owned(), "1e+25"),
            ("-9999999999999999999999999-01-01".to_owned(), "-1e+25"),
            (format!("+{}-01-01", "9".repeat(400)), "Infinity"),
        ] {
            let year = native_time_parse_date(&text)
                .payload()
                .payload()
                .field("year");
            for (dictionary, method) in [
                ("Show<std/prelude::Int>", "show"),
                ("Debug<std/prelude::Int>", "debug"),
            ] {
                let show = native_primitive_display_dictionary(dictionary);
                assert_eq!(show.field(method).apply(year.clone()).render(), expected);
            }
        }
        assert_eq!(
            native_float_format(9_007_199_254_740_992.0),
            "9007199254740992.0"
        );
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn constructors_preserve_date_time_and_offset_error_payloads() {
        for (year, month, day) in [
            (1900, 2, 29),
            (-100, 2, 29),
            (2024, 0, 1),
            (2024, 13, 1),
            (2024, 1, 0),
            (2024, 4, 31),
        ] {
            assert_eq!(
                error_text(&native_time_date(year, month, day)),
                format!("InvalidDate {{ year: {year}, month: {month}, day: {day} }}")
            );
        }
        for (hour, minute, second, nanosecond) in [
            (-1, 0, 0, 0),
            (24, 0, 0, 0),
            (0, -1, 0, 0),
            (0, 60, 0, 0),
            (0, 0, -1, 0),
            (23, 59, 60, 0),
            (0, 0, 0, -1),
            (0, 0, 0, 1_000_000_000),
        ] {
            assert_eq!(error_text(&native_time_clock(hour, minute, second, nanosecond)), format!("InvalidTime {{ hour: {hour}, minute: {minute}, second: {second}, nanosecond: {nanosecond} }}"));
        }
        for seconds in [
            -9_007_199_254_740_991,
            -64_801,
            64_801,
            9_007_199_254_740_991,
        ] {
            assert_eq!(
                error_text(&native_time_offset(seconds)),
                format!("InvalidUtcOffsetSeconds {seconds}")
            );
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn parsers_preserve_utf8_offsets_validation_order_and_number_error_spelling() {
        for (parse, text, offset) in [
            (native_time_parse_date as fn(&str) -> V, "", 0),
            (native_time_parse_date, "2024/01/01", 4),
            (native_time_parse_date, "2024-01/01", 7),
            (native_time_parse_date, "202🙂-01-01", 6),
            (native_time_parse_date, "あ2024-01-01", 6),
            (native_time_parse_clock, "01:0🙂:00", 7),
            (native_time_parse_clock, "🙂:00:00", 10),
            (native_time_parse_clock, "01:02:03.1234567890", 19),
            (native_time_parse_local, "2024-01-01T🙂", 15),
            (native_time_parse_local, "2024-01-01T01:0🙂:00", 18),
            (native_time_parse_local, "🙂TT", 4),
            (native_time_parse_offset, "2024-01-01T00:00:00+00:60", 23),
            (native_time_parse_offset, "2024-01-01T🙂+00:60", 15),
            (native_time_parse_offset, "🙂", 4),
        ] {
            assert_eq!(
                error_text(&parse(text)),
                format!("InvalidDateTimeText {{ offset: {offset} }}"),
                "{text:?}"
            );
        }
        assert_eq!(
            error_text(&native_time_parse_offset("2023-02-29T24:00:00+00:60")),
            "InvalidDate { year: 2023, month: 2, day: 29 }"
        );
        for (text, year) in [
            ("+9007199254740992-01-01", "9007199254740992"),
            ("+9007199254740993-01-01", "9007199254740992"),
            ("-9999999999999999999999999-01-01", "-1e+25"),
        ] {
            assert_eq!(
                error_text(&native_time_parse_date(text)),
                format!("InvalidDate {{ year: {year}, month: 1, day: 1 }}")
            );
        }
        let huge = format!("+{}-01-01", "9".repeat(400));
        assert_eq!(
            error_text(&native_time_parse_date(&huge)),
            "InvalidDate { year: Infinity, month: 1, day: 1 }"
        );
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn epoch_conversion_preserves_negative_fractions_leap_centuries_and_extreme_years() {
        for (text, nanoseconds) in [
            ("1970-01-01T00:00:00Z", "0"),
            ("1969-12-31T23:59:59.999999999Z", "-1"),
            ("2000-01-01T00:00:00Z", "946684800000000000"),
            ("2000-02-29T00:00:00Z", "951782400000000000"),
            ("0000-01-01T00:00:00Z", "-62167219200000000000"),
            ("-000001-12-31T00:00:00Z", "-62167305600000000000"),
        ] {
            let value = native_time_parse_offset(text).payload();
            let [instant, _] = native_time_parts::<2>(&value, "std/time::OffsetDateTime");
            assert_eq!(
                native_time_instant(instant).decimal(),
                nanoseconds,
                "{text}"
            );
            assert_eq!(native_time_format_offset(&value), text);
        }
        for text in [
            "+9007199254740991-12-31T23:59:59.999999999Z",
            "-9007199254740991-01-01T00:00:00Z",
            "-000400-02-29T00:00:00Z",
            "+010000-01-01T00:00:00Z",
        ] {
            assert_eq!(
                native_time_format_offset(&native_time_parse_offset(text).payload()),
                text
            );
        }
        let value = native_time_parse_offset("1969-12-31T23:59:59.999999999Z").payload();
        let instant = native_time_parts::<2>(&value, "std/time::OffsetDateTime")[0].clone();
        for (seconds, expected) in [
            (1, "1970-01-01T00:00:00.999999999+00:00"),
            (-1, "1969-12-31T23:59:58.999999999-00:00"),
            (64_800, "1970-01-01T17:59:59.999999999+18:00"),
            (-64_800, "1969-12-31T05:59:59.999999999-18:00"),
        ] {
            let offset = native_time_offset(seconds).payload();
            assert_eq!(
                native_time_format_offset(&native_time_at_offset(&offset, &instant)),
                expected
            );
        }
        drop((value, instant));
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}

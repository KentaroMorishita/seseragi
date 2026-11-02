struct NativeTimeZoneRules {
    id: &'static str,
    offsets: Vec<i64>,
    untils: Vec<i64>,
}

fn native_time_zone_base60(text: &str) -> f64 {
    let (sign, text) = text
        .strip_prefix('-')
        .map_or((1.0, text), |text| (-1.0, text));
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    let digit = |byte| match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'z' => byte - b'a' + 10,
        b'A'..=b'Z' => byte - b'A' + 36,
        _ => panic!("invalid bundled timezone digit"),
    };
    let mut value = 0.0;
    for byte in whole.bytes() {
        value = value * 60.0 + f64::from(digit(byte));
    }
    let mut scale = 1.0;
    for byte in fraction.bytes() {
        scale /= 60.0;
        value += f64::from(digit(byte)) * scale;
    }
    value * sign
}

impl NativeTimeZoneRules {
    fn unpack(packed: &'static str) -> Self {
        let parts: Vec<_> = packed.split('|').collect();
        let offsets: Vec<_> = parts[1]
            .split(' ')
            .map(|part| -(native_time_zone_base60(part) * 60.0).trunc() as i64)
            .collect();
        let offsets: Vec<_> = parts[2]
            .bytes()
            .map(|index| {
                offsets[native_time_zone_base60(std::str::from_utf8(&[index]).unwrap()) as usize]
            })
            .collect();
        let mut previous = 0.0;
        let mut untils: Vec<_> = parts[3]
            .split(' ')
            .take(offsets.len() - 1)
            .map(|part| {
                previous = (previous + native_time_zone_base60(part) * 60_000.0 + 0.5).floor();
                previous as i64
            })
            .collect();
        assert_eq!(
            untils.len() + 1,
            offsets.len(),
            "bundled timezone transition count"
        );
        untils.push(i64::MAX);
        Self {
            id: parts[0],
            offsets,
            untils,
        }
    }

    fn offset_at(&self, milliseconds: i64) -> i64 {
        self.offsets[self.untils.partition_point(|until| *until <= milliseconds)]
    }

    fn initial_offset(&self) -> i64 {
        let local = native_time_local(
            &native_time_date(1800, 1, 1).payload(),
            &native_time_clock(0, 0, 0, 0).payload(),
        );
        self.offset_at((native_time_epoch_nanoseconds(&local) / 1_000_000) as i64)
    }
}

fn native_time_zone_index(id: &str) -> Option<usize> {
    NATIVE_TIME_ZONE_NAMES
        .binary_search_by_key(&id, |(name, _)| *name)
        .ok()
        .map(|index| NATIVE_TIME_ZONE_NAMES[index].1)
}

fn native_time_zone_rules(index: usize) -> &'static NativeTimeZoneRules {
    static RULES: std::sync::OnceLock<Vec<std::sync::OnceLock<NativeTimeZoneRules>>> =
        std::sync::OnceLock::new();
    RULES.get_or_init(|| {
        NATIVE_TIME_ZONE_PACKED
            .iter()
            .map(|_| std::sync::OnceLock::new())
            .collect()
    })[index]
        .get_or_init(|| NativeTimeZoneRules::unpack(NATIVE_TIME_ZONE_PACKED[index]))
}

fn native_time_zone_database_version() -> Result<&'static str, &'static str> {
    if NATIVE_TIME_ZONE_PACKED.is_empty() {
        Err("bundled timezone database is unavailable")
    } else {
        Ok(NATIVE_TIME_ZONE_VERSION)
    }
}

fn native_time_load_zone(id: &str, version: Result<&str, &str>) -> NativeOutcome {
    let version = version.map_err(|detail| {
        NativeExit::failure(V::adt(
            "std/time::TimeZoneDatabaseUnavailable",
            V::text(detail),
        ))
    })?;
    if version != "2025b" {
        return Err(NativeExit::failure(V::adt(
            "std/time::TimeZoneDatabaseVersionMismatch",
            V::Record(Shared::new(BTreeMap::from([
                ("required".into(), V::text("2025b")),
                ("actual".into(), V::text(version)),
            ]))),
        )));
    }
    let index = native_time_zone_index(id)
        .ok_or_else(|| NativeExit::failure(V::adt("std/time::UnknownTimeZone", V::text(id))))?;
    Ok(native_time_value(
        "std/time::TimeZone",
        vec![V::text(native_time_zone_rules(index).id), V::text(version)],
    ))
}

fn native_time_zone_fields(zone: &V) -> (&str, &str) {
    let [id, version] = native_time_parts::<2>(zone, "std/time::TimeZone");
    (native_text(id), native_text(version))
}

fn native_time_zone_year(local: &V) -> i64 {
    let [date, _] = native_time_parts::<2>(local, "std/time::LocalDateTime");
    native_time_numbers::<3>(date, "std/time::LocalDate")[0]
}

fn native_time_zone_with_year(local: &V, year: i64) -> V {
    let [date, time] = native_time_parts::<2>(local, "std/time::LocalDateTime");
    let [_, month, day] = native_time_numbers::<3>(date, "std/time::LocalDate");
    native_time_local(&native_time_date(year, month, day).payload(), time)
}

fn native_time_zone_mapped_year(year: i64) -> i64 {
    if year <= 2499 {
        year
    } else {
        2100 + (year - 2100).rem_euclid(400)
    }
}

fn native_time_zoned(instant: &V, local: &V, offset: i64, zone: &V) -> V {
    native_time_value(
        "std/time::ZonedDateTime",
        vec![
            instant.clone(),
            local.clone(),
            native_time_offset(offset).payload(),
            zone.clone(),
        ],
    )
}

fn native_time_zoned_from_local(local: &V, zone: &V, offset: i64) -> V {
    let nanos =
        native_time_epoch_nanoseconds(local) - i128::from(offset) * NATIVE_TIME_NANOS_PER_SECOND;
    let instant = V::adt(
        "std/time::Instant",
        native_big_int_parse(&nanos.to_string()).payload(),
    );
    native_time_zoned(&instant, local, offset, zone)
}

fn native_time_at_zone(instant: &V, zone: &V) -> V {
    let nanos = native_time_instant(instant)
        .decimal()
        .parse::<i128>()
        .expect("Instant calendar year is outside the Int range");
    let utc = native_time_local_from_nanoseconds(nanos);
    let year = native_time_zone_year(&utc);
    let rules = native_time_zone_rules(
        native_time_zone_index(native_time_zone_fields(zone).0).expect("canonical TimeZone"),
    );
    let offset = if year < 1800 {
        rules.initial_offset()
    } else {
        let mapped = native_time_zone_with_year(&utc, native_time_zone_mapped_year(year));
        // The TS runtime truncates epoch nanoseconds toward zero before lookup.
        let seconds = native_time_epoch_nanoseconds(&mapped) / NATIVE_TIME_NANOS_PER_SECOND;
        rules.offset_at((seconds * 1000) as i64)
    };
    let local = native_time_local_from_nanoseconds(
        nanos + i128::from(offset) * NATIVE_TIME_NANOS_PER_SECOND,
    );
    native_time_zoned(instant, &local, offset, zone)
}

fn native_time_resolve_local(local: &V, zone: &V) -> V {
    let year = native_time_zone_year(local);
    let rules = native_time_zone_rules(
        native_time_zone_index(native_time_zone_fields(zone).0).expect("canonical TimeZone"),
    );
    if year < 1800 {
        return V::adt(
            "std/time::Unique",
            native_time_zoned_from_local(local, zone, rules.initial_offset()),
        );
    }
    let mapped = native_time_zone_with_year(local, native_time_zone_mapped_year(year));
    let nanos = native_time_epoch_nanoseconds(&mapped);
    let boundary = |index: usize| {
        let transition = index / 2;
        if transition + 1 == rules.offsets.len() {
            return i128::MAX;
        }
        let before = rules.offsets[transition];
        let after = rules.offsets[transition + 1];
        let offset = if index % 2 == 0 {
            before.min(after)
        } else {
            before.max(after)
        };
        i128::from(rules.untils[transition]) * 1_000_000
            + i128::from(offset) * NATIVE_TIME_NANOS_PER_SECOND
    };
    let (mut low, mut high) = (0, rules.offsets.len() * 2 - 1);
    while low < high {
        let middle = low + (high - low) / 2;
        if boundary(middle) <= nanos {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    let index = low / 2;
    let before = rules.offsets[index];
    if low % 2 == 0 {
        return V::adt(
            "std/time::Unique",
            native_time_zoned_from_local(local, zone, before),
        );
    }
    let after = rules.offsets[index + 1];
    if after < before {
        V::adt(
            "std/time::Ambiguous",
            V::Record(Shared::new(BTreeMap::from([
                (
                    "earlier".into(),
                    native_time_zoned_from_local(local, zone, before),
                ),
                (
                    "later".into(),
                    native_time_zoned_from_local(local, zone, after),
                ),
            ]))),
        )
    } else {
        let transition_local = native_time_local_from_nanoseconds(
            i128::from(rules.untils[index]) * 1_000_000
                + i128::from(before) * NATIVE_TIME_NANOS_PER_SECOND,
        );
        let original = native_time_zone_with_year(&transition_local, year);
        let nanos = native_time_epoch_nanoseconds(&original)
            - i128::from(before) * NATIVE_TIME_NANOS_PER_SECOND;
        V::adt(
            "std/time::Gap",
            V::Record(Shared::new(BTreeMap::from([
                (
                    "transition".into(),
                    V::adt(
                        "std/time::Instant",
                        native_big_int_parse(&nanos.to_string()).payload(),
                    ),
                ),
                ("offsetBefore".into(), native_time_offset(before).payload()),
                ("offsetAfter".into(), native_time_offset(after).payload()),
            ]))),
        )
    }
}

pub fn native_timezone_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/time::databaseVersion", [V::Unit]) => V::leaf(vec![], |_, _| {
            Ok(V::text(
                native_time_zone_database_version().unwrap_or("unavailable"),
            ))
        }),
        ("std/time::loadTimeZone", [_]) => V::leaf(arguments, |values, _| {
            native_time_load_zone(native_text(&values[0]), native_time_zone_database_version())
        }),
        ("std/time::timeZoneId", [zone]) => V::text(native_time_zone_fields(zone).0),
        ("std/time::timeZoneVersion", [zone]) => V::text(native_time_zone_fields(zone).1),
        ("std/time::atTimeZone", [instant, zone]) => native_time_at_zone(instant, zone),
        ("std/time::resolveLocal", [local, zone]) => native_time_resolve_local(local, zone),
        (
            "std/time::zonedInstant"
            | "std/time::zonedLocalDateTime"
            | "std/time::zonedOffset"
            | "std/time::zonedTimeZone",
            [value],
        ) => {
            let parts = native_time_parts::<4>(value, "std/time::ZonedDateTime");
            parts[match name {
                "std/time::zonedInstant" => 0,
                "std/time::zonedLocalDateTime" => 1,
                "std/time::zonedOffset" => 2,
                _ => 3,
            }]
            .clone()
        }
        ("std/time::Unique" | "std/time::Ambiguous" | "std/time::Gap", [value]) => V::adt(
            match name {
                "std/time::Unique" => "std/time::Unique",
                "std/time::Ambiguous" => "std/time::Ambiguous",
                _ => "std/time::Gap",
            },
            value.clone(),
        ),
        _ => panic!("Rust ABI: unadmitted native timezone operation"),
    }
}

#[cfg(test)]
mod timezone_tests {
    use super::*;
    use std::{
        fs,
        io::Write,
        path::{Path, PathBuf},
        process::Command,
    };

    mod temporary {
        include!("../../test-support/temporary.rs");
    }

    fn loaded(id: &str) -> V {
        native_time_load_zone(id, native_time_zone_database_version())
            .ok()
            .unwrap()
    }

    fn failure(id: &str, version: Result<&str, &str>) -> String {
        let Err(NativeExit::Failure(error)) = native_time_load_zone(id, version) else {
            panic!("expected TimeZoneError")
        };
        native_standard_error_text("std/time::TimeZoneError", &error, false, None)
    }

    #[test]
    fn aliases_versions_and_provider_failures_match_ts() {
        assert_eq!(native_time_zone_database_version(), Ok("2025b"));
        for (alias, canonical) in [
            ("US/Eastern", "America/New_York"),
            ("UTC", "Etc/UCT"),
            ("Europe/Kyiv", "Europe/Kiev"),
            ("Asia/Kathmandu", "Asia/Kathmandu"),
        ] {
            let zone = loaded(alias);
            assert_eq!(native_time_zone_fields(&zone), (canonical, "2025b"));
        }
        for id in [
            "",
            "europe/berlin",
            "Europe/Berlin ",
            "Europe/Berlin\n",
            "Unknown/雫🙂",
            "__proto__",
            "+01:00",
        ] {
            assert_eq!(
                failure(id, Ok("2025b")),
                format!("UnknownTimeZone {}", native_json_string(id))
            );
        }
        assert_eq!(
            failure("Europe/Berlin", Ok("2025a")),
            "TimeZoneDatabaseVersionMismatch { required: \"2025b\", actual: \"2025a\" }"
        );
        assert_eq!(
            failure("unknown", Err("database unavailable")),
            "TimeZoneDatabaseUnavailable \"database unavailable\""
        );
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn timezone_effects_remain_cold_and_release_captured_ids() {
        let plan =
            native_timezone_operation("std/time::loadTimeZone", vec![V::text("unknown-cold")]);
        let mut context = NativeContext::default();
        let Err(NativeExit::Failure(error)) = context.root(plan.clone()) else {
            panic!("expected UnknownTimeZone")
        };
        assert_eq!(
            native_standard_error_text("std/time::TimeZoneError", &error, false, None),
            "UnknownTimeZone \"unknown-cold\""
        );
        drop(error);
        assert!(matches!(context.root(plan), Err(NativeExit::Failure(_))));
        let version = context
            .root(native_timezone_operation(
                "std/time::databaseVersion",
                vec![V::Unit],
            ))
            .ok()
            .unwrap();
        assert_eq!(native_text(&version), "2025b");
        drop((context, version));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    fn offset_text(seconds: i64) -> String {
        native_time_format_offset(&native_time_at_offset(
            &native_time_offset(seconds).payload(),
            &native_clock_instant(0),
        ))
    }

    fn zoned_text(zoned: &V) -> String {
        let [instant, local, offset, zone] =
            native_time_parts::<4>(zoned, "std/time::ZonedDateTime");
        let (id, version) = native_time_zone_fields(zone);
        format!(
            "{}|{}|{}|{id}|{version}",
            native_time_instant(instant).decimal(),
            native_time_format_local(local),
            offset_text(native_time_offset_seconds(offset))
        )
    }

    fn resolution_text(value: &V) -> String {
        let payload = value.payload();
        if value.tagged("std/time::Unique") {
            format!("U|{}", zoned_text(&payload))
        } else if value.tagged("std/time::Ambiguous") {
            format!(
                "A|{}|{}",
                zoned_text(&payload.field("earlier")),
                zoned_text(&payload.field("later"))
            )
        } else {
            assert!(value.tagged("std/time::Gap"));
            format!(
                "G|{}|{}|{}",
                native_time_instant(&payload.field("transition")).decimal(),
                offset_text(native_time_offset_seconds(&payload.field("offsetBefore"))),
                offset_text(native_time_offset_seconds(&payload.field("offsetAfter")))
            )
        }
    }

    #[test]
    fn every_bundled_zone_and_transition_matches_ts_at_nanosecond_boundaries() {
        struct Scratch(PathBuf);
        impl Drop for Scratch {
            fn drop(&mut self) {
                fs::remove_dir_all(&self.0).unwrap();
            }
        }
        let scratch = Scratch(temporary::temporary_directory("seseragi-timezone-rules"));
        let mut queries =
            std::io::BufWriter::new(fs::File::create(scratch.0.join("queries.tsv")).unwrap());
        let mut expected =
            std::io::BufWriter::new(fs::File::create(scratch.0.join("native.txt")).unwrap());
        for (name, index) in NATIVE_TIME_ZONE_NAMES {
            let zone = loaded(name);
            let rules = native_time_zone_rules(*index);
            assert!(
                rules.untils.windows(2).all(|pair| pair[0] <= pair[1]),
                "{name}"
            );
            assert!(
                rules.offsets.iter().all(|offset| offset.abs() <= 64_800),
                "{name}"
            );
            for text in [
                "1799-12-31T23:59:59.999999999",
                "1800-01-01T00:00:00",
                "1969-12-31T23:59:59.999999999",
                "2499-12-31T23:59:59.999999999",
                "2500-01-01T00:00:00",
                "+012345-03-25T02:30:00",
                "+9007199254740991-06-07T08:09:10.123456789",
                "-9007199254740991-06-07T08:09:10.123456789",
            ] {
                let local = native_time_parse_local(text).payload();
                writeln!(queries, "local\t{}\t{text}", rules.id).unwrap();
                writeln!(
                    expected,
                    "{}",
                    resolution_text(&native_time_resolve_local(&local, &zone))
                )
                .unwrap();
            }
            if *name != rules.id {
                continue;
            }
            for (index, until) in rules.untils.iter().enumerate().take(rules.untils.len() - 1) {
                let transition = i128::from(*until) * 1_000_000;
                for nanos in [transition - 1, transition, transition + 1] {
                    let instant = V::adt(
                        "std/time::Instant",
                        native_big_int_parse(&nanos.to_string()).payload(),
                    );
                    writeln!(queries, "instant\t{}\t{nanos}", rules.id).unwrap();
                    writeln!(
                        expected,
                        "{}",
                        zoned_text(&native_time_at_zone(&instant, &zone))
                    )
                    .unwrap();
                }
                let before = rules.offsets[index];
                let after = rules.offsets[index + 1];
                for nanos in [
                    transition + i128::from(before.min(after)) * NATIVE_TIME_NANOS_PER_SECOND - 1,
                    transition + i128::from(before.min(after)) * NATIVE_TIME_NANOS_PER_SECOND,
                    transition + i128::from(before.max(after)) * NATIVE_TIME_NANOS_PER_SECOND - 1,
                    transition + i128::from(before.max(after)) * NATIVE_TIME_NANOS_PER_SECOND,
                ] {
                    let local = native_time_local_from_nanoseconds(nanos);
                    writeln!(
                        queries,
                        "local\t{}\t{}",
                        rules.id,
                        native_time_format_local(&local)
                    )
                    .unwrap();
                    writeln!(
                        expected,
                        "{}",
                        resolution_text(&native_time_resolve_local(&local, &zone))
                    )
                    .unwrap();
                }
            }
        }
        drop((queries, expected));
        let mut child = Command::new("bun")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/timezones.ts"))
            .arg(scratch.0.join("queries.tsv"))
            .arg(scratch.0.join("ts.txt"))
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if std::time::Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("timezone TS comparison exceeded its deadline");
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        assert!(status.success());
        let expected = fs::read_to_string(scratch.0.join("native.txt")).unwrap();
        let actual = fs::read_to_string(scratch.0.join("ts.txt")).unwrap();
        assert_eq!(expected.lines().count(), actual.lines().count());
        for (index, ((query, expected), actual)) in
            fs::read_to_string(scratch.0.join("queries.tsv"))
                .unwrap()
                .lines()
                .zip(expected.lines())
                .zip(actual.lines())
                .enumerate()
        {
            assert_eq!(expected, actual, "query {index}: {query}");
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}

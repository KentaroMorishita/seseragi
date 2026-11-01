#[path = "support/native.rs"]
mod support;

use support::run;

#[test]
fn checked_fixtures_execute_without_bun_or_ts() {
    let cases = [
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/ownership.ssrg"),
            "(42, 42, 11, 12)",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/shared-values.ssrg"),
            "(shared, shared, shared)",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/recursive-adt.ssrg"),
            "(3, 5)",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/structural-record.ssrg"),
            "(row, row)",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/higher-kinded.ssrg"),
            "(42, 42)",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/nested-dictionary.ssrg"),
            "(42, 42)",
        ),
        (
            include_str!("../../../examples/spec/fixtures/rust-backend/let-shadowing.ssrg"),
            "(42, 42, 42, 42)",
        ),
    ];
    for (source, expected) in cases {
        run(source, expected);
    }
}

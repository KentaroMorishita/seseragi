#[path = "support/native.rs"]
mod support;

use support::run;

#[test]
fn disjoint_higher_kinded_heads_keep_selected_and_saved_dictionaries() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/constructor-disjoint.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/constructor-disjoint.stdout")
            .trim_end(),
    );
}

#[test]
fn alias_constructors_keep_generic_child_evidence_and_saved_closures() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/alias-constructor.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/alias-constructor.stdout")
            .trim_end(),
    );
}

#[test]
fn disjoint_repeated_binders_keep_selected_and_saved_dictionaries() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/instance-disjoint.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/instance-disjoint.stdout")
            .trim_end(),
    );
}

#[test]
fn alias_heads_keep_selected_generic_higher_kinded_and_saved_dictionaries() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/instance-alias.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/instance-alias.stdout")
            .trim_end(),
    );
}

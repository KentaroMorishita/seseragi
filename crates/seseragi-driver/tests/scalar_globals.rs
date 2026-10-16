use seseragi_driver::{
    compile_local_project_with_profile, compile_module, BuildProfile, CompileInput,
};
use seseragi_lowering::{
    scalar_self_tail_loop_with_globals, CoreScalarGlobals, TypeScriptFunction,
};

#[test]
fn scalar_global_proofs_are_shared_by_ts_profiles_and_core_only_compilation() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/scalar-global.ssrg");
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let input = CompileInput::new("scalar.ssrg", "scalar/main", source).with_profile(profile);
        let compiled = compile_module(input).unwrap();
        let core = seseragi_driver::compile_core_module(input).unwrap();
        assert_eq!(compiled.core_ir, core.core_ir);
        let globals = CoreScalarGlobals::from_modules([&compiled.core_ir]);
        for (name, expected) in [
            ("sum", true),
            ("retained", false),
            ("shadow", true),
            ("localShadow", false),
            ("boolean", true),
        ] {
            let f = compiled
                .core_ir
                .functions
                .iter()
                .find(|f| f.symbol.ends_with(&format!("::{name}")))
                .unwrap();
            assert_eq!(
                scalar_self_tail_loop_with_globals(f, &compiled.core_ir.module, &globals).is_some(),
                expected,
                "{name}"
            );
            let flag = compiled
                .typescript_ir
                .functions
                .iter()
                .find_map(|f| match f {
                    TypeScriptFunction::ConstFunction {
                        name: n,
                        scalar_tail_loop,
                        ..
                    } if n == name => Some(*scalar_tail_loop),
                    _ => None,
                })
                .unwrap();
            assert_eq!(flag, expected, "{name} {profile:?}");
        }
    }
}

#[test]
fn namespace_and_reexport_scalar_globals_keep_their_package_owner_in_both_targets() {
    let loaded = seseragi_project::load_local_project(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/spec/fixtures/rust-backend-projects/scalar-globals"
    ))
    .unwrap();
    let core = seseragi_driver::compile_core_local_project(&loaded).unwrap();
    let globals =
        CoreScalarGlobals::from_modules(core.compiled.modules.values().map(|m| &m.core_ir));
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let project = compile_local_project_with_profile(&loaded, Some(profile)).unwrap();
        assert_eq!(project.compiled.modules.len(), 4);
        let mut loops = 0;
        for (owner, module) in &project.compiled.modules {
            assert_eq!(module.core_ir, core.compiled.modules[owner].core_ir);
            let mut imports = std::collections::BTreeSet::new();
            for binding in module
                .typescript_ir
                .source_imports
                .iter()
                .flat_map(|group| &group.bindings)
                .filter(|binding| !binding.type_only)
            {
                assert!(
                    imports.insert(&binding.local),
                    "duplicate import local {} in {owner}",
                    binding.local
                );
            }
            for function in &module.core_ir.functions {
                if let Some(proof) = scalar_self_tail_loop_with_globals(function, owner, &globals) {
                    loops += 1;
                    assert_eq!(proof.global_reads.len(), 1);
                    let canonical = &proof.global_reads.values().next().unwrap().symbol;
                    let binding_owner = canonical.rsplit_once("::").unwrap().0;
                    assert!(core.compiled.modules[binding_owner]
                        .core_ir
                        .bindings
                        .iter()
                        .any(|b| &b.symbol == canonical));
                    let name = function.symbol.rsplit_once("::").unwrap().1;
                    assert!(module.typescript_ir.functions.iter().any(|f| matches!(f, TypeScriptFunction::ConstFunction { name: n, scalar_tail_loop: true, .. } if n == name)));
                }
            }
        }
        assert_eq!(loops, 5);
    }
}

#[test]
fn forward_initializer_reads_are_still_diagnosed_before_target_optimization() {
    for steps in [0, 1] {
        let source = format!("let first: Int = sum {steps} 0\nlet increment: Int = 3\nfn sum n: Int -> total: Int -> Int = if n == 0 then total else sum (n - 1) (total + increment)\n");
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let input =
                CompileInput::new("forward.ssrg", "scalar/forward", &source).with_profile(profile);
            let analysis = seseragi_driver::analyze_module(input);
            assert!(analysis
                .diagnostics
                .diagnostics
                .iter()
                .any(|d| d.code == "SES-N0201"));
            assert!(compile_module(input).is_err());
            assert!(seseragi_driver::compile_core_module(input).is_err());
        }
    }
}

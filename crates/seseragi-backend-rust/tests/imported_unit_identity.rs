use seseragi_backend_rust::emit_rust_project;
use seseragi_lowering::CoreModule;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

static SERIAL: AtomicUsize = AtomicUsize::new(0);
const NOMINAL_UNIT: &str = "fixture/native-imported-unit-identity@0.0.0::domain::Unit";
const NOMINAL_UNIT_PROVIDER: &str = "fixture/native-imported-unit-identity@0.0.0::domain";
struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn execute(source: &str, expected: &[u8], exit: i32, trace: Option<&str>) {
    let directory = Scratch(std::env::temp_dir().join(format!(
        "seseragi-imported-unit-{}-{}",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&directory.0).unwrap();
    let input = directory.0.join("program.rs");
    let binary = directory.0.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    fs::write(&input, source).unwrap();
    let built = Command::new("rustc")
        .args(["--edition=2021", "-O", "-o"])
        .arg(&binary)
        .arg(input)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let executed = Command::new(binary)
        .current_dir(&directory.0)
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert_eq!(
        executed.status.code(),
        Some(exit),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(executed.stdout, expected);
    assert!(String::from_utf8_lossy(&executed.stderr).contains("managed-live=0"));
    if let Some(trace) = trace {
        assert_eq!(
            fs::read_to_string(directory.0.join("trace.txt")).unwrap(),
            trace
        );
    }
}

fn project(fixture: &str) -> (Vec<CoreModule>, String) {
    let loaded = seseragi_project::load_local_project(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/fixtures/rust-backend-projects")
            .join(fixture),
    )
    .unwrap();
    let checked = seseragi_driver::compile_core_local_project(&loaded).unwrap();
    (
        checked
            .compiled
            .order
            .iter()
            .map(|id| checked.compiled.modules[id].core_ir.clone())
            .collect(),
        checked.entry_module,
    )
}

#[test]
fn imported_anonymous_unit_does_not_bind_to_the_nominal_unit() {
    let (modules, entry) = project("imported-unit-identity");
    let artifact = emit_rust_project(&modules.iter().collect::<Vec<_>>(), &entry, "main")
        .expect("checked nominal Unit and anonymous Unit have separate identities");
    assert_eq!(artifact.metadata.modules.len(), 2);
    execute(&artifact.source, include_bytes!("../../../examples/spec/fixtures/rust-backend-projects/imported-unit-identity/expected.stdout"), 0, None);

    let mut reversed = modules.clone();
    for module in &mut reversed {
        module.external_type_bindings.reverse();
    }
    let reordered =
        emit_rust_project(&reversed.iter().collect::<Vec<_>>(), &entry, "main").unwrap();
    assert_eq!(artifact.source, reordered.source);
}

#[test]
fn nominal_unit_keeps_imported_dictionary_closures_cold_effects_and_file_finalizers() {
    let (modules, entry) = project("imported-unit-package");
    let references = modules.iter().collect::<Vec<_>>();
    for (name, exit, stdout, trace) in [
        (
            "main",
            0,
            include_bytes!("../../../examples/spec/fixtures/rust-backend-projects/imported-unit-package/expected.stdout").as_slice(),
            None,
        ),
        ("failed", 1, b"".as_slice(), Some("acquire\nbody: ()\nrelease\n")),
        ("crash", 70, b"".as_slice(), Some("acquire\nbody: ()\nrelease\n")),
    ] {
        let artifact = emit_rust_project(&references, &entry, name).unwrap();
        assert_eq!(artifact.metadata.modules.len(), 3);
        execute(&artifact.source, stdout, exit, trace);
    }
}

#[test]
fn a_transitive_type_reexport_preserves_its_definition_and_local_alias() {
    let (modules, entry) = project("imported-unit-reexport");
    let artifact = emit_rust_project(&modules.iter().collect::<Vec<_>>(), &entry, "main").unwrap();
    assert_eq!(artifact.metadata.modules.len(), 3);
    execute(&artifact.source, include_bytes!("../../../examples/spec/fixtures/rust-backend-projects/imported-unit-reexport/expected.stdout"), 0, None);
}

#[test]
fn imported_unit_alias_requires_its_checked_provider_owner_and_kind() {
    let (modules, entry) = project("imported-unit-identity");
    for mutation in 0..8 {
        let mut changed = modules.clone();
        let main = changed.iter_mut().find(|m| m.module == entry).unwrap();
        let nominal = main
            .external_type_bindings
            .iter()
            .position(|b| b.canonical == NOMINAL_UNIT)
            .unwrap();
        let provider = main.external_type_bindings[nominal]
            .provider
            .as_ref()
            .unwrap();
        assert_eq!(provider.module, NOMINAL_UNIT_PROVIDER);
        assert_eq!(provider.export, "Unit");
        let primitive = main
            .external_type_bindings
            .iter()
            .position(|b| b.canonical == "std/prelude::Unit")
            .unwrap();
        match mutation {
            0 => {
                main.external_type_bindings[nominal]
                    .provider
                    .as_mut()
                    .unwrap()
                    .module = "counterfeit/provider".into()
            }
            1 => {
                main.external_type_bindings[primitive]
                    .provider
                    .as_mut()
                    .unwrap()
                    .export = "Other".into()
            }
            2 => main.external_type_bindings[nominal].arity = Some(1),
            3 => main.external_type_bindings[primitive].arity = Some(1),
            4 => {
                let mut duplicate = main.module_dependencies[0].imports[0].clone();
                duplicate.canonical = "counterfeit::Unit".into();
                main.module_dependencies[0].imports.push(duplicate);
            }
            5 => main.external_type_bindings[nominal].canonical = "counterfeit::Unit".into(),
            6 => {
                let counterfeit = "counterfeit::Unit".to_owned();
                main.module_dependencies[0].imports[0].canonical = counterfeit.clone();
                main.external_type_bindings[nominal].canonical = counterfeit;
            }
            7 => {
                main.external_type_bindings[primitive].canonical = "std/counterfeit::Unit".into();
                main.external_type_bindings[primitive]
                    .provider
                    .as_mut()
                    .unwrap()
                    .module = "std/counterfeit".into();
            }
            _ => unreachable!(),
        }
        let error =
            emit_rust_project(&changed.iter().collect::<Vec<_>>(), &entry, "main").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.module.as_deref(), Some(entry.as_str()));
        let origin = error.origin.expect("checked type import span");
        assert_eq!(origin.source, "main.ssrg");
        assert_eq!((origin.start, origin.end), (9, 13));
    }
}

#[test]
fn matching_import_metadata_cannot_forge_an_unrelated_source_type_provider() {
    let (mut modules, entry) = project("imported-unit-identity");
    let mut unrelated = modules[0].clone();
    unrelated.module = format!("{}::unrelated", entry);
    unrelated.structs.clear();
    unrelated.functions.clear();
    let provider = unrelated.module.clone();
    modules.insert(1, unrelated);
    let main = modules
        .iter_mut()
        .find(|module| module.module == entry)
        .unwrap();
    let mut real_dependency = main.module_dependencies[0].clone();
    real_dependency
        .imports
        .retain(|import| import.namespace != "type");
    main.module_dependencies[0].module = provider.clone();
    main.module_dependencies[0].specifier = "./unrelated".into();
    main.module_dependencies[0]
        .imports
        .retain(|import| import.namespace == "type");
    main.module_dependencies.push(real_dependency);
    let nominal = main
        .external_type_bindings
        .iter_mut()
        .find(|binding| binding.canonical == NOMINAL_UNIT)
        .unwrap();
    assert_eq!(
        nominal.provider.as_ref().unwrap().module,
        NOMINAL_UNIT_PROVIDER
    );
    assert_eq!(nominal.provider.as_ref().unwrap().export, "Unit");
    nominal.provider.as_mut().unwrap().module = provider;
    let error = emit_rust_project(&modules.iter().collect::<Vec<_>>(), &entry, "main").unwrap_err();
    assert!(error.message.contains("provider owner"), "{error:?}");
    assert_eq!(error.module.as_deref(), Some(entry.as_str()));
    let origin = error.origin.unwrap();
    assert_eq!((origin.start, origin.end), (9, 13));
}

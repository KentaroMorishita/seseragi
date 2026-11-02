use serde_json::Value;
use seseragi_driver::{compile_module, CompileInput, CompiledModule};
use seseragi_project::{standard_module_interfaces, BuildProfile};
use seseragi_syntax::{InterfaceExport, InterfaceType, ModuleInterface};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[path = "../../../test-support/temporary.rs"]
mod temporary;

const SOURCE: &str =
    include_str!("../../../../examples/spec/fixtures/rust-backend/stdlib-map-parity.ssrg");
const STDOUT: &str =
    include_str!("../../../../examples/spec/fixtures/rust-backend/stdlib-map-parity.stdout");

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        Self(temporary::temporary_directory("seseragi-stdlib-parity"))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
}

fn map_interface() -> ModuleInterface {
    standard_module_interfaces()
        .into_iter()
        .find(|interface| interface.module == "std/map")
        .unwrap()
}

fn functions(interface: &ModuleInterface) -> impl Iterator<Item = &InterfaceExport> {
    interface
        .exports
        .iter()
        .filter(|export| export.declaration_kind.as_deref() == Some("function"))
}

fn signature(type_ref: &InterfaceType) -> (Vec<&InterfaceType>, &InterfaceType) {
    let mut parameters = Vec::new();
    let mut result = type_ref;
    while let InterfaceType::Function {
        parameter,
        result: next,
    } = result
    {
        parameters.push(parameter.as_ref());
        result = next;
    }
    (parameters, result)
}

fn typescript_type(type_ref: &InterfaceType, parameters: &[String]) -> String {
    match type_ref {
        InterfaceType::Named { name, arguments } => {
            let target = match name.as_str() {
                "Unit" => "Unit",
                "Int" => "number",
                "Bool" => "boolean",
                "String" => "string",
                "Map" => "runtime.Map",
                "Array" => "ReadonlyArray",
                "Maybe" => "Maybe",
                parameter if parameters.iter().any(|name| name == parameter) => parameter,
                _ => panic!("no TypeScript parity type for {type_ref:?}"),
            };
            if arguments.is_empty() {
                target.into()
            } else {
                format!(
                    "{target}<{}>",
                    arguments
                        .iter()
                        .map(|argument| typescript_type(argument, parameters))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        InterfaceType::Function { parameter, result } => format!(
            "((value: {}) => {})",
            typescript_type(parameter, parameters),
            typescript_type(result, parameters)
        ),
        InterfaceType::Tuple { elements } => format!(
            "readonly [{}]",
            elements
                .iter()
                .map(|element| typescript_type(element, parameters))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => panic!("no TypeScript parity type for {type_ref:?}"),
    }
}

fn typescript_witnesses(interface: &ModuleInterface) -> String {
    let mut source = String::from(
        "import * as runtime from '@seseragi/runtime/map';\n\
         import type { Iterable } from '@seseragi/runtime/collection';\n\
         import type { Eq } from '@seseragi/runtime/equality';\n\
         import type { Hash } from '@seseragi/runtime/hash';\n\
         import type { Unit } from '@seseragi/runtime/effect';\n\
         import type { Maybe } from '@seseragi/runtime/sum';\n",
    );
    for export in functions(interface) {
        let parameters = export
            .scheme
            .type_parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect::<Vec<_>>();
        let generics = if parameters.is_empty() {
            String::new()
        } else {
            format!("<{}>", parameters.join(", "))
        };
        let mut arguments = Vec::new();
        for constraint in &export.scheme.constraints {
            assert_eq!(
                constraint.trait_identity.as_deref(),
                Some(format!("std/prelude::{}", constraint.name).as_str())
            );
            assert!(matches!(
                constraint.name.as_str(),
                "Iterable" | "Eq" | "Hash"
            ));
            arguments.push(format!(
                "{}<{}>",
                constraint.name,
                constraint
                    .arguments
                    .iter()
                    .map(|argument| typescript_type(argument, &parameters))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        let (ordinary, result) = signature(&export.scheme.type_ref);
        arguments.extend(
            ordinary
                .iter()
                .map(|argument| typescript_type(argument, &parameters)),
        );
        let names = (0..arguments.len())
            .map(|index| format!("argument{index}"))
            .collect::<Vec<_>>();
        let declarations = names
            .iter()
            .zip(arguments)
            .map(|(name, type_ref)| format!("{name}: {type_ref}"))
            .collect::<Vec<_>>()
            .join(", ");
        source.push_str(&format!(
            "export function check_{}{generics}({declarations}): {} {{\n\
             return runtime.{}{generics}({});\n}}\n",
            export.name,
            typescript_type(result, &parameters),
            export.name,
            names.join(", ")
        ));
    }
    source
}

fn check_probe_coverage(core: &Value, interface: &ModuleInterface) -> Result<(), String> {
    fn calls<'a>(value: &'a Value, prefix: &str, found: &mut Vec<&'a Value>) {
        match value {
            Value::Object(fields) => {
                if fields.get("kind").and_then(Value::as_str) == Some("call")
                    && fields
                        .get("callee")
                        .and_then(Value::as_str)
                        .is_some_and(|callee| callee.starts_with(prefix))
                {
                    found.push(value);
                }
                for child in fields.values() {
                    calls(child, prefix, found);
                }
            }
            Value::Array(elements) => {
                for child in elements {
                    calls(child, prefix, found);
                }
            }
            _ => {}
        }
    }

    let expected = functions(interface)
        .map(|export| (export.symbol.as_str(), export))
        .collect::<BTreeMap<_, _>>();
    let mut found = Vec::new();
    calls(core, &format!("{}::", interface.module), &mut found);
    let mut covered = BTreeSet::new();
    for call in found {
        let callee = call["callee"].as_str().unwrap();
        let export = expected
            .get(callee)
            .ok_or_else(|| format!("probe calls an unknown API: {callee}"))?;
        covered.insert(callee);
        if call["arguments"].as_array().map(Vec::len)
            != Some(signature(&export.scheme.type_ref).0.len())
        {
            return Err(format!("probe argument count mismatch: {callee}"));
        }
        let dictionaries = call["evidence"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|evidence| evidence["constraint"]["traitIdentity"].as_str())
            .collect::<Vec<_>>();
        let constraints = export
            .scheme
            .constraints
            .iter()
            .map(|constraint| constraint.trait_identity.as_deref())
            .collect::<Vec<_>>();
        if dictionaries != constraints {
            return Err(format!("probe dictionary order mismatch: {callee}"));
        }
    }
    let missing = expected
        .keys()
        .filter(|symbol| !covered.contains(**symbol))
        .copied()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!("APIs missing from execution probe: {missing:?}"));
    }
    Ok(())
}

fn compile(profile: BuildProfile) -> CompiledModule {
    compile_module(
        CompileInput::new("stdlib-map-parity.ssrg", "stdlib/map-parity", SOURCE)
            .with_profile(profile),
    )
    .unwrap_or_else(|error| panic!("{error:#?}"))
}

fn successful(output: Output) -> Output {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn typecheck(directory: &Path, files: &[&str]) -> Output {
    let config = serde_json::json!({
        "compilerOptions": {
            "strict": true, "noEmit": true, "skipLibCheck": false,
            "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
            "allowImportingTsExtensions": true, "types": ["node"],
            "typeRoots": [root().join("node_modules/@types")],
            "lib": ["ES2023", "DOM", "DOM.Iterable"]
        },
        "files": files
    });
    fs::write(
        directory.join("tsconfig.json"),
        serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    Command::new("node")
        .arg(root().join("node_modules/typescript/bin/tsc"))
        .arg("--project")
        .arg(directory.join("tsconfig.json"))
        .current_dir(directory)
        .output()
        .unwrap()
}

#[test]
fn map_all_17_functions_match_strict_typescript_and_both_runtimes() {
    let scratch = Scratch::new();
    let interface = map_interface();
    assert_eq!(functions(&interface).count(), 17);
    for (name, profile) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let compiled = compile(profile);
        check_probe_coverage(
            &serde_json::to_value(&compiled.core_ir).unwrap(),
            &interface,
        )
        .unwrap();
        let directory = scratch.0.join(name);
        seseragi_runtime::build_main(
            &compiled,
            &directory,
            seseragi_runtime::BuildTarget::Process,
        )
        .unwrap();
        seseragi_runtime::stage_typescript_package(&directory).unwrap();
        fs::write(directory.join("raw.ts"), &compiled.generated.typescript).unwrap();
        fs::write(
            directory.join("witnesses.ts"),
            typescript_witnesses(&interface),
        )
        .unwrap();
        successful(typecheck(&directory, &["raw.ts", "witnesses.ts"]));
        let manifest: Value =
            serde_json::from_slice(&fs::read(directory.join("artifact-manifest.json")).unwrap())
                .unwrap();
        let output = successful(
            Command::new("bun")
                .arg(directory.join(manifest["entry"].as_str().unwrap()))
                .current_dir(&directory)
                .output()
                .unwrap(),
        );
        assert_eq!(
            std::str::from_utf8(&output.stdout).unwrap(),
            STDOUT,
            "{name}"
        );
        assert!(output.stderr.is_empty(), "{output:?}");
        if profile == BuildProfile::Development {
            let artifact =
                seseragi_backend_rust::emit_rust_module(&compiled.core_ir, "main").unwrap();
            fs::write(scratch.0.join("program.rs"), artifact.source).unwrap();
        }
    }
    let library = scratch.0.join("libseseragi_native_runtime.rlib");
    successful(
        Command::new("rustc")
            .args([
                "--edition=2021",
                "--crate-type=rlib",
                "--crate-name=seseragi_native_runtime",
            ])
            .arg(root().join("crates/seseragi-native-runtime/src/lib.rs"))
            .arg("-o")
            .arg(&library)
            .current_dir(&scratch.0)
            .output()
            .unwrap(),
    );
    let executable = scratch
        .0
        .join(format!("program{}", std::env::consts::EXE_SUFFIX));
    successful(
        Command::new("rustc")
            .arg("--edition=2021")
            .arg("--extern")
            .arg(format!("seseragi_native_runtime={}", library.display()))
            .arg(scratch.0.join("program.rs"))
            .arg("-o")
            .arg(&executable)
            .current_dir(&scratch.0)
            .output()
            .unwrap(),
    );
    let output = successful(
        Command::new(executable)
            .current_dir(&scratch.0)
            .output()
            .unwrap(),
    );
    assert_eq!(std::str::from_utf8(&output.stdout).unwrap(), STDOUT);
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn map_probe_rejects_missing_apis_arguments_and_dictionary_order() {
    let compiled = compile(BuildProfile::Development);
    let core = serde_json::to_value(&compiled.core_ir).unwrap();
    let interface = map_interface();
    check_probe_coverage(&core, &interface).unwrap();
    let mut missing = interface.clone();
    let mut future = functions(&interface).next().unwrap().clone();
    future.name = "future".into();
    future.symbol = "std/map::future".into();
    missing.exports.push(future);
    assert!(check_probe_coverage(&core, &missing)
        .unwrap_err()
        .contains("std/map::future"));
    missing = interface.clone();
    missing.exports.retain(|export| export.name != "get");
    assert!(check_probe_coverage(&core, &missing)
        .unwrap_err()
        .contains("unknown API: std/map::get"));
    for dictionary in [false, true] {
        let mut changed = interface.clone();
        let export = changed
            .exports
            .iter_mut()
            .find(|export| export.name == "fromEntries")
            .unwrap();
        if dictionary {
            export.scheme.constraints.swap(0, 1);
        } else {
            let InterfaceType::Function { result, .. } = &export.scheme.type_ref else {
                panic!()
            };
            export.scheme.type_ref = *result.clone();
        }
        let error = check_probe_coverage(&core, &changed).unwrap_err();
        assert!(error.contains(if dictionary {
            "dictionary order mismatch: std/map::fromEntries"
        } else {
            "argument count mismatch: std/map::fromEntries"
        }));
    }
}

#[test]
fn map_typescript_witnesses_reject_missing_exports_arguments_results_and_dictionaries() {
    let scratch = Scratch::new();
    seseragi_runtime::stage_typescript_package(&scratch.0).unwrap();
    for (mutation, expected) in [
        ("missing-export", "TS2339"),
        ("missing-argument", "TS2554"),
        ("extra-argument", "TS2554"),
        ("wrong-argument", "TS2345"),
        ("wrong-result", "TS2322"),
        ("missing-dictionary", "TS2554"),
    ] {
        let mut interface = map_interface();
        let export = interface
            .exports
            .iter_mut()
            .find(|export| export.name == "fromEntries")
            .unwrap();
        match mutation {
            "missing-export" => export.name = "missingFromEntries".into(),
            "missing-argument" => {
                let InterfaceType::Function { result, .. } = &export.scheme.type_ref else {
                    panic!()
                };
                export.scheme.type_ref = *result.clone();
            }
            "extra-argument" => {
                export.scheme.type_ref = InterfaceType::Function {
                    parameter: Box::new(InterfaceType::Named {
                        name: "Unit".into(),
                        arguments: vec![],
                    }),
                    result: Box::new(export.scheme.type_ref.clone()),
                };
            }
            "wrong-result" => {
                let InterfaceType::Function { result, .. } = &mut export.scheme.type_ref else {
                    panic!()
                };
                **result = InterfaceType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                };
            }
            "wrong-argument" => {
                let InterfaceType::Function { parameter, .. } = &mut export.scheme.type_ref else {
                    panic!()
                };
                **parameter = InterfaceType::Named {
                    name: "String".into(),
                    arguments: vec![],
                };
            }
            "missing-dictionary" => {
                export.scheme.constraints.pop();
            }
            _ => unreachable!(),
        }
        fs::write(
            scratch.0.join("witnesses.ts"),
            typescript_witnesses(&interface),
        )
        .unwrap();
        let output = typecheck(&scratch.0, &["witnesses.ts"]);
        assert!(!output.status.success(), "accepted {mutation}");
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(expected),
            "{mutation}: {output:?}"
        );
    }
}

#[test]
fn map_execution_detects_dictionary_reordering_even_when_typescript_accepts_it() {
    let scratch = Scratch::new();
    let directory = scratch.0.join("typescript");
    let compiled = compile(BuildProfile::Development);
    seseragi_runtime::build_main(
        &compiled,
        &directory,
        seseragi_runtime::BuildTarget::Process,
    )
    .unwrap();
    seseragi_runtime::stage_typescript_package(&directory).unwrap();
    let runtime_path = directory.join("node_modules/@seseragi/runtime/src/map.ts");
    let runtime = fs::read_to_string(&runtime_path).unwrap();
    let iterable = "  iterable: CollectionIterable<C, readonly [K, V]> | RuntimeDictionary,";
    let eq = "  eq: Eq<K> | RuntimeDictionary,";
    let reordered = runtime.replacen(
        &format!("{iterable}\n{eq}"),
        &format!("{eq}\n{iterable}"),
        1,
    );
    assert_ne!(reordered, runtime);
    fs::write(runtime_path, reordered).unwrap();
    fs::write(directory.join("raw.ts"), &compiled.generated.typescript).unwrap();
    fs::write(
        directory.join("witnesses.ts"),
        typescript_witnesses(&map_interface()),
    )
    .unwrap();
    successful(typecheck(&directory, &["raw.ts", "witnesses.ts"]));
    let output = Command::new("bun")
        .arg(directory.join("entry.ts"))
        .current_dir(&directory)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_ne!(output.stdout, STDOUT.as_bytes());
    assert!(String::from_utf8_lossy(&output.stderr).contains("runtime defect"));
}

#[test]
fn map_native_rejects_missing_apis_arguments_results_and_dictionaries() {
    use seseragi_lowering::{CoreExpr, CoreType};

    let compiled = compile_module(CompileInput::new(
        "map-negative.ssrg",
        "stdlib/map-negative",
        "import * as maps from \"std/map\"\n\
         pub fn probe -> maps.Map<Int, String> = maps.fromEntries [(1, \"one\")]\n",
    ))
    .unwrap();
    seseragi_backend_rust::emit_rust_module(&compiled.core_ir, "probe").unwrap();
    for mutation in ["missing-api", "argument", "result", "dictionary", "order"] {
        let mut core = compiled.core_ir.clone();
        let CoreExpr::Call {
            callee,
            arguments,
            evidence,
            type_ref,
            origin,
            ..
        } = &mut core.functions[0].body
        else {
            panic!()
        };
        let expected_origin = origin.clone();
        match mutation {
            "missing-api" => *callee = "std/map::missingFromEntries".into(),
            "argument" => arguments.clear(),
            "result" => {
                *type_ref = CoreType::Named {
                    name: "std/prelude::Bool".into(),
                    arguments: vec![],
                };
            }
            "dictionary" => {
                evidence.pop();
            }
            "order" => evidence.swap(0, 1),
            _ => unreachable!(),
        }
        let error = seseragi_backend_rust::emit_rust_module(&core, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "{mutation}");
        assert_eq!(error.origin, Some(expected_origin), "{mutation}");
    }
}

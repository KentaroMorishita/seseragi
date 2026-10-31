use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_module, CompileInput};
use seseragi_lowering::{CoreInstanceEvidence, CoreInstanceImplementation, CoreType};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "seseragi-derived-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn compare(source: &str, expected: &str) {
    compare_outputs(source, Some(expected), None, None);
}

fn symbol(name: &str) -> String {
    format!(
        "v_{}",
        format!("native/derived::{name}")
            .bytes()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}

fn compare_outputs(
    source: &str,
    expected: Option<&str>,
    native_main: Option<&str>,
    ts_main: Option<&str>,
) {
    let compiled = compile_module(CompileInput::new("derived.ssrg", "native/derived", source))
        .unwrap_or_else(|e| panic!("{e:?}\n{source}"));
    let artifact = emit_rust_module(&compiled.core_ir, "probe").unwrap_or_else(|e| panic!("{e:?}"));
    let scratch = Scratch::new();
    let rust = scratch.0.join("program.rs");
    let binary = scratch
        .0
        .join(format!("program{}", std::env::consts::EXE_SUFFIX));
    let generated = if let Some(main) = native_main {
        format!(
            "{}\n{main}",
            &artifact.source[..artifact.source.rfind("\nfn main()").unwrap()]
        )
    } else {
        artifact.source
    };
    fs::write(&rust, generated).unwrap();
    let output = Command::new("rustc")
        .args(["--edition=2021", "-O"])
        .arg(&rust)
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let native = Command::new(binary).env("PATH", "").output().unwrap();
    assert!(
        native.status.success(),
        "{}",
        String::from_utf8_lossy(&native.stderr)
    );
    if let Some(expected) = expected {
        assert_eq!(String::from_utf8_lossy(&native.stdout), expected);
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf();
    let typescript = compiled.generated.typescript.replace(
        "\"@seseragi/runtime/",
        &format!("\"{}/runtime/ts/src/", root.display()),
    );
    let ts = scratch.0.join("program.ts");
    fs::write(
        &ts,
        format!(
            "{typescript}\n{}\n",
            ts_main.unwrap_or("console.log(probe());")
        ),
    )
    .unwrap();
    let ts_output = Command::new("bun").arg(&ts).output().unwrap();
    assert!(
        ts_output.status.success(),
        "{}",
        String::from_utf8_lossy(&ts_output.stderr)
    );
    assert_eq!(
        native.stdout,
        ts_output.stdout,
        "native: {}\nTS: {}",
        String::from_utf8_lossy(&native.stdout),
        String::from_utf8_lossy(&ts_output.stdout)
    );
}

const TYPES: &str = r#"
type Color deriving Eq, Ord, Hash, Show, Debug = | Red | Green | Blue
type Tree<A> deriving Eq, Ord, Hash, Show, Debug = | Leaf A | Branch (Tree<A>, Tree<A>)
struct Point deriving Eq, Ord, Hash, Show, Debug { z: Int, a: Int }
newtype UserId deriving Eq, Ord, Hash, Show, Debug = Int
type Payload<A> deriving Eq, Ord, Hash, Show, Debug = | Empty | Loaded A
"#;

#[test]
fn eq_uses_selected_children_for_cases_recursive_generic_struct_newtype_and_record_values() {
    compare(
        &format!(
            r#"{TYPES}
fn same<A> left: A -> right: A -> Bool where Eq<A> = {{ let saved: A -> A -> Bool = (==); saved left right }}
pub fn probe -> String = show (
  Red == Red, Red == Green,
  Loaded 42 == Loaded 42, eq<Payload<Int>> Empty (Loaded 42),
  Branch (Leaf 1, Leaf 2) == Branch (Leaf 1, Leaf 2),
  Branch (Leaf 1, Leaf 2) == Branch (Leaf 1, Leaf 3),
  same (Point {{ z: 1, a: 2 }}) (Point {{ z: 1, a: 2 }}),
  UserId 42 == UserId 42,
  {{ z: 1, a: "雪🙂" }} == {{ a: "雪🙂", z: 1 }},
  {{ z: 1 }} == {{ z: 2 }}
)
"#
        ),
        "(True, False, True, False, True, False, True, True, True, False)\n",
    );
}

#[test]
fn ord_compares_declared_cases_and_fields_lexicographically_and_keeps_eq_parent() {
    compare(
        &format!(
            r#"{TYPES}
fn before<A> left: A -> right: A -> Bool where Ord<A> = {{ let saved = compare; match saved left right {{ Less -> True; _ -> False }} }}
pub fn probe -> String = show (
  Red < Green, Green < Blue, Blue > Red,
  before<Payload<Int>> Empty (Loaded (-999)), Loaded 1 < Loaded 2,
  Branch (Leaf 1, Leaf 9) < Branch (Leaf 2, Leaf 0),
  Branch (Leaf 1, Leaf 2) < Branch (Leaf 1, Leaf 3),
  before (Point {{ z: 0, a: 99 }}) (Point {{ z: 1, a: 0 }}),
  Point {{ z: 1, a: 0 }} < Point {{ z: 1, a: 1 }},
  UserId (-1) < UserId 0, match compare (Leaf 2) (Leaf 2) {{ Equal -> True; _ -> False }}
)
"#
        ),
        "(True, True, True, True, True, True, True, True, True, True, True)\n",
    );
}

#[test]
fn hash_matches_ts_seeds_wrapping_and_eq_for_recursive_generic_struct_and_newtype_values() {
    compare(
        &format!(
            r#"{TYPES}
fn agrees<A> left: A -> right: A -> Bool where Eq<A>, Hash<A> = left == right && hash left == hash right
pub fn probe -> String = show (
  hash Red, hash Green, hash Blue, hash<Payload<Int>> Empty, hash (Loaded 9007199254740991),
  hash (Point {{ z: -1, a: 9007199254740991 }}), hash (UserId 9007199254740991),
  hash (Branch (Leaf (-4294967296), Leaf 9007199254740991)),
  agrees (Branch (Leaf 1, Leaf 2)) (Branch (Leaf 1, Leaf 2)),
  agrees (Point {{ z: 1, a: 2 }}) (Point {{ z: 1, a: 2 }}),
  agrees (UserId 42) (UserId 42)
)
"#
        ),
        "(0, 1, 2, 0, -33555238, 620918998, 9007199254740991, 860300866, True, True, True)\n",
    );
}

#[test]
fn show_keeps_constructors_payload_nesting_and_declaration_order() {
    compare(&format!(r#"{TYPES}
struct Message deriving Show {{ z: String, a: Payload<String> }}
pub fn probe -> String = show (
  Red, Loaded "雪🙂", Branch (Leaf 1, Branch (Leaf 2, Leaf 3)),
  Point {{ z: 9, a: 1 }}, UserId 42,
  Message {{ z: "a\n\"b", a: Loaded "text" }},
  {{ z: Loaded 1, a: "text" }}
)
"#), "(Red, Loaded 雪🙂, Branch (Leaf 1, Branch (Leaf 2, Leaf 3)), Point { z: 9, a: 1 }, UserId 42, Message { z: a\n\"b, a: Loaded text }, { a: text, z: Loaded 1 })\n");
}

#[test]
fn debug_keeps_wrappers_and_quotes_escaped_strings_in_structs_records_and_payloads() {
    compare(&format!(r#"{TYPES}
struct Message deriving Debug {{ z: String, a: Payload<String> }}
pub fn probe -> String = debug (
  Red, Loaded "雪🙂", Branch (Leaf 1, Leaf 2),
  Point {{ z: 9, a: 1 }}, UserId 42,
  Message {{ z: "a\n\"b", a: Loaded "text" }},
  {{ z: Loaded "\t", a: "text" }}
)
"#), "(Red, Loaded \"雪🙂\", Branch (Leaf 1, Leaf 2), Point { z: 9, a: 1 }, UserId 42, Message { z: \"a\\n\\\"b\", a: Loaded \"text\" }, { a: \"text\", z: Loaded \"\\t\" })\n");
}

#[test]
fn generic_structs_and_newtypes_delegate_to_user_methods_and_preserve_phantom_parameters() {
    compare(r#"
type Key = | Key Int
instance Eq<Key> { fn eq left: Key -> right: Key -> Bool = match (left, right) { (Key a, Key b) -> a % 10 == b % 10 } }
instance Ord<Key> { fn compare left: Key -> right: Key -> Ordering = match (left, right) { (Key a, Key b) -> compare (a % 10) (b % 10) } }
instance Hash<Key> { fn hash value: Key -> Int = match value { Key a -> a % 10 } }
instance Show<Key> { fn show value: Key -> String = match value { Key a -> `key:${a}` } }
instance Debug<Key> { fn debug value: Key -> String = match value { Key a -> `debug:${a}` } }
struct Box<A> deriving Eq, Ord, Hash, Show, Debug { item: A }
newtype Wrapped<A> deriving Eq, Ord, Hash, Show, Debug = A
type Phantom<A> deriving Eq, Ord, Hash, Show, Debug = | Phantom
struct EmptyStruct deriving Eq, Ord, Hash, Show, Debug {}
pub fn probe -> String = {
  let a = Box { item: Key 1 }
  let b = Box { item: Key 11 }
  let phantom: Phantom<Float> = Phantom
  let savedShow: Box<Key> -> String = show
  let savedDebug: Box<Key> -> String = debug
  let savedHash: Box<Key> -> Int = hash
  let savedEq: Box<Key> -> Box<Key> -> Bool = eq
  let savedCompare: Box<Key> -> Box<Key> -> Ordering = compare
  show (savedEq a b, savedHash a == savedHash b, a < Box { item: Key 2 },
    match savedCompare a b { Equal -> True; _ -> False },
    savedShow a, savedDebug b, Wrapped (Key 1) == Wrapped (Key 11),
    hash (Wrapped (Key 1)), show (Wrapped (Key 1)), debug (Wrapped (Key 1)),
    phantom == phantom, hash phantom, show phantom,
    EmptyStruct {} == EmptyStruct {}, hash (EmptyStruct {}), show (EmptyStruct {}))
}
"#, "(True, True, True, True, Box { item: key:1 }, Box { item: debug:11 }, True, 1, Wrapped key:1, Wrapped debug:1, True, 0, Phantom, True, 0, EmptyStruct {})\n");
}

#[test]
fn records_keep_optional_presence_and_display_metadata() {
    compare(r#"
alias Row = { z?: Int, a: String }
type Event deriving Eq, Show, Debug = | Idle | Named Row
pub fn probe -> String = {
  let absent: Row = { a: "text" }
  let present: Row = { z: 1, a: "text" }
  let absentAgain: Row = { a: "text" }
  show (absent == absentAgain, absent == present, present == absent,
    show absent, debug present, Named absent == Named absentAgain,
    Named present != Named absent, show (Named absent), debug (Named present),
    Idle == Idle)
}
"#, "(True, False, False, { a: text, z?: Nothing }, { a: \"text\", z?: Just 1 }, True, True, Named { a: text, z?: Nothing }, Named { a: \"text\", z?: Just 1 }, True)\n");
}

#[test]
fn derived_factories_reject_forged_heads_members_children_and_wasm() {
    let compiled = compile_module(CompileInput::new(
        "derived.ssrg",
        "native/derived",
        &format!("{TYPES}\npub fn probe -> Int = 0"),
    ))
    .unwrap();
    let core = compiled.core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for index in 0..core.instances.len() {
        for mutation in 0..7 {
            let mut bad = core.clone();
            let instance = &mut bad.instances[index];
            match mutation {
                0 => instance.type_identity = None,
                1 => instance.arguments[0] = CoreType::Hole,
                2 => instance.identity = "fake::Eq<X>".into(),
                3 => instance.supertrait_count += 1,
                _ => {
                    let (symbol, payloads) = match &mut instance.implementation {
                        CoreInstanceImplementation::DerivedShow {
                            adt_symbol,
                            payload_evidence,
                        }
                        | CoreInstanceImplementation::DerivedStructural {
                            adt_symbol,
                            payload_evidence,
                            ..
                        } => (adt_symbol, payload_evidence),
                        _ => panic!(),
                    };
                    if mutation == 4 {
                        *symbol = "fake::X".into();
                    } else if payloads.is_empty() {
                        payloads.push(seseragi_lowering::CoreShowPayloadEvidence {
                            variant_symbol: "ghost".into(),
                            type_identity: "Int".into(),
                            evidence: CoreInstanceEvidence::Parameter { index: 99 },
                        });
                    } else if mutation == 5 {
                        payloads[0].evidence = CoreInstanceEvidence::Parameter { index: 99 };
                    } else {
                        payloads[0].variant_symbol = "ghost".into();
                    }
                }
            }
            let error = emit_rust_module(&bad, "probe").unwrap_err();
            assert_eq!(
                error.origin.as_ref(),
                Some(&core.instances[index].origin),
                "{index}:{mutation}:{error:?}"
            );
        }
    }
    let error = emit_program_wasm(&[&core], "native/derived", &["probe".into()]).unwrap_err();
    assert!(error.message.contains("target-specific"), "{error:?}");
}

#[test]
fn derived_standard_host_errors_match_ts_variants_messages_and_debug_redaction() {
    let source = r#"
import * as stdin from "std/stdin"
import * as text from "std/text"
type AppError deriving Show, Debug = | Input StdinError | Output ConsoleError
struct Envelope<A> deriving Show, Debug { error: A }
newtype Wrapped<A> deriving Show, Debug = A
fn rendered<A> value: A -> String where Show<A>, Debug<A> = show value + "/" + debug value
pub fn describe error: ConsoleError -> String = text.join "|" [
  rendered (Input stdin.StdinUnavailable), rendered (Input stdin.StdinReadFailure),
  rendered (Input stdin.ConcurrentStdinRead), rendered (Input (stdin.InvalidStdinUtf8 { offset: 42 })),
  rendered (Input (stdin.StdinLineTooLong { limitBytes: 1024 })), rendered (Input stdin.StdinPositionOverflow),
  rendered (Output error), rendered (Envelope { error }), rendered (Wrapped error)
]
pub fn probe -> String = "unused"
"#;
    let message = "host \"雪🙂\"\nsecret";
    let native = format!(
        r#"fn main() {{
        let error = V::Record(Shared::new(BTreeMap::from([("kind".into(), V::text("console-error")), ("message".into(), V::text({message:?}))])));
        let result = {}(vec![]).apply(error);
        println!("{{}}", result.render()); drop(result);
        clear_dictionaries(); clear_globals(); assert_eq!(LIVE.with(Cell::get), 0);
    }}"#,
        symbol("describe")
    );
    let ts = format!(
        "console.log(describe({{kind: \"console-error\", message: {}}}));",
        serde_json::to_string(message).unwrap()
    );
    let expected = format!("Input StdinUnavailable/Input StdinUnavailable|Input StdinReadFailure/Input StdinReadFailure|Input ConcurrentStdinRead/Input ConcurrentStdinRead|Input InvalidStdinUtf8 {{ offset: 42 }}/Input InvalidStdinUtf8 {{ offset: 42 }}|Input StdinLineTooLong {{ limitBytes: 1024 }}/Input StdinLineTooLong {{ limitBytes: 1024 }}|Input StdinPositionOverflow/Input StdinPositionOverflow|Output ConsoleError: {message}/Output ConsoleError {{ message: \"<redacted>\" }}|Envelope {{ error: ConsoleError: {message} }}/Envelope {{ error: ConsoleError {{ message: \"<redacted>\" }} }}|Wrapped ConsoleError: {message}/Wrapped ConsoleError {{ message: \"<redacted>\" }}\n");
    compare_outputs(source, Some(&expected), Some(&native), Some(&ts));
}

struct StandardErrors {
    types: std::collections::BTreeMap<String, (String, seseragi_syntax::InterfaceExport)>,
    constructors:
        std::collections::BTreeMap<String, Vec<(String, seseragi_syntax::InterfaceExport)>>,
}
impl StandardErrors {
    fn load() -> Self {
        let mut result = Self {
            types: Default::default(),
            constructors: Default::default(),
        };
        for module in seseragi_project::standard_module_interfaces() {
            for export in module.exports {
                if export.namespace == "type" {
                    result.types.insert(
                        export.symbol.clone(),
                        (module.module.clone(), export.clone()),
                    );
                }
                if let Some(owner) = &export.constructor_of {
                    result
                        .constructors
                        .entry(owner.clone())
                        .or_default()
                        .push((module.module.clone(), export));
                }
            }
        }
        result
    }
    fn payload(&self, ty: &seseragi_syntax::InterfaceType, module: &str) -> (String, String) {
        use seseragi_syntax::InterfaceType;
        match ty {
            InterfaceType::Named { name, arguments }
            | InterfaceType::ExternalNamed {
                canonical: name,
                arguments,
                ..
            } => {
                let simple = name.rsplit("::").next().unwrap();
                match simple {
                    "Int" => ("V::Int(42)".into(), "42".into()),
                    "Float" => ("V::Float(1.25)".into(), "1.25".into()),
                    "Bool" => ("V::Bool(true)".into(), "true".into()),
                    "Char" => ("V::Char('\\n')".into(), "\"\\n\"".into()),
                    "String" => {
                        let text = "雪🙂\"\n\0\u{85}";
                        (
                            format!("V::text({text:?})"),
                            serde_json::to_string(text).unwrap(),
                        )
                    }
                    "Unit" => ("V::Unit".into(), "undefined".into()),
                    "Maybe" => (
                        "V::adt(\"std/prelude::Nothing\", V::Unit)".into(),
                        "({tag: \"Nothing\"})".into(),
                    ),
                    "Array" => {
                        let (native, ts) = self.payload(&arguments[0], module);
                        (
                            format!("V::Array(Shared::new(vec![{native}]))"),
                            format!("[{ts}]"),
                        )
                    }
                    _ => {
                        let canonical = if name.contains("::") {
                            name.clone()
                        } else {
                            format!("{module}::{name}")
                        };
                        self.nominal(&canonical)
                    }
                }
            }
            InterfaceType::Record { fields, .. } => {
                let pairs: Vec<_> = fields
                    .iter()
                    .map(|f| {
                        let (native, ts) = self.payload(&f.type_ref, module);
                        (
                            format!("({:?}.into(), {native})", f.name),
                            format!("{}: {ts}", serde_json::to_string(&f.name).unwrap()),
                        )
                    })
                    .collect();
                (
                    format!(
                        "V::Record(Shared::new(BTreeMap::from([{}])))",
                        pairs
                            .iter()
                            .map(|p| p.0.as_str())
                            .collect::<Vec<_>>()
                            .join(",")
                    ),
                    format!(
                        "({{{}}})",
                        pairs
                            .iter()
                            .map(|p| p.1.as_str())
                            .collect::<Vec<_>>()
                            .join(",")
                    ),
                )
            }
            _ => panic!("unsupported test payload: {module}: {ty:?}"),
        }
    }
    fn constructor(
        &self,
        module: &str,
        export: &seseragi_syntax::InterfaceExport,
    ) -> (String, String) {
        if let seseragi_syntax::InterfaceType::Function { parameter, .. } = &export.scheme.type_ref
        {
            let (native, ts) = self.payload(parameter, module);
            (
                format!("V::adt({:?}, {native})", export.symbol),
                format!("({{tag: {:?}, value: {ts}}})", export.name),
            )
        } else {
            (
                format!("V::adt({:?}, V::Unit)", export.symbol),
                format!("({{tag: {:?}}})", export.name),
            )
        }
    }
    fn nominal(&self, name: &str) -> (String, String) {
        if name == "std/path::Path" {
            return (
                "native_path(\"tmp/雪🙂\")".into(),
                "__derivedTestPath(\"tmp/雪🙂\").value".into(),
            );
        }
        if matches!(name, "std/prelude::ConsoleError" | "std/log::LogError") {
            let kind = if name == "std/log::LogError" {
                "log-error"
            } else {
                "console-error"
            };
            return ("V::Record(Shared::new(BTreeMap::from([(\"kind\".into(), V::text(\"console-error\")), (\"message\".into(), V::text(\"secret雪🙂\\n\"))])))".replace("console-error", kind), "({kind: \"console-error\", message: \"secret雪🙂\\n\"})".replace("console-error", kind));
        }
        if let Some(constructors) = self.constructors.get(name) {
            let (module, export) = &constructors[0];
            return self.constructor(module, export);
        }
        let (module, export) = &self.types[name];
        self.payload(
            export
                .representation
                .as_ref()
                .unwrap_or_else(|| panic!("test representation: {name}")),
            module,
        )
    }
}

#[test]
fn every_standard_error_adt_variant_matches_ts_show_and_debug_through_a_derived_wrapper() {
    let catalog = include_str!("../../seseragi-lowering/src/display_ops.rs");
    let names: std::collections::BTreeSet<_> = catalog
        .split('"')
        .filter_map(|s| s.strip_prefix("Show<")?.strip_suffix('>'))
        .filter(|s| s.contains("Error") || s.ends_with("QueueClosed"))
        .collect();
    let errors = StandardErrors::load();
    let mut source = String::new();
    let mut native = String::from("fn main() {\n");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf();
    let mut ts = format!(
        "import {{ parse as __derivedTestPath }} from \"{}/runtime/ts/src/path.ts\";\n",
        root.display()
    );
    let mut count = 0;
    for (index, name) in names.into_iter().enumerate() {
        if name == "std/prelude::Js.Error" {
            continue;
        }
        let (module, export) = &errors.types[name];
        let debug = catalog.contains(&format!("\"Debug<{name}>\""));
        source.push_str(&format!("import {{ {} as Error{index} }} from {module:?}\ntype Wrapper{index} deriving Show{} = | Wrapped{index} Error{index}\npub fn describe{index} value: Error{index} -> String = show (Wrapped{index} value){}\n", export.name, if debug { ", Debug" } else { "" }, if debug { format!(" + \"|\" + debug (Wrapped{index} value)") } else { String::new() }));
        let values = if let Some(constructors) = errors.constructors.get(name) {
            constructors
                .iter()
                .map(|(module, export)| errors.constructor(module, export))
                .collect::<Vec<_>>()
        } else {
            vec![errors.nominal(name)]
        };
        for (native_value, ts_value) in values {
            native.push_str(&format!("{{ let value = {}(vec![]).apply({native_value}); println!(\"{{}}\", value.render()); }}\n", symbol(&format!("describe{index}"))));
            ts.push_str(&format!("console.log(describe{index}({ts_value}));\n"));
            count += 1;
        }
    }
    assert!(count >= 150, "{count} standard error variants");
    source.push_str("pub fn probe -> String = \"unused\"\n");
    native.push_str(
        "clear_dictionaries(); clear_globals(); assert_eq!(LIVE.with(Cell::get), 0); }\n",
    );
    compare_outputs(&source, None, Some(&native), Some(&ts));
}

#[test]
fn derived_eq_selects_standard_error_instances_and_dom_runtime_errors_keep_failure_evidence() {
    compare(r#"
import * as dec from "std/decimal"
import * as hex from "std/bytes/hex"
import * as dom from "std/web/dom"
type Failure deriving Eq, Show, Debug = | Decimal dec.DecimalParseError | Hex hex.HexDecodeError
type Dispatch<A> deriving Show, Debug = | Dispatch (dom.DomRuntimeError<A>)
pub fn probe -> String = show (
  Decimal (dec.InvalidDecimal { offset: 1 }) == Decimal (dec.InvalidDecimal { offset: 1 }),
  Decimal (dec.InvalidDecimal { offset: 1 }) == Decimal (dec.InvalidDecimal { offset: 2 }),
  Hex (hex.OddHexLength 3) == Hex (hex.OddHexLength 3),
  Hex (hex.OddHexLength 3) != Hex (hex.InvalidHexDigit { offset: 3 }),
  show (Dispatch (dom.DispatchFailure "雪🙂")), debug (Dispatch (dom.DispatchFailure "\n")),
  show<Dispatch<String>> (Dispatch (dom.DomFailure dom.DomTargetRemoved))
)
"#, "(True, False, True, True, Dispatch DispatchFailure 雪🙂, Dispatch DispatchFailure \"\\n\", Dispatch DomFailure DomTargetRemoved)\n");
}

#[test]
fn standard_error_dictionaries_reject_forged_types_children_dispatch_and_constructor_stages() {
    let source = "import * as stdin from \"std/stdin\"\ntype Failure deriving Show, Debug = | Input StdinError | Output ConsoleError\npub fn describe value: StdinError -> String = show value\npub fn probe -> String = show (stdin.InvalidStdinUtf8 { offset: 1 })\n";
    let core = compile_module(CompileInput::new("errors.ssrg", "native/derived", source))
        .unwrap()
        .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for index in 0..core.instances.len() {
        for mutation in 0..6 {
            let mut bad = core.clone();
            let instance = &mut bad.instances[index];
            let CoreInstanceImplementation::DerivedShow {
                payload_evidence, ..
            } = &mut instance.implementation
            else {
                panic!()
            };
            let CoreInstanceEvidence::Standard {
                identity,
                type_arguments,
                evidence_arguments,
            } = &mut payload_evidence[0].evidence
            else {
                panic!()
            };
            match mutation {
                0 => *identity = identity.replace("StdinError", "ConsoleError"),
                1 => *identity = identity.replace("std/prelude", "fake/prelude"),
                2 => *identity = identity.replace("Show", "Eq").replace("Debug", "Eq"),
                3 => type_arguments.push(CoreType::Hole),
                4 => evidence_arguments.push(seseragi_lowering::CoreCallEvidence {
                    constraint: seseragi_lowering::CoreInstanceConstraint {
                        name: "Show".into(),
                        trait_identity: Some("std/prelude::Show".into()),
                        arguments: vec![CoreType::Hole],
                    },
                    evidence: CoreInstanceEvidence::Parameter { index: 99 },
                }),
                _ => *identity = identity.replace("StdinError", "StdinError<A>"),
            }
            let error = emit_rust_module(&bad, "probe").unwrap_err();
            assert_eq!(
                error.origin.as_ref(),
                Some(&core.instances[index].origin),
                "{mutation}:{error:?}"
            );
        }
    }
    for mutation in 0..4 {
        let mut bad = core.clone();
        let function = bad
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::describe"))
            .unwrap();
        let seseragi_lowering::CoreExpr::Call {
            trait_dispatch,
            type_ref,
            evidence_argument_index,
            ..
        } = &mut function.body
        else {
            panic!()
        };
        match mutation {
            0 => trait_dispatch.as_mut().unwrap().trait_identity = "fake::Show".into(),
            1 => trait_dispatch.as_mut().unwrap().method = "debug".into(),
            2 => {
                *type_ref = CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                }
            }
            _ => *evidence_argument_index = Some(0),
        }
        assert!(emit_rust_module(&bad, "probe").is_err(), "{mutation}");
    }
    for mutation in 0..3 {
        let mut bad = core.clone();
        let function = bad
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let seseragi_lowering::CoreExpr::Call { arguments, .. } = &mut function.body else {
            panic!()
        };
        let seseragi_lowering::CoreExpr::Call {
            type_ref,
            evidence_argument_index,
            callee,
            ..
        } = &mut arguments[0]
        else {
            panic!()
        };
        match mutation {
            0 => {
                *type_ref = CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                }
            }
            1 => *evidence_argument_index = Some(0),
            _ => *callee = "fake/stdin::InvalidStdinUtf8".into(),
        }
        assert!(
            emit_rust_module(&bad, "probe").is_err(),
            "constructor:{mutation}"
        );
    }
    for class in ["Show", "Debug", "Eq"] {
        let source = format!("import * as dec from \"std/decimal\"\ntype Wrapped deriving {class} = | Wrapped dec.DecimalParseError\npub fn probe -> Int = 0");
        let core = compile_module(CompileInput::new("errors.ssrg", "native/derived", &source))
            .unwrap()
            .core_ir;
        assert!(emit_program_wasm(&[&core], "native/derived", &["probe".into()]).is_err());
    }
}

#[test]
fn nested_standard_error_documents_keep_ts_depth_limits() {
    let source = r#"
import * as process from "std/process"
type Chain<A> deriving Show, Debug = | Terminal A | Next (Chain<A>, Int)
type Wrapped deriving Show, Debug = | Wrapped process.ProcessError
fn nested depth: Int -> Chain<Wrapped> = if depth == 0 then Terminal (Wrapped (process.InvalidEnvironmentName "雪🙂\n")) else Next (nested (depth - 1), 0)
fn bare depth: Int -> Chain<process.ProcessError> = if depth == 0 then Terminal (process.InvalidEnvironmentName "雪🙂\n") else Next (bare (depth - 1), 0)
pub fn probe -> String = show (show (nested 126), show (nested 127), show (nested 128), debug (nested 127), show (bare 62), show (bare 63), debug (bare 63))
"#;
    compare_outputs(source, None, None, None);
}

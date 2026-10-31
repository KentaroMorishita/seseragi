use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn success(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn verify(source: &str, expected: &str) {
    verify_exit(source, expected, 0);
}

fn verify_exit(source: &str, expected: &str, exit: i32) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-collection-partials-")
        .tempdir()
        .unwrap();
    let input = scratch.path().join("main.ssrg");
    fs::write(&input, source).unwrap();
    for (profile, value) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let out = scratch.path().join(profile);
        success(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&input)
                .args(["--profile", profile, "--out-dir"])
                .arg(&out)
                .output()
                .unwrap(),
        );
        let compiled = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new("main.ssrg", "test/collection-partials", source)
                .with_profile(value),
        )
        .unwrap();
        let raw = out.join("raw.ts");
        fs::write(&raw, compiled.generated.typescript).unwrap();
        let mut files = vec![raw];
        if profile == "development" {
            files.push(out.join("entry.ts"));
        }
        let config = serde_json::json!({"compilerOptions": {
            "strict": true, "skipLibCheck": false, "noEmit": true,
            "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
            "allowImportingTsExtensions": true, "types": ["bun", "node"],
            "typeRoots": [root.join("node_modules/@types")],
            "lib": ["ES2023", "DOM", "DOM.Iterable"],
            "baseUrl": root, "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]}
        }, "files": files});
        let config_path = out.join("tsconfig.json");
        fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
        success(
            Command::new("node")
                .arg(root.join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(config_path)
                .output()
                .unwrap(),
        );
        for engine in ["bun", "node"] {
            let output = Command::new(engine)
                .arg(out.join("entry.js"))
                .output()
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(exit),
                "{profile}/{engine}: {output:?}"
            );
            assert_eq!(output.stdout, expected.as_bytes(), "{profile}/{engine}");
            assert_eq!(
                output.stderr,
                if exit == 0 {
                    b"".as_slice()
                } else {
                    b"seseragi: runtime defect\n".as_slice()
                },
                "{profile}/{engine}"
            );
        }
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&input)
            .args(["--profile", profile])
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(exit),
            "run/{profile}: {output:?}"
        );
        assert_eq!(output.stdout, expected.as_bytes(), "run/{profile}");
        if exit == 0 {
            assert!(output.stderr.is_empty(), "run/{profile}: {output:?}");
        } else {
            assert!(!output.stderr.is_empty(), "run/{profile}: {output:?}");
        }
    }
}

struct Case<'a> {
    function: &'a str,
    arguments: &'a [(&'a str, &'a str)],
    result: &'a str,
    stdout: &'a str,
    effect: bool,
}

#[test]
fn collection_helpers_preserve_every_partial_stage_in_typed_bindings_and_arguments() {
    let digits = ("Int -> Int -> Int", "digits");
    let positive = ("Int -> Bool", "positive");
    let array = ("Array<Int>", "[1,2,3]");
    let list = ("List<Int>", "`[1,2,3]");
    let initial = ("Int", "0");
    let cases = [
        Case {
            function: "reduce",
            arguments: &[initial, digits, array],
            result: "Int",
            stdout: "123\n",
            effect: false,
        },
        Case {
            function: "reduce",
            arguments: &[initial, digits, list],
            result: "Int",
            stdout: "123\n",
            effect: false,
        },
        Case {
            function: "reduce",
            arguments: &[initial, digits, ("Range<Int>", "1..=3")],
            result: "Int",
            stdout: "123\n",
            effect: false,
        },
        Case {
            function: "all",
            arguments: &[positive, array],
            result: "Bool",
            stdout: "True\n",
            effect: false,
        },
        Case {
            function: "any",
            arguments: &[("Int -> Bool", "\\n:Int -> n > 2"), list],
            result: "Bool",
            stdout: "True\n",
            effect: false,
        },
        Case {
            function: "sum",
            arguments: &[array],
            result: "Int",
            stdout: "6\n",
            effect: false,
        },
        Case {
            function: "product",
            arguments: &[list],
            result: "Int",
            stdout: "6\n",
            effect: false,
        },
        Case {
            function: "combine",
            arguments: &[("Array<String>", "[\"a\",\"b\",\"c\"]")],
            result: "String",
            stdout: "abc\n",
            effect: false,
        },
        Case {
            function: "join",
            arguments: &[
                ("String", "\"/\""),
                ("List<String>", "`[\"a\",\"b\",\"c\"]"),
            ],
            result: "String",
            stdout: "a/b/c\n",
            effect: false,
        },
        Case {
            function: "forEach",
            arguments: &[
                (
                    "Int -> Effect<{console:Console},ConsoleError,Unit>",
                    "\\n:Int -> println (show n)",
                ),
                array,
            ],
            result: "Effect<{console:Console},ConsoleError,Unit>",
            stdout: "1\n2\n3\n",
            effect: true,
        },
        Case {
            function: "map",
            arguments: &[("Int -> Int", "\\n:Int -> n + 1"), array],
            result: "Array<Int>",
            stdout: "[2, 3, 4]\n",
            effect: false,
        },
        Case {
            function: "map",
            arguments: &[("Int -> Int", "\\n:Int -> n + 1"), list],
            result: "List<Int>",
            stdout: "`[2, 3, 4]\n",
            effect: false,
        },
        Case {
            function: "arrays.filter",
            arguments: &[positive, array],
            result: "Array<Int>",
            stdout: "[1, 2, 3]\n",
            effect: false,
        },
        Case {
            function: "lists.filter",
            arguments: &[positive, list],
            result: "List<Int>",
            stdout: "`[1, 2, 3]\n",
            effect: false,
        },
        Case {
            function: "arrays.filterMap",
            arguments: &[
                (
                    "Int -> Maybe<String>",
                    "\\n:Int -> if n > 1 then Just (show n) else Nothing",
                ),
                array,
            ],
            result: "Array<String>",
            stdout: "[2, 3]\n",
            effect: false,
        },
        Case {
            function: "lists.filterMap",
            arguments: &[
                (
                    "Int -> Maybe<String>",
                    "\\n:Int -> if n > 1 then Just (show n) else Nothing",
                ),
                list,
            ],
            result: "List<String>",
            stdout: "`[2, 3]\n",
            effect: false,
        },
        Case {
            function: "arrays.flatMap",
            arguments: &[("Int -> Array<Int>", "\\n:Int -> [n,n]"), array],
            result: "Array<Int>",
            stdout: "[1, 1, 2, 2, 3, 3]\n",
            effect: false,
        },
        Case {
            function: "lists.flatMap",
            arguments: &[("Int -> List<Int>", "\\n:Int -> `[n,n]"), list],
            result: "List<Int>",
            stdout: "`[1, 1, 2, 2, 3, 3]\n",
            effect: false,
        },
        Case {
            function: "arrays.reduceRight",
            arguments: &[initial, digits, array],
            result: "Int",
            stdout: "60\n",
            effect: false,
        },
    ];
    let mut source = String::from(
        "import * as arrays from \"std/array\"\nimport * as lists from \"std/list\"\nfn digits acc:Int -> n:Int -> Int = acc * 10 + n\nfn positive n:Int -> Bool = n > 0\npub effect fn main = do {\n",
    );
    let mut expected = String::new();
    let mut receivers = String::new();
    for (index, case) in cases.iter().enumerate() {
        let call = |count: usize| {
            std::iter::once(case.function.to_owned())
                .chain(
                    case.arguments[..count]
                        .iter()
                        .map(|(_, expr)| format!("({expr})")),
                )
                .collect::<Vec<_>>()
                .join(" ")
        };
        let full = call(case.arguments.len());
        if case.effect {
            source.push_str(&format!("  {full}\n"));
        } else {
            source.push_str(&format!("  println (show ({full}))\n"));
        }
        expected.push_str(case.stdout);
        for stage in 0..case.arguments.len() {
            let ty = case.arguments[stage..]
                .iter()
                .rev()
                .fold(case.result.to_owned(), |result, (parameter, _)| {
                    format!("({parameter}) -> {result}")
                });
            let name = format!("saved{index}stage{stage}");
            let receiver = format!("receive{index}stage{stage}");
            receivers.push_str(&format!(
                "fn {receiver} callback:({ty}) -> ({ty}) = callback\n"
            ));
            let suffix = case.arguments[stage..]
                .iter()
                .map(|(_, expr)| format!(" ({expr})"))
                .collect::<String>();
            let declaration = format!("  let {name}: {ty} = {}\n", call(stage));
            if case.effect {
                source.push_str(&declaration);
                source.push_str(&format!("  {name}{suffix}\n"));
                source.push_str(&format!("  {receiver} ({}){suffix}\n", call(stage)));
                source.push_str(&format!("  {receiver} {name}{suffix}\n"));
            } else {
                source.push_str(&declaration);
                source.push_str(&format!("  println (show ({name}{suffix}))\n"));
                source.push_str(&format!(
                    "  println (show ({receiver} ({}){suffix}))\n",
                    call(stage)
                ));
                source.push_str(&format!("  println (show ({receiver} {name}{suffix}))\n"));
            }
            for _ in 0..3 {
                expected.push_str(case.stdout);
            }
        }
    }
    source.push_str("}\n");
    let source = source.replace(
        "pub effect fn main",
        &format!("{receivers}pub effect fn main"),
    );
    verify(&source, &expected);
}

#[test]
fn collection_partials_keep_callable_results_dictionaries_and_cold_effects() {
    verify(
        r#"import * as arrays from "std/array"
fn identity n:Int -> Int = n
fn step acc:(Int -> Int) -> n:Int -> (Int -> Int) = \value:Int -> acc value + n
fn total<C> values:C -> Int where Reducible<C,Int>,Zero<Int>,Add<Int,Int,Int> = {
  let saved:C -> Int = sum
  saved values
}
fn folded<C> values:C -> Int where Reducible<C,Int> = {
  let saved:C -> Int = reduce 0 (\acc:Int -> \n:Int -> acc * 10 + n)
  saved values
}
fn all value:Int -> Int = value + 1
pub effect fn main = do {
  let saved0:(Int -> Int) -> ((Int -> Int) -> Int -> (Int -> Int)) -> Array<Int> -> (Int -> Int) = reduce
  let saved1:((Int -> Int) -> Int -> (Int -> Int)) -> Array<Int> -> (Int -> Int) = reduce identity
  let saved2:Array<Int> -> (Int -> Int) = reduce identity step
  println (show (saved0 identity step [1,2,3] 10))
  println (show (saved1 step [1,2,3] 10))
  println (show (saved2 [1,2,3] 10))
  println (show (reduce identity step [1,2,3] 10))
  println (show (total [1,2,3]))
  println (show (folded `[1,2,3]))
  let local:Int -> Int = all
  println (show (local 41))
  let cold:Array<Int> -> Effect<{console:Console},ConsoleError,Unit> = forEach (\n:Int -> println (show n))
  println "created"
  cold [1,2]
  cold [3]
}
"#,
        "16\n16\n16\n16\n6\n123\n42\ncreated\n1\n2\n3\n",
    );
}

#[test]
fn collection_partial_evaluates_supplied_arguments_before_it_is_called() {
    verify_exit(
        r#"fn digits acc:Int -> n:Int -> Int = acc * 10 + n
pub effect fn main = do {
  println "before"
  let saved:Array<Int> -> Int = reduce (9007199254740991 + 1) digits
  println "unreachable"
}
"#,
        "before\n",
        70,
    );
}

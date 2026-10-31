use std::{fs, path::Path, process::Command};

#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

#[test]
fn comprehension_fixtures_match_ts_and_native() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixtures = root.join("examples/spec/fixtures/rust-backend");
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-comprehensions-")
        .tempdir()
        .unwrap();
    let cli = env!("CARGO_BIN_EXE_seseragi");
    for name in [
        "comprehensions",
        "comprehension-evaluation",
        "comprehension-large",
    ] {
        let source = fixtures.join(format!("{name}.ssrg"));
        let expected = fs::read(fixtures.join(format!("{name}.stdout"))).unwrap();
        let ts = successful(bounded(
            Command::new(cli).arg("run").arg(&source),
            scratch.path(),
        ));
        assert_eq!(ts.stdout, expected, "{name}: TS");
        let destination = scratch.path().join(name);
        successful(bounded(
            Command::new(cli)
                .args(["experimental", "build-native"])
                .arg(&source)
                .args(["--entry", "main", "--out-dir"])
                .arg(&destination),
            scratch.path(),
        ));
        let native = successful(bounded(
            Command::new(destination.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
                .env("PATH", "")
                .env("SESERAGI_NATIVE_METRICS", "1"),
            scratch.path(),
        ));
        assert_eq!(native.stdout, expected, "{name}: native");
        assert_eq!(native.stdout, ts.stdout, "{name}: parity");
        assert!(String::from_utf8_lossy(&native.stderr).contains("managed-live=0"));
    }
}

#[test]
fn ts_comprehension_evaluation_matches_the_native_trace_fixture() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/comprehension-evaluation.ssrg");
    let expected: Vec<String> = serde_json::from_str(include_str!(
        "../../../examples/spec/fixtures/rust-backend/comprehension-evaluation.trace.json"
    ))
    .unwrap();
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-comprehension-trace-")
        .tempdir()
        .unwrap();
    let bun = runtime("bun", scratch.path());
    for list in [false, true] {
        let source = if list {
            source
                .replace("pub fn probe -> Array<Int>", "pub fn probe -> List<Int>")
                .replace("[element x y |", "`[element x y |")
        } else {
            source.to_owned()
        };
        let input = scratch.path().join("main.ssrg");
        fs::write(&input, source).unwrap();
        let destination = scratch.path().join(if list { "list" } else { "array" });
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&input)
                .arg("--out-dir")
                .arg(&destination),
            scratch.path(),
        ));
        let module = destination.join("main.js");
        let mut generated = fs::read_to_string(&module).unwrap();
        for name in ["outer", "keep", "inner", "start", "accept", "element"] {
            let start = generated.find(&format!("var {name} = ")).unwrap();
            let body = start + generated[start..].find("=> ").unwrap() + 3;
            let end = body + generated[body..].find("\nvar ").unwrap() - 1;
            assert_eq!(&generated[end..end + 1], ";");
            generated.insert(end, ')');
            generated.insert_str(body, &format!("(__trace.push({name:?}), "));
        }
        generated.insert_str(0, "const __trace = [];\n");
        generated.push_str("\nprobe(undefined);\nconsole.log(JSON.stringify(__trace));\n");
        fs::write(&module, generated).unwrap();
        let output = successful(bounded(Command::new(&bun).arg(&module), scratch.path()));
        let trace: Vec<String> = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(trace, expected, "List result: {list}");
    }
}

#[path = "../../test-support/native_cache.rs"]
mod native_cache;

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "seseragi-program-wasm-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct OwnedChild {
    child: Child,
    stdout: PathBuf,
    stderr: PathBuf,
}
impl OwnedChild {
    fn start(command: &mut Command, directory: &Path) -> Self {
        native_cache::configure(command);
        let stdout = directory.join("stdout");
        let stderr = directory.join("stderr");
        command
            .stdout(Stdio::from(File::create(&stdout).unwrap()))
            .stderr(Stdio::from(File::create(&stderr).unwrap()));
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        Self {
            child: command.spawn().unwrap(),
            stdout,
            stderr,
        }
    }
    fn wait(&mut self, timeout: Duration) -> Output {
        let deadline = Instant::now() + timeout;
        let status = loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "deadline for owned PID {}\n{}",
                self.child.id(),
                fs::read_to_string(&self.stderr).unwrap()
            );
            std::thread::sleep(Duration::from_millis(10));
        };
        self.child.wait().unwrap();
        Output {
            status,
            stdout: fs::read(&self.stdout).unwrap(),
            stderr: fs::read(&self.stderr).unwrap(),
        }
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            #[cfg(unix)]
            unsafe {
                unsafe extern "C" {
                    fn kill(pid: i32, signal: i32) -> i32;
                }
                // Only this still-live child's dedicated process group.
                kill(-(self.child.id() as i32), 15);
                // Do not reap the group leader during the grace interval. Its
                // reserved PID prevents group-ID reuse before the final kill;
                // compiler descendants may outlive a terminated parent.
                std::thread::sleep(Duration::from_secs(2));
                kill(-(self.child.id() as i32), 9);
            }
            #[cfg(windows)]
            {
                if let Ok(mut terminator) = Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &self.child.id().to_string()])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                {
                    let until = Instant::now() + Duration::from_secs(5);
                    while Instant::now() < until {
                        if terminator.try_wait().ok().flatten().is_some() {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    let _ = terminator.kill();
                    let _ = terminator.wait();
                }
            }
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}
fn successful(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn bounded(command: &mut Command, directory: &Path) -> Output {
    OwnedChild::start(command, directory).wait(Duration::from_secs(120))
}
fn runtime(name: &str, directory: &Path) -> PathBuf {
    let output = successful(bounded(
        Command::new(if cfg!(windows) { "where" } else { "which" }).arg(name),
        directory,
    ));
    PathBuf::from(
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .next()
            .unwrap(),
    )
}

fn root() -> PathBuf {
    // Node24 cannot open script entries through Windows verbatim paths from
    // fs::canonicalize. Cargo's manifest path is absolute; use lexical parents.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root above the crate")
        .to_path_buf()
}

#[test]
fn typed_host_effect_callbacks_suspend_cleanup_and_release_their_last_host_references() {
    let scratch = Scratch::new();
    let output = scratch.0.join("host-effect");
    let source = root().join("examples/spec/fixtures/rust-backend-projects/program-wasm-host-effect-package/src/calculation.ssrg");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-wasm"])
            .arg(&source)
            .arg("--out-dir")
            .arg(&output)
            .args([
                "--export", "transfer", "--export", "prepared", "--export", "nested",
            ]),
        &scratch.0,
    ));
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("program-wasm.json")).unwrap()).unwrap();
    assert_eq!(manifest["abi"], "seseragi-program-wasm-host-effect-v3");
    for engine in ["bun", "node"] {
        let result = successful(bounded(
            Command::new(runtime(engine, &scratch.0))
                .arg(root().join("crates/seseragi-cli/tests/fixtures/program-wasm-host-effect.mjs"))
                .arg(&output)
                .env("PATH", ""),
            &scratch.0,
        ));
        assert_eq!(result.stdout, b"host Effect 23 cases passed\n");
    }
    fs::write(scratch.0.join("types.mts"), r#"
import {loadProgram, type HostEffect, type HostResult, type OwnedCallback, type OwnedEffect} from "./host-effect/program.mjs";
const request:HostEffect<string,string,string>=async (argument,context)=>{
  const source:string=argument;
  const signal:AbortSignal=context.signal;
  return signal.aborted ? {tag:"Failure",error:source} : {tag:"Success",value:source};
};
const release:HostEffect<string,never,undefined>=(_argument,_context)=>({tag:"Success",value:undefined});
const owner=await loadProgram(new Uint8Array());
const first:OwnedCallback<HostEffect<string,never,undefined>,OwnedCallback<string,OwnedEffect<string,string>>>=owner.exports.transfer(request);
const second:OwnedCallback<string,OwnedEffect<string,string>>=first(release);
const cold:OwnedEffect<string,string>=second("snow");
first.dispose();second.dispose();
const execution=cold.run();execution.cancel();
const outcome=await execution.result;
switch(outcome.tag){
  case "Success": { const value:string=outcome.value;break; }
  case "Failure": { const error:string=outcome.error;break; }
  case "Defect": { const messages:readonly string[]=outcome.messages;break; }
  case "Cancelled": break;
  default: { const exhaustive:never=outcome; }
}

const hosts:number=owner.ownership().hostCallbacks;
const waiting:boolean=owner.ownership().waitingHost;
const closing:Promise<void>=owner.dispose();await closing;
// @ts-expect-error a callback produces a typed outcome, not an untyped raw string
const raw:HostEffect<string,string,string>=(value)=>value;
// @ts-expect-error the request source argument stays String
owner.exports.transfer((input:number)=>({tag:"Success",value:String(input)}));
// @ts-expect-error release's failure channel is Never
const failure:HostEffect<string,never,undefined>=()=>({tag:"Failure",error:"failed"});
// @ts-expect-error the transport context is not a second Seseragi source argument
second("snow",{signal:new AbortController().signal});
// @ts-expect-error an explicit run is required before awaiting the typed outcome
const direct:Promise<HostResult<string,string>>=cold;
// @ts-expect-error every result must be narrowed before selecting its success value
const value:string=outcome.value;
"#).unwrap();
    fs::write(scratch.0.join("tsconfig.json"), r#"{"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"NodeNext","moduleResolution":"NodeNext","types":[],"lib":["ES2022","DOM"],"skipLibCheck":false},"files":["types.mts"]}"#).unwrap();
    successful(bounded(
        Command::new(runtime("node", &scratch.0))
            .arg(root().join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(scratch.0.join("tsconfig.json")),
        &scratch.0,
    ));
}

#[test]
fn standard_web_host_effect_build_keeps_typed_assets_and_source_stages() {
    use sha2::{Digest, Sha256};
    let scratch = Scratch::new();
    let package = root()
        .join("examples/spec/fixtures/rust-backend-projects/program-wasm-host-effect-package");
    for profile in ["development", "release"] {
        let site = scratch.0.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&package)
                .args(["--target", "web", "--profile", profile, "--out-dir"])
                .arg(&site),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(site.join("artifact-manifest.json")).unwrap())
                .unwrap();
        assert_eq!(manifest["profile"], profile);
        assert_eq!(manifest["programs"].as_array().unwrap().len(), 1);
        let program = &manifest["programs"][0];
        assert_eq!(program["abi"], "seseragi-program-wasm-host-effect-v3");
        assert_eq!(program["hostImport"], "@seseragi/programs/calculation");
        let directory = site.join("programs/calculation");
        let receipt: serde_json::Value =
            serde_json::from_slice(&fs::read(directory.join("program-wasm.json")).unwrap())
                .unwrap();
        assert_eq!(program["module"], receipt["module"]);
        for (name, arity) in [("transfer", 3), ("prepared", 2), ("nested", 3)] {
            let export = receipt["exports"]
                .as_array()
                .unwrap()
                .iter()
                .find(|export| export["name"] == name)
                .unwrap();
            assert_eq!(export["sourceArity"], arity);
        }
        for file in manifest["files"].as_array().unwrap() {
            let bytes = fs::read(site.join(file["path"].as_str().unwrap())).unwrap();
            assert_eq!(file["bytes"], bytes.len());
            assert_eq!(file["sha256"], format!("{:x}", Sha256::digest(&bytes)));
        }
        assert_eq!(
            fs::read(site.join("input.txt")).unwrap(),
            "初期値\n42\n".as_bytes()
        );
        for engine in ["bun", "node"] {
            let result =
                successful(bounded(
                    Command::new(runtime(engine, &scratch.0))
                        .arg(root().join(
                            "crates/seseragi-cli/tests/fixtures/program-wasm-host-effect.mjs",
                        ))
                        .arg(&directory)
                        .env("PATH", ""),
                    &scratch.0,
                ));
            assert_eq!(result.stdout, b"host Effect 23 cases passed\n");
            let lifetime =
                successful(bounded(
                    Command::new(runtime(engine, &scratch.0))
                        .arg(root().join(
                            "crates/seseragi-cli/tests/fixtures/program-wasm-host-lifetime.mjs",
                        ))
                        .arg(&site)
                        .env("PATH", ""),
                    &scratch.0,
                ));
            assert_eq!(
                lifetime.stdout,
                b"generated host page lifetime drained both owners and reported cleanup Defect\n"
            );
        }
        let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":[],"lib":["ES2022","DOM"],"skipLibCheck":false,"paths":{"@seseragi/programs/calculation":[directory.join("index.d.mts")]}},"files":[package.join("host/bridge.ts")]});
        let config_path = scratch.0.join(format!("{profile}-types.json"));
        fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
        successful(bounded(
            Command::new(runtime("node", &scratch.0))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(config_path),
            &scratch.0,
        ));
    }
}

#[test]
fn host_effect_channels_use_the_closed_value_codecs() {
    let scratch = Scratch::new();
    let source = scratch.0.join("channels.ssrg");
    fs::write(&source, r#"
import * as effects from "std/effect"
pub effect fn relay callback: ((Unit, Bool, Int, Maybe<String>, Either<String, Bool>) -> Effect<{}, (Unit, Bool, Int, Maybe<String>, Either<String, Bool>), (Unit, Bool, Int, Maybe<String>, Either<String, Bool>)>) -> input: (Unit, Bool, Int, Maybe<String>, Either<String, Bool>) -> (Unit, Bool, Int, Maybe<String>, Either<String, Bool>) fails (Unit, Bool, Int, Maybe<String>, Either<String, Bool>) = callback input
"#).unwrap();
    let output = scratch.0.join("channels");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-wasm"])
            .arg(&source)
            .arg("--out-dir")
            .arg(&output)
            .args(["--export", "relay"]),
        &scratch.0,
    ));
    let probe = scratch.0.join("channels.mjs");
    fs::write(&probe, r#"
import assert from "node:assert/strict";
import {readFile} from "node:fs/promises";
import {pathToFileURL} from "node:url";
const directory=process.argv[2];
const {loadProgram}=await import(pathToFileURL(`${directory}/program.mjs`));
const owner=await loadProgram(await readFile(`${directory}/program.wasm`));
try {
 for(const value of [
  [undefined,true,9007199254740991,{tag:"Just",value:"雪😀"},{tag:"Left",value:"typed"}],
  [undefined,false,-9007199254740991,{tag:"Nothing"},{tag:"Right",value:true}]
 ]) {
  for(const failing of [false,true]) {
   let calls=0;
   const partial=owner.exports.relay(async input=>{
    calls++;assert.deepEqual(input,value);
    return failing?{tag:"Failure",error:input}:{tag:"Success",value:input};
   });
   const plan=partial(value);partial.dispose();assert.equal(calls,0);
   assert.deepEqual(await plan.run().result,failing?{tag:"Failure",error:value}:{tag:"Success",value});
   assert.equal(calls,1);plan.dispose();assert.equal(owner.ownership().hostCallbacks,0);
  }
 }
} finally {await owner.dispose()}
assert.equal(owner.ownership().managedLiveAtClose,0);
assert.equal(owner.ownership().retired,false);
console.log("closed host channel codecs verified");
"#).unwrap();
    for engine in ["bun", "node"] {
        let result = successful(bounded(
            Command::new(runtime(engine, &scratch.0))
                .arg(&probe)
                .arg(&output)
                .env("PATH", ""),
            &scratch.0,
        ));
        assert_eq!(result.stdout, b"closed host channel codecs verified\n");
    }
}

#[test]
fn private_data_host_effect_factory_retains_state_and_drains_cleanup() {
    let scratch = Scratch::new();
    let output = scratch.0.join("host-ref");
    let source = root().join("examples/spec/fixtures/rust-backend/program-wasm-host-ref.ssrg");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-wasm"])
            .arg(&source)
            .arg("--out-dir")
            .arg(&output)
            .args(["--export", "create"]),
        &scratch.0,
    ));
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("program-wasm.json")).unwrap()).unwrap();
    assert_eq!(manifest["abi"], "seseragi-program-wasm-host-effect-v3");
    for engine in ["bun", "node"] {
        let result = successful(bounded(
            Command::new(runtime(engine, &scratch.0))
                .arg(root().join("scripts/probes/program-wasm-host-ref-probe.mjs"))
                .arg(&output)
                .env("PATH", ""),
            &scratch.0,
        ));
        assert_eq!(result.stdout,b"host private state: replay, detached leases, Failure/Defect, late-result cancellation, masked LIFO cleanup and roots released\n");
        assert!(result.stderr.is_empty(), "{result:?}");
    }
    fs::write(scratch.0.join("host-ref-types.mts"), r#"
import {loadProgram, type HostEffect, type OwnedCallback, type OwnedEffect} from "./host-ref/program.mjs";
const request:HostEffect<string,string,string>=async (input,context)=>{
  const signal:AbortSignal=context.signal;
  return signal.aborted ? {tag:"Failure",error:input} : {tag:"Success",value:input};
};
const owner=await loadProgram(new Uint8Array());
const creation:OwnedEffect<never,OwnedCallback<string,OwnedEffect<string,string>>>=owner.exports.create(request);
// @ts-expect-error a host request keeps its checked String input codec
owner.exports.create((_input:number,_context)=>({tag:"Success",value:"wrong"}));
// @ts-expect-error creation is a cold Effect, not an already-created callback
const eager:OwnedCallback<string,OwnedEffect<string,string>>=owner.exports.create(request);
const created=await creation.run().result;
if(created.tag==="Success") {
  const plan:OwnedEffect<string,string>=created.value("next");
  // @ts-expect-error the internal cell never becomes a host callback argument
  created.value({current:"next"});
}
await owner.dispose();
"#).unwrap();
    fs::write(scratch.0.join("host-ref-tsconfig.json"), r#"{"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"NodeNext","moduleResolution":"NodeNext","types":[],"lib":["ES2022","DOM"],"skipLibCheck":false},"files":["host-ref-types.mts"]}"#).unwrap();
    successful(bounded(
        Command::new(runtime("node", &scratch.0))
            .arg(root().join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(scratch.0.join("host-ref-tsconfig.json")),
        &scratch.0,
    ));
}

#[test]
fn private_data_host_effect_body_matches_raw_ts_and_native() {
    let scratch = Scratch::new();
    let original = fs::read_to_string(
        root().join("examples/spec/fixtures/rust-backend/program-wasm-host-ref.ssrg"),
    )
    .unwrap();
    let request = r#"
effect fn request input: String -> String fails String = match input {
  "fail" -> effects.fail "body failure"
  "defect" -> effects.die "body defect"
  _ -> effects.succeed input
}
effect fn unexpected message: String -> String = effects.die message
"#;
    let normal = r#"
effect fn oracle -> String fails String = do {
  let creation = create request
  first <- creation
  second <- creation
  one <- first "one"
  two <- first "two"
  other <- second "other"
  failed <- second "fail" |> effects.recover (\message: String -> effects.succeed message)
  after <- second ""
  effects.succeed (one + "|" + two + "|" + other + "|" + failed + "|" + after)
}
pub effect fn main = do { answer <- oracle () |> effects.recover unexpected; println answer }
"#;
    let defect = r#"
effect fn oracle -> String fails String = do { next <- create request; next "defect" }
pub effect fn main = do { answer <- oracle () |> effects.recover unexpected; println answer }
"#;
    for (label,main,code,expected) in [
        ("normal",normal,0,"seed:one|seed:one:inner::outer:two|seed:other|body failure|seed:other:inner::outer::inner::outer:\n"),
        ("defect",defect,70,""),
    ] {
        let text=format!("{original}\n{request}\n{main}");
        let source=scratch.0.join(format!("{label}.ssrg"));fs::write(&source,&text).unwrap();
        let native=scratch.0.join(format!("{label}-native"));
        successful(bounded(Command::new(env!("CARGO_BIN_EXE_seseragi")).args(["experimental","build-native"]).arg(&source).args(["--entry","main","--out-dir"]).arg(&native),&scratch.0));
        let result=bounded(Command::new(native.join(if cfg!(windows) {"program.exe"} else {"program"})).env("PATH","").env("SESERAGI_NATIVE_METRICS","1"),&scratch.0);
        assert_eq!(result.status.code(),Some(code),"{label}: {result:?}");assert_eq!(result.stdout,expected.as_bytes());
        let error=String::from_utf8_lossy(&result.stderr);assert!(error.contains("managed-live=0"),"{error}");
        if label=="defect" { assert!(error.contains("body defect"),"{error}"); }
        for (profile,value) in [("development",seseragi_project::BuildProfile::Development),("release",seseragi_project::BuildProfile::Release)] {
            let output=scratch.0.join(format!("{label}-{profile}"));
            successful(bounded(Command::new(env!("CARGO_BIN_EXE_seseragi")).arg("build").arg(&source).args(["--profile",profile,"--out-dir"]).arg(&output),&scratch.0));
            let raw=seseragi_driver::compile_module(seseragi_driver::CompileInput::new("host-ref.ssrg","proof/host-ref",&text).with_profile(value)).unwrap();
            fs::write(output.join("raw.ts"),raw.generated.typescript).unwrap();
            let mut files=vec![output.join("raw.ts")];if profile=="development" {files.push(output.join("entry.ts"));}
            let config=serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":["bun","node"],"typeRoots":[root().join("node_modules/@types")],"baseUrl":root(),"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},"lib":["ES2022","DOM"],"allowImportingTsExtensions":true,"skipLibCheck":false},"files":files});
            fs::write(output.join("tsconfig.json"),serde_json::to_vec(&config).unwrap()).unwrap();
            successful(bounded(Command::new(runtime("node",&scratch.0)).arg(root().join("node_modules/typescript/bin/tsc")).arg("--project").arg(output.join("tsconfig.json")),&scratch.0));
            for engine in ["bun","node"] {
                let result=bounded(Command::new(runtime(engine,&scratch.0)).arg(output.join("entry.js")).env("PATH",""),&scratch.0);
                assert_eq!(result.status.code(),Some(code),"{label}/{profile}: {result:?}");assert_eq!(result.stdout,expected.as_bytes());
                let error=String::from_utf8_lossy(&result.stderr);
                if code==0 {assert!(error.is_empty(),"{error}");} else {assert!(error.contains("Seseragi Defect:"),"{error}");}
            }
        }
    }
}

#[test]
fn private_data_host_effect_standard_package_keeps_the_original_typed_bridge() {
    let scratch = Scratch::new();
    let package =
        root().join("examples/spec/fixtures/rust-backend-projects/program-wasm-host-ref-package");
    assert_eq!(
        fs::read(package.join("src/calculation.ssrg")).unwrap(),
        fs::read(root().join("examples/spec/fixtures/rust-backend/program-wasm-host-ref.ssrg"))
            .unwrap()
    );
    for profile in ["development", "release"] {
        let site = scratch.0.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&package)
                .args(["--target", "web", "--profile", profile, "--out-dir"])
                .arg(&site),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(site.join("artifact-manifest.json")).unwrap())
                .unwrap();
        assert_eq!(manifest["profile"], profile);
        assert_eq!(manifest["programs"].as_array().unwrap().len(), 1);
        assert_eq!(
            manifest["programs"][0]["abi"],
            "seseragi-program-wasm-host-effect-v3"
        );
        assert_eq!(fs::read(site.join("input.txt")).unwrap(), "流れ".as_bytes());
        let directory = site.join("programs/calculation");
        for engine in ["bun", "node"] {
            let result = successful(bounded(
                Command::new(runtime(engine, &scratch.0))
                    .arg(root().join("scripts/probes/program-wasm-host-ref-probe.mjs"))
                    .arg(&directory)
                    .arg("package")
                    .env("PATH", ""),
                &scratch.0,
            ));
            assert_eq!(result.stdout,b"host private state: replay, detached leases, Failure/Defect, late-result cancellation, masked LIFO cleanup and roots released\n");
            assert!(result.stderr.is_empty(), "{result:?}");
        }
        let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":[],"lib":["ES2022","DOM"],"skipLibCheck":false,"paths":{"@seseragi/programs/calculation":[directory.join("index.d.mts")]}},"files":[package.join("host/bridge.ts")]});
        let path = scratch.0.join(format!("{profile}-types.json"));
        fs::write(&path, serde_json::to_vec(&config).unwrap()).unwrap();
        successful(bounded(
            Command::new(runtime("node", &scratch.0))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(&path),
            &scratch.0,
        ));
    }
}

#[test]
fn synchronous_and_multi_argument_host_functions_run_and_keep_ownership_rules() {
    let scratch = Scratch::new();
    let output = scratch.0.join("host-functions");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-wasm"])
            .arg(
                root().join("examples/spec/fixtures/rust-backend/program-wasm-host-functions.ssrg"),
            )
            .arg("--out-dir")
            .arg(&output)
            .args([
                "--export", "measured", "--export", "joined", "--export", "fetched", "--export",
                "labelled",
            ]),
        &scratch.0,
    ));
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("program-wasm.json")).unwrap()).unwrap();
    assert_eq!(manifest["abi"], "seseragi-program-wasm-host-effect-v3");
    let declared = fs::read_to_string(output.join("program.d.mts")).unwrap();
    for line in [
        "readonly \"measured\": OwnedCallback<(argument0: string) => number, OwnedCallback<string, number>>;",
        "readonly \"joined\": OwnedCallback<(argument0: string, argument1: string) => string, OwnedCallback<string, string>>;",
        "(argument0: string, argument1: string, context: HostEffectContext) => HostResult<string, string> | PromiseLike<HostResult<string, string>>",
    ] {
        assert!(declared.contains(line), "{declared}");
    }
    fs::write(
        scratch.0.join("run.mjs"),
        r#"import {readFile} from 'node:fs/promises';
import assert from 'node:assert/strict';
import {loadProgram, ProgramWasmClosed, ProgramWasmDefect} from './host-functions/program.mjs';
const bytes = await readFile(new URL('./host-functions/program.wasm', import.meta.url));
const owner = await loadProgram(bytes);
const {measured, joined, fetched, labelled} = owner.exports;
// A synchronous host function runs in place, with each source argument.
const measure = measured((text) => text.length * 10);
assert.equal(measure('雪🙂'), 30);
assert.equal(joined((left, right) => `${left}+${right}`)('p'), 'p+a+b');
// A host Effect function receives all its source arguments and the signal.
const calls = [];
const plan = fetched(async (key, step, {signal}) => {
  calls.push([key, step, signal instanceof AbortSignal]);
  return {tag: 'Success', value: key + step};
})('k');
assert.deepEqual(await plan.run().result, {tag: 'Success', value: 'k12!'});
assert.deepEqual(calls, [['k', '1', true], ['k1', '2', true]]);
plan.dispose();
const mixed = labelled((text) => `<${text}>`)(async (text) => ({tag: 'Success', value: text.toUpperCase()}))('x');
assert.deepEqual(await mixed.run().result, {tag: 'Success', value: '<<X>>'});
mixed.dispose();
// A callback keeps its host functions until it is disposed, and a disposed
// callback cannot be called.
const before = owner.ownership().hostCallbacks;
measure.dispose();
assert.equal(owner.ownership().hostCallbacks, before - 1);
assert.throws(() => measure('x'), ProgramWasmClosed);
// Disposing while a run waits for a host Effect cancels the run and waits
// for it before the instance closes.
const waiting = await loadProgram(bytes);
let started;
const pending = waiting.exports.fetched((key, step, {signal}) => new Promise((resolve, reject) => {
  started();
  signal.addEventListener('abort', () => reject(signal.reason));
}))('k');
const execution = pending.run();
await new Promise((resolve) => { started = resolve; });
await waiting.dispose();
assert.deepEqual(await execution.result, {tag: 'Cancelled'});
assert.equal(waiting.ownership().alive, false);
// The instance is busy during a synchronous host function, so a call back
// into it is a Defect that retires only that instance.
const reentrant = await loadProgram(bytes);
const inner = reentrant.exports.measured((text) => text.length);
assert.throws(
  () => reentrant.exports.measured(() => inner('again'))('x'),
  (error) => error instanceof ProgramWasmDefect && /Reentrant/.test(error.message),
);
assert.equal(reentrant.ownership().retired, true);
// A synchronous host function reached while an Effect runs cannot call
// back into the instance either.
const running = await loadProgram(bytes);
const saved = running.exports.measured((text) => text.length);
let labels = 0;
const guarded = running.exports.labelled((text) => (labels++ === 0 ? text : String(saved(text))))(async (text) => ({tag: 'Success', value: text}))('x');
await assert.rejects(
  guarded.run().result,
  (error) => error instanceof ProgramWasmDefect && /Reentrant/.test(error.message),
);
assert.equal(running.ownership().retired, true);
// A synchronous host function must answer synchronously; a refused Promise
// that later rejects is not left unhandled.
const promised = await loadProgram(bytes);
assert.throws(
  () => promised.exports.measured(async () => 1)('x'),
  (error) => error instanceof ProgramWasmDefect && /returned a Promise/.test(error.message),
);
const rejecting = await loadProgram(bytes);
assert.throws(
  () => rejecting.exports.measured(async () => { throw new Error('later'); })('x'),
  (error) => error instanceof ProgramWasmDefect && /returned a Promise/.test(error.message),
);
await new Promise((resolve) => setTimeout(resolve, 10));
// Other instances are not affected.
assert.equal(owner.exports.measured((text) => text.length)('abc'), 3);
await owner.dispose();
assert.equal(owner.ownership().managedLiveAtClose, 0);
console.log('host functions');
"#,
    )
    .unwrap();
    for engine in ["bun", "node"] {
        let result = successful(bounded(
            Command::new(runtime(engine, &scratch.0))
                .arg(scratch.0.join("run.mjs"))
                .env("PATH", ""),
            &scratch.0,
        ));
        assert_eq!(result.stdout, b"host functions\n", "{engine}");
    }
}

#[test]
fn console_provider_effects_run_through_the_run_environment_and_release_masked() {
    let scratch = Scratch::new();
    let output = scratch.0.join("console");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-wasm"])
            .arg(
                root()
                    .join("examples/spec/fixtures/rust-backend/program-wasm-console-provider.ssrg"),
            )
            .arg("--out-dir")
            .arg(&output)
            .args([
                "--export", "greet", "--export", "guarded", "--export", "quiet",
            ]),
        &scratch.0,
    ));
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("program-wasm.json")).unwrap()).unwrap();
    assert_eq!(manifest["abi"], "seseragi-program-wasm-host-effect-v3");
    let declared = fs::read_to_string(output.join("program.d.mts")).unwrap();
    assert!(
        declared.contains("readonly \"greet\": OwnedCallback<string, ProviderEffect<Readonly<{\"kind\": string; \"message\": string}>, undefined, { readonly console: ProgramConsole }>>;"),
        "{declared}"
    );
    let script = scratch.0.join("run.mjs");
    fs::copy(
        root().join("crates/seseragi-cli/tests/fixtures/program-wasm-console-provider.mjs"),
        &script,
    )
    .unwrap();
    for engine in ["bun", "node"] {
        let result = successful(bounded(
            Command::new(runtime(engine, &scratch.0))
                .arg(&script)
                .env("PATH", ""),
            &scratch.0,
        ));
        assert_eq!(result.stdout, b"console provider\n", "{engine}");
    }
    // The TypeScript runtime's own Console service is a valid provider.
    fs::write(
        scratch.0.join("types.mts"),
        r#"
import {loadProgram, type ProgramConsole, type ProviderEffect} from "./console/program.mjs";
import {liveConsole} from "@seseragi/runtime/console";
const provider: ProgramConsole = liveConsole;
const owner = await loadProgram(new Uint8Array());
const plan: ProviderEffect<Readonly<{kind: string; message: string}>, undefined, {readonly console: ProgramConsole}> = owner.exports.greet("x");
const execution = plan.run({console: provider});
// @ts-expect-error a provider Effect needs its environment
plan.run();
// @ts-expect-error a providerless Effect takes none
owner.exports.quiet("x").run({console: provider});
await execution.result;
"#,
    )
    .unwrap();
    let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":["bun","node"],"typeRoots":[root().join("node_modules/@types")],"baseUrl":root(),"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},"lib":["ES2022","DOM"],"allowImportingTsExtensions":true,"skipLibCheck":true},"files":["types.mts"]});
    fs::write(
        scratch.0.join("tsconfig.json"),
        serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    successful(bounded(
        Command::new(runtime("node", &scratch.0))
            .arg(root().join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(scratch.0.join("tsconfig.json")),
        &scratch.0,
    ));
}

#[test]
fn providers_without_a_bridge_or_an_owning_export_are_refused() {
    let scratch = Scratch::new();
    for (source, export, message) in [
        (
            "pub effect fn read _unit: Unit -> Maybe<String> with Stdin fails StdinError = readLine ()\n",
            "read",
            "an Effect that needs no provider other than Console",
        ),
        (
            "effect fn read _unit: Unit -> Maybe<String> with Stdin fails StdinError = readLine ()\npub effect fn say text: String -> Unit with Console fails ConsoleError = println text\n",
            "say",
            "native Stdin requires a target-specific ABI proof",
        ),
        (
            "import * as effects from \"std/effect\"\neffect fn say text: String -> Unit with Console fails ConsoleError = println text\npub effect fn quiet value: String -> String fails Never = effects.succeed value\n",
            "quiet",
            "a provider needs a selected export whose Effect requires it",
        ),
    ] {
        let path = scratch.0.join(format!("{export}.ssrg"));
        fs::write(&path, source).unwrap();
        let output = bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "build-wasm"])
                .arg(&path)
                .arg("--out-dir")
                .arg(scratch.0.join(export))
                .args(["--export", export]),
            &scratch.0,
        );
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{stderr}");
        assert!(!scratch.0.join(export).exists());
    }
}

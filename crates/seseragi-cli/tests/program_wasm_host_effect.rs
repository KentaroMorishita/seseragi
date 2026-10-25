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
    assert_eq!(manifest["abi"], "seseragi-program-wasm-host-effect-v1");
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
        assert_eq!(program["abi"], "seseragi-program-wasm-host-effect-v1");
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

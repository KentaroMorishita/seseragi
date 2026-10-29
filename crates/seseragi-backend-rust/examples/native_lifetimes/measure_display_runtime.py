"""Measure the original ordinary tuple Show kernel and separate allocator runs."""
import argparse
import json
import os
from pathlib import Path
import platform
import statistics

from measure_char_runtime import run, digest


RUST = r'''
fn batch(count: usize) -> u32 {
    let mut hash=2166136261_u32;
    for index in 0..count {
        let value=KERNEL(vec![]).apply(V::Int(std::hint::black_box(41+(index%10) as i64)));
        let V::Text(text)=std::hint::black_box(value) else {panic!("String")};
        for byte in text.get().bytes() {hash=(hash^u32::from(byte)).wrapping_mul(16777619);}
    }
    hash
}
fn main() {
    initialize_globals();
    let started=std::time::Instant::now(); let mut warmups=0;
    while warmups<3 || started.elapsed().as_secs_f64()<1.0 {
        std::hint::black_box(batch(1000)); warmups+=1;
    }
    let before=ALLOCATIONS.with(Cell::get); std::hint::black_box(batch(100));
    let nodes=(ALLOCATIONS.with(Cell::get)-before)/100;
    let mut samples=Vec::new();
    for _ in 0..5 {let now=std::time::Instant::now(); let checksum=batch(10000);
        let elapsed=now.elapsed().as_nanos(); assert_eq!(checksum,242191557);
        samples.push(elapsed);}
    clear_dictionaries(); clear_globals(); assert_eq!(LIVE.with(Cell::get),0);
    println!("{{\"samplesNs\":{:?},\"warmups\":{warmups},\"checksum\":242191557,\"managedAllocationsPerCall\":{nodes},\"managedLive\":0}}",samples);
}
'''
JS = '''import {kernel} from "./module.mjs";
function batch(count) { let hash=2166136261;
  for(let i=0;i<count;i++) {const value=kernel(41+i%10);
    for(let j=0;j<value.length;j++) hash=Math.imul(hash^value.charCodeAt(j),16777619)>>>0;}
  return hash; }
if(kernel(41)!=="(41, 42, True)") throw Error("value");
const start=process.hrtime.bigint();let warmups=0;
while(warmups<3 || process.hrtime.bigint()-start<1000000000n) {batch(1000);warmups++;}
const samplesNs=[];
for(let i=0;i<5;i++) {const now=process.hrtime.bigint(), checksum=batch(10000);
  const elapsed=Number(process.hrtime.bigint()-now);
  if(checksum!==242191557)throw Error(`checksum ${checksum}`);samplesNs.push(elapsed);}
console.log(JSON.stringify({samplesNs,warmups,checksum:242191557}));
'''
ALLOCATION = r'''
fn main() {
    let warm=std::env::args().nth(1).as_deref()==Some("--warm");
    initialize_globals();
    // Warm runtime/TLS storage, then release every dictionary before the cold
    // measurement. Count the next call together with its cache teardown.
    drop(KERNEL(vec![]).apply(V::Int(41)));
    if !warm {clear_dictionaries();}
    let cached_before=LIVE.with(Cell::get);
    let before=ALLOCATIONS.with(Cell::get);
    COUNT_SYSTEM.store(true,Relaxed);
    let result=KERNEL(vec![]).apply(V::Int(41));
    assert!(matches!(&result,V::Text(text) if text.get().as_str()=="(41, 42, True)"));
    drop(result);
    if !warm {clear_dictionaries();clear_globals();}
    COUNT_SYSTEM.store(false,Relaxed);
    let nodes=ALLOCATIONS.with(Cell::get)-before;
    clear_dictionaries();clear_globals();
    assert_eq!(LIVE.with(Cell::get),0);
    assert_eq!(SYSTEM_ALLOCS.load(Relaxed),SYSTEM_FREES.load(Relaxed));
    assert_eq!(SYSTEM_REQUESTED.load(Relaxed),SYSTEM_RELEASED.load(Relaxed));
    println!("{{\"allocations\":{},\"deallocations\":{},\"reallocations\":{},\"cumulativeRequestedBytes\":{},\"cumulativeReleasedBytes\":{},\"managedAllocations\":{},\"managedLive\":0,\"warm\":{},\"cachedManagedNodesBefore\":{},\"countsIncludeCacheTeardown\":{}}}",SYSTEM_ALLOCS.load(Relaxed),SYSTEM_FREES.load(Relaxed),SYSTEM_REALLOCS.load(Relaxed),SYSTEM_REQUESTED.load(Relaxed),SYSTEM_RELEASED.load(Relaxed),nodes,warm,cached_before,!warm);
}
'''


def main():
    parser=argparse.ArgumentParser()
    for name in ["cli", "typescript-emitter", "out"]:
        parser.add_argument("--"+name,required=True)
    parser.add_argument("--compare-with")
    args=parser.parse_args()
    root=Path(__file__).resolve().parents[4]
    source=root/"examples/spec/fixtures/rust-backend/tuple-show-kernel.ssrg"
    assert digest(source)=="77d0e76dfca0fc9f88fe45b6b9ed88d4869ad4fe6b03d1bf76af2c3c71a1f953"
    stage=Path(args.out).resolve();stage.mkdir(parents=True,exist_ok=False)
    cli,emitter=Path(args.cli).resolve(),Path(args.typescript_emitter).resolve()
    run([cli,"experimental","build-native",source,"--entry","probe","--out-dir",stage/"native"])
    env=dict(os.environ,PATH="")
    assert run([stage/"native/program"],env=env).strip()=="(41, 42, True)"
    run([emitter,source,stage/"ts"])
    config={"compilerOptions":{"strict":True,"noEmit":True,"skipLibCheck":False,
        "target":"ES2022","module":"ESNext","moduleResolution":"Bundler",
        "allowImportingTsExtensions":True,"types":["bun","node"],
        "typeRoots":[str(root/"node_modules/@types")],"baseUrl":str(root),
        "paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},"lib":["ES2023","DOM","DOM.Iterable"]},
        "files":[str(stage/"ts"/(p+".ts")) for p in ["development","release"]]}
    (stage/"tsconfig.json").write_text(json.dumps(config))
    run(["node",root/"node_modules/typescript/bin/tsc","-p",stage/"tsconfig.json"])
    run(["bun","build",stage/"ts/release.ts","--target=node","--format=esm","--outfile",stage/"module.mjs"])
    metadata=json.loads((stage/"native/rust-backend.json").read_text())
    symbol="v_"+(metadata["module"]+"::kernel").encode().hex()
    generated=(stage/"native/program.rs").read_text();offset=generated.rfind("\nfn main()")
    assert offset>1000
    allocator=Path(__file__).with_name("char_allocation.rs").read_text().split("fn main()",1)[0]
    for name,harness in [("timing",RUST),("allocation",allocator+ALLOCATION)]:
        (stage/(name+".rs")).write_text(generated[:offset]+harness.replace("KERNEL",symbol))
        run(["rustc","--edition=2021","-O",stage/(name+".rs"),"-o",stage/name])
    (stage/"measure.mjs").write_text(JS)
    allocation=json.loads(run([stage/"allocation"],env=env))
    warm_allocation=json.loads(run([stage/"allocation","--warm"],env=env))
    commands={"native":[stage/"timing"],"bun":["bun",stage/"measure.mjs"],"node":["node",stage/"measure.mjs"]}
    if args.compare_with:
        base=Path(args.compare_with).resolve();old=json.loads((base/"report.json").read_text())
        assert old["sourceSha256"]==digest(source)
        for name in ["timing","module.mjs","measure.mjs"]:assert old["controlsSha256"][name]==digest(base/name)
        commands={"baseline-native":[base/"timing"],"baseline-bun":["bun",base/"measure.mjs"],"baseline-node":["node",base/"measure.mjs"],**commands}
    names=list(commands);raw={name:[] for name in names}
    for iteration in range(len(names)*2):
        start=iteration%len(names)
        for name in names[start:]+names[:start]:
            data=json.loads(run(commands[name],env=env if "native" in name else None))
            assert data["checksum"]==242191557 and len(data["samplesNs"])==5
            if "native" in name:assert data["managedLive"]==0
            raw[name].append(data)
    report={"sourceSha256":digest(source),"generatedRustSha256":digest(stage/"native/program.rs"),
        "typescriptSha256":{p:digest(stage/"ts"/(p+".ts")) for p in ["development","release"]},
        "controlsSha256":{p:digest(stage/p) for p in ["timing","module.mjs","measure.mjs"]},
        "compilerIdentity":json.loads(run([cli,"--version-json"])),"host":platform.platform(),
        "runtimes":{p:run([p,"--version"]).strip() for p in ["rustc","bun","node"]},
        "protocol":{"callsPerSample":10000,"freshProcessesPerVariant":len(names)*2,
            "samplesPerProcess":5,"warmup":">=3x1000 calls and >=1 second","order":"balanced rotation",
            "bodyRewritten":False,"tsPhysicalHeapMeasured":False,"peakRssMeasured":False,
            "excluded":["compile","bundle","module load","startup","checksum assertion"],
            "allocatorUsedForTiming":False},"strictBothProfiles":True,"coldAllocationWithTeardown":allocation,
        "warmAllocationExcludingCacheTeardown":warm_allocation,
        "raw":raw,"medianUsPerCall":{name:statistics.median(n for row in rows for n in row["samplesNs"])/10000/1000 for name,rows in raw.items()}}
    (stage/"report.json").write_text(json.dumps(report,indent=2)+"\n")
    print(json.dumps({"medians":report["medianUsPerCall"],"allocation":allocation,"report":str(stage/"report.json")}))


if __name__=="__main__":main()

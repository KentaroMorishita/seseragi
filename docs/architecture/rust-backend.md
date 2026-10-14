# Rustバックエンドの設計と最初の実装

後続の[寿命管理とEffectの研究](../experiments/native-lifetimes.md)では、循環回収と同期Effect / Refの
opt-in生成経路を、同じSeseragiソースのTS実行と照合した。製品ABIへの統合はまだ行っていない。

## 判断と実装境界

Rustソース生成を第二のbackendとして段階的に実装する。Seseragiの型検査、instance選択、
評価順序の authority は既存frontendとCoreIrに置く。ユーザーにRustの所有権注釈を要求しない。
生成されたプログラムはRust標準ライブラリのみを使い、実行時にBun、Node、TS runtimeを必要としない。

最初のsliceはexperimentalな単一moduleの純粋計算である。既存`process` / `web` targetの意味は
変えず、CLIでは`experimental`配下から明示的に選ぶ。純粋entryの結果を表示するinspection
harnessを付ける。これはConsole / Show / Effectのnative実装を代替する契約ではない。

```text
Source → shared frontend → TypedHir → CoreCompilation / CoreIr
                                        ├→ TypeScriptIr → TS runtime
                                        └→ seseragi-backend-rust → Rust source + metadata
                                                                         → rustc → native
```

`seseragi-driver::compile_core_module`はTS lowering / emissionを呼ばない。
既存TSの`compile_module`とprojectのchecked-module loweringも同じCoreCompilation経路を使う。
`CompiledModule`の既存TS APIは互換性を維持する。backend間で一つの巨大な生成物structを共有せず、
Rust側は`RustArtifact`、`RustMetadata`、`RustBackendError`を返す。

## 値、closure、メモリ

ABI名は`rust-erased-pure-v1`。まず型消去した共通値`V`を使い、generic / HKTをRustのgenericsで
再現する必要をなくす。Unit / Int / Boolは即値、文字列・collection・ADT・関数・dictionaryは
immutableな共有ノードを使う。Intは既存言語のsafe integer範囲を維持し、算術で検査する。

closureは`captures: Vec<V>`と`fn(&[V], V) -> V`の組である。非捕捉のRust関数pointerを使うので、
値への参照がopaqueな`dyn Fn`の中に隠れない。CoreIrのfree-variable分析で必要なterm nameと
hidden dictionaryだけを捕捉し、不明な式はscope全体へ保守的に戻す。

共有cloneは内容を複製しない。最後の参照を破棄するときはthread内の作業キューへノードを入れ、
親の破棄で発生する子の破棄もキューへ積む。これにより深いADT・collection・captureの破棄が
Rustのcall stackを再帰的に消費しない。inspection表示も明示的なstackで走査する。

このsliceにRef、Signal、callback登録、local recursive closure、foreign objectはない。
構築済みimmutable値の捕捉とglobal関数へのcallだけでは、ユーザー値の強参照循環を構築できない。
参照カウント単独で全言語を実装できる、という判断ではない。

次のmutable / Effect sliceに入る前に、実際のowner→callback→owner graphをtraceするruntimeへ
進める。候補は明示的edgeを利用した循環検出付きRCとtracing GCである。一般の捕捉をWeakへ変えて
寿命を変える案は採らない。collection、closure captures、dictionaryの全edgeを走査できること、
外部rootの扱い、finalizer / cancellationとの順序を先に契約化する。性能だけでcollectorを選ばず、
強参照循環、深いgraph、返却closure、異常終了、長時間反復で回収を検証する。

## CoreIrで共有する分析

`seseragi-lowering`へtargetに依存しない三つの分析を置いた。CoreIr自体は書き換えない。

- `known_pure_call_shape`: 宣言のparameter数から部分適用・完全適用・返却関数への適用を区別する。
  return typeの矢印数をsource arityとして数えない。lexical shadowingとdeferred evidenceは
  backend側で除外する。
- `is_closed_local_evidence`: parameter evidenceを含まないlocal dictionary依存木を判定する。
  imported / standard / derived / supertrait / method constraintsはここで共有可能と断定しない。
  この判定だけで初期化移動を認めず、backend側でfactoryがmethod closureのみを構築することを確認する。
- `core_free_variables`: term nameとhidden evidence parameterを別に扱い、lambda、順次let、
  decision branchのscopeを解釈する。未対応shapeは`None`を返し、勝手にcaptureを落とさない。

Rustの既知関数の完全適用はdirect helperへ落とす。部分適用・alias・返却関数はcurried ABIを保つ。
通常のcallは各stageを適用してから次の引数を評価する。たとえば`returned a b`では、`returned a`の
bodyが実行されてから`b`を評価する。隠れたevidenceの挿入が必要なstagingは未対応として診断する。

dictionaryはthread内cacheで遅延共有する。static local factoryも入れ子のclosed factoryも共有し、
parameter-dependent factoryは共有しない。cache keyはinstance identity、type arguments、依存evidenceを
含めた選択済みCore evidenceに基づく。factory evaluation中はcacheをborrowしない。harness終了時に
明示的にcacheを空にする。TLS破棄時のcache teardownも検証する。

[TS実測](../experiments/core-ir-performance.md)で改善が見えた二つの処理をこの分析へ戻した。
TS backendはまだ新しい分析による最適化を使用していない。測定用TSコピーの改善率を、この実装の
RustやTSの製品性能として扱わない。TSへの適用はhidden evidence / import ABI / defect timingを
同じfixtureで検証する次の独立sliceとする。

## 対応範囲と診断

対応するのは、pure function、返却closure、部分適用、generic / HKTの型消去、local user instance
とnested dictionary、Int / Bool / String、Tuple / Array / field-only Record、local ADT、
if、constructor / Int / Bool decision、pure let sequence、限定したchecked Int演算と文字列加算。
全source moduleを検査し、未使用の未対応declarationも黙って消さない。

現在の未対応範囲：Effect、全stdlib、Float / Char / List、top-level let、local function、
record spread、複雑なpattern、standard dictionaryの値としてのmaterialization、derived instance、
supertrait / method constraints、deferred callable evidence、package / source module linking、
foreign / provider ABI、並列runtime、WASM target、source map、言語Defectのnative表現。
純粋再帰callはRust stackを使い、TCOの保証はない。深い値の破棄と深い関数callは別の境界である。

未対応CoreIrは`SES-RUST-0001`とsource byte rangeを持つtarget errorへする。frontendの型エラーは
既存診断を維持する。target検査に失敗した場合はRust sourceやnative outputを作らない。
生成コードのABI違反とInt defectは現在Rust panicとなる。Effect内で捕捉できる言語Defectや
非同期境界のfailureとしてはまだ実装していない。

## CLIと再現

Rust toolchainがあれば次を使える。`emit-rust`はrustcもBunも起動しない。
`build-native`はPATH上のrustcでhost native executableを作る。cross compilationはまだ公開しない。

```sh
cargo run --locked -p seseragi-cli -- experimental emit-rust \
  examples/spec/fixtures/rust-backend/ownership.ssrg \
  --entry probe --output /tmp/seseragi-ownership.rs

cargo run --locked -p seseragi-cli -- experimental build-native \
  examples/spec/fixtures/rust-backend/ownership.ssrg \
  --entry probe --out-dir target/rust-native-ownership

env PATH="" target/rust-native-ownership/program
# (42, 42, 11, 12)
```

`build-native`は`program.rs`、`rust-backend.json`、`program`（Windowsは`.exe`）を保存する。
metadataは生成sourceのbackend、experimental status、ABI、logical module、entry、edition、
生成code内のdirect call / closed dictionary site数を記録する。速度やallocation削減の証明ではない。
このmetadataは既存production artifact-manifest契約へまだ統合しない。

## 検証と次のslice

新backendのunit/runtime testsとnative execution fixturesをRust workspace gateへ組み込んだ。
CLI testは6 fixtureをTSとnativeで実行してstdout一致を確認する。native実行のPATHは空にする。
backend testは各fixtureを1万回反復し、cacheがwarmした後のlive node数が一定で、明示的な
cache teardown後に0となることを確認する。counterは管理ノードのみを数え、RSSではない。

```sh
cargo test --locked -p seseragi-backend-rust
cargo test --locked -p seseragi-cli --test rust_backend
bun run check:rust -- -p seseragi-driver -p seseragi-lowering -p seseragi-backend-rust
bun run check
```

shared driver / compiler基盤とCLIを変更するため、scoped testsの後にrepository-wide gateを実行する。
WASMの正式生成物はLinux x64で再生成して確認する。macOS生成物をcanonicalとして上書きしない。
実際のgate結果と未検証境界は作業の検証記録に残す。

次の順序は、(1)このpure targetのconformanceとartifact契約を広げる、(2)共通分析をTSへ適用して
再測定、(3)再帰call / closure / collectionのnative性能を計測、(4)trace可能なメモリ管理契約と
cold Effect runtimeを実装する、(5)provider / foreign ABI、package linking、WASMを追加する。
全言語対応やstable化はこの最初のsliceの完了条件ではない。

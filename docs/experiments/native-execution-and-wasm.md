# Native Effectの反復実行とprogram WASM

2026-10-15。`a5eb1894cfa4c292778b9b4889c610905c6038e3`を基点とする。
[前日の寿命管理研究](./native-lifetimes.md)から、三つの境界を進めた。

## 成果と製品境界

1. 研究用native Effectを、再帰呼び出しから命令と明示的な継続frameを処理するloopへ変更した。
2. TS runtimeのdeferがNodeでstack overflowになる不具合を修正した。これは製品runtimeの変更。
3. 同じfrontend / CoreIrから生成する純粋計算のprogram WASMを、既存TS targetで生成する
   SeseragiのWeb UIから呼び出した。UI sourceも計算sourceもSeseragiである。

nativeの命令VMとWASM projectionはopt-in研究であり、CLIのRust targetは引き続きpure ABI。
全stdlib、非同期provider、Fiber scheduler、Signalのnative実装、一般のWASM ABIは未実装。
TS runtimeの修正と、研究だけで成立した機能を混同しない。

## Nativeのstack safety

EffectをNative / Bind / Scoped / AcquireRelease / Recover / Release / Deferの命令で表す。
各命令は、GCから追える明示的なcapture列を持つ。実行loopは、次の計算、bindの継続、
failure handler、取得後のregistration、scope cleanupのframeをVecに保持する。
userのrelease callbackも解放時に呼び、そこから返るEffectを同じloopへ戻す。

Scope終了時には、元のoutcomeをframeへ保存してcancelをmaskする。finalizerをLIFOで実行し、
defectでも残りを試み、先頭defectとsuppressed順を保持する。finalizer実行中の追加acquisitionは
まだ閉じていない現在scopeへ登録する。取得成功からfinalizer登録までcheckpointを置かない。
継続やhandlerを呼ぶ前にもcancelを確認し、cancel後に正常継続を再開しない。

16 testsで、10万段のleft-associated bind、nested scope、deferを256 KiB stackで実行した。
実際の`deep-effect.ssrg`も既存frontend / CoreIrへ通してnative化し、同じstackで10万段を実行。
`deep: 42`は既存TS targetの出力と一致した。前日の7ケースと7 resource traceも維持する。

8 source比較は全走査・候補限定の両方で成功し、終了後のmanaged live / registryは0。
通常の小fixtureは毎checkpointでGCを強制する。deep sourceは128 checkpoint間隔、
巨大な静的planを保持するruntime単体testは終了後回収として、stack depthとGC停止時間を区別した。
GC間隔は研究設定で、collector debt / pause budgetを解決したという意味ではない。

これはEffectの命令chainの検証である。純粋再帰のTCO、一般のcallback内のRust再帰、
非同期wait、fairness、child supervision、parallel cleanupを証明するものではない。
命令とframeはpause / resume可能な実装へ進む土台だが、pending host resultのschedulerはまだない。

## TSで発見した不具合

旧deferは`thunk(unit)(environment, context)`を直接呼んでいた。
10万段のdeferをBun 1.3.11では実行できたが、同じruntimeをbundleしたNode 24.14.0では
`RangeError: Maximum call stack size exceeded`を再現した。

deferにもprivateなplanを持たせ、既存のbind interpreterがloopで展開するようにした。
構築時にthunkを呼ばず、実行ごとに再構築すること、failureとdefectのidentity、cancel後の
継続抑止とscope解放、bind / deferを交互に並べるケースを保持した。
100,000-step probeはBun、Node、Headless Chromium 155で成功した。
conformance runnerにもBun / Nodeのprobeを追加し、エンジン差をBunだけで見逃さないようにする。
時間の改善率や全Effectの性能保証は主張しない。

## UIとprogram WASMの混在

```text
wasm-compute.ssrg → shared frontend / CoreIr → pure Rust emitter
                                            → rustc wasm32-unknown-unknown → compute.wasm

web-ui.ssrg → existing web / TypeScript target → DOM app
                    ↓ checked foreign Int → Int boundary
                 scalar codec → compute.wasm
```

`wasm-compute.ssrg`の計算は`input * input + 7`。研究projectionは、checkedなUnit entryが
Int → Intの関数を返すことを確認し、scalar exportを追加する。その他のprojectionは生成前に拒否。
Rustのprimitive / managed表現をJSへ公開せず、i64 / JS BigIntを介してSeseragi Intのsafe rangeを
検証する。整数以外はWASMを呼ぶ前に拒否し、結果も同じ範囲へ検証する。

moduleのhost importは0。Rust 1.97.1の`-O -C strip=symbols`で27,200 bytes。
Nodeで1,000入力を実行し、各export内でpure ABIのmanaged liveが0へ戻ることをassertした。
同じSeseragi計算のTS実行も`11, 16`と一致した。このサイズは一つの小さな数値moduleの値で、
一般のprogram size、速度、起動時間の保証にはしない。

`web-ui.ssrg`はHtmlと`dom.app`で書き、既存CLIで`--target web`へbuildする。
foreign host glueは型を確認したscalar codecを呼ぶだけで、DOM実装をRustへ移していない。
ブラウザーで初期値2 → 結果11、+1 → 値3 / 結果16、reset → 値2 / 結果11を操作して確認した。
UIだけを手書きJSで置き換えた実証ではない。

RustのWASM panicはcaptureのDropを実行するとは限らない。overflowをtrapさせた後はcodecが
そのinstanceへのfunction参照を破棄し、以降の呼び出しを拒否する。defectをtyped failureへ
変換しない。NodeとChromiumでinstance retirementも確認した。
復旧には新しいinstanceが必要で、一般のsnapshot / retry / resource復旧ABIは未実装。

## 開発経路と残り

`verify_wasm.py`が、Rust生成、WASM compile、TS bridge bundle、UI sourceのweb build、
loader生成、Nodeによる型境界の検証をまとめる。生成済みUI artifactは手編集しない。
実証用loaderとHTMLは研究directoryへ別途生成する。

これは標準のpackage build graphやwatch / HMRにはまだ統合していない。今後必要なのは、
module選択とcapability診断、typed codecの一般化、source mapとdefect provenance、
依存更新時の再生成、旧instanceとcallback / subscriptionのdisposeをまとめること。
今回のABIは数値だけで、Ref、closure、Effect、DOM handleはJS / WASM境界を越えない。

## 再現と検証状態

```sh
cargo test --locked -p seseragi-backend-rust --example native_lifetimes
cargo build --locked -p seseragi-backend-rust --example native_lifetimes
python3 crates/seseragi-backend-rust/examples/native_lifetimes/verify.py \
  --emitter target/debug/examples/native_lifetimes --cli target/debug/seseragi --bun /path/to/bun-1.3.11
python3 crates/seseragi-backend-rust/examples/native_lifetimes/verify_wasm.py \
  --emitter target/debug/examples/native_lifetimes --cli target/debug/seseragi --bun /path/to/bun-1.3.11
```

WASM targetとNodeも必要。結果は`target/native-lifetimes/report.json`、
`target/native-wasm/report.json`、browser PNGへ保存する。CIにも同じsource比較とWASM実証を追加した。
TS runtime typecheck、defer/bindのBun tests、resource / concurrency / temporal probe、
Rust conformance crateのcheckはlocalで成功した。

製品runtimeを変更したため`bun run check`も実行したが、Rust workspaceのcompile中にディスクが
埋まり、ENOSPCで未成功。後続testにも領域不足が波及した。今回のgateが新しく生成した実行binaryと
実験のincremental cacheだけを識別して削除し、編集ファイルの完全性と対象checksを再確認した。
共有cache全体の削除、他worktreeの変更、canonical WASMの置換は行っていない。
全体gateを通過したとは扱わず、mainへのmerge / release / dogfoodは未実施。

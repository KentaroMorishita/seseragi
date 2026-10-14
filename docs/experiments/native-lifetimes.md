# Rust targetの寿命管理とEffectの研究

2026-10-14。実験の基点はRust backend foundationの`76519837aa3a9e780d9a525fc17a727ae670268d`。
製品のpure ABI、CLI、TS runtime、driver、CoreIr、Playground WASMは変更していない。
opt-in exampleへ二つの回収方式と同期Effectの小さな生成経路を実装した。

## 判断

Seseragiの値・返却関数・Ref・scopeを、sourceへ所有権注釈を加えずRustで実装する方向は成立する。
メモリ回収は参照カウントと循環検査、外部資源の解放はEffect scopeが担当する。
これは既存仕様の5.11と14.9に沿う分離であり、GC時のuser callbackやRust destructorを
Seseragiの観測可能な意味へ追加しない。

模型だけで判断せず、同じSeseragiソースを既存の型検査・CoreIrへ通し、実験用Rust生成と
既存TS生成の両方で実行した。native executableは標準ライブラリだけを使い、`PATH=""`で動く。
これは正式Rust targetのEffect対応ではない。CLIの`experimental`は引き続きpure ABIを使う。

## ソースからの照合

| 入力 | 両backendの観測結果 | native終了後 |
| --- | --- | --- |
| 既存Lesson 18 | `released: True` | live / registry = 0 |
| Ref → callback → Refの循環 | `answer: 42` | 循環検査が3ノードを回収、残存0 |
| cold Effectを構築して二度実行 | `before: 0`、`after: 2` | 残存0 |
| 入れ子scopeのfinalizer | `order: 3241` | 残存0 |
| scope外へ返すcallback | `escaped: 42` | 残存0 |
| typed failure後のscope解放とrecover | `failure released: 42` | 残存0 |
| release callbackのpure prefixがoverflow | bodyの出力後にdefect | 残存0 |

全走査と候補限定の両方で同じ結果になった。回収を各実験用Effect実行境界で強制し、実行中の
capture、Ref、次の継続、finalizerがrootとして保たれることも検証した。この強制頻度は製品の
GC schedulingではなく、GCによる誤回収を発見するstress設定である。

最後のケースはnative / TSともsuccessやtyped failureへ変換せず、defectとして終了する。
実験harnessのexit code 1と既存CLIのexit code 70は同じにせず、stdoutの順序とdefect分類を照合した。
nativeのdefectにsource mapやhost causeを付ける正式な診断ABIは未実装である。

研究中に二つの評価タイミングを修正した。release callbackは取得時ではなくfinalization時に
呼ぶ。CoreIrのsequenceはpure prefixと次のEffect値を先に構築し、そのEffectの実行後に
継続を構築する。sequence全体を無条件に遅延するとdefect timingが変わるため、既存TS loweringと
対応した段階で生成する。モデル側の都合でcoldnessや評価順序を読み替えない。

さらに、source frontendの型エラーと、型検査を通る未対応`yieldNow`を別々に拒否し、
どちらでもRust artifactを作らないことを確認した。未対応の関数や構造体も黙って消さない。

## メモリ模型

模型の値は`Rc<Node>`と明示的なcapture列を使う。各nodeのpayloadはcollectorが不要な辺を
切断できる私有`RefCell<Option<Value>>`に置く。Refだけがuser operationによる更新を持つ。
FunctionとEffectのcodeは非捕捉function pointerで、managed値はcapture列からすべて走査できる。
模型ではprimitiveもnodeにしており、製品pure ABIの即値Int / Boolとは異なる。

全走査では各nodeにsnapshot ownerを一つ持ち、実strong countからsnapshotと内部incoming edge数を
引く。差があるnodeを外部rootとして、その先を反復走査する。不要なpayloadを取り除く間は
snapshotが全候補を保持する。edgeの重複も数え、aliasや共有tailを誤って回収しない。

候補限定方式では、Handleのnon-zero decrementを弱参照の候補bufferへ記録する。その候補の
transitive outgoing graphだけを調べ、候補外からのincoming ownerは外部rootとして扱う。
候補がborrowされていれば、辺を切る前に中断し、次のsafe pointへ候補を戻す。
通常のRc Dropでregistry / candidateのentryも取り除く。weak entryが強参照0のallocationを
延々保持する形にはしない。

どちらもsingle-thread、non-moving、stop-the-world。走査にuser codeを使わず、managed payloadの
Dropにも言語finalizerを含めない。最後のownerの破棄には反復queueを使う。`unsafe`は使用しない。
一般のclosure captureをWeakへ変えることで寿命の意味を変更する設計ではない。

検証は14 tests。10万段の循環 / 非循環graphを256 KiB stackで回収し、200個のgraph条件
（100 seed × 2方式）を、Rc countを参照しない整数adjacencyの到達可能性oracleと比較した。
返却callback、alias、重複辺、共有tail、borrow中断、candidate再試行、foreign registrationの
root保持、nested scope、cancel masking、finalizer defect後の継続解放も含む。

## 回収停止時間の実測

macOS arm64、Rust 1.97.1、`rustc -O`。各条件3回の中央値。
割り当て・graph構築・最後のlive graphの破棄は計測区間に含めず、collectorの一回の停止だけを測った。
プロセスRSS、割り当て総時間、Web UIの操作時間、TS対Rustの性能比較ではない。

| 条件 | 全走査 | 候補限定 | 回収数 |
| --- | ---: | ---: | ---: |
| 1万個の2-node循環、ほぼ全てゴミ | 20,001ノード / 6.151 ms | 20,000ノード / 7.547 ms | 20,000 |
| 無関係な100,001-node live graph＋千個の循環 | 102,001ノード / 14.820 ms | 2,000ノード / 0.821 ms | 2,000 |

候補限定は無関係なlive graphの走査を避けられる一方、ゴミが大半の条件では候補探索のコストが
増えた。これは小さな合成graphの結果で、UIのframe budgetを満たす保証にはしない。
大きな候補graphなら停止も大きくなる。incremental化、collector debt / threshold、候補bufferの
上限、write barrier、allocator / registryコストを、製品採用前に調べる必要がある。

アルゴリズム自体の新規性は主張しない。内部参照を差し引いて外部rootを見つける考え方は
[CPythonのcollector実装](https://github.com/python/cpython/blob/main/Python/gc_free_threading.c)にもある。
本実験はそのparallel実装を移植したものではない。incremental tracingの候補である
[gc-arenaのmutation barrier](https://docs.rs/gc-arena/0.7.0/gc_arena/struct.Mutation.html)では、
mutationがmarking invariantを壊さない仕組みが必要になる。sourceへborrow/lifetimeの注釈を
要求せず、その制約を生成runtime内へ閉じ込められるかが比較基準になる。

## 資源とEffectの境界

研究用scopeの7 traceを、実際のTS runtime APIの7 traceと照合した。success、typed failure、
use defect、cancellation、finalizer defectのLIFOとsuppressed順序、explicit closeの冪等性、
cancel後のlate acquisition成功時のreleaseを含む。

同期のCoreIr生成経路はsucceed / fail / recover、Ref make / get / set、scoped / acquireRelease、
primitive Showとchecked Int、Consoleの成功する出力だけを扱う。cold planは再利用可能な値で、
scopeは実行ごとの状態である。finalizerをrunごとのframeへ登録し、実行中はcancelをmaskする。
defectでも残りのfinalizerを走らせ、先頭のfinalizer defectを優先する。

late acquisitionの比較は明示的なstate transition模型であり、nativeの非同期providerやFiberを
実装した証拠ではない。scheduler fairness、child supervision、pending host operationの観測、
cancel race、async finalizerの待機、環境service lookup、Ref update / modify、Signal transactionは
未検証。failure値もこのsliceではStringに限定し、全型のfailure ABIはまだない。
pure再帰と長いEffect bind chainのstack safety / TCOも、この研究で証明していない。

## Webと開発環境

最初の混在案は、既存web / TS targetがUIとDOMを担当し、明示した計算境界へWASMを配置する形。
server / CLIにはnative出力を使う。同じSeseragiの型検査・意味論を通し、境界にはtyped codecと
operation contractを置く。Rustのprivateな`Rc<Node>`、Ref、closure addressをJS ABIへ直接公開しない。
UIをTSで生成する場合も、SeseragiのUI sourceを書き直す要求にはしない。

WASMはJS importを通してDOMを扱えるので、UIは原理的に禁止される機能ではない。
[wasm-bindgenのDOM例](https://rustwasm.github.io/docs/wasm-bindgen/examples/dom.html)もある。
ただしSeseragiのnative / program-WASM backendにDOM bridgeを実装したわけではない。
既存Playgroundのcompiler WASMと、ユーザーprogramをWASMへ生成する機能も別である。

JS registryがWASM callbackを所有すると、それはnative heapから見て外部rootになる。
JSとWASMをまたぐ循環は各側のcollectorだけで解決したことにできない。本模型ではhost registryが
ある間はcallbackを保持し、scopeによるdetach後に回収することを確認した。実際のJS / WASM bridge
での回収、tokenのgeneration、handleの重複dispose、購読通知のlate completionは次の検証境界。

開発環境の受け入れ条件も、WASMファイル生成だけにしない。

- LSP / 型検査は同じfrontendを使い、出力先で未対応のcapabilityはsource位置つき診断にする。
- build graphがTS entry、WASM module、loader、typed ABIをまとめて追跡する。
- watchは依存の変更を検出し、再生成・再読み込み時に旧scope、subscription、callback rootを解放する。
- typed failure / defect / cancelをbridgeのPromise rejectionへまとめず、既存契約へ投影する。
- source map、設定されたRust / WASM toolchain、cold buildとincremental buildの時間を検証する。

この開発経路は設計条件であり、現時点の製品機能としては未実装。

## 再現と次の境界

```sh
cargo test --locked -p seseragi-backend-rust --example native_lifetimes
cargo build --locked -p seseragi-backend-rust --example native_lifetimes
cargo build --locked -p seseragi-cli
python3 crates/seseragi-backend-rust/examples/native_lifetimes/verify.py \
  --emitter target/debug/examples/native_lifetimes \
  --cli target/debug/seseragi --bun /absolute/path/to/bun-1.3.11
```

`target/native-lifetimes/report.json`にsource / CoreIr / generated Rust / 実験実装のhash、toolchain、
結果、nativeのmanaged node数、TS runtime trace比較、`rustc -O`での停止時間を保存する。
生成物と実行ログは同じdirectoryへ置く。CoreIr JSONは観測用artifactであり、`is_effect`が現在
serializationから省略されるため、JSONを再読込してnative backendへ渡す契約は検証していない。

研究example・専用fixture・CIの研究checkに限定した変更なので、scoped testsとsource / runtime
比較を実行した。製品compiler / runtime / WASM基盤を変更していないため、既存full gateとWASM
再生成はこのsliceでは実行しない。正式ABIへ統合する段階ではconformanceとfull gateを必要とする。

次の製品実装は、candidate方式の性能とメモリ上限の検証、Effect instruction / scopeの正式ABI、
stack-safe interpreter、非同期state machineのモデル検査、provider contractとの照合の順に進める。
Web混在はtyped value codecを最初の境界にし、その後にcallbackとcancel / reload disposalを検証する。
この研究だけで「GC問題・Effect・Web開発まで解決した」とは扱わない。

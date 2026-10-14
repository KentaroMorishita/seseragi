# CoreIrからRustを生成するPoC

2026-10-13に、`main`の`5210a2006266ce11ac60780af98d92d75ed45605`を基点として検証した。
実験コードはdriverのopt-in exampleに置き、製品のCLI、driver、TS runtime、WASM artifactは変更していない。

## 結果と判断

実際のSeseragiソースを既存frontendで型検査し、CoreIrからRustソースを生成して、
`rustc`でnative executableにする経路が成立した。6ケースの結果は既存TS backendと一致した。
各ケースを10万回実行し、entryから返された値を破棄した後の管理対象ノード数は毎回0だった。

所有権をユーザーへ要求せず、値の共有と返却クロージャを実装する方針には具体的な根拠ができた。
ただし、これは型を消去した共通値表現による純粋な言語sliceの検証である。
Rustの静的型を活用するbackend全体の設計、全stdlib移植、製品性能の証明ではない。

参照カウント単独では強参照の循環を回収できなかった。さらに、深いADT相当の値の再帰的な破棄で
stack overflowを再現した。現時点で「GCなしで全言語を支えられる」とは判断しない。

## 実行経路

```text
.ssrg
  → 既存parser / linker / semantics
  → TypedHir
  → 既存lower_typed_module
  → CoreIr
  → 実験用Rust emitter
  → Rustソース＋実験用runtime
  → rustc
  → native executable
```

TS lowering / emissionは呼ばない。コンパイル時にはRust toolchainを使う。
native実行は`PATH=""`で検証し、Bun、Node、rustcなどの外部コマンドを必要としなかった。
macOSの`otool -L`では、ownershipケースのbinaryはOSの`libSystem.B.dylib`だけを参照した。
Bun 1.3.11は比較対象の既存TS backendの実行にだけ使った。

## 値と関数の表現

- Unit / Int / Boolは即値。
- String / Array / Tuple / Record / ADT / dictionaryは、追跡可能な`Rc<Node<T>>`で共有。
- 関数は、共有された`dyn Fn(V) -> V`。捕捉値を所有し、呼び出し時に浅いcloneを使う。
- 複数引数関数はnested closureへ落とすので、部分適用と返却関数を同じ表現で扱う。
- generic / HKTの型検査とinstance選択は既存frontendが担当する。
  emitterはCoreIrの選択済みdictionaryとparameter evidenceを利用する。
  Rust側の型parameterは消去し、kindをRustのgenericsで表現する問題をこのPoCでは扱わない。
- local generic instanceはdictionary factoryとして生成し、入れ子のevidenceを渡す。
- ADTのconstructor tagにはcanonical symbolを使う。異なるnominal型を同一tagへまとめない。

`Shared::clone`は参照カウントを増やすだけで、中身のStringやcollectionを複製しない。
1 MBの文字列を1万回cloneするruntime testで、管理対象ノードの新規割り当てがなく、
同じallocationを共有することと、最後の参照を消すと回収されることを確認した。

管理対象ノードのlive / peak / allocation / cloneを数えている。
これはプロセス全体のRSSやすべてのallocator callの計測ではない。
StringやVecの内部buffer、検証用出力の割り当てはこの件数に含めない。

## ソースからnativeまでの検証

入力は`examples/spec/fixtures/rust-backend-poc/`に保存した。
比較用には同じソースへ`pub effect fn main = println (show (probe ()))`を付けて既存CLIで実行した。
native側の`probe`出力との一致を検証した。native側はEffectをlowerしていない。

| ケース | 結果 | 最大liveノード | 1 entryあたりの管理対象割り当て |
| --- | --- | ---: | ---: |
| generic ADTを捕捉する返却関数、trait evidence、部分適用 | `(42, 42, 11, 12)` | 7 | 10 |
| String入りgeneric ADTを共有し、関数を繰り返し呼ぶ | `(shared, shared, shared)` | 5 | 10 |
| 再帰ADTのtailを共有し、別々に走査する | `(3, 5)` | 11 | 18 |
| structural recordのwidth subtyping、capture | `(row, row)` | 5 | 8 |
| HKT引数のdictionaryを返却関数で利用する | `(42, 42)` | 6 | 8 |
| 入れ子のconstrained generic instanceのdictionary | `(42, 42)` | 11 | 14 |

各ケース10万回、すべてentry終了後live=0。これは小さな6ケースの結果であり、
長時間稼働するサーバーや全言語機能のメモリ安定性を証明するものではない。
実測native binaryは458,392〜479,560 bytesだった。
報告のelapsed値はsmoke実行時間で、TSに対する速度比較や性能保証には使わない。

## runtime模型による検証と限界

次はSeseragiソースのconformanceとは区別する。

1. cold計算を表す再利用可能なfactory：構築時には動かず、二度呼ぶと二度計算した。
   factoryの破棄でcaptureも回収された。Effect scheduler、failure、cancel、finalizerの検証はしていない。
2. owner → callback → ownerの強参照循環：外部参照を消しても2ノードが残った。
   明示的にcallback登録を消すと0になった。
3. callbackからownerへの戻り辺をWeakにした模型：owner破棄時に回収できた。
   一般の返却クロージャのcaptureをWeakに置換すると寿命の意味が変わり得る。
   Weakの利用は所有関係が明確な内部の戻り辺に限って検討する。
4. 2万段のADT相当値を256 KiB stackのthreadで破棄：構築には成功し、Drop時にstack overflowでabortした。
   この検証は独立child processで行い、再帰的破棄の限界を記録する。
   Seseragiソースからこの深さを構築・実行したという意味ではない。

循環回収、購読scopeによる切断、非再帰的な破棄、GCの採用を比較する必要がある。
このPoCはそれらを解決していない。

## 次に検証する設計

優先するのは単純なemitterの拡張より、値の管理方式の選定である。
実際のRef / Signal / callbackの所有graphで、scope終了後に何が残るかを調べる。
同時に、深いADTの破棄方法を決める。循環回収をGCへ任せる案も候補に残す。

関数の完全適用でもclosureとdictionaryを何度も構築しているため、このままでは割り当てが多い。
完全適用を直接callにし、静的dictionaryを再利用し、必要なcaptureだけを保持する方式を次に測る。
共有所有のclone削減と、Rustの静的型をどこまで使うかは、その後の設計課題となる。

Effect全体、async / 並行実行、resource、Signal、provider / foreign ABI、WASM、package linking、
local recursion、全stdlibは未検証。未対応のCoreIrはRustのコンパイル前にエラーへする。
PoCのエラーは文字列で、製品のstructured target diagnosticにはまだ統合していない。

## 再現方法

Rust toolchainと、比較用のBun 1.3.11を用意する。外部dependencyの追加は不要。

```sh
cargo test --locked -p seseragi-driver --example rust_backend_poc -- --nocapture
python3 crates/seseragi-driver/examples/rust_backend_poc/verify.py \
  --bun /absolute/path/to/bun-1.3.11
```

`target/rust-backend-poc/report.json`へfixture / emitterのhash、結果、管理対象ノードの件数、
toolchain情報、再帰的破棄の結果を保存する。ケース別CoreIr、生成Rust、native binary、TS比較ソース、
test logも同じdirectoryへ残す。Drop probeはstack overflowでchildが終了する可能性を含む。

単体の生成例：

```sh
cargo run --locked -p seseragi-driver --example rust_backend_poc -- \
  emit examples/spec/fixtures/rust-backend-poc/ownership.ssrg /tmp/ownership.rs 100000
rustc --edition=2021 -O /tmp/ownership.rs -o /tmp/ownership
env PATH="" /tmp/ownership
```

今回のhostはmacOS arm64、`rustc 1.96.1`。Bun比較は1.3.11。
exampleの3 test、生成runtimeの4 test、TS比較6ケースを検証した。
`bun run check:rust -- -p seseragi-driver`も成功した。

変更はopt-in example、専用fixture、実験記録に限定される。
製品compiler / runtime / WASM基盤やqueue統合を変更していないため、repository-wide full gateは実行しない。
正式backendとして統合する段階では、conformanceとfull gateを必要とする。

## 後続のTS実測

[TS実測とCoreIrの共通改善調査](./core-ir-performance.md)で、実際の生成TSの時間と保持量を測った。
閉じたdictionary共有と、既知関数の完全適用を直接callへlowerする処理を優先候補としている。
初期PoCの構築数をTS側の構築数へ読み替えず、それぞれを別に計測した。

# TS実測とCoreIrで共有できる性能改善の調査

2026-10-13、`8d42c0eac8ce6d7525c6934563c636551bf95a0f`を基点に調査した。
[Rust出力PoC](./rust-backend-poc.md)からの追試で、製品のcompiler / runtimeは変更していない。

## 結論

最優先の共通改善候補は、閉じたtrait evidenceのmaterialization共有と、既知関数の完全適用を直接callへ
lowerするためのcall planである。どちらも既存CoreIrの情報から判断でき、TS側でも改善余地を実測できた。

共通化するのは、calleeの宣言時arity、選択済みevidenceの依存関係、必要なcaptureを判断する解析である。
TSのmodule定数、Rustの共有所有、private direct entrypointといった具体的な表現は各backendが担当する。
すべてをCoreIrの書き換えだけで実現するという意味ではない。

capture livenessは共通解析の候補だが、使わないArrayを保持する問題は今回のTSケースでは再現しなかった。
具体instanceのdictionaryもTSではすでにmodule定数だった。Rust PoCの割り当て数をそのままTSの問題とは扱わない。

## 測定方法と適用範囲

- `examples/spec/fixtures/core-ir-performance/`の2ソースを既存driverでcompileし、CoreIr、TypeScriptIr、
  metadata、development / releaseの生成TSを保存した。この2ソースでは両profileのTSは同一だった。
- macOS arm64、Bun 1.3.11、通常の生成TSを直接importして実行した。
- 0〜4095の範囲で変化する8192個の入力を使い、結果をchecksumへ消費した。
  代表入力の結果検査、warmup、各sampleのchecksum検証を行った。
- 3つの新規Bunプロセスで、19ケースそれぞれ9 sampleを採取した。
  ケース別に約20 ms以上となる反復数へ調整し、測定順をrotateした。
  module loading、TS変換、source compile、結果検証はtimed regionに含めない。
- counterfactualは生成TSの別コピーへ適用した。通常の製品出力は変えていない。
- 構築回数は別コピーのTypeScript ASTへ計測コードを入れて数えた。
  arrow function / object literalの構築式が実行された回数であり、JIT後の物理的なallocation数ではない。
  その計測コードはtimingに使わない。runtime import内部の構築式はこの件数に含めない。
- memoryは非instrumented版で2万個の返却関数を保持し、GC後のJSC `heapSize` / `objectCount`等を比較した。
  全プロセスのRSSやallocation rateではない。
  [heapStatsの定義](https://bun.sh/reference/bun/jsc/HeapStats)に従い、snapshot前にfull GCを実行する。
- Rust側は現行の型を消去したPoCで8ケースを各1万回実行し、管理対象ノードの構築・共有回数を数えた。
  入力は固定で、TS / nativeの実行時間を比較する測定ではない。

この結果は小さなpure kernelと、このBun / hostに限る。アプリ全体、Node / V8、ブラウザ、Effectやproviderの性能へ
数値を外挿しない。handwritten controlはJIT状態によるプロセス間変動があり、固定の倍率を性能目標にはしない。
全sampleとプロセス別medianをJSONへ残した。

## 実際のTS出力で確認したこと

完全適用でも生成TSは`add(seed)(7)`だった。部分適用はそのままcurried functionを保持する。
pureなblockは`(() => { ... })()`として出ており、TS固有のIIFEも構築回数に現れた。

具体instanceはmoduleの定数objectになっていた。一方、入れ子のgeneric instanceでは、毎回次の形で
dictionaryをmaterializeしていた。

```ts
instanceReadBox(instanceReadBox(instanceReadInt)).read(Box(Box(seed)))
```

`seed`には依存しないdictionary treeだが、返却関数へ渡すとそのtreeとmethod closureも保持される。
ここは実行時間だけでなく、保持するheap graphも改善対象になった。

## counterfactualの結果

表は27 sampleを合わせたmedian。ns/opはこの測定条件の値である。

| ケース | 通常出力 | 実験出力 | 実験 |
| --- | ---: | ---: | --- |
| 完全適用 | 14.17 ns | 6.10 ns | private pure関数の完全適用だけを直接callへ変更 |
| 部分適用 | 24.42 ns | 24.71 ns | 上記実験のnegative control。部分適用は変更しない |
| 入れ子のgeneric read | 71.23 ns | 35.00 ns | 閉じたdictionaryの共有 |
| 同じ返却関数を二度呼ぶ | 137.16 ns | 95.17 ns | 閉じたdictionaryの共有 |
| 外へ返した関数を保持して呼ぶ | 110.54 ns | 62.40 ns | 閉じたdictionaryの共有 |

完全適用実験では、既存curried定義を残し、private `add`の完全適用siteを一つだけdirect helperへ向けた。
public / first-class / partial functionのABIを変更していない。生成された通常版と実験版のTypeScript typecheckも成功した。

dictionary共有は、CoreIrでlocal user-defined instanceを確認し、生成factoryのbodyがmethod closureとobjectを
構築するだけであることをASTで確認した。引数が既知の定数dictionaryまたは共有済みdictionaryであるcallだけを対象にした。
二つの閉じたfactory結果をmoduleへ置き、既存のmodule初期化文の順番とUnicode version検査を維持した。

引数のdictionaryに依存するfactoryは共有していない。`parameterized`と、別のnominal型`Other`のinstanceを
使うケースを実行し、後者では結果が`seed + 17`になることを確認した。
共有後もその経路の構築式は残り、結果と型検査は一致した。

| ケース | 通常のclosure構築式/op | 実験版 | 通常のobject構築式/op | 実験版 |
| --- | ---: | ---: | ---: | ---: |
| 完全適用 | 1 | 0 | 0 | 0 |
| 入れ子のgeneric read | 6 | 4 | 4 | 2 |
| 同じ返却関数を二度呼ぶ | 13 | 11 | 4 | 2 |
| 外へ返す関数 | 8 | 6 | 4 | 2 |

`match`等のTS IIFEもclosure件数に含む。これはGCが行ったallocationそのものの件数ではない。

## 返却関数の保持量

2万個の返却関数を保持したとき、GC後のJSC heapSize増分のプロセス間medianは次の通りだった。

| 経路 | heapSize増分 | objectCount増分 |
| --- | ---: | ---: |
| 入れ子dictionaryをcapture | 8,413,584 bytes | 約220,000 |
| dictionary共有後 | 4,046,176 bytes | 約100,000 |
| scalarだけをcaptureする比較用source | 1,856,714 bytes | 約40,000 |
| 使わないArray引数を受け取るsource | 1,813,968 bytes | 約40,000 |

最後のケースでは、512要素のArrayを2万回渡しても、保持後のArray数はbaselineから増えなかった。
CoreIrの自由変数も`seed`だけだった。使わないpayloadの保持をTS側の欠陥とは判断しない。

返却関数の格納先を空にして再度GCすると、objectCountの差はbaselineから-4〜39程度まで戻った。
これは測定process全体のsnapshotで、JITや検証処理のobjectも含む。差の完全な0を要求する測定ではない。
この試験からTS側のleakを結論しない。

## Rust PoCと共通改善の切り分け

| 経路 | Rust PoCの管理対象ノード構築/entry |
| --- | ---: |
| 完全適用 / 部分適用 | 4 / 4 |
| 即座に使うlambda | 3 |
| 使わないArray引数を受け取る関数 | 5 |
| 具体instanceのread | 4 |
| 入れ子generic read | 12 |
| 返却関数を二度使う | 14 |

entry wrapperも件数に含み、構築数と共有clone数は区別した。全ケースでentry終了後live=0だった。
Rust PoCの具体dictionaryは毎回作っており、TSで既に実現しているmodule共有へ追従するだけでも改善できる。
これを新しいCoreIr最適化が必要な問題とは扱わない。

Rust PoCは返却関数へのまとめた適用を表す一部のdeferred callable metadataを未対応として拒否する。
unused Arrayケースのnative probeは、関数を一度letへ受けてから呼ぶ二段階のソースで検証した。
TS版の対応状況や言語の意味の問題ではなく、PoCの未対応境界である。

## CoreIrで共有する解析の優先順位

1. **閉じたevidence treeのmaterialization plan。** instance identity、型引数、child evidenceをkeyにし、
   parameter依存と外部factoryを区別する。このfixtureのCoreIrには閉じたfactory参照6件とparameter依存1件があり、
   閉じたtreeの実現に二つの共有結果を使えた。型消去やbackend ABIを無視してkeyをまとめない。
   TSはmodule定数、Rustはbackendに合う共有所有で実現する。
2. **既知calleeのcall plan。** 宣言時のsource arityから完全適用、部分適用、返却関数への適用を分ける。
   function型のarrow数だけでは判定しない。`evidenceArgumentIndex`やdeferred evidenceを含むstaging、
   引数の左から右の一回評価、public callable ABIを維持する。最初の対象はprivate pure calleeに絞れる。
3. **capture / evidence liveness。** lexical bindingと隠れたdictionary captureを同じ解析で扱う。
   `captured`は`value`とdictionaryを、`captureOne`は`seed`だけを必要としていた。
   Rust emitterの余分なclone削減へ使えるが、TSのpayload leakは今回確認していない。

TSのpure block IIFE削減はbackend固有の候補。Rustの循環回収や深いDropもruntime固有の課題で、
今回の共通解析では解決しない。いずれも最適化時にpure calculationのdefectを前倒ししてよいという意味ではない。
たとえば返却関数内のchecked arithmeticを、関数の構築時へ移動しない。

## 再現と次の実装境界

```sh
python3 crates/seseragi-driver/examples/core_ir_performance/verify.py \
  --bun /absolute/path/to/bun-1.3.11
```

既存root / Playground dependencyとRust toolchainを使い、追加packageは導入しない。
`target/core-ir-performance/report.json`へ入力・測定コードのhash、全sampleファイル、CoreIr解析、
構築回数、native probeを保存する。ケース別の生成TS / counterfactual / CoreIrも同じdirectoryに残す。

計測script、生成TSとcounterfactualのtypecheck、Biome、Rust formatを検証し、driverのscoped Rust checkも成功した。
変更はopt-in exampleと専用fixtureに限定するため、repository-wide full gateはこの実験段階では実行しない。

次に製品へ入れる場合は、共通解析をRust crate側へ移し、まず閉じたlocal evidenceかprivate saturated callを対象にする。
評価順、defect timing、返却関数、generic / HKT、alias、import境界、Effectのcold性をconformanceで固定してから
両backendの実現へ接続する。compiler / runtime基盤への変更になる段階でfull gateとWASM再生成を実施する。
今回のcounterfactualは製品へ統合していない。

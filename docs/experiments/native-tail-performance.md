# 自己末尾再帰のTS / native計測

2026-10-15。進捗は[#816](https://github.com/KentaroMorishita/seseragi/issues/816)。
Rust targetの自己末尾再帰をloopへ落とした後の、小さなscalar kernelの比較である。
実装とstack / capture検証は[native runtime統合](./native-runtime-integration.md)に記載する。

## 方法と範囲

checked sourceの`sum remaining total`で、`remaining`を減らして加算する。
各sampleは200000〜200019の異なる入力20個を使う。期待checksumは400040001330。
各processで3回以上かつ1秒以上warmupしてから5 sample、各runtimeを4つの新規processで実行する。
実行順をrotateし、全sampleでchecksumを検証する。検証はtimed regionの外で行う。

nativeは通常のexperimental CLIから生成し、curried関数値を一度構築して呼ぶ。
TSは既存の`core_ir_performance` exampleから通常のrelease moduleを取得し、Bunの標準bundlerで
ES moduleへ変換する。compiler出力のbodyを編集せず、Bun / Nodeで同じmoduleを呼ぶ。
application packageのentryやmodule exportsを取り出すための書き換えは行わない。

source compile、rustc / bundling、module load、process startupはtimed regionに含めない。
native実行のPATHは空にし、終了後のmanaged liveは0を確認する。
比較用の`native-i64-control`だけは手書きのchecked i64 loopであり、compilerの製品出力ではない。
加減算のoverflowとsafe integer範囲を検査するが、一般のSeseragi意味論を実装したcontrolとは扱わない。

```sh
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
cargo build --locked -p seseragi-cli
cargo build --locked -p seseragi-driver --example core_ir_performance
python3 crates/seseragi-backend-rust/examples/native_lifetimes/measure_tail.py \
  --bun /absolute/path/to/bun-1.3.11
```

`target/native-lifetimes/tail-performance/report.json`にhost / tool version、入力と生成物のhash、
全sample、warmup回数、process別結果、native metadataを保存する。生成Rust / TS / bundleと
harnessも同じdirectoryへ残す。他のlocal build / conformanceが終了してから計測する。

## 型消去loopでの初回計測

macOS 26.2 arm64、Rust 1.97.1、Bun 1.3.11、Node 24.14.0。
表は20 sampleのmedian、1 callあたりの時間である。右列は各processのmedianの範囲で、信頼区間ではない。

| 経路 | median | process medianの範囲 |
| --- | ---: | ---: |
| 最適化前のRust生成コード | 7.70 ms | 7.08–8.00 ms |
| 通常TS出力 / Bun | 5.00 ms | 4.89–5.05 ms |
| 通常TS出力 / Node | 43.74 ms | 43.29–45.93 ms |
| 手書きchecked i64 control | 0.237 ms | 0.217–0.256 ms |

全runtimeでchecksumが一致し、nativeは全processで計算中のmanaged node構築が0だった。
この時点のkernelではBunがnativeより速かった。Rust生成だから常に高速とは結論しない。
手書きcontrolは製品の性能ではないが、scalar表現を直接扱う経路を検討する根拠になる。

timingとは別のnative実行をmacOS `sample`で1秒間観測した。top-of-stackの集計では汎用`binary`が
380、Vのdrop glueが211、VのCloneが152 sampleだった。単一の短いprofileから正確な内訳比率は
主張しないが、型消去した演算・clone / drop経路を調べる根拠が得られた。
raw profileは同じdirectoryの`native-sample.txt`へ保存した。

## 判断できること

このkernelにはRef、Effect実行、collection、foreign callがない。native counterが数えるのは
managed nodeの構築で、Rust全体のmallocやRSSではない。scalar loopでGCが必要ないことと、
型消去したVの処理が十分速いことは別に検証する。

この計測を根拠として、型消去ABIとsourceの意味を保った内部scalar loopの特殊化を検証した。
canonicalなprimitive identityと選択済み演算evidenceを条件にし、同名のuser typeを型名だけで
primitive扱いしない。overflowの発生順、partial / returned function、closure snapshotと
dictionary交換を維持する必要がある。

## 共有CoreIrのscalar loop認定後

[#822](https://github.com/KentaroMorishita/seseragi/issues/822)で、型と選択済みevidenceを
保持したCoreIrから認定する分析を追加した。builtin Int / Boolのparameterとresult、
literal / parameter参照、対応する標準Int演算、if、完全適用の直接自己末尾callに限定する。
generic、deferred evidence、nominalな同名型、capture / local function、sequence、
非末尾call、Effectなどは認定せず、既存の生成経路を使う。

TSは認定したbodyをstatement loopにし、反復ごとのtail markerと引数arrayを作らない。
nativeはprivate direct helper内部だけをi64 / boolで実行し、外側のVとcurried / partial ABIを
維持する。両経路とも次の全引数を左から右に評価した後でloop storageを更新する。
nativeのboxed / scalar経路は同じchecked Int primitiveを使う。

初回と同じsource hash、host、Rust / Bun / Node version、測定手順で再計測した。
生成bodyの手編集は行っていない。

| 経路 | 最適化前のmedian | 認定後のmedian | 認定後のprocess median範囲 |
| --- | ---: | ---: | ---: |
| Rust生成コード | 7.70 ms | 0.210 ms | 0.206–0.222 ms |
| 通常TS出力 / Bun | 5.00 ms | 1.78 ms | 1.63–2.41 ms |
| 通常TS出力 / Node | 43.74 ms | 6.10 ms | 6.04–6.39 ms |
| 手書きchecked i64 control | 0.237 ms | 0.209 ms | 0.205–0.210 ms |

controlの時間にも変動がある。これらは別の測定runであり、細かい倍率や一般のnative性能を
保証するものではない。入力hashは
`9bab89f473f633766197abf2a1e2f6bc31e9fc1d9a035b233815fbf434f9e4d3`。
新しい全sample、生成物hash、warmup、metadataは
`target/native-lifetimes/tail-performance-scalar/report.json`へ保存した。
nativeは全processでchecksum一致、終了後managed live 0、計算中managed node構築0だった。

境界検証は計測と別processで行う。canonicalな`scalar-tail-loop.ssrg`を使い、通常のV loopと
scalar helperで69件の結果 / overflowとEq / Sub / Addの呼出順を比較する。TSは
development / releaseの両方でstrict typecheckし、Bun / Nodeでも同じ69件を比較する。
safe integer上下限、再利用partial、Bool state、引数交換、乗算を含み、既存のclosure snapshotと
generic dictionary交換fixtureも実行する。ASTで認定されたTS loopにmarker / argument arrayが
ないこと、生成Rustのprivate helperにV clone / 汎用binary / Shared構築がないことを確認する。

```sh
cargo test --locked -p seseragi-lowering --lib
cargo test --locked -p seseragi-backend-rust
cargo test --locked -p seseragi-cli --test rust_backend
python3 crates/seseragi-backend-rust/examples/native_lifetimes/measure_tail.py \
  --bun /absolute/path/to/bun-1.3.11 \
  --out target/native-lifetimes/tail-performance-scalar
```

この一例の速度をアプリ、async、browser、全てのBun / Node / native programへ外挿しない。

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

## 計測結果

macOS 26.2 arm64、Rust 1.97.1、Bun 1.3.11、Node 24.14.0。
表は20 sampleのmedian、1 callあたりの時間である。右列は各processのmedianの範囲で、信頼区間ではない。

| 経路 | median | process medianの範囲 |
| --- | ---: | ---: |
| 現行Rust生成コード | 7.70 ms | 7.08–8.00 ms |
| 通常TS出力 / Bun | 5.00 ms | 4.89–5.05 ms |
| 通常TS出力 / Node | 43.74 ms | 43.29–45.93 ms |
| 手書きchecked i64 control | 0.237 ms | 0.217–0.256 ms |

全runtimeでchecksumが一致し、nativeは全processで計算中のmanaged node構築が0だった。
このkernelではBunが現行nativeより速い。Rust生成だから常に高速とは結論しない。
手書きcontrolは製品の性能ではないが、scalar表現を直接扱う経路を検討する根拠になる。

timingとは別のnative実行をmacOS `sample`で1秒間観測した。top-of-stackの集計では汎用`binary`が
380、Vのdrop glueが211、VのCloneが152 sampleだった。単一の短いprofileから正確な内訳比率は
主張しないが、型消去した演算・clone / drop経路を調べる根拠が得られた。
raw profileは同じdirectoryの`native-sample.txt`へ保存した。

## 判断できること

このkernelにはRef、Effect実行、collection、foreign callがない。native counterが数えるのは
managed nodeの構築で、Rust全体のmallocやRSSではない。scalar loopでGCが必要ないことと、
型消去したVの処理が十分速いことは別に検証する。

型消去ABIとsourceの意味を保ち、内部のscalar loopだけを特殊化できるかを次の候補とする。
canonicalなprimitive identityと選択済み演算evidenceを条件にし、同名のuser typeを型名だけで
primitive扱いしない。overflowの発生順、partial / returned function、closure snapshotと
dictionary交換を維持する必要がある。現時点でscalar特殊化を実装済みとはしない。

この一例の速度をアプリ、async、browser、全てのBun / Node / native programへ外挿しない。

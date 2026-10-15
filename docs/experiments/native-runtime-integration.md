# Native同期runtimeのCLI統合

2026-10-15。進捗の入口は[#809](https://github.com/KentaroMorishita/seseragi/issues/809)、
このsliceは[#810](https://github.com/KentaroMorishita/seseragi/issues/810)で追跡する。

## 実用経路

研究専用のemitterを選ばず、既存の`experimental emit-rust` / `experimental build-native`が
同じchecked frontend / CoreIrからpureと同期Effectを生成する。生成物はRust標準ライブラリだけを使う。

```sh
seseragi experimental build-native \
  examples/spec/fixtures/rust-backend-research/nested-scope.ssrg \
  --entry main --out-dir target/native-scope
env PATH="" target/native-scope/program
# order: 3241
```

pure entryのinspection契約とABI `rust-erased-pure-v1`を維持し、Effectを使う生成物を
`rust-erased-sync-v2`として識別する。metadataへentry kindと実際に必要なcapabilityを追加する。
単一moduleのexperimental経路であり、標準のproject build / watchへの統合はまだない。

## 値と循環回収

研究用Handleと製品用Vの二つをCLI内で使い分けず、既存VへReferenceとEffectを加える。
Unit / Int / Boolは即値のまま。immutable container、closure、dictionaryは共有したまま使う。
pure programではregistryを有効化しない。Refを使うmoduleだけ、user allocation前に有効にする。

weak registryのsnapshotに一時的な強参照を一つずつ持ち、全edgeの多重度を数える。
強参照数から内部edgeとsnapshotの一つを引いた残りが外部rootである。実行loopの継続、
typed failure、cleanup中のoutcome、scopeのfinalizerも通常のVを所有するのでrootとなる。
外部rootから到達できないRefのedgeを切り、既存の反復Drop queueで解放する。

全edgeはsafe Rustで走査し、user callbackをGC中に呼ばない。Refがborrow中なら、edgeを一つも
切る前にcollectionを延期する。返却callback、共有tail、重複edgeを保持する。
local recursive closureやopaque foreign captureはまだ生成できず、循環がRefを経由する前提を守る。
それらを追加する前にcollectorの回収契約も広げる必要がある。

full scanは512 checkpointごととroot解放後に行う。候補限定collectorやpause budgetは未統合。
この設定を長時間・大きなlive graphの性能保証として扱わない。

## Effectとresource

Native / Bind / Defer / Scoped / AcquireRelease / Recover / MapError / Releaseを明示的frameで
解釈する。bind / defer / scopeの深さがhost stackの深さにならない。
CoreIr sequenceではpure prefixと次のEffectをその場で構築し、残りはbind継続の中で構築する。
release callbackのpure prefixをresource取得時に実行しない。

Effectはcoldで再実行できる。failureは文字列へ変えず任意のVを保持し、recover / mapErrorへ
渡す。Refのupdate / modifyはcallbackが成功してから新しい値を保存する。
ConsoleのI/O失敗は標準のConsoleError shapeを持つtyped failureとする。
Console文字列出力、言語のShow、pure inspectionを混同しない。

scopeはLIFOでfinalizerを実行する。成功した取得とfinalizer登録の間にcancellation checkpointを
置かない。cleanup中はcancellationをmaskし、finalizerが失敗しても残りを試みる。
callback / native leafのpanicをDefectにしてscopeを閉じ、先頭とsuppressed順を保持して報告する。
collectorはlanguage finalizerを実行しない。

同期のcheckpoint / mask機構はruntime testで検証するが、OS signal、Fiber、async provider、
pending I/O、fairness、parallel cleanupとはまだ接続していない。
未処理typed failureの一般の選択済みShowとnative source frameも残る。

## 比較で見つかった本体の課題

- [#818](https://github.com/KentaroMorishita/seseragi/issues/818)：ConsoleのString契約と
  preludeのlegacy coercionが一致していない。nativeはprimitiveの既存出力をTSと比較し、
  compound値の直接出力は診断する。通常はshow / std/console.printValueで文字列化する。
- qualified Console APIのfailureがcanonicalな外部ConsoleErrorになると、TSのmain contractが
  標準Showを選べなかった。exact canonical identityの選択を追加し、同名の外部型を拒否する。
- [#820](https://github.com/KentaroMorishita/seseragi/issues/820)：末尾再帰fixtureの
  `let seed = seed + 1`がTSだけDefectになった。新しいconstのTDZがinitializerのouter参照を
  捕捉していた。共有CoreIr free-variable分析を使い、同名参照がある場合と解析できない場合は
  initializerをbindingの外で評価する。closure / polymorphic letを含め、pure / Effect / monad
  sequenceと両profileで比較し、生成TSのstrict typecheckも行う。
- [#821](https://github.com/KentaroMorishita/seseragi/issues/821)：CIの既存dev回復testで、
  compile失敗とgraph refresh成功の間にmanifestが修復される競合を検出した。
  ビルド前のsnapshotだけを確認済みにし、ビルド中の編集・修復を次のpollへ残す。既存dev 7 testsが成功した。

## 検証

runtime testsではcold replay、任意typed payload、Console I/O failure、cancellationと登録順、
finalizer defectとLIFO、borrow中の延期、200 graphの独立reachability oracle、
256 KiB stackで10万段のbind / scope / deferを検証する。

CLI testsは同じsourceをTS / nativeで実行し、stdout / exitと、Defect以外のstderrを比較する。
nativeのPATHは空にし、通常実行へmetricsを混ぜないこと、root終了後のmanaged liveが0であることも
確認する。sourceから生成した深いdeferとRef cycleも256 KiB stackで実行する。

```sh
cargo test --locked -p seseragi-backend-rust
cargo test --locked -p seseragi-cli --test rust_backend
cargo test --locked -p seseragi-runtime --lib contract::tests
```

full gateの実行理由はnative runtime / compiler backend基盤と共通main contractの変更である。
実行結果、CI SHA、未検証境界はIssue / PRへ記録する。正式WASM freshnessはLinux生成で確認し、
macOS生成物でcommitted版を置き換えない。

## 自己末尾再帰の追加

[#811](https://github.com/KentaroMorishita/seseragi/issues/811)の最初の範囲として、純粋関数の
直接の自己末尾callをloopへ生成する。if / decision / pure sequenceの末尾だけを辿り、
lambda内、alias、返却関数への追加適用、非末尾callは通常のclosure / callを維持する。

hidden dictionaryと全引数を一旦temporaryへ評価し、次のloop storageへまとめて代入する。
周回中のsource parameterはimmutable snapshotで、let shadowingとclosureの捕捉が次の周回の
storageと混ざらない。generic再帰で型引数の順序が変わればdictionaryの順序も更新する。

100万段のsum、部分適用を異なるseedで再利用する10万段のclosure捕捉、二つの異なるRead
instanceを入れ替える20万段の再帰を、checked sourceから生成して256 KiB stackで実行する。
終了後はdictionaryを含めたmanaged liveが0であることを確認する。これは自己末尾callの検証であり、
相互再帰やaliasを経由した再帰までconstant-stackと主張しない。

初回のcanonical Linux gateはa5524be44でcommitted WASMの更新を検出した。成功扱いせず、同じ
run 38722519017のLinux生成artifactからWASMを更新した。glueと宣言はbyte一致、WASMは
12,962,232 bytes / SHA-256 `45454a0459182bc59a8cf09e0529ef0bc8fcdf721dc7e2b82eeafa35d0e5f3ce`。
この更新を含む次のSHAで厳密なfreshnessとsource gateを再実行する。

let修正を含む13da264bbのLinux run 38724743261でも生成物を取得した。WASMは
12,974,111 bytes / SHA-256 `47fe0e2575b37b33c7cd103ee1533ba1269cd280e497a36f44045bfbcdd84205`。
同じSHAのnative jobは、runtime / backend / CLI、研究用16 tests、8 source / 7 trace比較、
program WASMと混在UIの検証をすべて通過した。追加のconformance差分は専用writerで再生成し、
このWASM更新とdev修正を含むSHAをrepository-wide source gateへ進める。

# Seseragi 実装計画

この文書は新仕様を実装へ渡すための非規範な計画です。言語の意味は`docs/spec/`、適合入力は
`examples/spec/`を正とします。現行TypeScript compilerの内部構造を互換契約にはしません。

## 1. 目標

- compiler、formatter、LSP、syntax highlight、playgroundが同じfrontendを使う。
- Seseragiの型、Effect、resource、module semanticsをTypeScript checkerへ委譲しない。
- TypeScriptは最初の正式backendであり、source languageの意味を決める層にはしない。
- 複数laneが同時に進んでも、共有AST fileやdiagnostic文字列の編集競合を常態化させない。
- 各段階をstable artifactとconformance fixtureで接続し、未完成な後段なしで前段を検証できる。

## 2. compiler pipeline

```text
SourceSnapshot
  -> TokenStream
  -> LosslessCst
  -> SurfaceAst
  -> ModuleInterface + ResolvedAst
  -> TypedHir
  -> CoreIr
  -> TypeScriptIr
  -> TypeScript + source map + module metadata
```

| artifact          | 所有する意味                                                          | 持たない意味                          |
| ----------------- | --------------------------------------------------------------------- | ------------------------------------- |
| `SourceSnapshot`  | canonical URI、UTF-8 bytes、version                                   | token、line/columnの派生cache         |
| `TokenStream`     | triviaを含むtoken kind、UTF-8 range、raw spelling                     | operator fixity、名前解決             |
| `LosslessCst`     | error nodeを含む全tokenのtree、stable child order                     | 型、desugar、symbol identity          |
| `SurfaceAst`      | Appendix grammarに沿う宣言・式・pattern、source range                 | import先symbol、inferred type         |
| `ModuleInterface` | public name、type scheme、fixity、instance head、deprecation metadata | function body、backend表現            |
| `ResolvedAst`     | canonical symbol ID、scope、import edge                               | 型推論結果、runtime layout            |
| `TypedHir`        | 型、kind、constraint evidence、coercion、exhaustiveness               | TypeScript syntax、optimization形状   |
| `CoreIr`          | 評価順を明示した小さい意味核、Effect operation、closure、ADT          | source sugar、host固有module spelling |
| `TypeScriptIr`    | JS/TS ABI、runtime call、async boundary、source-map origin            | Seseragiの型推論判断                  |

artifact間の変換は前段を破壊せず、新しい値を返します。nodeはrun内でstableなIDとUTF-8 source rangeを
持ちます。後段はsource textを再parseせず、必要なraw spellingは前段artifactから受け取ります。

## 3. 推奨repository境界

新compiler coreはRust workspace、生成先とhost runtimeはTypeScriptを推奨します。Rustを選ぶ理由はsource languageを
Rust風にするためではなく、parser・型検査・IRのdata ownership、native CLI/LSP、WASM playgroundで同じcoreを
共有しやすいためです。TypeScript backendは`.ts`を生成し、DOM、Node、browser service adapterはhostに近い
TypeScript runtimeとして残します。

```text
compiler/
  crates/
    source          SourceSnapshot、line index、span
    syntax          token、lossless CST、lexer
    parser          SurfaceAst、recovery、operator fixity
    diagnostics     code、range、fix、rendering-independent data
    project         manifest、module graph、interface cache
    semantics       names、types、kinds、traits、exhaustiveness
    hir             TypedHirとdesugar
    core-ir         評価順を固定したCoreIr
    backend-ts      TypeScriptIr、emitter、source map
    formatter       CST formatter
    driver          incremental queryとpublic compiler API
    cli             native command surface
    lsp             native language server adapter
    wasm            playground向けdriver adapter
runtime/
  typescript/       Effect、collection、interop、service、DOM runtime
```

実際のdirectory作成はWave 0開始時に行い、現行`src/`を途中で新coreへ混在させません。移行中は現行compilerと
新compilerを別command / package identityで実行し、同じ`.ssrg`に対する結果をconformance runnerが明示的に
選びます。「新実装の一部だけ現行parserへfallbackする」互換経路は作りません。

`source`から`core-ir`まではfilesystem、process、Node、browser APIへ依存できないpure coreにします。`cli`、
`lsp`、`wasm`は同じ`driver`を呼び、playground専用parserを持ちません。`runtime/typescript`はRust crateへ逆依存せず、
versioned ABI fixtureだけを共有します。これによりcompiler core、runtime、product toolingを別worktreeで進められます。

このlayoutは実装方式の推奨であり言語仕様ではありません。Rust採用を変更しても、2章のartifact境界とfixtureは
維持します。

## 4. 最初に固定するcontract

実装開始前に、次のschemaだけを小さく固定します。内部fieldの追加は許しますが、意味変更はconsumerと
fixtureを同じcommitで更新します。

1. token kind、trivia、UTF-8 range、lex errorのsnapshot schema
2. lossless CSTのnode kind、missing token、error node、再構築規則
3. diagnostic code、primary / related range、fix editのschema
4. canonical module path、symbol ID、operator fixityを含むmodule interface schema
5. type / kind / constraintのdebug表示とalpha normalization
6. Core IRの評価順、failure、cancellation、source origin invariant
7. TypeScript runtime ABI versionとgenerated module metadata

serializationはtest/debug artifactであり、language ABIではありません。schema majorが同じconsumerだけが読み、
compiler内部のclass名やmemory addressをsnapshotへ出しません。

## 5. 並列開発lane

### Wave 0: contractとharness

| lane | 主な所有範囲                            | 最初の成果物                                      |
| ---- | --------------------------------------- | ------------------------------------------------- |
| A    | source/token/CST schema                 | lexer snapshot、lossless再構築test                |
| B    | conformance runner、fixture discovery   | parse/diagnostic/run/shape targetの共通runner     |
| C    | runtime ABI、Effect/ADT/collection核    | versioned TypeScript runtime interfaceとunit test |
| D    | diagnostic/LSP wire、source map fixture | UTF position変換、diagnostic JSON、mapping test   |

Wave 0ではASTや型推論を並列実装しません。後続全laneが依存するartifactを小さく合意し、fixture runnerが
stub producerも検査できる状態を作ります。

### Wave 1: frontendとmodule graph

| lane | 主な所有範囲                           | 依存                    |
| ---- | -------------------------------------- | ----------------------- |
| A    | raw scanner、lexer、lossless CST       | token/CST contract      |
| B    | parser、operator header/fixity resolve | AのTokenStream/CST      |
| C    | package loader、module graph/interface | canonical path contract |
| D    | CST formatter、syntax/LSP token bridge | AのLosslessCst          |

parserはfile I/Oやpackage discoveryを行わず、module resolverはexpressionを再parseしません。formatterとLSPは
parserのprivate helperをcopyせず、公開frontend APIだけを使います。

### Wave 2: semantics

| lane | 主な所有範囲                               | 依存                         |
| ---- | ------------------------------------------ | ---------------------------- |
| A    | scope、名前解決、visibility、import        | SurfaceAst + ModuleInterface |
| B    | type/kind inference、generalization        | ResolvedAst                  |
| C    | trait coherence、instance selection、laws  | type engine interface        |
| D    | pattern exhaustiveness、Effect requirement | type engine interface        |

型表現とconstraint solverのownerは一laneに置きます。別laneが型variantを直接追加せず、query interfaceとfixtureを
先に追加します。trait、pattern、Effectはsolver pluginのような非公開hookではなく、versioned typed queryを使います。

### Wave 3: lowering、backend、product surface

| lane | 主な所有範囲                          | 依存                       |
| ---- | ------------------------------------- | -------------------------- |
| A    | Surface sugar -> TypedHir/CoreIr      | TypedHir contract          |
| B    | TypeScriptIrとcode generation         | CoreIr + runtime ABI       |
| C    | std runtime、Effect/Stream/Signal/DOM | runtime ABI                |
| D    | LSP、formatter、playground、CLI       | shared frontend + compiler |

CoreIrを飛ばしてSurfaceAstからTypeScriptを出すshortcutは作りません。product surfaceはcompilerをsubprocessで
模倣せず、同じlibrary entrypointとdiagnostic schemaを利用します。

## 6. ownershipとmerge規則

- 一つのartifact schemaには一人または一laneのownerを置く。全員共有ownerにはしない。
- laneごとにworktree / branchを分け、生成物やformat-only変更をfeature commitへ混ぜない。
- contract変更は`contract -> producer -> consumer`の順で小さくmergeする。巨大な全lane同時mergeを作らない。
- consumerが必要なfieldはproducer内部へ直接patchせず、先にqueryまたはartifact fieldのcontract testを追加する。
- shared enumへfeature固有variantを増やす前に、既存のextension pointで表せない理由を記録する。
- fixtureの期待値更新だけで意味変更を通さない。対応する規範仕様またはbug根拠を同じchangeへ含める。
- merge queueはWaveの依存順にrebaseし、greenな`check`と対象conformance suiteを一commitごとに保つ。

## 7. horizontal laneからvertical sliceへ移る条件

TokenStream、LosslessCst、SurfaceAst、ResolvedAst、TypedHir、CoreIrの最小contractが一度end-to-endで通った後は、
工程別laneを固定し続けません。以後は次のようなfeature sliceを一人が縦に通します。

```text
match guard
  grammar fixture
  -> CST/AST
  -> name/type/exhaustiveness
  -> CoreIr
  -> TypeScript output
  -> formatter/LSP snapshot
```

各sliceはartifact ownerのreviewを受けますが、五人へ順番にhandoffしません。工程別laneは共通基盤、性能改善、
難しいsolver変更に戻し、通常機能はvertical sliceでintegration待ちを減らします。

## 8. 最小end-to-end milestone

最初のcompiler milestoneはFizzBuzz全体ではなく、次を順に通します。

1. literal、`let`、parameter付きpure `fn`、application、`+`
2. Bool、`if`、tuple、`match`とexhaustiveness
3. Array、range、effectful `for`
4. `pub effect fn main`、Console requirement、typed failure
5. module importとTypeScript出力/source map

各段階でparse、type、CoreIr、generated TypeScript、runtime stdoutを別snapshotにします。後段失敗を前段成功と
混ぜず、playgroundへ接続するのは同じlibrary APIでlesson 01から05が通ってからです。

## 9. 実装開始gate

- Appendix productionが`grammar-coverage.json`の一groupへ一度だけ対応している。
- token/CST/diagnostic/module interfaceのschema draftがfixtureで読める。
- 最小end-to-end milestoneのinputと期待artifactが決まっている。
- runtime ABI version、generated module metadata、source-map originのownerが決まっている。
- laneごとの変更可能directoryとreview ownerが最初のissue / taskへ記載されている。

このgateは全仕様の実装完了を待つものではありません。共有境界が未定義なまま人数だけ増やして、後から一つの
巨大ASTとbackendへmergeする事態を防ぐための開始条件です。

最初の二項は`examples/spec/grammar-coverage.json`と`examples/spec/artifacts/schema-1/`で最小contractを
満たしました。basicに加えてmissing / error CSTとzero-width diagnosticを持つrecovery sourceを固定しています。
`examples/spec/artifacts/interface-schema-1/rich/`はdependency edge、public type / operator、instance headを
bodyから分離して固定し、module graph laneの入力も閉じました。

milestone 1の`pub let answer: Int = 42`はSurfaceAst、ResolvedAst、TypedHir、CoreIr、TypeScriptIr、generated
TypeScriptまで接続済みです。`runtime-schema-1/core/abi.json`は`core.int64`をTypeScript `bigint`表現として登録し、
generated moduleがABI majorとfeature requirementを宣言します。source map v3はportableな
`seseragi://artifact/basic` URI、sourcesContent、name mappingを持ち、origin mappingの開始gateも満たしました。

最初のEffect縦sliceは`stage-schema-1/effect-main/`で固定しました。parameterなし`effect fn main`は
implicit `Unit` parameterを持ち、closedな`Console` requirement、`ConsoleError` failure、`Unit` successを
ResolvedAst、TypedHir、CoreIrへ保持します。TypeScriptIrは`effect.console.println` featureだけを参照し、
backendはruntime ABI registryのstructured importからmoduleとexportを解決します。したがってhelper pathは
parser、型検査、CoreIrへ漏れません。このfixtureも手書きの期待contractであり、compiler実装完了を意味しません。

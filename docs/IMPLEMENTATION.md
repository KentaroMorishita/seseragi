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
crates/
  seseragi-source          SourceSnapshot、line index、span
  seseragi-syntax          token、lossless CST、lexer
  seseragi-parser          SurfaceAst、recovery、operator fixity
  seseragi-diagnostics     code、range、fix、rendering-independent data
  seseragi-project         manifest、module graph、interface cache
  seseragi-semantics       names、types、kinds、traits、exhaustiveness
  seseragi-hir             TypedHirとdesugar
  seseragi-core-ir         評価順を固定したCoreIr
  seseragi-backend-ts      TypeScriptIr、emitter、source map
  seseragi-formatter       CST formatter
  seseragi-driver          incremental queryとpublic compiler API
  seseragi-cli             native command surface
  seseragi-lsp             native language server adapter
  seseragi-wasm            playground向けdriver adapter
  seseragi-conformance     artifact / fixture runner
runtime/
  typescript/              Effect、collection、interop、service、DOM runtime
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

#### Wave 0の最初のissue分割

Wave 0は「compilerを作る」ではなく、「compilerを複数laneで作れる境界を固定する」ための
短い作業列です。最初は次のissueへ分けます。

| issue | lane | 主な変更directory                                                    | 完了条件                                                     |
| ----- | ---- | -------------------------------------------------------------------- | ------------------------------------------------------------ |
| 0-A1  | A    | `crates/seseragi-syntax/`、`examples/spec/artifacts/schema-1/`       | sourceをbyte単位で復元できるTokenStream skeleton             |
| 0-A2  | A/D  | `crates/seseragi-syntax/`、`crates/seseragi-diagnostics/`            | missing token / error node / UTF range変換test               |
| 0-B1  | B    | `crates/seseragi-conformance/`、`examples/spec/artifacts/`           | artifact schemaをdiscoverし、stub resultを比較するrunner     |
| 0-B2  | B    | `crates/seseragi-conformance/`、`examples/spec/fixtures/`            | lesson / fixture / project discoveryを同じcase modelへ正規化 |
| 0-C1  | C    | `runtime/typescript/`、`examples/spec/artifacts/runtime-schema-1/`   | runtime ABI registryを読み、feature requirementを検査        |
| 0-C2  | C/B  | `runtime/typescript/`、`examples/spec/artifacts/execution-schema-1/` | Effect runner contractをstdout / stderr / traceで比較        |
| 0-D1  | D    | `crates/seseragi-diagnostics/`、`crates/seseragi-source-map/`        | diagnostic JSONとsource map v3をpath非依存で比較             |

同じissueでproducerとconsumerを両方完成させようとしません。まず手書きfixtureを読むconsumerを作り、
次にproducerを小さく置き換えます。たとえば0-B1は現compilerへ接続せず、`examples/spec/artifacts/`の
JSONだけを読めれば完了です。

Wave 0中のdirectory所有は衝突回避のため次を初期値にします。

- syntax lane: `crates/seseragi-syntax/` とtoken / CST schemaだけを編集する。
- conformance lane: `crates/seseragi-conformance/` とfixture discoveryだけを編集する。
- runtime lane: `runtime/typescript/` とruntime ABI / execution schemaだけを編集する。
- diagnostic lane: `crates/seseragi-diagnostics/`、`crates/seseragi-source-map/`、LSP wire schemaだけを編集する。

shared schemaを変える場合は、owner laneのcontract commitを先に作り、他laneはそのcommitへ追随します。
これにより、AST型やdiagnostic文字列を全員が直接編集する状態を避けます。

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

### 6.1 worktreeとsubagent運用

複数agentで並列化する場合、同じworking treeを共有しません。main checkoutは統合・review・最終
commit用に残し、実作業はissue単位のworktreeで行います。

初期命名規則は次です。

| 対象            | 形式                             | 例                                 |
| --------------- | -------------------------------- | ---------------------------------- |
| branch          | `codex/wave{n}-{issue}-{slug}`   | `codex/wave0-0b1-conformance`      |
| worktree path   | `_worktrees/wave{n}-{issue}-{slug}` | `_worktrees/wave0-0b1-conformance` |
| subagent task名 | `wave{n}_{issue}_{slug}`         | `wave0_0b1_conformance`            |
| commit subject  | `<area>: <imperative summary>`   | `tooling: discover artifact cases` |

worktreeを切る単位は「lane」ではなく「merge可能なissue」です。0-B1と0-B2のように同じlaneでも、
同時に進めるなら別worktreeにします。逆にshared schemaを触るcontract changeは一つのworktreeだけで
行い、他worktreeはそのcommitを取り込んでからproducer / consumer作業を続けます。

Codex desktopではsandboxや権限profileがturnごとに変わることがあるため、標準のworktree置き場はrepo内の
ignored directoryである`_worktrees/`にします。repo外worktreeを使う場合は、権限profileが書き込み可能で
あることを先に確認します。

subagentへ渡すtaskには必ず次を含めます。

- 対象issue IDと目的。
- 触ってよいdirectory。
- 触ってはいけないdirectory、特に他lane ownerのschema。
- 期待するartifact / command。
- 実行してよいtargeted check。
- legacy全体testを回さないなどの制約。

subagent taskは「調査」ではなく、原則として小さい実装ticketにします。大きなlane名だけを渡さず、
10〜15分相当で差分が出る粒度へ切ります。各ticketは次のどちらかで終わらせます。

- targeted check済みのcommit。
- blocked理由、触ったfile、次に必要な判断を3行程度で返す。

成果なしでrate limitを消費しないため、subagent自身がさらにsubagentをspawnすることは通常禁止します。
再分割が必要な場合はmain orchestratorへ返し、mainがworktree、owner、merge順を切り直します。mainは
5〜10分おきにworktreeの`git status --short`を確認し、差分が出ないticketは止めるか、より小さい目的へ
再発行します。

subagentの成果をmainへ戻す条件は次です。

1. worktree内でtargeted checkが通っている。
2. `git diff --check`が通っている。
3. 変更がissueの許可directoryに収まっている。
4. schema変更がある場合、規範仕様またはartifact READMEも同時に更新されている。
5. mainへ取り込む前に、統合側で対象runnerを再実行する。

conflictが出た場合、先にcontract ownerの変更を優先し、producer / consumer側をrebaseします。生成物の
競合は手作業で混ぜず、producerを再実行できる段階なら再生成、まだ手書きfixtureならschema ownerが
期待値を一つに決めます。

### 6.2 merge順の原則

parallel issueは次の順で統合します。

1. schema / fixture contract
2. discovery / runner consumer
3. producer implementation
4. product surface integration
5. cleanup / format-only

たとえばsyntax laneでTokenStream fieldを増やしたい場合、まずartifact schemaと手書きfixtureをmergeし、
次にconformance runnerが新fieldを読めるようにし、最後にlexer producerを対応させます。これを逆順にすると、
producer内部の都合が仕様contractへ漏れます。

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

Effect実行境界は`examples/spec/artifacts/execution-schema-1/effect-main/`で固定しました。generated moduleは
Effect valueを返すだけで、entry runnerが`main ()`を一度呼び、root resource scopeとConsole serviceを提供し、
stdout / stderr / trace / exit分類を比較します。これによりbackend、runtime、host runnerを別laneで進められます。

## 10. フェーズ1: 型付きじゃんけんCLI

フェーズ1の最終成果物は、Seseragiで書いた型付きじゃんけんCLIを`.ssrg`からcompileし、生成TypeScriptと
versioned runtimeを通して実行できる状態です。正常入力と不正入力の両方をexecution fixtureで固定します。

この目標はサンプル専用のcompiler分岐を許可するものではありません。`Hand`、`Outcome`、`decide`などの名前を
compilerやruntimeが特別扱いせず、一般的なADT、constructor、tuple、pattern、match、Effectの組み合わせとして
成立しなければなりません。また、CLI完成を急いで未型付けの値やuncheckedなTypeScriptへ逃がさず、次のsliceを
一つずつ全stageへ通します。

| 順序 | slice | 完了条件 |
| ---- | ----- | -------- |
| P1-0 | canonical expression input | pure bodyの型付けとdiagnosticが`SurfaceExpr` / `ResolvedExpr`を読み、source tokenを再parseしない |
| P1-1 | tuple | tuple valueとtuple patternがSurfaceAst、TypedHir、CoreIr、TypeScriptIr、generated TypeScriptへ残る |
| P1-2 | ADT | user定義sum type、nullary / payload constructor、constructor valueの型検査とTS表現を固定する |
| P1-3 | match | constructor / tuple / wildcard pattern、binding scope、branch type、unreachable / non-exhaustive diagnosticを固定する |
| P1-4 | pure domain | `Hand`と`Outcome`を使う`decide` / `renderOutcome`が通常pipelineだけでcompileできる |
| P1-5 | typed failure | `succeed`、`fail`、`mapError`、do内pure binding、Either / Maybe相当の値をtyped Effectへ接続する |
| P1-6 | CLI execution | Console / Stdin environmentとrunnerを接続し、正常入力・不正入力のstdout / trace / exitを比較する |

2026-07-11時点ではP1-2からP1-4の最小縦sliceを通しています。非再帰ADT、nullary / payload constructor、
opaque export境界、constructor値と`let` annotationの型検査に加え、constructor / tuple / wildcard pattern、
arm-local binding、Bool guard、branch結果型、non-exhaustive / unreachable diagnosticを通常pipelineで処理します。
名前と型の所有関係はresolverの`SymbolId`から引き継ぎ、generic ADTの直接payloadはscrutineeの型引数で具体化します。

`examples/spec/artifacts/schema-1/rock-paper-scissors-domain/`は`Hand` / `Outcome`、`decide`、
`renderOutcome`をCSTからgenerated TypeScriptまで固定しています。CoreIrはscrutineeを一度だけ保持するtotal decisionを持ち、
TypeScriptIrで初めてtag比較へlowerします。生成TypeScriptは`tsc --noEmit`を通し、Bunから三つの勝敗結果を呼び出す
pure execution probeも確認済みです。compiler / runtimeにじゃんけん固有の名前分岐はありません。

現時点のexhaustiveness proofはclosedなlocal finite ADTとそのtuple直積が対象です。recursive ADT、importされたADT、
大きなpattern matrixは誤ったproofを出さず保守的に未証明へ倒します。これらと、container / record / function内部まで含む
generic payload substitution、nullary constructorの期待型からのcontextual specializationは独立sliceで完成させます。

P1-0は新機能の前提です。SurfaceAstが式を所有しても、TypedHirが再びsource tokenを走査するならmatchやpatternを
追加するたびに別parserが増えます。したがってpure expression consumerをSurfaceAstへ移してからtupleへ進み、
Effect側の移行はpure domain sliceを壊さない独立commitにします。

P1-1からP1-4では少なくとも次をartifactで別々に観測します。

- positive sourceに対するCST / SurfaceAst / ResolvedAst
- constructorとpatternのcanonical symbolを持つTypedHir
- scrutineeを一度だけ評価し、branch順を明示するCoreIr
- backend固有のtag layoutを初めて決めるTypeScriptIr
- non-exhaustive、unreachable、pattern type mismatchのstructured diagnostic
- `rock-paper-scissors-domain`のgenerated TypeScriptをNodeから呼ぶpure execution probe

exhaustivenessはTypeScriptの`switch`へ委譲しません。closed ADTとtuple patternをSeseragi semanticsで検査し、
CoreIrへ到達する時点で受理済みのdecision treeを持たせます。constructorのruntime表現もSurfaceAstやTypedHirへ
漏らさず、TypeScriptIr / runtime ABIの責務として固定します。

P1-5とP1-6はpure domainがgreenになってから開始します。typed failureは複数failureを暗黙unionせず、既存の
Effect方針どおりuser定義error ADTと`mapError`で寄せます。CLI entryはcold Effect valueを返し、runner境界だけが
実行します。

2026-07-11時点でP1-5の前半として、Effect bodyとEffect diagnosticsもresolver済み`SurfaceExpr`へ移行しました。
do内の`let name = pureExpr`はmonadic bindとは別のTyped / Core / TypeScript statementとして保持し、lexical `const`へ
lowerします。このstatement単独では`flatMap` helperを要求せず、前にEffect bindがある場合だけそのcontinuation内で
評価されます。EffectCall自身も具体化済み`R / E / A`を保持し、`succeed value`は引数型をsuccess型として
CoreIrまで渡します。`fail error`も引数のADT型から`Effect<{}, E, Never>`へ具体化し、runtimeのprivate
typed-failure carrierへ接続しました。failure ADT同士の明示変換は次段落の`mapError`へ接続します。

`mapError mapper source`も、mapperをresolver済みpure function、sourceをEffect expressionとして別々に型付けします。
mapperの入力型とsource failure型を検査し、sourceのenvironment / successを保持したままmapper結果を新しいfailure型に
します。これにより`UnknownHand input |> fail |> mapError InvalidHand`相当の不正入力経路が、pipeline糖衣なしでも
通常compiler pipelineを通るようになりました。

matchのliteral patternもInt / String / BoolをSurfaceAstからTypeScript生成まで縦断します。literalはresolverで
bindingやname referenceを生成せず、scrutinee型との一致をsemanticsで検査します。String / Intのようなopen domainは
catch-allがなければnon-exhaustive、重複literalはunreachableです。Boolは`True` / `False`を有限domainとして証明します。
CoreIrは比較対象とprojectionを保持し、TypeScript backendだけがbigintを含むstrict equalityへlowerします。

# Seseragi 実装計画

この文書は新仕様を実装へ渡すための非規範な計画です。言語の意味は`docs/spec/`、適合入力は
`examples/spec/`を正とします。現行TypeScript compilerの内部構造を互換契約にはしません。

## 1. 目標

- compiler、formatter、LSP、syntax highlight、playgroundが同じfrontendを使う。
- Seseragiの型、Effect、resource、module semanticsをTypeScript checkerへ委譲しない。
- TypeScriptは最初の正式backendであり、source languageの意味を決める層にはしない。
- 複数laneが同時に進んでも、共有AST fileやdiagnostic文字列の編集競合を常態化させない。
- 各段階をstable artifactとconformance fixtureで接続し、未完成な後段なしで前段を検証できる。

言語能力とproduct surfaceの二軸roadmapは[`ROADMAP.md`](./ROADMAP.md)を参照します。CLI、LSP、
playground、formatter、conformanceは最終Phaseへ延期せず、各言語Phaseを人間が利用・検証するthin adapterとして
同じdriver / diagnosticsを使います。

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
  seseragi-runtime         generated moduleのhost execution boundary
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

2026-07-14にsingle-file product gateを接続しました。`seseragi-cli`はfile I/Oとcommand解釈だけを所有し、
`seseragi-driver::compile_module`、shared terminal diagnostic renderer、`seseragi-runtime::run_main`を呼びます。
runtime crateはcompiler artifactから`main`のEffect contract、Console / Stdin requirement、selected `Show<E>`を
検証し、埋め込んだversioned TypeScript runtimeをBun target adapterへstageします。fixtureの`run.json`は読みません。

同日にLSP-0も接続しました。`seseragi-lsp`はstdio framing、LSP request model、capability negotiation、diagnostic wire
変換だけを所有し、open documentのcompileは`seseragi-driver::compile_module`を直接使います。compiler内部rangeは
UTF-8 byteのまま保持し、`seseragi-source::LineIndex`がnegotiated UTF-8 / UTF-16 / UTF-32 positionへ厳密変換します。
これによりLSP adapterはparser、resolver、type checker、diagnostic生成を複製せず、将来のWASM adapterも同じdriverの
structured artifactを利用できます。LSP-0はsingle-file diagnostics gateであり、module graph、hover、completionは
対応する言語能力Phaseへ残します。

Playground-0では`seseragi-wasm`が同じ`compile_module`とpublicなruntime entry contractをversioned JSONへ変換します。
playground adapterは生成TypeScriptをbrowser用Console / Stdin service providerで実行し、compilerやEffect semanticsを
再実装しません。sampleは`examples/spec/lessons/01-hello-world.ssrg?raw`を直接読み、WASM integration testはlesson 01に
加えてPhase 1累積じゃんけんをdeterministic inputで実行します。`bun run test:playground:wasm`がWASM生成とintegration、
`bun run build:playground`がWASMを含むproduction bundleを検証します。

Playground-1では`apps/playground`を旧React / Monaco UIから独立させ、CodeMirror 6、Seseragi専用stream language、
mobile panel、任意Stdinを実装しました。diagnostic rangeはshared UTF-8 byte rangeをUI boundaryでのみUTF-16へ変換し、
compiler diagnosticを再生成しません。初期application chunkは約18 KB、editor chunkは約330 KBです。約3.6 MBのbrowser
TypeScript transpilerはRun時だけlazy loadし、約900 KBのWASMもdriver初回利用時に初期化します。

VercelのGit buildへRust / rustup / wasm-pack installを持ち込むとhost差とdownload failureがdeploymentを壊すため、
`apps/playground/src/wasm/pkg`は例外的にversioned deployment artifactとしてcommitします。Rust compilerまたはWASM
adapter変更時は`bun run build:playground:wasm`で再生成し、browser execution integrationと同じcommitに含めます。
Vercelはfrozen app lockfileとViteだけでbuildします。これはcompiler semanticsのsnapshotを手修正する経路ではなく、
Rust producerからのみ生成する配布binaryです。
`bun run test:playground:wasm`はRust producerから再生成した後のGit差分も検査し、artifactのstalenessをCIで
検出します。

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
| P1-6 | CLI execution | Console / Stdin environmentとrunnerを接続し、正常入力・不正入力のstdout / stderr / trace / exitを比較する |

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
generic payload substitutionは独立sliceで完成させます。nullary constructorを含むstandard `Maybe` / `Either` familyは
resolverでlazyにmaterializeし、関数戻り型、注釈付き`let`、`if` / `match` branch、tuple要素から期待型を渡して
`Nothing` / `Just` / `Left` / `Right`の型引数を具体化するところまで接続済みです。

P1-5のruntime境界も通しています。TypeScript ABIは`Maybe`と`Either`をreadonly tagged unionとして表し、
`readLine`は`Maybe<String>`を返して入力行を`Just`、EOFをsingleton `Nothing`にします。generated moduleは
`core.maybe` / `core.either` requirementを型から収集します。standard constructorはcanonical symbolだけを
backend registryで`@seseragi/runtime/sum`のimportへ解決し、localな同名ADTは変換しません。`Nothing`は
singleton identityを保つruntime reference、`Just` / `Left` / `Right`はruntime callとしてTypeScriptIrに残します。
`standard-sum-values` execution fixtureは生成TypeScriptのpure Unit entryを実際に呼び、4つのtagged valueをJSONで
比較します。`parse-hand-either`はString literal matchを`Either<HandInputError, Hand>`へ載せ替え、typed String引数を
受けるpure executionで正常入力と不正入力を比較します。`effect-parse-hand`はこのpure functionを`fromEither`へ渡し、
`Either<HandInputError, Hand>`から`Effect<{}, HandInputError, Hand>`を推論してgenerated TypeScriptまで接続します。
runtimeは入力caseを構築時に一度だけ保持し、Effectのrun時にRightをsuccess、Leftをtyped failureとして公開します。
conformance runnerはEffectのsuccess / failure payloadをprivate sidecarで観測し、`effect-parse-hand-valid`で
`Rock` successまで比較します。

P1-6のend-to-end sliceとして、`schema-1/rock-paper-scissors-cli/`は`readLine`、`Maybe`、
`Either`、`fromEither`、`mapError`、do内pure binding、Console出力を一つの通常pipelineへ接続しました。
resolverが確定したexternal nominal typeだけをTypedHir / CoreIrのmodule import setへ残し、backendは
runtime ABIの`typeIdentity` / `typeImport`から`StdinError`と`ConsoleError`のtype-only importを生成します。
generated TypeScriptはcold Effect valueを返し、Stdin / Console serviceの構築と実行はhost runnerだけが行います。

`rock-paper-scissors-cli-valid`、`-invalid`、`-eof` execution fixtureは、それぞれUnit success、
`UnknownHand "lizard"`、`EndOfInput`を実runtimeで観測します。`AppError deriving Show`はselected instanceとして
TypedHir / typed interface / CoreIrへ残り、TypeScriptIrはvariantごとのpayload dictionaryを解決します。
backendはcompiler-privateな`__ssrg$instance$Show$0`を実exportし、generated module metadataは
`Show<AppError>`のtype identityとdictionary exportをsource公開exportとは分けて保持します。

runnerはrun.jsonのrequired environmentをgenerated typed interfaceの`Effect<R, E, A>`にあるclosed `R`と照合します。
非`Never`のfailure `E`には一意な`Show<E>`を要求し、local ADTはtype identityでgenerated dictionary metadataを、
standard `String` / `ConsoleError` / `StdinError`はruntime dictionaryを選択します。typed failureはJavaScript throwへ
変換せず、辞書でrenderした一行をstderrへ出してexit code 1に分類します。rendererまたはhost runtimeのdefectだけを
exit code 70に分類します。`capture-console`の実operation traceとConsole / Stdin host failureのtyped channel変換は、
いずれも同じgenerated CLIを実行するexecution fixtureで検査します。

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

P1-5とP1-6はpure domainがgreenになってから進めました。typed failureは複数failureを暗黙unionせず、既存の
Effect方針どおりuser定義error ADTと`mapError`で寄せます。CLI entryはcold Effect valueを返し、runner境界だけが
実行します。

P1-5ではEffect bodyとEffect diagnosticsもresolver済み`SurfaceExpr`へ移行しました。
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

2026-07-12にP1-0からP1-6のsingle-file gateを閉じました。正常入力、不正入力、EOFに加え、host Stdin / Console
failureをtyped channelへ変換するexecution case、captured Console operationのactual trace、matchのnon-exhaustive /
unreachable / pattern type mismatch、欠けたarm bodyから後続armを保持するrecoveryを固定しています。

2026-07-14にP1-6のuser-facing gateも閉じました。`cargo run -p seseragi-cli -- run examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg`
はsourceを通常driverでcompileし、processのstdin / stdout /
stderrへ直接接続します。CLI integration testは正常入力とtyped failure、compile diagnosticのsource rangeを検証します。
package directoryはmanifest / filesystem discovery完成前に推測で実行せず、Phase 2のPackage CLI gateへ残します。

## 11. フェーズ2: moduleへ分割した型付きじゃんけんCLI

フェーズ2の累積goal programは、フェーズ1の機能を別sampleへ置き換えるのではなく、同じじゃんけんCLIを
`domain.ssrg`、`input.ssrg`、`main.ssrg`へ分割したpackageです。ADT、tuple / literal match、Maybe / Either、
typed failure、do内pure binding、Stdin / Console、derived `Show`、正常・不正・EOF / host failure executionを残します。

このgoalから逆算した実装順は次です。

| 順序 | slice | 完了条件 |
| ---- | ----- | -------- |
| P2-0 | compiler driver | callerが物理source labelと論理module identityを別々に渡し、single moduleを全stageへ一度のanalysis結果からcompileする |
| P2-1 | project identity | manifest / root / relative pathを仕様どおりcanonicalizeし、同じfileの二重identity、root escape、case / NFC衝突を拒否する |
| P2-2 | linked imports | dependency `ModuleInterface`からnamed / alias / namespace importの型・値・constructorをresolverへ導入し、private / missing / duplicateを診断する |
| P2-3 | cross-module backend | imported symbol identityをCoreIrへ保持し、TypeScript module importとsource mapを生成する |
| P2-4 | instance closure | transitive import closureのinstance evidenceを選択し、imported derived `Show` dictionaryをgenerated module間で接続する |
| P2-5 | package execution | manifest entryからmodule graphをtopological compileし、分割RPSの全execution caseをフェーズ1と同じ結果で比較する |

P2-0は実装済みです。`seseragi-driver`はfilesystemやmanifestを読まず、project layerが確定したopaqueなmodule IDを
受け取ります。importを物理pathから推測して仮のcanonical symbolへ変換せず、project resolver未接続の間は
`SES-N0104`で停止します。Phase 1累積sourceが既存のTypedInterface、generated metadata、TypeScriptと一致する
regression testを持ち、conformanceのTypedHir以降もこのdriver経由です。

P2-1の最初のcontractとして`seseragi-project`を分離しました。source / test / benchmark / generated rootを
structural identityで区別し、manifest module pathをUnicode NFCへ正規化します。relative specifierは現在moduleの
directoryから同じroot内だけを解決し、`.ssrg`の有無を同じmoduleへ寄せ、root escape、absolute path、backslash、
dot / empty segmentを拒否します。package identityは独自の表示文字列へ潰さず、仕様どおりASCII package name、exact
SemVer、registry content digestまたはcanonical absolute pathのsource identityをstructuralに保持します。manifest /
lockfileからこの値を構築する検証、canonical filesystem pathの取得、case / symlink衝突はまだ実装していないため、
P2-1全体の完了とは扱いません。

module graphの最小contractとして、`seseragi-project::ModuleGraph`はlogical module identityだけをノードに持ち、
dependencyをimporterからdependencyへのedgeとして登録します。topological orderは依存を先に、独立nodeはcanonical
順に返し、未登録dependencyと循環は専用errorで返します。循環errorはcycleに到達する下流node全体ではなく、実際の
back-edge witnessだけを返します。これはsourceの読み込み、interfaceのlink、generated output
pathの選択を行わない純粋な順序付け層です。project loaderがidentityとedgeを確定した後、同じ順序を使ってinterface compileと
output planを組み立てるための基盤であり、graph全体のfilesystem discoveryやpackage executionを完了扱いにはしません。

P2-2の入力contractとして、syntax frontendは明示module identityを受け取る`UnlinkedModuleInterface`を返します。
これは現在moduleのpublic exports / operators / instancesとraw import occurrenceを保持しますが、dependency moduleや
imported symbolのcanonical IDをsource spellingから作りません。従来artifact互換の推測は旧producer内だけに隔離し、
project linkerとpublic driverはこの推測へ依存しません。

project linkerは、project resolverがspecifierごとに確定した`ModuleInterface`を使い、named importをtype / value /
trait namespaceへ展開します。alias、namespace import、operatorのscheme / fixity、duplicate import、missing exportを
import itemのbyte span付きで保持し、dependency bodyを読みません。`LinkedDependency`はtarget interface全体を持つため、
後続semanticsはfunction / constructor schemeとinstance closureをsourceから再構築しません。named type / value /
constructorとoperatorはresolver scopeへ入り、pure function schemeとADT familyをTypedHirへ接続済みです。
namespace-qualified pure callableとtype referenceは選択されたexportだけをcanonical importへ変換します。constructor pattern /
expressionも同じcanonical constructor identityへ解決し、imported ADT familyを使うscalar / tuple matchの網羅性と
unreachable armを検査します。trait / nested namespace、higher-order callable、generic imported ADTは後続の小さいgateに残るため、P2-2全体を
完了とは扱いません。

実compileでlinkerへ渡すのはshallow interfaceの推測値ではなく、dependencyをtopological orderで型検査した後の
`TypedModuleInterface::into_link_interface`です。これによりcompact inferred `effect fn`のR / E / Aをbodyから確定した
public schemeも、dependency bodyをlinkerで再読せずimportできます。shallow interfaceはgraph discoveryとfrontend
artifact用に残し、public contractの最終値として扱いません。

final interfaceの非generic・constraintなし`effect-function` schemeはimport先のEffect bodyでも利用できます。saturated
applicationは`TypedExpr::EffectInvoke` / `CoreExpr::EffectInvoke`としてintrinsic operationと分離し、TypeScriptではcoldな
通常のsource module callへlowerします。部分適用は通常のcurried `Call`として残します。genericまたはconstraint付きの
imported effect functionは、このcatalogを一般化する後続gateです。

同じcanonical spellingをtype / value namespaceへ持てるため、interface consumerはexportをsymbol文字列だけでkeyにせず、
`(namespace, symbol)`で識別します。non-opaqueな公開newtypeは型schemeと、一引数constructor schemeの両方をinterfaceへ
出し、一つのnamed importが両namespaceを導入します。opaque newtypeは型だけを公開します。

公開interfaceだけでは、同一packageの「private宣言は存在する」と単なるtypoを区別できません。そのためfrontendは
同じSurfaceAst passから、bodyと型schemeを含めずdeclaration name / namespace / visibility / canonical symbolだけを持つ
`ModuleHeader`も生成します。private宣言と、公開opaque ADT / newtypeのhidden constructorもheaderへ残し、current moduleの
`LinkedModule`まで保持します。これは公開interface artifactやdependency cacheとして配布しません。同一packageの
`ModuleLinkTarget`だけがheaderを持ち、private nameを`PrivateExport`、存在しないnameを`MissingExport`へ分けます。外部package
targetは公開interfaceだけを持ち、private declarationの存在を漏らしません。これらを`SES-N0102` / `SES-N0104`へ変換する
project diagnostic adapterはまだ未実装です。

`ModuleLinkTarget::same_package`はheaderの全public nameがfinal interfaceに存在することを検証します。したがってcompact
inferred effectのexportが欠けるshallow interfaceを誤ってlink inputへ使うと、semantic linking前に失敗します。
current module headerのnameもlink scopeへ先に予約するため、同じnamespaceのlocal declarationとimportが衝突した状態で
dependency schemeをlocal SymbolIdへ誤結合する経路はありません。

`resolve_linked_module`はlinkerが確定したnamed / alias / operator importをmodule scopeへ登録し、canonical dependency
symbolと完全な`InterfaceExport` schemeを`ResolvedImport`へ保持します。source itemを再解釈してcanonical IDを作らず、
同じnamed importからtype / value namespaceへ入るnewtypeも別symbolとして保持できます。現時点ではこのcontractを
TypedResolutionのpure callableへ接続し、alias経由のimported function callを型検査してcanonical calleeを
TypedHirへ残せます。さらにnon-genericな公開ADTはdependency interfaceの全constructorからsemantic familyを構築し、
importしていないvariantもscopeへ名前登録せずexhaustiveness witnessには残します。これにより選択importだけでmatchを
誤ってtotalと判定しません。higher-order callable、generic imported ADTは後続gateです。

unconstrained rank-1 generic pure functionはlocal / named importの同じcallable pathへ接続済みです。scheme parameterを
semantic type identityとして保持するため、`Int`や`String`だけでなくuser-defined ADTでもcallごとに独立して具体化されます。
型引数binderはTypedHirからCoreIr、TypeScriptIr、generated functionの最外arrowまで保持し、分割RPS projectはdependencyの
generic functionを異なる型で利用したgenerated module setをTypeScript type-checkして実行します。constraint付きfunction、
higher-order parameter / result、generic imported ADTはこのsliceへ混ぜず、個別gateのままです。

imported public callableのscheme内にあるnominal typeは、final provider interfaceが持つtype exportまたはそのdependencyのtype importから
canonical ownerとprovider module / exportを解決し、`ExternalNamed`としてTypedHirからCoreIrへ保持します。named importだけでなく
namespace選択でも同じ経路を使い、direct providerとfacade越しのtransitive providerを区別します。同じ`User` spellingを異なる
dependencyが公開してもoccurrenceごとのcanonical identityを混同しません。backendはprovider closureからexact type-only importと
衝突しないlocal aliasを計画し、schemeでtype自体をsource importしていない場合も必要なprovider outputを追加します。bindingや
output planが欠けた場合はstructured lowering errorとし、local spellingへfallbackしません。

linked moduleのdependency edgeはbinding一覧と分離してTypedHir / CoreIrへ保持します。各edgeはsource specifier、canonical
module identity、originを持ち、named bindingはnamespace、imported / local name、canonical symbolを追加で持ちます。
exhaustiveness専用のscope外ADT memberはbackend importへ漏らしません。bindingが空のedgeも残すため、type-only / namespace
importをruntimeから消してmodule初期化順を壊す経路はありません。raw Seseragi specifierはTypeScript pathではないため、
logical moduleからgenerated output specifierへの変換はP2-3のproject output planが所有します。

P2-3のbackend contractとして、`TypeScriptOutputPlan`はcanonical module identityごとにprojectが確定したESM
specifierを受け取ります。TypeScriptIrはsource module importをruntime ABI helper importと別fieldへ保持し、canonical
value symbolからalias済みbackend localを選びます。異なるmoduleの同名exportもcanonical identityで分離し、明示type
aliasはtype-only bindingを生成します。type-onlyまたはnamespace-only edgeはside-effect importも残すため、TypeScriptの
type erasureでdependency初期化が消えません。emitterとsource mapは共通のimport render planを使い、追加行数を同じ方法で
数えます。public driverはlink済みmoduleと`TypeScriptOutputPlan`を受け、同じanalysis結果からTypedHir、CoreIr、
TypeScriptIr、generated moduleまで通します。module graph全体のoutput path計画、dependency-firstのtopological compile、
namespace-qualified value call / type reference / constructor expression / constructor patternは接続済みです。direct dependencyで
選択したimported `Show` evidenceはdictionary export metadataとTypeScript source importまで接続済みです。一方、trait namespaceと
transitive instance closureは未接続のため、P2-4全体およびmanifest起点のpackage executionの完了とは扱いません。projectが選んだPOSIX形式の生成先pathから
importer相対specifierを作る小さなdriver helperも追加
しました。pathのcanonical性、依存module / 出力pathの重複、entry自身との衝突をdriver境界で検証し、backendは確定済み
specifierを描画するだけにします。これによりsourceの`.ssrg` specifierやcwdをbackendが再解釈する経路を作りません。

さらに、閉じた`ModuleGraph<String>`とsource / generated output pathの入力を受ける`compile_project`をdriverへ追加しました。
graphのlabeled edgeをsource specifierとしてlinkへ渡し、topological orderでdependencyのtyped public interfaceを確定してから
entryをlinked compileします。各nodeは同じ通常pipelineを通り、依存のTypeScript output pathからそのnode専用のoutput planを作ります。
input集合はgraph node集合と完全に一致し、source import specifier集合もgraphのlabeled edge集合と一致しなければ失敗します。
project全体でgenerated ESM output pathの重複を拒否し、`.js` ESM outputから対応する`.ts` / `.ts.map` artifact pathを導出して
generated metadataとsource mapの`file`へ固定します。このAPIはfilesystem discovery、manifest解決、artifact書き出し、host
executionを行わず、project/conformance runnerがそれらを所有できる境界を保ちます。

conformance側にはsingle-module artifactと別の`project-schema-1` laneを置きました。strictな`project.json`が閉じた
graphとsource / output / artifact directoryを明示し、runnerは`compile_project`を一度だけ呼んで全moduleのTypedHir、
TypedInterface、CoreIr、TypeScriptIr、generated metadata、TS、source mapを比較します。専用writerが同じproducerから
artifactを更新するため、snapshotを手で合わせてproject linkerを通ったように見せる経路はありません。最初のdomain-split
じゃんけんfixtureはcross-module ADT、constructor pattern、type importとvalue import、pure callを証明します。別の
`project-schema-1/imported-effect-console`はimported effect callを全stageへ固定します。transitive instance closureは後続gateです。

artifact比較後には、各moduleをmetadataのplanned `.ts` output pathへstageし、生成済みの`.js` ESM specifierを変えずに
project全体を`tsc --moduleResolution bundler`でtype-checkします。これにより、単一module artifactがgreenなだけで
dependency importの型・path解決が壊れている状態はproject compiler gateを通りません。これはcompile artifactの検証であり、
hostでの実行やpackage manifest解決はまだ行いません。

project fixtureの`execution.json`は、project compilerから再度生成したすべてのTypeScript moduleをplanned pathへstageし、
entry wrapperがgenerated metadataの`./dist/.../main.ts`をimportします。pure gateでは`openingMessage Unit -> String`がdomainの
`Rock` / `Scissors` constructorと`decide` / `renderOutcome`を越境して使い、Bun実行のstdoutまで比較します。したがって
artifactだけでcross-module importがgreenに見える状態は避けられます。runtime requirementはentryだけでなく
dependency-firstの全moduleから重複なしで集めます。`imported-effect-console`ではfinal in-memory TypedInterfaceのEffect契約を
検証し、Console environmentを構築してordinary ESM import経由のcold EffectをBunで実行し、stdout、actual operation trace、
success exitを比較します。`rock-paper-scissors-cli-split`はdomain / input / mainの三moduleを一度compileし、同じstaged module setに
対して正常入力、不正入力、EOF、Stdin host failure、Console host failureの五caseをBunで実行します。single-file版と同じ
typed failure payload、derived stderr、stdout、actual trace、process exitを比較するため、分割compileだけをgreenにしてruntime
compositionを未検証にする経路はありません。これはstrict `project.json`から始まるP2-5 execution gateであり、manifest entryと
filesystem package discoveryは引き続き未実装です。

`project-schema-1/imported-effect-failure`は、dependencyの`InputError deriving Show`とcompact `reject` Effectをmainがimportし、
local `AppError`へ`mapError`する組み合わせを固定します。generated mainはdriver output planが渡したexact dictionary exportを
source importし、staged executionはnested typed failure、derived stderr、process exit 1を比較します。これによりdirect
dependency evidenceはsnapshotだけでなくruntimeまで証明されますが、transitive provider chainはまだ対象外です。

project executionはdescriptorのargumentをfinal TypedInterfaceのcurried parameterへ照合し、arity、`Unit` / `String`、pure / Effect
modeをrunner起動前に検査します。Effect R / Eとhost adapterも同じfinal interfaceを正とし、local / importedな同名
`Effect`、`Console`、`Stdin`、standard failure型をruntime builtinへ誤分類しません。`capture-console`を選んだproject caseは
expected traceを必須にし、stdoutだけ一致してoperation / argument / orderが未検証になる経路を閉じています。project専用
descriptorはtop-levelのclosed serde schemaに加え、invocation mode / typed argument / environment record / host service /
exit / trace eventの各objectでもclosed key-setを検査します。legacy `run.json`の互換schemaを変えず、nested caseのassertion typoが
shared `Value` parserに無視されるfalse-greenを拒否します。

一つのprojectはroot `execution.json`か、ID順に読む`executions/<case>/execution.json`のどちらかを持てます。runnerはprojectを
一度だけcompileしてから全caseを実行し、mixed / empty / malformed layoutを拒否します。各nested descriptorのstdin / stdout /
stderr snapshotはcase directory相対で解決するため、case間で期待値を取り違えません。

P2-1以降では、次の二層を維持します。

- 小さいfixture: identity normalization、named / namespace import、private access、cycle、ambiguous import、
  imported constructor pattern、imported instance選択を個別に検査する。
- 累積integration package: 分割RPSをparse、resolve、type、CoreIr、TypeScriptIr、generated code、runtime executionまで通す。

現時点のderived `Show`は、local非generic ADTと限られたpayload evidenceを扱う閉じたsliceです。shallow
`ModuleInterface`の`InterfaceInstance`はidentityなしを許し、final `TypedInterface`だけがtraitとcanonical type identityから
canonical semantic identityを確定します。direct dependencyのinstanceはResolvedAstに保持され、derived `Show`の各variant
payloadはcanonical type identityでLocal / Imported / Standard evidenceを選び、その選択結果はTypedHir、CoreIr、TypeScriptの
dictionary source importとdriver output planまで残ります。
これはdirect dependency範囲のevidence transportであり、一般trait / instance solverやtransitive instance closureの完了とは
呼びません。P2-4の完了には、user moduleが公開したinstanceとdictionaryを別moduleから選択するだけでなく、transitiveな
provider chainを実行まで通すgateが必要です。

project resolverはpackage identityの文法をdriverへ再実装しません。driverのmodule IDはopaqueな入力とし、NFC、root tag、
dependency export map、symlink / case衝突はP2-1の唯一の所有者が決めます。これによりmodule graph追加時にAST、resolver、
TypedHir、CoreIr、runtime ABIを一斉に作り直す経路を避けます。

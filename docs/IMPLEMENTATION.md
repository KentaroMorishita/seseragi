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
| `AnalysisDocument`| diagnostic、symbol / scope、式型、callable、標準Reference query      | lowering、実行、editor固有UI          |
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

Wave 0以降のcompilerは`crates/`へ分離し、Rust driverをcanonical implementationとして育てました。
移行完了後に旧root `src/`のTypeScript compilerと互換commandを削除しています。
`runtime/ts`は生成コードの現行runtimeであり、compiler fallbackではありません。

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
これによりLSP adapterはparser、resolver、type checker、diagnostic生成を複製せず、WASM adapterも同じdriverの
structured artifactを利用できます。

LSP-1ではopen documentの各revisionを`analyze_module`で一度だけAnalysis snapshotへ変換し、hover、completion、
signature help、definition、diagnostic fix由来のquick fix、最小semantic tokenを接続しました。namespace completionは
resolved importと標準module interfaceを使い、local symbolと標準ReferenceはPlaygroundと同じidentity、型、説明を
返します。protocol positionの逆変換も`LineIndex`へ集約し、line外、mid-scalar、mid-surrogate requestはserver errorへ
せずnull / empty responseにします。`extensions/seseragi-spec-preview`はcompilerを複製しない薄い
`vscode-languageclient`になり、platform別VSIXへ同梱したnative stdio serverを既定で起動します。
旧extension IDは0.1.0の上書き更新と二重起動防止のため維持し、表示名とlanguage IDはSeseragiへ統一します。
起動前のversion handshake、initialize結果、status bar、Output Channel、restart commandでbinary discoveryと
互換性、crash recoveryをuserへ見える形にします。workspace references、
rename、workspace symbol、高度なincremental cacheはこのsingle-file sliceに含めません。

human-readable diagnostic sliceでは`seseragi-syntax::Diagnostic`を唯一のpresentation sourceとし、共通の
message、labels、notes、helps、fixes、expected / actual typeをJSONへ直列化します。terminal adapterはsnippetと
caret、LSP adapterはrelated informationとdata、Playgroundはrange navigation付きcardへ変換するだけです。
内部 `messageKey` は回帰fixtureと分類に残しますが、どのsurfaceも表示文字列としては使いません。

同日にformatter-0も接続しました。`seseragi-formatter`は共有lossless token / CSTだけを入力にし、token順、literal、
custom operator spellingを変更せずline ending、indent、trailing whitespace、末尾newlineをcanonical化します。
`seseragi-driver::format_module`がparse diagnosticsとformatter coreを束ね、native CLIはfile I/O、write / check modeだけを
所有します。Phase 1累積programをformat前後でcompileし、TypedHir、CoreIr、TypeScriptIr、generated TypeScriptが不変な
testをgateにします。resolved fixityがpublic artifactになっていない現段階でoperator spacingやline wrappingを
formatter独自に推測しません。

Playground-0では`seseragi-wasm`が同じ`compile_module`とpublicなruntime entry contractをversioned JSONへ変換します。
playground adapterは生成TypeScriptをbrowser用Console / Stdin service providerで実行し、compilerやEffect semanticsを
再実装しません。実行可能sampleは`examples/samples/<stable-slug>/`へmetadata、source、guide、期待出力を
まとめ、生成manifestがPlaygroundへ取り込みます。同じsourceをWASM integrationとnative CLI sample checkで
実行します。`bun run test:playground:wasm`がWASM生成とintegration、
`bun run build:playground`がWASMを含むproduction bundleを検証します。

Playground-1では`apps/playground`へCodeMirror 6、Seseragi専用stream language、mobile panel、任意Stdinを
実装しました。旧React / Monaco UIはRust移行完了後に削除しています。diagnostic rangeはshared UTF-8 byte rangeを
UI boundaryでのみUTF-16へ変換し、
compiler diagnosticを再生成しません。初期application chunkは約18 KB、editor chunkは約330 KBです。約3.6 MBのbrowser
TypeScript transpilerはRun時だけlazy loadし、約900 KBのWASMもdriver初回利用時に初期化します。

mobile surfaceはSample、overflow、Runを一行に固定し、Reference / Reset / whitespaceをfocus管理付きmenuへ移します。
Input panelはOutput headingの`aria-expanded` controlから開きます。analysis tooltipはCodeMirrorのhover sourceを
touch cursorからもactivateし、`visualViewport`のoffset / sizeをtooltip spaceへ渡します。signatureは同じstream
languageのtoken分類を再利用します。diagnostic cardはbyte rangeをdataとして保持し、共通UTF-8 -> UTF-16変換から
Unicode scalar基準の1-based行列を作り、navigation時にmobile Code panelへ戻します。
whitespace設定は行頭space / tabだけをCodeMirror decorationへ載せ、trailing whitespaceは標準highlighterを
再利用します。行中の通常spaceは装飾しません。

Analysis sliceでは`seseragi-driver::analyze_module`を`compile_module`と同じparse / link / semantic frontendへ
接続しました。`seseragi-semantics::AnalysisDocument`はresolverのsymbol / scope graphとTyped HIRを結合し、
diagnostic、symbol、型、完全・部分適用callable、definition、visible symbolをUTF-8 byte rangeで返します。
標準Referenceはtyping registry、Effect operation registry、Prelude sum type、standard module interface、operator
registryから生成します。`seseragi-wasm::analyze_single_file`はこのpure frontendだけを公開し、Playgroundの
debounced revision-safe analysis、CodeMirror hover、live diagnostic、検索可能なReferenceが同じsnapshotを使います。
`analysis-schema-1/shared-queries`はこのJSON contractをcanonical fixtureとして固定します。

sample catalogはLearnとDiscoverを分離します。Learnは複数のlearning path内だけで進捗を示し、
Discoverはtitle、summary、topicの検索とkind、topic、capability、featured/new filterを持ちます。
directoryを一つ追加するだけでcatalogへ自動検出され、中央配列や番号付きfilenameは更新しません。

typed formの最小sliceでは`std/web/html`のinterfaceを唯一の公開契約とし、`InputEvent` / `ChangeEvent`を
read-onlyなopaque struct representationとしてanalysis、type checker、generated TypeScriptへ通します。
browser runtimeはnative DOM Eventを公開せず、targetの`value` / `checked`を一度だけ読んだimmutable snapshotを
typed mapperへ渡します。`click` / `input` / `change` / `submit`はmount rootのdelegated listenerを一度だけ登録し、
再renderではevent binding tableを交換します。submitはMsgをqueueする前に同期的に`preventDefault`します。
preview iframeはevent生成に必要な`allow-forms`を持ちますが、document CSPの`form-action 'none'`で外部送信と
default navigationを拒否します。

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
lockfileからregistry identityを構築する検証はまだ実装していません。local path identityはfilesystem canonical pathから
構築します。entry reachabilityに依存しないsource root auditが全`.ssrg`を列挙し、NFC / case collision、非canonical
spelling、root外symlink、同じphysical file / directoryへの複数logical pathをsource parse前に拒否します。これによりlocal
filesystemに対するP2-1 identity gateは完了です。registry content identityはlockfile resolverの別gateに残します。

P2-1のmanifest sliceでは`seseragi-project::parse_manifest`を追加し、TOML 1.0 decoderから必須package identity input、
default / explicit layout、export map、executable entryとhost policyをtyped modelへ変換します。未知core table、重複key、
型違いは`SES-K0101`相当のsource range付き`ManifestError`になり、package名、exact SemVer、language range spelling、
module path、target ID、seed、signal / shutdown組合せ、layout root overlapをfilesystemへ触れる前に検証します。
dependencyはshort registry、alias付きregistry、local path formを区別するtyped modelへ昇格しました。dependency key、
package name、SemVer range、path spellingとsource指定の排他性をfilesystemへ触れる前に検証します。path formはまだ
manifest単体ではresolved identityを持ちません。`discover_local_package_graph`がpackage rootをcanonicalizeして対象manifestの
name / version / languageを照合し、`PackageIdentity`をnode、dependency keyをedgeにした閉じたgraphへ解決します。同じ
name / exact versionが異なるcanonical sourceから入る状態、declared package name不一致、cycleをcompile前に拒否します。
registry dependencyはlockfile resolverへ意味を委譲し、local graphがversionを選択しません。各nodeのsource module / export
subpathはimporter manifestのdependency keyを最長prefixで選び、graph edgeが持つexact package identityとtarget manifestの
export mapから解決します。未宣言dependencyは`SES-K0103`、公開されていないsubpathは`SES-N0104`で停止し、transitive
dependencyやprivate fileへfallbackしません。foreign、test、benchmark、tool tableは引き続きdeferred TOML valueであり、
この段階ではruntime semanticsを推測しません。

`load_local_project`はroot packageのrun entryからsource importを辿り、relative / `self/` edgeは同じpackage identity、bare
dependency edgeは上記export resolverのtarget identityへ接続します。各packageのsource rootとmodule fileは既存canonical
filesystem resolverを再利用し、`PackageIdentity + ModuleRoot::Source + ModulePath`をnodeにした閉じたgraphを返します。
`package-path-dependency` fixtureではroot `main`、dependency `lib` / `stats`の三nodeと二つのbare import edgeを固定しました。
このloader自身はcompiler stageを呼びません。`compile_local_project` adapterがpackage name / exact versionからopaque
package scopeとmodule IDを割り当て、package / version / module pathを分離したcanonical TypeScript output pathを計画して
shared `compile_project`へ渡します。runtime stagingは従来の一package runnerと同じ`CompiledProject + entry module`実行境界を
再利用します。`package-path-dependency-basic`はdependencyの公開String functionをEffect entryから呼び、CLIで`42`を出力します。
collections回収の最初の縦sliceとして、`schema-1/array-literal`でimmutable `Array<A>` literalを
SurfaceAst、TypedHir、CoreIr、TypeScriptIr、generated moduleまで接続しました。空配列は周囲の`Array<A>`から
要素型を受け取り、文脈がない場合や異なる要素型の混在は`SES-T0101`で停止します。TypeScript backendはtupleへ
潰さず`ReadonlyArray<A>`として保持します。既存full fixtureの`reduce`はgeneric higher-order collection APIとして
独立sliceに残し、fixture名や特定packageへ分岐して通しません。

`schema-1/generic-higher-order-call`では、function型parameterを通常のvalueとしてresolver / type checkerへ保持し、
そのparameterをcalleeにしたcurried callをTypedHir、CoreIr、TypeScriptIrへ接続しました。generic higher-order functionは
callback型の内側もtype parameter substitutionし、backendは`unknown`へ落とさずcurried TypeScript function型を生成します。
constraint付きcallableとcross-module higher-order schemeはinstance / interface evidenceを伴う後続gateであり、local callableの
型を狭めない形で分離しています。

`schema-1/operator-reference`では、parenthesized arithmetic operator `(+)`を通常のcurried function valueとして
SurfaceAst、name resolution、TypedHir、CoreIr、TypeScriptIrへ接続しました。backendはinlineの素の`bigint`演算へ
戻さず、通常のbinary expressionと同じchecked Int runtime helperを参照します。さらに
`schema-1/user-add-operator`では期待される`L -> R -> O`からlocal / imported / scoped `Add<L,R,O>`を一意に選び、
dictionaryの`add` methodをcurried callbackへlowerします。期待型の一部がcalleeの未解決scheme parameterでも、
instance headの既知部分とfunctional dependencyにより一意な候補だけを採用します。fixtureは
`Add<Score,Int,Score>`を`Array<Int>`の`reduce`へ渡し、左右同型という仮定へ固定されていないことも証明します。
期待型がないoperator sectionへ未解決constraintを残すgeneralized value schemeは別gateです。

`schema-1/pipeline-application`では、低優先順位適用`f $ x`とpipeline`x |> f`を型検査前に通常のcurried
applicationへdesugarします。pipelineの右辺には部分適用済みfunctionを置けるため、`value |> add offset`は
`add offset value`になります。両構文はSurfaceAst以降に専用nodeやruntime helperを残さず、通常callと同じ
resolver、type checker、CoreIr、TypeScriptIrを通ります。`execution-schema-1/pipeline-application`は`$`と
左結合pipeline chainの優先順位、checked Int演算、Console出力を生成runtimeで固定します。
operator-firstの複数行pipelineはCSTとSurfaceAstが共通の継続行判定を使います。
`schema-1/rock-paper-scissors-cli`がdo内のbind、pure let、最終resultを複数行へ折り返し、同じgenerated
TypeScriptと正常入力・不正入力・EOFのexecution結果を維持することを固定します。

`schema-1/constraint-arguments`では、`where Reducible<C, A>`をtrait名だけの文字列へ潰さず、型引数を持つ
structured constraintとしてSurfaceAst、shallow / typed interface、TypedHir、CoreIr、TypeScriptIrへ保持します。
`schema-1/array-reduce`では、その構造から標準`Reducible<Array<A>, A>`を飽和call時に選択し、選択済みの
canonical evidenceをTypedHirとCoreIrへ保持してから`@seseragi/runtime/array`へloweringします。対応instanceがない
concrete callは未解決名へfallbackせず`instance.missing`で停止します。`(+)`はfunction valueとして渡すときも
Seseragiのcurried ABIを保ち、checked Int helperを二引数で呼ぶadapterへ生成します。これによりbinary expressionと
higher-order callのoverflow semanticsが分岐しません。

Int算術はさらに、binary expressionとoperator sectionの双方で`Add<Int, Int, Int>`などの標準instanceを型付け時に
選択し、そのevidenceをTypedHirとCoreIrへ保持します。TypeScript loweringはoperator spellingと結果型からinstanceを
再発見せず、選択済みevidenceが対応する標準Int instanceを示す場合だけchecked runtime helperへloweringします。
これは標準Int instanceのevidence transportを固定するsliceであり、custom arithmetic instanceの探索やdictionary dispatchを
標準名への分岐で代用するものではありません。

`schema-1/string-add`は同じoperator typing境界で`Add<String, String, String>`を選び、選択済み
`std/string::Add` evidenceをTypedHirとCoreIrへ保持します。backendは数値変換やchecked Int helperを挟まず、
TypeScriptのString連結へ表現します。`execution-schema-1/string-add`は複数の左結合`+`とcurried function、
`$`、Consoleを組み合わせ、実際の出力まで固定します。これは`+`をoperand型から選ぶ最初の非Int standard
instanceです。

`schema-1/user-add-operator`はこの境界をuser-defined instanceへ開きます。型checkerはbinary `+`の左右operand、または
operator section `(+)`の期待関数型から
標準traitのfunctional dependency `(L, R) -> O`を解き、local / imported / scoped evidenceを通常のinstance
selection順で選択します。TypedHir / CoreIrはordered `Add<L, R, O>` constraintと選択済みevidenceを保持し、
TypeScriptIrはlocal dictionaryの`add` method call、またはそのcurried function adapterを生成します。
`project-schema-1/imported-user-add-operator`ではconsumerがproviderのdictionary exportをimportして同じcallを実行し、
型名やJavaScript tagをbackendで再判定しません。`semantic-diagnostics-schema-1/user-add-missing`は対応instanceがない
concrete operandを、`operator-reference-missing`は対応instanceがないsectionを`SES-T0201 instance.missing`で拒否します。
standard `Reducible<Array<Int>,Int>`とuser `Add<Score,Int,Score>` callbackの組合せに加え、generic
`where Reducible<C,A>` evidenceからuser dictionaryの`reduce` methodを呼ぶ経路も接続済みです。prelude `reduce`は
scoped / local / imported evidenceを標準operation evidenceより先に選び、materializable evidenceだけdictionary callへlowerします。
標準Array / Range / Listは従来の専用runtime ABIを維持するため、存在しないstandard dictionaryを捏造しません。
struct / newtype `operator`糖衣は後続です。

`schema-1/pure-comparison`はInt、Bool、Stringの`==` / `!=`で、それぞれのstandard `Eq<A>` evidenceを
TypedHirとCoreIrへ保持します。backendのstrict equality表現は変えませんが、型検査後にoperand spellingから
instanceを再発見しません。`execution-schema-1/pure-comparison-string`はcurried pure entryを二つのStringで
実行し、`!=`の結果をJSON Boolとして固定します。

`schema-1/user-eq-operator`は同じ境界をuser-defined `Eq<A>`へ開きます。`==`は選択済みlocal dictionaryの`eq`を呼び、
`!=`は別instanceを選ばず同じ`eq`結果を否定します。generic `where Eq<T>` bodyではparameter evidenceを保持し、
具体callはlocal dictionaryを渡します。`project-schema-1/imported-user-eq-operator`はprovider dictionary exportのimportと
closed TypeScript check、actual executionまで固定します。`semantic-diagnostics-schema-1/eq-missing`はinstanceのない
ADT比較を`instance.missing`で停止するため、JavaScript object identityへ黙ってfallbackしません。`deriving Eq`は後続gateです。

`schema-1/equality-operator-section`は標準operator registryで`(==)` / `(!=)`を関数値として認識し、期待される
`A -> A -> Bool`から通常の`Eq<A>` instance selectionを実行します。選択済みevidenceは専用nodeを増やさず
TypedHir / CoreIrのVariableへ保持し、local / scoped / imported dictionaryはcurried `eq` callbackへlowerします。
standard Int / Bool / String evidenceは共有strict-equality registryで検証して`===` / `!==`へlowerし、型名から
backendでinstanceを再選択しません。同名execution fixtureはlocal / generic scoped `Eq<Status>`とstandard `Eq<Int>`を実行し、
`semantic-diagnostics-schema-1/equality-operator-section-missing`は未解決evidenceを`SES-T0201`で停止します。
`schema-1/type-class-operator-section`は同じ通常application経路を`(<$>)` / `(<*>)` / `(>>=)`へ
伸ばします。共有`StandardTraitOperator` registryはsource operandからtrait method引数へのpermutationを持ち、
`>>=`のsource順`M<A> -> (A -> M<B>) -> M<B>`をdictionary ABIの`flatMap f value`へ並べ替えます。
TypeScript loweringは0 / 1 / 2個の既適用source引数を一つのadapterで扱い、bare section、部分適用、飽和callを
専用TypedHir / CoreIr nodeなしで生成します。lexicalな同名trait identityを既存resolverから引き継ぐため、userland
`Monad`でもinfixとsectionは同じdictionaryへdispatchします。fixtureはPrelude Maybe evidence、generic
`where Monad<M>` parameter evidence、Nothingのshort-circuitをactual executionし、対応instanceのない`Box`は
`SES-T0201 instance.missing`でlowering前に停止します。`<` / `<=` / `>` / `>=`の`Ord` sectionは後続sliceです。

operator sectionの可否は`OperatorSectionPolicy`へ集約します。算術・Eq・型クラスoperatorとvalid custom candidateだけを
関数値候補にし、短絡operator、pipeline、低優先順位application、Signal set、prefix not、rangeはoperator spanを持つ
Surface errorへ変換します。未接続の`Ord` / cons sectionも同じcompile boundaryで停止するため、body欠落がholeや
TypeScriptの`_`としてruntimeへ流れません。`schema-1/operator-section-forbidden`は各固定spellingを
`SES-P0001 parser.expected-expression`で拒否し、driverはparse error時点でloweringを開始しません。

user-defined instanceへ進む前提として、SurfaceAstの`instance`はhead / constraintだけでなく、各methodの型parameter、
parameter、戻り型、constraint、body、source spanを保持します。resolverもinstance scopeを親にmethod scopeを作り、
method signatureとbodyの型名・値名を通常のreferenceとして解決します。以前のようにfrontendでmethod bodyを捨てると、
instance searchだけを追加してもruntime dictionaryを生成できないため、この保持を先行gateにしています。methodのtrait契約との
型照合、coherence、TypedHir / CoreIr dictionary表現は後続sliceで接続し、このfrontend保持だけをinstance完成とは扱いません。
同じ理由でpublic `trait`のmethod signatureもSurfaceAstと`ModuleInterface`へ保持します。trait type parameterを外側の
contract、method固有type parameter / constraint / curried signatureを各method schemeとして分離するため、imported traitの
契約をcompiler-privateな標準名表へ戻さず、dependency interfaceから型照合できる境界になります。
`instance Trait<...>`のtrait名と、declaration / methodの`where Trait<...>`もsource上のname spanを保持し、
value / typeとは別のtrait namespaceでlocal、import、prelude symbolへ解決します。したがって後続のmethod契約照合と
instance searchはtraitの綴りを再解析せずresolved symbol identityを正にできます。未解決traitはinstance headでも
constraintでも通常の`SES-N0001`になり、標準traitはspecのprelude catalogからcanonical identityを得ます。
local user-defined instanceは、resolved trait identityから対応するtrait declarationを選び、type argumentをtrait parameterへ
置換してmethod集合、body有無、curried signature、method固有generic parameter、constraintを照合します。型比較はsource名を
再解析せずresolved type / trait symbol identityを使い、method genericはalpha normalize、record fieldとconstraint順はcanonicalize、
`F<A>`と`Either<E, _>`のような型構築子適用はhole substitution後に比較します。
`semantic-diagnostics-schema-1/{instance-contract,instance-contract-mismatch}`がpositive / negative diagnosticを固定します。これはlocal契約検査のgateであり、
instance searchとmethod call dispatchのgateとは分離します。

named importで得たpublic traitは、`ModuleInterface`に保持したmethod schemeを同じcontract modelへ正規化して照合します。
trait / method type parameterはinstance headで置換・alpha normalizeし、prelude type / traitとprovider nominal typeはcanonical identityで
比較します。provider側のsource spanをconsumer sourceへ誤表示しないため、cross-module contract diagnosticの関連位置はimport spanを
anchorにします。`project-schema-1/imported-trait-instance-contract`はtrait export、named import、consumer側instance契約検査を
closed projectのTypedHir / TypedInterface / CoreIr / TypeScriptIr / generated moduleまで固定します。これはinterface契約の検査gateであり、
instance method bodyやdictionaryがbackendへ残ることの証明ではありません。imported method constraintがprovider-localまたは
provider dependencyのtraitを参照する場合は、provider interfaceからcanonical trait bindingをresolved importへ運びます。
bindingが曖昧または解決不能な場合は綴り比較へfallbackせず、誤った契約一致を作りません。

instance headのTypedHir以降のschemaは`trait`とordered `arguments`を分離して保持します。従来の単一`head`型は
derived `Show<A>`のprimary typeしか表現できず、`Add<L, R, O>`や`Iterable<C, A>`を第一引数へ潰すため廃止しました。
`typeIdentity`はruntimeがnominal dictionary ownerを検証する場合だけ存在するoptional metadataです。coherenceとinstance selectionは
最終的に全argumentを含むcanonical `identity`とstructured headを正とし、`typeIdentity`へ意味判断を委譲しません。
`schema-1/user-instance-dictionary`では、local custom traitのconcrete instanceをcanonicalなtrait / type identityで
TypedHirへ選び、method parameterとbodyをCoreIr、TypeScriptIrへ運び、compiler-privateなdictionary objectとして生成します。
shallow interfaceのinstance headはfinal typed interfaceでcanonical evidenceへ置換し、一つのsource instanceを二重公開しません。
またcustom dictionaryだけでは`core.show.dictionary`を要求しないため、derived `Show`専用runtime ABIを一般instanceへ漏らしません。
instance type parameter、ordered head argument、constraintは各IRに独立して残し、単一primary typeや標準trait名へ一般機構を固定しません。
その前段として、instance method bodyはtop-level pure functionと同じresolved-expression analyzerとdiagnostic collectorを共有します。
`semantic-diagnostics-schema-1/instance-method-body`がdeclared `String`に対する`Int` bodyを`SES-T0101`として固定し、
instance専用の小型parserや型推論経路を増やしません。

同fixtureの`label`はscope内の`render value`を通常のcurried callとして解決し、argument型からconcrete local instanceを
一意に選択します。選択済みtrait identity、method、instance evidenceをTypedHirとCoreIrへ保持し、TypeScriptIrの
`dictionary-call`と生成TSのdictionary method invocationまで接続します。backendはruntime tagやJS constructorを見て
instanceを再選択しません。`schema-1/trait-method-candidates`では、resolverが同名methodを候補集合として保持し、
型checkerがargument / expected typeと利用可能なinstance evidenceを照合してtraitを一意に選びます。二つ以上残るcaseは
`semantic-diagnostics-schema-1/trait-method-ambiguous`で`SES-T0202`に固定します。
`schema-1/generic-instance-dispatch`は、constraintを持たないgeneric local instanceのheadからordered type argumentを推論し、
selected evidenceをTypedHir / CoreIrへ保持してgeneric dictionary factoryをTypeScriptIrの式として呼び出します。
backendはfactory適用をsource文字列へ連結せず、型引数と将来のevidence引数を持てる`type-application-call`として表現します。
これにより`instance<T> Tag<Maybe<T>>`を`Maybe<Int>`へ使うと、生成TSは対応するfactoryを`<bigint>()`で具体化してから
methodへdispatchします。`execution-schema-1/generic-instance-dispatch`は同じ生成moduleをConsole hostで実行し、
operation traceとstdoutを固定します。

`schema-1/constrained-instance-dispatch`は同じ式表現を使い、generic local instanceのordered constraintごとに
必要なlocal instanceを再帰選択します。selected evidence treeはTypedHir / CoreIrの`evidenceArguments`へ残し、
TypeScriptIrは外側factoryの型引数と内側dictionary expressionを別fieldで保持します。生成TSは
`Render<Maybe<Badge>>`のfactoryへ`Ready<Badge>` dictionaryを明示的に渡し、backendやruntimeが型名・tagから
instanceを再選択しません。循環するlocal evidence chainはcall siteの`instance.missing`として停止します。
`execution-schema-1/constrained-instance-dispatch`が生成module、Console trace、stdoutまで固定します。

instance constraint scopeはresolved trait identity、型引数、ordered parameter indexとしてexpression typingへ渡します。
`Just item -> ready item`のtrait callはlocal / standard instanceを再探索せず`parameter` evidenceを選び、
TypedHir / CoreIrからTypeScriptIrのcompiler-private evidence parameter経由のmethod callへlowerします。
parameter名にはSeseragi identifierと衝突しない`$`付きgenerated nameを使います。これにより`unknown` parameterの存在だけで
完了扱いできず、actual executionが内側dictionary methodの結果を観測します。

このsliceの再帰materializationはまずlocal evidenceとinstance-level constraint scopeを対象にします。
imported dictionary valueをfactory引数へ渡す経路は後述のdirect-provider project fixtureで接続済みです。
runtime dictionaryが実在するstandard `Show<Int>` / `Show<String>`をfactory引数へ渡す経路は
`schema-1/standard-show-evidence`で接続済みです。`Add` / `Eq` / `Iterable` / `Reducible`のstandard evidenceは
専用operation ABIのままで、dictionary valueを捏造しません。provider moduleを跨ぐ
constrained function callについても、後述のproject fixtureでconsumer-local evidenceまで接続します。

`schema-1/method-constraint-dispatch`はtrait method schemeの固有constraintを通常のcallable constraint列へ保持し、
primary trait dictionaryを先頭、method固有dictionaryを後続evidenceとして選択します。instance method bodyでは
instance-level constraint数をoffsetにした`parameter` evidenceを使うため、generic dictionary factoryのclosureと
method closureが同じgenerated名を誤って捕捉しません。TypeScript ABIはvalue parameterの後ろにmethod evidenceを
curried parameterとして追加し、`render(value)(labeledDictionary)`を生成します。missing evidenceはcall siteで
`instance.missing`となり、execution fixtureは内側の`label` dispatch結果を観測します。

ここまでで標準Array instance、local concrete user-defined instance、unconstrained generic local instance、
constraint付きgeneric local instanceの選択、evidence transport、dictionary dispatchを別々のfixtureで証明しました。
direct dependencyのconcrete user-defined instance searchとdictionary importも後述のproject fixtureで接続済みです。
materializableなstandard `Show<Int>` / `Show<String>`もlocal generic factoryのconstraint argumentへ接続済みです。
残るcoherenceの一般化、unresolved polymorphic constrained value、operation-only standard traitのdictionary ABIは
まだ完了gateではありません。instance-levelとmethod-specific scoped evidence、consumer-local evidence、direct providerの
generic / constraint付きfactory、transitive concrete / generic / constraint付きprovider dictionaryは同じcross-module ABIへ接続済みです。
Phase 3の一般trait / instance goal programではconcrete headだけを通る経路と区別して残りを回収します。

続く`schema-1/constrained-function-dispatch`は、local generic pure functionの`where Ready<T>`を
bodyのordered evidence scopeへ接続します。body内の`ready value`は`parameter` evidenceを選び、飽和call siteは
具体型からlocal `Ready<Badge>`を選択します。生成TSは通常のcurried value parameterの後ろへcompiler-private
dictionary parameterを置き、`describe(Active)(readyDictionary)`として呼びます。
`execution-schema-1/constrained-function-dispatch`が同じgenerated moduleをConsole hostで実行するため、
constraintをinterfaceへ保存しただけでも、bodyを特定型へmonomorphizeしただけでも完了になりません。

Phase 3の最初のHKT gateとして、type parameterを名前だけでなくarity付き`TypeParameter`としてfrontend / interfaceへ
保持します。通常parameter `A`はarity 0、`F<_>`はarity 1であり、instance headもtrait parameterが要求するkindと
照合します。`schema-1/type-constructor-kind-mismatch`は`Functor<Int>`相当をsource range付き
`trait.instance-kind-mismatch`で拒否し、method signatureの偶然の一致だけでkind errorを通しません。

`schema-1/functor-maybe`は`F<A>`とconcrete `Maybe<Int>`のunificationから型構築子`F = Maybe`と要素型を
同時に推論します。generic `transform<F<_>, A, B> where Functor<F>`のbodyは既存のconstraint parameter evidenceを、
concrete call siteは既存のlocal generic instance selectionを使います。したがってFunctor専用runtime helperや
`Maybe`名のbackend分岐を追加せず、選択済みdictionaryを通常のTypeScript dictionary callへ運びます。
`execution-schema-1/functor-maybe`は生成moduleをversioned runtimeで実行し、`Just 42`とConsole traceを固定します。

`schema-1/partial-functor-value`はpartial applicationの期待型を最終resultだけへ誤適用せず、未適用parameterを含む
残りの関数型全体へ照合します。`map increment`に期待される`Maybe<Int> -> Maybe<Int>`から`F = Maybe`を求め、
`Functor<Maybe>` dictionary methodを部分適用した値として`applyMapper`へ渡します。TypedHir / CoreIrは選択済みevidenceを
保持し、TypeScriptIrは通常のdictionary callを返すため、Functor専用closure nodeやruntime tag判定を追加しません。
execution fixtureが`Just 42`を観測します。

`schema-1/polymorphic-partial-constrained-function`は、concrete instanceをその場で選べないgeneric callerでも、outer
`where Ready<T>`のparameter evidenceを部分適用closureへ捕捉します。`schema-1/polymorphic-partial-functor`は同じ機構を
`where Functor<F>`と`transform f : F<A> -> F<B>`へ適用し、higher-order `applyMapper`へ渡します。TypedHir / CoreIrの
`deferredEvidenceTypeConstructorParameters`はdeferred parameterに含まれるHKT parameter名を明示し、backendだけが
`F<A>`を`unknown`へ消去します。constraint selection、kind、evidenceはTypeScript checkerへ委譲しません。二つの同名
execution fixtureはouter dictionary捕捉をactual dispatchで固定します。なお、outer evidenceを捕捉するvalueと、
constraint付きvalue schemeを独立してgeneralizeし後からdictionaryを受け取るvalueは区別し、後者は別gateに残します。

`project-schema-1/imported-functor-dispatch`はHKT function schemeをmodule ABIへ接続します。TypedDeclの
type-constructor parameterは名前だけでなくsourceのarityを保持し、final TypedInterfaceの`F<_>`を通常parameter `F`へ
潰しません。consumerはimported `incrementAll<F<_>> where Functor<F>`へ`Maybe<Int>`を渡して`F = Maybe`を推論し、
provider moduleのFunctor dictionary exportを通常のimport planningで受け取ります。closed TypeScript checkとactual Effect
executionまで通すことで、shallow interfaceだけがarityを持つ偽の完了経路を排除します。

`project-schema-1/imported-higher-order-functor`はこのABIへfunction valueのmodule transportを重ねます。providerの
`transform<F<_>, A, B> (A -> B) -> F<A> -> F<B>`をconsumerがlocal `increment`と`Maybe<Int>`で具体化し、function
parameterの構造的型検査、HKT推論、provider dictionary importを同時に実行します。imported callable catalogはinterface内の
function typeを再帰localizeし、誤ったmapper型は`SES-T0101`で拒否します。

`project-schema-1/imported-generic-adt-functor`はprelude型ではないpublic `Box<A>`を同じHKT経路へ接続します。dependency
resolverはimported ADT parameterへowner由来のcanonical symbolを与え、semantic catalogはconstructor payload内のparameterを
再帰的に置換します。consumerは`Boxed 41 |> transform increment`を書き、`Box<Int>` exhaustiveness、providerの
`Functor<Box>` dictionary、type-only `Box` import、runtime constructor importをclosed compileとexecutionで同時に固定します。
直接payloadだけでなく`Wrapped Maybe<A>`のnested substitutionもunit regressionで保持します。

`project-schema-1/imported-generic-adt-monad`はこのmodule ABIをFunctorだけで止めません。providerのuser-defined
`Box<A>`へFunctor / Applicative / Monadを積み、consumerはpublic `bind<M<_>, A, B> where Monad<M>`とconsumer-local
pure `do`を同じimported `Monad<Box>` evidenceで具体化します。evidence import planningはMonad factoryだけを孤立して
呼ばず、providerのApplicative factory、その親Functor dictionaryを順にmaterializeします。pure `do`はconsumerの
TypedHirでこのnested evidenceを保持し、生成TSは選択済みdictionaryの`flatMap`だけを呼びます。二経路のactual executionが
同じ42を出すため、local-only HKT、standard type name、backend tag分岐のいずれでも完了扱いできません。

TypeScriptにはnative HKT applicationがないため、`F<A>`をそのまま不正なTypeScript型として生成しません。
CoreIrはtype-constructor parameter名を保持し、backend parameter annotationだけを`unknown`へ消去します。
これはSeseragiのkind / type / instance selectionをTypeScriptへ委譲するものではなく、意味確定後のtarget ABI消去です。
concrete `Maybe<A>`などは従来どおりtagged union型を生成します。

`schema-1/applicative-maybe`はsupertraitを単なる宣言metadataにせず、dictionary ABIへ接続します。
`instance Applicative<Maybe>`の選択時には`Functor<Maybe>` evidenceを再帰選択し、Applicative factoryは受け取った
Functor dictionaryをspreadして`map`、`pure`、`apply`を一つのdictionaryとして公開します。instance定義時にも
required supertrait instanceを検査し、欠けていれば`trait.supertrait-instance-missing`で拒否します。
generic `where Applicative<F>` scopeは親`Functor<F>`を同じparameter indexへprojectするため、bodyは追加の暗黙引数なしで
`map`を使えます。これは`Maybe`名やruntime tagによる分岐ではなく、resolverが確定したtrait identityとtyped constraintから
構築したevidence treeです。

同じfixtureは`mapped`、`lifted`、`applyWrapped`を一つのpipelineで合成します。引数を一括して未解決期待型で型付けせず、
左から確定したsemantic argumentを後続引数の期待型へ反映します。また、rootが未解決type-constructor parameterの期待型は
constructor expressionへ押し付けず、実引数または外側のconcrete resultから具体化します。これにより入れ子になった
saturated constrained callの内側と外側へ、それぞれApplicative dictionaryを挿入できます。
`execution-schema-1/applicative-maybe`は生成moduleとversioned runtimeで`Just 42`を観測します。

`schema-1/applicative-validation`はApplicativeを単なるMaybeの成功経路で終わらせず、独立error accumulationへ接続します。
user-defined `Validation<E, A>`とrecursive `Errors<E>`を定義し、`instance<E> Applicative<Validation<E, _>>`の`apply`が
両側Invalidならordered error列を連結します。`pure makeUser <*> validateName name <*> validateAge age`は同じgeneric
dictionary selection / partial HKT applicationだけを通り、生成TSのactual executionで二errorの順序とValid側を固定します。
Monad instanceは意図的に存在しません。標準`Validation<E, A> = Invalid (NonEmptyList<E>) | Valid A`の公開moduleとruntime
ABIはこの意味gateを再利用する後続stdlib sliceであり、user-defined型名をcompiler builtinへ昇格させません。

`schema-1/monad-maybe`は同じ仕組みを`Monad<M> where Applicative<M>`へ一般化します。local Monad instanceの
factory argumentは具体化済みApplicative dictionaryで、その内部にFunctor dictionaryも含まれるため、生成Monad
dictionaryは`map`、`pure`、`apply`、`flatMap`を同じevidence objectとして公開します。`where Monad<M>` scopeでも
transitive supertraitを同じparameter indexへprojectし、`pure value |> flatMap f`を追加dictionaryなしでlowerします。

この入れ子callではcalleeの型構築子parameterとcallerの型構築子parameterが偶然同じ名前でもscopeは別です。
application inferenceは「置換後の表示名」だけで未解決判定せず、どのtype parameterが実引数または期待結果から
確定したかを保持します。これによりouter `M`でinner `M`を具体化した期待型をconstructor inferenceへ渡せます。
`execution-schema-1/monad-maybe`は`liftAndBind`と`bind`を通る`Just 44`、および`Nothing` short-circuitを観測します。
`<$>`、`<*>`、`>>=`は専用TypedHir/CoreIr nodeを増やしません。Surface ASTで順に`map f value`、
`apply wrapped value`、`flatMap f value`へdesugarし、operator tokenのrangeを合成したmethod nameへ保持します。
その後は通常のtrait method候補選択、constraint具体化、evidence挿入、dictionary emitを通ります。
lexerは3演算子を単一custom-operator tokenとして保持し、`|>`と同じ低優先順位・左結合でparseします。
line continuationもcustom operatorを認識するため、`value`の次行から`>>= next`を続けられます。

一般のtop-level custom infix operatorは、この固定3演算子へspellingを追加する方式ではありません。
Surface parserは未知custom operatorを含む式を、standard operatorも含めたsource順のflat chainとして保持します。
`<^>`のようにlexer上で複数tokenへ分かれる連続operator runも、declaration / importと共通の文字分類で一つの
spellingとrangeへ戻します。dotを含む`<.>`も同じraw spellingとして扱います。2文字未満、`<` / `>`だけのsymbol、
二項でない宣言は`SES-P0001`で拒否します。resolverは全local headerとdependency interfaceを得た後、一つのfixity tableでchainを
再結合します。standard operatorは既存Binary / applicationへ、custom operatorは通常のcurried function callへ正規化するため、
型推論とloweringはoperator名ごとの分岐を持ちません。`schema-1/custom-infix-operator`とexecution fixtureが
左・右結合、use-before-declaration、compiler-private TS名、actual resultを固定し、unknown / non-associative conflictは
`semantic-diagnostics-schema-1/custom-operator-{unknown,fixity-conflict}`でlowering前に拒否します。
imported operatorも同じpathを使います。linkerがoperator namespace、fixity、public function schemeを保持し、
semantic typingはprovider-local type / trait identityを通常のimported callable bindingへ解決します。TypeScript loweringは
canonical symbolから同じbyte-encoded ABI名を導出してES module bindingを作り、source spellingをidentifierへsanitizeしません。
`project-schema-1/imported-custom-infix-operator`がprovider export、consumer import、右結合call、closed typecheck、
actual executionを一つのfixtureで固定します。

括弧で囲んだcustom operator `(<^>)`は専用のsection nodeへ分岐せず、operator namespaceで解決される通常の
`Name`を`Grouped`に保持します。application flatteningはgroupを透過して既存のcallable schemeを具体化するため、
higher-order parameterへの受け渡しと部分適用は通常のcurried functionと同じ経路です。0 value argumentのconstrained
callableも既存のpartial constrained callへ合流し、期待関数型からgeneric argumentを具体化して、選択したdictionaryを
残りのcurried value parameterの後へ捕捉します。local bindingとimport bindingはcanonical `operator(...)` symbolから
同じbyte-encoded TypeScript名を導出し、spellingごとのbackend分岐を持ちません。spelling内に`::`を含む場合も
canonical markerを先に切り出すためmodule separatorと混同しません。`schema-1/custom-operator-section`とexecution fixtureが
local generic constrained function value / partial application、lowering testがimported function valueとprovider evidenceを
固定します。未解決sectionはoperator namespaceの`SES-P0101`、期待型もscoped evidenceもない未具体化constraintは
`SES-T0201`でbackend前に拒否します。

pure functionの`do`はEffect用`TypedExpr::DoBlock`へ混ぜず、`TypedExpr::MonadDo`として分離します。
宣言済みreturn type `M<A>`から型構築子`M`を取り出し、通常のlocal / parameter / imported instance探索で
`Monad<M>` evidenceを選択します。各bind値も同じ型構築子の適用であることを型検査し、payload型を後続scopeへ
渡します。CoreIrは選択済みevidenceを保持し、TypeScriptIrはそのdictionaryの`flatMap`でcontinuationを構築します。
pure letはcontinuation内の通常constであり、Effect runtime importやrequirementを増やしません。
`schema-1/monad-maybe`の`addMaybe`は二つの`Maybe<Int>`をbindして合計を`pure`し、成功時`Just 42`、
片方が`Nothing`なら後続continuationを実行せず`Nothing`になることをexecution fixtureで固定します。
この経路は`Maybe`のtagや型名をbackendで検査せず、選択済みuser dictionaryだけを呼びます。

`semantic-diagnostics-schema-1/monad-do-invalid`はdo bindのrefutable patternを
`do.refutable-bind-pattern`、異なる型構築子の混在を`do.monad-constructor-mismatch`、final expression欠落を
`do.missing-final-expression`として固定します。いずれも`SES-T0101`ですが、trait method選択失敗へ偽装せず、
原因となるpattern / expression / do blockのsource rangeをprimaryにします。

`schema-1/monad-either`は`Either<String, Int>`を、固定prefix `Either<String, _>`と最後のpayload `Int`へ
分解します。`F<A> ~ Either<String, Int>`のHKT inferenceは`F = Either<String, _>`を保持し、substitution時は
固定prefixの後ろへ新しいpayloadを適用します。Preludeの標準instance照合は型構築子arityから
`Either<String, _>`の残りarity 1を確認し、saturatedな値型や異なる型構築子を誤選択しません。
これによりFunctor / Applicative / Monadの標準dictionaryとdo loweringはcall siteのEither専用分岐なしで再利用され、
`execution-schema-1/monad-either`がdo版と明示lambdaによる`>>=`版の両方でRightの成功とLeftの短絡を
実行します。application typingは非lambda実引数を先に型付けしてgeneric substitutionを作り、元の引数位置に
対応する期待関数型をlambdaへ渡します。これは`Either`や`flatMap`名の判定ではなく、任意のhigher-order callへ
使うindexed argument inferenceです。解決不能なlambda parameterはHoleのままbackendへ送らずcompile diagnosticにします。

Prelude調査時点では、semantic preludeはtrait名とMaybe / Either constructorだけを登録し、trait method schemeと
標準instanceはuser sourceで再宣言しなければ使えませんでした。またbackendでmaterializeできる標準dictionaryは
Showだけで、sum runtime ABIはconstructorと表現に留まっていました。今回の最小sliceでは
Functor / Applicative / Monadのmethod schemeとsupertrait chain、Maybe / Eitherの6標準instanceをsemantic registryへ
集約し、同じidentityをlowering registryからreadonly runtime dictionaryへ接続します。Monad / Applicative dictionaryは
継承methodも同じobjectに含むため、supertrait evidenceのparameter projectionと同じ呼出規約を保ちます。
`monad-either`はこのPrelude経路へ移し、`monad-maybe`はuserlandでtraitとinstanceを定義できる独立sampleとして残します。
`schema-1/array-monad`はこの境界へArrayのFunctor / Applicative / Monadを追加します。runtime dictionaryは
既存`@seseragi/runtime/array`に置き、mapは順序保持、pureはsingleton、applyはfunctionを外側にしたCartesian product、
flatMapは各結果のsource-order連結として実装します。fixtureは3 dictionary identityを個別に選択してactual executionし、
Playgroundにも同じPrelude-only sourceを公開します。
`schema-1/list-monad`は同じ3 instanceをpersistent `List`へ追加します。各辞書は既存の`Empty` / `Cons`表現と
`fromArray` constructorを再利用し、mapは要素順を保持、pureはsingleton `Cons`、applyはfunction-majorの
Cartesian product、flatMapは各persistent Listをsource-orderで連結します。辞書の戻り値はArrayへ変換せずListのままです。
execution fixtureとPlayground sampleは3操作をPrelude宣言だけで実行し、runtime package probeはimmutable nodeと
dictionaryの順序規約を合わせて検査します。
`schema-1/effect-monad`は`Effect<R, E, _>`の固定environment / failure prefixを既存の部分適用型構築子推論へ
載せ、同じ3標準instanceを追加します。effect body内の通常applicationは共通のgeneric application推論を再利用しつつ、
子のEffect式をfirst-class `Effect<R, E, A>`引数として扱います。do bindでは同じ値からsuccess `A`だけを取り出します。
runtime dictionaryはcold性を保ち、mapはfailureを変更せず、applyはfunction effectからvalue effectの順に実行し、
flatMapは既存のenvironment intersection / failure union primitiveを使います。`Effect` runtime type importと`Never`の
TypeScript bottom typeもABIへ固定し、fixture、runtime probe、Playgroundが辞書経路を実行します。
`stdlib-schema-1/prelude/module.json`はsemantic registryから標準module surfaceを生成し、trait identity、HKT method
scheme、supertrait、15 standard instance、言語versionをconformanceで比較します。同じregistryはlocal user instanceの
headも検査し、登録済みtrait / 型構築子の末尾一引数をopenにしたheadと重なる場合は
`trait.instance-duplicate`でcompileを止めます。Maybe / Array / Listのarity 1、Eitherのarity 2、Effectのarity 3を
個別のcoherence分岐へ複製せず、既存type constructor arityを使います。これは標準headをsealedにするgateであり、
user-defined trait / typeを含む一般orphan ruleや標準moduleをpackage graphからimportする機構の完了ではありません。

`schema-1/monad-laws`はFunctor identity / composition、Applicative identity / homomorphism、Monad left identity /
right identity / associativityを一つの小さいuser-defined Maybe instance群で表現します。
`execution-schema-1/monad-laws`は七つの比較結果をConsole traceとstdoutまで固定し、selected dictionary、supertrait
factory chain、生成TypeScriptの意味がlawfulな代表instanceを壊していないことを検査します。これは任意の
user-defined instanceのlawfulnessを静的に証明する機能ではありません。

constraint付きimported instance factoryは、direct providerのmodule境界でmaterializeするところまで接続済みです。

このgateは飽和するconstrained function callに加え、concrete evidenceを選べるpartial callまで対象にします。
partial trait methodは期待関数型からevidenceを選び、`partial-functor-value`でfirst-class valueとして保持できます。
`partial-constrained-function`は通常のtop-level `where Ready<T>` callでも、TypedHir / CoreIrへ残りのvalue parameter型を
明示します。TypeScript backendはそのparameterを受け取るeta-expanded closureを生成し、全value argumentの後ろへ
dictionaryを渡す既存ABIを維持します。飽和callの結果自体がfunction型である場合はこのmetadataが空なので、誤って
eta expansionしません。outer generic scopeのunresolved type parameterに対応するdictionary captureはpolymorphic partial
fixtureで接続済みです。一方、constraint付きvalueをouter evidenceなしで独立してgeneralizeするにはschemeへconstraintを
保持する一般機構が必要です。runtime dictionaryが実在するstandard `Show<Int>` / `Show<String>`は
factory引数へ接続済みですが、operation-only standard traitは別gateです。Phase 3の累積goal programでは
generalized constrained value schemeを回収します。scoped captureだけをその完了と数えません。

`project-schema-1/imported-constrained-function`はproviderのpublic `where Ready<T>`をfinal interfaceから
consumerへ運び、constraint argumentのnominal型とtraitをそれぞれcanonical identityへ解決します。consumerは
provider traitへlocal `Ready<Badge>` instanceを定義し、imported `describe value`へそのdictionaryを渡します。
TypedHir / CoreIrはselected local evidenceを保持し、生成ESMはproviderの
`describe(value)(dictionary)`を呼び、closed-project TypeScript checkとEffect executionが`imported ready`を観測します。
provider-local trait identityを綴りで再解決しないため、named importとnamespace-selected callableが同じresolver境界を
使います。evidence parameterはTypeScript backend内だけのerased dictionary record型であり、Seseragiのtrait選択を
TypeScriptへ委譲しません。

`project-schema-1/imported-instance-dispatch`はさらに、providerが定義したconcrete `Ready<Badge>` instanceをfinal interfaceへ
載せ、consumerの`describe Active`に必要なdictionaryとして選択します。`InterfaceInstance`はsource spellingとは別に
canonical `traitIdentity`とordered `argumentIdentities`を保持するため、同名traitやmulti-parameter traitをinstance identityの
文字列parseへ戻さず区別できます。TypedHir / CoreIrの`imported` evidenceをdriver output planがproviderのdictionary exportへ
結び、生成ESMは`__ssrg$instance$Ready$0`をvalue importして実際のmethod resultをConsoleへ出します。
このdirect concrete gate自体はgeneric imported instanceのsubstitution、constraint evidenceの再帰materialization、facade越しの
provider importを扱いません。続く`project-schema-1/transitive-instance-dispatch`はprovider / facade / mainを一つのclosed graphで
compileし、mainのsource edgeをfacadeだけに保ったまま、facade final interfaceが運ぶconcrete instance evidenceからoriginal
providerのdictionary exportを選びます。生成main ESMはfacadeのvalueとproviderのdictionaryを別々にimportし、executionが
`provider ready`を観測します。driver output planはentryから到達可能なprovider closureだけをbackendへ渡し、provider outputが
欠けた場合はlocal fallbackせずlowering errorになります。

`project-schema-1/imported-generic-instance-dispatch`はfinal interfaceのgeneric instance headをconsumerのconcrete
constraintへunifyし、ordered type argumentsをImported evidenceへ保存します。output planはconcrete specializationを
新しいinstance identityとして捏造せず、providerが公開したtemplate identityのdictionary exportをimportします。
生成ESMはそのexportへTypeScript type argumentsを適用してfactoryを呼び、constrained functionへ得られたdictionaryを
渡します。concrete evidenceは引き続きcanonical `argumentIdentities`で選択するため、facade越しnominal identityを
source spellingへ弱めません。

`project-schema-1/imported-constrained-instance-dispatch`はproviderのgeneric instance
`Inspect<Maybe<T>> where Ready<T>`をconsumerの`Inspect<Maybe<Int>>`へsubstituteし、final interfaceに保存した
constraintのcanonical `traitIdentity`から`Ready<Int>` evidenceを再帰materializeします。検索順は既存のscoped parameter、
local instance、imported instanceを共有し、selected nested evidenceをTypedHir / CoreIrへ保持します。生成ESMはproviderの
Inspect factoryとReady dictionaryをimportし、`Inspect<Int>(Ready<Int>)`を実際にdispatchします。required evidenceが
存在しないcaseは`SES-T0201 instance.missing`で停止します。これによりdirect providerのconstraint付きfactoryは接続済みです。
`project-schema-1/transitive-constrained-instance-dispatch`はmainのsource edgeをfacadeだけに保ちながら、facade final interfaceが
transportした同じgeneric / constraint付きinstance metadataからoriginal providerを選択します。生成main ESMはfacadeの
`report`とproviderのInspect factory / Ready dictionaryを別々にimportし、`Inspect<Int>(Ready<Int>)`を三moduleのclosed
TypeScript checkとactual executionまで通します。provider closureはconcrete instance専用ではなく、template identityとnested
evidenceをそのまま運びます。materializableなstandard `Show<Int>` / `Show<String>`はsingle-fileのlocal generic factoryへ接続済みです。
P2-4の残りはtrait namespaceとunresolved polymorphic constrained valueであり、operation-only standard traitを
dictionary化する場合は独立ABI gateとして扱います。

続くlocal package loader sliceでは、`seseragi.toml`の`layout.source`と`run.entry`からsource moduleを読み、
既存syntax frontendが返すraw import occurrenceだけを使ってrelative / `self/` importを再帰発見します。
package root、source root、各module fileはfilesystemでcanonicalizeし、root外へ向くsymlink、case違い、非NFC spelling、
複数logical moduleが同じphysical fileへ収束する状態をproject errorとして拒否します。読み込んだmoduleはcanonical pathを
source labelに、`PackageIdentity + ModuleRoot::Source + ModulePath`をstructural identityに保持します。package / `std/` /
`gen/` importはlocal fileへ誤変換せず、dependency resolver未接続として停止します。typed dependencyのmanifest-level解決と
entryから到達するcross-package source graphに加え、各path dependencyのentryから到達しないsourceも含むroot全体のidentity
auditを接続済みです。auditはsourceをcompileする代わりではなく、build / test discoveryが後から同じidentityを再利用できる
filesystem gateです。

`package.language`はsource discoveryより先に実装言語version `0.1.0`へ照合します。exact、比較、intersection、`||`、
caret、tildeを仕様の上限規則で評価し、prereleaseは同じmajor / minor / patchのprerelease comparatorがrangeへ明記された
場合だけ候補にします。不適合はsource fileが存在しなくても`SES-K0101`で停止するため、未対応syntaxを偶然parseして
package互換と誤認する経路はありません。

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
unreachable armを検査します。imported traitのmethod contract、dictionary生成・dispatch、constraint付きhigher-order callable、
generic imported ADTはそれぞれproject executionへ接続済みです。nested namespaceなどの独立gateが残るため、P2-2全体を
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
`schema-1/newtype-user-id`はこのinterface / resolver contractを一constructorのnominal valueとしてTypedHir / CoreIr /
TypeScriptIrへ接続します。constructor適用はtagged wrapperを生成し、constructor patternだけがpayloadを取り出します。
`semantic-diagnostics-schema-1/newtype-no-coercion`はnewtypeをrepresentation型へ暗黙変換しない境界を固定します。
generic `Tagged<A> = A`もpayloadから`A`を推論するlowering testを持ち、特定newtype名の分岐ではありません。
`project-schema-1/imported-newtype`は一つのnamed importからtype / constructor両namespaceを導入し、consumer側のconstructor適用、
pattern unwrap、type-only / value import、actual executionまで固定します。現在のbackendは観測可能なconstructor境界を保つtagged
representationを選びます。将来のerasure最適化は同じ意味を保つ場合だけ許され、alias化を型検査の代用にはしません。

structural Record valueは`schema-1/record-profile`で最初の実行可能な縦sliceへ接続します。parserは`{ name, score: value }`を
fieldごとの式として保持し、`.`を先に単一の名前へ結合しません。resolverはreceiverがmodule namespace bindingの場合だけ
`domain.member`をcanonical import選択へ渡し、それ以外はreceiver式だけを名前解決します。type checkerは後者をrecord
memberとしてrequired fieldを選び、extra fieldを許すwidth subtypingを適用します。TypedHir / CoreIr / TypeScriptIrは
record literalとrequired / optional field accessを独立nodeとして保持し、backendはreadonly object type / literal / indexed accessを生成します。
これによりnamespaceとmemberの判断をTypeScript shapeへ委譲せず、後続structでも同じfield backendを再利用できます。
optional field accessはreceiverを一度だけ評価し、`Object.prototype.hasOwnProperty`でpresenceを判定してからstandard
`Just` / `Nothing` runtime constructorへ接続します。したがってmissingとpresentな値を同じ`undefined`へ潰さず、result型も
`Maybe<A>`としてTypedHirから生成interfaceまで保持します。record itemはfield / spreadのsource-order列として全IRへ残し、
spread operandのfield型とpresenceを引き継いだ後、後続itemで同名fieldを上書きします。backendも同じ順序でreadonly object
spreadを生成するため、各operandを一度だけ評価します。required fieldのrecord patternも同じfixtureへ接続し、`{ name }`の
irrefutable bindingと`{ label: "Player", name }`のrefutable literal testをstructural subset matchとして扱います。未指定fieldは
無視し、resolverはnested patternだけを既存scopeへ解決し、type checkerはscrutineeのrecord field型を各child patternへ渡します。
TypedHirはfield名とnested patternを保持し、CoreIr / TypeScriptIrのdecision projectionはscrutineeを一度だけ評価した後の
named field accessを生成します。存在しないfieldは`SES-T0101`で停止します。optional query pattern (`{ id? }` / `{ id?: Just value }`)
は、missingを`Nothing`へ変換するpattern projection ABIが必要な独立gateとして残し、required patternへ暗黙統合しません。

nominal StructはRecord backendを再利用しますが、意味境界は共有しません。SurfaceAstは`Player { name, score }`と
`Player { ...player, score: 42 }`をStruct nodeとして保持し、resolverは`Player`をtype namespaceへ解決します。
type checkerは宣言ownerを持つ`SemanticTypeKey::Struct`を構築し、fieldの不足・余分・重複・型不一致、spread sourceの
owner一致を検査してからRecord型のfield objectへlowerします。Struct patternもownerとfield contractを先に検査し、
受理後だけ既存のnamed decision projectionを再利用します。この順序により、同じfield集合のRecordや別Structを
TypeScriptのstructural compatibilityへ委譲しません。

TypeScript backendは各Structへprivate unique-symbol brandを含むreadonly object typeを生成し、型検査済みのliteral /
updateだけをそのnominal型へassertします。非opaqueなpublic Structのinterface representationにはclosed field recordを残し、
consumerのsemantic catalogがconstruct / update / access / patternを同じ規則で検査できるようにします。opaque Structは
representationを引き続き隠します。`schema-1/struct-profile`、`semantic-diagnostics-schema-1/struct-field-errors`、
`project-schema-1/imported-struct`がlocal positive / negative / cross-module executionを分離して固定します。
generic Structは宣言fieldのsemantic templateとactual field / contextual result / spread ownerを既存generic callable substitutionへ
渡し、宣言parameterごとのargumentを集めます。これにより`Box { value: 41 }`は`Box<Int>`になり、nested nominal argumentも
同じ再帰置換を使います。複数fieldの矛盾はinstantiated field contractとの通常比較、未決定parameterは
`struct.type-arguments-unresolved`で停止し、TypeScript inferenceへ意味判断を委譲しません。`schema-1/generic-struct`と同名executionが
field inference、generic member / pattern、same-argument spread update、generated `Box<bigint>`を固定します。
明示type argument付きconstructionはSurfaceAstにimplicit constructionと区別できるoptional引数列を保持し、resolverが各引数を
type namespaceへ接続します。type checkerは明示引数を期待型とfield inferenceより優先し、個数不一致を
`struct.type-argument-arity-mismatch`、hole残存を`struct.type-arguments-unresolved`で停止します。
`schema-1/explicit-generic-struct`はnested `Marker<Array<String>>`が空Array fieldへcontextを渡し、TypedHirから生成TypeScriptの
`Marker<ReadonlyArray<string>>`、actual executionまで固定します。opaque smart constructorのmodule境界は別gateで固定済みで、
struct derivingは後続sliceです。

公開interfaceだけでは、同一packageの「private宣言は存在する」と単なるtypoを区別できません。そのためfrontendは
同じSurfaceAst passから、bodyと型schemeを含めずdeclaration name / namespace / visibility / canonical symbolだけを持つ
`ModuleHeader`も生成します。private宣言と、公開opaque ADT / newtypeのhidden constructorもheaderへ残し、current moduleの
`LinkedModule`まで保持します。これは公開interface artifactやdependency cacheとして配布しません。同一packageの
`ModuleLinkTarget`だけがheaderを持ち、private nameを`PrivateExport`、存在しないnameを`MissingExport`へ分けます。外部package
targetは公開interfaceだけを持ち、private declarationの存在を漏らしません。これらを`SES-N0102` / `SES-N0104`へ変換する
project diagnostic adapterはまだ未実装です。

shared `compile_project`はmodule IDの文字列を分解してpackage境界を推測しません。project adapterが
`ProjectModuleInput::with_package_scope`でopaqueなvisibility scopeを渡し、同じscopeのedgeだけを
`ModuleLinkTarget::same_package`、異なるscopeまたは片側だけ未指定のedgeをexternal targetにします。従来のscope未指定
single-package input同士は互換のためsame-packageとして扱います。これによりcross-package compiler adapterを追加しても、
dependencyのprivate headerがimporterへ漏れません。

`ModuleLinkTarget::same_package`はheaderの全public nameがfinal interfaceに存在することを検証します。したがってcompact
inferred effectのexportが欠けるshallow interfaceを誤ってlink inputへ使うと、semantic linking前に失敗します。
current module headerのnameもlink scopeへ先に予約するため、同じnamespaceのlocal declarationとimportが衝突した状態で
dependency schemeをlocal SymbolIdへ誤結合する経路はありません。

`resolve_linked_module`はlinkerが確定したnamed / alias / operator importをmodule scopeへ登録し、canonical dependency
symbolと完全な`InterfaceExport` schemeを`ResolvedImport`へ保持します。source itemを再解釈してcanonical IDを作らず、
同じnamed importからtype / value namespaceへ入るnewtypeも別symbolとして保持できます。現時点ではこのcontractを
TypedResolutionのpure callableへ接続し、alias経由のimported function callを型検査してcanonical calleeを
TypedHirへ残せます。公開ADTはtype parameterを含めてdependency interfaceの全constructorからsemantic familyを構築し、
importしていないvariantもscopeへ名前登録せずexhaustiveness witnessには残します。これにより選択importだけでmatchを
誤ってtotalと判定しません。generic constructor payloadはconsumerのconcrete owner argumentsで再帰置換します。

unconstrained rank-1 generic pure functionはlocal / named importの同じcallable pathへ接続済みです。scheme parameterを
semantic type identityとして保持するため、`Int`や`String`だけでなくuser-defined ADTでもcallごとに独立して具体化されます。
型引数binderはTypedHirからCoreIr、TypeScriptIr、generated functionの最外arrowまで保持し、分割RPS projectはdependencyの
generic functionを異なる型で利用したgenerated module setをTypeScript type-checkして実行します。constraint付きfunction、
higher-order parameter、generic imported ADTも独立fixtureで接続済みです。higher-order resultとnested namespaceは個別gateです。

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
namespace-qualified value call / type reference / constructor expression / constructor patternは接続済みです。direct dependencyと
facade越しに選択したconcrete imported evidenceはdictionary export metadataとTypeScript source importまで接続済みです。
direct / facade越しproviderのgeneric / constraint付きimported factoryもdictionary export metadata、nested evidence、
TypeScript source importまで接続済みです。materializableなstandard `Show<Int>` / `Show<String>`もlocal factory引数へ接続済みです。
一方、trait namespaceとgeneralized constrained value schemeは未接続のため、
P2-4全体の完了とは扱いません。projectが選んだPOSIX形式の生成先pathから
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
`project-schema-1/imported-effect-console`はimported effect callを全stageへ固定します。
`project-schema-1/imported-constrained-function`はpublic callableのconstraint identityとconsumer-local dictionaryを
module境界越しに運び、generated ESMのclosed type-checkと実行まで固定します。`imported-instance-dispatch`と
`transitive-instance-dispatch`はprovider dictionary selectionをdirect / facade経路で分離検証します。

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
compositionを未検証にする経路はありません。strict `project.json`から始まるP2-5 execution gateに加え、local Package CLIは
同じsourceへ置いた`seseragi.toml`からentryとmodule graphを発見し、shared driverでcompileした全generated moduleを
`seseragi-runtime`がplanned pathへstageします。CLI integration testはfixture descriptorなしの`seseragi run .`で正常入力と
typed failure / exit 1を比較します。dependency manifest contractはtyped modelへ接続済みですが、dependency package graphは
引き続き未実装です。

local driver adapterがcompilerへ渡すopaque IDは、現在の閉じた一package graphでは`<package name>::<module path>`です。
これはstructural `PackageIdentity`の一般serialization規則ではありません。同名・同version・別sourceの混在を扱う
dependency package gateでは、project resolverがconfusion check後のstructural identityからcollision-freeなIDを割り当て、
CLIやdriverがpackage identityの文法を再実装しないことを完了条件にします。driverは引き続きIDをopaqueに扱うため、
この回収でresolver / TypedHir / CoreIrの表現を作り直す必要はありません。

`project-schema-1/imported-effect-failure`は、dependencyの`InputError deriving Show`とcompact `reject` Effectをmainがimportし、
local `AppError`へ`mapError`する組み合わせを固定します。generated mainはdriver output planが渡したexact dictionary exportを
source importし、staged executionはnested typed failure、derived stderr、process exit 1を比較します。これによりdirect
dependency evidenceはsnapshotだけでなくruntimeまで証明されます。このfixture自体はdirect dependencyを対象にし、
concrete transitive provider chainは`transitive-instance-dispatch`が別に固定します。

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

collectionの小さい縦sliceとして、`Range<Int>` literalとstandard `Reducible<Range<Int>, Int>`を
SurfaceAst、TypedHir、CoreIr、TypeScriptIr、versioned runtime ABI、Node / browser executionへ接続しました。
RangeはArrayへlowerせず、exclusive / inclusive境界を保持します。comprehensionもRange専用loopへ展開せず、
仕様3.9の`Iterable<C, A>` evidenceをTypedHir / CoreIrへ運びます。最初のgateとしてArray / Range source、
binding / wildcard generator、guard、複数generatorを通常pipelineへ接続しました。backendは単一generatorを
`collectMap`、後続generatorを持つ節を`collectFlatMap`へlowerし、source collectionとArray resultを混同しません。
`schema-1/range-comprehension`はRangeからeven squareを生成してArray `reduce`へ接続し、Node / browserで実行します。
`schema-1/array-comprehension`はArray同士のnested generatorを固定します。
`schema-1/comprehension-pattern-filter`はmatchと共有するTypeScript decision表現へconstructor / tuple patternをlowerし、
predicateで構造照合とguardを評価してから、transformで同じprojectionからbindingを復元します。runtime helperは
predicateが通った要素にだけtransformを呼ぶため、pattern不一致は仕様どおりfilterになり、暗黙の`MonadFail`や例外へ
変換しません。binding / wildcardだけの既存caseは直接lambdaの軽い形を維持し、local / imported
user-defined Iterable dictionaryも同じCore shapeから呼び出します。

その前提となる仕様10.7の`Iterator<A>`は、standard opaque external nominal typeと`unfold` / `next` callableとして
SurfaceAst以降の通常pipelineへ接続しました。runtime表現はimmutableな`next` closureだけを公開し、`unfold`は構築時に
stepを評価せず、`next`ごとに一度だけ評価して新しいrest Iteratorを返します。`schema-1/iterator-unfold`と同名execution
fixtureはnested `Maybe<(A, Iterator<A>)>`内のgeneric substitution、runtime type import、persistent observationを固定します。
conformanceのruntime package probeも遅延性と非破壊性を直接検査するため、user-defined IterableをArray / Rangeの
runtime shapeへ寄せず、dictionaryの`iterate`結果をこの共通pull境界へ流せます。

このgateは`schema-1/user-iterable-comprehension`で接続済みです。comprehension typingはsource collectionだけから
local `Iterable<C, A>` headを照合して`A`を推論し、generic関数では`where Iterable<C, A>`のscoped parameter evidenceを
優先します。imported headはprovider側のinterface typeをconsumerのcanonical type bindingへlocalizeしてから照合します。
backendはstandard Array / Range evidenceだけ既存collectorへ直接送り、それ以外は選択済みLocal / Imported / Parameter
dictionaryの`iterate`を呼び、`Iterator<A>`用`collectMap` / `collectFlatMap`へ渡します。
`project-schema-1/imported-iterable-comprehension`はdictionary provider import、type-only Iterator import、複数moduleの
runtime requirement closureとNode executionを固定します。これにより標準collection名のhardcodeだけでgreenになる経路を
排除しました。

同じfixtureはuser-defined `Reducible<Countdown, Int>`とgeneric `total<C> where Reducible<C, Int>`も固定します。
generic bodyのprelude `reduce`はscoped parameter dictionaryのmethodへlowerし、具体callはlocalまたはimported provider
dictionaryを渡します。custom dictionary method自身はfiniteなstandard Range reduceを組み合わせるため、標準operation ABIと
user dictionary ABIを一つのexecutionで検査します。`semantic-diagnostics-schema-1/reducible-missing`は対応instanceのない
concrete collectionを`instance.missing`で拒否します。standard Reducibleそのものをfirst-class dictionaryとして渡すABIは
引き続き独立gateであり、このsliceはuser evidence dispatchと混同しません。

同じcollection roadmapの次のsliceとして、仕様3.8のpersistent `List<A>` literalを
`schema-1/list-literal`でTokenStreamからgenerated TypeScriptまで接続しました。`` `[a, b] ``はlexerでtemplate literalと
区別し、Arrayと共有するhomogeneous collection typingを通しますが、backendでは`ReadonlyArray<A>`へ同一化しません。
`core.list.from-array` runtime ABIがimmutableな`Empty / Cons` chainを構築し、Listを明示するparameter annotationでは
`@seseragi/runtime/list`のnominal type importを使用します。

`schema-1/list-comprehension`では、standard `Iterable<List<A>, A>` / `Reducible<List<A>, A>`を既存のordered
trait argumentとselected evidenceへ合流させました。backtick内包表記はArray / Rangeと同じclause typing、pattern filter、
nested collector loweringを再利用し、最終collectorだけを`fromArray`でpersistent Listへ変換します。List sourceの走査は
immutable nodeを消費せず、generic `reduce`も選択済み`std/list::Reducible`からList runtimeへ接続します。同名execution
fixtureとPlayground sampleは`` `[1, 2, 3, 4, 5] ``からodd squareのListを作り、合計`35`をactual executionで固定します。

同じbacktick lexer境界のtemplate branchは、token textをbackendまで不透明な文字列として運ばず、textと
`${expression}`をSurfaceAstで分離します。interpolation内は既存expression parser / resolverを再利用し、独立した小型parserを
増やしません。各式は通常の型付け後に`Show<A>` constraintを解決し、selected standard / local / imported / parameter evidenceと
canonical trait identityをTypedHir / CoreIrへ保持します。TypeScript loweringはevidenceが指すdictionaryの`show` callとtextを
左結合のString連結へ展開するため、式はsource順に一度だけ評価されます。`schema-1/template-interpolation`と同名execution fixtureは
runtime `Show<String>`とlocal derived `Show<Badge>`の混在をactual outputまで固定します。`schema-1/struct-profile`は
runtime `Show<Int>`をStruct member / pattern bindingのtemplate interpolationから選択してactual executionし、Playgroundも
同じsourceをbundleします。
missing evidenceは`instance.missing`でbackend前に停止し、不正escapeはbackslashからescape末尾までを示す`SES-P0201`として
黙って空文字へ変換しません。

同fixtureのinterpolation parameterは仕様1.1のUnicode XIDとidentifier末尾`'`を組み合わせ、template scannerが`'`を
Char開始へ再解釈しないことも固定します。lexerはXID_Start / XID_Continueを直接検査し、TypeScript境界ではUnicodeを
そのまま保持します。JavaScript identifierに置けない`'`だけは、Seseragi sourceでは書けない`$`を使った`$prime`へ
符号化するため、`value'`と`value_`を同じbackend名へ潰しません。Playgroundのstream highlighterもtemplateをtext chunkと
interpolation expressionへ分け、`${...}`内を通常のSeseragi token規則で着色します。

現時点のderived `Show`は、local非generic ADTと限られたpayload evidenceを扱う閉じたsliceです。shallow
`ModuleInterface`の`InterfaceInstance`はidentityなしを許し、final `TypedInterface`だけがcanonical trait identityとordered head
argument identitiesからsemantic identityを確定します。reachable dependencyのinstanceはResolvedAstに保持され、derived `Show`の
payloadだけでなくconcrete user-defined trait constraintもLocal / Imported / Standard evidenceを区別します。選択結果は
TypedHir、CoreIr、TypeScriptのdictionary source importとdriver output planまで残ります。
concrete evidenceはdirect dependencyとfacade越しのprovider chainを、generic evidenceはdirect providerからの
factory具体化をexecutionまで通します。generic / constraint付きfactoryもsubstitutionと再帰evidence materializationを
direct / facade越しproviderで接続済みですが、imported instance solver全体の完了とは呼びません。P2-4の完了には
trait namespaceとunresolved polymorphic constrained valueが必要です。standard evidenceはruntime dictionaryが実在する
`Show<Int>` / `Show<String>`をfactory引数へ接続し、operation-only traitをmaterialize済みとは数えません。

project resolverはpackage identityの文法をdriverへ再実装しません。driverのmodule IDはopaqueな入力とし、NFC、root tag、
dependency export map、symlink / case衝突はP2-1の唯一の所有者が決めます。これによりmodule graph追加時にAST、resolver、
TypedHir、CoreIr、runtime ABIを一斉に作り直す経路を避けます。

# Seseragi 現在地

この文書は、新しいSeseragi仕様が「何を決め、どこまで検証され、何がまだ動かないか」を
一か所で確認するための非規範な進捗表です。言語の意味は [`spec/`](./spec/README.md) を、
機能ごとの検証対象は [`../examples/spec/COVERAGE.md`](../examples/spec/COVERAGE.md) を正とします。

## 現在の結論

新しいSeseragiは、言語と標準ライブラリの**仕様初稿がおおむね揃い、Rust再実装を縦sliceで進めている段階**です。
compiler全体は未完成ですが、lexer / SurfaceAst / resolver / typed HIR / CoreIr / TypeScriptIr / emitterと
versioned runtimeの最小経路は実際に動作しています。

lessonやfixture全体が実装済みという意味ではありません。実装済み範囲はRust conformance artifactを正規生成・比較し、
未接続のlesson / project fixtureは引き続き「新実装が満たすべき契約」です。

## 状態の意味

| 状態                | 意味                                                  |
| ------------------- | ----------------------------------------------------- |
| 仕様化済み          | `docs/spec/` に構文、型、評価、失敗、境界の規則がある |
| design exampleあり  | 学習用programまたは最小fixtureがある                  |
| fixture契約あり     | 入力と期待結果の形式が機械検証可能な形で置かれている  |
| conformance検証済み | 新仕様compilerを実際に通し、期待結果と比較できる      |
| 実装済み            | compiler、runtimeまたはtoolingが新仕様どおりに動く    |

`bun run check:spec-examples` が保証するのは、lessonの番号・前提・日本語説明・期待stdoutと、
fixture sidecarの形式・仕様節参照・diagnostic spanの整合です。`bun run conformance:artifacts` は
artifact bundleの発見、必須file、JSON envelope、参照snapshotの整合だけを検査します。Seseragi sourceの
parse、型検査、lowering、生成物比較はRust側の`cargo run -p seseragi-conformance -- .`が担当し、
Effectおよびpure execution fixtureについては生成moduleとversioned runtimeを実行します。

## 領域別の現在地

| 領域                                       | 仕様          | example / fixture                          | 新仕様実装         |
| ------------------------------------------ | ------------- | ------------------------------------------ | ------------------ |
| 基本文法、演算子、pattern                  | 初稿あり      | lessonあり、fixtureは一部                  | tuple / matchまで部分実装 |
| 型、generic、ADT、struct、record           | 初稿あり      | lessonあり、fixtureは一部                  | ADT / standard sum / rank-1 generic fnまで部分実装 |
| trait、Functor、Applicative、Monad、Monoid | 初稿あり      | lessonあり、law fixture不足                | 未着手             |
| custom infix operator                      | 初稿あり      | compile fixtureあり                        | 未着手             |
| Effect、resource、concurrency              | 初稿あり      | lesson、時間制御・cleanup fixtureあり      | Console / Stdin + imported non-generic Effect call / positive project executionまで部分実装 |
| Signal、Stream                             | 初稿あり      | lessonあり、runtime fixture不足            | 未着手             |
| module、package、project                   | 初稿あり      | module graph・lock・manifest fixtureあり   | linked rank-1 / namespace value・type・constructor / direct Show dictionary import + Effect project executionまで部分実装 |
| TypeScript interop、`.d.ts`変換            | 初稿あり      | load・ABI・変換snapshot fixtureあり        | 未着手             |
| collection、text、number、JSON             | 初稿あり      | lessonあり、境界fixture不足                | 未着手             |
| Bytes、Decimal、Regex、timezone            | 初稿あり      | lessonあり、fixture不足                    | 未着手             |
| filesystem、process、HTTP                  | 初稿あり      | cleanup・shutdown・body stream fixtureあり | 未着手             |
| pure HTML、SSR、DOM、hydration             | 初稿あり      | SSR lesson、DOM / hydration fixtureあり    | 未着手             |
| 性能モデル、最適化境界                     | 初稿あり      | shape・profile差分・stack fixtureあり      | 未着手             |
| diagnostics、formatter、LSP                | 契約初稿あり  | diagnostic fixtureは一部                   | frontend diagnosticを部分実装 |
| syntax highlight                           | token契約あり | spec preview拡張あり                       | 仮実装あり         |
| playground                                 | 共有契約あり  | 新仕様sourceあり                           | 新仕様対応は未着手 |

ここでいう「未着手」は刷新仕様に対する状態です。repositoryに存在する現行compilerやruntimeのcodeを
削除した、または何も実装されていない、という意味ではありません。現行実装は設計資料として残って
いますが、新仕様への適合をまだ主張しません。

## いま存在する成果物

- 規範仕様: `docs/spec/00-language.md` から `14-performance.md` とAppendix grammar
- 学習教材: `examples/spec/lessons/` の31 lesson
- versioned conformance入力: positive / diagnostic / runtime artifactとproject fixture
- coverage表: `docs/SPEC_COVERAGE.md` と `examples/spec/COVERAGE.md`
- grammar対応表: `examples/spec/grammar-coverage.json`
- token / stage / execution artifact契約: `examples/spec/artifacts/token-schema-1/`、
  `schema-1/`、`stage-schema-1/`、`execution-schema-1/`
- 横断監査記録: `docs/SPEC_AUDIT.md`
- 構造checker: `scripts/check-spec-examples.ts`
- artifact runner skeleton: `scripts/conformance-artifacts.ts`
- Rust compiler crates: syntax、source、project identity、diagnostic、semantics、lowering、single-module driver、runtime / conformance boundary
- TokenStreamからgenerated TypeScriptまでのschema-1 artifact producer / conformance比較
- cold Effect valueをversioned TypeScript runtimeで実行するConsole / Stdin execution fixture
- typed ADT、tuple、match、exhaustivenessを通す`rock-paper-scissors-domain` artifact
- standard `Maybe` / `Either`と、正常・不正入力を実行する`parse-hand-either` artifact
- pure `parseHand`をcold typed Effectへ変換し、success payloadを実行比較する`effect-parse-hand` artifact
- ADT / match / typed failure / Stdin / Consoleを統合し、正常・不正・EOFを実行比較する
  `rock-paper-scissors-cli` artifact
- 同じCLIでConsole operation trace、Stdin / Console host failureのtyped変換、derived `Show`によるstderrと
  exit code 1まで比較するexecution artifact
- cross-module ADT / pure callを実行する`project-schema-1/rock-paper-scissors-domain-split`、namespace constructorと
  generic callを実行する`namespace-generic-call`、imported cold EffectとConsole traceを実行する`imported-effect-console`
- 物理source pathと論理module identityを分離し、Phase 1の累積programをTokenStreamからgenerated
  TypeScriptまで一つのcompile結果として返すpublic Rust driver
- 表示確認用syntax highlight: `extensions/seseragi-spec-preview/`

数は進捗の目安にすぎません。lessonが存在しても、対応するpositive / negative / runtime fixtureが
揃うまでは、その機能をconformance検証済みとは扱いません。

## まだ終わっていないこと

### 1. 仕様の横断監査

章ごとの初稿はありますが、同じ概念を複数章から参照する箇所について、型signature、failure、
resource lifetime、grammar、exampleの最終突合が必要です。最初の型構文・性能境界passは完了し、結果を
`docs/SPEC_AUDIT.md`へ記録しています。新機能を増やすより先に残りの矛盾を減らします。

### 2. conformance fixtureの拡充

single-file / project fixtureは継続的に拡充中です。特に次が不足しています。

- operator precedence、型推論、kind、coherenceのpositive / negative case
- Signal transaction、Stream backpressureのruntime trace
- DOM reconciliation、event cleanup、hydration mismatchのproject fixture
- trait specialization、fusionのquality profile fixture
- formatter、LSP、semantic token、playgroundで共有するsnapshot

### 3. 新仕様compilerの実装

実装計画は`IMPLEMENTATION.md`を正本とし、lexer / lossless CST、parser、名前解決、型検査、
意味を単純化した内部表現、TypeScript出力、runtime adapterを分離して進めています。
TypeScriptは出力先であり、Seseragiの型やEffectの意味を決める正本にはしません。

### 4. productとしてのtooling

新frontendをcompiler、formatter、LSP、syntax highlight、playgroundで共有し、同じsourceが全surfaceで
同じ構文とdiagnosticになる状態は未達です。

### 5. 一般機能の未定義項目

standard API監査で再発見した`BigInt`の公開surface未定義は、`std/big-int`のmodule API、arithmetic
semantics、TypeScript境界、cost contract、lesson、compile fixtureへ移して解決しました。現時点で再登録中の
未定義項目はありません。

## 次の進行順

Phase 1のsingle-file累積programは完了gateを満たしました。次は同じprogramを捨てず、module / packageの
一般機構を加える順で進めます。

1. project resolverがpackage rootとsource rootからcanonical module identityを決め、driverへ物理pathとは別に渡す。
2. project layerが`ModuleGraph`のtopological orderでdependency interfaceをcompileし、generated output pathから
   `TypeScriptOutputPlan`を構築する。driverのlinked compile APIと、projectのentry / dependency output pathからimporter相対
   specifierへ変換するhelper、閉じたgraphをcompileする`compile_project`、backend側のalias、同名export、type-only edge、source
   map contractは固定済み。graphは実cycle witnessを返し、driverはgraph/source edge不一致、extra input、global output path衝突を
   拒否する。`project-schema-1`のconformance/writerで分割RPS domainを全IR・生成artifactまで固定し、planned output pathへ
   stageしたmodule setのTypeScript type-checkも行う。pure entryと単一positiveのimported Effect / Console entryはBunで実行済み。
   filesystem discovery、manifest解決、複数execution caseを持つpackage executionは未実装。
3. じゃんけんCLIをdomain / input / mainへ分割し、single-file版と同じtyped failure、Effect、正常・不正・EOF / host failureの
   execution結果を保つ。
4. direct dependencyのderived `Show` evidenceはcanonical type identityでResolvedAstからTypedHir / CoreIr / TypeScript source
   import / driver output planまで保持済み。次はtransitive provider chainを含む実行gateでinstance closureを完成させる。
5. trait / nested namespace、higher-order callable、generic imported ADTは、それぞれ一般機構を証明する独立gateで回収する。

namespace-qualified constructor expression / patternとimported ADT exhaustivenessは、小さいsemantics / lowering fixtureと
`project-schema-1/namespace-generic-call`の実行経路まで接続済みです。このため次の累積goalではnamespace機能を増やすこと自体を
目的化せず、Phase 1のCLIを複数moduleへ移したときに必要になるEffect contractとinstance closureを優先します。

## 完了と呼ぶ条件

言語機能を「完了」と呼べるのは、次をすべて満たしたときです。

- 規範仕様とAppendix grammarが一致する。
- 学習用exampleがあり、前提順に読める。
- 必要なpositive / negative / runtime fixtureがある。
- 新仕様compilerがfixtureを実際に検証する。
- formatter、LSP、highlight、playgroundが同じfrontendを使う。
- host resourceやinteropを含む場合、cleanupとfailureが検証される。

この条件を満たす前は、「仕様化済み」「fixtureあり」のように到達点を限定して表現します。

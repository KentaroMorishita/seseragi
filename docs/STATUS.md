# Seseragi 現在地

この文書は、新しいSeseragi仕様が「何を決め、どこまで検証され、何がまだ動かないか」を
一か所で確認するための非規範な進捗表です。言語の意味は [`spec/`](./spec/README.md) を、
機能ごとの検証対象は [`../examples/spec/COVERAGE.md`](../examples/spec/COVERAGE.md) を正とします。

## 現在の結論

新しいSeseragiは、言語と標準ライブラリの**仕様初稿がおおむね揃った段階**です。
一方、その仕様へ適合するparser、type checker、runtime、TypeScript emitterはまだ完成していません。

したがって、現在のlessonやfixtureは「新実装が満たすべき契約」であり、現行compilerで実行できる
ことを示すものではありません。

## 状態の意味

| 状態                | 意味                                                  |
| ------------------- | ----------------------------------------------------- |
| 仕様化済み          | `docs/spec/` に構文、型、評価、失敗、境界の規則がある |
| design exampleあり  | 学習用programまたは最小fixtureがある                  |
| fixture契約あり     | 入力と期待結果の形式が機械検証可能な形で置かれている  |
| conformance検証済み | 新仕様compilerを実際に通し、期待結果と比較できる      |
| 実装済み            | compiler、runtimeまたはtoolingが新仕様どおりに動く    |

`bun run check:spec-examples` が現在保証するのは、lessonの番号・前提・日本語説明・期待stdoutと、
fixture sidecarの形式・仕様節参照・diagnostic spanの整合です。`bun run conformance:artifacts` は
artifact bundleの発見、必須file、JSON envelope、参照snapshotの整合だけを検査します。Seseragi sourceの
parse、型検査、実行はまだ保証しません。

## 領域別の現在地

| 領域                                       | 仕様          | example / fixture                          | 新仕様実装         |
| ------------------------------------------ | ------------- | ------------------------------------------ | ------------------ |
| 基本文法、演算子、pattern                  | 初稿あり      | lessonあり、fixtureは一部                  | 未着手             |
| 型、generic、ADT、struct、record           | 初稿あり      | lessonあり、fixtureは一部                  | 未着手             |
| trait、Functor、Applicative、Monad、Monoid | 初稿あり      | lessonあり、law fixture不足                | 未着手             |
| custom infix operator                      | 初稿あり      | compile fixtureあり                        | 未着手             |
| Effect、resource、concurrency              | 初稿あり      | lesson、時間制御・cleanup fixtureあり      | 未着手             |
| Signal、Stream                             | 初稿あり      | lessonあり、runtime fixture不足            | 未着手             |
| module、package、project                   | 初稿あり      | module graph・lock・manifest fixtureあり   | 未着手             |
| TypeScript interop、`.d.ts`変換            | 初稿あり      | load・ABI・変換snapshot fixtureあり        | 未着手             |
| collection、text、number、JSON             | 初稿あり      | lessonあり、境界fixture不足                | 未着手             |
| Bytes、Decimal、Regex、timezone            | 初稿あり      | lessonあり、fixture不足                    | 未着手             |
| filesystem、process、HTTP                  | 初稿あり      | cleanup・shutdown・body stream fixtureあり | 未着手             |
| pure HTML、SSR、DOM、hydration             | 初稿あり      | SSR lesson、DOM / hydration fixtureあり    | 未着手             |
| 性能モデル、最適化境界                     | 初稿あり      | shape・profile差分・stack fixtureあり      | 未着手             |
| diagnostics、formatter、LSP                | 契約初稿あり  | diagnostic fixtureは一部                   | 未着手             |
| syntax highlight                           | token契約あり | spec preview拡張あり                       | 仮実装あり         |
| playground                                 | 共有契約あり  | 新仕様sourceあり                           | 新仕様対応は未着手 |

ここでいう「未着手」は刷新仕様に対する状態です。repositoryに存在する現行compilerやruntimeのcodeを
削除した、または何も実装されていない、という意味ではありません。現行実装は設計資料として残って
いますが、新仕様への適合をまだ主張しません。

## いま存在する成果物

- 規範仕様: `docs/spec/00-language.md` から `14-performance.md` とAppendix grammar
- 学習教材: `examples/spec/lessons/` の31 lesson
- conformance入力: positive 22件、diagnostic 9件
- multi-file conformance入力: project 41件
- coverage表: `docs/SPEC_COVERAGE.md` と `examples/spec/COVERAGE.md`
- grammar対応表: `examples/spec/grammar-coverage.json`
- stage / execution artifact契約: `examples/spec/artifacts/schema-1/`、
  `stage-schema-1/`、`execution-schema-1/`
- 横断監査記録: `docs/SPEC_AUDIT.md`
- 構造checker: `scripts/check-spec-examples.ts`
- artifact runner skeleton: `scripts/conformance-artifacts.ts`
- 表示確認用syntax highlight: `extensions/seseragi-spec-preview/`

数は進捗の目安にすぎません。lessonが存在しても、対応するpositive / negative / runtime fixtureが
揃うまでは、その機能をconformance検証済みとは扱いません。

## まだ終わっていないこと

### 1. 仕様の横断監査

章ごとの初稿はありますが、同じ概念を複数章から参照する箇所について、型signature、failure、
resource lifetime、grammar、exampleの最終突合が必要です。最初の型構文・性能境界passは完了し、結果を
`docs/SPEC_AUDIT.md`へ記録しています。新機能を増やすより先に残りの矛盾を減らします。

### 2. conformance fixtureの拡充

現在はsingle-file 31件、project 41件です。特に次が不足しています。

- operator precedence、型推論、kind、coherenceのpositive / negative case
- Signal transaction、Stream backpressureのruntime trace
- DOM reconciliation、event cleanup、hydration mismatchのproject fixture
- trait specialization、fusionのquality profile fixture
- formatter、LSP、semantic token、playgroundで共有するsnapshot

### 3. 新仕様compilerの実装

実装計画自体はまだ正本化していません。少なくとも、lexer / lossless CST、parser、名前解決、型検査、
意味を単純化した内部表現、TypeScript出力、runtime adapterを分離して進める必要があります。
TypeScriptは出力先であり、Seseragiの型やEffectの意味を決める正本にはしません。

### 4. productとしてのtooling

新frontendをcompiler、formatter、LSP、syntax highlight、playgroundで共有し、同じsourceが全surfaceで
同じ構文とdiagnosticになる状態は未達です。

### 5. 一般機能の未定義項目

standard API監査で再発見した`BigInt`の公開surface未定義は、`std/big-int`のmodule API、arithmetic
semantics、TypeScript境界、cost contract、lesson、compile fixtureへ移して解決しました。現時点で再登録中の
未定義項目はありません。

## 次の進行順

当面は仕様機能を横へ増やさず、次の順で進めます。

1. grammar productionへ不足するdiagnostic / formatter targetを追加する。
2. `IMPLEMENTATION.md`のWave 0としてconformance runnerとfrontend contractを実装する。
3. parserから段階的に実装し、formatter、LSP、highlight、playgroundへ同じfrontendを接続する。

## 完了と呼ぶ条件

言語機能を「完了」と呼べるのは、次をすべて満たしたときです。

- 規範仕様とAppendix grammarが一致する。
- 学習用exampleがあり、前提順に読める。
- 必要なpositive / negative / runtime fixtureがある。
- 新仕様compilerがfixtureを実際に検証する。
- formatter、LSP、highlight、playgroundが同じfrontendを使う。
- host resourceやinteropを含む場合、cleanupとfailureが検証される。

この条件を満たす前は、「仕様化済み」「fixtureあり」のように到達点を限定して表現します。

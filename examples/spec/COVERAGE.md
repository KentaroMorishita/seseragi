# Language feature coverage

この表は `docs/spec/` の規範的機能を、学習用lessonまたは機械検証fixtureへ結びつけます。
`covered` は対象sourceが存在する状態、`planned` は必要なtargetを特定したが未作成の状態です。
言語実装のconformance完了条件は、required行がすべてcoveredになり、positive / negativeの両方が必要な
機能でdiagnostic fixtureも通ることです。

| 仕様領域                                    | 学習用target            | 機械検証target                            | 状態    |
| ------------------------------------------- | ----------------------- | ----------------------------------------- | ------- |
| program、entry point、Effect main           | Lessons 01-10           | entry signature diagnostics               | partial |
| literal、application、`$`、pipeline         | Lessons 01-02           | precedence fixtures                       | partial |
| match、tuple、range、effectful for          | Lesson 01               | exhaustiveness diagnostics                | partial |
| newtype、deriving、operator overload        | Lesson 03               | coherence diagnostics                     | partial |
| Array/List、lambda、Either                  | Lesson 04               | collection boundary fixtures              | partial |
| ApplicativeとValidation accumulation        | Lesson 05               | Applicative law fixtures                  | partial |
| Monad、`>>=`、do desugar                    | Lesson 06               | Monad law / invalid bind pattern fixtures | partial |
| Signalとtransaction                         | Lesson 07               | glitch / subscription fixtures            | partial |
| Fiber、Deferred、cancellation               | Lesson 08               | scheduler / cancellation fixtures         | partial |
| Streamとbackpressure                        | Lesson 09               | merge / overflow fixtures                 | partial |
| resource scopeとfinalizer                   | Lesson 10               | exit / defect ordering fixtures           | partial |
| generic ADT / struct / alias / impl         | planned Lesson 11       | kind / inference diagnostics              | planned |
| record、spread、comprehension、全pattern    | planned Lesson 12       | parse / pattern diagnostics               | planned |
| custom trait / constraint / custom operator | planned Lesson 13       | orphan / fixity diagnostics               | planned |
| Semigroup / Monoid / transformer            | planned Lesson 14       | law fixtures                              | planned |
| retry / repeat / timeout                    | planned advanced lesson | deterministic Clock fixtures              | planned |
| module、visibility、re-export、cycle        | module guide            | `fixtures/projects/modules-*`             | planned |
| TypeScript foreign blockとABI               | interop guide           | `fixtures/projects/foreign-*`             | planned |
| `.d.ts`変換                                 | converter guide         | input / generated snapshot projects       | planned |
| manifest、dependency、lockfile              | package guide           | `fixtures/projects/package-*`             | planned |
| parser recovery、formatter、LSP、highlight  | lessons全体             | tooling snapshots                         | partial |

## 次に埋める順序

1. Lesson 11-14でcore languageと抽象化のpositive pathを完成させる。
2. 各Lessonに対応するcompile / diagnostic fixtureを追加する。
3. module、interop、packageをmulti-file project fixtureで固定する。
4. 全sourceをformatter、LSP、syntax preview、playgroundが共有するrunnerを作る。

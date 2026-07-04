# Language feature coverage

この表は `docs/spec/` の規範的機能を、学習用lessonまたは機械検証fixtureへ結びつけます。
`covered` は対象sourceが存在する状態、`planned` は必要なtargetを特定したが未作成の状態です。
言語実装のconformance完了条件は、required行がすべてcoveredになり、positive / negativeの両方が必要な
機能でdiagnostic fixtureも通ることです。

| 仕様領域                                    | 学習用target            | 機械検証target                            | 状態    |
| ------------------------------------------- | ----------------------- | ----------------------------------------- | ------- |
| program、entry point、Effect main           | Lessons 01-18           | entry signature diagnostics               | partial |
| literal、application、`$`、pipeline         | Lessons 01-05           | precedence fixtures                       | partial |
| match、tuple、range、effectful for          | Lessons 03 and 05       | exhaustiveness diagnostics                | partial |
| newtype、deriving、operator overload        | Lesson 07               | coherence diagnostics                     | partial |
| Array/List、lambda、Either                  | Lessons 04 and 08       | collection boundary fixtures              | partial |
| ApplicativeとValidation accumulation        | Lesson 09               | Applicative law fixtures                  | partial |
| Monad、`>>=`、do desugar                    | Lesson 10               | Monad law / invalid bind pattern fixtures | partial |
| Signalとtransaction                         | Lesson 15               | glitch / subscription fixtures            | partial |
| Fiber、Deferred、cancellation               | Lesson 16               | scheduler / cancellation fixtures         | partial |
| Streamとbackpressure                        | Lesson 17               | merge / overflow fixtures                 | partial |
| resource scopeとfinalizer                   | Lesson 18               | exit / defect ordering fixtures           | partial |
| generic struct / alias / impl               | Lesson 11               | kind / inference diagnostics              | partial |
| comprehension、Array / record pattern       | Lesson 12               | parse / pattern diagnostics               | partial |
| custom trait / constraint / custom operator | Lesson 13               | orphan / fixity diagnostics               | partial |
| Semigroup / Monoid                          | Lesson 14               | law fixtures                              | partial |
| generic ADT、recursive type、record spread  | Lesson 19               | recursion / spread diagnostics            | partial |
| monad transformer                           | Lesson 20               | transformer order fixtures                | partial |
| retry / repeat / timeout                    | planned advanced lesson | deterministic Clock fixtures              | planned |
| module、visibility、re-export、cycle        | module guide            | `fixtures/projects/modules-*`             | planned |
| TypeScript foreign blockとABI               | interop guide           | `fixtures/projects/foreign-*`             | planned |
| `.d.ts`変換                                 | converter guide         | input / generated snapshot projects       | planned |
| manifest、dependency、lockfile              | package guide           | `fixtures/projects/package-*`             | planned |
| parser recovery、formatter、LSP、highlight  | lessons全体             | tooling snapshots                         | partial |

## 次に埋める順序

1. 各Lessonに対応するcompile / diagnostic fixtureを追加する。
2. module、interop、packageをmulti-file project fixtureで固定する。
3. 全sourceをformatter、LSP、syntax preview、playgroundが共有するrunnerを作る。

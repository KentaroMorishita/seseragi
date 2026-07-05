# Language feature coverage

この表は `docs/spec/` の規範的機能を、学習用lessonまたは機械検証fixtureへ結びつけます。
`covered` は対象sourceが存在する状態、`planned` は必要なtargetを特定したが未作成の状態です。
言語実装のconformance完了条件は、required行がすべてcoveredになり、positive / negativeの両方が必要な
機能でdiagnostic fixtureも通ることです。

| 仕様領域                                    | 学習用target            | 機械検証target                             | 状態    |
| ------------------------------------------- | ----------------------- | ------------------------------------------ | ------- |
| program、entry point、Effect main           | Lessons 01-18           | entry signature diagnostics                | partial |
| literal、application、`$`、pipeline         | Lessons 01-05           | precedence fixtures                        | partial |
| optional record field / presence            | Lesson 27               | `compile/optional-record-field.ssrg`       | partial |
| match、tuple、range、effectful for          | Lessons 03 and 05       | exhaustiveness diagnostics                 | partial |
| newtype、deriving、operator overload        | Lesson 07               | coherence diagnostics                      | partial |
| Array/List、lambda、Either                  | Lessons 04 and 08       | collection boundary fixtures               | partial |
| Map / Set ordering and serialization        | Lesson 06               | seed / duplicate / canonical fixtures      | partial |
| ApplicativeとValidation accumulation        | Lesson 09               | Applicative law fixtures                   | partial |
| Monad、`>>=`、do desugar                    | Lesson 10               | Monad law / invalid bind pattern fixtures  | partial |
| Signalとtransaction                         | Lesson 15               | glitch / subscription fixtures             | partial |
| Effect / Stream requirement merge           | none                    | compile / invalid-position fixtures        | partial |
| Fiber、Deferred、cancellation               | Lesson 16               | scheduler / cancellation fixtures          | partial |
| Streamとbackpressure                        | Lesson 17               | merge / overflow fixtures                  | partial |
| resource scopeとfinalizer                   | Lesson 18               | exit / defect ordering fixtures            | partial |
| generic struct / alias / impl               | Lesson 11               | kind / inference diagnostics               | partial |
| comprehension、Array / record pattern       | Lesson 12               | parse / pattern diagnostics                | partial |
| custom trait / instance / custom operator   | Lesson 13               | instance syntax / orphan / fixity fixtures | partial |
| Semigroup / Monoid                          | Lesson 14               | law fixtures                               | partial |
| generic ADT、recursive type、record spread  | Lesson 19               | recursion / spread diagnostics             | partial |
| monad transformer                           | Lesson 20               | transformer order fixtures                 | partial |
| retry / repeat / timeout                    | planned advanced lesson | deterministic Clock fixtures               | planned |
| process signal / graceful shutdown          | process guide           | `fixtures/projects/process-shutdown-*`     | planned |
| Byte / Bytes / UTF-8                        | Lesson 21               | byte range / invalid UTF-8 fixtures        | partial |
| Decimal exact arithmetic / rounding         | Lesson 22               | parse / division / rounding fixtures       | partial |
| Regex / Unicode / byte spans                | Lesson 23               | syntax / empty match / Unicode fixtures    | partial |
| timezone / DST local resolution             | Lesson 24               | gap / overlap / tzdb mismatch fixtures     | partial |
| Path / filesystem resource ownership        | Lesson 25               | atomic / stream / cleanup fixtures         | partial |
| child process streaming / termination       | process guide           | `fixtures/projects/child-process-*`        | planned |
| HTTP streaming / connection lifetime        | HTTP guide              | `fixtures/projects/http-*`                 | planned |
| module、visibility、re-export、cycle        | module guide            | `fixtures/projects/modules-*`              | planned |
| TypeScript foreign blockとABI               | interop guide           | `fixtures/projects/foreign-*`              | planned |
| `.d.ts`変換                                 | converter guide         | input / generated snapshot projects        | planned |
| generic TS ABI / callback lifetime          | interop guide           | wrapper / callback resource projects       | planned |
| generated naming / declaration merge        | converter guide         | naming / namespace snapshot projects       | planned |
| source map / cross-language stack           | interop guide           | defect / rejection stack snapshots         | planned |
| manifest、dependency、lockfile              | package guide           | `fixtures/projects/package-*`              | planned |
| parser recovery、formatter、LSP、highlight  | lessons全体             | tooling snapshots                          | partial |
| diagnostic schema / inference explanation   | none                    | diagnostic JSON / explain snapshots        | partial |
| document comments / doctest                 | none                    | doc HTML / JSON / doctest snapshots        | planned |
| JsonEncode / JsonDecode deriving            | Lesson 26               | codec wire / strict field fixtures         | partial |
| pure Html props / children / SSR            | Lesson 27               | HTML escaping / props fixtures             | partial |
| Signal-driven DOM / event resource lifetime | planned advanced lesson | DOM reconciliation / hydration projects    | planned |
| performance erasure / stack safety          | none                    | differential / IR shape / benchmark suite  | planned |

## 次に埋める順序

1. 各Lessonに対応するcompile / diagnostic fixtureを追加する。
2. module、process、interop、packageをmulti-file project fixtureで固定する。
3. erasure、tail call、specialization、fusionをIR shape fixtureで固定する。
4. 全sourceをformatter、LSP、syntax preview、playgroundが共有するrunnerを作る。

# Language feature coverage

この表は `docs/spec/` の規範的機能を、学習用lessonまたは機械検証fixtureへ結びつけます。
`covered` は対象sourceが存在する状態、`planned` は必要なtargetを特定したが未作成の状態です。
言語実装のconformance完了条件は、required行がすべてcoveredになり、positive / negativeの両方が必要な
機能でdiagnostic fixtureも通ることです。

| 仕様領域                                      | 学習用target      | 機械検証target                                 | 状態    |
| --------------------------------------------- | ----------------- | ---------------------------------------------- | ------- |
| program、entry point、Effect main             | Lessons 01-18     | entry signature diagnostics / `project-schema-1/imported-effect-console` | partial |
| literal、application、`$`、pipeline           | Lessons 01-05     | literal / diagnostic / precedence fixtures     | partial |
| optional record field / presence              | Lesson 27         | `compile/optional-record-field.ssrg`           | partial |
| match、tuple、range、effectful for            | Lessons 03 and 05 | exhaustiveness diagnostics                     | partial |
| newtype、deriving、operator overload          | Lesson 07         | coherence diagnostics                          | partial |
| Array/List、lambda、Either                    | Lessons 04 and 08 | collection boundary fixtures                   | partial |
| pure / Effectful traversal short-circuit      | Lesson 04         | `compile/short-circuit-traversal.ssrg`         | covered |
| Map / Set ordering and serialization          | Lesson 06         | seed / duplicate / canonical fixtures          | partial |
| ApplicativeとValidation accumulation          | Lesson 09         | Applicative law fixtures                       | partial |
| Monad、`>>=`、do desugar                      | Lesson 10         | Monad law / invalid bind pattern fixtures      | partial |
| Signalとtransaction                           | Lesson 15         | `projects/signal-transaction-lifetime`         | covered |
| Effect / Stream requirement merge             | none              | compile / invalid-position fixtures            | partial |
| cross-module Effect function / cold invocation | module guide      | `project-schema-1/imported-effect-console`      | covered |
| Fiber、Deferred、cancellation                 | Lesson 16         | `projects/effect-stream-simultaneous-failure`  | partial |
| Streamとbackpressure                          | Lesson 17         | `projects/stream-*`, `effect-stream-*`         | partial |
| resource scopeとfinalizer                     | Lesson 18         | exit / defect ordering fixtures                | partial |
| generic struct / alias / impl                 | Lesson 11         | nested generic / kind / inference fixtures     | partial |
| rank-1 generic pure function / module import  | Lesson 11         | `project-schema-1/rock-paper-scissors-domain-split` | partial |
| comprehension、Array / record pattern         | Lesson 12         | parse / pattern diagnostics                    | partial |
| custom trait / instance / custom operator     | Lesson 13         | instance syntax / orphan / fixity fixtures     | partial |
| Semigroup / Monoid                            | Lesson 14         | law fixtures                                   | partial |
| generic ADT、local / mutual recursion、spread | Lesson 19         | local recursion / forward reference fixtures   | partial |
| monad transformer                             | Lesson 20         | transformer order fixtures                     | partial |
| retry / repeat / timeout                      | Lesson 29         | `projects/effect-temporal-control`             | covered |
| test declaration / discovery / runner         | test guide        | `projects/test-discovery`                      | partial |
| standard input / EOF / line decoding          | stdin guide       | `projects/stdin-lines`                         | partial |
| deterministic Random / secure Entropy         | random guide      | compile fixture / `projects/random-seed`       | partial |
| process signal / graceful shutdown            | process guide     | `projects/process-shutdown-*`                  | partial |
| process current directory / portable Path     | Lesson 25         | `compile/process-current-directory.ssrg`       | covered |
| Byte / Bytes / UTF-8                          | Lesson 21         | byte range / invalid UTF-8 fixtures            | partial |
| Hex / Base64 / Unicode grapheme / normalize   | Lesson 28         | `compile/bytes-and-unicode.ssrg`               | covered |
| Decimal exact arithmetic / rounding           | Lesson 22         | parse / division / rounding fixtures           | partial |
| Int / Float parse / format / safe arithmetic  | none              | `compile/number-apis.ssrg`                     | covered |
| BigInt exact arithmetic / checked failure     | Lesson 31         | `compile/big-int-apis.ssrg`                    | covered |
| Regex / Unicode / byte spans                  | Lesson 23         | syntax / empty match / Unicode fixtures        | partial |
| timezone / DST local resolution               | Lesson 24         | gap / overlap / tzdb mismatch fixtures         | partial |
| Path / filesystem resource ownership          | Lesson 25         | `projects/filesystem-temporary-cleanup`        | partial |
| child process streaming / termination         | process guide     | `projects/child-process-captured`              | partial |
| HTTP streaming / connection lifetime          | HTTP guide        | `projects/http-*`                              | partial |
| module、visibility、re-export、cycle          | module guide      | `fixtures/projects/modules-*`                  | covered |
| namespace-qualified value / type / constructor import | module guide | `project-schema-1/namespace-generic-call` | partial |
| TypeScript foreign blockとABI                 | interop guide     | `projects/foreign-*`, `typescript-abi-*`       | partial |
| `.d.ts`変換                                   | converter guide   | `projects/dts-*`                               | partial |
| generic TS ABI / callback lifetime            | interop guide     | `projects/{typescript-abi,dts-callback}-*`     | partial |
| generated naming / declaration merge          | converter guide   | `projects/dts-{generated*,*merge,namespace*}`  | partial |
| foreign module load mode / single-flight      | interop guide     | `projects/foreign-pure-load`, `foreign-task-*` | covered |
| source map / cross-language stack             | interop guide     | `projects/source-map-rejection`                | partial |
| manifest、dependency、lockfile                | package guide     | `fixtures/projects/package-*`                  | covered |
| parser recovery、formatter、LSP、highlight    | lessons全体       | tooling snapshots                              | partial |
| stable tool options / target capabilities     | none              | `projects/target-capabilities`                 | covered |
| diagnostic schema / inference explanation     | none              | diagnostic JSON / explain snapshots            | partial |
| document comments / doctest                   | none              | `projects/doc-tests`                           | partial |
| closed deprecation metadata / tooling         | none              | `compile/deprecation-metadata.ssrg`            | covered |
| JsonEncode / JsonDecode deriving              | Lesson 26         | codec wire / strict field fixtures             | partial |
| pure Html props / children / SSR              | Lesson 27         | HTML escaping / props fixtures                 | partial |
| Signal-driven DOM / event resource lifetime   | Lesson 30         | `projects/dom-*`                               | covered |
| performance erasure / stack safety            | none              | `projects/performance-*`                       | partial |
| benchmark discovery / baseline regression     | none              | `projects/benchmark-discovery`                 | covered |

## 次に埋める順序

1. 各Lessonに対応するcompile / diagnostic fixtureを追加する。
2. module、process、interop、packageをmulti-file project fixtureで固定する。
3. erasure、tail call、specialization、fusionをIR shape fixtureで固定する。
4. 全sourceをformatter、LSP、syntax preview、playgroundが共有するrunnerを作る。

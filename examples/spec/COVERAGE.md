# Language feature coverage

この表は `docs/spec/` の規範的機能を、学習用lessonまたは機械検証fixtureへ結びつけます。
`covered` は対象sourceが存在する状態、`planned` は必要なtargetを特定したが未作成の状態です。
言語実装のconformance完了条件は、required行がすべてcoveredになり、positive / negativeの両方が必要な
機能でdiagnostic fixtureも通ることです。

| 仕様領域                                      | 学習用target      | 機械検証target                                 | 状態    |
| --------------------------------------------- | ----------------- | ---------------------------------------------- | ------- |
| program、entry point、Effect main             | Lessons 01-18     | entry signature diagnostics / `project-schema-1/{imported-effect-console,rock-paper-scissors-cli-split}` | partial |
| literal、application、`$`、pipeline           | Lessons 01-05     | `schema-1/pipeline-application` + `execution-schema-1/pipeline-application` + literal / diagnostic fixtures | partial |
| template interpolation / `Show<A>` evidence  | Lessons 01-05     | `schema-1/{template-interpolation,template-invalid-escape}` + `semantic-diagnostics-schema-1/template-missing-show` + `execution-schema-1/template-interpolation`; Stringとlocal derived ADTのdictionary dispatch、複数行 / escaped `${` frontend | covered |
| optional record field / presence              | Lesson 27         | `compile/optional-record-field.ssrg`           | partial |
| match、tuple、effectful for                   | Lessons 03 and 05 | RPS artifacts + exhaustiveness diagnostics      | partial |
| `Range<Int>` literal / standard `Reducible`  | Lessons 05 and 12 | `schema-1/{range-reduce,range-invalid-endpoint}` + `execution-schema-1/range-reduce` | covered |
| newtype、deriving、operator overload          | Lesson 07         | `schema-1/{pure-comparison,string-add,user-add-operator}` + `execution-schema-1/{pure-comparison-string,string-add,user-add-operator}` + `semantic-diagnostics-schema-1/{user-add-missing,operator-reference-missing}` + `project-schema-1/imported-user-add-operator`; standard / local / imported / scoped `Add` evidence、binary / contextual operator section dictionary dispatch、user `Add` callbackによるArray reduce。generalized constrained value scheme、struct / newtype糖衣、user Eqは未接続 | partial |
| Array/List、Iterator、lambda、Either          | Lessons 04 and 08 | `schema-1/{array-literal,list-literal,list-comprehension,iterator-unfold,user-iterable-comprehension}` + `execution-schema-1/{list-comprehension,iterator-unfold,user-iterable-comprehension}` + `semantic-diagnostics-schema-1/reducible-missing` + `project-schema-1/imported-iterable-comprehension`; persistent `Empty / Cons` List literal、List `Iterable` / `Reducible`、lazy `unfold` / `next` runtime ABI、local / scoped / imported user `Iterable` / `Reducible` dictionary execution | partial |
| pure / Effectful traversal short-circuit      | Lesson 04         | `compile/short-circuit-traversal.ssrg`         | covered |
| Map / Set ordering and serialization          | Lesson 06         | seed / duplicate / canonical fixtures          | partial |
| HKT / `Functor<F<_>>` / generic `map`          | Lessons 09-11     | `schema-1/{functor-maybe,partial-functor-value,polymorphic-partial-functor,type-constructor-kind-mismatch,monad-maybe,monad-either,monad-laws}` + execution fixtures + `project-schema-1/{imported-functor-dispatch,imported-higher-order-functor,imported-generic-adt-functor,imported-generic-adt-monad}`; kind arity、constructor inference、期待関数型とgeneric outer constraintからのpartial evidence capture、部分適用`Either<E, _>`、parameter / local / imported evidence、higher-order functionとuser generic ADT transport、`<$>` dictionary execution、identity / composition law regression | partial |
| ApplicativeとValidation accumulation          | Lesson 09         | `schema-1/{applicative-maybe,applicative-validation,monad-maybe,monad-either,monad-laws}` + execution fixtures + `project-schema-1/imported-generic-adt-monad`; Functor supertrait、`pure` / `apply` / inherited `map`、部分適用constructorとimported user ADTのfactory materialization、`<*>` dictionary execution、user-defined Validationのordered error accumulation、identity / homomorphism law regression。標準`std/validation` / `NonEmptyList` ABIは未接続 | partial |
| Monad、`>>=`、do desugar                      | Lesson 10         | `schema-1/{monad-maybe,monad-either,monad-laws}` + execution fixtures + `semantic-diagnostics-schema-1/monad-do-invalid` + `project-schema-1/imported-generic-adt-monad`; transitive supertrait、generic `pure` / `flatMap`、`>>=`、dictionary-driven pure do、Maybe / Either short-circuit、imported user ADTのbind / pure do、do固有diagnostic、left / right identityとassociativity law regression | partial |
| Signalとtransaction                           | Lesson 15         | `projects/signal-transaction-lifetime`         | covered |
| Effect / Stream requirement merge             | none              | compile / invalid-position fixtures            | partial |
| cross-module Effect function / cold invocation | module guide      | `project-schema-1/{imported-effect-console,rock-paper-scissors-cli-split}` | covered |
| imported public scheme nominal provenance     | module guide      | direct / transitive / namespace / same-spelling compiler tests | covered |
| direct / transitive imported user instance dispatch | module guide | `project-schema-1/{imported-instance-dispatch,transitive-instance-dispatch}` | covered |
| multi-case project execution / closed descriptor | module guide   | `project-schema-1/rock-paper-scissors-cli-split/executions/*` | covered |
| Fiber、Deferred、cancellation                 | Lesson 16         | `projects/effect-stream-simultaneous-failure`  | partial |
| Streamとbackpressure                          | Lesson 17         | `projects/stream-*`, `effect-stream-*`         | partial |
| resource scopeとfinalizer                     | Lesson 18         | exit / defect ordering fixtures                | partial |
| generic struct / alias / impl                 | Lesson 11         | nested generic / kind / inference fixtures     | partial |
| rank-1 generic / higher-order pure function / operator section / module import | Lesson 11 | `schema-1/generic-higher-order-call` + `schema-1/operator-reference` + `project-schema-1/rock-paper-scissors-domain-split` | partial |
| comprehension、Array / List / record pattern  | Lesson 12         | `schema-1/{range-comprehension,array-comprehension,list-comprehension,comprehension-pattern-filter,comprehension-invalid-source,user-iterable-comprehension}` + `execution-schema-1/{range-comprehension,list-comprehension,comprehension-pattern-filter,user-iterable-comprehension}` + `project-schema-1/imported-iterable-comprehension`; Array / Range / List / local / scoped / imported user `Iterable<C, A>` evidence、generic user `Reducible<C, A>` parameter dispatch、Array / List result、guard、multiple generator、constructor / tuple pattern filter、dictionary `iterate` / `reduce` execution | partial |
| custom trait / instance / custom operator     | Lesson 13         | `interface-schema-1/{basic-trait,rich,constrained-instance}` preservation + `semantic-diagnostics-schema-1/{instance-contract,instance-contract-mismatch,instance-method-body,trait-method-ambiguous}` contract / body / ambiguity validation + `schema-1/{user-instance-dictionary,trait-method-candidates,generic-instance-dispatch,constrained-instance-dispatch,constrained-function-dispatch,partial-constrained-function,polymorphic-partial-constrained-function,method-constraint-dispatch,standard-show-evidence}` local method body / selected evidence / same-named method selection / concrete, unconstrained generic, recursively constrained local dictionary call lowering, instance and method scoped evidence consumption, saturated / concrete partial / outer-generic partial local constrained function dictionary passing, and runtime `Show<String>` dictionary factory argument + ordered multi-argument instance heads through TypedHir / CoreIr / TypeScriptIr / generated metadata + `project-schema-1/{imported-trait-instance-contract,imported-constrained-function,imported-generic-instance-dispatch,imported-constrained-instance-dispatch,transitive-constrained-instance-dispatch}` imported contract、provider-local constraint identity、consumer-local / direct / transitive provider generic / recursively constrained dictionary passing、closed TypeScript check and execution + resolved trait / constraint names + orphan / fixity / `project-schema-1/imported-effect-failure` | partial |
| structured constraint arguments / evidence passing | Lesson 13    | `schema-1/{constraint-arguments,array-reduce,int-arithmetic,operator-reference,string-add}` + `execution-schema-1/{array-reduce,string-add}` | partial |
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
| manifest、dependency、lockfile                | package guide     | `fixtures/projects/package-*` + split RPS manifest discovery | partial (path dependency execution + source identity audit) |
| parser recovery、formatter、LSP、highlight    | lessons全体       | tooling snapshots + Phase 1 format round-trip   | partial |
| shared-driver playground / browser host       | Lesson 01         | `tests/playground-wasm.integration.ts`         | covered |
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
4. formatter-0はshared lossless frontendとdriverへ接続済み。resolved fixityを使うcanonical spacing / wrappingと
   range formatを追加し、grammar coverageのformatter targetを実行snapshotへ昇格する。
5. local Package CLIはsplit RPSをfixture descriptorなしの`seseragi run .`で実行済み。registry / alias / path dependencyの
   typed manifest contractとcanonical local package identity graphも固定した。dependency export解決、cross-package source graph、
   shared driver compile、`package-path-dependency-basic`のCLI実行、entry非到達fileを含むsource identity auditまで固定済み。次は
   full collection fixtureとregistry / lockfile resolutionを独立に回収する。

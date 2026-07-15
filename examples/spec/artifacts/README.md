# Compiler artifact contracts

このdirectoryは、compiler stage間で受け渡すdebug / conformance artifactのschema fixtureです。
言語の意味やpublic ABIではなく、複数実装laneを疎結合に検証するためのversioned test contractです。

`schema-1/basic/` は同じsourceに対する次の四artifactを固定します。

- `tokens.json`: triviaとEOFを含むlossless token列
- `cst.json`: token rangeを参照するlossless CST骨格
- `diagnostics.json`: frontend共通diagnostic envelope
- `semantic-diagnostics.json`: type / semantics stageのdiagnostic envelope
- `interface.json`: module consumerが読む公開interface
- `surface-ast.json`: surface declarationとbody expression / pattern。後続stageがある場合はstage chainの入口
- `resolved-ast.json`と`typed-hir.json`: Rust conformance runnerで個別に比較できるfrontend / semantics stage snapshot
- `typed-interface.json`: typed bodyから推論した公開contractを含むpublic API snapshot
- `core-ir.json`から`typescript-ir.json`: backend loweringとemitterを含む最小end-to-end snapshot
- `generated-module.json`と`main.ts`: runtime requirementと公開export listを含むemitter結果
- `main.ts.map`: portable Seseragi URIとsourcesContentを持つsource map v3

`schema-1/recovery/`は式が欠けた編集中sourceを固定します。token列は入力を失わず、CSTはzero-widthの
`error-expr`とmissing expressionを持ち、diagnosticのprimaryも挿入位置のzero-width rangeです。Errorを持つ
moduleは公開interfaceへ不完全なsymbolを出しません。

`schema-1/effect-do/`は最小の`effect fn main`と、最後のmonadic expressionをresultとして持つ
`do { succeed () }`をfrontend artifactとして固定します。

`schema-1/effect-do-pure-let/`はdo block内のpure bindingをmonadic bindと区別して固定します。pure initializerは
通常の式として型付け・lowerし、生成TypeScriptではlexical `const`になります。pure letだけを理由に
`effect.core.flatMap`を要求せず、最後のEffect valueだけがrunner境界で実行されます。

`schema-1/effect-succeed-value/`はgenericな`succeed`を固定します。EffectCall自身が具体化済み`R / E / A`を
TypedHirに持ち、文字列引数から`Effect<{}, Never, String>`を導出したままCoreIrとruntime callへ渡します。

`schema-1/effect-fail-adt/`はuser定義ADT値をfailure channelへ送るgenericな`fail`を固定します。
`fail Invalid`は`Effect<{}, AppError, Never>`へ具体化され、生成moduleはcoldなruntime helperを呼ぶだけで
`throw`や`await`を直接出しません。typed failureとdefectの区別はruntime runner境界が保持します。

`schema-1/effect-map-error-adt/`はfailure ADTの明示変換を固定します。payload constructor
`InvalidHand: HandInputError -> AppError`をmapper関数として解決し、nested `fail`のfailureだけをAppErrorへ変換します。
sourceのenvironmentとsuccess型は保持し、生成moduleもcoldな`mapError(mapper, effect)`の合成だけを出力します。

SurfaceAstの`let`、`fn`、`effectFn`は有効なsourceでは`body`を必ず持ちます。applicationはcurried
applicationを保つ一引数nodeとして左へnestし、括弧は`grouped`で保持します。`do`はEffect専用構文として
扱わず、bind / pure let / expression itemと最後のresultを分離します。item terminatorは規範grammarどおり
改行または`;`であり、既知operation名やarityを使って同一行を暗黙分割しません。import itemは元名とaliasの
UTF-8 byte spanを保持し、project linkerのmissing / duplicate / private access diagnosticがimport文全体へ退化しません。

`schema-1/tuple-value/`は2要素以上のtuple valueをSurfaceAst、TypedHir、CoreIr、TypeScriptIr、生成TSまで
固定します。TypeScript backendはtuple用runtime helperを要求せず、同じ長さのreadonly tupleとして出力します。
`surface-schema-1/tuple-pattern/`はname / wildcardを含むnested tuple patternの共通SurfaceAst表現を固定し、
pattern typingとdecision tree loweringはmatch sliceでこの表現を引き継ぎます。

`schema-1/rock-paper-scissors-domain/`はclosed ADTとtuple patternを組み合わせたpure domain sliceです。
`Hand` / `Outcome`、`decide`、`renderOutcome`を通常pipelineでCSTからgenerated TypeScriptまで生成し、
TypedHirのexhaustiveness proof、CoreIrのsingle-scrutinee decision、TypeScriptIrのtag testとresidual fallbackを
固定します。生成物はruntime match helperを要求せず、metadataはADT / String表現だけを宣言します。

`schema-1/parse-hand-literals/`はInt / String / Bool literal patternのうち、CLI入力に必要なString matchを
ADT constructorと組み合わせて固定します。openなString domainには最後のbinding armを要求し、CoreIrのliteral testを
TypeScriptのstrict equalityへlowerします。`HandParse`はstandard `Either`接続前のmonomorphicなdomain resultであり、
`Maybe` / `Either`のpublic ABIを代用するものではありません。

`schema-1/standard-sum-values/`はstandard `Either` / `Maybe`の4 constructorをlocal ADT値と組み合わせ、
期待型によるgeneric具体化からTypeScript runtime bindingまでを固定します。`Right` / `Left` / `Just`はimport済みcall、
`Nothing`はcallしないsingleton referenceです。TypeScriptIrのruntime requirementとsource map名もcanonical registryから
生成し、localな同名constructorはruntime importへ変換しません。`execution-schema-1/standard-sum-values/`は
pure Unit entryをEffect runnerへ渡さず直接呼び、生成値のJSONをruntime package込みで比較します。

`schema-1/parse-hand-either/`はString literal matchの結果をstandard `Either<HandInputError, Hand>`で返します。
`execution-schema-1/parse-hand-either-valid/`と`parse-hand-either-invalid/`はpure entryへtyped String引数を渡し、
生成TypeScriptとversioned runtimeを通した`Right Rock` / `Left (UnknownHand input)`をJSONで固定します。

`schema-1/effect-parse-hand/`は同じpure `parseHand`を`fromEither`でcold Effectへ変換します。
TypedHirは`Either<HandInputError, Hand>`の型引数からfailure / successを取り出し、CoreIrとTypeScriptIrは
`effect.core.fromEither` runtime helperへのcallを保持します。生成moduleはEffectを実行せず、`async` / `await`や
直接throwを出しません。`execution-schema-1/effect-parse-hand-valid/`はEffect runnerが観測した
`Right Rock`由来のsuccess payloadを`expected.exit`と比較します。

`schema-1/rock-paper-scissors-cli/`はフェーズ1の統合sliceです。pureな`parseHand` / `parseInput` / `decide` /
`renderOutcome`を、Stdin Effect、`fromEither`、`mapError`、do内pure binding、Console Effectへ通常pipelineだけで
接続します。externalな`StdinError` / `ConsoleError`はresolverのcanonical identityからruntime ABIのtype-only
importへlowerされ、localな同名型を特別扱いしません。`AppError deriving Show`はTypedHirからCoreIr、
fully-resolvedなTypeScriptIr render plan、generated dictionary exportとinstance metadataまで縦に固定します。

`execution-schema-1/rock-paper-scissors-cli-valid/`、`-invalid/`、`-eof/`は生成TypeScriptとversioned runtimeを実行し、
Unit success、`UnknownHand` failure、`EndOfInput` failureを比較します。runnerはtyped interfaceの`Effect<R, E, A>`から
environmentとfailure contractを読み、`Show<E>`に対応するgenerated dictionaryまたはstandard runtime dictionaryを
選択します。typed failureはdictionaryでrenderしたstderrとexit code 1、runtime defectはexit code 70として区別し、
Consoleのactual operation traceもsnapshotと比較します。

`execution-schema-1/rock-paper-scissors-cli-stdin-failure/`と`-console-failure/`は、host serviceの
失敗をruntime defectへ逃がさず、宣言済みのtyped failureへ変換してから`mapError`、`AppError`のderived
`Show`、stderr、exit code 1まで通します。成功caseはcaptured Console operationの引数・stdoutを比較し、
入力失敗caseは副作用が発生していない空traceを比較します。

`semantic-diagnostics-schema-1/match-non-exhaustive/`、`match-unreachable-arm/`、
`match-pattern-type-mismatch/`はmatchのcoverageとpattern typingをregistry code付きで固定します。
`schema-1/match-missing-arm-body-recovery/`は欠けたarm bodyを`SES-P0001`へ写しつつ、後続armをSurfaceAstの
unit testで保持するfrontend recoveryを固定します。

`schema-1/multiple-lets/`は複数top-level declarationのCST分割と、public declarationだけをinterfaceへ出す
最小contractです。TypedHirではprivate declarationも同一compiler run内のbodyとして保持します。

`resolved-ast.json`のenvelopeはschema 2です。公開APIだけの`ModuleInterface`とは別に、private宣言を含む
Surface declaration/body、lexical scope、namespace別symbol、source referenceから`SymbolId`への解決結果、
未解決reference issueを保持します。型検査はこのtableを入力とし、raw spellingからscope探索を再実装しません。
interfaceだけをprojectする互換producerはcompiler内部に残しますが、ResolvedAst conformanceの正本ではありません。

`token-schema-1/`はlexer lane専用のTokenStream fixtureです。CST、diagnostic、module interfaceを要求せず、
fixed operator、comment、literal、nested type argument表面構文、trivia、UTF-8 byte range、EOF、
effect / do表面構文、range operator、member access、lossless reconstructionだけを先に固定します。`>>`のような
operator runを型文脈で分割するかどうかはparser stageの責務で、lexer stageは入力textを失わず保存することを
優先します。

rangeはすべて0-based、end-exclusiveのUTF-8 byteです。tokenの`raw`をEOF以外すべて連結するとsourceと
byte単位で一致しなければなりません。CSTの`startToken` / `endToken`はtoken indexのhalf-open rangeです。

artifactのJSONは人が手で実装する入力ではありません。新compilerが生成し、conformance runnerが比較します。
schema fieldを変更するときはproducerとconsumerを同じ巨大changeへ混ぜず、schema fixture、producer、consumerの
順に移行します。schema majorが異なるartifactを黙って読み替えません。

schema 1 artifactの正規更新はRust writerを使います。`--only`を省略した場合はcase内にすでに存在するartifactだけを
更新し、新しいstageを追加する場合は明示します。TypedHir以降はpublic driverの同一compile結果から書き、compile
error時にfrontend artifactだけを部分更新しません。

```sh
cargo run -p seseragi-conformance --bin write_schema1_artifact -- \
  examples/spec/artifacts/schema-1/basic
cargo run -p seseragi-conformance --bin write_schema1_artifact -- \
  examples/spec/artifacts/schema-1/new-case --only=tokens,cst,diagnostics
```

複数sourceを通常のproject pipelineでcompileするfixtureは`project-schema-1/`へ分離します。`project.json`が
logical module ID、source、generated ESM output、artifact directory、labeled import edge、期待topological orderを明示し、
single-file schemaの`main.ssrg`規約を流用しません。各moduleは`typed-hir.json`、`typed-interface.json`、`core-ir.json`、
`typescript-ir.json`、`generated-module.json`、`main.ts`、`main.ts.map`を持ちます。正規更新は専用writerを使います。
artifact比較後、runnerは各`main.ts`をmetadataのplanned `.ts` output pathへstageし、project全体をTypeScriptで
type-checkします。生成された`.js` ESM importはこの検証のために書き換えません。

同じproject fixtureが`execution.json`を持つ場合、`ProjectExecution` runnerはsourceを通常のclosed project pipelineで
再compileし、すべてのgenerated TypeScriptをplanned pathへstageしてから、entry wrapperが指定entry moduleの元の`.js`
specifierをimportします。schema 1の最初の形はpure JSON invocationだけを許可し、process exit、stdout、stderrを比較します。
期待runtime requirementはdependency-firstのclosed project全moduleからstableに集めます。Effect runner、typed failure
rendering、imported instance dictionary、manifest/package loaderはこのschemaのscope外です。

```sh
cargo run -p seseragi-conformance --bin write_project_schema1_artifact -- \
  examples/spec/artifacts/project-schema-1/rock-paper-scissors-domain-split
```

最初の`rock-paper-scissors-domain-split`は、domain moduleのADT / tuple match / unconstrained rank-1 generic functionと、main moduleからの
type import / value import、constructor pattern、imported pure callを固定します。同じgeneric functionをuser ADTと`String`へ独立して
具体化し、generated module setをTypeScript type-checkしたうえでpure entryを実行します。これはfull CLI executionではなく、
分割moduleのcompiler artifact gateです。

`namespace-generic-call`はnamespace aliasをruntime object accessへ変えず、`domain.identity`、`domain.Hand`、
`domain.Rock / Paper / Scissors`をdependency interfaceのcanonical value / typeへ解決する小機能fixtureです。選択されたmemberだけが
generated TypeScriptのnamed importへ現れ、未使用namespace edgeは従来どおりside-effect importとして残ります。generic functionを
user ADTと`String`へ独立して具体化し、qualified constructorの生成とexhaustive matchをartifact比較、TypeScript type-check、
pure project executionまで固定します。trait / nested namespaceはこのcaseのscope外です。

`imported-trait-instance-contract`はprovider moduleのpublic trait method schemeをnamed importし、consumer moduleの
user-defined instanceがtrait type argument substitutionとprovider nominal typeのcanonical identityを使って契約一致することを
closed project compileで固定します。instance method dictionaryのlowering / runtime dispatchはこのcaseのscope外です。

`schema-1/user-instance-dictionary`はlocal custom traitのconcrete instance methodをTypedHir、CoreIr、TypeScriptIrへ運び、
生成TypeScriptのcompiler-private dictionary objectまで固定します。final typed interfaceはshallow headをcanonical identity付き
instanceへ置換し、custom dictionaryだけではderived `Show`用runtime importを要求しません。trait method callからのinstance selection、
dictionary dispatch、generic / constrained factory、imported dictionaryは後続gateです。

`schema-1/*/typed-hir.json`は`resolved-ast.json`の後続stageとして単独で追加できます。TypedHir producerを
Rust conformance runnerへ接続するとき、同じfixtureに`core-ir.json`や`typescript-ir.json`を同時に固定する
必要はありません。backend loweringやTypeScript emitterまで固定するcaseだけが`core-ir.json`、
`typescript-ir.json`、`generated-module.json`を持ち、full lowering chainとして検査されます。

`typed-interface-schema-1/`は、bodyを読まないshallow `interface.json` では表現できないpublic contractを
TypedHirから生成して固定するstageです。たとえばcompact inferred `effect fn` はsource上に `with` /
`fails` / success型を書きませんが、`typed-interface.json` では `Effect<R, E, A>` の正規型として公開APIに
現れます。

`semantic-diagnostics-schema-1/`は、parseには成功したが型・意味論としては受理できないsourceのdiagnosticを
固定します。syntax/CST由来の`diagnostics.json`とは分け、compact inferred `effect fn` のbodyがEffect型へ
解決できないcaseなど、typed stageで初めて分かる問題を扱います。

初期runner skeletonは次でartifact bundleを発見し、必須file、JSON envelope、参照snapshotを検査します。

```sh
bun run conformance:artifacts
```

machine-readableなreportだけが必要な場合は、Bunのcommand echoを抑えて次を使います。

```sh
bun run --silent conformance:artifacts --json
```

Rust再実装側のconformance runnerも同じ目的でJSON出力を持ちます。通常runのsummaryは
`kind: "conformance-run"`、case discoveryだけが必要な場合は`kind: "conformance-list"`です。

```sh
cargo run -p seseragi-conformance -- . --json
cargo run -p seseragi-conformance -- . --list --json
```

このrunnerはまだcompiler producerを起動しません。手書きcontractを読むconsumerを先に固定し、後からlexer、
parser、backend、host runnerのproducerを一つずつ接続します。

`interface.json`はsource bodyやbackend layoutを含めず、公開symbol、type scheme、operator、instance、dependencyだけを
持ちます。private bodyを必要とする最適化は同一compiler runのTypedHir / CoreIrを使い、interface cacheへ漏らしません。

`interface-schema-1/rich/`はmodule graph lane向けの独立bundleです。複数sourceを入力にし、main moduleから
dependency edge、公開newtype、公開custom operator、coherenceに必要なinstance headだけをinterfaceへ保存します。
frontend artifactと同じbundleへ押し込まず、module resolverがTokenStreamやCSTの内部shapeへ依存することを防ぎます。

`runtime-schema-1/core/abi.json`はTypeScript runtimeのfeature registryです。generated moduleは必要feature IDと
ABI majorだけを記録し、compiler内部のruntime helper pathを推測しません。import不要なInt表現もfeatureとして
登録し、ADTは`core.adt`、standard `Maybe` / `Either`は`core.maybe` / `core.either`のreadonly tagged object表現として
同じ互換性検査へ載せます。`readLine`のEOFは`undefined`ではなくsingleton `Nothing`です。function helperは
`runtime-helper`、`Nothing`のようなimport valueとconstructor functionは`runtime-binding`として区別し、どちらも
module / exportのstructured pairを持ちます。
Show ABIは`core.show.dictionary`をtype-only dictionary contract、`core.string.show` /
`effect.console.error.show` / `effect.stdin.error.show`をruntime bindingとして登録します。
TypeScriptIrはfeature IDとlocal bindingだけを保持します。

`stage-schema-1/effect-main/`は最初のEffect縦sliceです。parameterなし`effect fn`をimplicit Unit parameter、
closed Console requirement、ConsoleError failure、Unit successへ展開し、runtime featureからprintln importを
解決します。TypeScript backendはSeseragi EffectをPromiseやthrowへ勝手に変換せず、runtime Effect valueを返します。

`execution-schema-1/effect-main/`は、generated moduleが返すEffect valueをhost adapterがどう実行するかを
固定します。entry runnerは`main ()`を一度呼び、root resource scopeでEffectを実行し、required environmentへ
Console serviceを提供します。required environmentはclosedですが、actual host environmentは追加serviceを持てます。
成功時はUnit valueとexit code 0、Console traceとstdout snapshotを比較します。
また、entryが参照するgenerated moduleのruntime requirementsを`expected.runtimeRequirements`と比較し、
実行fixtureが依存するruntime ABI featureを固定します。

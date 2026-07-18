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

`schema-1/pipeline-application/`は`$`と`|>`を専用IRへ残さず、通常のcurried applicationとして全stageへ
通します。`16 |> add 5 |> double |> describe`は`describe (double (add 5 16))`と同じ意味です。
`execution-schema-1/pipeline-application/`は同じsourceをConsole hostで実行し、checked Int helperを経た
`Pipeline answer: 42`とactual operation traceを固定します。

`schema-1/custom-infix-operator/`は未知operatorを含む式をSurfaceAstのflat chainへ保持し、全local headerを
読んだ後に`infixl` / `infixr`とstandard precedenceから再結合します。custom nodeやraw JavaScript operatorを増やさず、
通常のcurried function callとしてTypedHir、CoreIr、TypeScriptIrへ運びます。同名execution fixtureは宣言位置より前にある
右結合式の9と左結合式の5をactual outputで固定します。`semantic-diagnostics-schema-1/custom-operator-unknown`と
`custom-operator-fixity-conflict`は、未解決spellingと非結合chainをそれぞれ`SES-P0101` / `SES-P0102`で
backend前に停止します。`custom-operator-invalid-declaration`は1文字、generic delimiterと衝突する
angle-only spelling、二項でない宣言を`SES-P0001`で拒否します。

`project-schema-1/imported-custom-infix-operator/`はpublic operatorのfixityとschemeをdependency interfaceから
consumerへ渡し、右結合の`10 <^> 3 <^> 2`をproviderの`__ssrg$operator$3c5e3e` exportへlowerします。
consumerは同じencoded名をES module importし、raw `<^>`を生成TypeScriptへ残しません。closed TypeScript checkと
project executionは`Imported custom infix: 9`を固定します。

`schema-1/custom-operator-section/`はgeneric constrained `(<^>)`を通常のcurried function値としてhigher-order
parameterへ渡し、同じ値を部分適用できることを全IRと生成TypeScriptへ固定します。期待関数型で`A = Int`を具体化し、
選択した`Difference<Int>` dictionaryを残りのvalue parameterの後へ捕捉します。同名execution fixtureは両経路が7になる
actual outputを確認します。imported operator sectionはproviderのencoded ES module bindingとevidenceをcallbackへ渡し、
raw spellingをbackendへ残しません。`semantic-diagnostics-schema-1/custom-operator-section-unknown/`は未定義sectionを
`SES-P0101 operator.unknown`、`custom-operator-section-unresolved-evidence/`は期待型なしの未具体化constraintを
`SES-T0201 instance.missing`でlowering前に停止します。

`schema-1/string-add/`は`Add<String, String, String>`のstandard evidenceをTypedHirとCoreIrへ保持し、
複数の左結合`+`をString連結へlowerします。`execution-schema-1/string-add/`はcurried invitation functionと
`$`、Consoleを組み合わせ、暗黙の数値変換やInt runtime helperなしでactual outputを固定します。

`schema-1/user-add-operator/`はlocal `Add<Score, Int, Score>`を期待関数型から選び、operator section `(+)`を
生成dictionaryのcurried `add` callbackへlowerしてstandard Array `reduce`へ渡します。
`execution-schema-1/user-add-operator/`はactual outputを固定し、`project-schema-1/imported-user-add-operator/`は
provider dictionary exportのimportとproject executionまで固定します。
`semantic-diagnostics-schema-1/{user-add-missing,operator-reference-missing}/`は対応instanceのないbinary / sectionを
`instance.missing`で拒否します。

`schema-1/user-iterable-comprehension/`は一つの`Countdown`へuser-defined `Iterable<Countdown, Int>`と
`Reducible<Countdown, Int>`を定義します。内包表記はdictionaryの`iterate`、generic
`total<C> where Reducible<C, Int>`はscoped parameter dictionaryの`reduce` methodを呼び、具体callはlocal evidenceを渡します。
`project-schema-1/imported-iterable-comprehension/`は両dictionaryをproviderからimportして同じactual executionを固定します。
標準Array / Range / Listのreduceは従来のoperation ABIを使い、user collectionを標準型名やruntime shapeで分岐しません。
`semantic-diagnostics-schema-1/reducible-missing/`は対応instanceのないconcrete callを`SES-T0201`で拒否します。

`schema-1/newtype-user-id/`は`newtype UserId = Int`をaliasへ潰さず、一constructorのnominal valueとして
constructor適用、payload pattern、全IR、tagged TypeScript表現へ接続します。同名execution fixtureがactual outputを固定し、
`semantic-diagnostics-schema-1/newtype-no-coercion/`はrepresentation型への暗黙unwrapを拒否します。
`project-schema-1/imported-newtype/`は一つのnamed importがtype / constructor両namespaceを導入し、consumer側でconstruct / unwrapして
actual executionすることを固定します。generic newtypeのpayload inferenceはlowering regression testで検証します。

`schema-1/pure-comparison/`はInt、Bool、Stringの比較へstandard `Eq<A>` evidenceを保持します。
`execution-schema-1/pure-comparison-string/`は二引数のpure curried entryを実行し、String `!=`のBool結果を
JSON outputで固定します。`schema-1/user-eq-operator/`はlocal `Eq<Status>` dictionaryを`==`から呼び、generic
`where Eq<T>`の`!=`はparameter dictionaryの同じ`eq`結果を否定します。同名executionと
`project-schema-1/imported-user-eq-operator/`がlocal / scoped / imported dispatchを固定し、`eq-missing/`は
instanceのないADT比較を拒否します。strict equalityやJavaScript object identityをEq instance選択の代用にはしません。

`schema-1/equality-operator-section/`は`(==)` / `(!=)`をcurried関数値としてhigher-order callへ渡し、
local / generic scoped `Eq<Status>` dictionaryとstandard `Eq<Int>`の選択済みevidenceを同じpipelineで保持します。
`execution-schema-1/equality-operator-section/`はdictionary callbackとstandard strict equalityの両方をactual executionし、
`semantic-diagnostics-schema-1/equality-operator-section-missing/`は対応instanceのない関数値を`instance.missing`で停止します。

`schema-1/type-class-operator-section/`は`(<$>)` / `(<*>)` / `(>>=)`をsource順のcurried関数値として
higher-order call、部分適用、generic `where Monad<M>`内の飽和callへ渡します。共有registryのoperand permutationにより
`>>=`だけを個別desugarせず`flatMap continuation source`へlowerし、Prelude Maybeとlexicalなuserland同名traitの
dictionary identityを通常infixと共有します。`execution-schema-1/type-class-operator-section/`はmap / apply / bindと
Nothing short-circuitをactual outputで固定し、`semantic-diagnostics-schema-1/type-class-operator-section-missing/`は
Monad instanceのない`Box`を`SES-T0201 instance.missing`でlowering前に停止します。

`schema-1/operator-section-forbidden/`は`(&&)` / `(||)` / `(??)` / `(|>)` / `($)` / `(:=)` / `(!)` /
`(..)` / `(..=)`を関数値にせず、operator自身のrangeを持つ`SES-P0001 parser.expected-expression`で停止します。
未接続の`Ord` / cons sectionもruntime holeへ流さず、valid custom candidateは従来どおりsemanticsで解決します。

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
instanceへ置換し、custom dictionaryだけではderived `Show`用runtime importを要求しません。`label`の`render value`は
argument型からselected local evidenceを固定し、TypeScriptIrの`dictionary-call`と生成dictionary method invocationまで通します。
constrained factoryのevidence引数とimported dictionaryは後続gateです。

`schema-1/trait-method-candidates`は同じ`present`を宣言する二つのlocal traitをresolverのcandidate集合に残し、
argument型とlocal instance evidenceから一意なtraitを選びます。選択済みcanonical identityはTypedHir / CoreIrに残り、
TypeScriptIrと生成moduleは対応する二つのdictionaryへ別々にdispatchします。型とinstanceを使っても一意に選べないcaseは
`semantic-diagnostics-schema-1/trait-method-ambiguous`が`SES-T0202`として固定します。

`schema-1/generic-instance-dispatch`は`instance<T> Tag<Maybe<T>>`を`Maybe<Int>`のmethod callへ選択し、
推論した`Int`をselected evidenceのordered `typeArguments`としてTypedHir / CoreIrへ固定します。TypeScriptIrは
dictionary factoryを`type-application-call`として表現し、生成moduleとCLI実行は`<bigint>()`で具体化したdictionaryから
methodを呼びます。`execution-schema-1/generic-instance-dispatch`は同じ生成moduleをConsole hostで実行し、
actual operation traceとstdoutを固定します。

`schema-1/constrained-instance-dispatch`は`instance<T> Render<Maybe<T>> where Ready<T>`を
`Maybe<Badge>`へ選択するとき、必要な`Ready<Badge>`のlocal instanceを再帰選択します。orderedな
`evidenceArguments`をTypedHir / CoreIrへ保持し、TypeScriptIrのdictionary factory callと生成TSの
`<Badge>(readyDictionary)`へlowerします。`execution-schema-1/constrained-instance-dispatch`は同じ生成moduleを
Console hostで実行します。`Render` methodの`Just item -> ready item`はinstance constraintを
`parameter` evidenceとしてTypedHir / CoreIrへ保持し、生成TSではfactory closureのcompiler-private evidence名からdispatchします。
このcaseはcall siteでのlocal evidence materializationとmethod bodyでの消費を両方固定するgateです。
imported evidenceのfactory引数化はdirect / transitive project fixtureで、materializableなstandard `Show<Int>` / `Show<String>`は
`schema-1/standard-show-evidence`で接続します。first-class constrained functionのvalue schemeと、
operation-only standard traitのdictionary ABIは独立した後続gateです。

`schema-1/standard-show-evidence`はruntimeが公開する`Show<String>` dictionaryを選び、
`instance<T> Render<Maybe<T>> where Show<T>`のlocal generic factoryへ渡します。factory内のscoped evidenceはさらに
`acknowledge<T> where Show<T>`を飽和させ、生成TSは`@seseragi/runtime/show`の`stringShow`を実際に消費します。
`execution-schema-1/standard-show-evidence`がstdoutまで固定するため、standard identityをIRへ保存しただけではgreenになりません。
`schema-1/struct-profile`は`Show<Int>` dictionaryをtemplate interpolationから選び、実際のdecimal出力を固定します。
`Add` / `Eq` / `Iterable` / `Reducible`は専用operation ABIのままであり、このfixtureは存在しないdictionaryを捏造しません。

`schema-1/generic-struct`は`pub struct Box<A> { value: A }`のconstructionから`A = Int`を推論し、
`pub let inferred`のcontractを`Box<Int>`としてTypedInterfaceへ固定します。generic `replace`は元のBoxから同じargumentを保つ
spread update、generic `unwrap`はStruct patternとmember substitutionを通り、生成TypeScriptはnominal brandを保った
`Box<bigint>`を出力します。`execution-schema-1/generic-struct`とPlaygroundの`Generic Structの推論`が42のactual outputを固定します。

`schema-1/explicit-generic-struct`は明示constructionのnested `Marker<Array<String>>`をSurfaceAst / ResolvedAstへ保持し、
空Array fieldへ明示contextを渡します。TypedHirと生成TypeScriptは`Marker<ReadonlyArray<string>>`を保持し、
`execution-schema-1/explicit-generic-struct`が`Explicit generic Struct: ready`をactual outputで固定します。

`schema-1/partial-functor-value`は`map increment`を期待関数型`Maybe<Int> -> Maybe<Int>`から具体化し、
選択済み`Functor<Maybe>` dictionary methodの部分適用をhigher-order引数へ渡します。TypedHir / CoreIrのevidence、
TypeScriptIrのdictionary call、生成TS、`execution-schema-1/partial-functor-value`の`Just 42`を一つのgateで固定します。
通常のconstrained top-level functionを未飽和で保持するABIはこのcaseの完了条件に含みません。

`schema-1/partial-constrained-function`は通常の`where Ready<T>`付きtop-level関数を一引数だけ適用し、
残りの`String -> String`をhigher-order関数へ渡します。選択済みdictionaryは全value parameterの後ろへ置く既存ABIを
維持するため、TypedHir / CoreIrの`deferredEvidenceParameters`を使って生成TSをeta-expandします。
`execution-schema-1/partial-constrained-function`が`Badge is ready!`を固定し、dictionaryを早く渡す誤生成を拒否します。
`schema-1/polymorphic-partial-constrained-function`はouter generic constraintのparameter evidenceをpartial closureへ捕捉し、
`schema-1/polymorphic-partial-functor`は同じ経路で`Functor<F>`とdeferred `F<A>`を運びます。execution fixtureはそれぞれ
actual dictionary dispatchを固定し、HKT parameter annotationは生成TypeScriptだけ`unknown`へ消去します。
outer evidenceなしでconstraintを保持するgeneralized value schemeは独立した後続gateです。

`project-schema-1/imported-functor-dispatch`はpublic `incrementAll<F<_>> where Functor<F>`のconstructor arityをfinal
TypedInterfaceへ保存し、consumerの`Maybe<Int>`からHKTを具体化します。consumer generated ESMはproviderのfunctionと
`Functor<Maybe>` dictionaryを同じmoduleからimportし、closed TypeScript checkとEffect executionで`Just 42`を固定します。

`project-schema-1/imported-higher-order-functor`はpublic `transform<F<_>, A, B>`の`(A -> B)` parameterをinterfaceから
consumerへ運びます。consumer-local `increment`をfirst-class functionとして渡し、HKT具体化とprovider dictionary importを
同じcallで行い、closed TypeScript checkとEffect executionで`Imported mapper: Just 42`を固定します。

`project-schema-1/imported-generic-adt-functor`はuser-defined `Box<A>`と`Functor<Box>`をproviderへ置きます。consumerは
`Boxed 41 |> transform increment`を書き、generic constructor payload、`Box<Int>` pattern、type-only owner import、provider
dictionaryを同じcallへ統合します。executionは`Imported Box Functor: 42`を固定します。

`project-schema-1/imported-generic-adt-monad`は同じuser-defined `Box<A>`へFunctor / Applicative / Monadを定義します。
consumerはimported generic `bind`とconsumer-local pure `do`の両方で`Monad<Box>`を選び、providerの
Functor -> Applicative -> Monad dictionary factory chainを実行します。executionは両経路の
`Imported Box Monad: 42`を固定します。

`schema-1/applicative-validation`はuser-defined `Validation<E, A>`とrecursive `Errors<E>`を使います。
`pure makeUser <*> validateName name <*> validateAge age`がgeneric Applicative dictionaryを通り、両入力がInvalidなら
left-to-rightにerrorを蓄積します。同名execution fixtureは二errorの順序とValid側の両方を固定します。

`schema-1/comprehension-pattern-filter`はArray generatorの`Just value`と`(1, value)`を通常のpattern decisionへlowerし、
不一致要素を除外してからpayload / tuple bindingをelement式とguardへ渡します。
`execution-schema-1/comprehension-pattern-filter`はconstructor filterの合計4とtuple filterの合計40をConsole traceまで固定し、
pattern不一致でtransformを実行する誤生成を拒否します。sourceの走査はstandard Array `Iterable` runtimeを使います。
user-defined Iterable / Reducible dictionary dispatchは`user-iterable-comprehension`とimported project fixtureで分離検証します。

`schema-1/method-constraint-dispatch`はtrait method自身の`where Labeled<A>`をinstance-level constraintと分離し、
method bodyではordered `parameter` evidence、call siteではprimary `Render<Badge>` dictionaryに続く
`Labeled<Badge>` dictionaryとして運びます。生成TSは通常のmethod引数の後ろへcompiler-private evidence parameterを追加し、
`execution-schema-1/method-constraint-dispatch`がinner dictionary callの結果をConsole traceとstdoutで固定します。

`schema-1/constrained-function-dispatch`はlocal generic pure functionの`where Ready<T>`をbody scopeと
飽和call siteの両方へ接続します。TypedHir / CoreIrはbodyのtrait callへ`parameter` evidence、
callerへ具体的なlocal evidenceを保持します。TypeScriptIrと生成TSはcurried value parameterの後ろへ
compiler-private dictionary parameterを追加し、`execution-schema-1/constrained-function-dispatch`が
actual dictionary methodの結果をConsole traceとstdoutで固定します。first-class partial constrained function、
imported constrained functionはこのsingle-file fixtureの完了条件に含みません。

`project-schema-1/imported-constrained-function`はproviderのpublic constrained function schemeにある
provider-local trait identityをresolved importへ運び、consumerのlocal instance dictionaryを飽和callへ渡します。
provider / consumer双方のTypedHir、CoreIr、TypeScriptIr、generated ESMを正規writerで固定し、closed TypeScript checkと
Effect project executionがactual dictionary methodの結果を観測します。

`project-schema-1/imported-instance-dispatch`はproviderのconcrete user-defined instanceをcanonical trait identityとordered
argument identitiesで選択し、consumer generated ESMがprovider dictionary exportを直接importします。Effect executionは
provider methodの実結果を観測するため、interface transportだけでgreenにはなりません。

`project-schema-1/transitive-instance-dispatch`は同じ能力をprovider / facade / mainへ分け、main sourceがfacadeしかimportしない状態で
original provider dictionaryをvalue importします。facadeはprovider nominal type、trait constraint、instance contractをfinal
interfaceへtransportし、driverはreachable output closureからprovider ESM specifierとexact dictionary exportを計画します。
closed TypeScript checkとEffect executionが`provider ready`を観測します。generic / constraint付きimported factoryはdirect / transitive
どちらのcaseにも含みません。

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

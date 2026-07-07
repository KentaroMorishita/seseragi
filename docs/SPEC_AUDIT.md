# Seseragi 仕様横断監査

この文書は、章単位では見つけにくい矛盾を追跡する非規範な監査記録です。言語の意味は
[`spec/`](./spec/README.md)、機能の網羅状況は[`SPEC_COVERAGE.md`](./SPEC_COVERAGE.md)を正とします。

## 監査の観点

各passでは、少なくとも次を本文、Appendix grammar、diagnostic、exampleの間で突き合わせます。

- 採用しないと宣言した構文や型が、別章の公開signatureへ混入していないか。
- 同じ型、operation、糖衣のsignatureと展開規則が章ごとに一致するか。
- pure / Effect、typed failure / defect / cancellationの境界が変わっていないか。
- resource、Stream、Signal、DOMのorderingとlifetimeが最適化規則を含めて一致するか。
- sourceとして書ける構文がAppendix grammarにあり、失敗にはstable diagnosticがあるか。
- lessonとfixtureが、本文にない意味を独自に作っていないか。

## 完了したpass

### 2026-07-05: 型構文と性能境界

| 確認項目                           | 結果                                                                                         |
| ---------------------------------- | -------------------------------------------------------------------------------------------- |
| arbitrary union / intersection禁止 | generic Effect APIに未定義の`R & { service: Service }`が残っていた                           |
| requirement合成                    | 一般intersectionではない第一型引数限定のrequirement mergeとして定義した                      |
| grammar / diagnostic               | `&`のparse骨格、`SES-T0501`、positive / negative fixtureを追加した                           |
| syntax preview                     | 単独`&`をlogical `&&`やcustom operatorと別tokenへ分類した                                    |
| fixture checker                    | diagnostic codeとseverityが12.13 registryに存在・一致することを検査するようにした            |
| Signal abstraction                 | Functor / Applicativeのみで、Monadを持たない正本は一致していた                               |
| performance optimization           | 評価順、Effect、cancellation、finalizer、Stream demand、DOM event順をas-if境界として確認した |
| recursion                          | direct self tail callのstack safetyを型章と性能章で一致させた                                |

requirement mergeの解決では、既存の公開APIを一般intersection型へ拡張していません。`R & S`は
Effect / Streamのrequirement field集合をcompile時に正規化する型表現だけです。値のintersection、
runtime merge、row remainderを導入しません。

### 2026-07-05: inherent methodとtrait instance

同じ`impl` keywordがnominal型固有methodと型クラスinstanceの二つを表し、headの形だけで意味を
切り替えていました。両者は名前解決、visibility、coherence、orphan rule、backend loweringが異なるため、
次のsurfaceへ分離しました。

```seseragi
impl User {
  pub fn displayName self: User -> String = self.name
}

instance Show<User> {
  fn show value: User -> String = value.displayName
}
```

`impl`はinherent methodと標準operator糖衣、`instance`はtrait dictionaryだけに使います。旧形の
`impl Trait<Type>`は`SES-T0502`とし、grammar、reserved word、syntax preview、lesson、positive / negative
fixtureを同期しました。

### 2026-07-05: 標準ライブラリsignature

`std/array`、`std/list`、`std/map`、`std/set`だけが`empty`をpolymorphic valueとして公開し、Monoid、Bytes、
Streamはparameterなし関数でした。userlandの公開`let`には明示的なpolymorphic scheme構文がなく、標準moduleだけの
特権的な値になるため、すべて`fn empty<...> -> T`と`empty ()`へ統一しました。

module sectionごとの関数名重複はありませんでした。transform / lookup / aggregateはdata-lastで一致しています。
複数入力を持つhost APIは最後のpath、command、requestをprimary subjectと明記しました。

合成failureは次の二形に固定しました。

- upstreamを駆動するadapter: `Either<UpstreamError, LocalError>`
- resourceをcallbackへ渡すbracket API: `Either<ResourceError, UseError>`

これによりfilesystem、buffer、child process、HTTPとtemporary resource APIでEitherの左右が異なる理由を
operation phaseから説明でき、任意union errorを導入せずに済みます。

### 2026-07-05: module、package、TypeScript load境界

Seseragi module importはI/Oを起こさない一方、JavaScript moduleは評価時に任意のcodeを実行できます。以前は
「side effectがある場合はlazy loader」とだけあり、同じhost moduleを複数foreign blockからpure / task混在で
参照した場合のload時点、並行初回call、失敗後の扱いが未定義でした。

resolved host module identity単位で次へ固定しました。

- pure memberが一件でもあればpure-loadとし、transitive host評価全体のpure保証を要求する。
- opaque typeとtask memberだけならtask-loadとし、最初のforeign Taskまで評価しない。
- 並行した初回task callはsingle-flight loadを共有する。
- loadのsuccess / failureをruntime中memoizeし、initializerを暗黙retryしない。
- cancelされたwaiterはload自体をcancelせず、共有結果をcacheできる。
- specifier spellingではなくlockfileへ記録したexact identityをcache keyにする。

`.d.ts` converterの既定module evaluationはtaskです。symbolをpureへ承認するにはmodule評価もpureへ明示承認し、
task-load moduleのvalue、enum member、getterをpure valueとして自動生成しません。generated metadataと現在設定の
evaluation modeが違う場合はstale bindingとしてbuildを拒否します。

instanceだけを到達可能にするimportはsemantic useとして扱い、unused warningやorganize-importsで削除しないことも
module規則へ追加しました。

project fixtureには`project.expect.json`を必須とし、`lock: "generate"`ならrunnerがtemporary copyへoffline lockfileを
生成する規則を定めました。これによりtool-owned lockfileのwire formatをexampleから捏造せず検証できます。
最初の`foreign-task-load` fixtureは、mainの最初のEffectより後にhost moduleを一度だけloadし、二回のtask callで
initializerを繰り返さないstdout traceを固定します。現在のcheckerはproject schema、spec参照、snapshotだけを検査し、
実runは新仕様runner実装後のconformance targetです。

### 2026-07-05: 一般言語の生活必需品

意図的不採用一覧を除外し、literal、local declaration、process I/O、text / number / bytes、random、test runnerを
一般的なprogramの観点で照合しました。高度な抽象の代替では埋まらない未定義項目を
`SPEC_COVERAGE.md`の「未定義・要設計」へ優先度付きで分離しました。

明確な内部矛盾は`Char`です。9.4のstandard instanceと10.7のRegex error payloadが`Char`を要求していますが、2.2の
組み込み型、1.2のliteral、Appendix grammarに存在しません。String / templateのescapeもexact grammarがなく、frontend
実装前に同時に閉じる必要があります。

実用上の大きな穴はtest discovery、standard input、Random / secure entropy、numeric APIの完全signatureです。
`Process.arguments`とenvironmentは既にあります。mutable variable、exception、null、early return、arbitrary union、
break / continue keywordは正本で意図的に不採用ですが、traversalの短絡combinatorは代替として未完です。

### 2026-07-06: literal frontend契約

CharをUnicode scalar一個の組み込み型として追加し、character literal、String、templateのescapeを閉じました。
integer / Float literalは基数、separator、exponent、範囲、binary64丸め、最長一致を定義しました。`Char`の
TypeScript ABIはscalar一個のstringとして境界検査し、`.d.ts`のstringからは自動推論しません。

Appendix grammar、diagnostic registry、syntax previewを同期し、positive fixture一件とinvalid escape、invalid Char、
invalid numericのnegative fixture三件を追加しました。formatterは有効なescapeとnumber spellingを保持します。

### 2026-07-06: traversalの短絡

`break` / `continue`を非局所control transferとして追加せず、pure側は`reduceUntil`と`Next / Done`、Effect側は
`forEachUntil`と`Continue / Break`を標準化しました。通常のeffectful `for`は全件逐次走査の糖衣のままです。
正常な短絡をtyped failure、defect、cancellationへ流用せず、callbackの通常の戻り値として区別します。

Array / Listの`find`、`findIndex`、`takeWhile`、`dropWhile`は既に目的別に短絡します。genericな
`reduceUntil`はIterableだけを要求するため、有限性を仮定せずDoneへ到達した時点で終了します。

### 2026-07-06: local named functionと再帰

block itemへ`fn`、`effect fn`、local `rec` groupを追加しました。local function名は自身のbodyと宣言後、
rec groupのmember名はgroup全体とgroup後でscopeに入ります。後続local functionへのforward referenceを
暗黙hoistせず、相互再帰はrec groupだけで表します。closureは宣言より前のlexical bindingだけをcaptureします。

self recursion、local mutual recursion、forward-reference rejectionをcompile / diagnostic fixtureへ固定しました。
local functionも直接self tail callの一定host stack保証を持ちます。

### 2026-07-06: test discoveryとrunner

testを新構文にせず、`std/test`のimmutableなTest treeと固定export `pub let tests: test.Test`で定義しました。
test rootはmodule path順にcompile / discoveryし、suite treeのsource順を合わせたfull nameでcaseを識別します。
filter、exact selection、jobs、timeout、seed、case isolation、report順、exit codeをtool contractとして固定しました。

runnerはcaseごとにClock、Random、Console、Loggerとroot resource scopeを分離し、timeout、leak、defectをcase
failureとして扱います。`projects/test-discovery`はparallel jobsでもreportをdiscovery順に固定します。

nested genericはgrammar上すでに再帰的なtype argumentとして定義済みでした。現行実装で壊れやすかった連続する
`>>` / `>>>`を型parserが複数のclose delimiterとして扱うことを`nested-generic-types` fixtureへ固定しました。

### 2026-07-06: standard input

出力用Consoleと分離したStdin serviceを追加し、readChunk、readLine、line Streamを定義しました。EOFと空行、
strict UTF-8、line limit、absolute byte offset、concurrent read、cancellationとhost readの競合、sticky EOFを
区別しています。正常なEOFはNothingで、input failureやinvalid UTF-8へ偽装しません。

Stdinは共有cursorなのでStreamを再実行してもinputをreplayせず、その時点の残りから続けます。cancellationは
stdinをcloseせず、競合して読み終えたbytesを内部bufferへ戻して欠落を防ぎます。`projects/stdin-lines`で空行と
EOFを含むhost input / stdout traceを固定しました。

### 2026-07-06: Randomとsecure Entropy

再現可能な疑似乱数Randomと秘密用途のEntropyを別serviceへ分離しました。Randomは
`seseragi-xoshiro256ss-v1`のseed展開・state transition・Int / Float / Bytes変換を固定し、range samplingは
rejection方式、shuffleはFisher-Yatesです。`projects/random-seed`でseed 42の先頭outputをsnapshot化しました。

Entropyはhost CSPRNGからBytesだけを返し、seed、replay、Float、range helperを持ちません。test runnerは既定で
Entropyを提供せず、fakeを使うtestだけが明示provideします。Random bytesをtokenやnonceへ使わない境界を
compile fixtureへ固定しました。Map / Set hash seedのmanifest spellingも`"entropy"`へ明確化しました。

### 2026-07-06: release性能、tooling、runtime lifecycle fixture

性能検証を内部IR textの固定ではなく、意味付きshape predicate、development / release differential、runtime
stack safetyへ分離しました。`newtype-erased`、`surface-sugar-erased`、`self-tail-loop`はrelease outputの性質を
検査し、pass名やIR node名を公開ABIにしません。100,000段のEffect bindと直接self tail callを両profileで実行する
projectも追加し、「最適化した記録」ではなく結果とstack safetyを合格条件にしました。

toolingでは`seseragi doc --test`のblock ID、実行順、report、exit code、atomic outputを固定し、check / run /
compile_failを一つのdocument commentで検証するprojectを追加しました。HTML / JSON document artifactのwire snapshotは
引き続き未作成なので、coverageはpartialです。

runtime lifecycleは次をproject fixtureへ固定しました。

- Signal: subscribe時のcurrent値、multi-source transactionのglitch-freeな一回通知、unsubscribe後の無通知。
- Stream: terminalごとのcold再実行、bufferのFIFO、terminal scope終了時のresource release。
- DOM: synthetic eventの逐次dispatch、Signal transaction後の一回reconcile、dispatch failure後のlistener /
  subscription cleanup、HydrateStrict mismatch時の既存DOM不変更。

この作業中、`std/effect`が名前だけ列挙していた`attempt`、`fromEither`、`fromMaybe`、`service`、`provideSome`の
signature欠落を発見しました。typed failureだけをEitherへ移す境界と、同型serviceを曖昧lookupしないenvironment
selectorを定義し、compile fixtureへ固定しました。

TypeScript namespace変換には、foreign target stringがexact top-level export keyなのにnested runtime memberを
自動生成できると読める矛盾がありました。一度未定義項目へ戻した後、foreign block内だけのcontextual
`namespace local = "HostKey" { ... }`として閉じました。各segmentはexact own property、`.`入りkeyは一つのkeyのまま、
task lookup failureはBindingLookup phaseです。module aliasのlower-name規則を保ち、runtime namespace `.d.ts` snapshotと
hand-written compile fixtureへ固定しました。

### 2026-07-06: simultaneous failureとsibling cleanup

parallel Effect、Stream merge、DOM event queueを同じscheduler turnのfailure選択とcleanup境界で照合しました。

- Effect.parallelは同じturnなら入力index最小のfailureを選ぶ。
- Stream.mergeは同じturnならleft failureを選ぶ。
- どちらもloserをcancelし、winner / loser双方のscope finalizer完了後に結果を公開する。
- 一つのscope内はLIFOだが、独立sibling scopeのfinalizer外部操作順は保証せず、並行実行を許す。
- DOM dispatchはbounded FIFOを一件ずつ完了させるため、複数dispatch failureのsame-turn winner自体が存在しない。
  最初のdispatch failureで受付を止め、queueを破棄してlistener / subscription / dispatch Fiberをcleanupする。

EffectとStreamが同時にleft / rightで失敗し、両finalizer flagがTrueになってからleft failureを観測するproject fixtureを
追加しました。これによりthread数を変えてもwinnerは固定しつつ、sibling finalizerを不必要に直列化しません。

### 2026-07-06: Appendix grammar terminal coverage

Appendix EBNFのquoted terminalを`examples/spec`以下の全Seseragi sourceと機械照合しました。未登場だった
Bool短絡演算子、累乗、Functor map演算子、List literal、match guard、foreign propertyを一つのpositive
fixtureへ固定しました。構造checkerは今後、grammarへ追加されたterminalがlesson / fixtureのいずれにも
現れなければ失敗します。

これは各productionの意味検証やformatter round-tripを証明するものではありません。まず字句上の孤立を
なくすpassであり、production単位のpositive / negative検証はcoverage表に従って引き続き追加します。

### 2026-07-06: standard API costとstorage retention

14.4が要求する計算量、allocation、sharing、入力保持をstandard APIと照合しました。Regexのlinear-time、
Bytesのslice / copy、Streamのbounded bufferは既に明文化されていました。一方、Array / Listの部分結果、
Map / Setのhash operation、String substringには保持境界がありませんでした。

10.2へworst-case / expected / amortized、callback cost、strict resultの共通表記を追加しました。Array / Listは
結果外の要素を隠れて保持せず、Map / Setはexpected cost、collision worst-case、persistent updateのsharing、
削除済みkey / valueの非保持を定義しました。Stringはbyte / scalar indexのcostを分け、小さいsubstringが
巨大な入力storageを保持しないことを保証します。grapheme APIも同じstorage規則へ揃えました。

allocation個数やhost object shapeは観測保証にせず、live resultの漸近storageとuser valueの到達性だけを
公開契約にしています。古いpersistent valueを利用側が保持している場合、その値が所有するentryの回収までは
要求しません。

この照合で、10.8が`BigInt`を標準提供すると宣言し、9.4がZero / One instanceまで要求している一方、公開module
surfaceと算術意味が存在しないことも発見しました。性能規則からAPIを推測せず、P1未定義項目として
`SPEC_COVERAGE.md`へ戻しました。

### 2026-07-06: BigInt surface

再登録したBigIntを`std/big-int`のstandard opaque typeとして閉じました。専用literalやIntとのimplicit
conversionは追加せず、parse / format、明示Int変換、checked division / remainder / powerを提供します。
operatorは任意精度のexact加減乗算、0方向の除算、被除数と同符号の余り、非負Int指数として定義しました。

TypeScriptではIntと同じhost `bigint`へ写像しますが、`.d.ts` converterの既定は引き続き64-bit検査付きIntです。
BigIntは明示overrideだけで選びます。bit lengthとdigit数に基づくstorage / cost上限、Lesson 31、positive fixtureを
追加し、P1登録を解消しました。

### 2026-07-06: Appendix grammar production graph

terminalの出現検査に加え、構造checkerがAppendix EBNFをproduction単位で読み取るようにしました。重複定義、
終端`;`の欠落、未定義production参照、start production `module`から到達不能なproductionを拒否します。
現在の109 productionはすべて定義済みかつ到達可能です。

この検査はgrammar自体の閉包を保証しますが、各productionをparserが受理することやformatter round-tripは
まだ保証しません。次のpassではproductionとconformance source / diagnostic / formatter snapshotの対応を
明示metadataへ移します。

### 2026-07-06: fixed token highlight parity

Appendixの固定operator / separatorをsyntax previewと照合し、専用scopeがなかったlambda `\\`、ADT variantと
comprehensionの`|`、field accessの`.`、terminatorの`;`を追加しました。custom operator文字集合へ誤って
入っていたbackslashも1.8に合わせて除外しました。

構造checkerは主要な固定operator、separator、custom operator sampleが対応するTextMate scopeへexact matchする
ことを検査します。genericなcustom operator ruleが`<`だけを拾って固定tokenの欠落を隠すことは許しません。

### 2026-07-06: grammar production target map

109 productionを12の責務groupへ分け、各groupを最低一件のpositive sourceへ対応づけた
`examples/spec/grammar-coverage.json`を追加しました。checkerはAppendix productionの過不足・重複、group ID、
target pathを検査します。当初はdiagnostic / formatter targetの空を許し、未検証をcoveredと偽装しない
方針にしました。

2026-07-07にformatter targetを全groupで必須化しました。これは新formatterが存在するという意味ではなく、
各production groupに将来のround-trip入力を最低一件割り当てる契約です。diagnostic targetは、意味的な
negative caseをまだ決めていないgroupでは空を許します。

実装を並列化する前提も`docs/IMPLEMENTATION.md`へ分離しました。共有frontendをartifact pipelineとして定義し、
初期のhorizontal laneからcontract安定後のvertical feature sliceへ移る条件、ownership、merge順、最小end-to-end
milestoneを固定しています。

### 2026-07-06: compiler stage artifact schema

Wave 0の最小contractとして、`pub let answer: Int = 42`をtoken、CST、diagnostic、module interfaceへ写した
schema 1 fixtureを追加しました。tokenはtriviaとEOFを含み、raw連結がsource bytesと一致します。CSTは
half-open token range、interfaceはbodyを含まないpublic schemeだけを持ちます。

checkerはUTF-8 byte range、tokenの連続性とlossless再構築、EOF、CST containment、diagnostic envelope、
interface symbolとsource rangeを検査します。これはhand-written expected artifactのcontract検証であり、
新compilerが生成できるという主張ではありません。次にrecoveryと非empty diagnosticを別fixtureで固定します。

recovery fixtureでは式が欠けた公開letを、lossless token列、zero-width `error-expr`、missing expression、
`SES-P0001`へ写しました。diagnostic ID、registry code / severity、message key、UTF-8 primary rangeもcheckerで
検査します。compile errorを持つmodule interfaceは不完全なexportを公開しません。

module interfaceはfrontend artifactと別bundleにし、relative dependencyのcanonical module / imported symbol、
public newtype、custom operatorのsymbol / fixity / precedence / scheme、coherenceに必要なinstance headを固定しました。
checkerはsource range、UTF-8 boundary、symbol namespace、dependency source、operator規則を検査します。module graph
consumerはCSTやfunction bodyへ依存せず、このinterfaceだけをcache入力にできます。

最小のstage chainとしてpublic Int定数をSurfaceAst、ResolvedAst、TypedHir、CoreIr、TypeScriptIrへ順に写し、
`export const answer: bigint = 42n;`まで固定しました。checkerはmodule / symbol / valueのstage間一致、runtime
feature参照、generated metadata、最小emitter結果を検査します。

runtime ABI schema 1はpackage identityとABI major、feature IDを持ち、`core.int64`をsigned 64-bit境界付きの
TypeScript bigint表現として登録します。Int表現にhelper importが不要でもfeature negotiationへ含めます。
source mapはECMAScript v3、portable Seseragi URI、sourcesContent、answer symbolとliteralのmappingを固定しました。
absolute pathを含めず、generated metadataからmap artifactを参照します。

Effectの最小stage chainでは、parameterなし関数のimplicit Unit規則、closed Console environment、
ConsoleError failure、Unit success、`console.println` operationを各段で固定しました。runtime ABIのhelper importは
opaqueな文字列ではなくmodule / exportのstructured pairです。checkerはTypeScriptIrのfeature参照からimportを解決し、
生成TypeScriptとsource mapまで照合します。Promise化やthrow化はこの内部module境界では行いません。

### 2026-07-07: Effect entry execution

`execution-schema-1/effect-main`で、generated moduleが返すEffect valueをhost runnerが実行する境界を固定しました。
runnerは`main ()`を一度呼んでcold Effectを得て、root resource scopeとConsole serviceを提供します。
required environmentはclosed structural recordとして照合しますが、actual host environmentに追加serviceが
存在しても構いません。成功時はUnit、exit code 0、Console trace、stdout / stderr snapshotを比較します。

このfixtureにより、TypeScript backendがEffectをPromise / throw / direct console writeへ潰さないことと、
process的な観測結果をrunner側の契約として扱うことを分離しました。

### 2026-07-07: TokenStream producer skeleton

Rust workspaceを開始し、`seseragi-syntax`が最小TokenStream producer、`seseragi-conformance`が
TokenStream artifact consumerを持つようにしました。最初の`schema-1/basic` / `recovery`に加えて、
lexer lane専用の`token-schema-1/lexical-operators`を追加し、CSTやinterfaceを要求せずに`fn`、`->`、
`|>`、lambda、comment、trivia、EOF、lossless reconstructionを検査します。

このpassはparserや型検査の完成を意味しません。token-only schemaを分けたことで、lexerのcoverageを
parser laneの未実装状態から独立して増やせるようにしました。

`token-schema-1/literals-and-nested-types`では、文字列、template literal、boolean、wildcard、array literal、
および`Array<Maybe<Int>>`のようなnested type argument表面構文をTokenStream contractへ追加しました。
lexerは`>>`をoperator runとしてlosslessに保持します。型構文内でこれを2つの閉じ山括弧として読むかどうかは
parserのcontextual responsibilityであり、custom operator tokenizationだけでnested type parsingを解決済みとは
扱いません。

`token-schema-1/effect-do-block`では、`effect fn`、environment / failure header、`do` block、monadic bind
`<-`、function application `$`、brace、Unit call `()`をTokenStream contractへ追加しました。Effectの意味論や
do desugaringではなく、次のparser sliceへ渡す表面tokenを固定するためのfixtureです。

## 次のpass

1. grammar productionごとのpositive / negative / formatter round-trip対応を機械化する。
2. cost / retention契約をbenchmarkとinstrumented runtime profileへ対応づける。
3. Wave 0のconformance runner skeletonを実装計画へ落とす。

未完了passがあるため、この文書は仕様全体に矛盾がないことを証明しません。passごとに発見事項を本文へ
反映し、解決をfixtureへ固定してから完了として追記します。

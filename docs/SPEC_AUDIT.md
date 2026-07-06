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

## 次のpass

1. parallel Effect / merged Stream / DOM dispatchのsimultaneous failureとfinalizer優先順位をtrace表で照合する。
2. grammarの全productionをlesson / fixture tokenへ対応づけ、formatter・highlightとのtoken差を調べる。
3. standard APIの計算量、allocation、storage retention記述を14章のcost classと照合する。

未完了passがあるため、この文書は仕様全体に矛盾がないことを証明しません。passごとに発見事項を本文へ
反映し、解決をfixtureへ固定してから完了として追記します。

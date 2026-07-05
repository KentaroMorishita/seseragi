# 14. 性能モデルと最適化境界

## 14.1 目的

Seseragiは抽象化を減らして速さを得るのではなく、意味を保ったまま不要な実行時層を消去できる
言語を目指します。本章では、sourceから予測できるコスト、backendが消去してよい抽象、最適化が
越えてはならない意味論上の境界を定めます。

「zero-cost」は、すべてのprogramが手書きJavaScriptと同じ速度になるという保証ではありません。
Seseragiでは次の限定した意味で使います。

> その抽象が要求する意味を手で直接実装した場合と比べ、抽象を選んだことだけを理由とする実行時の
> allocation、dispatch、検査をrelease backendが残さずに済むこと。

実行時間、memory量、JITの最適化結果はhost、入力、service実装に依存するため、言語semanticsとして
特定の数値を保証しません。公式backendの性能退行は14.12のbenchmarkとprofileで管理します。

## 14.2 三種類の保証

性能に関する記述を次の三種類に分けます。

1. **意味論上の保証**: 評価回数、評価順序、stack safety、collectionの計算量など、実装が守る規則。
2. **消去可能性**: runtimeから取り除いても意味が変わらない抽象。実装はその余地を壊してはなりません。
3. **quality of implementation**: specialization、inlining、fusionなど、同じ意味をより少ない仕事で実行する品質。

意味論への適合と性能profileへの適合は別に判定します。遅いが正しいbackendは言語へ適合できますが、
公式release backendを名乗るには14.11のprofileを満たさなければなりません。

## 14.3 観測可能性とas-if rule

3.1の値、左から右の一回評価、Effectの実行順、typed failure、defect、cancellation、finalizer、Streamの
demand、Signal transaction、DOM event順は観測可能です。最適化後も同じ結果と順序を持たなければ
なりません。

値の参照同一性、ADTやrecordのhost object shape、trait dictionaryの有無、closureや中間collectionの
allocation自体は観測できません。backendは、観測可能な意味とforeign ABIを保つ限り自由に表現を
変更できます。

実時間とmemory exhaustionは実装や負荷で変わるため、最適化前後で同一になるとは限りません。ただし、
compilerは逐次Effectを自動並列化したり、cancellation checkpointを越えて処理を移動したり、resourceの
acquire / release範囲を変えたりしてはなりません。速くなった結果としてtimeoutとの競争結果が変わる
ことと、happens-before関係を変更することは区別します。

## 14.4 抽象化のcost class

標準的な抽象を、sourceから予想すべきcost classへ分類します。

| class                | 代表例                                                      | source上のcost expectation                               |
| -------------------- | ----------------------------------------------------------- | -------------------------------------------------------- |
| erased               | type alias、型引数、newtype境界、`$`、pipeline、`effect fn` | 抽象自体はruntime workを要求しない                       |
| static evidence      | trait constraint、deriving、operator overload               | compile時に選択し、generic codeではdictionaryが残りうる  |
| value representation | ADT、tuple、record、Array、List                             | 値、tag、要素、必要な構造を保持する                      |
| closure              | lambda、partial application、高階関数                       | captureが必要ならclosureと保持期間を持ちうる             |
| proportional work    | `map`、`filter`、`reduceUntil`、Show、JSON、SSR             | 入力、出力、または短絡位置に応じて処理する               |
| runtime graph        | Effect、Stream、Signal、Fiber、DOM mount                    | scheduling、queue、subscription、cleanupの管理costを持つ |
| boundary conversion  | foreign binding、Bytes変換、`.d.ts` adapter                 | safety contractに必要な検査やcopyを持ちうる              |

classは個々のoperationの正確な計算量を置き換えません。標準ライブラリは、非自明なoperationについて
worst-caseまたはamortized計算量、allocation、storage sharing、入力保持を公開API文書へ記載します。

## 14.5 消去されるsurface abstraction

次はruntimeで独立した意味を持ちません。

- type aliasと型parameterはruntime descriptorを要求しない。
- newtypeのwrap / unwrapはvalidation、copy、user codeを実行せず、参照identityも作らない。
- `$` とpipelineは1.6の関数適用へ展開され、operator objectを作らない。
- `effect fn`はEffect型を読みやすくする宣言糖衣で、追加のEffect layerを作らない。
- `do`は4.12のbind列と同じ意味で、do block専用のruntime objectを作らない。
- derivingはcompile時に通常のinstance実装を生成し、runtime reflectionでfieldを列挙しない。

newtypeをforeignへ公開するopaque ABI wrapper、debug instrumentation、runtime codecなど、別の契約が
明示的に要求する境界処理はこの消去規則の例外です。境界を越えた後のSeseragi内部表現にまでwrapperを
持ち込む理由にはできません。

## 14.6 関数、curry、trait

複数parameterの関数は意味上curriedですが、すべての段階を個別のhost closureとして実装する必要は
ありません。既知の関数へ引数が十分に与えられ、中間の部分適用結果が観測されない呼び出しは、backendが
multi-argument direct callへlowerできます。引数はsource順に一度だけ評価します。

部分適用した関数値が残る場合、未適用argumentと必要なcaptureを保持するclosure costを持ちえます。
captureしないlambdaは共有して構いません。closureの参照identityは観測できません。
local functionは参照する前方のlexical bindingだけをcaptureし、使わないbindingや後続bindingをclosure
environmentへ保持する必要はありません。local rec groupは相互参照を実装する共有environmentを持ちえます。

trait instanceは型検査時に一意に決まります。具体型が分かるcall siteはdirect callまたはspecialized codeへ
変換できます。別moduleのgeneric関数、first-class function、separate compilation境界ではdictionaryを
保持して構いません。runtime tag、constructor名、object shapeからinstanceを再探索してはなりません。

monomorphizationはcode sizeを増やすため、すべてのgeneric callへ強制しません。backendはcall頻度、code size、
別module境界を考慮してdictionary共有とspecializationを選べます。同じpublic ABIと意味を保つ限り、この
選択はprogramから観測できません。

## 14.7 data、collection、fusion

ADTはvariantを判別する情報を必要としますが、nullary constructorの共有、tagの整数化、payloadのunboxed化を
許します。record、tuple、structのfield layoutはforeign ABIを除き観測できません。不変性は毎操作で全体copyを
要求する意味ではなく、安全なstructural sharingとcopy-on-writeを許します。

Arrayはstrict・contiguous、Listはpersistent linked listという9.3の契約を保ちます。Arrayの中間結果を返す
`map`や`filter`は意味上strictです。ただし、中間値が他から参照されず、callbackの回数・順序・defectと最終結果が
同じなら、次のようなpipelineを一回の走査へfusionできます。

```seseragi
values
  |> map normalize
  |> filter isValid
  |> reduce initial combine
```

fusionはcallbackを省略、複製、並列化してはなりません。中間collectionが名前へbindされる、複数回使われる、
または各段の結果がEffectやforeign境界へ渡る場合は、その観測可能な使用を保ちます。Stream fusionはさらに
demand、buffer、overflow、cancellation、finalizerの境界を保たなければなりません。
`reduceUntil`をfusionする場合、`Done`を返した要素より後のsourceやcallbackを評価してはなりません。

## 14.8 recursionとstack safety

module-levelとlocalの直接self tail callは一定のhost call stackで実行できなければなりません。backendはloopまたはtrampolineへ
lowerします。tail positionは関数の最終式、`if`のtail branch、`match`のtail arm、tail blockの最終式を再帰的に
含みます。finalizer登録や未完了の後処理を越える呼び出しはtail positionではありません。

相互再帰と非tail recursionには一定stackを保証しません。stack exhaustionはdefectです。標準Effectの長いbind列、
`reduceUntil` / `forEachUntil`を含む標準の逐次走査、Streamの長時間consumer、Signal notification queueは、
入力件数に比例してhost call stackを増やしてはなりません。

## 14.9 Effect、Stream、Signal

Effectはcoldな計算、error channel、environment、cancellation、resource scopeを表すためruntime管理を必要とします。
実装は入れ子のhost Promiseとclosureだけへ素朴に展開する必要はなく、typed instruction、interpreter、trampoline、
specialized loopを使えます。

`map`、`flatMap`、`provide`、`mapError`の連鎖は、次を保つ範囲でまとめられます。

- Effectを構築しただけでは実行しない。
- 各user functionをsource順に一度だけ呼ぶ。
- typed failure、defect、cancellationを混同しない。
- cancellation checkpointとfinalizerのLIFO・exactly-onceを保つ。
- environment serviceのscopeを越えてlookupを移動しない。

Monad transformerは、基礎Monadと追加channelが表すcaseを保持する必要があります。transformerを選んだだけで
同じ意味の余分なwrapperを重ねる必要はありませんが、Maybe / Either / Stateなど各layerの分岐やstate伝播まで
zero-costになるとは保証しません。

Signalは一transactionにつき影響するderived nodeを高々一度再計算する5.13の保証を持ちます。compilerは
transactionをまたいで更新をdropまたはmergeしてはなりません。subscriberを持たないderived graphの解放、
Streamのbackpressure、queueのboundednessは性能最適化ではなくresource semanticsです。

通常値のmemory回収方式はbackend固有で、deterministic destructorやobject到達不能時のuser callbackを
言語semanticsにしません。file、socket、subscription、DOM listenerなどのresourceはGCへ委ねず、5.11のscopeと
finalizerで解放します。closure、slice、structural sharingが大きなbacking storageを保持しうるAPIは、その保持条件と
明示copy operationを標準ライブラリ文書へ記載します。memory exhaustionは5.10のdefectです。

## 14.10 HTMLとDOM

Htmlは不変値で参照identityを公開しないため、closedなstatic subtreeのhoist、同じnormalized propの共有、
SSR bufferへの直接出力を許します。event message、closure、Signal snapshotなどrenderごとに変わる値をcaptureする
subtreeを、静的だと推測して共有してはなりません。

DOM rendererは13.10のkey、unkeyed相対位置、focus、selection、listener lifetimeを保ちます。「最小編集回数」は
複数の同値patchがあるため言語保証にしません。少なくとも同じtag / namespaceのkeyed nodeを不必要にreplaceせず、
一つの安定Signal transactionにつき一回だけreconcileします。

Html treeのallocationが実測上支配的になった場合も、component hook、mutable virtual node、暗黙memoizationを
言語へ追加する前に、static hoist、arena、specialized renderer、incremental builderをbackend / libraryで検証します。

## 14.11 build profile

すべてのprofileは同じ言語semanticsを持ちます。

- development profileはdiagnostic、読みやすいgenerated code、source map、instrumentationを優先できる。
- release profileはdead code elimination、newtype erasure、direct call lowering、inlining、specialization、fusion、
  static data hoistを適用できる。

profileによってtyped failure、evaluation order、integer semantics、resource lifetime、foreign ABIを変えては
なりません。debug instrumentationは値identityをuserへ公開せず、無効時のprogram meaningへ影響しません。

公式TypeScript release backendの最低profileは、少なくとも次を満たします。

- type alias、型引数、pureなnewtype wrap / unwrapだけを理由とするruntime objectを生成しない。
- `$`、pipeline、`effect fn`、do notation専用のruntime objectを生成しない。
- direct self tail recursionをhost loopまたはtrampolineへlowerする。
- Effect / Streamの長い逐次chainをhost stack overflowさせない。
- source mapで最適化後のfailureとdefectを元sourceへ対応づける。

aggressive inlining、全面monomorphization、すべてのcollection fusion、最小DOM patchは最低profileに含めません。

## 14.12 検証とbenchmark

性能設計は次の三層で検証します。

1. semantic differential test: development / releaseで同じvalue、Effect trace、diagnostic originを確認する。
2. shape test: typed IRまたはgenerated module snapshotで、消去対象、tail loop、dictionary、fusion候補を確認する。
3. benchmark: wall time、allocation、throughput、latencyを同一toolchainとhost versionで継続計測する。

benchmarkの絶対値は言語conformanceではありません。baseline、toolchain、warmup、input sizeを固定し、統計的な
退行をrelease quality gateとして扱います。最低benchmark suiteは次を含みます。

- saturated call、partial application、generic trait call、specialized trait call
- Arrayのmap / filter / reduce pipelineとList traversal
- 100,000段以上のEffect bindとself tail recursion
- Signal fan-out transactionとStream backpressure
- JSON encode / decode、SSR、keyed DOM update
- BytesとTypeScript foreign境界の意図的copy

`seseragi benchmark` はbenchmark rootの `.ssrg` をcanonical module path順に列挙し、全moduleをcompileしてから
一件も実行せずdiscoveryします。正確に `pub let benchmarks: benchmark.Benchmark` を持つmoduleだけを対象にし、
full nameは `module::suite::case` です。`--filter` / `--exact`、case 0件、cancellation、resource teardownは
test runnerと同じ選択・ownership規則を使います。caseはcanonical discovery orderで実行し、同じprocess内で
caseを並列実行しません。

各caseはwarmup回をreportせず実行した後、calibrationで決めた反復回数を一sampleとしてmanifestのsamples回
測ります。sample値はbody一回あたりのnanosecondsで、monotonic clockの開始・終了とrunner loop overheadを含みます。
既定reportはmedian、median absolute deviation、minimum、maximum、sample count、iterations、input sizeを出します。
case間でGCの実行を要求できず、runnerはGC availabilityをmetadataへ記録するだけで意味を補正しません。

`--save-baseline PATH` はschema 1のJSONをatomic replaceで保存します。baselineは少なくともlanguage / compiler / runtime
version、release profile、target adapter identityとversion、OS、architecture、CPU model、logical core count、
timer resolution、manifest benchmark設定、case full name、input size、iterations、全sample値とsummaryを持ちます。
source absolute path、timestamp、hostnameは比較identityへ含めません。同じcase名の重複や非finite値は保存しません。

`--baseline PATH` は同じschemaを読み、toolchain major、release profile、target adapter、OS、architecture、CPU model、
timer resolutionが一致しなければ比較を拒否します。caseの追加はnew、削除はmissingとしてreportし、既定では
regression failureにしません。同名caseのinput sizeが違う場合は比較不能です。current medianがbaseline medianに
`regression_threshold_percent` を加えた値を厳密に超えたcaseをregressionとします。thresholdはCLIでそのrunだけ
上書きできます。少なくとも一件のregression、case failure、invalid / incompatible baselineはexit code 1、
compile / discovery / option errorは2、成功は0です。machine-readable reportはbaselineと同じschema familyを使い、
caseをcanonical discovery orderで並べます。

最適化を要求するための`inline`、ownership、borrow、unsafeなどのsource annotationは現時点で追加しません。
profileとbenchmarkで解決できない実測上の問題があり、意味論上の必要性を説明できた場合だけ、独立した言語機能として
検討します。

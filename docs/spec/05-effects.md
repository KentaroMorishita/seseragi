# 5. 失敗・Effect・副作用

## 5.1 純粋な式

通常のSeseragi関数は、引数だけから結果を計算します。外部状態の読み書き、時刻、乱数、
console、network、filesystemは通常の値を返す関数として公開できません。

回復可能な副作用と非同期処理は `Effect<R, E, A>` で表します。純粋性はannotationではなく、
利用する値の型によって保たれます。

## 5.2 Maybe

```seseragi
type Maybe<A> =
  | Nothing
  | Just A
```

`Maybe<A>` は、値が存在しないこと自体が正常な場合に使います。失敗理由が必要な場合は
`Either<E, A>` を使います。

`maybeValue ?? fallback` はMaybe専用のfallback構文です。

```seseragi
let displayName = cachedName ?? requestedName ?? "anonymous"
```

左operandは `Maybe<A>`、右operandは `A` で、結果は `A` です。右結合かつ短絡評価で、
`Just value ?? fallback` はfallbackを評価せずvalueを返し、`Nothing ?? fallback` だけがfallbackを
評価します。上の連鎖は `cachedName ?? (requestedName ?? "anonymous")` と解釈します。

`??` をEither、nullableなforeign値、通常値へoverloadしません。Eitherは `withDefault`、
foreign nullableは境界でMaybeへ変換してから扱います。

## 5.3 Either

```seseragi
type Either<E, A> =
  | Left E
  | Right A
```

Eitherは評価済みの同期結果です。`Left` と `Right` のどちらも通常の値で、生成時に追加の
計算は起きません。

## 5.4 Effect

`Effect<R, E, A>` は、環境 `R` を要求し、`E` で失敗するか `A` で成功する、遅延された
計算です。

```text
R: 必要なserviceを表すstructural record
E: 回復可能なerror
A: success値
```

```seseragi
alias AppEnv = {
  http: Http,
  clock: Clock,
}

alias App<A> = Effect<AppEnv, AppError, A>
```

Effectはcoldです。作っただけでは実行しません。同じEffectを二度実行すると計算も二度
実行します。

### `effect fn`

`effect fn` はEffectを返す関数を読みやすく宣言する糖衣です。

```seseragi
effect fn greet name: String -> Unit
with Console
fails ConsoleError =
  println $ `hello ${name}`
```

これは次の正規型を持ちます。

```seseragi
fn greet name: String
  -> Effect<{ console: Console }, ConsoleError, Unit> = ...
```

surfaceの戻り型 `Unit` はEffectのsuccess型です。`with`はenvironment requirement、`fails`は
failure型を表します。`with`を省略すると `{}`、`fails`を省略すると `Never` です。body自体は
対応するEffect型を持たなければならず、通常値を暗黙にEffectへ持ち上げません。

parameterを書かない宣言にも1.4の匿名Unit parameter規則を適用します。

```seseragi
effect fn initialize -> Unit = succeed ()
```

この型は `Unit -> Task<Never, Unit>` であり、呼び出しは `initialize ()` です。

`with Console` は標準serviceが宣言するcanonical requirement名を使う糖衣で、ここでは
`{ console: Console }` へ展開します。標準serviceにcanonical名がない場合や、同じservice型を
複数要求する場合は `with primary: Database, replica: Database` のようにfield名を明示します。
`effect fn` は型表記だけの糖衣であり、Effectの評価・実行境界を変更しません。

### effectful `for`

`for pattern <- values { body }` は、Iterableを `iterate` の順に逐次処理するEffect式です。

```seseragi
for user <- users {
  println $ user.name
}
```

patternはirrefutableでなければなりません。bodyは `Effect<R, E, Unit>`、式全体も
`Effect<R, E, Unit>` です。これは `forEach (\pattern -> body) values` へdesugarします。
空collectionは何もせず成功し、最初のfailureで停止します。`break`と`continue`はありません。
parallel実行へ自動変換せず、必要な場合は `forEachParallel` を明示します。
無限Iterator由来のIterableでは終了しません。非同期・時間的な値の列にはIterableではなくStreamを
使います。

## 5.5 environment requirement

`R` は必要なservice fieldだけを持つrecord型です。Effectを合成すると、各計算が要求する
fieldの和集合がblock全体のrequirementになります。同名fieldは同じ型でなければなりません。
Effectの第一型引数がrecord型でなければwell-formedness errorです。

```seseragi
fn program id: UserId -> Effect<{ http: Http, clock: Clock }, AppError, Report> =
  do {
    user <- Http.fetchUser id
    now <- Clock.now ()
    pure { user, generatedAt: now }
  }
```

`Effect<R1, E, A>` は、`R2` が `R1` の全fieldを同じ型で持つ場合、
`Effect<R2, E, A>` が必要な位置へrequirement wideningできます。これはEffectの `R` だけに
認める規則で、一般のgeneric型のvarianceではありません。

environmentは `Effect.provide` で渡します。余分なfieldを持つrecordもrecord width subtypingで
利用できます。service取得は `Effect.service` を使い、global singletonを暗黙参照しません。

## 5.6 error channel

`E` は回復可能な失敗です。Effectを同じdo blockで合成するにはerror型を揃えます。
異なるerror型は `mapError` でapplicationのADTへ明示変換します。任意のunion error型を
compilerが自動生成することはありません。

`recover` はfailure channelだけを扱います。defectとcancellationは捕捉しません。

## 5.7 Task

環境serviceを要求しないEffectをTaskと呼びます。

```seseragi
alias Task<E, A> = Effect<{}, E, A>
```

Taskは独立した実行意味論を持たず、Effectの特殊化です。`Task<E, _>` に対する
Functor、Applicative、Monad操作は `Effect<{}, E, _>` のinstanceです。

```seseragi
fn loadProfile id: UserId -> Task<HttpError, Profile> =
  do {
    user <- fetchUser id
    fetchProfile user
  }
```

## 5.8 順次・並列実行

`flatMap`、`>>=`、do blockはsource順に逐次実行します。式を並べただけでは並列になりません。
並列実行は `Effect.parallel` で明示します。

parallelの結果順、failure選択、cancellationは標準ライブラリ契約に従います。並列化により
外部操作の順序が変わるため、compilerが逐次Effectを自動並列化してはなりません。

## 5.9 実行境界

Effectを実行する操作は通常のSeseragi codeへ公開しません。hostは `main`、test runner、
callback adapterなどの明示された境界でだけEffectを実行します。

実行時にhostはenvironmentを一度組み立て、`R` の全serviceを満たすことを型検査済みwrapperで
保証します。entry pointの `R` にあるserviceをhost targetが提供できない場合、packageの
実行準備をコンパイルまたは起動前に拒否します。Effect実行中にservice lookupを失敗させません。

## 5.10 defect

回復可能な失敗と、program継続を保証できないdefectを区別します。

defectの例:

- 整数overflowとzero division
- compilerまたはruntimeの内部不変条件違反
- memory exhaustion
- safe wrapperの契約に違反したforeign値

defectはEitherやEffectのfailure channelへ暗黙変換されません。通常codeからcatchできません。
入力や外部失敗として想定できるものをdefectにしてはなりません。

## 5.11 cancellationとresource

Effectはcooperative cancellationを持ちます。hostがcancellationを要求すると、未完了のEffectは
子Effectへ伝播し、登録されたfinalizerをLIFOで一度だけ実行します。

resourceは `Effect.bracket acquire use release` で扱います。`release` はsuccess、failure、
cancellationのすべてで実行します。cancellationは `E` の値ではありません。

### scheduler fairness

Effect runtimeはcooperative schedulerを使います。core APIはFiber priorityを公開せず、runnable Fiberは
FIFO queueへ入ります。継続的にrunnableなFiberは、ほかのFiberがcheckpointへ到達し続ける限り
最終的に再実行されます。
これをweak fairnessとします。wall-clock時間や実行step数の均等性は保証しません。

async wait、`yieldNow`、Queue / Deferred / Semaphoreの待機、sleep、I/O境界はcheckpointです。
cancellationもcheckpointで観測します。Effectを作らない長時間のpure計算や無限loopは自動preempt
されず、ほかのFiberをstarveさせられます。そのような計算は明示的に分割して `yieldNow` を挟みます。

同じscheduler turnで複数Fiberをrunnableにするoperationは、入力順またはwaiter登録順でqueueへ
追加します。runtimeのthread数が違っても、仕様で定めたresult順、failure選択、resource解放順を
変えてはなりません。

### Fiber supervision

すべてのFiberはroot `main` scopeまたは最も内側の `scoped` resource scopeに所属します。`fork` は
現在scopeのchildを作り、unscoped daemon Fiberを作りません。

scope終了時は未完了childすべてへcancellationを要求し、各childのfinalizer完了を待ってからscopeを
閉じます。parentのsuccess、failure、cancellationのどの場合も同じです。childのtyped failureは
Fiberに保存され、`join` しない限りparentのerror channelへ暗黙伝播しません。test runtimeは
未観測failureをdiagnosticにできますが、programのtyped resultを変更しません。

`join` はchildのsuccessを返し、typed failureを呼び出し側のfailureにし、child cancellationなら
join中のFiberもcancellationされます。cancellationを値として調べる場合は `await` で `FiberExit` を
取得します。joinまたはawaitしているFiber自身がcancelされた場合は待機だけを解除し、target Fiberを
cancelしません。`interrupt` はidempotentで、対象へcancellationを要求しfinalizer完了まで待ちます。

## 5.12 Signal

Signalは時間とともに変化する値を、型付きのdependency graphとして表します。

```text
Signal<A>         読み取り専用のsignal
MutableSignal<A>  setできるsource signal。Signal<A>としても使える
```

`MutableSignal<A>` は `Signal<A>` へ暗黙に読み取り権限を落とせます。逆変換はできません。
これはSignal固有のcoercionで、一般のgeneric subtypingではありません。

生成、読み取り、更新、購読はruntime stateへ触るためEffectを返します。pure codeがSignalを
持つことはできますが、現在値を暗黙に読むことはできません。

```seseragi
fn make<A> initial: A -> Task<Never, MutableSignal<A>>
fn read<A> signal: Signal<A> -> Task<Never, A>
fn set<A> value: A -> signal: MutableSignal<A> -> Task<Never, Unit>
fn update<A> f: (A -> A) -> signal: MutableSignal<A> -> Task<Never, Unit>

fn planSet<A> value: A -> signal: MutableSignal<A> -> SignalChange
fn planUpdate<A> f: (A -> A) -> signal: MutableSignal<A> -> SignalChange
fn transaction changes: Array<SignalChange> -> Task<Never, Unit>
```

`SignalChange` は `std/signal` が公開するstandard opaque typeです。異なるvalue型のchangeを同じArrayへ
安全に入れられますが、target signalやvalueを取り出すoperationは公開しません。

`set` と `update` はそれぞれ一件のtransactionです。複数sourceをatomicに変える場合は
`planSet` / `planUpdate` を適用したい順にArrayへ並べ、`transaction`へ渡します。同じsignalへのchangeは
Array順に適用し、planUpdateの関数はそれ以前にstagingされた値を受け取ります。

changeの関数評価中にdefectが起きた場合、またはcommit前にcancellationされた場合は、一件も公開せず
元の値を保ちます。全changeの計算成功後に一度だけcommitし、derived graphの再計算とsubscriber通知が
完了するまではcancellationを観測しません。transactionはI/Oや任意Effectを内包せず、Signal更新だけを
atomicにします。

## 5.13 derived Signal

derived Signalはpure関数で構築します。

```seseragi
fn map<A, B> f: (A -> B) -> source: Signal<A> -> Signal<B>
fn combine<A, B, C>
  f: (A -> B -> C) -> left: Signal<A> -> right: Signal<B> -> Signal<C>
fn constant<A> value: A -> Signal<A>
fn distinct<A> source: Signal<A> -> Signal<A>
where Eq<A>
fn switchMap<A, B>
  f: (A -> Signal<B>) -> source: Signal<A> -> Signal<B>
```

dependencyはmap/combineなどのconstructorから明示的に決まります。関数実行中のreadを追跡する
暗黙dependency trackingはありません。

更新はtransactionです。一つのsource更新に対して、影響するderived Signalをtopological
orderで高々一度再計算し、graph全体が安定してからsubscriberを呼びます。subscriberが一時的に
不整合な組み合わせを観測するglitchを許しません。

同じ値をsetしても既定では更新として通知します。重複除去は `std/signal` の `distinct` とEq instanceで
明示します。

SignalはFunctorとApplicative instanceを持ちます。Functorのmapは上の `map`、Applicativeのpureは
`constant`、applyはglitch-freeなcombineです。constantはsource dependencyを持たず、値を変更しません。

`distinct` はsourceの現在値を初期値とし、Eqで直前の公開値と等しい更新を通知しません。
`switchMap` はsource値ごとに `f` を一度評価し、選ばれたinner Signalだけをdependencyにします。
source更新時は旧innerを解除して新innerへ切り替え、安定した新innerの現在値を一度公開します。
この切替は同じtransaction内で行い、旧innerと新innerが混ざった値をsubscriberへ見せません。

## 5.14 `*` と `:=`

`*source` はSignalの現在値をsnapshotとして読む固定prefix operatorです。

```seseragi
do {
  current <- *count
  Console.println `count: ${current}`
}
```

`*source` は `Signal.read source` へdesugarし、`source: Signal<A>`または
`MutableSignal<A>`に対して型 `Task<Never, A>` を持ちます。`A`を直接返すpureなdereferenceでは
ありません。したがって、通常のpure式でSignalの現在値が暗黙に変化することはありません。
二項の`*`は乗算のままで、prefix位置の`*`だけがSignal readです。

`target := value` はMutableSignalだけに使える固定operatorです。

```seseragi
do {
  count := 1
  count.update (\current -> current + 1)
}
```

`target := value` は `Signal.set value target` へdesugarし、型は `Task<Never, Unit>` です。
通常変数やstruct fieldへの代入には使えません。Signalの現在値を暗黙に読み込まないため、
現在値に基づく更新はatomicな `update` を使います。

## 5.15 subscribeとlifetime

```seseragi
fn subscribe<R, A>
  observer: (A -> Effect<R, Never, Unit>)
  -> signal: Signal<A>
  -> Effect<R, Never, Subscription>
fn unsubscribe subscription: Subscription -> Task<Never, Unit>
```

subscribe成功時、observerを現在値で一度呼び、その後の更新を順番に通知します。同じSignalの
observerは登録順に実行し、一つのobserverの実行中に次の通知を重ねません。

observerがSignalを更新した場合、その更新は現在transaction終了後の新しいtransactionへqueue
します。同期re-entrancyは発生しません。

observerはfailureを外へ残せません。fallibleな処理はobserver内でrecoverするか、SignalをStreamへ
変換してfailure channelを持つconsumerで処理します。defectが起きたobserverはruntimeの
defect policyに従って停止し、他のsubscriberへ通常failureとして伝播しません。

Subscriptionは `std/signal` が公開するstandard opaque typeで、idempotentにunsubscribeできます。
Effectのresource scopeへ登録したsubscriptionは
scope終了、failure、cancellationのいずれでも解除します。subscriberを持たないderived Signalは
dependency購読を解放できなければなりません。

SignalはFunctorとApplicative instanceを持ちますが、Monad instanceは持ちません。動的な
dependency切替はlifetime semanticsを伴うため、`switchMap` など名前付きresource operationで
明示します。

## 5.16 例外とalgebraic effect

Seseragiにthrow / catch / try構文はありません。外部例外はinterop adapterが捕捉し、
`Either<ForeignError, A>` または `Effect<R, ForeignError, A>` に変換します。

`perform` / `handle` によるalgebraic effect構文もありません。Effectのenvironment、typed error、
resource、cancellationと役割が重なる別のeffect systemを同時に導入しません。

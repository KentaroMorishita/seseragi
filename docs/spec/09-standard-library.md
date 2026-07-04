# 9. 標準ライブラリの契約

この章のfunction code blockは公開signatureを示し、実装本体を省略しています。

## 9.1 core prelude

preludeは自動importされ、次を提供します。

- `Unit`, `Never`, `Bool`, `Int`, `Float`, `String`
- `Maybe`, `Either`, `Array`, `List`, `Effect`, `Task`
- それぞれのconstructor
- `Eq`, `Ord`, `Show`, `Debug`, `Semigroup`, `Monoid`
- `Functor`, `Applicative`, `Monad`, `Reducible`
- 算術operator用trait
- trait methodとしての `map`, `apply`, `pure`, `flatMap`, `reduce`
- `identity`, `const`, `compose`, `flip`, `show`, `sum`, `product`
- `Console`, `ConsoleError`, `print`, `println`, `printValue`

preludeの名前は明示importでshadowingできません。local bindingではshadowingできます。

## 9.2 MaybeとEither

型を保った `map`、`apply`、`flatMap` は、Functor、Applicative、Monadのtrait methodを使います。
Maybe/Either moduleが最低限追加する固有操作は次です。

```seseragi
fn withDefault<A> fallback: A -> value: Maybe<A> -> A

fn mapLeft<E, F, A> f: (E -> F) -> value: Either<E, A> -> Either<F, A>
fn mapRight<E, A, B> f: (A -> B) -> value: Either<E, A> -> Either<E, B>
```

`maybeValue ?? fallback` は `withDefault fallback maybeValue` と同じ結果を持ちますが、fallbackを
必要になるまで評価しない構文です。

`Maybe` と `Either<E, _>` はFunctor、Applicative、Monad instanceを持ちます。Eitherは最初の
`Left` を保ちます。validation errorの蓄積は別の `Validation` 型で提供し、Eitherの
Applicativeの意味を変えません。

Functor、Applicative、Monadはこの順のsupertrait関係を持ちます。Monad instanceは同じ
型構築子に対するApplicativeとFunctorのinstanceも必要です。

## 9.3 collection

Arrayはstrict・contiguous・indexed collection、Listはpersistent linked listです。両方に
`map`、`filter`、`reduce`、`length`、`isEmpty` を提供します。
`reduce initial step values` は先頭から末尾へ処理します。initialは必須で、空collectionでは
initialをそのまま返します。initialを省略して空collectionで失敗するoverloadは提供しません。
逆方向が必要なcodeはcollection moduleの明示的な`reduceRight`を使いますが、通常の集約の
標準形にはしません。

partial functionは標準APIにしません。

- `head: List<A> -> Maybe<A>`
- `tail: List<A> -> Maybe<List<A>>`
- `get: Int -> Array<A> -> Maybe<A>`
- `find: (A -> Bool) -> Array<A> -> Maybe<A>`

Array/List間の変換は `Array.toList` と `List.toArray` です。変換順序を保ち、暗黙には
呼び出されません。comprehension、literal、patternの意味は3.7と3.8に従います。

tuple、record、Array、Listには、全要素がEqを満たす場合の構造的なEq instanceを提供します。
nominalなstructとADTのEqは、型を定義したmoduleが明示的なimplまたはderivingでinstanceを
与えます。

## 9.4 EqとOrd

```seseragi
type Ordering =
  | Less
  | Equal
  | Greater

trait Ord<A>
where Eq<A> {
  fn compare left: A -> right: A -> Ordering
}
```

`Eq.eq` は反射律・対称律・推移律を満たさなければなりません。Floatの標準EqはIEEE 754の
NaNにより反射律を満たせないため提供しません。Floatの比較は `Float.ieeeEq`、
`Float.totalCompare` など、意味を示す名前付き関数を使います。

Ordは全順序です。`compare` は `Less | Equal | Greater` を返し、Eqと整合しなければ
なりません。

```seseragi
trait Hash<A> {
  fn hash value: A -> Int
}
```

HashはMapとSetのkeyに使うhashを定義します。`Eq.eq x y` がTrueなら
`Hash.hash x == Hash.hash y` でなければなりません。逆は要求しません。

## 9.5 SemigroupとMonoid

```seseragi
trait Semigroup<A> {
  fn append left: A -> right: A -> A
}

trait Monoid<A>
where Semigroup<A> {
  fn empty unit: Unit -> A
}
```

Semigroupは結合律、Monoidは左単位元・右単位元を満たします。custom operatorの実装から
これらを利用できますが、特定のoperator symbolをMonoidへ固定しません。

```seseragi
fn combine<F<_>, A> values: F<A> -> A
where Reducible<F>, Monoid<A>
```

`combine` はcollectionをその要素型のMonoidで結合します。変換してから結合する場合も、
専用の `foldMap` という名前へ圧縮せず、`map` と意味を表すreducerをpipelineで組み合わせます。

```seseragi
numbers |> sum

orders
  |> map (\order -> order.total)
  |> sum

numbers |> reduce 0 (+)
```

標準reducerとして少なくとも `sum`、`product`、`combine`、`any`、`all`、`join` を提供します。
用途名で読める場合はgenericな `reduce` より用途名を優先します。

String、List、Arrayは連結をMonoidとして持ちます。`Maybe<A>` は `Semigroup<A>` がある場合、
Nothingを単位元とするMonoidを持ちます。

IntにはcanonicalなMonoid instanceを定義しません。加算と乗算のどちらも妥当でcoherenceに
反するため、nominal wrapperで選びます。

```seseragi
type Sum<A> = | Sum A
type Product<A> = | Product A
```

数値の `Sum` は加算、`Product` は乗算のMonoid instanceを持ちます。WriterTの `W` もこの
Monoid契約でlogを結合します。旧来の特別な `monoid` 宣言や `>>>` dispatchはありません。

## 9.6 数値演算

Intの `+`, `-`, `*`, `**` は結果が64 bit範囲外ならdefectになります。Intの `/` は0方向へ
丸め、`%` は被除数と同じ符号の余りを返します。0による `/` と `%`、負の指数によるIntの
`**` はdefectです。回復可能に扱うcodeは `Int.checkedDiv` などのchecked APIを使います。

Floatの演算はIEEE 754に従います。NaNとinfinityは有効なFloat値です。Floatは標準のEqと
Ord instanceを持ちません。

Boolに対する `&&`, `||`, `!` は型クラス演算ではなく、言語組み込みです。

## 9.7 Functor、Applicative、Monad

標準ライブラリは少なくとも次のFunctor instanceを提供します。

- `Maybe`
- `Either<E, _>`
- `Array`
- `List`
- `Effect<R, E, _>`。`Task<E, _>` はそのaliasとして同じinstanceを使う
- `Signal`

各instanceは同じ `map` の型schemeを実装します。concrete型ごとのruntime helperを使うことは
できますが、それらはbackend内部であり、sourceから型別helperを選ぶ必要はありません。
ApplicativeとMonadの標準instanceは各型のfailure、順序、組み合わせ方を個別に定義した場合だけ
提供します。FunctorであることだけからMonadを自動生成しません。

lawは次のとおりです。Eqで観測できる範囲で成立しなければなりません。

- Functor identity: `map identity x == x`
- Functor composition: `map (compose f g) x == map f (map g x)`
- Applicative identity、homomorphism、interchange、composition
- Monad left identity、right identity、associativity

`Effect<R, E, _>` の等価性は、同じenvironmentとobserverに対するsuccess値、failure値、
外部操作順で定義します。

## 9.8 Effect

最低限、次を提供します。

以下の `R` はstructural record型だけを量化するmeta-variableです。

```seseragi
fn succeed<R, E, A> value: A -> Effect<R, E, A>
fn fail<R, E, A> error: E -> Effect<R, E, A>
fn mapError<R, E, F, A>
  f: (E -> F) -> effect: Effect<R, E, A> -> Effect<R, F, A>
fn recover<R, E, F, A>
  f: (E -> Effect<R, F, A>)
  -> effect: Effect<R, E, A>
  -> Effect<R, F, A>
fn provide<R, E, A>
  environment: R -> effect: Effect<R, E, A> -> Task<E, A>
fn parallel<R, E, A>
  effects: Array<Effect<R, E, A>> -> Effect<R, E, Array<A>>
fn forEach<F<_>, R, E, A>
  action: (A -> Effect<R, E, Unit>)
  -> values: F<A>
  -> Effect<R, E, Unit>
where Iterable<F>
```

異なるenvironment requirementを合成するときは、型検査器が5.5のrequirement wideningで
共通のrecord型へ揃えてから、上のsame-`R` operationを適用します。

parallelの結果順は入力順です。最初に観測したfailureを返して残りをcancelします。同じ
scheduler tickで複数failureを観測した場合は、入力indexが小さいものを返します。

`forEach` は先頭から末尾へ逐次実行し、各actionが成功してから次へ進みます。空collectionは
何もせず成功します。最初のfailureで停止し、後続actionを開始しません。結果を収集する操作は
`traverse`、並列実行は `forEachParallel` と別名で提供します。

Task moduleは `Effect<{}, E, A>` にspecializeした同名operationを提供します。
`map`と`flatMap`はEffect固有の別関数ではなく、FunctorとMonadのtrait methodです。

## 9.9 monad transformer

標準ライブラリは次のnominal transformerを個別moduleで提供します。

```seseragi
struct MaybeT<M<_>, A> {
  run: M<Maybe<A>>,
}

struct EitherT<E, M<_>, A> {
  run: M<Either<E, A>>,
}

struct ReaderT<R, M<_>, A> {
  run: R -> M<A>,
}

struct StateT<S, M<_>, A> {
  run: S -> M<(A, S)>,
}

struct WriterT<W, M<_>, A> {
  run: M<(A, W)>,
}
```

各transformerは `Monad<M>` constraintのもとでFunctor、Applicative、Monad instanceを持ち、
base monadを持ち上げる `lift` を自身のmoduleで提供します。WriterTはさらに `Monoid<W>` を
要求します。

transformerの順序は意味を持ちます。StateTの外側にEitherを置くstackと、EitherTの外側に
Stateを置くstackを自動変換しません。failure時にstateを保持するか捨てるかなどの違いを、
選んだstackが決めます。

通常のapplication codeでは、具体stackをaliasしてdo notationを使います。

```seseragi
alias AppValidation<A> = EitherT<ValidationError, Effect<AppEnv, AppError, _>, A>
```

Effectのenvironmentとerror channelで足りる場合、ReaderTやEitherTを重ねる必要はありません。
MaybeT、StateT、WriterTなど、追加のcomposition semanticsが必要な場合だけ使います。

## 9.10 resourceとSignal

`Effect.bracket` はacquire、use、releaseを受け、success、failure、cancellationのすべてで
releaseを一度だけ実行します。

Signal moduleは `make`、`read`、`set`、`update`、`map`、`combine`、`distinct`、`subscribe`、
`unsubscribe`、`switchMap` を提供します。正確なtransactionとlifetime semanticsは5.12から
5.15に従います。prefix `*` は `read`、`:=` は `set` の固定構文糖です。

## 9.11 ShowとDebug

`Show<A>` はuser-facingな安定した文字列表現です。

```seseragi
trait Show<A> {
  fn show value: A -> String
}
```

`show` は純粋で、I/O、throw、global locale参照を行いません。localeやformat optionが必要な
表示は引数を取る名前付きformatterとして定義します。

backtick templateのinterpolationは `Show` を使います。たとえば
`` `user: ${user}` `` は概念上 `"user: " <> show user` へ展開されます。templateは
compilerが構文として検査しますが、`show`自体は通常のtrait methodであり、compiler builtinや
マクロではありません。

`Debug<A>` はdeveloper向け表現です。compiler/runtime version間で文字列互換性を保証せず、
protocol、永続化、snapshot formatへ使ってはなりません。secretを含む型はDebugを実装しないか、
明示的にredactします。

Show/Debugと出力先は別概念です。型classを実装しただけで値がconsoleへ書かれることは
ありません。

## 9.12 Console

Consoleはhostがenvironmentへ提供するserviceです。`print` をcompiler builtinや
`console.log` への特別loweringとして扱いません。

```seseragi
fn print text: String
  -> Effect<{ console: Console }, ConsoleError, Unit>

fn println text: String
  -> Effect<{ console: Console }, ConsoleError, Unit>

fn printValue<A> value: A
  -> Effect<{ console: Console }, ConsoleError, Unit>
where Show<A>

fn flush unit: Unit
  -> Effect<{ console: Console }, ConsoleError, Unit>
```

`print` は文字列をそのままstdoutへ書き、改行を加えません。`println` は末尾へ `\n` を一つ
加えます。`printValue value` は `print (show value)` です。stderr出力は
`Console.error` / `Console.errorLine` を使います。

同じdo block内のConsole Effectはsource順に実行します。buffering、terminal encoding、
broken pipeなどhost由来の失敗はConsoleErrorです。test hostはConsole serviceを差し替えて
出力を値として検証できます。

## 9.13 structured logging

application logはConsole文字列出力と分け、Logger serviceを使います。

```seseragi
type LogLevel =
  | LogTrace
  | LogDebug
  | LogInfo
  | LogWarn
  | LogFailure

type LogValue =
  | LogString String
  | LogInt Int
  | LogFloat Float
  | LogBool Bool

struct LogEvent {
  level: LogLevel,
  message: String,
  fields: List<(String, LogValue)>,
}

fn log event: LogEvent
  -> Effect<{ logger: Logger }, LogError, Unit>
```

実際のtimestamp、JSON化、送信先、batchingはLogger implementationの責務です。library codeが
直接consoleへ書かず、必要なLogger requirementをEffectの `R` に現します。

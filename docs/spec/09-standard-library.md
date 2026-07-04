# 9. 標準ライブラリの契約

この章のfunction code blockは公開signatureを示し、実装本体を省略しています。

## 9.1 core prelude

preludeは自動importされ、次を提供します。

- `Unit`, `Never`, `Bool`, `Int`, `Float`, `String`
- `Maybe`, `Either`, `Array`, `List`, `Effect`, `Task`
- それぞれのconstructor
- `Eq`, `Ord`, `Show`, `Debug`, `Semigroup`, `Monoid`
- `Functor`, `Applicative`, `Monad`
- 算術operator用trait
- `identity`, `const`, `compose`, `flip`, `show`
- `Console`, `ConsoleError`, `print`, `println`, `printValue`

preludeの名前は明示importでshadowingできません。local bindingではshadowingできます。

## 9.2 MaybeとEither

最低限、次の関数を提供します。

```seseragi
fn map<A, B> f: (A -> B) -> value: Maybe<A> -> Maybe<B>
fn flatMap<A, B> f: (A -> Maybe<B>) -> value: Maybe<A> -> Maybe<B>
fn withDefault<A> fallback: A -> value: Maybe<A> -> A

fn mapLeft<E, F, A> f: (E -> F) -> value: Either<E, A> -> Either<F, A>
fn mapRight<E, A, B> f: (A -> B) -> value: Either<E, A> -> Either<E, B>
fn flatMap<E, A, B>
  f: (A -> Either<E, B>) -> value: Either<E, A> -> Either<E, B>
```

`Maybe` と `Either<E, _>` はFunctor、Applicative、Monad instanceを持ちます。Eitherは最初の
`Left` を保ちます。validation errorの蓄積は別の `Validation` 型で提供し、Eitherの
Applicativeの意味を変えません。

Functor、Applicative、Monadはこの順のsupertrait関係を持ちます。Monad instanceは同じ
型構築子に対するApplicativeとFunctorのinstanceも必要です。

## 9.3 collection

Arrayはstrict・contiguous・indexed collection、Listはpersistent linked listです。両方に
`map`、`filter`、`foldLeft`、`foldRight`、`length`、`isEmpty` を提供します。

partial functionは標準APIにしません。

- `head: List<A> -> Maybe<A>`
- `tail: List<A> -> Maybe<List<A>>`
- `get: Int -> Array<A> -> Maybe<A>`
- `find: (A -> Bool) -> Array<A> -> Maybe<A>`

tuple、record、Array、Listには、全要素がEqを満たす場合の構造的なEq instanceを提供します。
nominalなstructとADTのEqは自動生成せず、型を定義したmoduleが明示的にinstanceを与えます。

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
fn combineAll<A> values: List<A> -> A
where Monoid<A>

fn foldMap<F<_>, A, M> f: (A -> M) -> values: F<A> -> M
where Foldable<F>, Monoid<M>
```

String、List、Arrayは連結をMonoidとして持ちます。`Maybe<A>` は `Semigroup<A>` がある場合、
Nothingを単位元とするMonoidを持ちます。

IntにはcanonicalなMonoid instanceを定義しません。加算と乗算のどちらも妥当でcoherenceに
反するため、nominal wrapperで選びます。

```seseragi
struct Sum<A> { value: A }
struct Product<A> { value: A }
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
fn map<R, E, A, B>
  f: (A -> B) -> effect: Effect<R, E, A> -> Effect<R, E, B>
fn flatMap<R, E, A, B>
  f: (A -> Effect<R, E, B>)
  -> effect: Effect<R, E, A>
  -> Effect<R, E, B>
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
```

異なるenvironment requirementを合成するときは、型検査器が5.5のrequirement wideningで
共通のrecord型へ揃えてから、上のsame-`R` operationを適用します。

parallelの結果順は入力順です。最初に観測したfailureを返して残りをcancelします。同じ
scheduler tickで複数failureを観測した場合は、入力indexが小さいものを返します。

Task moduleは `Effect<{}, E, A>` にspecializeした同名operationを提供します。

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
5.15に従います。`:=` は `set` の固定構文糖です。

## 9.11 ShowとDebug

`Show<A>` はuser-facingな安定した文字列表現です。

```seseragi
trait Show<A> {
  fn show value: A -> String
}
```

`show` は純粋で、I/O、throw、global locale参照を行いません。localeやformat optionが必要な
表示は引数を取る名前付きformatterとして定義します。

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

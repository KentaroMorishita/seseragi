# 9. 標準ライブラリの契約

この章のfunction code blockは公開signatureを示し、実装本体を省略しています。

## 9.1 core prelude

preludeは自動importされ、次を提供します。

- `Unit`, `Never`, `Bool`, `Int`, `Float`, `Char`, `String`
- `Maybe`, `Either`, `Array`, `List`, `Iterator`, `Effect`, `Task`, `Signal`, `MutableSignal`
- 公開constructorを持つ型について、そのconstructor
- `Eq`, `Ord`, `Show`, `Debug`, `Zero`, `One`, `Semigroup`, `Monoid`, `JsonEncode`, `JsonDecode`
- `Functor`, `Applicative`, `Monad`, `Iterable`, `Reducible`, `Traversable`
- 算術operator用trait
- trait methodとしての `map`, `apply`, `pure`, `flatMap`, `iterate`, `reduce`, `traverse`
- `identity`, `const`, `compose`, `flip`, `show`, `encodeJson`, `decodeJson`, `todo`
- `sum`, `product`, `combine`, `any`, `all`, `join`
- `Console`, `ConsoleError`, `print`, `println`, `printValue`
- `Stdin`, `StdinError`, `readLine`
- `Random`, `Entropy`

preludeの名前は明示importでshadowingできません。local bindingではshadowingできます。

`todo` はtoolingが未実装branchのplaceholderへ使うstandard symbolです。

```text
todo : forall A. String -> A
```

型推論上は任意の期待型Aを満たしますが、standard todo symbolへの参照が一つでも残るmoduleは
`SES-T0001` Errorでbuildできません。runtime panicへlowerせず、release artifactへ混入できません。local bindingが
todoをshadowしている位置へcode actionを適用する場合、toolはcollision-freeなnamespace aliasで `std/prelude` を
importし、そのqualified todoを使います。

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

Array/List間の変換は `std/array` の `toList` と `std/list` の `toArray` です。変換順序を保ち、
暗黙には呼び出されません。comprehension、literal、patternの意味は3.8と3.9に従います。

tuple、record、Array、Listには、全要素がEqを満たす場合の構造的なEq instanceを提供します。
nominalなstruct、ADT、newtypeのEqは、型を定義したmoduleが明示的なimplまたはderivingでinstanceを
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
NaNにより反射律を満たせないため提供しません。Floatの比較は `std/float` の `ieeeEq`、
`totalCompare` など、意味を示す名前付き関数を使います。

Ordは全順序です。`compare` は `Less | Equal | Greater` を返し、Eqと整合しなければ
なりません。

```seseragi
trait Hash<A> {
  fn hash value: A -> Int
}
```

HashはMapとSetのkeyに使うhashを定義します。`Eq.eq x y` がTrueなら
`Hash.hash x == Hash.hash y` でなければなりません。逆は要求しません。

Int、BigInt、Decimal、Bool、Char、String、Unitには標準Eq、Ord、Hash instanceを提供します。Boolは
`False < True`、Charはcode point順、StringはUnicode scalar列の辞書式順序、Unitは唯一の値をEqualとします。
StringのEqはUnicode scalar列の完全一致です。FloatにはcanonicalなEq、Ord、Hash instanceを
提供しません。

## 9.5 SemigroupとMonoid

```seseragi
trait Semigroup<A> {
  fn append left: A -> right: A -> A
}

trait Monoid<A>
where Semigroup<A> {
  fn empty -> A
}

trait Zero<A> {
  fn zero -> A
}

trait One<A> {
  fn one -> A
}
```

Semigroupは結合律、Monoidは左単位元・右単位元を満たします。custom operatorの実装から
これらを利用できますが、特定のoperator symbolをMonoidへ固定しません。

parameterなし関数の規則はtrait methodにも適用します。したがって `empty` の型は `Unit -> A` で、
呼び出しは `empty ()` です。Monoid lawの単位元はこの呼び出し結果を指します。
`zero` と `one` も同様に `zero ()`、`one ()` と呼びます。trait value memberという別構文は
導入しません。

ZeroとOneは数値集約の単位元だけを表し、SemigroupやMonoid instanceを暗黙に作りません。
Int、Float、BigInt、DecimalはZeroとOneを持ちます。Floatのzeroはpositive zero、oneはpositive
oneです。IEEE 754演算は結合律やbit-levelの単位元lawを満たさないため、FloatへMonoidを導出しません。

```seseragi
fn combine<C, A> values: C -> A
where Reducible<C, A>, Monoid<A>

fn sum<C, A> values: C -> A
where Reducible<C, A>, Zero<A>, Add<A, A, A>

fn product<C, A> values: C -> A
where Reducible<C, A>, One<A>, Mul<A, A, A>

fn any<C, A> predicate: (A -> Bool) -> values: C -> Bool
where Iterable<C, A>

fn all<C, A> predicate: (A -> Bool) -> values: C -> Bool
where Iterable<C, A>

fn join<C> separator: String -> values: C -> String
where Reducible<C, String>
```

`combine` は `empty ()` をinitialにして、collectionをその要素型のMonoidでsource順に結合します。
変換してから結合する場合も、
専用の `foldMap` という名前へ圧縮せず、`map` と意味を表すreducerをpipelineで組み合わせます。
`sum` はzeroから加算、`product` はoneから乗算をsource順に行います。`any` は最初のTrue、`all` は
最初のFalseでshort-circuitします。空collectionではそれぞれFalseとTrueです。`join` は要素間だけに
separatorを挿入し、空collectionでは空Stringを返します。

```seseragi
numbers |> sum

orders
  |> map (\order -> order.total)
  |> sum

numbers |> reduce 0 (+)
```

標準集約として少なくとも `sum`、`product`、`combine`、`any`、`all`、`join` を提供します。
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
`**` はdefectです。回復可能に扱うcodeは `std/int.checkedDivide` などのchecked APIを使います。

BigIntの `+`, `-`, `*` は任意精度でoverflowしません。`/` は0方向へ丸め、`%` は被除数と同じ符号の
余りを返します。0による `/` と `%`、負のInt指数によるBigIntの `**` はdefectです。回復可能に扱う
codeは`std/big-int`のchecked APIを使います。IntとBigIntの暗黙変換やmixed arithmetic instanceはありません。

Floatの演算はIEEE 754に従います。NaNとinfinityは有効なFloat値です。Floatは標準のEqと
Ord instanceを持ちません。

Boolに対する `&&`, `||`, `!` は型クラス演算ではなく、言語組み込みです。

## 9.7 Functor、Applicative、Monad

標準ライブラリは少なくとも次のFunctor instanceを提供します。

- `Maybe`
- `Either<E, _>`
- `Array`
- `List`
- `NonEmptyList`
- `Effect<R, E, _>`。`Task<_>` は `Effect<{}, Never, _>` のaliasとして同じinstanceを使う
- `Signal`
- `Stream<R, E, _>`
- `Validation<E, _>`

各instanceは同じ `map` の型schemeを実装します。concrete型ごとのruntime helperを使うことは
できますが、それらはbackend内部であり、sourceから型別helperを選ぶ必要はありません。
ApplicativeとMonadの標準instanceは各型のfailure、順序、組み合わせ方を個別に定義した場合だけ
提供します。FunctorであることだけからMonadを自動生成しません。

Maybe、Either、Array、List、NonEmptyList、Effect、StreamはApplicativeとMonadを持ちます。
ValidationとSignalはApplicativeを持ちますがMonadを持ちません。

- Array / List / NonEmptyListの `pure` はsingletonです。`apply functions values` はfunctionを
  source順、その内側でvalueをsource順に走査するCartesian productです。`flatMap` は各結果を
  source順に連結します。NonEmptyListの結果は常にnon-emptyです。
- EitherのApplicativeとMonadは最初のLeftで停止します。ValidationのApplicativeだけが10.4の規則で
  独立したerrorを蓄積します。依存する次のvalidationを選ぶMonadは提供しません。
- Effectの `apply wrapped value` はwrappedを完了してからvalueを実行する左から右の逐次合成です。
  Applicativeで依存関係がないことと、runtimeで並列に走ることは別です。並列実行は10.11の
  `parallel` / `traverseParallel` で明示します。
- Streamのpure、apply、flatMapは10.12のcold・sequential semanticsに従います。applyは
  `wrapped |> flatMap (\f -> values |> map f)` と同じで、各functionについてvaluesを新しく実行します。
- Signalのpure/applyは5.13、動的dependency切替をtrait methodにしない理由は5.15に従います。

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
fn defer<R, E, A>
  thunk: (Unit -> Effect<R, E, A>) -> Effect<R, E, A>
fn mapError<R, E, F, A>
  f: (E -> F) -> effect: Effect<R, E, A> -> Effect<R, F, A>
fn recover<R, E, F, A>
  f: (E -> Effect<R, F, A>)
  -> effect: Effect<R, E, A>
  -> Effect<R, F, A>
fn provide<R, E, A>
  environment: R -> effect: Effect<R, E, A> -> Effect<{}, E, A>
fn parallel<R, E, A>
  effects: Array<Effect<R, E, A>> -> Effect<R, E, Array<A>>
fn forEach<C, R, E, A>
  action: (A -> Effect<R, E, Unit>)
  -> values: C
  -> Effect<R, E, Unit>
where Iterable<C, A>

type LoopControl =
  | Continue
  | Break

fn forEachUntil<C, R, E, A>
  action: (A -> Effect<R, E, LoopControl>)
  -> values: C
  -> Effect<R, E, Unit>
where Iterable<C, A>
```

`defer thunk` はEffectが実行されるたびにthunkを一度呼び、その時点で返されたEffectを同じscopeで実行します。
Effect値の構築時にはthunkを呼びません。反復可能なEffectでstrictなpure計算も毎回作り直す場合に使います。

異なるenvironment requirementを合成するときは、型検査器が5.5のrequirement wideningで
共通のrecord型へ揃えてから、上のsame-`R` operationを適用します。

parallelの結果順は入力順です。最初に観測したfailureを返して残りをcancelします。同じ
scheduler tickで複数failureを観測した場合は、入力indexが小さいものを返します。
failureを選んだ後も全childへcancellationを要求し、各childのfinalizer完了を待ってからfailureを返します。同じtickで
既にfailureしたchildもcleanupを省略しません。sibling finalizer間の外部操作順は5.11のとおり保証しません。

`forEach` は先頭から末尾へ逐次実行し、各actionが成功してから次へ進みます。空collectionは
何もせず成功します。最初のfailureで停止し、後続actionを開始しません。結果を収集する操作は
`traverse`、並列実行は `forEachParallel` と別名で提供します。

`forEachUntil`も逐次実行し、actionが`Continue`で成功すれば次へ、`Break`で成功すれば後続を
開始せずUnitで成功します。failureはそのままfailure、cancellationは通常のEffect規則に従います。
`LoopControl`をerror channelへ埋め込まず、user errorと正常な短絡を区別します。

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

transformer型と次のoperationは、それぞれ `std/transformer/maybe`、`either`、`reader`、`state`、
`writer` moduleから公開します。同名の `run` と `lift` はmodule aliasで修飾します。

```seseragi
// std/transformer/maybe
fn run<M<_>, A> value: MaybeT<M, A> -> M<Maybe<A>>
fn fromMaybe<M<_>, A> value: Maybe<A> -> MaybeT<M, A>
where Monad<M>
fn lift<M<_>, A> value: M<A> -> MaybeT<M, A>
where Monad<M>

// std/transformer/either
fn run<E, M<_>, A> value: EitherT<E, M, A> -> M<Either<E, A>>
fn fromEither<E, M<_>, A> value: Either<E, A> -> EitherT<E, M, A>
where Monad<M>
fn lift<E, M<_>, A> value: M<A> -> EitherT<E, M, A>
where Monad<M>

// std/transformer/reader
fn run<R, M<_>, A> environment: R -> value: ReaderT<R, M, A> -> M<A>
fn ask<R, M<_>> -> ReaderT<R, M, R>
where Monad<M>
fn asks<R, M<_>, A> f: (R -> A) -> ReaderT<R, M, A>
where Monad<M>
fn local<R, M<_>, A>
  f: (R -> R) -> value: ReaderT<R, M, A> -> ReaderT<R, M, A>
where Monad<M>
fn lift<R, M<_>, A> value: M<A> -> ReaderT<R, M, A>
where Monad<M>

// std/transformer/state
fn run<S, M<_>, A> initial: S -> value: StateT<S, M, A> -> M<(A, S)>
fn get<S, M<_>> -> StateT<S, M, S>
where Monad<M>
fn put<S, M<_>> value: S -> StateT<S, M, Unit>
where Monad<M>
fn modify<S, M<_>> f: (S -> S) -> StateT<S, M, Unit>
where Monad<M>
fn lift<S, M<_>, A> value: M<A> -> StateT<S, M, A>
where Monad<M>

// std/transformer/writer
fn run<W, M<_>, A> value: WriterT<W, M, A> -> M<(A, W)>
fn tell<W, M<_>> output: W -> WriterT<W, M, Unit>
where Monad<M>, Monoid<W>
fn listen<W, M<_>, A> value: WriterT<W, M, A> -> WriterT<W, M, (A, W)>
where Monad<M>, Monoid<W>
fn lift<W, M<_>, A> value: M<A> -> WriterT<W, M, A>
where Monad<M>, Monoid<W>
```

各transformerは `Monad<M>` constraintのもとでFunctor、Applicative、Monad instanceを持ち、
base monadを持ち上げる `lift` を自身のmoduleで提供します。WriterTはさらに `Monoid<W>` を
要求します。

MaybeTはNothing、EitherTはLeftを観測すると後続のtransformer計算を開始しません。それ以前にbase
monadで実行済みのoperationは取り消しません。ReaderTは同じenvironmentを後続へ渡し、`local` の変更は
指定したsubcomputationだけへ適用します。StateTは左から右にstateを渡します。WriterTはMonoidの
`append` で左から右にoutputを蓄積します。

`run` はtransformerを一層だけ外し、base monadを実行しません。`lift` もbase monadをその場で実行せず、
stackへ埋め込むだけです。異なるtransformer間の暗黙liftや、自動的なstack順序の変更はありません。

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

Intのstandard ShowとDebugは`std/int.format`と同じcanonicalな符号付き10進表記を返します。桁区切りや
先頭の`+`は加えません。FloatのShowとDebugは`std/float.format`と同じcanonical表記を使います。
finite値は同じbinary64へround-tripする最短decimalとし、整数に見える表記には`.0`を付けます。
negative zeroは`-0.0`、特殊値は`NaN`、`Infinity`、`-Infinity`、指数はlowercase `e`かつ
plus sign・先頭zeroなしです。Charのstandard Showはそのscalar一個を含むString、Stringのstandard Showは同じStringを返します。
quoteやescapeを含むsource表現はDebugの責務であり、Showは自動でquoteを加えません。
Boolのstandard Showは`True`または`False`、Unitのstandard Showは`()`を返します。

primitiveのstandard instance matrixは次のとおりです。`Never`はruntime valueを持たないため、
直接描画されませんが、`Maybe<Never>`や`Either<Never, A>`の到達不能側を含むconditional
instanceを合成できるunreachable dictionaryをShow/Debugの両方に持ちます。

| primitive | Show | Debug |
| --- | --- | --- |
| `Int` | canonical signed decimal | canonical signed decimal |
| `Float` | canonical binary64 decimal | canonical binary64 decimal |
| `Bool` | `True` / `False` | `True` / `False` |
| `Char` | unquoted scalar | quoted and escaped scalar |
| `String` | identity | quoted and escaped text |
| `Unit` | `()` | `()` |
| `Never` | unreachable evidence | unreachable evidence |

Stringは`Add<String, String, String>`のstandard instanceを持ち、`left + right`は二つのStringを
その順で連結した新しいStringを返します。数値や他の型を暗黙にStringへ変換しません。異なる型を含む
連結は`show`や名前付きformatterで明示的にStringへ変換してから行います。

Int、Bool、Stringはそれぞれ`Eq`のstandard instanceを持ちます。`==`は選択された`Eq<A>.eq`、
`!=`はその結果の否定です。異なる型同士を暗黙変換して比較せず、対応する`Eq` instanceがない型の
比較はcompile errorです。

backtick templateのinterpolationは `Show` を使います。たとえば
`` `user: ${user}` `` は概念上 `"user: " <> show user` へ展開されます。templateは
compilerが構文として検査しますが、`show`自体は通常のtrait methodであり、compiler builtinや
マクロではありません。

`Debug<A>` はdeveloper向け表現です。compiler/runtime version間で文字列互換性を保証せず、
protocol、永続化、snapshot formatへ使ってはなりません。secretを含む型はDebugを実装しないか、
明示的にredactします。

```seseragi
trait Debug<A> {
  fn debug value: A -> String
}
```

BoolとUnitのstandard DebugはそれぞれShowと同じ表記です。Charはsingle quote、Stringは
double quoteで囲み、quote、backslash、NUL、backspace、tab、line feed、form feed、
carriage returnをescapeします。その他のC0/C1 controlは大文字hexの`\u{...}`でescapeします。
このescapeはhostの`JSON.stringify`や`toString`へ意味を委譲しません。

standard Show/Debug instanceは、文字列結果に加えてruntime内部のimmutableなrender documentを
組み立てます。render documentのnodeはtext、soft line、連結、indent、delimiter付きitem列です。
これはlanguage上のtrait methodの戻り型を変えるpublic APIではなく、標準instanceと後続の
composite instanceが共有する内部契約です。同じinstanceから次のlayoutを生成できます。

- compactはsoft lineをspaceにし、一行へ配置する。
- multilineは空のdelimiterを`[]`のように閉じたまま保ち、singletonを含む非空itemを一行ずつ
  一段indentして配置する。
- autoはまずcompactを生成し、改行を含まず既定80 Unicode scalar以内なら採用し、それ以外は
  multilineを採用する。呼び出し側は上限とindent幅を明示的に変更できる。
- nested delimiterは現在のindent depthを引き継ぐ。collection itemとfieldはinstanceが渡した
  source順を保ち、rendererがsortし直さない。

Array、List、Maybe、Eitherは次のconditional standard instanceを持ちます。

- `Show<Array<A>> where Show<A>` / `Debug<Array<A>> where Debug<A>`
- `Show<List<A>> where Show<A>` / `Debug<List<A>> where Debug<A>`
- `Show<Maybe<A>> where Show<A>` / `Debug<Maybe<A>> where Debug<A>`
- `Show<Either<E, A>> where Show<E>, Show<A>` /
  `Debug<Either<E, A>> where Debug<E>, Debug<A>`

Arrayは`[a, b]`、Listは`` `[a, b] ``としてsource順に描画します。Maybeは
`Nothing`または`Just value`、Eitherは`Left error`または`Right value`です。
Debugも同じ外形を使いますが、各payloadをDebug dictionaryで描画するため、Stringなどの
引用符とescapeはShowと異なります。multilineではcollection itemを一段ずつindentし、
constructor payloadはconstructor名の次行へ一段indentします。composite instanceは
子dictionaryのrender documentをそのまま合成し、hostの配列文字列化やobject inspectionへ
委譲しません。

`Range<Int>`は`Show<Range<Int>> where Show<Int>`と
`Debug<Range<Int>> where Debug<Int>`を持ちます。終端を含まない値は`start..end`、含む値は
`start..=end`としてbound自体を描画します。`5..5`のempty、`5..=5`のsingleton、
`10..=1`のdescending emptyも同じ構造を保ち、要素列へ展開しません。現在のRangeは3.9の
とおりunit stepであり、compact / multilineのどちらでも一行のsource-like表記です。
Rangeがcollectionや後続のtuple・recordへ入った場合も、bound dictionaryを再帰的に合成します。

tupleは各要素の`Show`または`Debug`を左から右へ満たすとき、compiler提供の構造的
dictionaryを持ちます。compactでは`(first, second)`、multilineでは同じ要素順を一段
indentしたdocumentとして描画します。

closed structural recordは、全fieldの対応するdictionaryを満たすときcompiler提供の
`Show` / `Debug`を持ちます。structural recordのfield順は型上の宣言順ではなくfield名の
昇順をcanonical順とし、compilerがfield名・optional flag・dictionaryを同じ順序でruntimeへ
渡します。required fieldは`name: value`、optional fieldは存在時に
`name?: Just value`、欠如時に`name?: Nothing`と表示するため、Unit等のpresent値とabsenceを
混同しません。open recordは未知のfieldを失わず描画できないため構造的instanceを持ちません。
runtimeは`Object.keys`、`JSON.stringify`、host inspectionを使用しません。

tuple / closed structural recordのcompiler提供instanceはsealedです。同じ構造headへの明示的な
user instance宣言はstandard instanceとの重複として`SES-T0202`になります。nominal Struct /
ADT / Newtypeのinstance生成はstructural instanceとは別のderiving契約です。

standard error型の表示instanceは次のとおりです。generic errorはpayload側のevidenceを
dictionary factoryへ渡し、host objectの暗黙文字列化へfallbackしません。

| error型 | Show | Debug |
| --- | --- | --- |
| `ConsoleError` | 利用者向けの失敗概要 | error種別を保ち、host messageを`"<redacted>"`へ置換 |
| `StdinError` | constructorと公開payload | constructorとquoted / escaped payload |
| `std/web/dom.DomError` | constructorと公開payload | constructorとquoted / escaped payload |
| `std/web/dom.DomRuntimeError<E>` | `Show<E>`があるとき合成 | `Debug<E>`があるとき合成 |
| `std/web/html.HtmlBuildError` | constructorと公開payload | constructorとquoted / escaped payload |

secretを含み得る型は、Debug instanceを提供しないか、型の契約でredactionを明示します。
standard `ConsoleError`のDebugはhost message、stack、causeを公開せず、Showもhost error objectや
stackを直接描画しません。user定義型がDebugをderiveする場合、その全fieldがDebugへ参加するため、
secret fieldを持つ型ではderiveせずredaction済みの明示instanceを定義します。

`Effect<R, E, A>` / `Task<A>`、`Signal<A>` / `MutableSignal<A>`、function、`DomTarget`、
`Html<Action>`、`Attribute`、その他のhost handleまたはopaque runtime valueにはstandard
Show / Debug instanceを提供しません。これらへ`show`、`debug`、template interpolation、
`printValue`を使うと、通常のinstance選択が`SES-T0201`と
`no Show<T> instance`または`no Debug<T> instance`を報告します。runtime object inspectionや
`String(value)`へfallbackしません。

standard composite dictionaryとcompilerがderiveしたnominal dictionaryは、一回の表示につき
128段の共有深度上限を持ちます。上限へ到達した子は`…`として描画し、stack overflowになるまで
再帰しません。user定義instance内の任意再帰はlanguageが推測できないため、instance作者が
停止条件を持たせます。

surfaceごとの既定layoutは次で固定します。

- template interpolation、`printValue`、Effect failureのstderrは`compact`。
- Playgroundの実行結果と将来のvalue inspectorが保持するDebugは`auto`。
- 明示的なrenderer APIを使うtoolは`compact` / `multiline` / `auto`を選べる。

Show/Debugと出力先は別概念です。型classを実装しただけで値がconsoleへ書かれることは
ありません。

## 9.12 Console

Consoleはhostがenvironmentへ提供するserviceです。`print` をcompiler builtinや
`console.log` への特別loweringとして扱いません。

Consoleのcanonical requirement名は `console` です。`effect fn` の `with Console` は
`with console: Console`、正規Effect型の `{ console: Console }` と同じrequirementです。

```seseragi
fn print text: String
  -> Effect<{ console: Console }, ConsoleError, Unit>

fn println text: String
  -> Effect<{ console: Console }, ConsoleError, Unit>

fn printValue<A> value: A
  -> Effect<{ console: Console }, ConsoleError, Unit>
where Show<A>

fn flush
  -> Effect<{ console: Console }, ConsoleError, Unit>
```

`print` は文字列をそのままstdoutへ書き、改行を加えません。`println` は末尾へ `\n` を一つ
加えます。`printValue value` は `print (show value)` です。stderr出力は
`Console.error` / `Console.errorLine` を使います。

同じdo block内のConsole Effectはsource順に実行します。buffering、terminal encoding、
broken pipeなどhost由来の失敗はConsoleErrorです。test hostはConsole serviceを差し替えて
出力を値として検証できます。

`ConsoleError` はstandard opaque error型で、ShowとDebug instanceを持ちます。Showは利用者向けの
失敗概要を返し、host固有error objectやstack traceを文字列へ埋め込みません。Debugは
`ConsoleError { message: "<redacted>" }`としてhost messageを公開しません。

standard inputは出力用Consoleと別のStdin serviceです。canonical requirement名は`stdin`で、
`with Stdin`は`with stdin: Stdin`へ展開します。preludeの`readLine`はstandard default limitを使います。

```seseragi
fn readLine
  -> Effect<{ stdin: Stdin }, StdinError, Maybe<String>>
```

`StdinError`はShowとDebug instanceを持ち、constructorと公開されているlimit等のpayloadだけを
表示します。

parameterなし関数なので呼び出しは`readLine ()`です。EOFはNothing、空行は`Just ""`であり、
EOFやinput failureを空Stringへまとめません。binary read、明示limit、line Streamは10.14の`std/stdin`を
使います。

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

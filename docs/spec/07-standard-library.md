# 7. 標準ライブラリの契約

## 7.1 core prelude

prelude は自動 import され、次を提供します。

- `Unit`, `Never`, `Bool`, `Int`, `Float`, `String`
- `Maybe`, `Either`, `Array`, `List`, `Task`
- それぞれの constructor
- `Eq`, `Ord`, `Show`
- `Functor`, `Applicative`, `Monad`
- 算術演算子用 trait
- `identity`, `const`, `compose`, `flip`
- `show`

prelude の名前は明示 import で shadowing できません。local binding では shadowing できます。

## 7.2 Maybe と Either

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

`Maybe` と `Either<E, _>` は Functor、Applicative、Monad instance を持ちます。
`Either` は最初の `Left` を保ちます。validation error の蓄積は別の `Validation` 型で
提供し、Either の Applicative の意味を変えません。

Functor、Applicative、Monad はこの順の supertrait 関係を持ちます。したがって Monad
instance は同じ型構築子に対する Applicative と Functor の instance も必要です。

## 7.3 collection

Array は strict・contiguous・indexed collection、List は persistent linked list です。
両方に `map`、`filter`、`foldLeft`、`foldRight`、`length`、`isEmpty` を提供します。

partial function は標準 API にしません。

- `head: List<A> -> Maybe<A>`
- `tail: List<A> -> Maybe<List<A>>`
- `get: Int -> Array<A> -> Maybe<A>`
- `find: (A -> Bool) -> Iterable<A> -> Maybe<A>`

tuple、record、Array、List には、全要素が `Eq` を満たす場合の構造的な Eq instance を
標準ライブラリが提供します。nominal な struct と ADT の Eq は自動生成せず、型を
定義した module が明示的に instance を与えます。

## 7.4 Eq と Ord

`Eq.eq` は反射律・対称律・推移律を満たさなければなりません。Float の標準 Eq は
IEEE 754 の NaN により反射律を満たせないため提供しません。Float の比較は用途を
明示する `Float.ieeeEq`、`Float.totalCompare` などを使います。

`Ord` は全順序です。`compare` は `Less | Equal | Greater` を返し、Eq と整合しなければ
なりません。

## 7.5 数値演算

Int の `+`, `-`, `*`, `**` は結果が 64 bit 範囲外なら defect になります。Int の `/` は
0 方向へ丸め、`%` は被除数と同じ符号の余りを返します。0 による `/` と `%`、および
負の指数による Int の `**` は defect です。失敗を入力として扱うコードは、事前検査か
`Int.checkedDiv` などの checked API を使います。

Float の演算は IEEE 754 に従います。NaN と infinity は有効な Float 値です。Float は
標準の `Eq` と `Ord` instance を持たないため、比較方法を名前付き関数で選びます。

Bool に対する `&&`, `||`, `!` は型クラス演算ではなく、言語組み込みです。

## 7.6 Functor、Applicative、Monad

law は次のとおりです。`==` で観測できる範囲で成立しなければなりません。

- Functor identity: `map identity x == x`
- Functor composition: `map (compose f g) x == map f (map g x)`
- Applicative identity、homomorphism、interchange、composition
- Monad left identity、right identity、associativity

`Task<E, _>` の等価性は同じ observer に対する成功値・失敗値・外部操作順で定義します。

## 7.7 Task

最低限、次を提供します。

```seseragi
fn succeed<E, A> value: A -> Task<E, A>
fn fail<E, A> error: E -> Task<E, A>
fn map<E, A, B> f: (A -> B) -> task: Task<E, A> -> Task<E, B>
fn flatMap<E, A, B>
  f: (A -> Task<E, B>) -> task: Task<E, A> -> Task<E, B>
fn mapError<E, F, A> f: (E -> F) -> task: Task<E, A> -> Task<F, A>
fn recover<E, F, A>
  f: (E -> Task<F, A>) -> task: Task<E, A> -> Task<F, A>
fn parallel<E, A> tasks: Array<Task<E, A>> -> Task<E, Array<A>>
```

`parallel` の結果順は入力順です。最初に観測した failure を返して残りを cancel します。
同じ scheduler tick で複数 failure を観測した場合は、入力 index が小さいものを返します。

## 7.8 表示と debug

値の user-facing な表示は `Show<A>` trait を使います。compiler が型名を出す debug
表現と、application が利用する Show を分けます。`print` は純粋関数ではなく、
`Console.print: String -> Task<ConsoleError, Unit>` として提供します。

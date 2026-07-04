# 10. 標準ライブラリsurface

## 10.1 方針

標準ライブラリは、Seseragiで通常のapplicationを書くために毎回作り直したくない機能を
提供します。preludeへ全名を入れるのではなく、用途別の `std/` moduleとして提供します。

標準ライブラリは言語versionと同じcompatibility contractに従い、特定のJavaScript、Node、
browser runtimeへ依存しません。I/Oはservice interfaceとEffectで抽象化します。

## 10.2 APIの共通規則

- 関数はcurryされ、data argumentを最後に置く。
- partial operationはMaybeまたはEitherを返す。
- defectを起こしうるoperationは `unsafe` submoduleに隔離する。
- collection operationは入力を変更しない。
- I/O、時刻、乱数、並行処理はEffectを返す。
- errorはmodule固有のADTで表し、Stringだけをerror channelにしない。
- resourceを獲得するAPIはScopeまたはbracket契約を持つ。

この規則により、次のpipelineを標準形にします。

```seseragi
import * as arrays from "std/array"

users
  |> arrays.filter isActive
  |> map toSummary
  |> arrays.sortBy (\user -> user.name)
```

## 10.3 prelude

preludeは9.1の型・trait・基本関数だけを自動importします。Map、JSON、HTTPなど用途が限定される
名前は自動importしません。

preludeの目的は「最小programが書けること」であり、「標準ライブラリを全部見せること」では
ありません。

## 10.4 `std/maybe`、`std/either`、`std/validation`

型を保った `map`、`apply`、`flatMap` はpreludeのtrait methodを使います。各moduleは最低限、
次の固有操作を追加します。

- `withDefault`, `orElse`
- `fromNullable`, `toNullable` はinterop module側に置く
- `mapLeft`, `bimap`, `fold`, `swap`
- `sequence`, `traverse`
- `Validation<E, A>` とerror蓄積Applicative
- `NonEmptyList<E>` を使うvalidation helper

Eitherはfail-fast、Validationはerror accumulationです。同じoperatorで意味を切り替えません。

## 10.5 `std/collection`

次のpersistent collectionを必須とします。

- `Array<A>`: contiguous indexed sequence
- `List<A>`: linked sequence
- `NonEmptyList<A>`: 空でないList
- `Map<K, V>`: insertion orderを保持するpersistent map
- `Set<A>`: insertion orderを保持するpersistent set
- `Range<Int>`: Int range

Mapのkey `K` とSetの要素 `A` はEqとHashを要求します。HashはEqと整合し、同値な値は同じhashを
返さなければなりません。

MapとSetは挿入順で反復します。既存keyのvalue更新では位置を変えず、削除後に再挿入すると
末尾へ移動します。EqとHashはlookupに使い、反復順を変えません。

`map`、`reduce`、`sum`、`product`、`combine`、`any`、`all` はconcrete moduleごとに複製せず、
preludeのtrait methodとgeneric reducerを使います。Array/List固有の操作はそれぞれ
`std/array` と `std/list` から明示importします。

### `std/array`

`empty` は `forall A. Array<A>` のpolymorphic valueです。ほかに次を提供します。

```seseragi
fn singleton<A> value: A -> Array<A>
fn fromIterable<C, A> values: C -> Array<A>
where Iterable<C, A>
fn toList<A> values: Array<A> -> List<A>

fn filter<A> predicate: (A -> Bool) -> values: Array<A> -> Array<A>
fn filterMap<A, B> f: (A -> Maybe<B>) -> values: Array<A> -> Array<B>
fn flatMap<A, B> f: (A -> Array<B>) -> values: Array<A> -> Array<B>
fn reduceRight<A, B>
  initial: B -> step: (A -> B -> B) -> values: Array<A> -> B

fn find<A> predicate: (A -> Bool) -> values: Array<A> -> Maybe<A>
fn findIndex<A> predicate: (A -> Bool) -> values: Array<A> -> Maybe<Int>
fn take<A> count: Int -> values: Array<A> -> Array<A>
fn drop<A> count: Int -> values: Array<A> -> Array<A>
fn takeWhile<A> predicate: (A -> Bool) -> values: Array<A> -> Array<A>
fn dropWhile<A> predicate: (A -> Bool) -> values: Array<A> -> Array<A>

fn zip<A, B> right: Array<B> -> left: Array<A> -> Array<(A, B)>
fn zipWith<A, B, C>
  f: (A -> B -> C) -> right: Array<B> -> left: Array<A> -> Array<C>
fn unzip<A, B> values: Array<(A, B)> -> (Array<A>, Array<B>)
fn append<A> suffix: Array<A> -> values: Array<A> -> Array<A>
fn concat<A> values: Array<Array<A>> -> Array<A>
fn reverse<A> values: Array<A> -> Array<A>

fn sort<A> values: Array<A> -> Array<A>
where Ord<A>
fn sortBy<A, K> key: (A -> K) -> values: Array<A> -> Array<A>
where Ord<K>
fn groupBy<A, K> key: (A -> K) -> values: Array<A> -> Map<K, Array<A>>
where Eq<K>, Hash<K>

fn length<A> values: Array<A> -> Int
fn isEmpty<A> values: Array<A> -> Bool
fn get<A> index: Int -> values: Array<A> -> Maybe<A>
fn head<A> values: Array<A> -> Maybe<A>
fn last<A> values: Array<A> -> Maybe<A>
fn init<A> values: Array<A> -> Maybe<Array<A>>
fn tail<A> values: Array<A> -> Maybe<Array<A>>

fn chunksOf<A>
  size: Int -> values: Array<A> -> Either<SizeError, Array<Array<A>>>
fn windows<A>
  size: Int -> values: Array<A> -> Either<SizeError, Array<Array<A>>>
```

### `std/list`

`empty` は `forall A. List<A>` のpolymorphic valueです。ほかに次を提供します。

```seseragi
fn singleton<A> value: A -> List<A>
fn fromIterable<C, A> values: C -> List<A>
where Iterable<C, A>
fn toArray<A> values: List<A> -> Array<A>

fn filter<A> predicate: (A -> Bool) -> values: List<A> -> List<A>
fn filterMap<A, B> f: (A -> Maybe<B>) -> values: List<A> -> List<B>
fn flatMap<A, B> f: (A -> List<B>) -> values: List<A> -> List<B>
fn reduceRight<A, B>
  initial: B -> step: (A -> B -> B) -> values: List<A> -> B

fn find<A> predicate: (A -> Bool) -> values: List<A> -> Maybe<A>
fn findIndex<A> predicate: (A -> Bool) -> values: List<A> -> Maybe<Int>
fn take<A> count: Int -> values: List<A> -> List<A>
fn drop<A> count: Int -> values: List<A> -> List<A>
fn takeWhile<A> predicate: (A -> Bool) -> values: List<A> -> List<A>
fn dropWhile<A> predicate: (A -> Bool) -> values: List<A> -> List<A>

fn zip<A, B> right: List<B> -> left: List<A> -> List<(A, B)>
fn zipWith<A, B, C>
  f: (A -> B -> C) -> right: List<B> -> left: List<A> -> List<C>
fn unzip<A, B> values: List<(A, B)> -> (List<A>, List<B>)
fn append<A> suffix: List<A> -> values: List<A> -> List<A>
fn concat<A> values: List<List<A>> -> List<A>
fn reverse<A> values: List<A> -> List<A>

fn sort<A> values: List<A> -> List<A>
where Ord<A>
fn sortBy<A, K> key: (A -> K) -> values: List<A> -> List<A>
where Ord<K>
fn groupBy<A, K> key: (A -> K) -> values: List<A> -> Map<K, List<A>>
where Eq<K>, Hash<K>

fn length<A> values: List<A> -> Int
fn isEmpty<A> values: List<A> -> Bool
fn get<A> index: Int -> values: List<A> -> Maybe<A>
fn head<A> values: List<A> -> Maybe<A>
fn last<A> values: List<A> -> Maybe<A>
fn init<A> values: List<A> -> Maybe<List<A>>
fn tail<A> values: List<A> -> Maybe<List<A>>

fn chunksOf<A>
  size: Int -> values: List<A> -> Either<SizeError, List<List<A>>>
fn windows<A>
  size: Int -> values: List<A> -> Either<SizeError, List<List<A>>>
```

`SizeError` は `std/collection` が公開します。

```seseragi
type SizeError deriving Eq, Show =
  | NonPositiveSize Int
```

Array/List operationの共通規則は次です。

- callbackは、`reduceRight` を除いてsource順に呼ぶ。`find`、`findIndex`、`takeWhile`、
  `dropWhile` は結果確定時点でshort-circuitする。
- `filterMap` は `Nothing` を除き、`Just value` のvalueを残す。`flatMap` は各結果をsource順に
  連結する。
- `reduceRight` は末尾から先頭へ進み、`step element accumulator` を呼ぶ。空ならinitialを返す。
- `take count` は `count <= 0` ならempty、長さ以上なら元と同じ要素列。`drop count` は
  `count <= 0` なら元と同じ要素列、長さ以上ならempty。
- `zip` と `zipWith` は短い側の長さで停止する。`append suffix values` とpipelineで書いた
  `values |> append suffix` は、valuesの後ろへsuffixを連結する。
- `sort` と `sortBy` はstable sort。`sortBy` はkeyをsource順に各要素一度だけ評価する。
- `groupBy` はkeyをsource順に各要素一度だけ評価し、Mapのkeyを初出順、各groupの要素を
  source順に保つ。
- 負indexと範囲外indexの `get` は `Nothing`。空collectionの `head`、`last`、`init`、`tail` は
  `Nothing`。singletonの `init` と `tail` は `Just empty`。
- `chunksOf size` は隣接する重複なしのchunkへ分割し、最後だけsize未満でも残す。
  `windows size` は幅sizeの隣接する重複ありwindowを一要素ずつずらす。sizeが長さを超える場合は
  emptyを返す。どちらも `size <= 0` なら `Left (NonPositiveSize size)`。

すべての結果collectionはstrictです。`fromIterable` は `next` が `Nothing` になるまで走査するため、
無限Iteratorに適用すると終了しません。Array operationは新しいArrayを返し、List operationは
意味を変えない範囲で既存tailを共有できます。どちらも入力値を変更しません。

Arrayの `length`、`isEmpty`、`get`、`head`、`last` と、Listの `isEmpty`、`head`、`tail` は
O(1)です。Listの `length`、`get`、`last`、`init` と、`append suffix values` はvaluesの長さに
対してO(n)です。Listのappend結果はsuffixを共有できます。sortはO(n log n) comparisons、groupByは
適切なHash instanceのもとでexpected O(n)です。ほかの変換は入力と出力の要素数に対して線形です。

### `std/non-empty-list`

`NonEmptyList<A>` は少なくとも一要素を持つstandard opaque typeです。

```seseragi
fn singleton<A> value: A -> NonEmptyList<A>
fn cons<A> head: A -> tail: List<A> -> NonEmptyList<A>
fn fromList<A> values: List<A> -> Maybe<NonEmptyList<A>>
fn toList<A> values: NonEmptyList<A> -> List<A>
fn head<A> values: NonEmptyList<A> -> A
fn tail<A> values: NonEmptyList<A> -> List<A>
fn reduce1<A>
  step: (A -> A -> A) -> values: NonEmptyList<A> -> A
```

`fromList` だけが空入力で `Nothing` を返し、ほかのoperationはpartialではありません。`reduce1` は
先頭をinitialとして残りをsource順に処理します。initialなしのgeneric `reduce` overloadは作らず、
非空性が型にある場合だけ別名で提供します。

### `std/map`

`std/map` は `Map<K, V>` 型を公開します。`empty` の型schemeは次です。`forall` はsource syntaxではなく
仕様上の表記です。

```text
empty : forall K V. Map<K, V>
```

ほかに次の関数を提供します。

```seseragi
fn singleton<K, V> key: K -> value: V -> Map<K, V>
where Eq<K>, Hash<K>

fn fromEntries<C, K, V> entries: C -> Map<K, V>
where Iterable<C, (K, V)>, Eq<K>, Hash<K>

fn get<K, V> key: K -> values: Map<K, V> -> Maybe<V>
where Eq<K>, Hash<K>

fn containsKey<K, V> key: K -> values: Map<K, V> -> Bool
where Eq<K>, Hash<K>

fn insert<K, V> key: K -> value: V -> values: Map<K, V> -> Map<K, V>
where Eq<K>, Hash<K>

fn upsert<K, V>
  key: K
  -> update: (Maybe<V> -> V)
  -> values: Map<K, V>
  -> Map<K, V>
where Eq<K>, Hash<K>

fn remove<K, V> key: K -> values: Map<K, V> -> Map<K, V>
where Eq<K>, Hash<K>

fn filter<K, V>
  predicate: (K -> V -> Bool) -> values: Map<K, V> -> Map<K, V>

fn mapValues<K, A, B> f: (A -> B) -> values: Map<K, A> -> Map<K, B>

fn mapKeysWith<K1, K2, V>
  resolve: (V -> V -> V)
  -> key: (K1 -> K2)
  -> values: Map<K1, V>
  -> Map<K2, V>
where Eq<K2>, Hash<K2>

fn mergeWith<K, V>
  resolve: (V -> V -> V)
  -> right: Map<K, V>
  -> left: Map<K, V>
  -> Map<K, V>
where Eq<K>, Hash<K>

fn keys<K, V> values: Map<K, V> -> Array<K>
fn values<K, V> source: Map<K, V> -> Array<V>
fn entries<K, V> values: Map<K, V> -> Array<(K, V)>
fn size<K, V> values: Map<K, V> -> Int
fn isEmpty<K, V> values: Map<K, V> -> Bool
```

`empty` は期待型または後続operationからKとVを推論するpolymorphicなexported `let` です。
`upsert` は既存keyなら `Just current`、未登録なら `Nothing` をupdateへ一度渡し、返された値を
保存します。callbackは一度だけ呼びます。

- `fromEntries` の重複keyは最後のvalueを採用するが、keyの位置は最初の出現位置を保つ。
- 既存keyの `insert` / `upsert` は挿入位置を保ち、新規keyは末尾へ追加する。
- `remove` 後に同じkeyをinsertすると末尾へ入る。存在しないkeyのremoveは元と同じmappingを返す。
- `filter` と `mapValues` はkeyと挿入順を保つ。Mapは `Functor<Map<K, _>>` instanceを持ち、
  generic `map` は `mapValues` と同じ意味を持つ。
- `mapKeysWith` で複数keyが同じ出力keyになった場合、source順に
  `resolve current incoming` を呼ぶ。出力keyの位置は最初の出現位置を保つ。
- `mergeWith resolve right left` はleftの順序を保ち、rightだけにあるkeyをrightの順で末尾へ加える。
  重複keyでは `resolve leftValue rightValue` を一度呼び、left側の位置へ結果を保存する。
- `keys`、`values`、`entries` とIterable/Reducible instanceは挿入順。`size` と `isEmpty` はO(1)。

MapのEqは挿入順を比較せず、同じkeyが同じvalueへ対応するかを比較します。標準instanceは
`Eq<K>`、`Hash<K>`、`Eq<V>` を要求します。MapのShowは `Show<K>` と `Show<V>`、Debugは
`Debug<K>` と `Debug<V>` を要求し、観測可能な挿入順でentryを表示します。

### `std/set`

`std/set` は `Set<A>` 型を公開します。`empty` は `forall A. Set<A>` のpolymorphic valueです。
ほかに次の関数を提供します。

```seseragi
fn singleton<A> value: A -> Set<A>
where Eq<A>, Hash<A>

fn fromIterable<C, A> values: C -> Set<A>
where Iterable<C, A>, Eq<A>, Hash<A>

fn contains<A> value: A -> values: Set<A> -> Bool
where Eq<A>, Hash<A>

fn insert<A> value: A -> values: Set<A> -> Set<A>
where Eq<A>, Hash<A>

fn remove<A> value: A -> values: Set<A> -> Set<A>
where Eq<A>, Hash<A>

fn filter<A> predicate: (A -> Bool) -> values: Set<A> -> Set<A>

fn map<A, B> f: (A -> B) -> values: Set<A> -> Set<B>
where Eq<B>, Hash<B>

fn union<A> right: Set<A> -> left: Set<A> -> Set<A>
where Eq<A>, Hash<A>

fn intersection<A> right: Set<A> -> left: Set<A> -> Set<A>
where Eq<A>, Hash<A>

fn difference<A> removed: Set<A> -> values: Set<A> -> Set<A>
where Eq<A>, Hash<A>

fn isSubsetOf<A> superset: Set<A> -> values: Set<A> -> Bool
where Eq<A>, Hash<A>

fn toArray<A> values: Set<A> -> Array<A>
fn toList<A> values: Set<A> -> List<A>
fn size<A> values: Set<A> -> Int
fn isEmpty<A> values: Set<A> -> Bool
```

- `fromIterable` は最初の出現位置を保って重複を除く。
- 既存要素のinsertは位置を変えず、remove後の再insertは末尾へ入る。
- `filter`、`intersection`、`difference` はleftまたはvaluesの順序を保つ。
- `map` で複数要素が同値になった場合は最初の出現位置を保って一つにする。Setは形を保つ
  lawful Functorにできないため、generic `map` のinstanceを持たず、`sets.map` を明示して使う。
- `union right left` はleftを保ち、rightだけにある要素をrightの順で末尾へ加える。
- `toArray`、`toList` とIterable/Reducible instanceは挿入順。`size` と `isEmpty` はO(1)。

SetのEqは挿入順を比較せず、同じ要素を持つかを比較します。標準instanceは `Eq<A>` と `Hash<A>` を
要求します。SetのShowは `Show<A>`、Debugは `Debug<A>` を要求し、観測可能な挿入順で要素を
表示します。

## 10.6 collection trait

標準traitとして次を提供します。

```seseragi
trait Iterable<C, A> {
  fn iterate values: C -> Iterator<A>
}

trait Reducible<C, A>
where Iterable<C, A> {
  fn reduce<B>
    initial: B -> step: (B -> A -> B) -> values: C -> B
}

trait Traversable<F<_>>
where Functor<F> {
  fn traverse<G<_>, A, B>
    f: (A -> G<B>) -> values: F<A> -> G<F<B>>
  where Applicative<G>
}
```

methodの型schemeは次です。

```text
iterate : forall C A. Iterable<C, A> => C -> Iterator<A>
reduce  : forall C A B. Reducible<C, A> => B -> (B -> A -> B) -> C -> B
traverse: forall F G A B. Traversable<F>, Applicative<G>
          => (A -> G<B>) -> F<A> -> G<F<B>>
```

Iterableは順序付き反復、Reducibleは有限collectionの集約、TraversableはApplicative effectを
保った走査を意味します。`reduce`は先頭から末尾へ処理し、initialを必須とします。

IterableとReducibleは `C -> A` のfunctional dependencyを持ち、一つのconcrete collection型 `C` に
要素型 `A` は一つだけです。同じ `C` に異なる `A` を与えるimplはcoherence errorです。この形により、
`Array<A>` のような型構築子だけでなく、`Range<Int>` や `Map<K, V>` のentry反復も表せます。
Reducible instanceは、すべての値について有限回の `next` で終了することを契約します。compilerは
このlawを証明しませんが、標準instanceとlaw testは検査します。

`Iterator<A>` はpureなpull iteratorです。`std/iterator` は少なくとも次を提供します。

```seseragi
fn unfold<S, A>
  step: (S -> Maybe<(A, S)>) -> initial: S -> Iterator<A>

fn next<A> iterator: Iterator<A> -> Maybe<(A, Iterator<A>)>
```

Iteratorはstandard opaque typeでありpersistentです。同じiteratorへ `next` を複数回適用すると
同じ結果を返し、元のiteratorを消費・変更しません。`Just (value, rest)` は次の値と残り、
`Nothing` は終了です。`unfold` はstepを即座に呼ばず、各 `next` 呼び出しがstepを一度呼びます。
memoizeは保証しません。Iterator自体はpureで、I/O、throw、Promiseを隠せません。

Iterable instanceの `iterate` はcollectionの規定順序を保たなければなりません。user-defined
Iterableは `unfold` でIteratorを構築できます。Iteratorは有限性を型に持たないため、無限Iteratorを
全件変換またはeffectful `for`へ渡した計算は終了しない場合があります。

標準instanceは少なくとも次です。

- `Iterable<Array<A>, A>`、`Reducible<Array<A>, A>`、`Traversable<Array>`
- `Iterable<List<A>, A>`、`Reducible<List<A>, A>`、`Traversable<List>`
- `Iterable<NonEmptyList<A>, A>`、`Reducible<NonEmptyList<A>, A>`、`Traversable<NonEmptyList>`
- `Iterable<Range<Int>, Int>`、`Reducible<Range<Int>, Int>`
- `Iterable<Iterator<A>, A>`。有限性を保証できないためReducibleにはしない
- `Iterable<Map<K, V>, (K, V)>`、`Reducible<Map<K, V>, (K, V)>`
- `Iterable<Set<A>, A>`、`Reducible<Set<A>, A>`

RangeはIntを順番に生成し、MapとSetは挿入順を使います。range literalが構築するのは
`Range<Int>` だけです。

data argumentを最後に置くため、import後は次を標準的な書き方とします。

```seseragi
numbers |> reduce 0 (+)

orders
  |> map (\order -> order.total)
  |> sum
```

前者は `reduce 0 (+) numbers` へ展開されます。後者は変換と集約を別の段として見せるため、
初見でも処理の流れを追えます。compilerは意味を変えずにpipelineをfusionして構いません。

## 10.7 `std/text`

Stringを扱う最低APIは次です。

```text
isEmpty, lengthScalars, lengthBytes
concat, join, split, lines, words
trim, trimStart, trimEnd
startsWith, endsWith, contains
replace, replaceAll
toLower, toUpper, caseFold
sliceScalars, scalarAt
encodeUtf8, decodeUtf8
parseInt, parseFloat
```

Unicode scalar、UTF-8 byte、grapheme clusterを混同しません。grapheme操作は
`std/text/grapheme`、正規化は `std/text/unicode` で明示します。

`words` はUnicode whitespaceで分割して空要素を除きます。punctuationは削除しません。
`caseFold` はlocale非依存のUnicode default case foldingです。

```text
words    : String -> Array<String>
caseFold : String -> String
```

Regexは `std/regex` に置き、compile failureをEitherで返します。literal Stringを暗黙にRegexへ
変換しません。

## 10.8 `std/number`

IntとFloatについて、checked arithmetic、parse、format、clamp、min/max、abs、sign、powerを
提供します。

- checked operationはMaybeまたはEitherを返す。
- saturating operationは名前に `saturating` を含める。
- wrapping operationは名前に `wrapping` を含める。
- FloatのNaN、infinity、total orderingは名前付きAPIで扱う。

任意精度整数 `BigInt`、十進数 `Decimal`、byte列 `Bytes` は標準moduleとして提供し、preludeには
入れません。UTF-8、filesystem、HTTP bodyはArray<Int>ではなくBytesを使います。

## 10.9 `std/json` とdecoder

```seseragi
type Json =
  | JsonNull
  | JsonBool Bool
  | JsonNumber Decimal
  | JsonString String
  | JsonArray (Array<Json>)
  | JsonObject (Map<String, Json>)

alias Decoder<A> = Json -> Either<DecodeError, A>
alias Encoder<A> = A -> Json
```

parse、stringify、field、optionalField、index、array、record、oneOf、map、flatMapを提供します。
decoder errorはJSON path、期待型、実値の概要を保持します。

`Js.Unknown` からapplication型へ直接castせず、JSONまたはdomain decoderを通します。

## 10.10 `std/time` と `std/random`

time moduleは少なくとも `Instant`、`Duration`、`LocalDate`、`LocalTime`、`OffsetDateTime` を
区別します。timezone変換失敗やparse失敗をEitherで返します。

現在時刻とsleepはClock serviceを要求します。

```seseragi
fn now -> Effect<{ clock: Clock }, ClockError, Instant>
fn sleep duration: Duration -> Effect<{ clock: Clock }, Never, Unit>
```

乱数はRandom serviceを要求し、testではseed済みserviceへ差し替えられます。pure関数がglobal
random sourceを読みません。

## 10.11 `std/effect`

Effect moduleは9.8のoperationに加え、次を提供します。

- environment: `service`, `provideSome`
- value/error変換: `attempt`, `fromEither`, `fromMaybe`
- temporal control: `timeout`, `retry`, `repeat`
- resource: `bracket`, `scoped`, `acquireRelease`

この節ではconcurrencyとsynchronization primitiveのsignatureを固定します。上のoperationのretry policy、
timeout result、resource exit caseはruntime resource契約と一緒に別節で固定します。

```seseragi
type FiberExit<E, A> =
  | FiberSucceeded A
  | FiberFailed E
  | FiberCancelled

fn fork<R, E, A> effect: Effect<R, E, A>
  -> Effect<R, Never, Fiber<E, A>>
fn await<E, A> fiber: Fiber<E, A> -> Task<Never, FiberExit<E, A>>
fn poll<E, A> fiber: Fiber<E, A> -> Task<Never, Maybe<FiberExit<E, A>>>
fn join<E, A> fiber: Fiber<E, A> -> Task<E, A>
fn interrupt<E, A> fiber: Fiber<E, A> -> Task<Never, Unit>
fn yieldNow -> Task<Never, Unit>

fn race<R, E, A>
  left: Effect<R, E, A>
  -> right: Effect<R, E, A>
  -> Effect<R, E, A>

fn scoped<R, E, A> effect: Effect<R, E, A> -> Effect<R, E, A>
```

`Fiber<E, A>` はstandard opaque typeです。`race` は最初に終了したsuccessまたはfailureを返し、loserへ
cancellationを要求してfinalizer完了を待ちます。同じscheduler turnで両方が終了した場合はleftを
選びます。`scoped` は内側scopeを作り、5.11のsupervision規則でresourceとchild Fiberを閉じます。

bounded parallelismはvalidated valueで指定します。

```seseragi
type ParallelismError deriving Eq, Show =
  | NonPositiveParallelism Int

fn parallelism value: Int -> Either<ParallelismError, Parallelism>
fn unboundedParallelism -> Parallelism

fn forEachParallel<C, R, E, A>
  parallelism: Parallelism
  -> f: (A -> Effect<R, E, Unit>)
  -> values: C
  -> Effect<R, E, Unit>
where Reducible<C, A>

fn traverseParallel<C, R, E, A, B>
  parallelism: Parallelism
  -> f: (A -> Effect<R, E, B>)
  -> values: C
  -> Effect<R, E, Array<B>>
where Reducible<C, A>
```

`parallelism` は正数だけを受理し、`unboundedParallelism ()` は有限入力の全要素を開始可能にします。
parallel operationは入力をsource順にindex付けし、最大parallelism件だけ同時実行します。結果Arrayは
完了順でなく入力順です。最初に観測したfailureで未開始要素を開始せず、実行中Fiberをcancelします。
同じturnの複数failureは入力indexが小さいものを選びます。

Iterableを逐次処理する基本signatureは次です。

```seseragi
fn forEach<C, R, E, A>
  f: (A -> Effect<R, E, Unit>)
  -> values: C
  -> Effect<R, E, Unit>
where Iterable<C, A>
```

`forEach` は `iterate values` の順に一件ずつEffectを実行し、最初のfailureで停止します。
`forEachParallel` は同じ名前のoverloadではなく、並行数と結果順序を明示する別APIです。

### `std/ref`

```seseragi
fn make<A> initial: A -> Task<Never, Ref<A>>
fn get<A> reference: Ref<A> -> Task<Never, A>
fn set<A> value: A -> reference: Ref<A> -> Task<Never, Unit>
fn update<A> f: (A -> A) -> reference: Ref<A> -> Task<Never, Unit>
fn modify<A, B>
  f: (A -> (B, A)) -> reference: Ref<A> -> Task<Never, B>
```

Refはstandard opaque typeで、全operationはlinearizableです。update/modifyのpure callbackはatomic
section内で一度だけ呼び、Effectを返せません。Refはsubscriberを持たない同期primitiveで、Signalとは
別の型です。callbackがdefectで停止した場合は新しい値を保存しません。

### `std/deferred`

```seseragi
fn make<E, A> -> Task<Never, Deferred<E, A>>
fn await<E, A> deferred: Deferred<E, A> -> Task<E, A>
fn poll<E, A> deferred: Deferred<E, A>
  -> Task<Never, Maybe<Either<E, A>>>
fn complete<E, A>
  result: Either<E, A> -> deferred: Deferred<E, A> -> Task<Never, Bool>
fn succeed<E, A> value: A -> deferred: Deferred<E, A> -> Task<Never, Bool>
fn fail<E, A> error: E -> deferred: Deferred<E, A> -> Task<Never, Bool>
```

Deferredは一度だけ完了するstandard opaque typeです。最初のcompleteだけTrueを返して全waiterを登録順に
runnableにし、以後はFalseです。awaitのcancellationはそのwaiterだけを除き、Deferredを完了させません。
完了とcancellationが同じturnなら、先にschedulerへ登録されたeventを採用します。

### `std/queue`

```seseragi
type QueueCreateError deriving Eq, Show =
  | NonPositiveCapacity Int

type QueueClosed deriving Eq, Show =
  | QueueClosed

fn bounded<A> capacity: Int -> Task<QueueCreateError, Queue<A>>
fn unbounded<A> -> Task<Never, Queue<A>>
fn offer<A> value: A -> queue: Queue<A> -> Task<QueueClosed, Unit>
fn take<A> queue: Queue<A> -> Task<QueueClosed, A>
fn tryOffer<A>
  value: A -> queue: Queue<A> -> Task<Never, Either<QueueClosed, Bool>>
fn tryTake<A>
  queue: Queue<A> -> Task<Never, Either<QueueClosed, Maybe<A>>>
fn size<A> queue: Queue<A> -> Task<Never, Int>
fn close<A> queue: Queue<A> -> Task<Never, Unit>
```

QueueはFIFOのstandard opaque typeです。boundedは正capacityだけを受理します。offerは空きができるまで、
takeは値が来るまでsuspendし、waiterもFIFOです。待機中offerのcancellationは値をenqueueせず、待機中
takeのcancellationは値をconsumeしません。acceptとcancellationが同じturnなら登録順で決めます。

closeはidempotentです。close後のofferと待機中offerはQueueClosedになります。buffer済みvalueとclose時点で
成立できるtakeはFIFOでdrainし、その後のtakeはQueueClosedです。tryOfferは待たず、成功enqueueなら
Right True、満杯ならRight False、closedならLeftです。tryTakeも待たず、value、empty、closedを
それぞれRight (Just value)、Right Nothing、Leftで区別します。

### `std/semaphore`

```seseragi
type SemaphoreCreateError deriving Eq, Show =
  | NonPositivePermits Int

fn make permits: Int -> Task<SemaphoreCreateError, Semaphore>
fn acquire semaphore: Semaphore -> Task<Never, Permit>
fn release permit: Permit -> Task<Never, Unit>
fn withPermit<R, E, A>
  semaphore: Semaphore
  -> effect: Effect<R, E, A>
  -> Effect<R, E, A>
fn available semaphore: Semaphore -> Task<Never, Int>
```

SemaphoreとPermitはstandard opaque typeです。makeは正permit数だけを受理します。acquire waiterは
FIFOで、待機中cancellationはpermitを消費しません。Permitは取得元Semaphoreに属し、releaseは
idempotentです。withPermitはacquire後にeffectを一度実行し、success、failure、cancellationのすべてで
releaseしてから終了します。

## 10.12 `std/stream`

`Stream<R, E, A>` はenvironment `R` を要求し、`E` で失敗しうる0個以上の非同期値を表すstandard
opaque typeです。Stream valueはcoldで再利用可能なdescriptionです。terminal operationを実行するたびに
独立したproducerとresource scopeを作り、同じStream valueを二度実行してもsubscriptionやiteratorを
共有しません。

constructorと基本変換は次です。値を生成しないconstructorも周囲の型へ合わせられるよう `R` と `E` を
量化します。

```seseragi
fn empty<R, E, A> -> Stream<R, E, A>
fn singleton<R, E, A> value: A -> Stream<R, E, A>
fn fromArray<R, E, A> values: Array<A> -> Stream<R, E, A>
fn fromIterable<C, R, E, A> values: C -> Stream<R, E, A>
where Iterable<C, A>
fn fromEffect<R, E, A> effect: Effect<R, E, A> -> Stream<R, E, A>
fn unfold<S, R, E, A>
  step: (S -> Maybe<(A, S)>) -> initial: S -> Stream<R, E, A>

fn map<R, E, A, B>
  f: (A -> B) -> stream: Stream<R, E, A> -> Stream<R, E, B>
fn filter<R, E, A>
  predicate: (A -> Bool) -> stream: Stream<R, E, A> -> Stream<R, E, A>
fn filterMap<R, E, A, B>
  f: (A -> Maybe<B>) -> stream: Stream<R, E, A> -> Stream<R, E, B>
fn mapError<R, E, F, A>
  f: (E -> F) -> stream: Stream<R, E, A> -> Stream<R, F, A>
fn flatMap<R, E, A, B>
  f: (A -> Stream<R, E, B>)
  -> stream: Stream<R, E, A>
  -> Stream<R, E, B>

fn take<R, E, A> count: Int -> stream: Stream<R, E, A> -> Stream<R, E, A>
fn drop<R, E, A> count: Int -> stream: Stream<R, E, A> -> Stream<R, E, A>
fn chunk<R, E, A>
  size: Int
  -> stream: Stream<R, E, A>
  -> Either<SizeError, Stream<R, E, Array<A>>>
```

`fromArray` はsnapshotしたArrayをsource順に出します。`fromIterable` はterminal operationごとに
`iterate values` で新しいIteratorを作ります。`fromEffect` は実行ごとにeffectを一度だけ実行し、successを
一件出すかfailureで終了します。`unfold` のstepはpureで、`Nothing` まで逐次評価します。

`flatMap` はouter source順かつinnerを一つずつ最後まで実行するsequential concat-mapです。並行な
flat-mapを暗黙に選びません。Streamは `Stream<R, E, _>` ごとにFunctorとMonad instanceを持ち、generic
`map` と `flatMap` は上の意味を使います。`take` と `drop` のcount規則はArray/Listと同じです。
`chunk` は隣接要素を最大size件のArrayにし、最後の短いchunkも残します。sizeが正でなければStreamを
開始せず `Left (NonPositiveSize size)` を返します。

複数sourceの合成は順序をAPIごとに固定します。

```seseragi
fn concat<R, E, A>
  suffix: Stream<R, E, A>
  -> prefix: Stream<R, E, A>
  -> Stream<R, E, A>
fn zip<R, E, A, B>
  right: Stream<R, E, B>
  -> left: Stream<R, E, A>
  -> Stream<R, E, (A, B)>
fn merge<R, E, A>
  right: Stream<R, E, A>
  -> left: Stream<R, E, A>
  -> Stream<R, E, A>
```

`concat suffix prefix` はprefix完了後にsuffixを開始します。`zip` は両sourceから一件ずつ要求し、短い側が
終了した時点で長い側をcancelします。`merge` は両sourceを同時に開始し、利用可能になった値から出します。
同じscheduler turnならleftを先に出します。一方が通常終了しても他方を継続し、一方がfailureなら他方を
cancelしてfinalizer完了後にそのfailureを返します。同じturnのfailureはleftを選びます。mergeは各sourceに
高々一件だけ未完了のdemandを出し、sourceごとに高々一件の完成済みvalueを保持します。

### demand、buffer、overflow

既定のStreamはpull-basedです。downstreamが次の値を要求するまでupstreamへ新しいdemandを出しません。
各operatorは受け取ったdemandを満たすために必要な分だけ上流へ要求します。`filter` は一件出せるまで、
`chunk` はsize件またはupstream終了まで要求します。`merge`、time operator、明示bufferだけは並行producerを
進められますが、未消費値を仕様で定めたcapacityより多く保持しません。`debounce` は最新一件だけを保持し、
`throttleFirst` はwindow中の後続値を保持せず破棄します。

```seseragi
type BufferCapacityError deriving Eq, Show =
  | NonPositiveBufferCapacity Int

type DropStrategy deriving Eq, Show =
  | DropOldest
  | DropLatest

type BufferOverflowError deriving Eq, Show =
  | BufferOverflow Int

fn bufferCapacity value: Int
  -> Either<BufferCapacityError, BufferCapacity>

fn buffer<R, E, A>
  capacity: BufferCapacity
  -> stream: Stream<R, E, A>
  -> Stream<R, E, A>
fn bufferDropping<R, E, A>
  strategy: DropStrategy
  -> capacity: BufferCapacity
  -> stream: Stream<R, E, A>
  -> Stream<R, E, A>
fn bufferFailing<R, E, A>
  capacity: BufferCapacity
  -> stream: Stream<R, E, A>
  -> Stream<R, Either<E, BufferOverflowError>, A>
```

`BufferCapacity` は正数だけを表すstandard opaque typeです。`buffer` は満杯ならproducerをsuspendする
lossless backpressureです。`bufferDropping DropOldest` は満杯時に最古の未消費値を捨てて新値を入れ、
`DropLatest` は到着した新値を捨てます。`bufferFailing` はsource failureを `Left error`、満杯時の
overflowを `Right (BufferOverflow capacity)` として一度だけ失敗し、producerをcancelします。

bufferのconsumerがcancelされた場合、待機中producerへcancellationを伝播して保持値を破棄します。
producerの完了後は保持値をFIFOでdrainしてから完了します。producer failureは保持値より優先し、保持値を
破棄してfailureを通知します。

### time operator

```seseragi
fn debounce<R, E, A>
  duration: Duration
  -> stream: Stream<R, E, A>
  -> Stream<R & { clock: Clock }, E, A>
fn throttleFirst<R, E, A>
  duration: Duration
  -> stream: Stream<R, E, A>
  -> Stream<R & { clock: Clock }, E, A>
```

`debounce` は値の後にdurationだけ新値が来なければ最新値を出し、sourceが通常終了した場合はpending値を
直ちに出して終了します。`throttleFirst` はwindow先頭の値を出し、duration中の後続値を捨てます。
durationがzeroならどちらも全値をsource順に出します。負Durationは構築できません。どちらもClock serviceを
要求し、consumer cancellation時はtimerとupstreamをcancelします。

### terminal operationとresource

```seseragi
fn runCollect<R, E, A> stream: Stream<R, E, A> -> Effect<R, E, Array<A>>
fn runFold<R, E, A, B>
  initial: B
  -> step: (B -> A -> B)
  -> stream: Stream<R, E, A>
  -> Effect<R, E, B>
fn runForEach<R, E, A>
  action: (A -> Effect<R, E, Unit>)
  -> stream: Stream<R, E, A>
  -> Effect<R, E, Unit>
```

terminal operationは値をsource順に処理し、最初のfailureでproducerをcancelします。`runForEach` は前の
actionが成功するまで次の値を要求しません。`runCollect` は有限Stream専用で、型から有限性は判定しません。
無限Streamでは完了せず、値を保持し続けます。

Stream実行はterminal operationのscopeに所属します。success、failure、cancellationのすべてでproducer、
timer、subscriptionをcancelし、登録と逆順にfinalizerを完了してからterminal Effectを終了します。
consumerのearly terminationである `take`、短い側で終わる `zip` も同じ規則で不要なupstreamを閉じます。

### Signalとの変換

```seseragi
type SignalStart deriving Eq, Show =
  | EmitCurrent
  | ChangesOnly

fn fromSignalBuffered<A>
  start: SignalStart
  -> capacity: BufferCapacity
  -> signal: Signal<A>
  -> Stream<{}, Never, A>
fn fromSignalDropping<A>
  start: SignalStart
  -> strategy: DropStrategy
  -> capacity: BufferCapacity
  -> signal: Signal<A>
  -> Stream<{}, Never, A>
fn fromSignalLatest<A>
  start: SignalStart -> signal: Signal<A> -> Stream<{}, Never, A>

fn runIntoSignal<R, E, A>
  target: MutableSignal<A>
  -> stream: Stream<R, E, A>
  -> Effect<R, E, Unit>
```

`EmitCurrent` はsubscription開始時のsnapshotを先頭値にし、`ChangesOnly` は開始後のtransactionだけを
対象にします。snapshot取得とsubscription登録はatomicで、その間の更新を失いません。
`fromSignalBuffered` は満杯ならSignal observerをsuspendするlossless変換です。
`fromSignalDropping` は指定strategyで欠落させます。`fromSignalLatest` は未消費値を常に最新一件へ置き換える
conflationで、transaction境界は保ちますが中間値の個数は保ちません。同じtransactionで同じSignalが複数回
変更されても、glitch-free graphが公開する安定値だけを一件として扱います。

`runIntoSignal` は各値を順に一件のSignal transactionとしてtargetへsetし、source完了まで戻りません。
background化は `fork $ runIntoSignal target stream` と明示し、隠れたdaemon Fiberを作りません。source
failureは呼び出し側へ返り、targetは最後に成功した値を保持します。

## 10.13 `std/signal`

5.12から5.15のSignal、MutableSignal、SignalChange、Subscription APIを提供します。生成、snapshot
read、単一更新、multi-signal transaction、derived graph、distinct、switchMap、subscriptionを
このmoduleの公開surfaceとします。UI framework固有bindingは標準coreへ含めず、adapter packageとして
提供します。

## 10.14 Console、Logger、filesystem、process

`std/console` と `std/log` は9.12、9.13のserviceを提供します。

`std/path` はpureなpath操作、`std/fs` はFileSystem serviceを要求するEffectを提供します。
最低限、read/write、streaming、directory listing、metadata、atomic replace、temporary resourceを
含めます。

`std/process` はargumentとenvironmentの読み取り、child process、exit statusを扱います。
process終了を通常関数から行わず、mainの戻り値をhostがexit codeへ変換します。

## 10.15 `std/http`

HTTPは `Request`、`Response`、`Method`、`Status`、`Headers`、`Body` とHttpClient serviceを
提供します。

- request送信は `Effect<{ http: HttpClient }, HttpError, Response>`。
- bodyは小さい値向けBytesとstreamingの両方を持つ。
- JSON decodeはstd/json decoderを明示的に使う。
- non-2xx statusをtransport errorへ自動変換しない。
- timeout、redirect、retryはoptionまたはEffect combinatorで明示する。

server APIはclientと別moduleにし、runtime adapterが提供できる場合だけ利用します。

## 10.16 `std/test`

test moduleはassertion、table test、property test、law test、Effect test runtimeを提供します。

- Eq diffとstructural diff
- expected failureとtimeout
- deterministic Clock、Random、Console、Logger
- resource leakと未終了Fiberの検出
- Functor、Applicative、Monad、Semigroup、Monoid law helper

test自体も通常のSeseragi moduleであり、特別な型検査規則を持ちません。

## 10.17 optional adapter

DOM、Node、database driver、web frameworkなどhost固有APIは標準言語semanticsではありません。
`.d.ts` converterとforeign bindingを使うadapter packageとして提供します。

adapterが標準service interfaceを実装できる場合、application codeはhostを変えても同じEffect
signatureを維持できます。

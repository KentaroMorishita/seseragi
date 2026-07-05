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

型を保った `map`、`apply`、`flatMap` はpreludeのtrait methodを使います。MaybeとEitherの各moduleは
最低限、次の固有操作を追加します。

- `withDefault`, `orElse`
- `fromNullable`, `toNullable` はinterop module側に置く
- `mapLeft`, `bimap`, `fold`, `swap`
- `sequence`, `traverse`

Eitherはfail-fastです。Validationは独立した入力のerror accumulationを次の型で表します。

```seseragi
type Validation<E, A> =
  | Valid A
  | Invalid (NonEmptyList<E>)

fn valid<E, A> value: A -> Validation<E, A>
fn invalid<E, A> error: E -> Validation<E, A>
fn invalidMany<E, A> errors: NonEmptyList<E> -> Validation<E, A>
fn fromEither<E, A> value: Either<E, A> -> Validation<E, A>
fn toEither<E, A> value: Validation<E, A> -> Either<NonEmptyList<E>, A>
```

ValidationはFunctorとApplicative instanceを持ち、Monad instanceを持ちません。`pure` は `Valid`、
`map` はValidの値だけを変換します。Applicative applyは次の規則です。

```text
Valid f       <*> Valid value       = Valid (f value)
Invalid left  <*> Valid _           = Invalid left
Valid _       <*> Invalid right     = Invalid right
Invalid left  <*> Invalid right     = Invalid (left <> right)
```

両側Invalidではleftのerror列を先、rightを後にしてNonEmptyListを連結します。したがってcurried関数へ
左から `<*>` で入力を与えると、入力順に全errorを蓄積します。後続処理が前のsuccess値へ依存する場合は
Eitherやdomain固有ADTへ切り替え、Validationへfail-fastなflatMapを追加しません。

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

### Map / Setのhash seedとserialization

`Hash.hash` が返すIntはpureなuser-level hashです。runtimeはその結果をprocess-localなhash seedで
mixしてからMap / Setの内部indexへ使います。seedはMap / Setの値、Eq、Show、Debug、反復順、生成code、
serialized dataの一部ではありません。同じ内容を異なるseedで保持したcollectionは観測上同じ値です。

default seedはprocess開始時にhostのcryptographically secureなentropyから一度生成します。pureな
collection operationがRandom serviceやglobal random valueを読むわけではありません。seedによって
変化できるのはbucket配置とperformanceだけで、callback回数・評価順・出力順を含む言語semanticsは
変化しません。persistent operationは入力collectionのseedを引き継ぎ、二つのMap / Setを結合する
operationはdata-last側、つまり結果の順序の基準になるleft / values側のseedへ再indexします。

`Hash.hash` の具体的な数値とstandard instanceのalgorithmはlanguage ABIではなく、version間の安定性を
保証しません。hash値、bucket配置、seedをfile、network、cache keyへ保存してはなりません。hostにsecure
entropyがなく、manifestで固定seedも指定されていないtargetは、application codeを開始する前にruntime
startup errorで停止しなければなりません。

Map / Setをsequenceとしてserializeする標準contractは次です。

- Mapは挿入順のkey-value pair列、Setは挿入順のvalue列としてencodeする。
- decodeは `fromEntries` / `fromIterable` と同じ重複規則を使う。Mapは最後のvalueと最初の位置、Setは
  最初の位置を保つ。
- hash seedとbucket配置はencodeせず、decode先runtimeのseedで新しくindexする。
- formatがstring keyだけを持つ場合、任意のMapをobjectへ暗黙変換しない。Map<String, V>用の明示的な
  object adapterか、pair列adapterを選ぶ。
- insertion orderは値として観測できますがMap / SetのEqには含まれません。したがって通常encodeは
  deterministicでも、Eqな二値から同じbytesを作るcanonical encodingとは限りません。
- signing、content hash、reproducible artifact用のcanonical encoderはOrdを要求し、Mapはkey、Setはvalueの
  昇順にencodeする。同じOrdでEqualな値はEqでも同値でなければならず、同順位を入力順で補わない。

JSONではMapを既定でentryの2要素Array列、SetをvalueのArrayとして表します。JSON objectとの変換は
Map<String, V>専用の名前付きadapterです。canonical JSONはJSON自身のnumber / string escaping規則に加え、
上記のOrd順を使います。encoder / decoder導出のsurface syntaxは別途定義しますが、このdata表現と重複規則を
変更してはなりません。

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

### `std/regex`

Regexはhostの正規表現objectではなく、Seseragiが定義するimmutableなstandard opaque typeです。
pattern compileはpureで、通常の不正patternをexceptionやdefectにしません。

```seseragi
type RegexCompileErrorKind deriving Eq, Show =
  | UnexpectedRegexEnd
  | UnexpectedRegexToken Char
  | InvalidRegexEscape
  | InvalidRegexRange
  | InvalidRegexQuantifier
  | DuplicateCaptureName String
  | UnsupportedRegexFeature String

struct RegexCompileError deriving Eq, Show {
  kind: RegexCompileErrorKind,
  offset: Int
}

struct RegexOptions deriving Eq, Show {
  caseInsensitive: Bool,
  multiline: Bool,
  dotMatchesNewline: Bool
}

struct RegexSpan deriving Eq, Ord, Show {
  start: Int,
  end: Int
}

struct RegexCapture deriving Eq, Show {
  span: RegexSpan,
  text: String
}

struct RegexMatch deriving Eq, Show {
  span: RegexSpan,
  text: String,
  captures: Array<Maybe<RegexCapture>>,
  named: Map<String, Maybe<RegexCapture>>
}

fn defaultOptions -> RegexOptions
fn compile pattern: String -> Either<RegexCompileError, Regex>
fn compileWith options: RegexOptions
  -> pattern: String
  -> Either<RegexCompileError, Regex>

fn isMatch pattern: Regex -> text: String -> Bool
fn find pattern: Regex -> text: String -> Maybe<RegexMatch>
fn findAll pattern: Regex -> text: String -> Array<RegexMatch>
fn split pattern: Regex -> text: String -> Array<String>
fn replaceAll pattern: Regex -> replacement: String -> text: String -> String
fn replaceAllWith pattern: Regex
  -> replacement: (RegexMatch -> String)
  -> text: String
  -> String
fn escape text: String -> String
```

`compile` は `defaultOptions ()` を使います。defaultは三つのBoolがすべてFalseです。offsetとmatch spanは
0-based・end-exclusiveのUTF-8 byte offsetです。RegexCapture.textとRegexMatch.textは対応spanのStringを
返すため、利用側がbyte boundaryを再検証せず安全に内容を使えます。capturesは左括弧順のcapturing groupを
並べ、group 0を含みません。namedは宣言順を保ち、matchしなかったgroupもkeyを残してNothingにします。

必須pattern syntaxはliteral、escape、`.`、character classとnegation、concatenation、`|`、capturing group、
`(?:...)`、`(?<name>...)`、`*`、`+`、`?`、`{m}`、`{m,}`、`{m,n}`、`^`、`$`、`\A`、`\z` です。
quantifierはgreedyです。最も左で開始するmatchを選び、同じ開始位置ではpattern内で先に書かれたalternativeを
優先し、その内部でgreedy quantifierを最大化します。backreference、look-around、atomic group、conditional、
recursion、inline flag、lazy / possessive quantifierはUnsupportedRegexFeatureです。このsubsetは入力長に対して
linear timeでmatchできなければならず、hostのbacktracking engineへ意味を委譲しません。

patternと入力はUnicode scalar列としてmatchします。`.` はdefaultでline terminator以外のscalar一つ、
dotMatchesNewlineでは任意のscalar一つです。`\d` はASCII `[0-9]`、`\s` はUnicode White_Space、`\w` は
Unicode Alphabetic、Mark、Decimal_Number、Connector_Punctuationのいずれかです。`\p{Property}` と
`\P{Property}` はruntimeが公開するUnicode property tableを使います。caseInsensitiveはlocale非依存の
Unicode simple case foldingです。multiline=Falseの `^` / `$` は入力全体、Trueではline境界にもmatchし、
`\A` / `\z` は常に入力全体だけを表します。

findAllとsplitは左から右へnon-overlapping matchを使います。空matchの後は、入力末尾でなければUnicode scalar
一つ進め、末尾ならそのmatchを含めて終了します。replaceAllのreplacementは `$` や `\` を展開しないliteral
Stringです。captureを使う置換はreplaceAllWithで明示します。escapeは入力全体をliteralとしてmatchできる
pattern fragmentを返します。

compiler、runtime、formatter、LSPはlanguage release metadataに同じUnicode data versionを公開しなければ
なりません。このversionはcaseFold、White_Space、regex property、identifier判定で共通です。generated artifact
には要求versionを記録し、異なるUnicode versionのruntimeはcode実行前にABI mismatchとして拒否します。
TypeScript backendでもJS RegExpへ意味を委譲せず、この契約に適合するruntime engineを呼びます。

## 10.8 `std/number` と `std/bytes`

IntとFloatについて、checked arithmetic、parse、format、clamp、min/max、abs、sign、powerを
提供します。

- checked operationはMaybeまたはEitherを返す。
- saturating operationは名前に `saturating` を含める。
- wrapping operationは名前に `wrapping` を含める。
- FloatのNaN、infinity、total orderingは名前付きAPIで扱う。

任意精度整数 `BigInt`、十進数 `Decimal`、byte列 `Bytes` は標準moduleとして提供し、preludeには
入れません。UTF-8、filesystem、HTTP bodyはArray<Int>ではなくBytesを使います。

### `std/decimal`

`Decimal` は有限な任意精度十進数を表すstandard opaque type、`DecimalContext` は丸めを伴う計算の
有効桁数とrounding modeを表すstandard opaque typeです。Decimalの値は符号付き係数と10進scaleで
表現できますが、公開上は末尾の0を除いたcanonical valueです。したがって `1.0` と `1.00` は同じ値で、
負のzeroはzeroへ正規化されます。固定小数点の表示scaleはDecimal自身には保存しません。

```seseragi
type DecimalParseError deriving Eq, Show =
  | InvalidDecimal { offset: Int }

type DecimalContextError deriving Eq, Show =
  | NonPositiveDecimalPrecision Int

type DecimalArithmeticError deriving Eq, Show =
  | DecimalDivisionByZero
  | NonTerminatingDecimal

type DecimalConversionError deriving Eq, Show =
  | DecimalNotIntegral
  | DecimalOutsideIntRange
  | DecimalOutsideFloatRange
  | FloatNotFinite

type RoundingMode deriving Eq, Show =
  | HalfEven
  | HalfUp
  | TowardZero
  | AwayFromZero
  | Floor
  | Ceiling

fn parse text: String -> Either<DecimalParseError, Decimal>
fn fromInt value: Int -> Decimal
fn toIntExact value: Decimal -> Either<DecimalConversionError, Int>
fn fromFloat context: DecimalContext
  -> value: Float
  -> Either<DecimalConversionError, Decimal>
fn toFloat value: Decimal -> Either<DecimalConversionError, Float>

fn context precision: Int -> rounding: RoundingMode
  -> Either<DecimalContextError, DecimalContext>
fn precision context: DecimalContext -> Int
fn rounding context: DecimalContext -> RoundingMode

fn divideExact divisor: Decimal -> dividend: Decimal
  -> Either<DecimalArithmeticError, Decimal>
fn divide context: DecimalContext
  -> divisor: Decimal
  -> dividend: Decimal
  -> Either<DecimalArithmeticError, Decimal>
fn quantize scale: Int
  -> rounding: RoundingMode
  -> value: Decimal
  -> Decimal
```

`parse` が受理するsyntaxはASCIIの
`[+-]?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?` だけです。前後の空白、
桁区切り、NaN、infinityは受理せず、最初に不正となるUTF-8 byte offsetを返します。`Show` は指数表記を
使わないcanonical decimal表記を生成し、不要な小数点と末尾の0を出力しません。入力末尾でtokenが
不足する場合のoffsetはStringのUTF-8 byte lengthです。

Decimalは値に基づくEq、Ord、Hashと、Zero、One、Add、Sub、Mul instanceを持ちます。加算、減算、乗算は
丸めないexact operationです。除算は常にexactとは限らないためDiv instanceと `/` を持ちません。
`divideExact divisor dividend` は `dividend / divisor` を計算し、0除算または有限十進数にならない結果を
Eitherで返します。丸める除算はpositiveな有効桁数とrounding modeを持つDecimalContextを必須にします。

HalfEvenはちょうど中間なら最後の保持桁を偶数へ、HalfUpは中間なら絶対値を大きくする方向へ丸めます。
TowardZero、AwayFromZero、Floor、Ceilingはそれぞれ0方向、0から離れる方向、負の無限大方向、正の無限大
方向です。`quantize scale mode value` は小数部をscale桁へ丸めます。負scaleは10、100など整数側の桁を
丸めます。結果は再びcanonical valueになるため、通貨のような固定表示scaleはformatterまたはdomain
newtypeで表現します。

Int、Float、Decimal間にimplicit conversionはありません。`fromInt` はexactです。`toIntExact` は小数部または
Int範囲外を拒否します。`fromFloat` はfinite Floatの正確なbinary valueを十進値とみなし、contextの有効桁数と
modeで丸めます。NaNとinfinityはFloatNotFiniteです。`toFloat` は最も近いfinite Floatへties-to-evenで丸め、
finite Floatの範囲を超える値をDecimalOutsideFloatRangeとして拒否します。JSONとTypeScript foreign境界では
DecimalをJS `number` へ暗黙変換せず、既定adapterはcanonical Stringを使います。

### `std/bytes`

`Byte` は0以上255以下の整数だけを表すstandard opaque newtype、`Bytes` はimmutableで有限な連続
byte列を表すstandard opaque typeです。

```seseragi
type ByteError deriving Eq, Show =
  | ByteOutOfRange Int

type BytesSliceError deriving Eq, Show =
  | InvalidByteRange { start: Int, end: Int, length: Int }

fn byte value: Int -> Either<ByteError, Byte>
fn toInt value: Byte -> Int

fn empty -> Bytes
fn singleton value: Byte -> Bytes
fn fromArray values: Array<Byte> -> Bytes
fn fromInts values: Array<Int> -> Either<ByteError, Bytes>
fn toArray values: Bytes -> Array<Byte>
fn toInts values: Bytes -> Array<Int>

fn length values: Bytes -> Int
fn isEmpty values: Bytes -> Bool
fn get index: Int -> values: Bytes -> Maybe<Byte>
fn slice start: Int -> end: Int -> values: Bytes
  -> Either<BytesSliceError, Bytes>
fn copy values: Bytes -> Bytes
fn append suffix: Bytes -> values: Bytes -> Bytes
fn concat values: Array<Bytes> -> Bytes
```

`byte` と `fromInts` は範囲外の最初のIntで `ByteOutOfRange` を返します。`fromInts` は入力Arrayを
変更せず、source順に各値を一度検査します。`get` は負indexまたは範囲外ならNothingです。

`slice start end values` は0-based・end-exclusiveです。`0 <= start <= end <= length` を満たさない場合、
元のstart、end、lengthを持つInvalidByteRangeを返します。成功したsliceはbacking storageを共有して
O(1)にできます。Bytesがimmutableなので共有は観測できず安全です。小さいsliceだけを長期間保持して
大きいbacking storageを解放したいcodeは `copy` を使います。copy結果は内容が等しい独立storageを持ちます。

appendとconcatは入力順を保つ新しいBytesを返します。concatは結果領域を一度だけ確保でき、空Arrayでは
`empty ()` と同じ内容です。length、isEmpty、getはO(1)、copy、append、concatは出力byte数に対してO(n)
です。ByteとBytesは内容に基づくEq、Ord、Hash instanceを持ち、Ordはunsigned byteの辞書式順序です。

UTF-8変換は `std/text` から次を提供します。

```seseragi
type Utf8DecodeError deriving Eq, Show =
  | InvalidUtf8 { offset: Int }

fn encodeUtf8 text: String -> Bytes
fn decodeUtf8 bytes: Bytes -> Either<Utf8DecodeError, String>
fn decodeUtf8Lossy bytes: Bytes -> String
```

decodeUtf8は最初の不正sequenceのbyte offsetを返します。decodeUtf8Lossyは各最大不正subsequenceを
Unicode replacement character U+FFFD一つへ置換します。encodeUtf8はUnicode scalarを標準UTF-8へ
変換し、invalid scalarを生成しません。

TypeScript foreign境界でBytesとUint8Arrayを変換する既定adapterは、引数・戻り値の両方向でcopyします。
mutableなhost viewとのaliasをSeseragi Bytesへ持ち込んではなりません。zero-copy borrowはlifetimeと
mutationを保証できるtarget固有 `unsafe` adapterだけが提供でき、標準foreign変換にはしません。

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

### temporal control

```seseragi
fn timeout<R, E, A>
  duration: Duration
  -> effect: Effect<R, E, A>
  -> Effect<R & { clock: Clock }, E, Maybe<A>>
fn timeoutFail<R, E, A>
  error: E
  -> duration: Duration
  -> effect: Effect<R, E, A>
  -> Effect<R & { clock: Clock }, E, A>
```

`timeout` はeffectが時間内に成功すれば `Just value`、期限到達なら `Nothing` で成功し、sourceのtyped
failureはそのままfailureにします。`timeoutFail` は期限到達を指定した `E` で失敗させます。期限到達時は
sourceへcancellationを要求し、finalizer完了後に結果を返します。source終了とtimerが同じscheduler turnなら
sourceのsuccessまたはfailureを選びます。zero Durationでもsourceを先に開始し、同じtie-breakを使います。

retryとrepeatは同じschedule modelを使います。

```seseragi
type ScheduleDecision deriving Eq, Show =
  | ScheduleStop
  | ScheduleContinue Duration

type ScheduleError deriving Eq, Show =
  | NegativeRecurrences Int

fn schedule<A>
  decide: (Int -> A -> ScheduleDecision) -> Schedule<A>
fn recurs<A> additionalRuns: Int -> Either<ScheduleError, Schedule<A>>
fn spaced<A>
  additionalRuns: Int
  -> delay: Duration
  -> Either<ScheduleError, Schedule<A>>
fn whileInput<A> predicate: (A -> Bool) -> Schedule<A>

fn retry<R, E, A>
  policy: Schedule<E>
  -> effect: Effect<R, E, A>
  -> Effect<R & { clock: Clock }, E, A>
fn repeat<R, E, A>
  policy: Schedule<A>
  -> effect: Effect<R, E, A>
  -> Effect<R & { clock: Clock }, E, A>
```

`Schedule<A>` はstandard opaque typeです。decideのIntは観測したfailureまたはsuccessの回数で、1から
始まります。`ScheduleStop` は現在のfailureを返す、または現在のsuccessを最終結果にします。
`ScheduleContinue delay` はdelay後にEffectをもう一度最初から実行します。decideはpureで各観測につき
一度だけ呼びます。

`recurs n` はdelayなしで最大n回追加実行し、`spaced n delay` は各追加実行前にdelayします。nが負なら
`Left (NegativeRecurrences n)` です。`whileInput predicate` はpredicateがTrueの間、zero Durationで
追加実行します。zero Durationの待機もscheduler checkpointです。retryはsuccessで直ちに終了し、repeatは
failureで直ちに終了します。delay中または再実行中のcancellationは以後の実行を開始しません。

### resource scope

```seseragi
type EffectExit<E, A> =
  | EffectSucceeded A
  | EffectFailed E
  | EffectCancelled

fn bracket<R, E, A, B>
  acquire: Effect<R, E, A>
  -> use: (A -> Effect<R, E, B>)
  -> release: (A -> EffectExit<E, B> -> Effect<R, Never, Unit>)
  -> Effect<R, E, B>
fn acquireRelease<R, E, A>
  acquire: Effect<R, E, A>
  -> release: (A -> Effect<R, Never, Unit>)
  -> Effect<R, E, A>
fn ensuring<R, E, A>
  finalizer: Effect<R, Never, Unit>
  -> effect: Effect<R, E, A>
  -> Effect<R, E, A>
fn onCancel<R, E, A>
  finalizer: Effect<R, Never, Unit>
  -> effect: Effect<R, E, A>
  -> Effect<R, E, A>
fn scoped<R, E, A> effect: Effect<R, E, A> -> Effect<R, E, A>
```

`bracket` はacquireが成功した場合だけreleaseを一度実行し、useの終了状態を `EffectExit` で渡します。
release完了後にuseのsuccess、failure、cancellationを再現します。`acquireRelease` はacquireした値を現在の
scopeへ登録して返し、そのscope終了時にreleaseします。`ensuring` はすべての終了状態でfinalizerを実行し、
`onCancel` はcancellation時だけ実行します。

`scoped` は内側scopeを作り、登録resourceとchild Fiberを5.11の規則で閉じます。同じscopeのfinalizerは
登録と逆順です。inner scopeを閉じ終わるまでouter scopeの次のfinalizerへ進みません。finalizerのtyped
failure禁止、cancellation mask、defect時の継続規則も5.11に従います。

### concurrency

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
```

`Fiber<E, A>` はstandard opaque typeです。`race` は最初に終了したsuccessまたはfailureを返し、loserへ
cancellationを要求してfinalizer完了を待ちます。同じscheduler turnで両方が終了した場合はleftを
選びます。

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

Streamのenvironment requirementはEffectと同じ5.5のwideningを持ちます。また
`Stream<R, Never, A>` は `Stream<R, E, A>` が必要な位置へfailure wideningできます。success型や
Never以外のerror型にはvarianceを持たず、異なるerrorは `mapError` で明示的に揃えます。

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

`std/process` はhost processを表す `Process` serviceと、portableなsignal分類を提供します。
canonical requirement名は `process` で、`effect fn` の `with Process` は
`with process: Process` と同じです。

```seseragi
type ProcessSignal deriving Eq, Ord, Show =
  | Interrupt
  | Terminate
  | Hangup
  | Quit
  | User1
  | User2

type ProcessError deriving Eq, Show =
  | UnsupportedProcessSignal ProcessSignal
  | ReservedProcessSignal ProcessSignal
  | InvalidArgumentEncoding Int
  | InvalidEnvironmentName String
  | InvalidEnvironmentEncoding String
  | CurrentDirectoryUnavailable

fn arguments -> Effect<{ process: Process }, ProcessError, Array<String>>
fn environment name: String
  -> Effect<{ process: Process }, ProcessError, Maybe<String>>
fn currentDirectory
  -> Effect<{ process: Process }, ProcessError, String>
fn signals watched: NonEmptyList<ProcessSignal>
  -> Stream<{ process: Process }, ProcessError, ProcessSignal>
```

`arguments ()` はexecutable名を含めないapplication引数をsource順に返します。`environment name` は
存在しないkeyを `Nothing`、存在する値を `Just value` にします。host byte列をUnicode Stringへ
変換できない場合は位置またはkeyを持つProcessErrorです。environment全体をMapとしてsnapshotするAPIは
coreへ置かず、必要なkeyだけを明示的に読みます。

`signals watched` はterminal operationごとにhandlerを登録するcold Streamで、scope終了時に解除します。
targetが扱えないsignalはStream開始時に `UnsupportedProcessSignal` で失敗します。handler登録後のsignalは
hostの観測順に出します。同じ種類が未消費の間に繰り返された場合は一件へcoalesceし、異なる種類は最初に
観測した順を保ちます。watched内の重複は最初の一件だけを使います。Stream開始が失敗した場合はhandlerを
一つも残さず、正常に登録したhandlerもscope終了時に以前のhost設定へ戻します。OS signal handler内で
user codeを直接実行しません。

### termination signalとgraceful shutdown

`run.signal_mode` が `cancel` の場合、InterruptとTerminateはhostが予約し、`signals` で要求すると
`ReservedProcessSignal` になります。最初のInterruptまたはTerminateでhostはroot main scopeへ
cooperative cancellationを要求し、child Fiberとfinalizerの完了を最大 `shutdown_grace_ms` 待ちます。
時間内に完了すればgraceful shutdownです。

grace period中の二回目のInterrupt / Terminate、またはgrace period超過はforced terminationです。
forced terminationでは未完了finalizer、buffer flush、foreign callback完了を保証しません。hostは可能なら
stderrへforced termination diagnosticを一度出します。

`run.signal_mode` が `forward` の場合、hostは最初のtermination signalを `signals` subscriberへ渡し、
rootを自動cancelしません。applicationはsignal Streamと長時間処理を `race` するなどしてmainを正常終了
させます。二回目のInterrupt / Terminateだけはforced terminationです。subscriberがないsignalも一回目は
無視されるため、forward modeの選択はapplicationの責任です。

forward modeでsignalを受けたapplicationがmainをUnit successとして閉じた場合はcode 0です。signalを
受け取った事実だけで130 / 143へ変えません。130 / 143はcancel modeでhostがrootをcancelした場合、または
二回目のsignalでforced terminationした場合に使います。

process-capable targetの終了codeは次で固定します。

| root mainの終了理由             | exit code |
| ------------------------------- | --------: |
| Unit success                    |         0 |
| 未処理typed failure             |         1 |
| runtime defect                  |        70 |
| host cancellation（signalなし） |       130 |
| Interruptによる終了             |       130 |
| Terminateによる終了             |       143 |

typed failureではhostが `show error` の結果をstderrへ一件のmessageとして出してからcode 1で終了します。
文字列末尾にnewlineがなければ一つ加え、内部newlineは保持します。defectはruntime diagnosticと
stack traceを出し、Showを使いません。
signalを持たないtargetは同じ分類をhost resultとして報告し、架空のprocess codeをapplicationへ公開
しません。通常関数から現在processを即時終了するAPIは提供しません。

child process APIは現在processのshutdown APIと分離し、childのexit statusを値として返します。
child processの完全な公開signatureはfilesystem / Bytes contractと一緒に固定します。

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

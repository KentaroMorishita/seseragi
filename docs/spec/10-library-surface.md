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

値を変換・検索・集約する関数は、pipelineのsubjectを最後のparameterに置きます。複数のresourceや
destinationを扱うI/O関数では、最後のparameterをprimary subjectとして本文で明示します。たとえば
`writeChunks mode source path`はpathへのwrite、`runStreaming input command`はcommandの実行、
`exchange body request`はrequestのexchangeです。

異なるfailure sourceを一つのchannelへ保つ場合、upstreamを実行するadapterは
`Either<UpstreamError, LocalError>`を使い、Leftをupstream、Rightをadapter / host failureとします。
resourceを獲得してuser callbackへ渡すbracket型APIは`Either<ResourceError, UseError>`を使い、Leftを
acquire / release、Rightをcallback failureとします。通常のunion error型へ暗黙統合しません。

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

`empty` はparameterなしgeneric関数です。ほかに次を提供します。

```seseragi
fn empty<A> -> Array<A>
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

`empty` はparameterなしgeneric関数です。ほかに次を提供します。

```seseragi
fn empty<A> -> List<A>
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
- `take count` は `count <= 0` なら`empty ()`、長さ以上なら元と同じ要素列。`drop count` は
  `count <= 0` なら元と同じ要素列、長さ以上なら`empty ()`。
- `zip` と `zipWith` は短い側の長さで停止する。`append suffix values` とpipelineで書いた
  `values |> append suffix` は、valuesの後ろへsuffixを連結する。
- `sort` と `sortBy` はstable sort。`sortBy` はkeyをsource順に各要素一度だけ評価する。
- `groupBy` はkeyをsource順に各要素一度だけ評価し、Mapのkeyを初出順、各groupの要素を
  source順に保つ。
- 負indexと範囲外indexの `get` は `Nothing`。空collectionの `head`、`last`、`init`、`tail` は
  `Nothing`。singletonの `init` と `tail` は `Just (empty ())`。
- `chunksOf size` は隣接する重複なしのchunkへ分割し、最後だけsize未満でも残す。
  `windows size` は幅sizeの隣接する重複ありwindowを一要素ずつずらす。sizeが長さを超える場合は
  `empty ()`を返す。どちらも `size <= 0` なら `Left (NonPositiveSize size)`。

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

`std/map` は `Map<K, V>` 型を公開します。`empty`を含む次の関数を提供します。

```seseragi
fn empty<K, V> -> Map<K, V>
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

`empty ()` は期待型または後続operationからKとVを推論します。
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

`std/set` は `Set<A>` 型を公開します。`empty`を含む次の関数を提供します。

```seseragi
fn empty<A> -> Set<A>
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

type ReduceStep<A> =
  | Next A
  | Done A

fn reduceUntil<C, A, B>
  initial: B
  -> step: (B -> A -> ReduceStep<B>)
  -> values: C
  -> B
where Iterable<C, A>
```

methodの型schemeは次です。

```text
iterate : forall C A. Iterable<C, A> => C -> Iterator<A>
reduce  : forall C A B. Reducible<C, A> => B -> (B -> A -> B) -> C -> B
traverse: forall F G A B. Traversable<F>, Applicative<G>
          => (A -> G<B>) -> F<A> -> G<F<B>>
reduceUntil: forall C A B. Iterable<C, A>
             => B -> (B -> A -> ReduceStep<B>) -> C -> B
```

Iterableは順序付き反復、Reducibleは有限collectionの集約、TraversableはApplicative effectを
保った走査を意味します。`reduce`は先頭から末尾へ処理し、initialを必須とします。
`reduceUntil`は`std/collection`のgeneric関数で、trait methodではありません。stepが`Next next`を
返す間は次へ進み、`Done result`で後続要素を評価せずresultを返します。空ならinitialを返します。
Iterableだけを要求するため有限性を仮定せず、無限IteratorでもDoneへ到達すれば終了できます。

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

numbers
  |> reduceUntil 0 (\total number ->
       if number < 0 then Done total else Next (total + number))

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
```

数値のparseはtext操作ではないため、`std/int.parse` と `std/float.parse` を使います。
`std/text` は同名の別実装や暗黙trimを提供しません。

`std/char`はCharとcode pointの明示変換を提供します。

```seseragi
fn codePoint value: Char -> Int
fn fromCodePoint value: Int -> Maybe<Char>
fn toString value: Char -> String
```

`fromCodePoint`は0から`0x10FFFF`の範囲外またはsurrogateならNothingです。`toString`はscalar一個を
含むStringを返します。`std/text.scalarAt`はscalar indexを取り、範囲外ならNothingを返します。

```seseragi
fn scalarAt index: Int -> text: String -> Maybe<Char>
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

`Int` はsigned 64-bit、`Float` はIEEE 754 binary64です。通常のInt演算はoverflowをruntime defectに
しますが、境界を通常値として扱うcodeには `std/int` のchecked、saturating、wrapping operationを
提供します。これらはhost languageの整数表現へ意味を委譲しません。FloatのNaN、infinity、negative zero、
total orderingも名前付きAPIで扱います。

任意精度整数 `BigInt`、十進数 `Decimal`、byte列 `Bytes` は標準moduleとして提供し、preludeには
入れません。UTF-8、filesystem、HTTP bodyはArray<Int>ではなくBytesを使います。

`std/number` は丸め方向を取る標準APIで共有する次の型をexportします。

```seseragi
type RoundingMode deriving Eq, Show =
  | HalfEven
  | HalfUp
  | TowardZero
  | AwayFromZero
  | Floor
  | Ceiling
```

### `std/int`

```seseragi
type IntParseError deriving Eq, Show =
  | EmptyInt
  | InvalidIntRadix Int
  | InvalidIntDigit { offset: Int, radix: Int }
  | IntOutsideRange

type IntDivisionError deriving Eq, Show =
  | IntDivisionByZero
  | IntDivisionOverflow

type IntPowerError deriving Eq, Show =
  | NegativeIntExponent Int
  | IntPowerOverflow

fn minValue -> Int
fn maxValue -> Int

fn parse text: String -> Either<IntParseError, Int>
fn parseRadix radix: Int -> text: String -> Either<IntParseError, Int>
fn format value: Int -> String
fn formatRadix radix: Int -> value: Int -> Either<IntParseError, String>

fn checkedAdd right: Int -> left: Int -> Maybe<Int>
fn checkedSubtract right: Int -> left: Int -> Maybe<Int>
fn checkedMultiply right: Int -> left: Int -> Maybe<Int>
fn checkedNegate value: Int -> Maybe<Int>
fn checkedAbs value: Int -> Maybe<Int>
fn checkedDivide divisor: Int -> dividend: Int
  -> Either<IntDivisionError, Int>
fn checkedRemainder divisor: Int -> dividend: Int
  -> Either<IntDivisionError, Int>
fn checkedPower exponent: Int -> base: Int -> Either<IntPowerError, Int>

fn saturatingAdd right: Int -> left: Int -> Int
fn saturatingSubtract right: Int -> left: Int -> Int
fn saturatingMultiply right: Int -> left: Int -> Int
fn saturatingNegate value: Int -> Int
fn saturatingAbs value: Int -> Int
fn saturatingPower exponent: Int -> base: Int -> Either<IntPowerError, Int>

fn wrappingAdd right: Int -> left: Int -> Int
fn wrappingSubtract right: Int -> left: Int -> Int
fn wrappingMultiply right: Int -> left: Int -> Int
fn wrappingNegate value: Int -> Int
fn wrappingAbs value: Int -> Int
fn wrappingPower exponent: Int -> base: Int -> Either<IntPowerError, Int>

fn minimum right: Int -> left: Int -> Int
fn maximum right: Int -> left: Int -> Int
fn clamp lower: Int -> upper: Int -> value: Int -> Int
fn sign value: Int -> Int
```

`parse` はASCIIの `[+-]?(0|[1-9][0-9]*)` だけを受理します。`parseRadix` は2以上36以下のradixと
`[+-]?[0-9A-Za-z]+` を受け、radixを超えるdigitを拒否します。どちらも空白、`_`、`0x` などの
literal prefixを受理しません。InvalidIntDigitのoffsetは最初の不正UTF-8 byteです。構文が正しくても
signed 64-bit範囲外ならIntOutsideRangeです。formatはprefixもseparatorも持たないcanonical表記を返し、
formatRadixの10以上のdigitはlowercaseです。zeroは常に `"0"` で、`"-0"` を生成しません。

checkedAddなどoverflowだけが失敗原因のoperationはNothingを返します。divisionは0除算に加え、
`minValue () / -1` と同じ演算で表現範囲を超える場合をIntDivisionOverflowにします。remainderも同じ
operand pairをoverflowとして拒否します。商は0方向へ切り捨て、remainderはdividendと同符号です。
powerのexponentは0以上で、`0 ^ 0` は1です。

saturating operationは正確な数学結果を最小値または最大値へclampします。wrapping operationは64-bit
two's-complementの下位64 bitをIntとして解釈します。negative exponentはsaturating / wrappingでも
NegativeIntExponentであり、checkedPowerだけがoverflowをIntPowerOverflowにします。minimum、maximum、
clampは通常のInt順序を使い、`lower > upper` のclampはprogrammer errorとしてruntime defectです。
signは負なら-1、zeroなら0、正なら1です。

### `std/float`

```seseragi
type FloatParseError deriving Eq, Show =
  | EmptyFloat
  | InvalidFloat { offset: Int }
  | FloatParseOverflow

type FloatConversionError deriving Eq, Show =
  | FloatNotFinite
  | FloatOutsideIntRange

fn nan -> Float
fn positiveInfinity -> Float
fn negativeInfinity -> Float

fn parse text: String -> Either<FloatParseError, Float>
fn format value: Float -> String
fn fromInt value: Int -> Float
fn fromIntExact value: Int -> Maybe<Float>
fn toInt rounding: RoundingMode -> value: Float
  -> Either<FloatConversionError, Int>

fn isNaN value: Float -> Bool
fn isFinite value: Float -> Bool
fn isInfinite value: Float -> Bool
fn isNegativeZero value: Float -> Bool
fn ieeeEq left: Float -> right: Float -> Bool
fn totalCompare left: Float -> right: Float -> Ordering
fn minimumNumber right: Float -> left: Float -> Float
fn maximumNumber right: Float -> left: Float -> Float
fn clampNumber lower: Float -> upper: Float -> value: Float -> Maybe<Float>
fn abs value: Float -> Float
fn sign value: Float -> Maybe<Int>
fn power exponent: Float -> base: Float -> Float
fn roundIntegral rounding: RoundingMode -> value: Float -> Float
```

`parse` は数値literalと同じdecimal grammarから `_` を除いたsyntax、およびcase-sensitiveな `NaN`、
`Infinity`、`-Infinity` だけを受理します。前後の空白は受理しません。有限な綴りがinfinityへoverflow
する場合はFloatParseOverflow、invalid syntaxは最初の不正UTF-8 byte offsetです。zeroへのunderflowは
符号を保ちます。

`format` はfinite値を同じbinary64へround-tripする最短のdecimal ASCIIで返します。整数に見える有限値にも
`.0` を一つ付けてFloat表記であることを保ち、negative zeroは `"-0.0"` です。指数を使う場合はlowercase
`e`、指数のplus signなし、先頭zeroなしとします。特殊値は `"NaN"`、`"Infinity"`、`"-Infinity"` です。
NaN payloadとsignはcanonical outputへ保存しません。

fromIntはIEEE ties-to-evenで変換し、fromIntExactは往復して同じIntになる場合だけJustです。toIntは指定modeで
整数へ丸めてから範囲を検査します。roundIntegralはFloatのまま整数値へ丸め、NaN、infinity、zeroの符号を
保存します。HalfUpとAwayFromZeroの意味は上のRoundingMode定義に従います。

Floatの通常のEqとOrdは提供しません。ieeeEqはNaNとの比較を常にFalse、`-0.0` と `0.0` の比較をTrueに
します。totalCompareはIEEE totalOrderに従い、すべてのNaNを
一つのcanonical NaNとして扱います。minimumNumber / maximumNumberは片方だけがNaNなら他方を返し、両方NaN
ならcanonical NaNです。zero同士ではminimumNumberがnegative zero、maximumNumberがpositive zeroを返します。
clampNumberはboundかvalueがNaN、またはtotalCompareでlowerがupperより大きければNothingです。absはNaNを
canonical NaNへし、signはNaNだけNothing、negative zeroを含むzeroはJust 0です。powerはIEEE 754 / ECMAScript
Math.pow互換の特殊値表に従い、backend差を許しません。

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

### `std/bytes/hex` と `std/bytes/base64`

binaryとportable textの変換はBytes本体と分離したsubmoduleで提供します。

```seseragi
type HexDecodeError deriving Eq, Show =
  | OddHexLength Int
  | InvalidHexDigit { offset: Int }

fn encode bytes: Bytes -> String
fn decode text: String -> Either<HexDecodeError, Bytes>
```

`std/bytes/hex.encode` は各byteをlowercaseのASCII 2桁で出し、prefixやseparatorを付けません。decodeは
ASCII `[0-9A-Fa-f]*` を受理し、大小どちらも同じBytesへ変換します。奇数長ならUTF-8 byte lengthを
OddHexLengthへ、非ASCIIを含む不正digitなら最初のbyte offsetをInvalidHexDigitへ入れます。空Stringと
empty Bytesは相互に変換されます。したがってencode結果は一意ですが、decode入力はuppercaseも許します。

```seseragi
type Base64DecodeError deriving Eq, Show =
  | InvalidBase64Length Int
  | InvalidBase64Digit { offset: Int }
  | InvalidBase64Padding { offset: Int }
  | NonCanonicalBase64Bits { offset: Int }

fn encode bytes: Bytes -> String
fn decode text: String -> Either<Base64DecodeError, Bytes>
fn encodeUrl bytes: Bytes -> String
fn decodeUrl text: String -> Either<Base64DecodeError, Bytes>
```

`std/bytes/base64.encode` / decodeはRFC 4648のstandard alphabetを使います。encodeは長さを4の倍数にし、
必要なら末尾へ一つまたは二つの `=` を付けます。decodeはこのpadded canonical formだけを受理し、空白、
改行、URL-safe alphabet、padding省略を許しません。paddingは末尾だけに置け、不要または過剰なpaddingと、
最後の有効sextetにzeroでない未使用bitがある入力を拒否します。

encodeUrl / decodeUrlはRFC 4648 URL and Filename Safe alphabetの `-` と `_` を使い、paddingを一切
出力・受理しません。長さの4余りが1ならInvalidBase64Lengthです。standard alphabetは受理しません。
すべてのoffsetは0-based UTF-8 byteで、length errorは入力のbyte length、padding / trailing bit errorは
その原因となる最初のASCII byteを指します。decoderは最初のerrorで停止し、部分的なBytesを返しません。
両variantとも `decode (encode bytes)` が元のBytesと等しく、encode outputは同じBytesに対して一意です。

### `std/text/grapheme`

Stringの通常APIはUnicode scalar indexを使います。人が一文字として知覚する単位でcursor、truncate、表示上の
sliceを行うcodeはextended grapheme cluster APIを明示的に使います。

```seseragi
type GraphemeSliceError deriving Eq, Show =
  | InvalidGraphemeRange { start: Int, end: Int, length: Int }

fn length text: String -> Int
fn clusters text: String -> Array<String>
fn byteBoundaries text: String -> Array<Int>
fn at index: Int -> text: String -> Maybe<String>
fn slice start: Int -> end: Int -> text: String
  -> Either<GraphemeSliceError, String>
```

分割はtoolchain metadataが指定するUnicode versionのUAX #29 extended grapheme cluster規則に従います。
clustersの各要素は元Stringの空でない連続substringで、concatするとbyte単位で元へ戻ります。
byteBoundariesは0からUTF-8 byte lengthまでを昇順に返し、空Stringでは `[0]`、それ以外ではcluster数+1件です。
atは0-based cluster index、sliceは0-based・end-exclusiveです。負indexや範囲外のatはNothing、
`0 <= start <= end <= length` を満たさないsliceは元の値を持つInvalidGraphemeRangeです。分割はUnicode
normalizationを行わず、同じ見た目の異なるscalar列を同一化しません。

### `std/text/unicode`

```seseragi
type NormalizationForm deriving Eq, Show =
  | NFC
  | NFD
  | NFKC
  | NFKD

type UnicodeGeneralCategory deriving Eq, Ord, Show =
  | UppercaseLetter | LowercaseLetter | TitlecaseLetter
  | ModifierLetter | OtherLetter
  | NonspacingMark | SpacingMark | EnclosingMark
  | DecimalNumber | LetterNumber | OtherNumber
  | ConnectorPunctuation | DashPunctuation
  | OpenPunctuation | ClosePunctuation
  | InitialPunctuation | FinalPunctuation | OtherPunctuation
  | MathSymbol | CurrencySymbol | ModifierSymbol | OtherSymbol
  | SpaceSeparator | LineSeparator | ParagraphSeparator
  | Control | Format | PrivateUse | Unassigned

fn version -> String
fn normalize form: NormalizationForm -> text: String -> String
fn isNormalized form: NormalizationForm -> text: String -> Bool
fn generalCategory value: Char -> UnicodeGeneralCategory
fn isAlphabetic value: Char -> Bool
fn isWhitespace value: Char -> Bool
fn isDecimalDigit value: Char -> Bool
fn isMark value: Char -> Bool
fn simpleCaseFold value: Char -> Char
fn fullCaseFold text: String -> String
```

normalizeとisNormalizedは指定Unicode versionのcanonical / compatibility normalizationを実装し、
localeやhost ICU versionへ依存しません。normalizeはidempotentです。StringとCharはsurrogateを含まないため、
UnicodeGeneralCategoryにSurrogateはありません。property predicateはUnicode derived core propertyと
White_Space / General_Category tableを使います。simpleCaseFoldは一scalarから一scalarへのdefault case
folding、fullCaseFoldは複数scalarへの展開を含むdefault full case foldingです。どちらもlocale非依存で、
`std/text.caseFold` はfullCaseFoldと同じ結果を返します。versionはcompiler、runtime、formatter、LSPが
artifact metadataで共有する `major.minor.patch` のUnicode data versionです。

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

type JsonPathSegment deriving Eq, Show =
  | JsonField String
  | JsonIndex Int

type DecodeErrorKind deriving Eq, Show =
  | ExpectedJsonType String
  | MissingJsonField String
  | UnknownJsonField String
  | UnknownJsonTag String
  | InvalidJsonValue String

struct DecodeError deriving Eq, Show {
  path: Array<JsonPathSegment>,
  kind: DecodeErrorKind
}

type JsonParseError deriving Eq, Show =
  | InvalidJsonSyntax { offset: Int, message: String }
  | DuplicateJsonField { path: Array<JsonPathSegment>, field: String }

type JsonReadError deriving Eq, Show =
  | JsonSyntaxFailure JsonParseError
  | JsonDecodeFailure DecodeError

trait JsonEncode<A> {
  fn encodeJson value: A -> Json
}

trait JsonDecode<A> {
  fn decodeJson value: Json -> Either<DecodeError, A>
}

fn parse text: String -> Either<JsonParseError, Json>
fn stringify value: Json -> String
fn encodeString<A> value: A -> String
where JsonEncode<A>
fn decodeString<A> text: String -> Either<JsonReadError, A>
where JsonDecode<A>
```

field、optionalField、index、array、record、oneOf、map、flatMap combinatorも提供します。decoder errorのpathは
rootからfailure位置までのfield / index順です。combinatorはpathを外側から一件ずつ加え、最も内側の
DecodeErrorKindを保持します。

parseはRFC 8259のUTF-8 JSONを受理し、numberをexact Decimalとして読みます。NaN、infinity、comment、trailing
commaを受理しません。objectの重複fieldは後勝ちにせずDuplicateJsonFieldです。offsetは0-based UTF-8 byteです。
stringifyは空白なし、objectのMap挿入順、array順、canonical Decimal表記、必要最小限のJSON escapeで出力します。
同じJson valueから同じbytesを生成しますが、MapのEqが順序を無視するためEqなobject同士のcanonical hashを
保証するものではありません。

Stringはquoteとbackslashをescapeし、U+0008 / 0009 / 000A / 000C / 000Dをそれぞれ `\b`、`\t`、`\n`、
`\f`、`\r`、残りのU+0000..001Fをuppercase 4桁 `\u00XX` で出します。それ以外のUnicode scalarはUTF-8
literalで出し、`/` やnon-ASCIIを任意にescapeしません。

Bool、String、Int、Decimal、Unit、Json、Maybe、Either、Array、List、tuple、Map、Set、structural recordにstandard
JsonEncode / JsonDecode instanceを提供します。UnitとNothingはJsonNull、Justは内容、Eitherは
`{"tag":"Left","value":...}` / `{"tag":"Right","value":...}` です。FloatはNaN / infinityとdecimal
round-trip policyが一意でないためstandard instanceを持たず、Decimalまたは明示converterを使います。

Map<String, V>はobject、ほかのMap<K, V>はentryの2要素Array列、SetはArrayとして10.5の順序と重複規則を使います。
structural recordのfieldはUnicode scalarの辞書順でencodeし、decodeでは全fieldを必須として未知fieldを拒否します。
`optionalField` を使う手書きDecoderだけが欠落を許可します。Maybe fieldもdefault derivingでは必須で、Nothingは
missingではなく明示JsonNullです。

structural record typeで `field?: A` と宣言されたoptional fieldだけは、absentならobject fieldを省略し、presentなら
Aのcodecでencodeします。decodeではmissingをabsentとして受理し、presentなJsonNullをabsenceへ変換しません。
requiredなMaybe<A> fieldは従来どおり必須で、NothingをJsonNullとしてencodeします。これによりschema上のmissingと
explicit nullを区別します。

named structのderived codecはfield名をsource spellingのまま使い、declaration順でencodeします。decodeは全fieldを
一度要求し、未知fieldを拒否してconstructorを呼びます。ADTは次の表現です。

- payloadなし: `{"tag":"Idle"}`
- payloadあり: `{"tag":"Loaded","value":...}`

tagはconstructor名そのものでcase-sensitiveです。payloadがtupleまたはrecordでも常にvalue field一つの中へencode
し、wrapper fieldとflattenしません。newtypeのderived codecは内部値へtransparentに委譲します。opaque宣言でも
derivingを選んだmoduleがinstanceを公開すればcodec利用は表現を直接公開せず許可されます。

generic derivingはfield / payloadから必要なJsonEncode<A> / JsonDecode<A> constraintを生成します。recursive ADTは
constructorを一段decodeしてから再帰するguarded recursionだけを許可し、aliasだけを循環するnon-productive codecを
拒否します。同じ型へexplicit instanceとderivingを両方定義できません。wire名のrename、default、unknown-field許可、
version migrationが必要な型はderivingへannotationを足さず、通常のexplicit instance / Decoder combinatorで定義します。

encodeStringはencodeJson後にstringify、decodeStringはparse後にdecodeJsonします。JsonReadErrorはsyntaxとdomain
decodeを混同しません。

`Js.Unknown` からapplication型へ直接castせず、JSONまたはdomain decoderを通します。

## 10.10 `std/time` と `std/random`

time moduleはtimeline上の時点、calendar上の壁時計、固定offset、timezone ruleを別の型にします。
`Instant`、`Duration`、`LocalDate`、`LocalTime`、`LocalDateTime`、`UtcOffset`、`OffsetDateTime`、
`TimeZone`、`ZonedDateTime` は相互に暗黙変換しないstandard opaque typeです。

InstantはUnix epoch `1970-01-01T00:00:00Z` からの整数nanosecondによるPOSIX timelineです。UTC leap
secondを独立した時点として表さず、LocalTimeのsecondは0から59です。calendarはastronomical year
numberingを持つproleptic Gregorian calendarです。Durationは0以上 `2^63 - 1` 以下の整数nanosecond数で、
負値と範囲外値は構築できません。

```seseragi
type DateTimeError deriving Eq, Show =
  | InvalidDate { year: Int, month: Int, day: Int }
  | InvalidTime { hour: Int, minute: Int, second: Int, nanosecond: Int }
  | InvalidUtcOffsetSeconds Int
  | InvalidDateTimeText { offset: Int }

type DurationError deriving Eq, Show =
  | NegativeDuration Int
  | DurationOutsideRange

type TimeZoneError deriving Eq, Show =
  | UnknownTimeZone String
  | TimeZoneDatabaseUnavailable String
  | TimeZoneDatabaseVersionMismatch { required: String, actual: String }

type LocalResolution deriving Eq, Show =
  | Unique ZonedDateTime
  | Ambiguous { earlier: ZonedDateTime, later: ZonedDateTime }
  | Gap {
      transition: Instant,
      offsetBefore: UtcOffset,
      offsetAfter: UtcOffset
    }

fn zeroDuration -> Duration
fn nanoseconds value: Int -> Either<DurationError, Duration>
fn milliseconds value: Int -> Either<DurationError, Duration>
fn seconds value: Int -> Either<DurationError, Duration>
fn minutes value: Int -> Either<DurationError, Duration>
fn hours value: Int -> Either<DurationError, Duration>
fn toNanoseconds value: Duration -> Int
fn addDuration right: Duration -> left: Duration
  -> Either<DurationError, Duration>

fn localDate year: Int -> month: Int -> day: Int
  -> Either<DateTimeError, LocalDate>
fn localTime hour: Int -> minute: Int -> second: Int -> nanosecond: Int
  -> Either<DateTimeError, LocalTime>
fn localDateTime date: LocalDate -> time: LocalTime -> LocalDateTime
fn utcOffset seconds: Int -> Either<DateTimeError, UtcOffset>

fn parseLocalDate text: String -> Either<DateTimeError, LocalDate>
fn parseLocalTime text: String -> Either<DateTimeError, LocalTime>
fn parseLocalDateTime text: String -> Either<DateTimeError, LocalDateTime>
fn parseOffsetDateTime text: String -> Either<DateTimeError, OffsetDateTime>

fn formatLocalDate value: LocalDate -> String
fn formatLocalTime value: LocalTime -> String
fn formatLocalDateTime value: LocalDateTime -> String
fn formatOffsetDateTime value: OffsetDateTime -> String

fn atOffset offset: UtcOffset -> instant: Instant -> OffsetDateTime
fn offsetInstant value: OffsetDateTime -> Instant
fn offsetLocalDateTime value: OffsetDateTime -> LocalDateTime

fn databaseVersion
  -> Effect<{ timeZones: TimeZones }, Never, String>
fn loadTimeZone id: String
  -> Effect<{ timeZones: TimeZones }, TimeZoneError, TimeZone>
fn timeZoneId zone: TimeZone -> String
fn timeZoneVersion zone: TimeZone -> String
fn atTimeZone instant: Instant -> zone: TimeZone -> ZonedDateTime
fn resolveLocal local: LocalDateTime -> zone: TimeZone -> LocalResolution
fn zonedInstant value: ZonedDateTime -> Instant
fn zonedLocalDateTime value: ZonedDateTime -> LocalDateTime
fn zonedOffset value: ZonedDateTime -> UtcOffset
fn zonedTimeZone value: ZonedDateTime -> TimeZone
```

duration constructorは単位をnanosecondへexactに変換します。負入力は元の値をNegativeDurationへ、単位変換または
加算が `2^63 - 1` nanosecondsを超える場合はDurationOutsideRangeを返します。zeroDurationは0 nanosecondsです。
Durationは値に基づくEq、Ord、Hashを持ち、implicitなInt変換やFloat second constructorを持ちません。

date/time parserはASCIIのextended ISO 8601 subsetだけを受理します。LocalDateはyear、month、dayを
`-` で区切ります。year 0から9999は4桁、それ以外は符号と6桁以上のexpanded yearです。
LocalTimeは `HH:mm:ss` と省略可能な1〜9桁の小数second、LocalDateTimeはdateとtimeを `T` で連結した
形式です。OffsetDateTimeは末尾に `Z` または `+HH:mm` / `-HH:mm` を要求します。空白、locale依存表記、
timezone abbreviationを受理せず、error offsetはUTF-8 byte offsetです。formatは同じsyntaxのcanonical
表記を返し、小数secondの末尾0を除き、zero offsetは `Z` にします。UtcOffsetは絶対値18時間以下で、
18時間の場合minuteとsecondが0の値だけを許可します。

TimeZone IDはIANA tz databaseのcase-sensitiveなcanonical zone nameです。abbreviationと現在のsystem
timezoneから推測せず、link aliasを受理した場合も `timeZoneId` はdatabase内のcanonical targetを返します。
TimeZone valueはload時のrule snapshotとdatabase versionを保持するため、後からserviceが更新されてもpureな
`atTimeZone` と `resolveLocal` の結果は変わりません。

tzdbの最初のtransitionより前は最初のnon-DST standard offset、最後のtransitionより後はdatabaseに含まれる
POSIX continuation ruleを使います。continuation ruleがないzoneは最後のnon-DST standard offsetを使います。
このextrapolationもtzdb versionの一部で、host APIの範囲や現在日時によって結果を変えません。

resolveLocalはDSTなどによる壁時計の重複をAmbiguous、欠落をGapとして返し、暗黙にearlier / laterを選んだり
gapを前後へshiftしたりしません。Ambiguousの二値はInstantの昇順です。Gap.transitionはoffset変更が発生する
最初のInstantで、offsetBefore / offsetAfterはその前後のoffsetです。policyを持つapplication helperはこのADTを
matchしてdomain固有の選択を明示します。

TimeZones serviceはexactなIANA release IDを公開します。applicationとtestは `seseragi.lock` が固定したversionを
要求し、default runtime serviceのversionが違えばapplication code開始前に
TimeZoneDatabaseVersionMismatchです。testは小さなsynthetic rule setを持つserviceへ差し替えられます。
Instant、fixed offsetだけを使う処理はTimeZonesを要求しません。
`with TimeZones` はcanonical requirement `{ timeZones: TimeZones }` へ展開します。

serialized ZonedDateTimeはInstant、canonical zone ID、tzdb versionを保持します。decode時に同じversionが
なければ黙って現在ruleで再解釈しません。明示的なrebase adapterだけが別versionのTimeZoneへInstantを
投影できます。LocalDateTimeだけのserializationにはtimezoneもoffsetも存在しないため、後から推測しません。

現在時刻とsleepはClock serviceを要求します。

```seseragi
fn now -> Effect<{ clock: Clock }, ClockError, Instant>
fn sleep duration: Duration -> Effect<{ clock: Clock }, Never, Unit>
```

疑似乱数とcryptographic entropyは別serviceです。Randomは再現可能なsimulation、sampling、shuffle用で、
秘密鍵、session token、nonce、saltへ使ってはなりません。Entropyだけが秘密用途のhost CSPRNGを表します。
pure関数がglobal random sourceを読みません。

```seseragi
type RandomRangeError deriving Eq, Show =
  | EmptyRandomIntRange { lower: Int, upperExclusive: Int }
  | InvalidProbability Float

type RandomConfigError deriving Eq, Show =
  | NonPositiveRandomSize Int
  | RandomSizeTooLarge Int

opaque type RandomSize

fn randomSize bytes: Int -> Either<RandomConfigError, RandomSize>

fn algorithmId
  -> Effect<{ random: Random }, Never, String>
fn nextBool
  -> Effect<{ random: Random }, Never, Bool>
fn nextInt
  -> Effect<{ random: Random }, Never, Int>
fn intBetween lower: Int -> upperExclusive: Int
  -> Effect<{ random: Random }, RandomRangeError, Int>
fn unitFloat
  -> Effect<{ random: Random }, Never, Float>
fn chance probability: Float
  -> Effect<{ random: Random }, RandomRangeError, Bool>
fn randomBytes size: RandomSize
  -> Effect<{ random: Random }, Never, Bytes>
fn choose<A> values: NonEmptyList<A>
  -> Effect<{ random: Random }, Never, A>
fn shuffle<A> values: Array<A>
  -> Effect<{ random: Random }, Never, Array<A>>
```

Randomのcanonical requirement名は`random`です。各operationは必要なoutput数だけservice stateをatomicに進めます。同じseed、
algorithmId、同じ逐次call列はbackendによらず同じ結果です。共有Randomへ複数Fiberから同時callした場合は
linearizableですが、どのFiberが先にstateを取るかはscheduler依存です。並列testでsequenceを固定したい場合は
caseごとに独立serviceを提供し、一つのserviceをFiber間共有しません。

standard algorithm IDは`seseragi-xoshiro256ss-v1`です。signed 64-bit seedをtwo's-complement unsigned値として
SplitMix64へ入れ、連続4 outputをxoshiro256\*\*の`(s0, s1, s2, s3)`にします。すべての演算はunsigned 64-bit
modulo arithmetic、`>>`はlogical shiftです。

```text
splitmix state += 0x9E3779B97F4A7C15
z = state
z = (z xor (z >> 30)) * 0xBF58476D1CE4E5B9
z = (z xor (z >> 27)) * 0x94D049BB133111EB
output = z xor (z >> 31)

xoshiro output = rotl(s1 * 5, 7) * 9
t = s1 << 17
s2 xor= s0; s3 xor= s1; s1 xor= s2; s0 xor= s3
s2 xor= t; s3 = rotl(s3, 45)
```

RandomSizeは1 byteから1 MiBです。nextIntはoutput bit patternをsigned two's-complement Intとして返します。
nextBoolはoutputのleast-significant bit、unitFloatは`output >> 11`の53 bitを`2^53`で割った`[0, 1)`の
binary64です。randomBytesはoutputをlittle-endianに並べ、
要求sizeで切ります。intBetweenはlower inclusive / upperExclusiveで、空rangeを拒否し、64-bit rejection
samplingによりmodulo biasを作りません。chanceはNaNまたは`[0, 1]`外を拒否し、0は常にFalse、1は常にTrueです。
chooseはNonEmptyListなので空入力errorを持たず、shuffleはsourceを変更しないFisher-Yatesです。
algorithmId、invalid range / probability、chance 0 / 1はstateを進めません。nextBool、nextInt、unitFloat、
0と1以外のchanceは一output、randomBytesは`ceil(size / 8)` outputを使います。intBetweenとchooseは
rejectionが完了するまで、shuffleは各swap indexのrejectionが完了するまで進めます。一operationの途中へ
別FiberのRandom callを割り込ませません。

secure entropyは次を提供します。

```seseragi
type EntropyConfigError deriving Eq, Show =
  | NonPositiveEntropySize Int
  | EntropySizeTooLarge Int

type EntropyError deriving Eq, Show =
  | EntropyUnavailable
  | EntropyReadFailure

opaque type EntropySize

fn entropySize bytes: Int -> Either<EntropyConfigError, EntropySize>
fn secureBytes size: EntropySize
  -> Effect<{ entropy: Entropy }, EntropyError, Bytes>
```

Entropyのcanonical requirement名は`entropy`です。EntropySizeは1 byteから1 MiBです。secureBytesはhost OSまたは
同等のcryptographically secure generatorから要求量すべてを返し、partial resultを公開しません。seed指定、
sequence replay、algorithmId、Float / range helperを提供しません。cancellationとhost completionが競合した場合も
返されなかったsecret bytesを後続callへ再利用しません。test runnerはEntropyを既定提供せず、必要なtestだけが
明示的なfake serviceをprovideします。

Map / Setのprocess-local hash seed、temporary file suffix、default Random seedがsecure entropyを必要とする場合も、
Random serviceから取得しません。これらはtarget adapterがapplication開始前またはresource作成時にEntropyと同等の
host sourceからdomain-separatedに取得します。

## 10.11 `std/effect`

Effect moduleは9.8のoperationに加え、次を提供します。

- environment: `service`, `provideSome`
- value/error変換: `attempt`, `fromEither`, `fromMaybe`
- temporal control: `timeout`, `retry`, `repeat`
- resource: `bracket`, `scoped`, `acquireRelease`

```seseragi
fn attempt<R, E, A>
  effect: Effect<R, E, A>
  -> Effect<R, Never, Either<E, A>>
fn fromEither<R, E, A>
  value: Either<E, A>
  -> Effect<R, E, A>
fn fromMaybe<R, E, A>
  error: E
  -> value: Maybe<A>
  -> Effect<R, E, A>
```

`attempt`はtyped failureを`Left`、successを`Right`としてsuccess channelへ移します。defectとcancellationを捕捉せず、
environment requirementと評価時点を変えません。`fromEither`は`Left`をtyped failure、`Right`をsuccessにし、
`fromMaybe error`は`Nothing`を指定errorのtyped failure、`Just`をsuccessにします。どちらも入力値をEffect構築時に
再評価せず、返したEffectを実行した時に保存済みcaseを一度公開します。

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

type LoopControl =
  | Continue
  | Break

fn forEachUntil<C, R, E, A>
  f: (A -> Effect<R, E, LoopControl>)
  -> values: C
  -> Effect<R, E, Unit>
where Iterable<C, A>
```

`forEach` は `iterate values` の順に一件ずつEffectを実行し、最初のfailureで停止します。
`forEachUntil`は同じ順序で、`Continue`なら次へ進み、`Break`なら後続actionを開始せずUnitで成功します。
空collectionはactionを呼ばず成功します。actionのfailure、defect、cancellationは通常どおり伝播し、Breakを
failureやcancellationとして観測しません。途中終了と入力順が本質なのでparallel版は提供しません。
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

### `std/stdin`

Stdinは現在processのstandard input cursorを表すhost serviceで、Console出力やchild processのstdinとは
別のcapabilityです。canonical requirement名は`stdin`です。

```seseragi
type StdinConfigError deriving Eq, Show =
  | NonPositiveReadSize Int
  | ReadSizeTooLarge Int
  | NonPositiveLineLimit Int
  | LineLimitTooLarge Int

opaque type ReadSize
opaque type LineLimit

fn readSize bytes: Int -> Either<StdinConfigError, ReadSize>
fn lineLimit bytes: Int -> Either<StdinConfigError, LineLimit>
fn defaultReadSize -> ReadSize
fn defaultLineLimit -> LineLimit

type StdinError deriving Eq, Show =
  | StdinUnavailable
  | StdinReadFailure
  | ConcurrentStdinRead
  | InvalidStdinUtf8 { offset: Int }
  | StdinLineTooLong { limitBytes: Int }
  | StdinPositionOverflow

fn readChunk size: ReadSize
  -> Effect<{ stdin: Stdin }, StdinError, Maybe<Bytes>>
fn readLine
  -> Effect<{ stdin: Stdin }, StdinError, Maybe<String>>
fn readLineWith limit: LineLimit
  -> Effect<{ stdin: Stdin }, StdinError, Maybe<String>>
fn lines limit: LineLimit
  -> Stream<{ stdin: Stdin }, StdinError, String>
```

ReadSizeは1 byteから1 MiB、LineLimitは1 byteから64 MiBです。defaultReadSizeは64 KiB、
defaultLineLimitは1 MiBです。validated valueにより、read operation開始後にsize configurationで失敗しません。

readChunkは現在cursorから1 byte以上size以下のBytesを返し、EOFならNothingです。空Bytesを返しません。
readLineWithはLFまでを一行とし、LFと直前のoptionalなCRを除きます。bare CRは内容に残します。`\n`は
`Just ""`、EOF前のterminatorなしnon-empty bytesも最後の一行、bufferが空のEOFはNothingです。
readLineは`readLineWith (defaultLineLimit ())`です。

lineはstrict UTF-8でdecodeします。invalid sequenceではStdin全体の先頭からの0-based byte offsetを
InvalidStdinUtf8に入れ、そのlineをterminatorまたはEOFまで消費します。limitはterminatorを除くbyte数へ適用し、
超えた場合は残りのlineを保持せずterminatorまたはEOFまでdiscardしてStdinLineTooLongを返します。したがって
errorを処理して次のreadを行うと次のlineから再開します。cursor offsetがInt範囲を超える前に
StdinPositionOverflowで失敗し、それ以後のreadも同じfailureです。

一つのStdin serviceで同時にactiveにできるreadは一件です。別のreadChunk、readLine、lines terminal executionが
activeなら、後から開始したoperationはbytesを消費せずConcurrentStdinReadで失敗します。read完了、failure、
cancellationでleaseを解放します。host readがcancellationと競合してbytesを返した場合、そのbytesをservice内部へ
戻し、次のreadから観測できるようにして欠落させません。cancellationやStream終了はprocess共有stdinをcloseしません。

EOFはstickyです。linesはterminal execution時点のcursorからreadLineWithを逐次実行するcold Streamですが、
外部stdinをreplayしません。同じStream値を再実行すると、その時点で残っているinputから続けます。各lineへdemandが
来るまで次のlineを読みません。test hostはbyte列、read chunk境界、EOF、read failureを決定的に提供できます。

`std/path` はpureなpath操作、`std/fs` はFileSystem serviceを要求するEffectを提供します。
Pathはhost Stringのaliasではなく、`/` をseparatorに使うportableなlexical pathを表すstandard opaque
typeです。POSIX root `/`、Windows drive root `C:/`、UNC root `//server/share/`、relative pathを区別します。
backslashとNULを受理せず、case foldingやUnicode normalizationを行いません。

```seseragi
type PathError deriving Eq, Show =
  | EmptyPath
  | PathContainsNul { offset: Int }
  | PathContainsBackslash { offset: Int }
  | InvalidDriveRoot
  | InvalidUncRoot
  | InvalidPathSegment String
  | AbsoluteChildPath

fn parse text: String -> Either<PathError, Path>
fn render value: Path -> String
fn current -> Path
fn isAbsolute value: Path -> Bool
fn normalize value: Path -> Path
fn join child: Path -> base: Path -> Either<PathError, Path>
fn child name: String -> base: Path -> Either<PathError, Path>
fn parent value: Path -> Maybe<Path>
fn fileName value: Path -> Maybe<String>
fn extension value: Path -> Maybe<String>
```

parseは`.` と `..` を含むsegmentを保持します。normalizeだけが空segmentと `.` を除き、通常segmentの直後に
ある `..` を相殺します。absolute pathでrootより上へ出る `..` はrootに留め、relative path先頭の `..` は
保持します。filesystemへ渡す前のnormalizeはsecurity boundaryではなく、symlinkを解決しません。

`join child base` はrelative childだけをbase末尾へ加え、absolute childをAbsoluteChildPathとして拒否します。
`child name base` はseparator、`.`、`..` を含まない単一segmentだけを加えます。currentはrelativeな `.` です。
parentはrootとcurrentでNothing、fileNameはrootでNothingです。extensionは最後のnameについて、先頭以外の
最後の`.`より後を返し、`.gitignore` と末尾`.`はNothingです。

FileSystem serviceのcanonical requirement名は `fileSystem` で、`with FileSystem` は
`with fileSystem: FileSystem` へ展開します。

```seseragi
type FileType deriving Eq, Ord, Show =
  | RegularFile
  | Directory
  | SymbolicLink
  | OtherFileType

type FileSystemOperation deriving Eq, Ord, Show =
  | ReadFile
  | WriteFile
  | OpenDirectory
  | ReadMetadata
  | CreateDirectory
  | RemovePath
  | MovePath
  | CanonicalizePath
  | CreateTemporary

type FileSystemErrorKind deriving Eq, Show =
  | FileNotFound
  | FileAlreadyExists
  | PermissionDenied
  | NotADirectory
  | IsADirectory
  | DirectoryNotEmpty
  | SymbolicLinkLoop
  | CrossDeviceMove
  | PathNotSupported
  | FileSystemUnavailable
  | OtherFileSystemError String

struct FileSystemError deriving Eq, Show {
  operation: FileSystemOperation,
  path: Path,
  otherPath: Maybe<Path>,
  kind: FileSystemErrorKind
}

struct FileMetadata deriving Eq, Show {
  fileType: FileType,
  sizeBytes: Int,
  modified: Maybe<Instant>,
  created: Maybe<Instant>
}

struct DirectoryEntry deriving Eq, Show {
  name: String,
  path: Path,
  fileType: Maybe<FileType>
}

type WriteMode deriving Eq, Show =
  | Replace
  | CreateNew
  | Append

type FileTextError deriving Eq, Show =
  | FileAccessFailure FileSystemError
  | FileUtf8Failure Utf8DecodeError

fn exists path: Path
  -> Effect<{ fileSystem: FileSystem }, FileSystemError, Bool>
fn metadata path: Path
  -> Effect<{ fileSystem: FileSystem }, FileSystemError, FileMetadata>
fn symlinkMetadata path: Path
  -> Effect<{ fileSystem: FileSystem }, FileSystemError, FileMetadata>
fn canonicalize path: Path
  -> Effect<{ fileSystem: FileSystem }, FileSystemError, Path>

fn readBytes path: Path
  -> Effect<{ fileSystem: FileSystem }, FileSystemError, Bytes>
fn readTextUtf8 path: Path
  -> Effect<{ fileSystem: FileSystem }, FileTextError, String>
fn readChunks size: BufferCapacity -> path: Path
  -> Stream<{ fileSystem: FileSystem }, FileSystemError, Bytes>

fn writeBytes mode: WriteMode -> content: Bytes -> path: Path
  -> Effect<{ fileSystem: FileSystem }, FileSystemError, Unit>
fn writeTextUtf8 mode: WriteMode -> content: String -> path: Path
  -> Effect<{ fileSystem: FileSystem }, FileSystemError, Unit>
fn writeChunks<R, E>
  mode: WriteMode
  -> source: Stream<R, E, Bytes>
  -> path: Path
  -> Effect<R & { fileSystem: FileSystem }, Either<E, FileSystemError>, Unit>
fn writeAtomic content: Bytes -> path: Path
  -> Effect<{ fileSystem: FileSystem }, FileSystemError, Unit>

fn list path: Path
  -> Stream<{ fileSystem: FileSystem }, FileSystemError, DirectoryEntry>
fn createDirectory path: Path
  -> Effect<{ fileSystem: FileSystem }, FileSystemError, Unit>
fn createDirectories path: Path
  -> Effect<{ fileSystem: FileSystem }, FileSystemError, Unit>
fn removeFile path: Path
  -> Effect<{ fileSystem: FileSystem }, FileSystemError, Unit>
fn removeDirectory path: Path
  -> Effect<{ fileSystem: FileSystem }, FileSystemError, Unit>
fn move destination: Path -> source: Path
  -> Effect<{ fileSystem: FileSystem }, FileSystemError, Unit>

fn withTemporaryDirectory<R, E, A>
  prefix: String
  -> use: (Path -> Effect<R, E, A>)
  -> Effect<R & { fileSystem: FileSystem }, Either<FileSystemError, E>, A>
fn withTemporaryFile<R, E, A>
  prefix: String
  -> use: (Path -> Effect<R, E, A>)
  -> Effect<R & { fileSystem: FileSystem }, Either<FileSystemError, E>, A>
```

existsはNotFoundとdangling symlinkでFalse、存在確認できればTrue、それ以外はtyped failureを返します。
permissionやI/O障害を「存在しない」に変換しません。metadataはsymlinkを辿り、symlinkMetadataはlink自身を
返します。canonicalizeは存在するpathについて`.`、`..`、symlinkを解決したabsolute Pathを返しますが、
sandbox脱出を許可する認可APIではありません。

readBytes / readTextUtf8は内容全体をmemoryへ保持するsmall-file APIです。readTextUtf8はstrict UTF-8 decodeを
行います。small-file APIはhandleを閉じ終えてから成功し、正常処理後のclose failureもtyped failureです。
readChunksはterminal operation開始時にfileを開き、BufferCapacity以下のnon-empty Bytesを順に出し、EOFで
終了します。consumer cancellation、failure、early terminationのすべてでhandleを閉じます。

Replaceは既存fileをtruncateするか新規作成、CreateNewは既存pathならFileAlreadyExists、Appendは各writeを
file末尾へ加えます。writeChunksはsource failureをLeft、filesystem failureをRightにします。sourceとwriteが
同時に失敗した場合はsource failureを返し、write failureをdiagnosticへ添付します。
Replace、Append、writeChunksはatomicityを保証せず、failureやcancellationまでに書けたprefixを残せます。

writeAtomicはtargetと同じdirectoryへtemporary fileを作り、全Bytesを書いてflushし、targetへatomic replace
します。成功後は新内容全体、失敗時は旧内容全体またはtarget不在のどちらかで、partial targetを残しません。
target directory自体のdurability flushをhostが提供できない場合もatomic visibilityは必須ですが、power-loss
durabilityは保証せずcapability diagnosticを出せます。temporary artifactは失敗時にbest effortで削除します。

listの順序はfilesystem依存で、deterministic処理はnameを明示sortします。entry.fileTypeはdirectory scanで
追加I/Oなしに判定できない場合Nothingです。Stream終了時にはdirectory handleを閉じ、正常終了時のclose失敗は
FileSystemError、既存failureまたはcancellation後のclose失敗はdiagnosticへ添付します。

createDirectoryはparentが存在する場合だけ一階層を作り、createDirectoriesは欠けたancestorも作ります。
createDirectoriesは対象が既存directoryなら成功します。removeFileはregular fileまたはsymlinkだけ、
removeDirectoryは空directoryだけを削除し、どちらもsymlinkを辿りません。moveはdestinationが存在すれば
FileAlreadyExists、別filesystemならCrossDeviceMoveで、copy-and-deleteへ暗黙fallbackしません。同じfilesystemで
成功するmoveは他processからatomicに観測できなければなりません。

withTemporaryDirectory / withTemporaryFileはsecure randomなsuffixをprefixへ加え、exclusiveに作成してuseへ
渡します。作成先はFileSystem serviceが構成したtemporary rootです。prefixはpath separator、NUL、`.`、`..` を
含まない単一segmentでなければならず、不正ならCreateTemporary / PathNotSupportedです。withTemporaryFileは
空のregular fileを作ります。useの終了状態にかかわらずcleanupし、acquire / cleanup failureはLeft、use failureは
Rightです。
use failureとcleanup failureが両方起きた場合はRightのuse failureを返してcleanup failureをdiagnosticへ添付します。
temporary directoryはこのAPIが作成したtreeだけを再帰削除し、symlinkを辿りません。useが対象をすでに削除して
いた場合のcleanupは成功です。activeなtemporary root自体をstandard FileSystem経由でmoveしようとすると
PermissionDeniedです。cleanupは作成時のresource identityを追跡し、同じpathへ外部から置かれた別resourceを
誤って削除してはなりません。

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
  -> Effect<{ process: Process }, ProcessError, Path>
fn signals watched: NonEmptyList<ProcessSignal>
  -> Stream<{ process: Process }, ProcessError, ProcessSignal>
```

`arguments ()` はexecutable名を含めないapplication引数をsource順に返します。`environment name` は
存在しないkeyを `Nothing`、存在する値を `Just value` にします。host byte列をUnicode Stringへ
変換できない場合は位置またはkeyを持つProcessErrorです。environment全体をMapとしてsnapshotするAPIは
coreへ置かず、必要なkeyだけを明示的に読みます。

`currentDirectory ()` は呼び出し時点のworking directoryをabsoluteなportable Pathへ変換します。Windowsの
host separatorは `/` へ変換し、drive / UNC rootは `std/path` の表現に従います。hostから取得できない、
Unicode Stringへ変換できない、またはportable Pathとして表せない場合はCurrentDirectoryUnavailableです。
lexical normalizeやsymlink解決は行いません。相対PathをFileSystemへ渡す場合のbaseは、このoperationで
観測するworking directoryと同じです。

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

### child process

child processは `ChildProcesses` serviceを要求します。canonical requirement名は `childProcesses` で、
`with ChildProcesses` は `with childProcesses: ChildProcesses` へ展開します。現在processのProcess serviceとは
capabilityを分離し、libraryがargumentsを読む権限だけでchildを起動できるようにはしません。

```seseragi
type Executable deriving Eq, Show =
  | SearchPath String
  | ExecutablePath Path

type ChildProcessConfigError deriving Eq, Show =
  | EmptyExecutableName
  | ExecutableNameContainsSeparator String
  | ArgumentContainsNul { index: Int, offset: Int }
  | EnvironmentNameContainsNul String
  | EnvironmentValueContainsNul String
  | InvalidCaptureLimit Int

type ChildOutputChannel deriving Eq, Ord, Show =
  | ChildStdout
  | ChildStderr

type ChildProcessError deriving Eq, Show =
  | ChildSpawnFailed { executable: Executable, detail: String }
  | ChildInputAfterClose
  | ChildOutputReadFailed { channel: ChildOutputChannel, detail: String }
  | UnsupportedChildSignal ProcessSignal
  | ChildInputFailed String
  | ChildOutputLimitExceeded {
      channel: ChildOutputChannel,
      limitBytes: Int
    }
  | ChildWaitFailed String
  | ChildTerminationFailed String

type ChildExitStatus deriving Eq, Show =
  | ChildExited Int
  | ChildSignaled ProcessSignal
  | ChildHostTerminated String

type ChildInput deriving Eq, Show =
  | WriteChildStdin Bytes
  | CloseChildStdin
  | SignalChild ProcessSignal
  | KillChild

type ChildEvent deriving Eq, Show =
  | ChildStdoutChunk Bytes
  | ChildStderrChunk Bytes
  | ChildExitedWith ChildExitStatus

struct CapturedProcess deriving Eq, Show {
  status: ChildExitStatus,
  stdout: Bytes,
  stderr: Bytes
}

fn command executable: Executable
  -> Either<ChildProcessConfigError, Command>
fn addArgument value: String -> command: Command
  -> Either<ChildProcessConfigError, Command>
fn addArguments values: Array<String> -> command: Command
  -> Either<ChildProcessConfigError, Command>
fn inDirectory path: Path -> command: Command -> Command
fn setEnvironment name: String -> value: String -> command: Command
  -> Either<ChildProcessConfigError, Command>
fn unsetEnvironment name: String -> command: Command
  -> Either<ChildProcessConfigError, Command>
fn clearEnvironment command: Command -> Command
fn terminationGrace duration: Duration -> command: Command -> Command
fn outputBuffer capacity: BufferCapacity -> command: Command -> Command

fn captureLimit bytes: Int
  -> Either<ChildProcessConfigError, CaptureLimit>
fn defaultCaptureLimit -> CaptureLimit

fn runStreaming<R, E>
  input: Stream<R, E, ChildInput>
  -> command: Command
  -> Stream<
      R & { childProcesses: ChildProcesses },
      Either<E, ChildProcessError>,
      ChildEvent
    >
fn runCaptured
  limit: CaptureLimit
  -> input: Bytes
  -> command: Command
  -> Effect<
      { childProcesses: ChildProcesses },
      ChildProcessError,
      CapturedProcess
    >
fn runInherited command: Command
  -> Effect<{ childProcesses: ChildProcesses }, ChildProcessError, ChildExitStatus>
```

SearchPath nameは空でなく、`/` と `\` を含まない単一nameです。最終的なCommand environmentのPATHを
ChildProcesses serviceのtarget固有search ruleで解釈します。clearEnvironment後にPATHを設定していなければ
SearchPathはChildSpawnFailedです。ExecutablePathはabsoluteならそのpath、relativeならCommandのcurrent
directory、未指定ならChildProcesses serviceのconfigured working directoryを基準にします。

commandはrun開始時にChildProcesses serviceのconfigured environmentを既定でsnapshotします。
setEnvironmentはkeyを設定、unsetEnvironmentは一keyを除外、clearEnvironmentは空environmentを基準に
切り替えます。同じkeyへの後の操作が前を上書きします。addArgumentとenvironment builderはNULを拒否し、
host encodingへ変換できない場合はChildSpawnFailedです。Commandはimmutableで、builderは新しいCommandを
返します。default termination graceは5秒、output bufferは各channel 16 chunksです。

`runStreaming input command` はcold Streamです。terminal operationごとにchildを一つ新規spawnし、inputも
その実行ごとに一度実行します。stdin、stdout、stderrはpipeへ接続します。WriteChildStdinはBytes全体を順に
書き終えるまで次のinputを要求せず、CloseChildStdinとinput正常終了はstdinをflushして閉じます。closeは
idempotentですが、close後のWriteChildStdinはChildInputAfterCloseです。

SignalChildはchildへportable signalを要求し、unsupportedならStreamを失敗させてcleanupします。KillChildは
hostのuncatchable forced terminationを要求しますが、exit観測までは待ちます。nonzero exitとsignal exitは
通常のChildExitedWith eventで、Stream failureではありません。

stdout / stderrはoutputBufferで指定したchunk数の内部lossless bufferを別々に持ち、満杯なら対応pipeのreadを
止めてchildへOS backpressureをかけます。一chunkは最大64 KiBです。各channel内のBytes順序を保ち、channel間は
hostがread completionを観測した順です。chunk境界は意味を持たず、空Bytesを出しません。input writeと二つの
output readはconcurrentに進め、どれか一方向のpipeだけでchildをdeadlockさせません。両pipeのEOFとchild statusを
すべて観測してからChildExitedWithを最後の一件として出し、その後Streamを正常終了します。

input StreamがEで失敗した場合はstdinを閉じ、childをtermination手順で終了させてからLeft errorで失敗します。
childへのwrite、spawn、wait、signalが失敗した場合はRight ChildProcessErrorです。consumer cancellationまたは
surrounding scope終了ではstdinを閉じ、Terminateを試し、Commandのtermination graceだけhost monotonic timeで
待ち、未終了ならKillChildを送ってreapします。このcleanupはapplicationのClock serviceを要求しません。
Terminateを扱えないtargetまたはchildではgraceを待たずKillChildへ進みます。childがinputより先に終了した場合は
input Streamをcancelし、pipeをdrainしてから最後のexit eventを出します。同時failureは最初に観測したfailureを
返し、cleanup中に判明した後続failureをdiagnosticへ添付します。

runCapturedはinput writeとstdout / stderr readをconcurrentに進め、stdinを閉じてからchild終了まで待ち、
stdout / stderrをそれぞれCaptureLimitまで保持します。既定limitは
各channel 8 MiBです。どちらかがlimitを超えた時点でchildをtermination手順に従って終了し、
ChildOutputLimitExceededで失敗します。runInheritedはparent hostのstdin / stdout / stderrをそのまま継承し、
終了statusだけを返します。どちらもcancellation時はrunStreamingと同じtermination / reapを保証します。

ChildExitedのcodeはhostが報告したnon-negative integerを保持し、0だけをsuccessとみなします。signalによる終了を
正確に観測できるtargetはChildSignaled、signal modelを持たないtargetやhost固有terminationは
ChildHostTerminatedです。標準APIはhost codeをPOSIXの128+signal番号へ捏造しません。

childのreap完了前にrun APIを終了してzombieを残してはなりません。通常termination failureはtyped failureです。
すでに別failureまたはcancellationを処理中のtermination / reap failureはprimary resultを置き換えずdiagnosticへ
添付します。forced host terminationだけはこの保証外です。

## 10.15 `std/http`

HTTP clientは `Request`、`Response`、`Method`、`Status`、`Headers`、`Body` とHttpClient serviceを
提供します。canonical requirement名は `http` で、`with HttpClient` は `with http: HttpClient` へ
展開します。

```seseragi
type HttpBuildError deriving Eq, Show =
  | InvalidHttpUrl { offset: Int }
  | UnsupportedHttpScheme String
  | HttpUrlContainsUserInfo
  | HttpUrlContainsFragment
  | InvalidHttpMethod String
  | InvalidHeaderName String
  | InvalidHeaderValue { name: String, offset: Int }
  | ManagedHttpHeader String
  | InvalidHttpStatus Int
  | InvalidHttpBodyLimit Int

type HttpError deriving Eq, Show =
  | HttpDnsFailure String
  | HttpConnectionFailure String
  | HttpTlsFailure String
  | HttpProtocolFailure String
  | HttpRequestBodyFailure String
  | HttpRequestLengthMismatch { declared: Int, actual: Int }
  | HttpResponseBodyLimitExceeded { limitBytes: Int }
  | HttpClientUnavailable

type HttpVersion deriving Eq, Ord, Show =
  | Http1_0
  | Http1_1
  | Http2
  | Http3

struct ResponseHead deriving Eq, Show {
  version: HttpVersion,
  status: Status,
  headers: Headers
}

struct Response deriving Eq, Show {
  head: ResponseHead,
  body: Bytes,
  trailers: Headers
}

type HttpEvent deriving Eq, Show =
  | InformationalResponse ResponseHead
  | ResponseStarted ResponseHead
  | ResponseBodyChunk Bytes
  | ResponseTrailers Headers

get : Method
head : Method
post : Method
put : Method
patch : Method
delete : Method
options : Method
connect : Method
trace : Method

fn customMethod text: String -> Either<HttpBuildError, Method>
fn methodText value: Method -> String
fn status code: Int -> Either<HttpBuildError, Status>
fn statusCode value: Status -> Int
fn isInformational value: Status -> Bool
fn isSuccess value: Status -> Bool
fn isRedirection value: Status -> Bool
fn isClientError value: Status -> Bool
fn isServerError value: Status -> Bool

fn parseUrl text: String -> Either<HttpBuildError, HttpUrl>
fn renderUrl value: HttpUrl -> String

emptyHeaders : Headers
fn appendHeader name: String -> value: String -> headers: Headers
  -> Either<HttpBuildError, Headers>
fn setHeader name: String -> value: String -> headers: Headers
  -> Either<HttpBuildError, Headers>
fn removeHeader name: String -> headers: Headers -> Headers
fn headerValues name: String -> headers: Headers -> Array<String>
fn headerEntries headers: Headers -> Array<(String, String)>

fn request method: Method -> url: HttpUrl -> Request
fn withRequestHeader name: String -> value: String -> request: Request
  -> Either<HttpBuildError, Request>
fn withoutRequestHeader name: String -> request: Request -> Request

fn emptyBody<R, E> -> Body<R, E>
fn bytesBody<R, E> content: Bytes -> Body<R, E>
fn streamBody<R, E> content: Stream<R, E, Bytes> -> Body<R, E>

fn bodyLimit bytes: Int -> Either<HttpBuildError, HttpBodyLimit>
fn defaultBodyLimit -> HttpBodyLimit

fn exchange<R, E>
  body: Body<R, E>
  -> request: Request
  -> Stream<
      R & { http: HttpClient },
      Either<E, HttpError>,
      HttpEvent
    >
fn sendBytes
  limit: HttpBodyLimit
  -> body: Bytes
  -> request: Request
  -> Effect<{ http: HttpClient }, HttpError, Response>
fn sendEmpty
  limit: HttpBodyLimit
  -> request: Request
  -> Effect<{ http: HttpClient }, HttpError, Response>
```

Methodは大文字ASCII tokenです。standard value以外はcustomMethodで検証します。Statusは100以上999以下の
integerを保持します。classification helperは先頭digitだけで判定し、未知statusも値として扱います。

HttpUrlはabsoluteなASCII URLだけを表し、schemeは `http` または `https`、hostはASCII domain、IPv4、
bracket付きIPv6です。Unicode hostは暗黙IDNA変換せず、呼び出し側が明示的にASCII formへ変換します。parseUrlは
schemeとdomainをlowercase、default portを除去、pathのdot segmentを解決し、percent escapeを大文字へ正規化
します。reserved characterをpercent decodeせず、query parameterの順序と重複を保ちます。userinfoとfragmentは
request targetに含めずbuild errorです。

Headersはimmutableなordered multi-mapです。nameはRFC tokenのASCII、valueはCR、LF、NULを含まないbyte列へ
変換可能なStringです。比較はASCII case-insensitiveで、headerEntriesはnameをlowercaseにして追加順を保ちます。
appendHeaderは同名valueを末尾へ追加、setHeaderは同名entryをすべて除いて最初の位置へ一件置き、存在しなければ
末尾へ置きます。headerValuesは追加順です。Set-Cookieなどを壊すため、複数valueをcommaで暗黙結合しません。

Requestはmethod、URL、headersを持つimmutable valueで、bodyは実行時に別引数として与えます。Bodyはcoldで、
emptyBodyとbytesBodyは再利用可能、streamBodyは元Streamの実行ごとに新しく内容を生成します。Content-Lengthを
指定し、実際のbody byte数が違えば送信を中断してHttpRequestLengthMismatchです。指定しない場合はprotocolが
framingを選びます。GETやHEADへbodyを付けること自体はcoreで禁止せず、server semanticsに委ねます。

Body<R, E>、Request、Headers、Method、Status、HttpUrl、HttpBodyLimitはstandard opaque typeです。Requestの
初期headersはemptyHeadersです。`connection`、`proxy-connection`、`keep-alive`、`transfer-encoding`、`upgrade`、
`te`、HTTP/2・HTTP/3 pseudo-headerはclientが管理し、withRequestHeaderで設定するとManagedHttpHeaderです。
HostはURLから生成します。Content-Lengthは設定できますが、重複・不正値は送信開始前にHttpProtocolFailureです。

exchangeはterminal operationごとに新しいrequestを一件開始するcold Streamです。0件以上の
InformationalResponse、finalなResponseStarted一件、0件以上のnon-empty ResponseBodyChunk、省略可能な
ResponseTrailers一件の順に出して正常終了します。statusがnon-2xxでもResponseStartedとして成功し、transport
failureへ変換しません。一chunkは最大64 KiBで、consumer demandがなければsocket readを止めるlossless
backpressureです。

request bodyのpull / writeとresponse readはconcurrentに進めます。serverがbody完了前にfinal responseを返した
場合はrequest body Streamをcancelし、responseを最後まで処理します。body StreamのEはLeft、DNS、TLS、protocol、
socket failureはRight HttpErrorです。同時failureは最初に観測したものを返し、cleanup failureはdiagnosticへ
添付します。

consumer cancellation、timeout、early terminationではrequest bodyをcancelし、HTTP/2・HTTP/3 streamをreset、
HTTP/1 connectionは安全に再利用できなければcloseします。response bodyをEOFまで読み、protocol上再利用可能な
connectionだけをHttpClient poolへ戻します。exchange終了前にsocket / stream ownershipを必ず解放し、pool自体は
HttpClient serviceのscopeに所属します。

sendBytesとsendEmptyはexchangeを実行してbodyをmemoryへ集めるsmall-response APIです。default limitは8 MiBで、
超過時はresponse streamをcancelしてHttpResponseBodyLimitExceededです。informational responseは破棄し、final
head、body、trailersだけをResponseへ入れます。

redirect、cookie jar、authentication、retry、timeout、content encoding decodeは自動適用しません。redirectは
3xx ResponseとLocationをapplicationが明示処理し、retry / timeoutはEffectまたはSchedule combinatorで包みます。
body BytesはHTTP transfer framingを除去した後、Content-Encodingをdecodeする前の値です。JSON decodeは
`std/json` Decoder、text decodeはcharsetを確認するadapterを明示的に使います。

server APIはclientと別moduleにし、runtime adapterが提供できる場合だけ利用します。

## 10.16 `std/test`

test moduleはassertion、test tree、property test、law test、Effect test runtimeを提供します。testは
新しいdeclaration構文ではなく、通常のpureな値です。

```seseragi
alias TestEnvironment = {
  clock: Clock,
  random: Random,
  console: Console,
  logger: Logger
}

type TestFailure deriving Eq, Show =
  | AssertionFailed {
      message: String,
      expected: Maybe<String>,
      actual: Maybe<String>
    }
  | ExpectedTypedFailure
  | TypedFailureDidNotMatch String
  | ExplicitTestFailure String

opaque type Test

fn test
  name: String
  -> body: Effect<TestEnvironment, TestFailure, Unit>
  -> Test
fn suite name: String -> children: Array<Test> -> Test
fn skip reason: String -> child: Test -> Test
fn timeout duration: Duration -> child: Test -> Test

fn equal<A> expected: A -> actual: A -> Task<TestFailure, Unit>
where Eq<A>, Debug<A>
fn notEqual<A> unexpected: A -> actual: A -> Task<TestFailure, Unit>
where Eq<A>, Debug<A>
fn isTrue actual: Bool -> Task<TestFailure, Unit>
fn isFalse actual: Bool -> Task<TestFailure, Unit>
fn fail message: String -> Task<TestFailure, Unit>
fn expectFailure<R, E, A>
  predicate: (E -> Bool)
  -> effect: Effect<R, E, A>
  -> Effect<R, TestFailure, Unit>
where Debug<E>
```

`equal` / `notEqual`はEqで判定し、失敗時だけDebugを用いてexpected / actualを作ります。user-facingな
snapshot textをDebugへ依存させません。`expectFailure`はpredicateを満たすtyped failureだけを成功とし、
successはExpectedTypedFailure、predicateを満たさないfailureはTypedFailureDidNotMatchです。defectと
cancellationをtyped failureとして捕捉しません。application固有errorは呼び出し側が`mapError`で
ExplicitTestFailureなどへ変換します。

`Test`はcaseまたはsuiteを表すimmutableなopaque treeです。test bodyはTest値を構築した時点では実行せず、
runnerが選択したcaseだけを一度実行します。suiteはchildrenのsource順を保ちます。skipはbodyを実行せず、
timeoutはrunnerのhost monotonic timeで測ります。timeoutによる終了はbodyのcancellationとfinalizer完了を待ち、
TestFailureのerror channelへ偽装しません。未終了child Fiber、登録解除されていないtest-owned resource、
uncaught defectはcase failureです。

test rootから発見される値の型は正確に`Test`です。bodyが要求する標準serviceが少ない場合は
Effect requirement wideningでこのenvironmentへ揃えます。FileSystem、HTTP、databaseなど追加serviceが必要なら、
testを構築する前にtest doubleを`provide`し、exported Testへ未解決requirementを残しません。

runnerはcaseごとに独立したTestEnvironmentを作ります。Clockはmonotonic zeroとUnix epochから始まるvirtual
clock、Randomはrunner seedから同じ初期状態、ConsoleとLoggerはcase専用captureです。case間でservice instance、
Fiber、resource scope、captured outputを共有しません。property testは失敗時にseedと最小化後のinputをreportし、
同じtool version・seed・propertyから同じ生成列を再現しなければなりません。

Functor、Applicative、Monad、Semigroup、Monoidのlaw helperは通常のTest treeを返し、runnerだけの別実行経路を
持ちません。test moduleも通常のSeseragi moduleであり、test export以外の特別な型検査規則を持ちません。

## 10.17 `std/benchmark`

benchmarkは新しいdeclarationではなく、runnerが発見する通常のpureな値です。

```seseragi
alias BenchmarkEnvironment = {
  random: Random,
  console: Console,
  logger: Logger
}

type BenchmarkFailure deriving Eq, Show =
  | ExplicitBenchmarkFailure String

opaque type Benchmark

fn benchmark
  name: String
  -> body: Effect<BenchmarkEnvironment, BenchmarkFailure, Unit>
  -> Benchmark
fn suite name: String -> children: Array<Benchmark> -> Benchmark
fn inputSize size: Int -> child: Benchmark -> Benchmark
fn blackBox<A> value: A -> A
fn fail message: String -> Effect<{}, BenchmarkFailure, Unit>
```

Benchmark treeのname規則、重複検査、source順はTestと同じです。inputSizeは正のproblem sizeをreport metadataへ
付け、nested指定では最も内側を使います。runnerはwarmupとsampleごとにbodyを一度以上逐次実行します。
一回のbody全体が測定対象で、resource setupを除外したい場合はBenchmark値の外でmutable stateを共有せず、
body内で測りたいoperationだけを反復可能なpure inputから構築します。blackBoxは引数を一度評価して同じ値を返し、
compilerが値をcompile-time constantとして消去することだけを防ぐoptimization barrierです。memory allocation、
identity、Effect順序を追加で変更しません。

exported Benchmarkへ未解決requirementを残せません。追加serviceはBenchmark構築前にtest doubleまたはlocal serviceを
provideします。Randomはcase名とrunner seedから導く独立stream、ConsoleとLoggerはcase専用captureです。
benchmark bodyのtyped failure、defect、resource leakは測定値にせずcase failureです。runnerのmonotonic measurement
clockはBenchmarkEnvironmentへ公開せず、applicationのClock serviceで測定を偽装できません。

## 10.18 optional adapter

pureなHtml treeとSSRは `std/web/html`、browser DOM capabilityは `std/web/dom` として13章の標準contractを
持ちます。Dom serviceの実装はtarget adapterですが、Htmlの意味、escape、event順、resource lifetimeをhostへ
委譲しません。

Node、database driver、追加web frameworkなどhost固有APIは標準言語semanticsではありません。`.d.ts` converterと
foreign bindingを使うadapter packageとして提供します。

adapterが標準service interfaceを実装できる場合、application codeはhostを変えても同じEffect
signatureを維持できます。

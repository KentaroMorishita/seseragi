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
users
  |> Array.filter isActive
  |> map toSummary
  |> Array.sortBy (\user -> user.name)
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

MapとSetは `Eq<A>` と `Hash<A>` を要求します。HashはEqと整合し、同値な値は同じhashを返さなければ
なりません。

MapとSetは挿入順で反復します。既存keyのvalue更新では位置を変えず、削除後に再挿入すると
末尾へ移動します。EqとHashはlookupに使い、反復順を変えません。

各sequenceは最低限、次を提供します。

```text
empty, singleton, fromIterable, toArray, toList
filter, filterMap, flatMap
reduce, sum, product, combine
find, findIndex, any, all
take, drop, takeWhile, dropWhile
zip, zipWith, unzip
append, concat, reverse
sort, sortBy, groupBy
length, isEmpty, nonEmpty
get, head, last, init, tail
chunksOf, windows
```

`get`, `head`, `last`, `init`, `tail` はMaybeを返します。`sort` はOrd、`sortBy` は比較keyのOrdを
要求します。範囲外、空collectionをdefectにしません。

`std/map` は少なくとも次を提供します。

`empty` の型schemeは次です。`forall` はsource syntaxではなく仕様上の表記です。

```text
empty : forall K V. Map<K, V>
```

ほかに少なくとも次の関数を提供します。

```seseragi
fn get<K, V> key: K -> values: Map<K, V> -> Maybe<V>
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

fn entries<K, V> values: Map<K, V> -> Array<(K, V)>
```

`empty` は期待型または後続operationからKとVを推論するpolymorphicなexported `let` です。
`upsert` は既存keyなら `Just current`、未登録なら `Nothing` をupdateへ一度渡し、返された値を
保存します。
既存keyの更新は挿入位置を保ち、新規keyは末尾へ追加します。`entries` は挿入順です。

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

- `service`, `provide`, `provideSome`
- `attempt`, `fromEither`, `fromMaybe`
- `timeout`, `retry`, `repeat`
- `race`, `parallel`, `forEach`, `forEachParallel`
- `bracket`, `scoped`, `acquireRelease`
- `fork`, `join`, `interrupt`

並行処理用に `Fiber<E, A>`、`Queue<A>`、`Ref<A>`、`Semaphore`、`Deferred<E, A>` を提供します。
これらの生成と操作はEffectです。

Refはatomic stateで、subscriberを持ちません。Signalはreactive graph、Refは同期primitiveです。

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

## 10.12 `std/stream`

`Stream<R, E, A>` はenvironment `R` を要求し、`E` で失敗しうる0個以上の非同期値です。

```text
fromArray, fromEffect, unfold
map, filter, filterMap, flatMap
take, drop, chunk
merge, zip, concat
debounce, throttle
runCollect, runFold, runForEach
```

Streamはbackpressureとcancellationを伝播し、resource scope終了時にproducerを停止します。
Signalとの変換は初期値・購読lifetime・loss policyを引数で明示します。

## 10.13 `std/signal`

5.12から5.15のSignal APIを提供します。UI framework固有bindingは標準coreへ含めず、adapter
packageとして提供します。

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

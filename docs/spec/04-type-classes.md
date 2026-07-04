# 4. 型クラス

## 4.1 目的

型クラスは「型がどの操作を実装できるか」を表します。継承関係や runtime object の
interface ではありません。型検査時に instance が一意に選ばれ、関数へ辞書として
渡される意味を持ちます。

## 4.2 宣言

```seseragi
pub trait Eq<A> {
  fn eq x: A -> y: A -> Bool
}
```

trait methodは型だけを宣言し、本体を持ちません。trait parameterは型または、
`F<_>` のような型構築子parameterにできます。traitは `where` でsupertraitを要求できます。

```seseragi
pub trait Functor<F<_>> {
  fn map<A, B> f: (A -> B) -> value: F<A> -> F<B>
}

pub trait Applicative<F<_>>
where Functor<F> {
  fn pure<A> value: A -> F<A>
  fn apply<A, B> wrapped: F<A -> B> -> value: F<A> -> F<B>
}

pub trait Monad<M<_>>
where Applicative<M> {
  fn flatMap<A, B> f: (A -> M<B>) -> value: M<A> -> M<B>
}
```

`F<_>` は値の型ではなく型構築子parameterです。`Either<E, _>` と
`Effect<R, E, _>` のような部分適用は、対応するkindの型構築子が要求される文脈で使えます。
kind、arity、部分適用の基本規則は[型システム 2.10](./02-types.md#210-kindと型構築子)に
従います。

## 4.3 instance

```seseragi
impl Eq<UserId> {
  fn eq x: UserId -> y: UserId -> Bool = x.value == y.value
}

impl Functor<Maybe> {
  fn map<A, B> f: (A -> B) -> value: Maybe<A> -> Maybe<B> =
    match value {
      Nothing -> Nothing
      Just x -> Just (f x)
    }
}

impl<E> Functor<Either<E, _>> {
  fn map<A, B> f: (A -> B) -> value: Either<E, A> -> Either<E, B> =
    match value {
      Left error -> Left error
      Right x -> Right (f x)
    }
}
```

instance の method 型は trait 宣言を型引数で置換した型と一致しなければなりません。
instance 自身も型 parameter と constraint を持てますが、すべての型 parameter は
instance head に出現しなければなりません。instance は値ではなく、明示的に生成・
受け渡しできません。

## 4.4 constraint

constraint は型 parameter の後ろの `where` に書きます。

```seseragi
fn member<A> target: A -> values: List<A> -> Bool
where Eq<A> =
  any (\value -> value == target) values
```

複数 constraint は `,` で区切ります。constraint を満たす instance が call site で
一意に存在しなければコンパイルエラーです。

## 4.5 coherence

プログラム全体で同じ trait 適用に複数の instance は存在できません。instance は、
trait または instance head の最外型を定義した module にだけ宣言できます。これを
orphan rule と呼びます。

instance の優先順位、overlap、局所 instance、孤児 instance はありません。この規則に
より、import の増減だけで既存コードの意味が変わることを防ぎます。

tuple、recordなど名前を持たない組み込み構造へのstructural instanceはcompilerが提供する
標準instanceだけを許可します。userlandは匿名record型をinstance headにできません。

aliasはinstance identityを作りません。instance headのaliasは展開してcoherence判定するため、
aliasと展開先へ別instanceを定義できません。

## 4.6 標準演算子との関係

次の演算子は標準 trait method の構文糖です。

| 演算子               | trait method        |
| -------------------- | ------------------- |
| `==`, `!=`           | `Eq.eq`             |
| `<`, `<=`, `>`, `>=` | `Ord.compare`       |
| `+`                  | `Add<L, R, O>.add`  |
| `-`                  | `Sub<L, R, O>.sub`  |
| `*`                  | `Mul<L, R, O>.mul`  |
| `/`                  | `Div<L, R, O>.div`  |
| `%`                  | `Rem<L, R, O>.rem`  |
| `**`                 | `Pow<L, R, O>.pow`  |
| `<$>`                | `Functor.map`       |
| `<*>`                | `Applicative.apply` |
| `>>=`                | `Monad.flatMap`     |

算術traitは左operand `L`、右operand `R`、出力 `O` を明示します。同じ `(trait, L, R)` に
複数の `O` を定義できません。これはoperator結果をoperand型だけから一意に決める
functional dependencyです。

算術traitは次の形を持ちます。Sub、Mul、Div、Rem、Powもmethod名以外は同じ形です。

```seseragi
trait Add<L, R, O> {
  fn add left: L -> right: R -> O
}
```

異なる型を組み合わせるinstanceは宣言できますが、暗黙変換は発生しません。
`Mul<Vector, Float, Vector>` は `Vector * Float` を許可しますが、`Int + Float` は対応instanceが
明示定義されていなければエラーです。

struct内の `operator` 宣言はこれら標準trait instanceの糖衣です。coherenceとorphan ruleは
通常のinstanceと同じです。

`operator ==` は `Eq<A>.eq` の糖衣で、戻り型はBoolです。`!=` はEq.eqの結果を否定するため、
個別instanceを持ちません。

## 4.7 laws

`Eq`、`Ord`、`Functor`、`Applicative`、`Monad` には標準ライブラリ文書で定める law が
あります。law は型検査では証明しませんが、instance の契約です。標準ライブラリは
law test helper を提供します。

## 4.8 method と型クラスの違い

- `impl User { ... }` は User 固有の名前空間に method を加える。
- `impl Eq<User> { ... }` は既存の抽象操作に User の instance を与える。
- method は `user.displayName` と名前解決する。
- trait method は constraint または完全な型情報から辞書を選ぶ。

両者を同じ dispatch 規則にはしません。

## 4.9 do notation

`do` は `Monad<M>` に対するbind列を読みやすく書く構文です。特定の `Effect` や `Task` に
限定しません。

```seseragi
do {
  user <- fetchUser id
  posts <- fetchPosts user.id
  let visible = filter isVisible posts
  pure { user, posts: visible }
}
```

do blockには次をsource順に書けます。

- `pattern <- monadicExpression`: bindしてpatternへ値を渡す。
- `let pattern = pureExpression`: monadを実行せずlocal値をbindする。
- `monadicExpression`: 結果を捨てて次へ進む。
- 最後のmonadic expression: block全体の結果。

最後の式は必須です。`pure value` は現在の `Applicative<M>` の `pure` を使います。

## 4.10 desugar

do blockは次の規則で右からdesugarします。

```text
do { x <- mx; rest }  = mx >>= (\x -> do { rest })
do { let x = v; rest } = [xをvへ束縛して] do { rest }
do { mx; rest }       = mx >>= (\_ -> do { rest })
do { last }           = last
```

これは意味の定義であり、backendは同じ評価順序とscopeを保つ別実装を選べます。

bind左辺はirrefutable patternでなければなりません。literal、ADT constructor、固定長Array
など失敗しうるpatternは使えません。pattern失敗を暗黙の `MonadFail` に変換しません。
必要ならbind後に網羅的な `match` を書きます。

## 4.11 do blockの型

各monadic expressionは同じ型構築子 `M<_>` を持ち、`Monad<M>` instanceが一意に必要です。
最初のbind、最後の式、または期待型から `M` を決定します。複数候補が残る場合は曖昧
エラーです。

異なるmonadをdo blockへ暗黙liftしません。transformerの `lift`、`Effect.fromEither` などを
明示して同じ `M` に揃えます。これにより、どのfailure・state・environmentが使われるかを
構文から追跡できます。

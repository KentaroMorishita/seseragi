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

trait method は型だけを宣言し、本体を持ちません。trait parameter は型または、
`F<_>` のような一階の型構築子 parameter にできます。trait は `where` で supertrait を
要求できます。

```seseragi
pub trait Functor<F<_>> {
  fn map<A, B> f: (A -> B) -> value: F<A> -> F<B>
}

pub trait Applicative<F<_>>
where Functor<F> {
  fn pure<A> value: A -> F<A>
  fn apply<A, B> wrapped: F<A -> B> -> value: F<A> -> F<B>
}
```

高階型は trait parameter と constraint の中だけで使えます。一般の値の型として
`F<_>` を使うことはできません。`Either<E, _>` と `Task<E, _>` のように、ちょうど
一つの `_` を残した型構築子の部分適用は trait の文脈で使えます。

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

## 4.6 標準演算子との関係

次の演算子は標準 trait method の構文糖です。

| 演算子               | trait method        |
| -------------------- | ------------------- |
| `==`, `!=`           | `Eq.eq`             |
| `<`, `<=`, `>`, `>=` | `Ord.compare`       |
| `+`                  | `Add.add`           |
| `-`                  | `Sub.sub`           |
| `*`                  | `Mul.mul`           |
| `/`                  | `Div.div`           |
| `%`                  | `Rem.rem`           |
| `**`                 | `Pow.pow`           |
| `<$>`                | `Functor.map`       |
| `<*>`                | `Applicative.apply` |
| `>>=`                | `Monad.flatMap`     |

算術 trait は左右と結果が同じ型です。異なる型を混ぜる演算は名前付き関数で表します。
これにより `Int + Float` のような暗黙変換は発生しません。

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

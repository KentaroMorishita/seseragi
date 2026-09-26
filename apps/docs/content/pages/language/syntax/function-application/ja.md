---json
{
  "schema": 1,
  "id": "language.syntax.function-application",
  "locale": "ja",
  "route": "/ja/docs/language/syntax/function-application/",
  "kind": "language",
  "title": "関数適用",
  "summary": "空白でカリー化された関数を適用し、束縛順序、評価順序、型引数、部分適用の規則を理解します。",
  "availability": "available",
  "spec": [
    "docs/spec/01-syntax.md#1.4",
    "docs/spec/02-types.md#2.7",
    "docs/spec/03-data-and-expressions.md#3.1"
  ],
  "examples": ["examples/spec/lessons/02-values-and-functions.ssrg"],
  "prerequisites": [],
  "related": [],
  "next": "",
  "referenceIdentities": []
}
---
# 関数適用

Seseragiでは、関数の後ろに引数を空白で続けて適用します。関数適用はすべての中置演算子より強く結合し、左結合なので、`add 1 2`は`(add 1) 2`を意味します。

## 引数を一つずつ適用する

関数の宣言には`fn`を使います。parameterの矢印一つがカリー化された引数一つを導入し、関数適用一回が引数一つを渡します。最初の引数だけを適用すると、新しい関数が返ります。

:::example {"title":"カリー化された関数と左結合の適用","source":"examples/spec/lessons/02-values-and-functions.ssrg"}
:::

parameterは左から右へ束縛されます。`add base 22`では、Seseragiは`add`、`base`、`22`の順に、それぞれ一度だけ評価します。通常の関数適用は正格で、最後の引数が揃うと関数本体をすぐに評価します。

:::common-mistake {"title":"括弧は複数引数呼び出しを作りません"}
`f(x, y)`はSeseragiの関数呼び出し構文ではありません。括弧は一つの式のgroup、tuple、またはUnitを表します。カリー化された引数は`f x y`で渡し、parameter型がtupleの場合だけtupleを渡します。
:::

## 部分適用

`add`の型が`Int -> Int -> Int`なら、`add 20`の型は`Int -> Int`です。返された関数は最初の引数を保持しますが、関数本体を先に評価しません。残りの引数を渡した時点で本体を評価します。

明示的なparameterを持たない関数には、匿名のUnit parameterがあります。型は`Unit -> A`になり、呼び出し側は`()`を明示的に適用します。

## 型引数と演算子

明示的な型引数はcalleeへ空白なしで続け、`identity<String> value`と書きます。空白がある`identity < value`は比較としてparseされます。custom infix operatorもoperandとの間に空白が必要です。一方、`(<+>)`のようなoperator valueは通常のカリー化された関数として適用できます。

:::design-rationale {"title":"関数適用はEffectを実行しません"}
`Effect<R, E, A>`を返す関数を適用すると、Effect値が構築されます。Effect本体は実行されません。実行は明示的なruntime boundaryでだけ始まるため、評価順序と外部作用が型に現れたままになります。
:::

## 複雑な引数をgroup化する

関数適用は、直後の強く結合した式を引数に取ります。中置式、conditional、match、do block、lambdaを一つの引数として渡し、周囲のprecedenceによる別のparseを避ける必要がある場合は括弧を使います。

calleeが関数でない場合は型errorです。引数の型が一致しない場合、明示型引数が不完全な場合、または残ったtrait constraintを解決できない場合も型検査に失敗します。

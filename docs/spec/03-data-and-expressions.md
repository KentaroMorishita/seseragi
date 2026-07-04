# 3. データ・式・パターン

## 3.1 評価規則

Seseragi は strict evaluation です。関数の本体は適用時に評価し、引数は左から右へ
一度だけ評価します。Effectの中身だけは実行境界まで評価しません。

値の同一性は観測できません。参照同一性を比較する演算子はありません。

## 3.2 let とブロック

```seseragi
let result = {
  let x = load input
  let y = normalize x
  y
}
```

ブロックは 0 個以上の `let` の後に最後の式を持ち、その式の値を返します。
最後の式を省略したブロックは `Unit` を返します。`let` の scope は、その直後から
ブロックまたは module の末尾までです。shadowing は許可します。

destructuring let も pattern を使いますが、反駁可能な pattern は使えません。

```seseragi
let (x, y) = point
let { name, age } = user
```

## 3.3 条件式

```seseragi
if score >= 80 then "pass" else "retry"
```

条件は必ず `Bool` です。`else` は必須で、両 branch は同じ型でなければなりません。
片方が `Never` の場合はもう片方の型になります。

## 3.4 ADT

ADT は有限個の variant を持つ nominal 型です。

```seseragi
pub type LoadState<A, E> =
  | Idle
  | Loading
  | Loaded A
  | Failed E
```

constructor は同名の値を module scope に導入します。

- `Idle: LoadState<A, E>`
- `Loaded: A -> LoadState<A, E>`
- `Failed: E -> LoadState<A, E>`

複数 payload は tuple 一つで表し、constructor 自体は常に一引数です。

```seseragi
type Shape =
  | Point
  | Circle Float
  | Rect (Float, Float)
```

## 3.5 struct

struct は名前付き field を持つ nominal product 型です。

```seseragi
pub struct User {
  id: UserId,
  name: String,
}

let user = User { id, name: "Aki" }
let renamed = User { ...user, name: "Mio" }
```

生成時は全 field が必要です。同名変数による field shorthand を使えます。spread は
同じ struct 型の値を一つだけ先頭に置けます。後続 field が値を置き換えます。

通常の公開struct fieldは宣言moduleの外からも読み取れます。表現を隠す型は `opaque struct`
と公開smart constructorを使います。

## 3.6 newtype

newtypeは「同じ表現だが意味の違う値」を、structのfield boilerplateなしで表します。

```seseragi
newtype UserId deriving Eq, Ord, Hash, Show = Int

let id = UserId 42
let raw = match id {
  UserId value -> value
}
```

constructor名は型名と同じです。constructor適用は内部値を一度だけ評価してwrapし、patternは
内部値をbindします。field access、record spread、struct updateはnewtypeに使えません。

型、constructor、patternを公開する規則と、`opaque` による表現隠蔽は6.4に従います。

## 3.7 record

record は名前を持たない structural product 型です。

```seseragi
let point = { x: 1, y: 2 }
let moved = { ...point, x: point.x + 1 }
```

record field 名は一意でなければなりません。複数 spread を左から右に適用でき、後続の
field が前の値を置き換えます。同じ field の型を変更する更新は、新しい record 型を
作ります。

## 3.8 tuple、Array、List

```seseragi
let pair = ("answer", 42)
let numbers = [1, 2, 3]
let names = `["Aki", "Mio"]
```

- `(A, B, ...)` は固定長 tuple。1 要素 tuple はありません。
- `[a, b]` は `Array<A>`。
- `` `[a, b] `` は `List<A>`。
- `head : tail` は List の cons で、右結合。

literalの要素は同じ型でなければなりません。空literalの要素型は期待型から決めます。
期待型がない `let empty = []` と ``let empty = `[]`` は要素型を決定できないため、型注釈が
必要です。

Arrayはstrictに全要素を保持する不変の連続列で、indexによるrandom accessを提供します。
Listは`Empty`と`Cons A (List<A>)`からなる不変のpersistent linked listです。先頭へのconsと
tail取得を定数時間で行えます。runtime表現は観測できず、backendは意味を保つ別表現を
選べます。

ArrayとListは別の型で、暗黙変換しません。変換は `std/array` の `toList` と `std/list` の
`toArray` を使います。
`array[index]` の型は `Maybe<A>` です。負のindexと範囲外indexは `Nothing` になります。
Listにはindex構文を提供しません。

Array patternは `[first, second, ...rest]`、List patternは
`` `[first, second, ...rest] `` と書きます。patternは先頭から照合し、`rest` は元と同じ
collection型です。restを持たないpatternは長さも完全に一致しなければなりません。

## 3.9 range と comprehension

`start..end` は終端を含まず、`start..=end` は終端を含む `Range<Int>` です。step は常に
1 で、start が end より大きい range は空です。降順には標準ライブラリ関数を使います。

```seseragi
let arraySquares = [n * n | n <- 1..=10, n % 2 == 0]
let listSquares = `[n * n | n <- 1..=10, n % 2 == 0]
```

`[expression | ...]` はArray、`` `[expression | ...] `` はListを返します。generatorとguardを
左から右へ処理し、複数generatorは左側を外側とする入れ子です。generator sourceの型 `C` と
要素型 `A` には `Iterable<C, A>` instanceが必要です。sourceのcollection型と結果のcollection型は
独立しており、暗黙のArray/List変換を意味しません。

generator clauseは `pattern <- source`、guard clauseはBool式です。patternが一致しない要素は
そのgeneratorから除外されます。各source式は、それを囲む外側generatorの一要素につき一度、
左から右に評価し、`iterate` が返す順序で走査します。comprehension自体はpureで、Effectを
generator sourceにできません。

各generatorはsourceを一度評価して `iterate` を一度呼び、`next` が `Nothing` を返すまで
`Just (value, rest)` のvalueを順にpatternへ照合します。この展開規則により、user-defined
IterableでもArray/List/Rangeと同じcomprehension semanticsになります。

## 3.10 match と pattern

```seseragi
match state {
  Idle -> "idle"
  Loading -> "loading"
  Loaded value -> show value
  Failed error when retryable error -> "retry"
  Failed _ -> "failed"
}
```

pattern は literal、binding、`_`、constructor、tuple、record、struct、Array、List を
入れ子にできます。同じ pattern 内で同じ名前を二度 bind できません。

guard は `Bool` です。guard 付き arm は網羅性に寄与しません。arm は上から評価し、
最初に pattern と guard が一致した arm の式を評価します。すべての arm の結果型は
一致しなければなりません。

`match` は必ず網羅的でなければなりません。到達不能な arm はコンパイルエラーです。

## 3.11 lambda

```seseragi
\x -> x + 1
\x: Int -> x + 1
\x y -> x + y
```

parameter型は文脈から推論できる場合に省略できます。複数parameterは最後の `->` の前へ
並べ、curried lambdaの入れ子へdesugarします。

```text
\x y -> body = \x -> \y -> body
```

parameterは左から右へscopeへ入り、後続parameterの型注釈とbodyから参照できます。

## 3.12 impl と method

```seseragi
impl User {
  fn displayName self: User -> String = self.name
}
```

`impl T` は nominal 型 `T` を定義した module にだけ書けます。method の第一 parameter
は `self: T` でなければなりません。同じ型に対する impl block は複数書けますが、
method 名は重複できません。

## 3.13 struct/newtypeのoperator overload

structとnewtypeは固定された標準operatorをoverloadできます。

```seseragi
struct Vector {
  x: Float,
  y: Float,
}

impl Vector {
  operator + self -> other: Vector -> Vector =
    Vector { x: self.x + other.x, y: self.y + other.y }

  operator * self -> scale: Float -> Vector =
    Vector { x: self.x * scale, y: self.y * scale }
}
```

`operator` 宣言は通常のinherent methodではなく、対応する標準trait instanceの糖衣です。
第一parameter `self` の型は囲むnominal型として補われます。

```text
operator + : Add<Vector, Vector, Vector>
operator * : Mul<Vector, Float, Vector>
```

利用できるbinary operatorは `+`, `-`, `*`, `/`, `%`, `**`, `==` です。`!=` は `==` の
否定として得ます。順序比較は一貫した全順序を必要とするため、個別operatorではなく
`Ord` instanceを実装します。

nominal型のoverload宣言では独自operator token、優先順位、結合性を宣言できません。
custom operatorは1.8のtop-level構文で別に定義します。operator解決はoperandの静的型で
行い、runtime dispatchはありません。

generic structでも宣言できます。

```seseragi
struct GenericVector<A> {
  x: A,
  y: A,
}

impl<A> GenericVector<A>
where Add<A, A, A> {
  operator + self -> other: GenericVector<A> -> GenericVector<A> =
    GenericVector { x: self.x + other.x, y: self.y + other.y }
}
```

同じtrait instanceを明示 `impl Add<...>` と `operator` 糖衣の両方で定義すると重複instance
エラーです。

newtypeでも同じ規則を使います。内部表現のoperator instanceはnewtypeへ自動継承されません。

```seseragi
newtype Score = Int

impl Score {
  operator + self -> other: Score -> Score =
    match (self, other) {
      (Score left, Score right) -> Score (left + right)
    }
}
```

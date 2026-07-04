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

## 3.6 record

record は名前を持たない structural product 型です。

```seseragi
let point = { x: 1, y: 2 }
let moved = { ...point, x: point.x + 1 }
```

record field 名は一意でなければなりません。複数 spread を左から右に適用でき、後続の
field が前の値を置き換えます。同じ field の型を変更する更新は、新しい record 型を
作ります。

## 3.7 tuple、Array、List

```seseragi
let pair = ("answer", 42)
let numbers = [1, 2, 3]
let names = `["Aki", "Mio"]
```

- `(A, B, ...)` は固定長 tuple。1 要素 tuple はありません。
- `[a, b]` は `Array<A>`。
- `` `[a, b] `` は `List<A>`。
- `head : tail` は List の cons で、右結合。

Array と List は別の型で、暗黙変換しません。`array[index]` の型は `Maybe<A>` です。
負の index と範囲外 index は `Nothing` になります。

## 3.8 range と comprehension

`start..end` は終端を含まず、`start..=end` は終端を含む `Range<Int>` です。step は常に
1 で、start が end より大きい range は空です。降順には標準ライブラリ関数を使います。

```seseragi
let squares = [n * n | n <- 1..=10, n % 2 == 0]
```

comprehension はArrayを返します。generatorとguardを左から右へ処理します。
generatorのsource型が `F<A>` のとき、`Iterable<F>` constraintを満たさなければなりません。

## 3.9 match と pattern

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

## 3.10 lambda

```seseragi
\x -> x + 1
\x: Int -> x + 1
```

parameter 型は文脈から推論できる場合に省略できます。複数 parameter は lambda を
入れ子にします。

## 3.11 impl と method

```seseragi
impl User {
  fn displayName self: User -> String = self.name
}
```

`impl T` は nominal 型 `T` を定義した module にだけ書けます。method の第一 parameter
は `self: T` でなければなりません。同じ型に対する impl block は複数書けますが、
method 名は重複できません。

## 3.12 structのoperator overload

structは固定された標準operatorをoverloadできます。

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
第一parameter `self` の型は囲むstruct型として補われます。

```text
operator + : Add<Vector, Vector, Vector>
operator * : Mul<Vector, Float, Vector>
```

利用できるbinary operatorは `+`, `-`, `*`, `/`, `%`, `**`, `==` です。`!=` は `==` の
否定として得ます。順序比較は一貫した全順序を必要とするため、個別operatorではなく
`Ord` instanceを実装します。

struct内のoverload宣言では独自operator token、優先順位、結合性を宣言できません。
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

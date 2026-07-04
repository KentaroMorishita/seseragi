# 2. 型システム

## 2.1 基本性質

Seseragi は静的型付きです。すべての式は実行前に型を持ち、型検査に失敗した
プログラムは実行されません。

型推論は Hindley-Milner 型推論を基礎とし、次を加えます。

- nominal な ADT と struct
- immutable record の structural typing
- trait constraint
- `Never` からの底型 coercion

暗黙の `Any` はありません。型を確定できない場合は `Unknown` に逃がさず、注釈を
要求します。

## 2.2 組み込み型

| 型       | 意味                          |
| -------- | ----------------------------- |
| `Int`    | 符号付き 64 bit 整数          |
| `Float`  | IEEE 754 binary64             |
| `Bool`   | `True` または `False`         |
| `String` | Unicode scalar value の不変列 |
| `Unit`   | 唯一の値 `()` を持つ型        |
| `Never`  | 値を一つも持たない型          |

`Never` は任意の型が要求される位置で使えます。`Unit` と `Never` は異なります。

## 2.3 型構築子

```text
Maybe<A>       欠如する可能性のある値
Either<E, A>   E で失敗したか A で成功した同期的な結果
Task<E, A>     E で失敗しうる遅延・非同期計算
Array<A>       不変の連続列
List<A>        不変の連結リスト
(A, B, ...)    tuple
{ x: A, y: B } record
A -> B         関数
```

型引数の区切りは `,` です。関数型は右結合なので `A -> B -> C` は
`A -> (B -> C)` です。

## 2.4 型注釈と推論

関数 parameter と公開宣言には型注釈が必要です。非公開 `let` の型は推論できます。
関数の戻り型は必須です。これは module API と再帰関数の意味を安定させるためです。

```seseragi
fn add x: Int -> y: Int -> Int = x + y

let count = 3
let label: String = "ready"
```

型 parameter は宣言名の直後に書きます。

```seseragi
fn identity<A> value: A -> A = value
```

## 2.5 多相性と value restriction

トップレベルの関数宣言は一般化されます。`let` は右辺が値である場合だけ一般化されます。
値とは、literal、lambda、constructor の値適用、tuple、record、Array、List のうち、
全要素が値であるものです。

関数適用、method 適用、`if`、`match`、`Task` を生成する計算結果は一般化されません。
これを value restriction と呼びます。

## 2.6 nominal 型と structural 型

ADT と struct は nominal です。field 構成が同じでも別の struct は互換ではありません。

record は structural です。record `R1` は、`R2` が要求する全 field を同じ型で持つとき
`R2` の subtype です。field は不変なので、この width subtyping は読み取りに対して
安全です。

```seseragi
fn nameOf value: { name: String } -> String = value.name

let user = { id: 1, name: "Aki" }
let name = nameOf user
```

record field の型に対する暗黙の depth subtyping はありません。

### 型 alias

```seseragi
pub alias UserName = String
```

alias は型に別名を付けますが、新しい nominal 型を作りません。上の `UserName` と
`String` は同じ型です。alias は循環できません。値を区別したい場合は、一 field の
struct または ADT を使います。

## 2.7 関数型と curry

複数 parameter の関数宣言は、単一 parameter 関数の入れ子です。

```seseragi
fn add x: Int -> y: Int -> Int = x + y
```

の型は `Int -> Int -> Int` で、`add 1` の型は `Int -> Int` です。
parameter は左から右へ bind されます。

## 2.8 型の同一性と coercion

Seseragi に一般的な暗黙変換はありません。

- `Int` と `Float` は別の型。`toFloat` を明示する。
- `A` は自動で `Maybe<A>` や `Either<E, A>` にならない。
- struct は同じ field の record にならない。
- `Js.Nullable<A>` は `Maybe<A>` にならない。境界関数を使う。

許される暗黙 coercion は `Never` から任意型への coercion と record の width subtyping
だけです。

## 2.9 再帰

名前付き関数と type 宣言は再帰できます。再帰する関数は戻り型を必ず注釈します。
通常の `let` は自身を参照できません。相互再帰関数は同じ `rec` group に置きます。

```seseragi
rec {
  fn even n: Int -> Bool = if n == 0 then True else odd (n - 1)
  fn odd n: Int -> Bool = if n == 0 then False else even (n - 1)
}
```

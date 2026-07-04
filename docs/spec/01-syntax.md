# 1. 字句・構文・演算子

## 1.1 ソーステキスト

ソースは UTF-8 です。改行は LF と CRLF を受理し、意味上は同じです。識別子は
Unicode XID_Start で始まり、後続に Unicode XID_Continue または `'` を持てます。
キーワードは ASCII で、大文字と小文字を区別します。

```seseragi
let userName = "Aki"
let 次の値 = 42
let account' = update account
```

`//` から行末まではコメントです。ブロックコメントはありません。

## 1.2 リテラル

- `42`, `-42`: `Int`
- `3.14`, `-0.5`: `Float`
- `True`, `False`: `Bool`
- `"text"`: `String`
- `` `hello ${name}` ``: `String` のテンプレート
- `` `[1, 2] ``: `List<Int>` のような List literal
- `()`: `Unit`

負号は数値リテラルの一部ではなく前置演算子です。整数は符号付き 64 bit、Float は
IEEE 754 binary64 です。整数演算はラップせず、範囲外になった時点で runtime defect
として停止します。

backtick の直後が `[` なら List literal、それ以外なら template String と字句解析します。

String は Unicode scalar value の列です。添字アクセスは提供せず、文字・grapheme・
byte のどれを扱うかを標準ライブラリ関数で明示します。

## 1.3 レイアウト

改行とインデントは、ブロックの境界を決めません。ブロックは `{` と `}` で囲みます。
宣言や式の区切りには改行または `;` を使えます。括弧内では改行を自由に置けます。

formatter は 2 spaces を標準表現としますが、インデント幅は構文の意味を持ちません。

## 1.4 関数適用

通常の関数適用は空白で書き、左結合です。

```seseragi
add 1 2
map toString values
```

`add 1 2` は `(add 1) 2` です。`f(x, y)` という複数引数呼び出し構文はありません。
括弧は式のグループ化、tuple、Unit にだけ使います。

引数のない特別な関数もありません。遅延計算を表す関数は `Unit -> A` とし、
`clock ()` のように呼びます。

## 1.5 method 呼び出し

`value.method arg` は `Type.method value arg` へ解決される糖衣構文です。

```seseragi
account.deposit 1000
```

method は動的 dispatch ではありません。receiver の静的型と、対応する `impl` から
一意に名前解決します。`value method arg` という空白だけの method 構文はありません。

## 1.6 パイプラインと低優先順位適用

```seseragi
value |> normalize |> validate
show $ expensiveComputation input
```

- `x |> f` は `f x`。
- `f $ x` は `f x`。
- `|>` は左結合。
- `$` は右結合で、すべての二項演算子より優先順位が低い。

`|` 単独は ADT variant と comprehension の区切りに使い、pipeline には使いません。

## 1.7 演算子

優先順位は高い順に次のとおりです。同じ段の演算子は、明記がなければ左結合です。

|  段 | 演算子                                       | 結合       |
| --: | -------------------------------------------- | ---------- |
|   9 | field / method `.`, index `[]`               | 左         |
|   8 | 関数適用                                     | 左         |
|   7 | `!`, unary `-`                               | 右         |
|   6 | `**`                                         | 右         |
|   5 | `*`, `/`, `%`                                | 左         |
|   4 | `+`, `-`, list cons `:`                      | `:` のみ右 |
|   3 | `==`, `!=`, `<`, `<=`, `>`, `>=`             | 結合不可   |
|   2 | `&&`                                         | 左         |
|   1 | <code>&#124;&#124;</code>                    | 左         |
|   0 | `>>=`, `<$>`, `<*>`, <code>&#124;&gt;</code> | 左         |
|  -1 | `$`                                          | 右         |

比較演算子を連鎖できません。`a < b < c` はエラーで、`a < b && b < c` と書きます。
`&&` と `||` は短絡評価します。それ以外の二項演算子は左 operand、右 operand の順に
評価します。

## 1.8 予約語

次は予約語です。

```text
as alias else False fn foreign from if impl import let match pub pure rec
struct then trait True type when where
```

型名、constructor、trait 名は大文字、値、関数、field、module alias は小文字から
始めることを要求します。`_` は wildcard で、名前として参照できません。

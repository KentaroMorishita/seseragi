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
- `` `hello ${name}` ``: 型安全な `String` template
- `` `[1, 2] ``: `List<Int>` のような List literal
- `()`: `Unit`

負号は数値リテラルの一部ではなく前置演算子です。整数は符号付き 64 bit、Float は
IEEE 754 binary64 です。整数演算はラップせず、範囲外になった時点で runtime defect
として停止します。

backtick の直後が `[` なら List literal、それ以外なら template String と字句解析します。

template内の `${expression}` は任意の式を一度だけ評価し、その型 `A` に対する
`Show<A>` instanceで文字列化します。instanceが存在しない場合はコンパイルエラーです。
評価は左から右で、template全体は純粋な `String` です。

```seseragi
let message = `user: ${user}, score: ${score}`
```

これは概念上、文字列断片と `show user`、`show score` の連結へ展開されます。ただし展開は
意味の定義であり、同じ結果と評価順序を保つ限り、backendは中間文字列を作らない実装を
選べます。format placeholder専用の書式言語、可変長引数、暗黙のI/Oはありません。

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

明示型引数はcallee名へ空白なしで続けます。`identity<String> value` は型適用、
`identity < value` は比較です。custom infix operatorはoperandとの間に空白を必須とし、
`left <+> right` のように書きます。operator関数値 `(<+>)` の内側には空白を入れません。

情報を受け取らない関数は、名前のないUnit parameterを省略して宣言できます。

```seseragi
fn clock -> Instant = ...
```

意味上は名前のない `Unit` parameterを一つ持ち、関数の型は `Unit -> Instant`、呼び出しは
`clock ()` です。`Unit` は型、`()` はその唯一の値です。formatterも名前のないUnit parameterを
上の省略形で出力します。

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
- `$` は右結合で、Signal更新 `:=` を除くすべての二項演算子より優先順位が低い。

`|` 単独は ADT variant と comprehension の区切りに使い、pipeline には使いません。

`$` は通常の関数適用と同じ型付け・評価を持つ、括弧削減のための固定構文です。

```seseragi
show $ normalize $ load input
render $ values |> map toView
consume $ if ready then cached else build ()
```

それぞれ次と同じ意味です。

```seseragi
show (normalize (load input))
render (map toView values)
consume (if ready then cached else build ())
```

`f $ g $ x` は `f $ (g $ x)` と解析します。左辺を一度評価し、次に右辺を評価して、一引数の
関数適用を一度行います。`$` 自体は評価を遅延せず、Effectを実行せず、関数を複数引数で
呼び出す仕組みにもなりません。左辺が関数型でない場合は通常の関数適用と同じ型エラーです。

`$` の直後では改行でき、右辺に `if`、`match`、`do`、lambdaを括弧なしで置けます。`$` は
overload、再定義、`($)`による関数値化ができません。compilerは型検査前に通常applicationへ
desugarしても、同じ制約を直接生成しても構いません。

## 1.7 演算子

優先順位は高い順に次のとおりです。同じ段の演算子は、明記がなければ左結合です。

|  段 | 演算子                                       | 結合       |
| --: | -------------------------------------------- | ---------- |
|   9 | field / method `.`, index `[]`               | 左         |
|   8 | 関数適用                                     | 左         |
|   7 | `!`, unary `-`, signal read unary `*`        | 右         |
|   6 | `**`                                         | 右         |
|   5 | `*`, `/`, `%`                                | 左         |
|   4 | `+`, `-`, list cons `:`                      | `:` のみ右 |
|   3 | `==`, `!=`, `<`, `<=`, `>`, `>=`             | 結合不可   |
|   2 | `&&`                                         | 左         |
|   1 | <code>&#124;&#124;</code>                    | 左         |
|   0 | Maybe fallback `??`                          | 右・短絡   |
|  -1 | `>>=`, `<$>`, `<*>`, <code>&#124;&gt;</code> | 左         |
|  -2 | `$`                                          | 右         |
|  -3 | signal set `:=`                              | 右         |

比較演算子を連鎖できません。`a < b < c` はエラーで、`a < b && b < c` と書きます。
`&&`、`||`、`??` は短絡評価します。それ以外の二項演算子は左operand、右operandの順に
評価します。`??` の型とfallback規則は5.2に従います。

二項の算術・比較・cons・型クラスoperatorは、括弧で囲むと通常のcurried function値として
参照できます。

```seseragi
let add = (+)
let total = numbers |> reduce 0 (+)
```

overloadされたoperator referenceの型は期待型とtrait constraintから決めます。候補が一意に
ならなければ型注釈が必要です。短絡評価する `(&&)` と `(||)`、pipeline、`$`、Signal用operator、
prefix operatorは関数値として参照できません。

## 1.8 custom operator

userlandは二項infix operatorをtop-levelで定義できます。

```seseragi
pub operator<A> infixr 5 <+>
  left: A -> right: A -> A
where Semigroup<A> =
  append left right
```

この宣言は、次を同時に定義します。

- symbol `<+>`
- 右結合 `infixr`
- 優先順位 `5`
- 型scheme `forall A. Semigroup<A> => A -> A -> A`
- operator本体

`left <+> right` は `(<+>) left right` へdesugarします。`(<+>)` は通常のcurried function値として
参照・部分適用できます。

fixityは `infixl`、`infixr`、`infix` のいずれかです。`infix` は非結合で、同じoperatorを
括弧なしに連鎖できません。優先順位は0から8の整数で、値が大きいほど強く結合します。
関数適用とpostfix accessより強いcustom operatorは作れません。

operator symbolはASCIIの次の文字を2文字以上組み合わせます。

```text
! $ % & * + - . / : < = > ? @ ^ | ~
```

標準operator、`->`、`<-`、`..`、`...`、`//`、`??`、`:=` は予約済みで再定義できません。
prefixとpostfixのcustom operatorはありません。

`<<`、`>>`、`>>>` のように `<` と `>` だけからなるsymbolはgeneric delimiterと衝突するため、
custom operatorとして宣言できません。

custom operatorは値namespaceとは別のoperator namespaceに入り、明示importします。

```seseragi
import { operator <+> } from "./semigroup"
```

同じsymbolを二つscopeへ導入すると曖昧エラーです。同じ優先順位で異なるfixityのoperatorを
一つの式に混ぜる場合は括弧が必要です。operator宣言はmodule interfaceのpre-scanで収集する
ため、source内の宣言位置より前でも使えます。

raw scannerは宣言済みoperatorを参照せず、operator文字列をlosslessに保持します。header scan、
module interface解決、flat operator chain、fixity resolutionの共通手順は
[Parser・formatter・language server契約](./12-tooling.md)に従います。

標準operatorの型別overloadはcustom operator定義ではなく、standard trait instanceまたは
struct内の `operator` 糖衣で行います。

## 1.9 予約語

次は予約語です。

```text
as alias do effect else fails False fn for foreign from if impl import infix infixl infixr let
match newtype opaque operator pub rec struct then trait True type when where with
```

`constructor`、`method`、`property`、`value`、`pure`、`task` はforeign block内だけでkeywordに
なるcontextual keywordです。`self` はimpl methodの第一parameter位置でだけ特別な意味を
持ちます。

型名、constructor、trait 名は大文字、値、関数、field、module alias は小文字から
始めることを要求します。`_` は wildcard で、名前として参照できません。

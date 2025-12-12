# Seseragi 言語仕様（ソースコード準拠）

この文書は `src/lexer.ts` / `src/parser.ts` / `src/ast.ts` / `src/codegen/**` / `src/module-resolver.ts` に基づき、現在実装されている Seseragi の言語仕様を体系的にまとめたものです。  
実装が未完の箇所や将来拡張用に予約されている構文・キーワードは、その旨を明記しています。

---

## 1. 概要

Seseragi は **式中心の関数型言語**で、プログラムは `.ssrg` ファイルに記述します。

- 主要特徴
  - カリー化（右結合）を前提とした関数定義と呼び出し
  - 代数的データ型（ADT）とパターンマッチ
  - レコード / 構造体（struct）とメソッド・演算子オーバーロード
  - Maybe / Either / List / Array / Signal / Task / Tuple などの組み込み型とモナド演算子
  - Promise ブロックと `try` による Either 化
- 実行モデル
  - `seseragi run file.ssrg` で TypeScript に変換・実行するトランスパイル方式。
  - セマンティクスの多くは `@seseragi/runtime` の関数に対応します。

---

## 2. 字句仕様

### 2.1 コメント / 空白 / 改行

- 行コメント: `//` から行末まで。トークン種別 `COMMENT`。
- 空白はトークン `WHITESPACE` として lexer が保持するが、parser はほぼ無視する。
- 改行はトークン `NEWLINE`。多くの構文で改行は区切りとして扱われる。

### 2.2 リテラル

- 整数: `INTEGER`（10進）。型は `Int`。
- 浮動小数: `FLOAT`。型は `Float`。
- 文字列: `STRING`（`"..."` / `'...'`）。型は `String`。
- テンプレート文字列: `` `...${expr}...` `` → `TemplateExpression`（型は `String`）。
- 真偽値: `True` / `False` → `BOOLEAN`（型は `Bool`）。
- Unit リテラル: `()` は式としてパースされ、型は `Unit`。

### 2.3 識別子

- `IDENTIFIER`: `[A-Za-z_][A-Za-z0-9_']*` 相当。
- 先頭が大文字の識別子は **コンストラクタ名** として扱われる可能性がある。

### 2.4 キーワード（実装済み）

`fn`, `effectful`, `let`, `type`, `struct`, `impl`, `operator`, `monoid`,  
`import`, `as`, `from`, `return`,  
`if`, `then`, `else`,  
`match`, `when`,  
`promise`, `resolve`, `reject`, `try`,  
`is`

### 2.5 予約済み（lexer にあるが未実装）

`elevate`, `pure`, `perform`, `case`, `Void`（型としては実装済み）  
これらは現状の parser / codegen では文法として使用できません。

---

## 3. プログラム構造

プログラムは **文（Statement）の列**。

### 3.1 文の種類

1. 関数定義
   - `fn name <TypeParams?> Signature = Expr`
   - `fn name <TypeParams?> Signature { Block }`
   - `effectful fn ...` で副作用関数としてマーク（コード生成・LSP用途）
2. 変数束縛
   - `let name : Type? = Expr`
   - パターン束縛（分割代入）
     - タプル: `let (x, y, ...) = Expr`
     - レコード: `let {x, y, ...} = Expr`
     - 構造体: `let StructName {x, y, ...} = Expr`
3. 型定義
   - ADT: `type Name<T...?> = | Ctor Type* | ...`
   - 型エイリアス: `type Name<T...?> = Type`
   - `type Name { field: Type, ... }` は struct 型定義（ADT と同じ `TypeDeclaration` に落ちる）
4. struct 定義
   - `struct Name { field: Type, ... }`
5. impl ブロック
   - `impl Name { fn ... ; operator ... ; monoid ... }`
6. import
   - `import { item (as alias)? (, ...)? } from "relative/path"`
7. return
   - ブロック内の早期終了用: `return Expr`
8. 式文
   - 上記に当てはまらないトップレベルの式は `ExpressionStatement` として扱う。

---

## 4. 型システム

### 4.1 基本型

プリミティブ型（`PrimitiveType`）:

- `Int`, `Float`, `Bool`, `String`, `Char`, `Unit`, `Void`
- `Void` は型として存在するが、値の構文は未提供。

### 4.2 関数型

- 右結合: `A -> B -> C` は `A -> (B -> C)`。
- 単一引数関数のカリー化が標準。
- `fn` 定義では引数の列と戻り値型から `FunctionType` を構築。

### 4.3 ジェネリクス

- 型パラメータ: `fn id<T> x: T -> T = x`
- 型引数: `f<Int> 1`
- `TypeParameter` はスコープごとに管理され、未指定なら推論（ただし推論は限定的）。

### 4.4 Union / Intersection

- Union: `A | B`
- Intersection: `A & B`
- 実装上、Union / Intersection に現れる型は **事前に定義されている必要がある**（parser で検査）。

### 4.5 タプル / レコード / struct 型

- タプル型: `(A, B, C)`（2要素以上必須）
- レコード型: `{x: A, y: B}`
- struct 型: `StructType`（`struct` 定義によって環境に登録される）

### 4.6 組み込みジェネリック型（構文上の既知名）

parser の型存在チェックで特別扱いされる:

`Maybe<T>`, `Either<L, R>`, `List<T>`, `Array<T>`, `Tuple<T...>`,  
`Signal<T>`, `Task<T>`

---

## 5. 式

### 5.1 式の優先順位（高 → 低）

1. **primary / postfix**
   - リテラル、識別子、括弧、配列・リスト・レコード・struct リテラル、ラムダ、promise、resolve/reject、try 等
   - postfix: `.field`, `[index]`, `method args`, `as Type`
   - 括弧なし関数適用（空白区切り）もここで右結合的に処理
2. 冪乗: `**`
3. 乗除剰余: `* / %`
4. 加減: `+ -`
5. 比較 / 等価 / 型判定:
   - `== != < > <= >=`
   - `is Type` → `IsExpression`
6. 論理積 / 論理和: `&& ||`
7. 範囲: `a .. b`, `a ..= b` → `RangeLiteral`
8. cons: `head : tail`（右結合）
9. 逆パイプ: `a ~ f` → `ReversePipe`
10. ファンクターマップ: `f <$> x` → `FunctorMap`
11. アプリカティブ適用: `fs <*> xs` → `ApplicativeApply`
12. モナドバインド: `m >>= f` → `MonadBind`
13. パイプ: `x | f` → `Pipeline`（概念的には `|>`）
14. nullish coalescing: `a ?? b` → `NullishCoalescingExpression`
15. 三項: `cond ? a : b` → `TernaryExpression`
16. 関数適用演算子: `f $ x`（右結合）

※ parser 実装の順に忠実に記載。

### 5.2 関数適用

#### 括弧なし（標準）

`f x y` は `(f x) y` として解析される。

#### 括弧あり

`f(x, y)` は `FunctionCall`（複数引数）。

#### 型引数付き

`f<Int, String> x`  
`<...>` の直後が数値リテラル等の場合は比較演算子として扱われるため、曖昧性回避のロジックが入っている。

### 5.3 if 式

```
if cond then expr1 else expr2
```

`ConditionalExpression`。改行は任意箇所で許容。

### 5.4 match 式

```
match expr {
  Pattern -> Expr
  Pattern -> Expr
}
```

- ケースは 1 つ以上必要。
- ケース結果の型は全て同一である必要がある（typechecker）。

### 5.5 ブロック式

```
{
  Statement*
  Expr?
}
```

- 最後が式の場合は戻り値扱い（`return` がなくても可）。
- `return Expr` が現れた場合、その時点でブロックの戻り値が確定。

### 5.6 ラムダ式

```
\x -> Expr
\x: T -> Expr
\x y -> Expr   // カリー化
```

引数型 `_` は推論用プレースホルダ（typechecker 側で型変数に置換）。

### 5.7 レコード / struct / 配列 / リスト

#### レコードリテラル

```
{ x: 1, y: 2 }
{ x, y }          // shorthand
{ ...r, z: 3 }    // spread
```

型は `RecordType`（フィールド型を式から推論）。

#### struct リテラル

```
Point { x: 1, y: 2 }
Point { x, y }
Point { ...p, y: 3 }
```

`StructExpression`。  
`struct` 定義のフィールドが全て与えられる必要があり、型一致も検査される。

#### 配列リテラル

`[1, 2, 3]` → `ArrayLiteral`（JavaScript 配列に変換）。

#### 配列アクセス（安全アクセス）

`arr[i]` → **Maybe を返す安全アクセス**に変換される。

- 範囲外なら `Nothing`
- 範囲内なら `Just(value)`

#### タプルリテラル

```
(a, b)
(a, b, c)
```

カンマ区切りで 2 要素以上の場合 `TupleExpression`。  
単なるグルーピングの括弧 `(expr)` とは区別される。

#### リスト（Seseragi List）

リストシュガーはバッククォート付き:

```
`[1, 2, 3]   // ListSugar
```

生成コードは `Cons(1, Cons(2, Cons(3, Empty)))`。

cons 演算子:

`a : b` → `Cons(a, b)`

### 5.8 範囲リテラル

- `a .. b` : end を含まない配列
- `a ..= b` : end を含む配列

いずれも JS の `Array.from` で生成される。

### 5.9 リスト内包表記

配列を返す版:

```
[ Expr | x <- iterable, y <- iterable2, filter1, filter2 ]
```

Seseragi List を返す sugar:

```
`[ Expr | x <- iterable, filter ]
```

生成コードは `filter` → `map` → 必要なら `arrayToList`。

### 5.10 Signal

- 生成: `Signal expr` または `Signal(expr)` → `createSignal(expr)`
- 代入: `sig := v` → `setSignal(sig, v)`  
  （二項演算子 `:=` も特別扱いで `.setValue` に変換される経路がある）
- 組み込み:
  - `subscribe signal observer`
  - `unsubscribe signal`
  - `detach signal`

### 5.11 モナド / ファンクタ演算子

式レベルの演算子として以下を持つ:

- `|` : Pipeline  
- `~` : ReversePipe（概念的には `<|`）
- `<$>` : FunctorMap
- `<*>` : ApplicativeApply
- `>>=` : MonadBind
- `>>>` : FoldMonoid（実装は placeholder）
- `$` : FunctionApplicationOperator

対象型は `Maybe`, `Either`, `List`, `Array`, `Signal`, `Task`, `Tuple`。  
コード生成時に左辺の型から適切な `mapX / applyX / bindX` をディスパッチする。

### 5.12 Nullish Coalescing（??）

`a ?? b`:

- `a` が `Maybe<T>` の場合 `fromMaybe(b, a)`
- `a` が `Either<L, R>` の場合 `fromRight(b, a)`
- 両辺が Maybe / Either の場合は `Just/Right` なら左、そうでなければ右の中身を返す特別処理
- それ以外は JS の `??`

### 5.13 Promise ブロック / resolve / reject

```
promise<T?> {
  Statement*
  Expr?
}
```

生成コード: `() => new Promise<T>((resolve, reject) => { ... })`

- ブロック内の `resolve x` / `reject e` は JS の `resolve(x)` / `reject(e)`。
- ブロック外の `resolve<T?> x` は `() => Promise.resolve<T>(x)` として独立した Promise 関数になる。

### 5.14 try 式

```
try expr
try expr as ErrorType
```

`expr` を Either 化する糖衣構文:

- 同期式 → `() => Either<L, T>`
- Promise 系式（`promise {...}` / `resolve` / `reject` など） → `async () => Promise<Either<L, T>>`

型注釈 `as L` が無い場合は `Left(String(error))`。

---

## 6. パターン

`match` と `let` の分割代入で利用。

### 6.1 基本

- ワイルドカード: `_`
- 変数パターン: `x`
- リテラル: `1`, `1.0`, `"a"`, `True`

### 6.2 コンストラクタパターン

- 引数あり: `Some x`, `Circle 5.0 r`
- 引数なし: `None`

コンストラクタ名の判定は

- 組み込みコンストラクタ（`Just`, `Nothing`, `Left`, `Right`, `Cons`, `Empty`, `Signal`, `Task`）
- もしくは `type` 定義で登録された ADT のバリアント

に一致する場合に行われる。

### 6.3 Or パターン

`p1 | p2 | p3`

### 6.4 Guard パターン

`pattern when condition`

### 6.5 タプル / レコード / struct / 配列パターン

- タプル: `(p1, p2, ...)`
- レコード: `{x, y}`, `{x: px, y: py}`
- struct: `StructName {x, y}`
- 配列:
  - `[]`
  - `[p1, p2, ...rest]`
  - `[...rest]`

### 6.6 リストパターン（Seseragi List）

バッククォート付き:

```
`[p1, p2, ...rest]
`[]
```

---

## 7. ADT / struct / impl

### 7.1 ADT（代数的データ型）

```
type Maybe<T> =
  | Just T
  | Nothing
```

- 先頭バリアントの前に `|` が必要（parser 仕様）。
- コンストラクタ引数は **型列**として書く（`RGB Int Int Int`）。
- 引数がない場合のバリアントは `Unit` として保持される。

### 7.2 型エイリアス

```
type UserId = Int
type Pair<T, U> = (T, U)
```

### 7.3 struct

```
struct Point {
  x: Int
  y: Int
}
```

### 7.4 impl ブロック（メソッド）

```
impl Point {
  fn add self: Point -> other: Point -> Point = ...
}
```

- `self` は第一引数の暗黙受け取りとして扱われる。
- メソッド呼び出し:
  - `p add q`（空白区切り）
  - `p.add(q)`（括弧版）

### 7.5 operator 宣言（演算子オーバーロード）

```
impl Point {
  operator + self: Point -> other: Point -> Point = self add other
}
```

対応演算子:
`+ - * / % ** == != < > <= >= && ||`

構造体の二項演算は `__dispatchOperator(left, op, right)` に変換される。

### 7.6 monoid 宣言

```
impl String {
  monoid {
    identity ""
    operator + self: String -> other: String -> String = ...
  }
}
```

- `identity` は現状キーワード化されておらず識別子として消費される。
- `>>>` による `foldMonoid` は実装上 placeholder。

---

## 8. モジュール / import

```
import { f, TypeName as T } from "./path/to/module"
```

- `from` に指定できるのは **相対パスのみ**（`./` / `../`）。
- `.ssrg` 拡張子は省略可（resolver が補完）。
- インポート対象として解決されるエクスポート:
  - `fn` 定義
  - `type` / `struct` 定義
  - `impl` は struct に紐づく形で内部保持される
- `let` は **意図的にエクスポートされない**。

---

## 9. 組み込み関数 / コンストラクタ

### 9.1 組み込み関数

単一引数（括弧なし呼び出しが標準）:

`print`, `putStrLn`, `show`, `toString`, `toInt`, `toFloat`, `head`, `tail`,  
`typeof`, `typeof'`, `unsubscribe`, `detach`

二引数:

`subscribe signal observer`

### 9.2 組み込みコンストラクタ

- Maybe: `Just`, `Nothing`
- Either: `Left`, `Right`
- List: `Empty`, `Cons`
- Task: `Task`
- Signal: `Signal`（式としては `SignalExpression` / `createSignal` に変換）

---

## 10. 既知の制限 / 未完部分

- 静的型検査はプリミティブ・関数・struct・レコード・match・一部コンストラクタに限定されている。
  - モナド演算子、配列 / List / Signal / Promise / Try / Nullish などは typechecker で未対応。
  - 実際の型挙動は codegen と runtime に従う。
- `elevate`, `pure`, `perform`, `case` は予約語だが構文未実装。
- `foldMonoid`（`>>>`）は placeholder で、identity の扱い含め未完成。

---

## 付録: 記法まとめ

- 関数: `fn name x: T -> U = expr`
- effectful: `effectful fn print(msg: String) -> IO<Unit> = ...`
- ADT: `type Maybe<T> = | Just T | Nothing`
- struct: `struct P { x: Int, y: Int }`
- impl/method: `impl P { fn add self: P -> other: P -> P = ... }`
- operator: `operator + self: P -> other: P -> P = ...`
- match: `match v { Just x -> x, Nothing -> 0 }`
- list: `` `[1,2,3] `` / `a : b`
- array: `[1,2,3]` / `arr[i]`
- monad ops: `<$>`, `<*>`, `>>=`, `|`, `~`, `$`
- promise: `promise<Int> { resolve 1 }`
- try: `try (promise { ... }) as MyError`

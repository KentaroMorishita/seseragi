# Seseragi 言語仕様書

## 1. はじめに

Seseragiは、TypeScriptにコンパイルされる静的型付けプログラミング言語です。不変性、代数的データ型、モナドといった関数型プログラミングの要素を取り入れつつ、クリーンで可読性の高い構文を維持することで、堅牢で表現力豊かなプログラミング体験を提供することを目指しています。

**思想と特徴:**

*   **型安全性**: 型推論を備えた強力な静的型システムにより、コンパイル時にエラーを検出します。
*   **不変性**: 変数はデフォルトで不変（イミュータブル）であり、より安全で予測可能なコードを促進します。
*   **表現力**: パターンマッチング、演算子オーバーロード、高階関数などの機能により、簡潔でパワフルなコード記述が可能です。
*   **関数型のコア**: 第一級関数、カリー化、モナド (`Maybe`, `Either`) をサポートし、状態やエラーをエレガントに扱えます。
*   **TypeScriptとの相互運用性**: 人間が読めるTypeScriptに直接コンパイルされるため、広大なJavaScriptエコシステムとシームレスに連携できます。

## 2. 基本的な構文

### 2.1. コメント

単一行コメントは `//` で始まります。

```seseragi
// これは一行コメントです
let x = 10 // これもコメントです
```

### 2.2. 変数束縛 (`let`)

変数は `let` キーワードで宣言され、不変です。変数名は小文字で始める必要がありますが、数字、アンダースコア (`_`)、アポストロフィ (`'`) を含めることができます。

```seseragi
let message = "こんにちは、Seseragi！"
let user_age = 25
let x' = x + 10 // アポストロフィも変数名に使えます
```

### 2.3. 出力 (`print` と `show`)

-   `print`: 変数や式の生の値をそのまま出力します。
-   `show`: 値を整形し、より人間が読みやすい表現で出力します。

```seseragi
let myTuple = (1, "test")
print myTuple // コンパイル後の生の表現で出力
show myTuple // "(1, \"test\")" のように整形して出力
```

## 3. 型システム

### 3.1. 基本型

-   `Int`: 符号付き整数 (例: `42`, `-10`)
-   `Float`: 浮動小数点数 (例: `3.14`, `-0.5`)
-   `String`: 文字列データ (例: `"hello"`)。バッククォートで文字列補間が可能です: `` `こんにちは、${name}さん！` ``
-   `Bool`: 真偽値、`True` または `False`

### 3.2. 型推論と型注釈

Seseragiの型システムはほとんどの型を自動で推論します。しかし、コロン (`:`) を使って明示的に型注釈を記述することもできます。

```seseragi
let num = 42          // Int型だと推論される
let name: String = "Alice" // 明示的な型注釈
```

## 4. 関数

### 4.1. 関数の定義とカリー化

関数は `fn` キーワードで定義します。Seseragiのすべての関数は自動的にカリー化されます。

```seseragi
// fn 関数名 引数1: 型1 -> 引数2: 型2 -> 戻り値型 = 本体
fn add x: Int -> y: Int -> Int = x + y

// 完全適用
let sum = add 5 3 // 8

// 部分適用（カリー化）
let addFive = add 5
let sum2 = addFive 3 // 8
```

### 4.2. ブロック構文

複数行にわたる関数を定義する場合は `{}` を使います。ブロック内の最後の式が戻り値になります。

```seseragi
fn calculate x: Int -> Int {
  let doubled = x * 2
  doubled + 10 // この式が戻り値になる
}
```

### 4.3. 高階関数とラムダ式

関数は第一級市民です。つまり、他の関数に引数として渡したり、戻り値として返したりできます。無名関数（ラムダ式）はバックスラッシュ (`\`) を使って定義します。

```seseragi
// 高階関数
fn applyTwice f: (Int -> Int) -> x: Int -> Int = f(f(x))

// ラムダ式
let double = \x -> x * 2

show $ applyTwice double 5 // 20
```

## 5. 演算子

-   **算術演算子**: `+`, `-`, `*`, `/`, `%` (剰余)
-   **比較演算子**: `==`, `!=`, `>`, `<`, `>=`, `<=`
-   **論理演算子**: `&&` (AND), `||` (OR), `!` (NOT)
-   **関数適用演算子 (`$`)**: 括弧を減らすために使われる低優先度の演算子です。
    `show $ double $ add 5 3`

## 6. 制御フロー

### 6.1. `if-then-else` 式

Seseragiの `if` は式であり、必ず `else` 節が必要です。

```seseragi
let message = if score > 60 then "合格" else "不合格"
```

### 6.2. 三項演算子

`if-then-else` の糖衣構文（シンタックスシュガー）です。

```seseragi
let message = score > 60 ? "合格" : "不合格"
```

### 6.3. パターンマッチング (`match`)

`match` 式は、値の構造を分解し、その形に応じて処理を分岐させる強力な制御フロー機能です。

```seseragi
fn describe value: Maybe<Int> -> String = match value {
  Just n when n > 10 -> "大きな数です"
  Just n -> "小さな数です"
  Nothing -> "値がありません"
}

// タプルのマッチング
match (x, y) {
  (0, 0) -> "原点"
  (_, 0) -> "X軸上"
  (0, _) -> "Y軸上"
  (a, b) -> `点(${a}, ${b})`
}
```

## 7. データ構造

### 7.1. タプル (Tuple)

固定サイズの順序付きコレクションで、異なる型の要素を持つことができます。

```seseragi
let point: (Int, Int) = (10, 20)

// 分割代入
let (x, y) = point
let (name, _) = ("Alice", 25) // `_` で要素を無視
```

### 7.2. レコード (Record)

名前のない構造的な型で、キーと値のペアの集まりです。

```seseragi
let person = { name: "Bob", age: 40 }
let name = person.name // フィールドへのアクセス
```

### 7.3. Array

JavaScriptの配列に対応する、順序付きのミュータブルなコレクションです。

-   **定義**: `[1, 2, 3]`
-   **長さ**: `myArray.length`
-   **安全なアクセス**: `myArray[index]` は `Maybe` 型 (`Just value` または `Nothing`) を返し、範囲外エラーを防ぎます。

### 7.4. List

関数型プログラミングで一般的な、不変（イミュータブル）で再帰的なデータ構造です。

-   **定義**: `` `[1, 2, 3] `` (糖衣構文), `1 : 2 : 3 : \`[]` (cons演算子)
-   **操作**: `head list` (`^list`) と `tail list` (`>>list`)

### 7.5. 内包表記と範囲演算子

ArrayとListの両方で、宣言的に新しいコレクションを生成するための内包表記がサポートされています。

-   **範囲演算子**: `1..5` (5を含まない), `1..=5` (5を含む)

```seseragi
// 配列内包表記
let squares = [x * x | x <- 1..=5] // [1, 4, 9, 16, 25]

// 条件付きリスト内包表記
let evenSquares = `[x * x | x <- 1..=10, x % 2 == 0]
```

## 8. ジェネリクス

ジェネリクスにより、複数の型に対して再利用可能なコードを書くことができます。

### 8.1. ジェネリック関数

関数名の後に `<T>` を付けてジェネリック関数を定義します。

```seseragi
fn identity<T> x: T -> T = x

let num = identity 42       // TはIntと推論される
let str = identity<String> "hello" // 型を明示的に指定
```

### 8.2. ジェネリック型

型エイリアスや構造体もジェネリックにできます。

```seseragi
type Box<T> = { value: T }
struct Node<T> { value: T, next: Maybe<Node<T>> }
```

## 9. ユーザー定義型

### 9.1. 型エイリアス (`type`)

既存の型に新しい名前を付けます。

```seseragi
type UserId = Int
type Point = { x: Float, y: Float }
```

### 9.2. 構造体 (Struct)

関連するデータをまとめるための名前付きの複合型です。

```seseragi
struct Point { x: Float, y: Float }

// インスタンス化
let p1 = Point { x: 1.0, y: 2.0 }

// 更新構文（新しい不変インスタンスを生成）
let p2 = Point { ...p1, y: 3.0 }
```

#### メソッドと演算子オーバーロード (`impl`)

`impl` ブロック内で、構造体にメソッドやオーバーロードされた演算子を実装できます。

```seseragi
impl Point {
  // メソッド
  fn scale self -> factor: Float -> Point = ...

  // 演算子オーバーロード
  operator + self -> other: Point -> Point = ...
}

let p3 = p1.scale 2.0
let p4 = p1 + p2
```

### 9.3. 代数的データ型 (ADT)

複数の異なる表現を取りうる型で、「タグ付き合併型」とも呼ばれます。`type` と `|` で定義します。

```seseragi
// 単純な列挙型
type Color = | Red | Green | Blue

// データを持つADT
type Shape = | Circle Float | Rectangle Float Float
```

### 9.4. 合併型と交差型

-   **合併型 (`|`)**: 複数の型のうちのいずれか一つであることを示します。
-   **交差型 (`&`)**: 複数の型をすべて満たすことを示します。

```seseragi
type StringOrInt = String | Int
type HasIdentity = { id: Int } & { name: String }
```

## 10. モナドとエラーハンドリング

### 10.1. `Maybe<T>`

値が存在しない可能性を表す型です。`Just T` (値が存在) または `Nothing` (値がない) のいずれかを取ります。

### 10.2. `Either<E, T>`

計算が二つの結果のうちどちらかになることを表す型です。`Left E` (通常はエラー) または `Right T` (成功した値) を取ります。

### 10.3. モナド演算子

モナドを扱うための中置演算子により、クリーンで連鎖的な計算が可能になります。

-   **Functor (`<$>`)**: モナド内の値に関数を適用します。
-   **Applicative (`<*>`)**: モナド内の関数をモナド内の値に適用します。
-   **Monad (`>>=`)**: モナドを返す関数を連鎖させます。

```seseragi
// 失敗する可能性のある処理を連鎖させる
let result = Just "10"
  >>= parseInt   // Either<String, Int> を返す
  >>= (\n -> if n > 0 then Right n else Left "正数ではありません")

show result // Right 10
```

## 11. CLIツール

`seseragi` コマンドは、コードのコンパイル、実行、フォーマットを行うためのエントリーポイントです。

-   `seseragi run <file.ssrg>`: Seseragiファイルを実行します。
-   `seseragi <file.ssrg>`: SeseragiファイルをTypeScript (`.ts`) にコンパイルします。
-   `seseragi fmt <file.ssrg>`: Seseragiファイルをフォーマットします。
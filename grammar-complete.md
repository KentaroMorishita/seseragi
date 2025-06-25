# Seseragi言語 完全文法仕様

この文書は、Seseragi言語の最終的な目標仕様を定義します。全ての機能が実装された状態での完全な言語仕様です。

## 目次

1. [言語概要と設計思想](#1-言語概要と設計思想)
2. [字句要素](#2-字句要素)
3. [型システム](#3-型システム)
4. [変数定義](#4-変数定義)
5. [関数定義](#5-関数定義)
6. [演算子システム](#6-演算子システム)
7. [制御構造](#7-制御構造)
8. [データ構造](#8-データ構造)
9. [モナドシステム](#9-モナドシステム)
10. [副作用管理](#10-副作用管理)
11. [型クラスとインスタンス](#11-型クラスとインスタンス)
12. [モジュールシステム](#12-モジュールシステム)
13. [高度な機能](#13-高度な機能)
14. [標準ライブラリ](#14-標準ライブラリ)
15. [エラーハンドリング](#15-エラーハンドリング)
16. [形式的文法定義](#16-形式的文法定義)

---

## 1. 言語概要と設計思想

Seseragiは、TypeScriptにトランスパイルされる純粋関数型プログラミング言語です。以下の設計原則に基づいています：

### 1.1 核となる設計原則
- **純粋関数型プログラミング** - 副作用を明示的に管理
- **静的型付け** - コンパイル時の型安全性を保証
- **不変性** - すべての変数は不変（immutable）
- **カリー化** - すべての関数はデフォルトでカリー化
- **モナド** - 副作用、エラー、オプション値の安全な処理

### 1.2 実行環境
- **ターゲット**: TypeScript/JavaScript
- **ランタイム**: bun（高速JavaScript実行環境）
- **互換性**: Node.js, Deno, ブラウザ

### 1.3 開発哲学
- **型安全性**: ランタイムエラーの最小化
- **関数合成**: 小さな関数を組み合わせる設計
- **予測可能性**: 副作用の明示的管理
- **開発効率**: 強力な型推論とツール支援

---

## 2. 字句要素

### 2.1 リテラル表記

#### 2.1.1 整数リテラル (`Int`)
```seseragi
42          // 10進数
0xFF        // 16進数  
0o755       // 8進数
0b1010      // 2進数
-123        // 負の数
1_000_000   // 区切り文字（読みやすさ向上）
```

#### 2.1.2 浮動小数点リテラル (`Float`)
```seseragi
3.14        // 基本形
2.0         // 整数部分が必要
0.5         // 小数部分のみ
1.23e4      // 指数表記 (12300.0)
1.23e-4     // 負の指数 (0.000123)
-2.5        // 負の浮動小数点数
3.14_159    // 区切り文字
```

#### 2.1.3 文字列リテラル (`String`)
```seseragi
"Hello, World!"        // 基本的な文字列
"Hello\nWorld"         // エスケープシーケンス
"Say \"Hello\""        // ダブルクォートのエスケープ
""                     // 空文字列
"Line 1\nLine 2"       // 複数行
"Unicode: \u{1F600}"   // Unicode文字
```

**エスケープシーケンス:**
- `\n` - 改行
- `\t` - タブ
- `\\` - バックスラッシュ
- `\"` - ダブルクォート
- `\r` - キャリッジリターン
- `\0` - ヌル文字
- `\u{XXXX}` - Unicode文字

#### 2.1.4 文字リテラル (`Char`)
```seseragi
'a'         // 基本的な文字
'Z'         // 大文字
'1'         // 数字文字
'\n'        // エスケープシーケンス
'\''        // シングルクォートのエスケープ
'\u{1F600}' // Unicode文字
```

#### 2.1.5 真偽値リテラル (`Bool`)
```seseragi
True        // 真
False       // 偽
```

#### 2.1.6 Unit リテラル
```seseragi
()          // Unit型の値
```

### 2.2 識別子の命名規則

#### 2.2.1 変数名・関数名
- 小文字で開始
- 英数字とアンダースコア `_` を使用可能
- キャメルケースを推奨

```seseragi
// 有効な識別子
x
count
userName
calculateTotal
isValid
item2
_private
```

#### 2.2.2 型名
- 大文字で開始
- 英数字とアンダースコア `_` を使用可能
- パスカルケースを推奨

```seseragi
// 有効な型名
Int
UserAccount
DatabaseConnection
MyType2
```

### 2.3 予約語と定義済み要素

#### 2.3.1 予約語（言語キーワード）
```
// 定義キーワード
fn type let impl monoid operator

// 制御フローキーワード  
if then else match case return

// 副作用管理キーワード
effectful elevate pure perform

// モジュールキーワード
import as export module

// その他
where otherwise when guard
```

#### 2.3.2 定義済み型・値
**プリミティブ型:**
```
Int Float Bool String Char Unit
```

**コレクション型:**
```
List Array Maybe Either IO Result
```

**定義済み値:**
```
True False Nothing Just Left Right Ok Error
```

### 2.4 コメント記法
Seseragiでは`//`を使用してコメントを記述します。

```seseragi
// これは単行コメントです
let x = 42  // 行末コメント

// 複数行にわたるコメントは
// 各行に // を記述します
// このようにシンプルで一貫性があります

fn calculateArea radius :Float -> Float {
  // 円の面積を計算
  // πr² の公式を使用
  let pi = 3.14159
  pi * radius * radius
}
```

---

## 3. 型システム

### 3.1 基本型

#### 3.1.1 プリミティブ型
- **`Int`** - 整数型（32ビット符号付き）
- **`Float`** - 浮動小数点数型（64ビット倍精度）
- **`Bool`** - 真偽値型（`True` または `False`）
- **`String`** - 文字列型（UTF-8エンコーディング）
- **`Char`** - 単一文字型（Unicode文字）
- **`Unit`** - 戻り値を持たない場合の型

#### 3.1.2 コレクション型
- **`List<T>`** - 任意の型Tの不変リスト
- **`Array<T>`** - 任意の型Tの不変配列
- **`Map<K, V>`** - キーKと値Vの連想配列
- **`Set<T>`** - 任意の型Tの集合

#### 3.1.3 モナド型
- **`Maybe<T>`** - オプション値（`Just T` または `Nothing`）
- **`Either<L, R>`** - エラーと成功の表現（`Left L` または `Right R`）
- **`IO<T>`** - 副作用を持つ型Tを包むモナド
- **`Result<T, E>`** - 結果とエラーの表現（`Ok T` または `Error E`）

### 3.2 型構成子とジェネリクス

#### 3.2.1 ジェネリック型定義
```seseragi
type Container<T> {
  value :T
  metadata :String
}

type Pair<A, B> {
  first :A
  second :B
}

// 制約付きジェネリクス（将来機能）
type Comparable<T: Ord> {
  value :T
}
```

#### 3.2.2 型エイリアス
```seseragi
type UserId = Int
type UserName = String
type Point = Pair<Float, Float>
type StringMap<T> = Map<String, T>
```

### 3.3 代数的データ型

#### 3.3.1 列挙型
```seseragi
type Color = Red | Green | Blue | Custom String

type HttpStatus = 
  | Ok 
  | NotFound 
  | InternalServerError
  | Custom Int String
```

#### 3.3.2 再帰的データ型
```seseragi
type Tree<T> = 
  | Leaf T 
  | Node T (Tree<T>) (Tree<T>)

type List<T> = 
  | Nil 
  | Cons T (List<T>)
```

### 3.4 型推論

#### 3.4.1 Hindley-Milner型推論
```seseragi
// 型注釈なしでも型が推論される
let identity x = x              // <T> T -> T
let compose f g x = f (g x)     // <A,B,C> (B -> C) -> (A -> B) -> A -> C
let map f list = List.map f list // <A,B> (A -> B) -> List<A> -> List<B>
```

#### 3.4.2 型注釈の推奨パターン
```seseragi
// トップレベル関数は型注釈推奨
fn processUser :UserId -> IO<UserInfo>

// 複雑な型は明示的に
fn complexTransform :List<Either<Error, User>> -> Maybe<List<User>>

// ローカル変数は推論に任せる
fn example user :User -> String {
  let name = user.name        // String に推論
  let age = user.age          // Int に推論
  name ++ " (" ++ toString age ++ ")"
}
```

---

## 4. 変数定義

### 4.1 基本的な変数定義

変数は`let`キーワードで定義し、すべて不変です。

```seseragi
// 基本的な定義
let x = 42
let name = "Seseragi"
let flag = True

// 明示的な型注釈
let count :Int = 100
let message :String = "Hello"
let isActive :Bool = False

// 複雑な型
let numbers :List<Int> = [1, 2, 3, 4, 5]
let result :Maybe<String> = Just "success"
let userMap :Map<String, User> = Map.empty
```

### 4.2 パターンマッチングによる束縛

```seseragi
// タプルの分解
let (x, y) = (10, 20)

// リストの分解
let [first, second, ...rest] = [1, 2, 3, 4, 5]

// レコードの分解
let User { name, age } = user

// Maybeの分解（パターンマッチング内で）
match maybeValue {
  Just value -> let processedValue = transform value
  Nothing -> let defaultValue = getDefault()
}
```

### 4.3 スコープと束縛

```seseragi
let global = 42

fn example input :Int -> Int {
  let local = global * 2    // 外側のスコープにアクセス
  let result = local + input
  
  // ローカルスコープ
  if input > 0 then {
    let positive = input * 2
    positive + result
  } else {
    result
  }
}
```

---

## 5. 関数定義

### 5.1 基本的な関数定義パターン

#### 5.1.1 ワンライナー形式（推奨）
```seseragi
fn add x :Int -> y :Int -> Int = x + y
fn double n :Int -> Int = n * 2
fn greet name :String -> String = "Hello, " ++ name
```

#### 5.1.2 ブロック形式
```seseragi
fn processData input :String -> String {
  let cleaned = String.trim input
  let processed = String.toUpper cleaned
  "Processed: " ++ processed
}

fn calculateSum numbers :List<Int> -> Int {
  let filtered = List.filter (\x -> x > 0) numbers
  List.fold (+) 0 filtered
}
```

#### 5.1.3 明示的return
```seseragi
fn findMax list :List<Int> -> Maybe<Int> {
  if List.isEmpty list then {
    return Nothing
  } else {
    return Just (List.maximum list)
  }
}
```

#### 5.1.4 シグネチャ分離形式
```seseragi
// 型シグネチャを事前に宣言
fn fibonacci :Int -> Int
fn factorial :Int -> Int

// 実装を後で定義
fn fibonacci n = if n <= 1 then n else fibonacci (n - 1) + fibonacci (n - 2)
fn factorial n = if n <= 1 then 1 else n * factorial (n - 1)
```

### 5.2 高階関数と関数型

#### 5.2.1 関数を引数とする関数
```seseragi
fn applyTwice f :(Int -> Int) -> x :Int -> Int = f (f x)
fn map f :(a -> b) -> list :List<a> -> List<b> = List.map f list
fn filter predicate :(a -> Bool) -> list :List<a> -> List<a> = List.filter predicate list
```

#### 5.2.2 関数を返す関数
```seseragi
fn makeAdder n :Int -> (Int -> Int) = \x -> x + n
fn compose f :(b -> c) -> g :(a -> b) -> (a -> c) = \x -> f (g x)
```

#### 5.2.3 ラムダ式
```seseragi
// 短いラムダ式
let double = \x -> x * 2
let add = \x y -> x + y

// 複雑なラムダ式
let processUser = \user -> {
  let name = user.name
  let age = user.age
  User { name: String.toUpper name, age: age + 1 }
}
```

### 5.3 特殊な関数定義ケース

#### 5.3.1 パターンマッチング関数
```seseragi
// 複数のパターンで同じ関数を定義
fn length :List<a> -> Int
fn length [] = 0
fn length (_ :: xs) = 1 + length xs

// Maybe型でのパターンマッチング
fn fromMaybe :a -> Maybe<a> -> a
fn fromMaybe default Nothing = default
fn fromMaybe _ (Just value) = value
```

#### 5.3.2 ガード付き関数
```seseragi
fn fibonacci :Int -> Int
fn fibonacci n 
  | n <= 0 = 0
  | n == 1 = 1
  | otherwise = fibonacci (n - 1) + fibonacci (n - 2)

fn classify :Int -> String
fn classify x
  | x < 0 = "negative"
  | x == 0 = "zero"
  | x > 100 = "large"
  | otherwise = "positive"
```

#### 5.3.3 相互再帰関数
```seseragi
// シグネチャを事前に宣言（必須）
fn isEven :Int -> Bool
fn isOdd :Int -> Bool

// 相互再帰の実装
fn isEven n = if n == 0 then True else isOdd (n - 1)
fn isOdd n = if n == 0 then False else isEven (n - 1)
```

### 5.4 カリー化と部分適用

```seseragi
// すべての関数は自動的にカリー化される
fn add x :Int -> y :Int -> z :Int -> Int = x + y + z

// 部分適用
let addTen = add 10        // Int -> Int -> Int
let addTenFive = add 10 5  // Int -> Int
let result = addTenFive 3  // 18

// パイプラインでの活用
let numbers = [1, 2, 3, 4, 5]
let doubled = numbers | List.map ((*) 2)
let filtered = doubled | List.filter ((>) 5)
```

---

## 6. 演算子システム

### 6.1 演算子の優先順位と結合性

演算子の優先順位（高い順）：

| 優先順位 | 演算子 | 結合性 | 説明 |
|----------|--------|--------|------|
| 10 | `()`, `[]`, `.` | 左結合 | 関数呼び出し、配列アクセス、フィールドアクセス |
| 9 | `!`, `-` (単項) | 右結合 | 論理否定、単項マイナス |
| 8 | `**` | 右結合 | べき乗 |
| 7 | `*`, `/`, `%` | 左結合 | 乗除算、余り |
| 6 | `+`, `-` | 左結合 | 加減算 |
| 5 | `++` | 右結合 | 文字列連結、リスト連結 |
| 4 | `==`, `!=`, `<`, `<=`, `>`, `>=` | 左結合 | 比較演算子 |
| 3 | `&&` | 右結合 | 論理積 |
| 2 | `\|\|` | 右結合 | 論理和 |
| 1 | `\|`, `~`, `<$>`, `<*>`, `>>=`, `>>>` | 左結合 | 関数型演算子 |
| 0 | `$` | 右結合 | 関数適用（最低優先順位） |

### 6.2 算術演算子

```seseragi
// 基本算術演算
let sum = 5 + 3        // 8
let diff = 10 - 4      // 6
let product = 6 * 7    // 42
let quotient = 15 / 3  // 5
let remainder = 17 % 5 // 2
let power = 2 ** 3     // 8

// 浮動小数点演算
let floatSum = 3.14 + 2.86     // 6.0
let floatDiv = 22.0 / 7.0      // 3.142857...
```

### 6.3 比較演算子

```seseragi
// 等価比較
let equal = 5 == 5        // True
let notEqual = 5 != 3     // True

// 大小比較
let less = 3 < 5          // True
let lessEqual = 5 <= 5    // True
let greater = 5 > 3       // True
let greaterEqual = 5 >= 5 // True

// 文字列比較
let strEqual = "hello" == "hello"  // True
let strLess = "apple" < "banana"   // True (辞書順)
```

### 6.4 論理演算子

```seseragi
// 論理演算
let and = True && False   // False
let or = True || False    // True
let not = !True           // False

// 短絡評価
let result = condition && expensiveFunction()
let fallback = maybeValue || defaultValue
```

### 6.5 文字列・リスト演算子

```seseragi
// 文字列連結
let greeting = "Hello" ++ " " ++ "World"  // "Hello World"

// リスト連結
let combined = [1, 2] ++ [3, 4]          // [1, 2, 3, 4]

// リスト構築
let list = 1 :: [2, 3, 4]                // [1, 2, 3, 4]
```

### 6.6 関数型演算子

#### 6.6.1 パイプライン演算子 (`|`)
```seseragi
// 左から右への関数適用
let result = 2 
  | add 3 
  | multiply 4 
  | toString         // "20"

// 複雑なデータ処理
let processed = users
  | List.filter (\u -> u.age >= 18)
  | List.map (\u -> u.name)
  | List.sort
```

#### 6.6.2 逆パイプ演算子 (`~`)
```seseragi
// 右から左の部分適用
fn add3 x :Int -> y :Int -> z :Int -> Int = x + y + z

let result = 10 ~ add3 20 30  // add3 10 20 30 = 60
```

#### 6.6.3 関数適用演算子 (`$`)
```seseragi
// 括弧を減らす
print $ toString $ add 10 5     // print (toString (add 10 5))
let result = map (\x -> x * 2) $ filter (\x -> x > 0) $ [1, -2, 3, -4]
```

#### 6.6.4 ファンクターマップ演算子 (`<$>`)
```seseragi
// ファンクター内の値に関数を適用
let doubled = (\x -> x * 2) <$> Just 5    // Just 10
let mapped = (\x -> x ++ "!") <$> ["hello", "world"]  // ["hello!", "world!"]
```

#### 6.6.5 アプリカティブ演算子 (`<*>`)
```seseragi
// アプリカティブファンクター
fn add x :Int -> y :Int -> Int = x + y

let result = Just add <*> Just 5 <*> Just 3  // Just 8
let combined = [add] <*> [1, 2] <*> [10, 20] // [11, 21, 12, 22]
```

#### 6.6.6 モナドバインド演算子 (`>>=`)
```seseragi
// モナドチェーン
fn safeDivide x :Int -> y :Int -> Maybe<Int> = 
  if y == 0 then Nothing else Just (x / y)

let result = Just 20 
  >>= safeDivide 10 
  >>= safeDivide 2   // Just 1
```

#### 6.6.7 モノイド畳み込み演算子 (`>>>`)
```seseragi
// モノイドの畳み込み
let sum = [1, 2, 3, 4, 5] >>> (+)        // 15
let product = [1, 2, 3, 4, 5] >>> (*)    // 120
let concat = ["hello", " ", "world"] >>> (++)  // "hello world"
```

### 6.7 カスタム演算子

```seseragi
// カスタム中置演算子の定義
operator (|>) left right = right left       // パイプライン演算子の別名
operator (<|) left right = left right       // 逆パイプライン演算子

// 使用例
let result = 5 |> double |> toString         // "10"
let result2 = toString <| double <| 5        // "10"
```

---

## 7. 制御構造

### 7.1 条件分岐

#### 7.1.1 if-then-else式
```seseragi
// 基本的な条件分岐
let result = if x > 0 then "positive" else "non-positive"

// 複雑な条件
let status = if age >= 18 && hasLicense then "can drive" else "cannot drive"

// ネストした条件
let grade = if score >= 90 then "A"
           else if score >= 80 then "B"
           else if score >= 70 then "C"
           else "F"

// ブロック形式
let result = if condition then {
  let temp = calculate something
  temp * 2
} else {
  defaultValue
}
```

#### 7.1.2 when式（簡潔な条件分岐）
```seseragi
// when式による簡潔な記述
let message = when {
  age < 13 -> "child"
  age < 20 -> "teenager"  
  age < 65 -> "adult"
  otherwise -> "senior"
}
```

### 7.2 パターンマッチング

#### 7.2.1 基本的なmatch式
```seseragi
match value {
  pattern1 -> expression1
  pattern2 -> expression2
  _ -> defaultExpression
}
```

#### 7.2.2 Maybe型のパターンマッチング
```seseragi
match maybeValue {
  Just x -> "Found: " ++ toString x
  Nothing -> "Not found"
}

// ネストしたパターン
match nestedMaybe {
  Just (Just x) -> "Nested value: " ++ toString x
  Just Nothing -> "Outer just, inner nothing"
  Nothing -> "Outer nothing"
}
```

#### 7.2.3 Either型のパターンマッチング
```seseragi
match result {
  Left error -> "Error: " ++ error
  Right value -> "Success: " ++ toString value
}
```

#### 7.2.4 リストのパターンマッチング
```seseragi
match list {
  [] -> "Empty list"
  [x] -> "Single element: " ++ toString x
  x :: xs -> "Head: " ++ toString x ++ ", tail length: " ++ toString (length xs)
  [x, y] -> "Two elements: " ++ toString x ++ ", " ++ toString y
  [x, y, ...rest] -> "First two: " ++ toString x ++ ", " ++ toString y
}
```

#### 7.2.5 カスタム型のパターンマッチング
```seseragi
type Shape = Circle Float | Rectangle Float Float | Triangle Float Float Float

fn area shape :Shape -> Float {
  match shape {
    Circle radius -> pi * radius * radius
    Rectangle width height -> width * height
    Triangle a b c -> {
      let s = (a + b + c) / 2.0
      sqrt (s * (s - a) * (s - b) * (s - c))
    }
  }
}
```

#### 7.2.6 ガード条件付きパターン
```seseragi
match number {
  x when x > 100 -> "very large"
  x when x > 50 -> "large"
  x when x > 0 -> "positive"
  _ -> "non-positive"
}

// 複雑なガード条件
match user {
  User { age, isActive: True } when age >= 18 -> "Active adult"
  User { age } when age < 18 -> "Minor"
  _ -> "Inactive user"
}
```

### 7.3 反復処理

Seseragiは純粋関数型言語のため、従来のループは提供せず、関数的な反復処理を推奨します。

#### 7.3.1 再帰による反復
```seseragi
fn factorial n :Int -> Int {
  if n <= 1 then 1 else n * factorial (n - 1)
}

fn fibonacci n :Int -> Int = match n {
  0 -> 0
  1 -> 1
  n -> fibonacci (n - 1) + fibonacci (n - 2)
}
```

#### 7.3.2 高階関数による反復
```seseragi
// map, filter, fold等を使用
let doubled = List.map (\x -> x * 2) [1, 2, 3, 4, 5]
let evens = List.filter (\x -> x % 2 == 0) [1, 2, 3, 4, 5]
let sum = List.fold (+) 0 [1, 2, 3, 4, 5]

// unfold による生成
let naturals = List.unfold (\n -> Just (n, n + 1)) 1  // [1, 2, 3, ...]
```

---

## 8. データ構造

### 8.1 レコード型

#### 8.1.1 基本的なレコード定義
```seseragi
type Person {
  name :String
  age :Int
  email :String
}

type Point {
  x :Float
  y :Float
}

type User {
  id :Int
  person :Person
  isActive :Bool
  joinedAt :String
}
```

#### 8.1.2 レコードの作成とアクセス
```seseragi
// レコード作成
let person = Person { 
  name: "Alice", 
  age: 30, 
  email: "alice@example.com" 
}

// フィールドアクセス
let name = person.name
let age = person.age

// ネストしたアクセス
let user = User { 
  id: 1, 
  person: person, 
  isActive: True, 
  joinedAt: "2024-01-01" 
}
let userName = user.person.name
```

#### 8.1.3 レコード更新
```seseragi
// 単一フィールド更新
let olderPerson = person { age: person.age + 1 }

// 複数フィールド更新
let updatedPerson = person { 
  name: "Alice Smith", 
  age: person.age + 1 
}

// ネストした更新
let updatedUser = user { 
  person: user.person { name: "Alice Johnson" },
  isActive: False 
}
```

### 8.2 代数的データ型

#### 8.2.1 列挙型
```seseragi
type Color = Red | Green | Blue | Yellow

type Status = Active | Inactive | Pending | Suspended

type HttpMethod = GET | POST | PUT | DELETE | PATCH
```

#### 8.2.2 値を持つバリアント
```seseragi
type Shape = 
  | Circle Float                    // 半径
  | Rectangle Float Float           // 幅、高さ
  | Triangle Float Float Float      // 三辺の長さ

type Result<T, E> = 
  | Ok T 
  | Error E

type Option<T> = 
  | Some T 
  | None
```

#### 8.2.3 再帰的データ型
```seseragi
type Tree<T> = 
  | Leaf T 
  | Node T (Tree<T>) (Tree<T>)

type List<T> = 
  | Nil 
  | Cons T (List<T>)

type Json = 
  | JNull
  | JBool Bool
  | JNumber Float
  | JString String
  | JArray (List<Json>)
  | JObject (Map<String, Json>)
```

### 8.3 タプル型

```seseragi
// タプル型の定義
type Pair<A, B> = (A, B)
type Triple<A, B, C> = (A, B, C)

// タプルの作成
let point = (3.14, 2.71)
let person = ("Alice", 30, True)

// タプルの分解
let (x, y) = point
let (name, age, isActive) = person

// タプルへのアクセス
let firstName = person.0   // "Alice"
let userAge = person.1     // 30
```

### 8.4 コレクション型

#### 8.4.1 リスト
```seseragi
// リストの作成
let numbers = [1, 2, 3, 4, 5]
let names = ["Alice", "Bob", "Charlie"]
let empty = []

// リスト操作
let head = List.head numbers        // Just 1
let tail = List.tail numbers        // Just [2, 3, 4, 5]
let length = List.length numbers    // 5
let reversed = List.reverse numbers // [5, 4, 3, 2, 1]

// リスト構築
let newList = 0 :: numbers          // [0, 1, 2, 3, 4, 5]
let combined = [1, 2] ++ [3, 4]     // [1, 2, 3, 4]
```

#### 8.4.2 配列
```seseragi
// 配列の作成
let arr = Array.fromList [1, 2, 3, 4, 5]
let zeros = Array.make 10 0         // 10個の0で埋められた配列

// 配列操作
let element = Array.get 2 arr       // Just 3
let updated = Array.set 2 42 arr    // 新しい配列を返す
let length = Array.length arr       // 5
```

#### 8.4.3 Map（連想配列）
```seseragi
// Mapの作成
let userMap = Map.fromList [
  ("alice", User { name: "Alice", age: 30 }),
  ("bob", User { name: "Bob", age: 25 })
]

// Map操作
let alice = Map.get "alice" userMap           // Just User
let withCharlie = Map.set "charlie" charlie userMap
let keys = Map.keys userMap                   // ["alice", "bob"]
let values = Map.values userMap               // [User, User]
```

#### 8.4.4 Set（集合）
```seseragi
// Setの作成
let numberSet = Set.fromList [1, 2, 3, 2, 1]  // Set {1, 2, 3}

// Set操作
let hasMember = Set.member 2 numberSet        // True
let added = Set.add 4 numberSet               // Set {1, 2, 3, 4}
let removed = Set.remove 2 numberSet          // Set {1, 3}
let size = Set.size numberSet                 // 3

// 集合演算
let union = Set.union set1 set2
let intersection = Set.intersection set1 set2
let difference = Set.difference set1 set2
```

---

## 9. モナドシステム

### 9.1 Maybe モナド

#### 9.1.1 基本的な使用
```seseragi
// Maybe値の作成
let someValue = Just 42
let nothingValue = Nothing

// 安全な演算
fn safeDivide x :Int -> y :Int -> Maybe<Int> = 
  if y == 0 then Nothing else Just (x / y)

let result1 = safeDivide 10 2  // Just 5
let result2 = safeDivide 10 0  // Nothing
```

#### 9.1.2 Functor操作
```seseragi
// map操作（<$>）
let doubled = (\x -> x * 2) <$> Just 5    // Just 10
let nothing = (\x -> x * 2) <$> Nothing   // Nothing

// 連続したmap
let result = Just 5 
  <$> (\x -> x * 2) 
  <$> (\x -> x + 1)    // Just 11
```

#### 9.1.3 Monad操作
```seseragi
// bind操作（>>=）
let chained = Just 10 
  >>= (\x -> Just (x * 2))
  >>= (\x -> if x > 15 then Just x else Nothing)  // Just 20

// do記法（将来機能）
let computation = do {
  x <- Just 10
  y <- safeDivide 20 x
  z <- safeDivide y 2
  return (z + 1)
}
```

### 9.2 Either モナド

#### 9.2.1 エラーハンドリング
```seseragi
type ValidationError = 
  | EmptyName 
  | InvalidAge 
  | InvalidEmail

fn validateUser name :String -> age :Int -> email :String -> Either<ValidationError, User> {
  if String.isEmpty name then Left EmptyName
  else if age < 0 || age > 150 then Left InvalidAge
  else if not (String.contains "@" email) then Left InvalidEmail
  else Right (User { name: name, age: age, email: email })
}
```

#### 9.2.2 エラーの連鎖処理
```seseragi
let processUser userData = 
  parseUserData userData
  >>= validateUser
  >>= saveToDatabase
  >>= sendWelcomeEmail

// 失敗時は最初のエラーで停止
match processUser rawData {
  Left error -> handleError error
  Right user -> showSuccess user
}
```

### 9.3 IO モナド

#### 9.3.1 副作用の管理
```seseragi
// IOアクションの定義
effectful fn readFile path :String -> IO<String>
effectful fn writeFile path :String -> content :String -> IO<Unit>
effectful fn println message :String -> IO<Unit>

// IOアクションの合成
fn copyFile source :String -> dest :String -> IO<Unit> = do {
  content <- readFile source
  writeFile dest content
  println ("Copied " ++ source ++ " to " ++ dest)
}
```

#### 9.3.2 純粋関数内でのIO
```seseragi
fn processData data :String -> IO<String> {
  elevate (println "Processing data...") >>=
  (\_ -> IO.pure (String.toUpper data)) >>=
  (\result -> elevate (println "Processing complete") >>=
  (\_ -> IO.pure result))
}
```

### 9.4 カスタムモナド

#### 9.4.1 State モナド
```seseragi
type State<S, A> = S -> (A, S)

impl State<S, A> {
  fn run state :S -> self :State<S, A> -> (A, S) = self state
  
  fn get :State<S, S> = \s -> (s, s)
  
  fn put newState :S -> State<S, Unit> = \_ -> ((), newState)
  
  fn modify f :(S -> S) -> State<S, Unit> = \s -> ((), f s)
}

// Monadインスタンス
impl State<S, A> {
  fn pure value :A -> State<S, A> = \s -> (value, s)
  
  fn bind f :(A -> State<S, B>) -> self :State<S, A> -> State<S, B> = 
    \s -> let (a, s') = self s in f a s'
}
```

#### 9.4.2 Reader モナド
```seseragi
type Reader<R, A> = R -> A

impl Reader<R, A> {
  fn run env :R -> self :Reader<R, A> -> A = self env
  
  fn ask :Reader<R, R> = \r -> r
  
  fn asks f :(R -> A) -> Reader<R, A> = \r -> f r
}
```

---

## 10. 副作用管理

### 10.1 効果システム

#### 10.1.1 effectful関数の定義
```seseragi
// 副作用を持つ関数は明示的にマーク
effectful fn printMessage message :String -> IO<Unit> {
  IO.perform (systemPrint message)
}

effectful fn readUserInput prompt :String -> IO<String> {
  IO.perform (systemPrompt prompt)
}

effectful fn getCurrentTime -> IO<String> {
  IO.perform systemCurrentTime
}
```

#### 10.1.2 純粋関数と効果的関数の分離
```seseragi
// 純粋関数 - 副作用なし、常に同じ入力で同じ出力
fn calculateTax amount :Float -> rate :Float -> Float = amount * rate

// 効果的関数 - 副作用あり
effectful fn saveTaxRecord amount :Float -> tax :Float -> IO<Unit> {
  let record = TaxRecord { amount: amount, tax: tax, timestamp: getCurrentTime() }
  writeToDatabase record
}

// 効果の合成
effectful fn processPayment amount :Float -> IO<Unit> = do {
  tax <- calculateTax amount 0.08  // 純粋関数の呼び出し
  saveTaxRecord amount tax         // 効果的関数の呼び出し
  printMessage ("Payment processed: " ++ toString amount)
}
```

### 10.2 elevate による効果の持ち上げ

#### 10.2.1 純粋関数内での効果の使用
```seseragi
fn processUserData data :UserData -> IO<ProcessedData> {
  // 純粋な処理
  let validated = validateData data
  let normalized = normalizeData validated
  
  // 効果を必要とする処理
  elevate (logInfo "Processing user data") >>=
  (\_ -> 
    let processed = complexProcessing normalized
    elevate (logInfo "Processing complete") >>=
    (\_ -> IO.pure processed)
  )
}
```

#### 10.2.2 効果の制御フロー
```seseragi
fn safeFileOperation filename :String -> IO<Either<String, String>> = do {
  fileExists <- elevate (checkFileExists filename)
  if fileExists then {
    content <- elevate (readFile filename)
    return (Right content)
  } else {
    return (Left ("File not found: " ++ filename))
  }
}
```

### 10.3 IOモナドの詳細実装

#### 10.3.1 基本的なIO操作
```seseragi
// ファイルシステム操作
effectful fn readTextFile path :String -> IO<String>
effectful fn writeTextFile path :String -> content :String -> IO<Unit>
effectful fn appendToFile path :String -> content :String -> IO<Unit>
effectful fn deleteFile path :String -> IO<Unit>
effectful fn createDirectory path :String -> IO<Unit>

// ネットワーク操作
effectful fn httpGet url :String -> IO<String>
effectful fn httpPost url :String -> body :String -> IO<String>

// システム操作
effectful fn getEnvironmentVariable name :String -> IO<Maybe<String>>
effectful fn getCurrentDirectory -> IO<String>
effectful fn executeCommand command :String -> IO<String>
```

#### 10.3.2 エラーハンドリングを伴うIO
```seseragi
type IOError = 
  | FileNotFound String 
  | PermissionDenied String 
  | NetworkError String 
  | UnknownError String

effectful fn safeReadFile path :String -> IO<Either<IOError, String>> = do {
  try {
    content <- readTextFile path
    return (Right content)
  } catch {
    FileNotFoundError msg -> return (Left (FileNotFound msg))
    PermissionError msg -> return (Left (PermissionDenied msg))
    error -> return (Left (UnknownError (toString error)))
  }
}
```

### 10.4 並行性と非同期処理

#### 10.4.1 非同期IO操作
```seseragi
// 非同期操作の定義
effectful fn asyncReadFile path :String -> IO<Promise<String>>
effectful fn asyncHttpGet url :String -> IO<Promise<String>>

// 並行実行
effectful fn fetchMultipleUrls urls :List<String> -> IO<List<String>> = do {
  promises <- List.map asyncHttpGet urls | sequence
  results <- Promise.all promises
  return results
}
```

#### 10.4.2 リソース管理
```seseragi
// リソースの安全な管理
effectful fn withFile path :String -> action :(FileHandle -> IO<A>) -> IO<A> = do {
  handle <- openFile path
  try {
    result <- action handle
    return result
  } finally {
    closeFile handle
  }
}

// 使用例
effectful fn processLargeFile path :String -> IO<Int> = 
  withFile path (\handle -> do {
    content <- readFromHandle handle
    let lineCount = countLines content
    return lineCount
  })
```

---

## 11. 型クラスとインスタンス

### 11.1 impl ブロックによるメソッド実装

#### 11.1.1 基本的なメソッド定義
```seseragi
type Person {
  name :String
  age :Int
}

impl Person {
  fn greet self -> String = "Hello, I'm " ++ self.name
  
  fn isAdult self -> Bool = self.age >= 18
  
  fn haveBirthday self -> Person = 
    self { age: self.age + 1 }
    
  fn introduce self -> other :Person -> String = 
    self.greet() ++ ". Nice to meet you, " ++ other.name
}

// 使用例
let alice = Person { name: "Alice", age: 25 }
let greeting = alice.greet()           // "Hello, I'm Alice"
let adult = alice.isAdult()            // True
let older = alice.haveBirthday()       // Person { name: "Alice", age: 26 }
```

#### 11.1.2 静的メソッド
```seseragi
impl Person {
  // コンストラクタ的な静的メソッド
  fn create name :String -> age :Int -> Person = 
    Person { name: name, age: age }
    
  fn createChild name :String -> Person = 
    Person.create name 0
    
  fn createAdult name :String -> Person = 
    Person.create name 18
}

// 使用例
let bob = Person.create "Bob" 30
let child = Person.createChild "Charlie"
```

### 11.2 型クラス（trait的機能）

#### 11.2.1 Show型クラス
```seseragi
// Show型クラスの定義（標準ライブラリ）
trait Show<T> {
  fn show :T -> String
}

// Person型にShowを実装
impl Show<Person> {
  fn show person :Person -> String = 
    "Person { name: \"" ++ person.name ++ "\", age: " ++ toString person.age ++ " }"
}

// 使用例
let alice = Person { name: "Alice", age: 25 }
let displayed = show alice  // "Person { name: \"Alice\", age: 25 }"
```

#### 11.2.2 Eq型クラス
```seseragi
trait Eq<T> {
  fn equal :T -> T -> Bool
  fn notEqual :T -> T -> Bool = \x y -> not (equal x y)
}

impl Eq<Person> {
  fn equal p1 :Person -> p2 :Person -> Bool = 
    p1.name == p2.name && p1.age == p2.age
}

// 演算子のオーバーロード
operator == <T: Eq<T>> (left :T, right :T) -> Bool = equal left right
operator != <T: Eq<T>> (left :T, right :T) -> Bool = notEqual left right
```

#### 11.2.3 Ord型クラス
```seseragi
type Ordering = LT | EQ | GT

trait Ord<T> extends Eq<T> {
  fn compare :T -> T -> Ordering
  fn lessThan :T -> T -> Bool = \x y -> compare x y == LT
  fn greaterThan :T -> T -> Bool = \x y -> compare x y == GT
  fn lessEqual :T -> T -> Bool = \x y -> compare x y != GT
  fn greaterEqual :T -> T -> Bool = \x y -> compare x y != LT
}

impl Ord<Person> {
  fn compare p1 :Person -> p2 :Person -> Ordering = 
    let nameComparison = compare p1.name p2.name
    if nameComparison == EQ 
    then compare p1.age p2.age 
    else nameComparison
}
```

### 11.3 モノイド

#### 11.3.1 Monoid型クラス
```seseragi
trait Monoid<T> {
  fn empty :T
  fn append :T -> T -> T
}

// Stringのモノイドインスタンス
impl Monoid<String> {
  fn empty = ""
  fn append s1 :String -> s2 :String -> String = s1 ++ s2
}

// List<T>のモノイドインスタンス
impl Monoid<List<T>> {
  fn empty = []
  fn append l1 :List<T> -> l2 :List<T> -> List<T> = l1 ++ l2
}
```

#### 11.3.2 モノイド操作
```seseragi
// モノイド畳み込み演算子（>>>）
let concatenated = ["hello", " ", "world", "!"] >>> append  // "hello world!"
let combined = [[1, 2], [3, 4], [5, 6]] >>> append         // [1, 2, 3, 4, 5, 6]

// mconcat関数
fn mconcat<T: Monoid<T>> :List<T> -> T = List.fold append empty

let result = mconcat ["a", "b", "c"]  // "abc"
```

### 11.4 カスタム演算子

#### 11.4.1 演算子の定義
```seseragi
// カスタム型での演算子定義
type Vector {
  x :Float
  y :Float
}

impl Vector {
  // ベクトル加算
  operator + (v1 :Vector, v2 :Vector) -> Vector = 
    Vector { x: v1.x + v2.x, y: v1.y + v2.y }
    
  // ベクトル減算
  operator - (v1 :Vector, v2 :Vector) -> Vector = 
    Vector { x: v1.x - v2.x, y: v1.y - v2.y }
    
  // スカラー倍
  operator * (scalar :Float, v :Vector) -> Vector = 
    Vector { x: scalar * v.x, y: scalar * v.y }
    
  // 内積
  operator · (v1 :Vector, v2 :Vector) -> Float = 
    v1.x * v2.x + v1.y * v2.y
}

// 使用例
let v1 = Vector { x: 1.0, y: 2.0 }
let v2 = Vector { x: 3.0, y: 4.0 }
let sum = v1 + v2              // Vector { x: 4.0, y: 6.0 }
let scaled = 2.0 * v1          // Vector { x: 2.0, y: 4.0 }
let dotProduct = v1 · v2       // 11.0
```

#### 11.4.2 演算子の優先順位指定
```seseragi
// 優先順位の明示的指定
operator + precedence 6 left      // 加算
operator * precedence 7 left      // 乗算
operator · precedence 8 left      // 内積（より高い優先順位）
operator ** precedence 9 right    // べき乗（右結合）
```

---

## 12. モジュールシステム

### 12.1 モジュールの定義とエクスポート

#### 12.1.1 基本的なモジュール構造
```seseragi
// ファイル: Math/Vector.ssrg
module Math.Vector

// 公開する型
export type Vector {
  x :Float
  y :Float
}

// 公開する関数
export fn create x :Float -> y :Float -> Vector = Vector { x: x, y: y }
export fn magnitude v :Vector -> Float = sqrt (v.x * v.x + v.y * v.y)
export fn normalize v :Vector -> Vector = {
  let mag = magnitude v
  Vector { x: v.x / mag, y: v.y / mag }
}

// プライベート関数（exportしない）
fn helper x :Float -> Float = x * 2.0
```

#### 12.1.2 選択的エクスポート
```seseragi
module Utils.String

// 型のエクスポート
export type TextProcessingOptions {
  trimWhitespace :Bool
  toLowerCase :Bool
  removeSpecialChars :Bool
}

// 関数の選択的エクスポート
export {
  processText,
  splitWords,
  joinWords
}

fn processText options :TextProcessingOptions -> text :String -> String = 
  text 
  | (if options.trimWhitespace then String.trim else identity)
  | (if options.toLowerCase then String.toLower else identity)
  | (if options.removeSpecialChars then removeSpecialChars else identity)

// プライベート関数
fn removeSpecialChars text :String -> String = 
  String.filter Char.isAlphaNumeric text
```

### 12.2 モジュールのインポート

#### 12.2.1 基本的なインポート
```seseragi
// 特定の要素をインポート
import Math.Vector::{Vector, create, magnitude}

// モジュール全体をインポート
import Math.Vector as Vec

// 使用例
let v = create 3.0 4.0        // Vector { x: 3.0, y: 4.0 }
let mag = magnitude v         // 5.0
let v2 = Vec.normalize v      // 正規化されたベクトル
```

#### 12.2.2 エイリアスを使ったインポート
```seseragi
import Http.Client::{get, post, HttpError}
import Database.User as UserDB
import Database.Product as ProductDB
import Utils.Logger::{logInfo, logError, LogLevel}

fn fetchAndSaveUser userId :String -> IO<Either<String, Unit>> = do {
  userResult <- get ("https://api.example.com/users/" ++ userId)
  match userResult {
    Left (HttpError msg) -> {
      logError ("Failed to fetch user: " ++ msg)
      return (Left msg)
    }
    Right userData -> {
      user <- parseUser userData
      UserDB.save user
      logInfo ("User saved: " ++ userId)
      return (Right ())
    }
  }
}
```

#### 12.2.3 条件付きインポート
```seseragi
// 開発環境でのみインポート
import when (BuildConfig.isDevelopment) {
  Debug.Logger::{debugLog, trace}
}

// プラットフォーム固有のインポート
import when (Platform.isNode) {
  Node.FileSystem as FS
} otherwise {
  Browser.LocalStorage as Storage
}
```

### 12.3 モジュールの階層構造

#### 12.3.1 パッケージとネームスペース
```
project/
├── src/
│   ├── Core/
│   │   ├── Types.ssrg          // Core.Types
│   │   ├── Functions.ssrg      // Core.Functions
│   │   └── Monads.ssrg         // Core.Monads
│   ├── Data/
│   │   ├── List.ssrg           // Data.List
│   │   ├── Map.ssrg            // Data.Map
│   │   └── Set.ssrg            // Data.Set
│   ├── Http/
│   │   ├── Client.ssrg         // Http.Client
│   │   ├── Server.ssrg         // Http.Server
│   │   └── Types.ssrg          // Http.Types
│   └── Main.ssrg               // Main
└── package.ssrg
```

#### 12.3.2 再エクスポート
```seseragi
// ファイル: Core.ssrg - パッケージのメインエクスポート
module Core

// 子モジュールを再エクスポート
export import Core.Types::{*}
export import Core.Functions::{map, filter, fold}
export import Core.Monads::{Maybe, Either, IO}

// 便利な関数を定義してエクスポート
export fn pipeline<A, B, C> f :(A -> B) -> g :(B -> C) -> (A -> C) = 
  \x -> g (f x)
```

### 12.4 循環依存の解決

#### 12.4.1 前方宣言
```seseragi
// ファイル: AST.ssrg
module AST

// 型の前方宣言
declare type Expression
declare type Statement

// 相互参照する型の定義
export type Expression = 
  | Literal Int
  | Variable String
  | Block (List<Statement>)

export type Statement = 
  | Expression Expression
  | Assignment String Expression
```

#### 12.4.2 インターフェース分離
```seseragi
// ファイル: Interfaces/Renderable.ssrg
module Interfaces.Renderable

export trait Renderable<T> {
  fn render :T -> String
}

// ファイル: UI/Component.ssrg
module UI.Component

import Interfaces.Renderable::{Renderable}

export type Component {
  name :String
  children :List<Component>
}

impl Renderable<Component> {
  fn render component :Component -> String = 
    "<" ++ component.name ++ ">" ++ 
    (component.children | List.map render | String.join "") ++
    "</" ++ component.name ++ ">"
}
```

---

## 13. 高度な機能

### 13.1 型推論と型注釈

#### 13.1.1 Hindley-Milner型推論
```seseragi
// 型推論により自動的に型が決定される
let identity x = x                    // <T> T -> T
let compose f g x = f (g x)          // <A,B,C> (B -> C) -> (A -> B) -> A -> C
let flip f x y = f y x               // <A,B,C> (A -> B -> C) -> B -> A -> C

// 部分適用による型の具体化
let addOne = (+) 1                   // Int -> Int
let double = (*) 2                   // Int -> Int
let stringLength = String.length     // String -> Int
```

#### 13.1.2 型注釈による制約
```seseragi
// 明示的な型注釈で型を制約
fn processNumbers :List<Int> -> Int = 
  List.fold (+) 0  // Intに制約されるため(+)はInt -> Int -> Int

// ジェネリック型の制約
fn sortBy<T> compareFn :(T -> T -> Ordering) -> list :List<T> -> List<T> = 
  List.sortWith compareFn list

// 型クラス制約
fn showAll<T: Show<T>> :List<T> -> String = 
  List.map show >> String.join ", "
```

### 13.2 高階カインド型（将来機能）

#### 13.2.1 カインド推論
```seseragi
// * -> * カインドの型構成子
type Container<T> = { value: T }     // Container :: * -> *
type Either<L, R> = Left L | Right R // Either :: * -> * -> *

// (* -> *) -> * カインドの型構成子
type Transformer<F<_>, T> = F<T>     // Transformer :: (* -> *) -> * -> *

// 高階関数での使用
fn liftTransformer<F<_>: Functor<F>, T, U> 
  f :(T -> U) -> 
  transformer :Transformer<F, T> -> 
  Transformer<F, U> = 
  fmap f transformer
```

### 13.3 マクロシステム（将来機能）

#### 13.3.1 構文マクロ
```seseragi
// マクロの定義
macro for (item in items) body = 
  List.map (\item -> body) items

macro unless condition body = 
  if not condition then body else ()

// 使用例
let doubled = for (x in [1, 2, 3, 4, 5]) (x * 2)
unless (List.isEmpty list) (processItems list)
```

#### 13.3.2 型レベルマクロ
```seseragi
// 型レベルでの計算
macro type Add<N, M> = /* 型レベル計算 */
macro type Length<L> = /* リストの長さを型レベルで計算 */

// 依存型的な使用（将来機能）
fn safeIndex<N: Nat, T> index :N -> list :List<T> -> 
  where Length<List<T>> > N -> T = 
  List.unsafeIndex index list  // 境界チェック不要
```

### 13.4 並行性プリミティブ

#### 13.4.1 軽量スレッド（Green Threads）
```seseragi
// 非同期計算の定義
effectful fn asyncComputation x :Int -> Async<Int> = do {
  yield  // 他のタスクに制御を譲る
  let result = expensiveCalculation x
  return result
}

// 並行実行
effectful fn runConcurrently computations :List<Async<Int>> -> IO<List<Int>> = do {
  results <- Async.parallel computations
  return results
}
```

#### 13.4.2 チャネル通信
```seseragi
// チャネルベースの通信
effectful fn producer channel :Channel<Int> -> IO<Unit> = do {
  for (i in [1..100]) {
    Channel.send channel i
    yield
  }
  Channel.close channel
}

effectful fn consumer channel :Channel<Int> -> IO<List<Int>> = do {
  results <- Channel.collect channel
  return results
}

effectful fn pipeline -> IO<List<Int>> = do {
  channel <- Channel.create
  fork (producer channel)
  results <- consumer channel
  return results
}
```

### 13.5 最適化機能

#### 13.5.1 末尾呼び出し最適化
```seseragi
// 末尾再帰関数は自動的に最適化される
fn factorial n :Int -> Int = factorialHelper n 1
  where
    factorialHelper 0 acc = acc
    factorialHelper n acc = factorialHelper (n - 1) (n * acc)

// 相互末尾再帰も最適化
fn isEven n :Int -> Bool = if n == 0 then True else isOdd (n - 1)
fn isOdd n :Int -> Bool = if n == 0 then False else isEven (n - 1)
```

#### 13.5.2 インライン化
```seseragi
// 小さな関数は自動的にインライン化
inline fn square x :Int -> Int = x * x

// 使用箇所で展開される
let result = square 5  // -> let result = 5 * 5
```

#### 13.5.3 デッドコード除去
```seseragi
// 使用されない関数や変数は自動的に除去
fn usedFunction x :Int -> Int = x * 2
fn unusedFunction x :Int -> Int = x * 3  // コンパイル時に除去

let usedVariable = 42
let unusedVariable = 100  // コンパイル時に除去

// 条件分岐の最適化
let result = if True then 42 else 0  // -> let result = 42
```

---

## 14. 標準ライブラリ

### 14.1 Core モジュール

#### 14.1.1 基本型操作
```seseragi
module Core.Int
export fn toString :Int -> String
export fn fromString :String -> Maybe<Int>
export fn abs :Int -> Int
export fn sign :Int -> Int
export fn max :Int -> Int -> Int
export fn min :Int -> Int -> Int

module Core.Float  
export fn toString :Float -> String
export fn fromString :String -> Maybe<Float>
export fn round :Float -> Int
export fn ceiling :Float -> Int
export fn floor :Float -> Int
export fn isNaN :Float -> Bool
export fn isInfinite :Float -> Bool

module Core.String
export fn length :String -> Int
export fn isEmpty :String -> Bool
export fn trim :String -> String
export fn split :String -> String -> List<String>
export fn join :String -> List<String> -> String
export fn contains :String -> String -> Bool
export fn startsWith :String -> String -> Bool
export fn endsWith :String -> String -> Bool
export fn toUpper :String -> String
export fn toLower :String -> String
```

#### 14.1.2 関数操作
```seseragi
module Core.Function
export fn identity<T> :T -> T = \x -> x
export fn const<T, U> :T -> U -> T = \x _ -> x
export fn flip<A, B, C> :(A -> B -> C) -> B -> A -> C = \f x y -> f y x
export fn compose<A, B, C> :(B -> C) -> (A -> B) -> A -> C = \f g x -> f (g x)
export fn curry<A, B, C> :((A, B) -> C) -> A -> B -> C = \f x y -> f (x, y)
export fn uncurry<A, B, C> :(A -> B -> C) -> (A, B) -> C = \f (x, y) -> f x y
```

### 14.2 データ構造

#### 14.2.1 List モジュール
```seseragi
module Data.List

// 基本操作
export fn empty<T> :List<T> = []
export fn singleton<T> :T -> List<T> = \x -> [x]
export fn cons<T> :T -> List<T> -> List<T> = \x xs -> x :: xs
export fn head<T> :List<T> -> Maybe<T>
export fn tail<T> :List<T> -> Maybe<List<T>>
export fn length<T> :List<T> -> Int
export fn isEmpty<T> :List<T> -> Bool

// 変換
export fn map<A, B> :(A -> B) -> List<A> -> List<B>
export fn filter<T> :(T -> Bool) -> List<T> -> List<T>
export fn fold<A, B> :(A -> B -> A) -> A -> List<B> -> A
export fn foldRight<A, B> :(A -> B -> B) -> B -> List<A> -> B
export fn reverse<T> :List<T> -> List<T>

// 組み合わせ
export fn append<T> :List<T> -> List<T> -> List<T>
export fn concat<T> :List<List<T>> -> List<T>
export fn zip<A, B> :List<A> -> List<B> -> List<(A, B)>
export fn zipWith<A, B, C> :(A -> B -> C) -> List<A> -> List<B> -> List<C>

// 検索
export fn find<T> :(T -> Bool) -> List<T> -> Maybe<T>
export fn findIndex<T> :(T -> Bool) -> List<T> -> Maybe<Int>
export fn elem<T: Eq<T>> :T -> List<T> -> Bool
export fn notElem<T: Eq<T>> :T -> List<T> -> Bool

// ソート
export fn sort<T: Ord<T>> :List<T> -> List<T>
export fn sortBy<T> :(T -> T -> Ordering) -> List<T> -> List<T>
export fn sortOn<T, U: Ord<U>> :(T -> U) -> List<T> -> List<T>

// 部分リスト
export fn take<T> :Int -> List<T> -> List<T>
export fn drop<T> :Int -> List<T> -> List<T>
export fn takeWhile<T> :(T -> Bool) -> List<T> -> List<T>
export fn dropWhile<T> :(T -> Bool) -> List<T> -> List<T>
export fn slice<T> :Int -> Int -> List<T> -> List<T>
```

#### 14.2.2 Map モジュール
```seseragi
module Data.Map

export type Map<K, V>

// 構築
export fn empty<K, V> :Map<K, V>
export fn singleton<K, V> :K -> V -> Map<K, V>
export fn fromList<K: Ord<K>, V> :List<(K, V)> -> Map<K, V>

// 更新
export fn insert<K: Ord<K>, V> :K -> V -> Map<K, V> -> Map<K, V>
export fn delete<K: Ord<K>, V> :K -> Map<K, V> -> Map<K, V>
export fn update<K: Ord<K>, V> :(Maybe<V> -> Maybe<V>) -> K -> Map<K, V> -> Map<K, V>

// 検索
export fn lookup<K: Ord<K>, V> :K -> Map<K, V> -> Maybe<V>
export fn member<K: Ord<K>, V> :K -> Map<K, V> -> Bool
export fn size<K, V> :Map<K, V> -> Int

// 変換
export fn map<K, A, B> :(A -> B) -> Map<K, A> -> Map<K, B>
export fn mapWithKey<K, A, B> :(K -> A -> B) -> Map<K, A> -> Map<K, B>
export fn filter<K, V> :(V -> Bool) -> Map<K, V> -> Map<K, V>
export fn filterWithKey<K, V> :(K -> V -> Bool) -> Map<K, V> -> Map<K, V>

// 変換
export fn keys<K, V> :Map<K, V> -> List<K>
export fn values<K, V> :Map<K, V> -> List<V>
export fn toList<K, V> :Map<K, V> -> List<(K, V)>

// 集合操作
export fn union<K: Ord<K>, V> :Map<K, V> -> Map<K, V> -> Map<K, V>
export fn intersection<K: Ord<K>, V> :Map<K, V> -> Map<K, V> -> Map<K, V>
export fn difference<K: Ord<K>, V> :Map<K, V> -> Map<K, V> -> Map<K, V>
```

### 14.3 IO とファイルシステム

#### 14.3.1 IO モジュール
```seseragi
module System.IO

// 標準入出力
effectful export fn print :String -> IO<Unit>
effectful export fn println :String -> IO<Unit>
effectful export fn readLine -> IO<String>
effectful export fn readChar -> IO<Char>

// ファイル操作
effectful export fn readFile :String -> IO<String>
effectful export fn writeFile :String -> String -> IO<Unit>
effectful export fn appendFile :String -> String -> IO<Unit>
effectful export fn deleteFile :String -> IO<Unit>

// ディレクトリ操作
effectful export fn createDirectory :String -> IO<Unit>
effectful export fn removeDirectory :String -> IO<Unit>
effectful export fn listDirectory :String -> IO<List<String>>
effectful export fn getCurrentDirectory -> IO<String>
effectful export fn setCurrentDirectory :String -> IO<Unit>

// ファイル情報
effectful export fn fileExists :String -> IO<Bool>
effectful export fn directoryExists :String -> IO<Bool>
effectful export fn fileSize :String -> IO<Int>
effectful export fn getModificationTime :String -> IO<String>
```

#### 14.3.2 Process モジュール
```seseragi
module System.Process

export type ExitCode = Success | Failure Int

// プロセス実行
effectful export fn execute :String -> IO<ExitCode>
effectful export fn executeWithOutput :String -> IO<(ExitCode, String, String)>
effectful export fn spawn :String -> List<String> -> IO<ProcessHandle>

// 環境変数
effectful export fn getEnv :String -> IO<Maybe<String>>
effectful export fn setEnv :String -> String -> IO<Unit>
effectful export fn getAllEnv -> IO<Map<String, String>>

// システム情報
effectful export fn getArgs -> IO<List<String>>
effectful export fn getProgName -> IO<String>
effectful export fn exitWith :ExitCode -> IO<Unit>
```

### 14.4 ネットワーク

#### 14.4.1 HTTP Client
```seseragi
module Network.HTTP.Client

export type HttpMethod = GET | POST | PUT | DELETE | PATCH
export type HttpHeaders = Map<String, String>
export type HttpRequest {
  method: HttpMethod
  url: String
  headers: HttpHeaders
  body: Maybe<String>
}

export type HttpResponse {
  status: Int
  headers: HttpHeaders
  body: String
}

// HTTP リクエスト
effectful export fn request :HttpRequest -> IO<Either<String, HttpResponse>>
effectful export fn get :String -> IO<Either<String, HttpResponse>>
effectful export fn post :String -> String -> IO<Either<String, HttpResponse>>
effectful export fn put :String -> String -> IO<Either<String, HttpResponse>>
effectful export fn delete :String -> IO<Either<String, HttpResponse>>

// ヘルパー関数
export fn withHeaders :HttpHeaders -> HttpRequest -> HttpRequest
export fn withAuth :String -> HttpRequest -> HttpRequest
export fn withTimeout :Int -> HttpRequest -> HttpRequest
```

### 14.5 JSON処理

#### 14.5.1 JSON モジュール
```seseragi
module Data.JSON

export type JSON = 
  | JNull
  | JBool Bool
  | JNumber Float
  | JString String
  | JArray (List<JSON>)
  | JObject (Map<String, JSON>)

// パース
export fn parse :String -> Either<String, JSON>
export fn parseWith :JSONConfig -> String -> Either<String, JSON>

// エンコード
export fn encode :JSON -> String
export fn encodePretty :JSON -> String

// アクセス
export fn get :String -> JSON -> Maybe<JSON>
export fn getIn :List<String> -> JSON -> Maybe<JSON>
export fn set :String -> JSON -> JSON -> JSON
export fn update :String -> (JSON -> JSON) -> JSON -> JSON

// 型安全なデコーダー
export trait FromJSON<T> {
  fn fromJSON :JSON -> Either<String, T>
}

export trait ToJSON<T> {
  fn toJSON :T -> JSON
}

// 基本型のインスタンス
impl FromJSON<Int> { ... }
impl FromJSON<String> { ... }
impl FromJSON<Bool> { ... }
impl FromJSON<List<T>> where T: FromJSON<T> { ... }

impl ToJSON<Int> { ... }
impl ToJSON<String> { ... }
impl ToJSON<Bool> { ... }
impl ToJSON<List<T>> where T: ToJSON<T> { ... }
```

---

## 15. エラーハンドリング

### 15.1 例外処理の禁止

Seseragiは純粋関数型言語として、例外によるエラーハンドリングを禁止し、型によるエラー表現を推奨します。

### 15.2 Maybe型による失敗の表現

#### 15.2.1 基本的な使用
```seseragi
// 失敗する可能性のある操作
fn safeDivide x :Float -> y :Float -> Maybe<Float> = 
  if y == 0.0 then Nothing else Just (x / y)

fn safeHead list :List<T> -> Maybe<T> = match list {
  [] -> Nothing
  x :: _ -> Just x
}

// チェーン操作
fn calculateSafely a :Float -> b :Float -> c :Float -> Maybe<Float> = 
  safeDivide a b >>= \result1 ->
  safeDivide result1 c >>= \result2 ->
  Just (result2 * 2.0)
```

### 15.3 Either型による詳細なエラー情報

#### 15.3.1 カスタムエラー型
```seseragi
type ParseError = 
  | InvalidSyntax String Int  // エラーメッセージと行番号
  | UnexpectedEOF
  | InvalidCharacter Char Int

type ValidationError = 
  | Required String           // 必須フィールド名
  | InvalidFormat String      // フィールド名
  | OutOfRange String Int Int // フィールド名、最小値、最大値

fn parseInteger input :String -> Either<ParseError, Int> = 
  if String.isEmpty input then Left (InvalidSyntax "Empty input" 1)
  else if String.all Char.isDigit input then Right (String.toInt input)
  else Left (InvalidSyntax "Non-digit character found" 1)

fn validateAge age :Int -> Either<ValidationError, Int> = 
  if age < 0 then Left (OutOfRange "age" 0 150)
  else if age > 150 then Left (OutOfRange "age" 0 150)
  else Right age
```

#### 15.3.2 エラーの合成
```seseragi
// 複数のエラーを蓄積
type ValidationResult<T> = Either<List<ValidationError>, T>

fn validateUser name :String -> age :Int -> email :String -> ValidationResult<User> = 
  let nameResult = validateName name
  let ageResult = validateAge age  
  let emailResult = validateEmail email
  
  // Applicativeを使用してエラーを蓄積
  User <$> nameResult <*> ageResult <*> emailResult

// エラーメッセージの生成
fn formatValidationErrors errors :List<ValidationError> -> String = 
  errors 
  | List.map formatSingleError
  | String.join "; "
  
fn formatSingleError error :ValidationError -> String = match error {
  Required field -> field ++ " is required"
  InvalidFormat field -> field ++ " has invalid format"
  OutOfRange field min max -> field ++ " must be between " ++ toString min ++ " and " ++ toString max
}
```

### 15.4 Result型による統一的なエラーハンドリング

#### 15.4.1 Result型の定義
```seseragi
type Result<T, E> = Ok T | Error E

// Eitherとの相互変換
fn fromEither either :Either<E, T> -> Result<T, E> = match either {
  Left e -> Error e
  Right t -> Ok t
}

fn toEither result :Result<T, E> -> Either<E, T> = match result {
  Ok t -> Right t
  Error e -> Left e
}
```

#### 15.4.2 Result型の活用
```seseragi
type IOError = 
  | FileNotFound String
  | PermissionDenied String  
  | NetworkError String

effectful fn safeReadFile path :String -> IO<Result<String, IOError>> = do {
  try {
    content <- readFile path
    return (Ok content)
  } catch {
    FileNotFoundException msg -> return (Error (FileNotFound msg))
    PermissionException msg -> return (Error (PermissionDenied msg))
    error -> return (Error (NetworkError (toString error)))
  }
}

// エラーハンドリングのパターン
effectful fn processFile path :String -> IO<Unit> = do {
  result <- safeReadFile path
  match result {
    Ok content -> {
      processedContent <- processContent content
      writeFile (path ++ ".processed") processedContent
      println ("Processed file: " ++ path)
    }
    Error (FileNotFound _) -> 
      println ("File not found: " ++ path)
    Error (PermissionDenied _) -> 
      println ("Permission denied: " ++ path)
    Error (NetworkError msg) -> 
      println ("Network error: " ++ msg)
  }
}
```

### 15.5 デバッグとログ機能

#### 15.5.1 ログシステム
```seseragi
module Debug.Logger

export type LogLevel = Debug | Info | Warning | Error

export type LogEntry {
  level: LogLevel
  message: String
  timestamp: String
  module: String
}

effectful export fn log :LogLevel -> String -> IO<Unit>
effectful export fn debug :String -> IO<Unit> = log Debug
effectful export fn info :String -> IO<Unit> = log Info  
effectful export fn warning :String -> IO<Unit> = log Warning
effectful export fn error :String -> IO<Unit> = log Error

// 条件付きログ
effectful export fn logWhen :Bool -> LogLevel -> String -> IO<Unit> = 
  \condition level message -> 
    if condition then log level message else IO.pure ()

// デバッグビルドでのみ有効
effectful export fn debugLog :String -> IO<Unit> = 
  when BuildConfig.isDebug (debug message)
```

#### 15.5.2 アサーション
```seseragi
module Debug.Assert

// 開発時のアサーション（プロダクションでは無効化）
fn assert condition :Bool -> message :String -> Unit = 
  when BuildConfig.isDebug {
    if not condition then 
      panic ("Assertion failed: " ++ message)
    else 
      ()
  }

fn assertEq<T: Eq<T>> expected :T -> actual :T -> String -> Unit = 
  assert (expected == actual) ("Expected " ++ show expected ++ ", got " ++ show actual)

// 使用例
fn safeDivide x :Float -> y :Float -> Float = 
  assert (y != 0.0) "Division by zero"
  x / y
```

---

## 16. 形式的文法定義（EBNF）

### 16.1 字句要素

```ebnf
(* 基本要素 *)
letter = 'A' | 'B' | ... | 'Z' | 'a' | 'b' | ... | 'z' ;
digit = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' ;
alphanumeric = letter | digit ;

(* 識別子 *)
identifier = lowercase_letter, { alphanumeric | '_' } ;
type_identifier = uppercase_letter, { alphanumeric | '_' } ;
uppercase_letter = 'A' | 'B' | ... | 'Z' ;
lowercase_letter = 'a' | 'b' | ... | 'z' ;

(* リテラル *)
integer = [ '-' ], digit, { digit | '_' } ;
hex_integer = '0', ( 'x' | 'X' ), hex_digit, { hex_digit | '_' } ;
oct_integer = '0', ( 'o' | 'O' ), oct_digit, { oct_digit | '_' } ;
bin_integer = '0', ( 'b' | 'B' ), bin_digit, { bin_digit | '_' } ;

float = [ '-' ], digit, { digit }, '.', digit, { digit }, [ exponent ] ;
exponent = ( 'e' | 'E' ), [ '+' | '-' ], digit, { digit } ;

string = '"', { string_char }, '"' ;
string_char = printable_char - '"' | escape_sequence ;
escape_sequence = '\', ( 'n' | 't' | 'r' | '0' | '\' | '"' | unicode_escape ) ;
unicode_escape = 'u', '{', hex_digit, { hex_digit }, '}' ;

char = "'", ( printable_char - "'" | escape_sequence ), "'" ;
boolean = 'True' | 'False' ;
unit = '(', ')' ;

(* コメント *)
comment = '//', { printable_char }, newline ;
```

### 16.2 型表現

```ebnf
type_expr = 
    simple_type
  | function_type
  | generic_type
  | tuple_type
  | parenthesized_type ;

simple_type = type_identifier ;

function_type = type_expr, '->', type_expr ;

generic_type = type_identifier, '<', type_expr, { ',', type_expr }, '>' ;

tuple_type = '(', type_expr, ',', type_expr, { ',', type_expr }, ')' ;

parenthesized_type = '(', type_expr, ')' ;
```

### 16.3 式

```ebnf
expr = 
    literal
  | identifier
  | function_call
  | lambda_expr
  | if_expr
  | match_expr
  | binary_expr
  | unary_expr
  | list_expr
  | record_expr
  | tuple_expr
  | parenthesized_expr ;

literal = integer | float | string | char | boolean | unit ;

function_call = expr, '(', [ expr, { ',', expr } ], ')' ;

lambda_expr = '\', pattern, { pattern }, '->', expr ;

if_expr = 'if', expr, 'then', expr, 'else', expr ;

match_expr = 'match', expr, '{', { pattern, '->', expr }, '}' ;

binary_expr = expr, binary_op, expr ;
binary_op = 
    '+' | '-' | '*' | '/' | '%' | '**'
  | '++' | '::'
  | '==' | '!=' | '<' | '>' | '<=' | '>='
  | '&&' | '||'
  | '|' | '~' | '$'
  | '<$>' | '<*>' | '>>=' | '>>>' ;

unary_expr = unary_op, expr ;
unary_op = '-' | '!' ;

list_expr = '[', [ expr, { ',', expr } ], ']' ;

record_expr = type_identifier, '{', [ field_binding, { ',', field_binding } ], '}' ;
field_binding = identifier, ':', expr ;

tuple_expr = '(', expr, ',', expr, { ',', expr }, ')' ;

parenthesized_expr = '(', expr, ')' ;
```

### 16.4 パターン

```ebnf
pattern = 
    literal_pattern
  | identifier_pattern
  | constructor_pattern
  | list_pattern
  | record_pattern
  | tuple_pattern
  | wildcard_pattern
  | parenthesized_pattern ;

literal_pattern = literal ;

identifier_pattern = identifier ;

constructor_pattern = type_identifier, { pattern } ;

list_pattern = 
    '[', ']'
  | '[', pattern, { ',', pattern }, ']'
  | pattern, '::', pattern ;

record_pattern = type_identifier, '{', [ field_pattern, { ',', field_pattern } ], '}' ;
field_pattern = identifier, ':', pattern ;

tuple_pattern = '(', pattern, ',', pattern, { ',', pattern }, ')' ;

wildcard_pattern = '_' ;

parenthesized_pattern = '(', pattern, ')' ;
```

### 16.5 定義

```ebnf
definition = 
    variable_def
  | function_def
  | type_def
  | module_def
  | import_def ;

variable_def = 'let', pattern, [ ':', type_expr ], '=', expr ;

function_def = 
    function_signature, '=', expr
  | function_signature, '{', { statement }, '}'
  | function_signature  (* シグネチャのみ *) ;

function_signature = 'fn', identifier, { parameter }, '->', type_expr ;
parameter = identifier, ':', type_expr ;

type_def = 
    record_type_def
  | variant_type_def
  | type_alias_def ;

record_type_def = 'type', type_identifier, [ generic_params ], '{', { field_def }, '}' ;
field_def = identifier, ':', type_expr ;

variant_type_def = 'type', type_identifier, [ generic_params ], '=', variant, { '|', variant } ;
variant = type_identifier, { type_expr } ;

type_alias_def = 'type', type_identifier, [ generic_params ], '=', type_expr ;

generic_params = '<', identifier, { ',', identifier }, '>' ;

module_def = 'module', module_name, { definition } ;
module_name = type_identifier, { '.', type_identifier } ;

import_def = 'import', module_name, [ import_spec ] ;
import_spec = 
    'as', identifier
  | ':', '{', [ import_item, { ',', import_item } ], '}'
  | ':', '{', '*', '}' ;
import_item = identifier | type_identifier ;
```

### 16.6 文

```ebnf
statement = 
    expr_statement
  | definition
  | return_statement ;

expr_statement = expr ;

return_statement = 'return', expr ;
```

### 16.7 プログラム

```ebnf
program = { definition } ;
```

---

この完全文法仕様により、Seseragi言語の最終的な目標が明確に定義されました。すべての機能が実装された際の言語の全貌を示しており、段階的な実装計画の指針となります。
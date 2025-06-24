# Seseragi プログラミング言語仕様

## 1. 言語概要

Seseragiは、TypeScriptにトランスパイルされる純粋関数型プログラミング言語です。以下の特徴を持ちます：

- **純粋関数型プログラミング** - 副作用を明示的に管理
- **静的型付け** - コンパイル時の型安全性を保証
- **不変性** - すべての変数は不変（immutable）
- **カリー化** - すべての関数はデフォルトでカリー化
- **モナド** - 副作用、エラー、オプション値の安全な処理

---

## 2. 予約語

```
fn          // 関数定義
type        // 型定義
let         // 変数定義
impl        // 型の実装ブロック
monoid      // モノイド定義
operator    // 演算子定義
return      // 明示的な返り値
effectful   // 副作用を持つ関数
elevate     // 純粋関数内での副作用関数呼び出し
import      // モジュールインポート
as          // エイリアス
match       // パターンマッチング
case        // パターンマッチングのケース
if          // 条件式
then        // 条件分岐 - 真の場合
else        // 条件分岐 - 偽の場合
pure        // 値をモナドに持ち上げ
perform     // モナドの副作用実行
```

---

## 3. 基本型

### 3.1 プリミティブ型
- **`Int`** - 整数型
- **`Float`** - 浮動小数点数型
- **`Bool`** - 真偽値型（`True` または `False`）
- **`String`** - 文字列型
- **`Char`** - 単一文字型
- **`Unit`** - 戻り値を持たない場合の型

### 3.2 コレクション型
- **`List<T>`** - 任意の型Tのリスト
- **`Array<T>`** - 任意の型Tの配列

### 3.3 モナド型
- **`Maybe<T>`** - `Just T` または `Nothing`
- **`Either<L, R>`** - 左側がエラー型L、右側が成功型R
- **`IO<T>`** - 副作用を持つ型Tを包むモナド

---

## 4. 変数定義

変数は`let`キーワードで定義し、すべて不変です。

```rust
let x :Int = 10
let name :String = "Seseragi"
let flag :Bool = True
```

---

## 5. 関数定義

### 5.1 基本的な関数定義

**ワンライナー形式:**
```rust
fn add a :Int -> b :Int -> Int = a + b
```

**ブロック形式:**
```rust
fn add a :Int -> b :Int -> Int {
    a + b  // 最終行が自動的に返される
}

fn addExplicit a :Int -> b :Int -> Int {
    return a + b  // 明示的なreturn
}
```

### 5.2 高階関数

関数を引数として受け取る場合、関数型をカッコで囲みます。

```rust
fn applyTwice f :(Int -> Int) -> a :Int -> Int {
    f(f(a))
}

let result = applyTwice (add 2) 5  // 結果: 9
```

### 5.3 カリー化と部分適用

すべての関数はカリー化されており、部分適用が可能です。

```rust
fn triple a :Int -> b :Int -> c :Int -> Int = a + b + c

let partialFn = triple 1 2  // a=1, b=2 で部分適用
let result = partialFn 3    // c=3 を適用して評価: 6
```

---

## 6. 演算子

### 6.1 パイプライン演算子 (`|`)

関数適用を左から右に繋げます。

```rust
fn add1 x :Int -> Int = x + 1
fn square x :Int -> Int = x * x

let result = 2 | add1 | square  // 結果: 9 ((2+1)^2)
```

### 6.2 逆パイプ演算子 (`~`)

右オペランドの関数の第一引数に左オペランドを適用します。

```rust
fn triple a :Int -> b :Int -> c :Int -> Int = a + b + c

let result = 1 ~ triple 2 3  // 1 + 2 + 3 = 6
```

### 6.3 モナドバインド演算子 (`>>=`)

モナドの値を次のモナド関数に流し込みます。

```rust
fn addFive x :Int -> Maybe<Int> = Just (x + 5)

let result = Just 10 >>= addFive  // 結果: Just 15
```

### 6.4 モノイド畳み込み演算子 (`>>>`)

モノイドの畳み込み操作を行います。

```rust
let sum = List<Int>[1, 2, 3] >>> (+)  // 結果: 6
```

---

## 7. 制御フロー

### 7.1 条件分岐

`if then else`式を使用して条件分岐を行います。

```rust
let result = if age < 18 then "Child" else "Adult"
```

### 7.2 条件式の評価

比較演算子の結果は`Bool`型として評価され、`if`式で直接使用できます。

```rust
let condition = x < 10 && y > 5  // Bool型
let result = if condition then "OK" else "NG"
```

### 7.3 パターンマッチング

`match`キーワードを使用してパターンマッチングを行います。これはHaskellの`case`式に相当し、代数的データ型の値を分解して処理します。

```rust
match maybeValue {
    Just x  -> handleJust x
    Nothing -> handleNothing
}

match eitherValue {
    Left error -> handleError error
    Right value -> handleSuccess value
}
```

**if式とmatch式の使い分け:**
- **if式**: 単純な条件による値の選択 (`if condition then value1 else value2`)
- **match式**: パターンマッチングによる代数的データ型の分解と処理

---

## 8. 型定義

### 8.1 ユーザー定義型

```rust
type Person {
    name :String
    age :Int
}

type Wallet {
    amount :Int
}
```

### 8.2 型の実装 (`impl`)

メソッドや演算子は`impl`ブロック内で定義します。

```rust
impl Wallet {
    fn add self -> other :Wallet -> Wallet {
        Wallet { amount: self.amount + other.amount }
    }
    
    fn sub self -> other :Wallet -> Wallet {
        Wallet { amount: self.amount - other.amount }
    }
}
```

### 8.3 演算子定義

中置記法で使用する演算子は`operator`で明示的に定義します。

```rust
impl Wallet {
    operator + (self, other :Wallet) -> Wallet {
        self.add other
    }
    
    operator - (self, other :Wallet) -> Wallet {
        self.sub other
    }
}

// 使用例
let wallet1 = Wallet { amount: 100 }
let wallet2 = Wallet { amount: 50 }
let result = wallet1 + wallet2  // Wallet { amount: 150 }
```

### 8.4 モノイド定義

モノイドは結合律を持つ演算と単位元を定義します。

```rust
impl Wallet {
    monoid {
        identity Wallet { amount: 0 }
        
        operator + (self, other :Wallet) -> Wallet {
            self.add other
        }
    }
}
```

---

## 9. モナド

### 9.1 `Maybe<T>`

値の存在を安全に扱うモナドです。

```rust
fn safeDivide x :Int -> y :Int -> Maybe<Int> {
    if y == 0 then Nothing else Just (x / y)
}

let result = safeDivide 10 2 >>= (\x -> Just (x * 2))  // Just 10
```

### 9.2 `Either<L, R>`

エラーと成功を表現するモナドです。

```rust
fn parseInt str :String -> Either<String, Int> {
    if String.isInt str
    then Right (String.toInt str)
    else Left ("Not a valid number: " ++ str)
}
```

### 9.3 `IO<T>`

副作用を管理するモナドです。

```rust
effectful fn printMessage msg :String -> IO<Unit> {
    IO.pure (println msg)
}

effectful fn readInput -> IO<String> {
    IO.pure (readLine)
}
```

---

## 10. 副作用の管理

### 10.1 `effectful`関数

副作用を持つ関数は`effectful`キーワードで定義します。

```rust
effectful fn writeFile filename :String -> content :String -> IO<Unit> {
    IO.pure (systemWriteFile filename content)
}
```

### 10.2 `elevate`による副作用関数の呼び出し

純粋関数内で副作用関数を呼び出す場合は`elevate`を使用します。

```rust
fn processData data :String -> IO<String> {
    elevate (printMessage "Processing...") >>=
    (\_ -> IO.pure (data ++ " processed"))
}
```

---

## 11. モジュールシステム

### 11.1 インポート

```rust
import math::{sin, cos, pi}
import data::List as DataList
import shopping::{Product, Cart}
```

### 11.2 ファイル構造

- モジュールはファイルパスベースで管理
- ディレクトリ区切りは`::`を使用
- 型衝突を避けるために`as`でエイリアス可能

---

## 12. エラーハンドリング

### 12.1 `Maybe`を使用したエラーハンドリング

```rust
fn safeOperation x :Int -> Maybe<Int> {
    if x > 0 then Just (x * 2) else Nothing
}

let result = safeOperation 5 >>= (\x -> Just (x + 1))  // Just 11
```

### 12.2 `Either`を使用したエラーハンドリング

```rust
fn validateAge age :Int -> Either<String, Int> {
    if age >= 0 && age <= 150
    then Right age
    else Left "Invalid age"
}
```

---

## 13. 使用例

```rust
// 型定義
type User {
    name :String
    age :Int
}

// 関数定義
fn createUser name :String -> age :Int -> Maybe<User> {
    if age >= 0 && age <= 150
    then Just (User { name: name, age: age })
    else Nothing
}

fn greetUser user :User -> String {
    "Hello, " ++ user.name ++ "! You are " ++ String.fromInt user.age ++ " years old."
}

// メイン処理
let result = createUser "Alice" 25 >>= (\user -> Just (greetUser user))
// 結果: Just "Hello, Alice! You are 25 years old."
```
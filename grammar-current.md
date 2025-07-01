# Seseragi言語 実装済み文法仕様

この文書は、現在実装済みで動作確認されているSeseragi言語の機能のみを記載します。

**⚠️ 重要**: この文書の内容は全て実装済みで、テストが通っており、実際に動作します。

## 目次

1. [実装状況概要](#1-実装状況概要)
2. [字句要素](#2-字句要素)
3. [基本型](#3-基本型)
4. [変数定義](#4-変数定義)
5. [関数定義](#5-関数定義)
6. [演算子](#6-演算子)
7. [制御構造](#7-制御構造)
8. [データ構造](#8-データ構造)
9. [モナド基礎](#9-モナド基礎)
10. [CLI機能](#10-cli機能)
11. [実行例](#11-実行例)

---

## 1. 実装状況概要

### ✅ 動作確認済み機能
- 基本リテラル（整数、浮動小数点、文字列、真偽値）
- 変数定義（`let`）
- 関数定義（ワンライナー、ブロック形式）
- 基本演算子（算術、比較、論理）
- 特殊演算子（パイプライン`|`、逆パイプ`~`、関数適用`$`）
- if-then-else式
- Maybe型とEither型の基本機能
- CLI（compile, run, format）

### ❌ 未実装機能
- パターンマッチング（match式）
- 型クラス（impl, monoid）
- 副作用管理（effectful, IO）
- モジュールシステム（import/export）
- カスタム型定義（type）
- 高度な型推論

---

## 2. 字句要素

### 2.1 実装済みリテラル

#### 2.1.1 整数リテラル
```seseragi
42          // 10進数
-123        // 負の数
```

#### 2.1.2 浮動小数点リテラル
```seseragi
3.14        // 基本形
-2.5        // 負の数
```

#### 2.1.3 文字列リテラル
```seseragi
"Hello, World!"    // 基本的な文字列
""                 // 空文字列
```

#### 2.1.4 真偽値リテラル
```seseragi
True        // 真
False       // 偽
```

### 2.2 識別子

```seseragi
// 変数名・関数名（小文字開始）
x
count
userName
isValid

// 型名（大文字開始） - 基本型のみ使用可能
Int
Float
String
Bool
Maybe
Either
```

### 2.3 コメント
```seseragi
// 単行コメントのみサポート
let x = 42  // 行末コメント
```

---

## 3. 基本型

### 3.1 プリミティブ型
- **`Int`** - 整数型
- **`Float`** - 浮動小数点数型
- **`Bool`** - 真偽値型
- **`String`** - 文字列型

### 3.2 モナド型（基本実装）
- **`Maybe<T>`** - `Just T` または `Nothing`
- **`Either<L, R>`** - `Left L` または `Right R`

---

## 4. 変数定義

### 4.1 基本的な変数定義

```seseragi
// 型推論による定義
let x = 42
let name = "Alice"
let flag = True

// 明示的な型注釈（推奨）
let count: Int = 100
let message: String = "Hello"
let isActive: Bool = False
```

### 4.2 制約
- すべての変数は不変（immutable）
- 再代入は不可

---

## 5. 関数定義

### 5.1 ワンライナー形式

```seseragi
// 基本的な関数
fn add x: Int -> y: Int -> Int = x + y
fn double n: Int -> Int = n * 2
fn greet name: String -> String = "Hello, " ++ name

// 引数なし関数
fn getMessage -> String = "Hello from function!"
fn getNumber -> Int = 42
```

### 5.2 ブロック形式

```seseragi
fn processData input: String -> String {
  let cleaned = input
  "Processed: " ++ cleaned
}

fn max x: Int -> y: Int -> Int {
  if x > y then x else y
}
```

### 5.3 カリー化

すべての関数は自動的にカリー化され、部分適用が可能です。

```seseragi
fn add x: Int -> y: Int -> Int = x + y

let addFive = add 5    // Int -> Int（部分適用）
let result = addFive 3 // 8
```

---

## 6. 演算子

### 6.1 算術演算子

```seseragi
let sum = 5 + 3        // 8
let diff = 10 - 4      // 6
let product = 6 * 7    // 42
let quotient = 15 / 3  // 5
let remainder = 17 % 5 // 2
```

### 6.2 比較演算子

```seseragi
let equal = 5 == 5        // True
let notEqual = 5 != 3     // True
let less = 3 < 5          // True
let greater = 5 > 3       // True
let lessEqual = 5 <= 5    // True
let greaterEqual = 5 >= 5 // True
```

### 6.3 論理演算子

```seseragi
let and = True && False   // False
let or = True || False    // True
let not = !True           // False
```

### 6.4 文字列演算子

```seseragi
let greeting = "Hello" ++ " " ++ "World"  // "Hello World"
```

### 6.5 特殊演算子

#### 6.5.1 パイプライン演算子 (`|`)
```seseragi
fn add1 x: Int -> Int = x + 1
fn double x: Int -> Int = x * 2

let result = 5 | add1 | double  // 12 ((5+1)*2)
```

#### 6.5.2 逆パイプ演算子 (`~`)
```seseragi
fn add x: Int -> y: Int -> Int = x + y

let result = 10 ~ add 5  // add 10 5 = 15
```

#### 6.5.3 関数適用演算子 (`$`)
```seseragi
fn toString x: Int -> String = x  // 仮の実装

let result = toString $ add 10 5  // toString (add 10 5)
```

---

## 7. 制御構造

### 7.1 条件分岐（if-then-else）

```seseragi
// 基本的な条件分岐
let result = if x > 0 then "positive" else "non-positive"

// ネストした条件
let grade = if score >= 90 then "A"
           else if score >= 80 then "B"
           else "F"

// ブロック形式
let result = if condition then {
  let temp = calculate something
  temp * 2
} else {
  defaultValue
}
```

---

## 8. データ構造

### 8.1 リスト（基本サポート）

```seseragi
// リストリテラル（基本的な形）
let numbers = [1, 2, 3, 4, 5]
let names = ["Alice", "Bob", "Charlie"]
let empty = []
```

### 8.2 制約
- カスタム型定義（`type`）は未実装
- レコード型は未実装
- パターンマッチングは未実装

---

## 9. モナド基礎

### 9.1 Maybe型

```seseragi
// Maybe値の作成
let someValue = Just 42
let nothingValue = Nothing

// 基本的な使用
fn safeDivide x: Int -> y: Int -> Maybe<Int> = 
  if y == 0 then Nothing else Just (x / y)

let result1 = safeDivide 10 2  // Just 5
let result2 = safeDivide 10 0  // Nothing
```

### 9.2 Either型

```seseragi
// Either値の作成
let successValue = Right 42
let errorValue = Left "Error occurred"

// 基本的な使用
fn parseNumber str: String -> Either<String, Int> = 
  if str == "42" then Right 42 else Left "Invalid number"

let parsed1 = parseNumber "42"       // Right 42
let parsed2 = parseNumber "invalid"  // Left "Invalid number"
```

### 9.3 制約
- モナド演算子（`>>=`, `<$>`, `<*>`）は未実装
- do記法は未実装
- カスタムモナドは未実装

---

## 10. CLI機能

### 10.1 利用可能なコマンド

#### 10.1.1 コンパイル
```bash
# SeseragiファイルをTypeScriptにトランスパイル
seseragi compile input.ssrg --output output.ts
```

#### 10.1.2 実行
```bash
# Seseragiファイルを直接実行
seseragi run input.ssrg
```

#### 10.1.3 フォーマット
```bash
# コードのフォーマット
seseragi format input.ssrg --in-place
```

### 10.2 生成されるTypeScript

基本的なSeseragiコードは以下のようなTypeScriptにトランスパイルされます：

```typescript
// カリー化関数の実装
const curry = (fn: Function) => {
  return function curried(...args: any[]) {
    if (args.length >= fn.length) {
      return fn.apply(this, args);
    } else {
      return function(...args2: any[]) {
        return curried.apply(this, args.concat(args2));
      };
    }
  };
};

// Maybe型の実装
type Maybe<T> = { tag: 'Just'; value: T } | { tag: 'Nothing' };

// Either型の実装
type Either<L, R> = { tag: 'Left'; value: L } | { tag: 'Right'; value: R };
```

---

## 11. 実行例

### 11.1 基本的なプログラム

**入力ファイル (`example.ssrg`):**
```seseragi
// 基本的な変数と関数
let x = 42
let y = 3.14
let message = "Hello, Seseragi!"

fn add a: Int -> b: Int -> Int = a + b
fn greet name: String -> String = "Hello, " ++ name

// if-then-else
fn max a: Int -> b: Int -> Int = if a > b then a else b

// パイプライン演算子
fn double x: Int -> Int = x * 2
fn increment x: Int -> Int = x + 1

let result = 5 | increment | double  // 12

// Maybe型の使用
fn safeDivide x: Int -> y: Int -> Maybe<Int> = 
  if y == 0 then Nothing else Just (x / y)

let division = safeDivide 10 2  // Just 5
```

**実行:**
```bash
seseragi run example.ssrg
```

### 11.2 生成されるTypeScript

```typescript
// Generated TypeScript code
const curry = (fn: Function) => { /* カリー化実装 */ };

type Maybe<T> = { tag: 'Just'; value: T } | { tag: 'Nothing' };
const Just = <T>(value: T): Maybe<T> => ({ tag: 'Just', value });
const Nothing: Maybe<any> = { tag: 'Nothing' };

// 変数定義
const x = 42;
const y = 3.14;
const message = "Hello, Seseragi!";

// 関数定義（カリー化済み）
const add = curry((a: number, b: number): number => a + b);
const greet = curry((name: string): string => "Hello, " + name);

// その他の実装...
```

---

## 今後の実装予定

詳細な実装計画については [`ROADMAP.md`](ROADMAP.md) を参照してください。

次に実装予定の主要機能：
1. **パターンマッチング** - `match`式の実装
2. **型チェッカー** - 静的型検証
3. **モナド演算子** - `>>=`, `<$>`, `<*>`の実装
4. **カスタム型定義** - `type`キーワード

---

**最終更新**: 2025-06-25  
**動作確認済みバージョン**: 実装コミット efc0bb0
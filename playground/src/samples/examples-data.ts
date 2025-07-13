// examples-data.ts - 全サンプルファイルのデータ（セクション分割済み）

import { parseCodeSections } from "./section-parser"
import type { CodeSection } from "./section-parser"

// 実際のサンプルファイルの内容（examplesディレクトリから）
const sampleFiles = {
  "basics/01-hello-world": `// 01-hello-world.ssrg
// Seseragiプログラミング言語への最初の一歩

print "=== Hello, World! ==="

// =============================================================================
// 基本的な出力方法
// =============================================================================

print "--- 出力の基本 ---"

// print: 値をそのまま出力
print "Hello, World!"
print 42
print True

// show: 値を見やすく整形して出力
show "Hello, World!"
show 42
show True

// =============================================================================
// 関数適用演算子 ($)
// =============================================================================

print "--- 関数適用演算子 $ ---"

// 簡単な関数を定義
fn double x: Int -> Int = x * 2

// 括弧を使った書き方
show (double 5)  // 10

// $ を使った書き方
show $ double 5  // 10

// $ は括弧を減らすことができて便利
print $ double 10  // 20
show $ double 20   // 40`,

  "basics/02-types-and-variables": `// 02-types-and-variables.ssrg
// Seseragiの基本型と変数について学ぶ

print "=== 基本型と変数 ==="

// =============================================================================
// 変数の命名規則
// =============================================================================

print "--- 命名規則 ---"

// 変数名は小文字で始める
let userName = "Alice"
let userAge = 25

// アポストロフィー（'）が使える
let x = 10
let x' = 20
let result' = x + x'

// スネークケースも可能
let user_name = "Bob"
let max_count = 100

show userName    // "Alice"
show x'          // 20
show result'     // 30
show user_name   // "Bob"

// =============================================================================
// 基本型
// =============================================================================

print "--- 基本型 ---"

// Int型: 整数
let number = 42
show number  // 42

// String型: 文字列
let message = "Hello, Seseragi!"
show message  // "Hello, Seseragi!"

// Bool型: 真偽値
let flag = True
show flag  // True

// Float型: 浮動小数点数
let pi = 3.14
show pi  // 3.14

// =============================================================================
// 算術演算
// =============================================================================

print "--- 算術演算 ---"

// 基本的な算術演算子
let sum = 10 + 5
let difference = 10 - 5
let product = 6 * 7
let quotient = 20 / 4

show sum        // 15
show difference // 5
show product    // 42
show quotient   // 5

// =============================================================================
// 比較演算
// =============================================================================

print "--- 比較演算 ---"

// 比較演算子
let isEqual = 5 == 5
let isNotEqual = 5 != 3
let isGreater = 7 > 4
let isLess = 3 < 7

show isEqual    // True
show isNotEqual // True
show isGreater  // True
show isLess     // True

// =============================================================================
// 文字列操作
// =============================================================================

print "--- 文字列操作 ---"

// 文字列補間
let name = "Alice"
let greeting = \`Hello, \${name}!\`
show greeting  // "Hello, Alice!"`,

  "basics/03-functions": `// 03-functions.ssrg
// Seseragiの関数について学ぶ

print "=== 関数 ==="

// =============================================================================
// 関数の命名規則
// =============================================================================

print "--- 関数の命名規則 ---"

// 関数名も変数と同じルール
fn sayHello -> String = "Hello!"
fn getUserName -> String = "User"

// アポストロフィーも使える
fn calculate x: Int -> Int = x * 2
fn calculate' x: Int -> Int = x * 3

show $ sayHello()    // "Hello!"
show $ calculate 5   // 10
show $ calculate' 5  // 15

// =============================================================================
// 関数定義の基本構文
// =============================================================================

print "--- 関数定義の基本構文 ---"

// 基本構文: fn 関数名 引数: 型 -> 戻り値型 = 式
// 引数なしの関数は () を付けて呼び出す
fn hello -> String = "Hello world"
show $ hello()  // "Hello world"

// 関数自体を表示すると
show hello  // () = > "Hello world"

// 引数ありの関数
fn square x: Int -> Int = x * x
show $ square 5  // 25

// 文字列を扱う関数
fn greet name: String -> String = \`Hello, \${name}!\`
show $ greet "Alice"  // "Hello, Alice!"

// =============================================================================
// 型の書き方
// =============================================================================

print "--- 型の書き方 ---"

// 基本型
fn getNumber -> Int = 42
fn getMessage -> String = "Hello"
fn getFlag -> Bool = True
fn getPi -> Float = 3.14

// 関数を実行
show $ getNumber()  // 42
show $ getMessage()  // "Hello"
show $ getFlag()    // True
show $ getPi()      // 3.14

// 複数引数の型指定
fn concat first: String -> second: String -> String = first + second

show $ concat "Hello" " World"  // "Hello World"

// =============================================================================
// ブロック構文を使った関数定義
// =============================================================================

print "--- ブロック構文 ---"

// 複数行の処理が必要な場合は {} を使う（= は不要）
// ブロック内の最後の式が戻り値になる
fn blockExample x: Int -> Int {
  let doubled = x * 2
  let added = doubled + 10
  let result = added * 3
  result  // これが戻り値
}

show $ blockExample 5  // 60

// 中間計算を含む関数
fn circleArea radius: Float -> Float {
  let pi = 3.14159
  let radiusSquared = radius * radius
  pi * radiusSquared
}

show $ circleArea 5.0  // 78.53975

// 複数の変数を使う関数
fn areaCalc width: Int -> height: Int -> Int {
  let area = width * height
  let message = \`面積は \${area} です\`
  print message // 面積は 40 です
  area          // 最後の式が戻り値
}

show $ areaCalc 5 8  // 40

// =============================================================================
// カリー化された関数
// =============================================================================

print "--- カリー化された関数 ---"

// 複数の引数を持つ関数（カリー化）
fn add x: Int -> y: Int -> Int = x + y
fn multiply x: Int -> y: Int -> Int = x * y

// 部分適用
let add5 = add 5
let triple = multiply 3

show $ add5 10    // 15
show $ triple 7   // 21

// 完全適用
show $ add 3 4      // 7
show $ multiply 6 9 // 54

// =============================================================================
// 高階関数
// =============================================================================

print "--- 高階関数 ---"

// 関数を引数に取る関数
fn applyFunc func: (Int -> Int) -> value: Int -> Int = func value

// 使用する関数を定義
fn quad x: Int -> Int = x * 4
fn increment x: Int -> Int = x + 1
fn addTen x: Int -> Int = x + 10

// 高階関数を適用
show $ applyFunc quad 5         // 20
show $ applyFunc increment 20   // 21
show $ applyFunc addTen 5       // 15

// =============================================================================
// 実用的な関数例
// =============================================================================

print "--- 実用的な関数例 ---"

// 数値計算の関数
fn power base: Int -> exp: Int -> Int =
  exp == 0 ? 1 : base * power base (exp - 1)

// 階乗を求める関数
fn factorial n: Int -> Int =
  n <= 1 ? 1 : n * factorial (n - 1)

// 文字列を繰り返す関数
fn repeatStr str: String -> times: Int -> String =
  times <= 0 ? "" : \`\${str}\${repeatStr str (times - 1)}\`

show $ power 2 3          // 8
show $ factorial 5        // 120
show $ repeatStr "Hi " 3  // "Hi Hi Hi "

// 実行方法:
// seseragi run examples/basics/03-functions.ssrg`,

  "basics/04-conditionals": `// 04-conditionals.ssrg
// Seseragiの条件分岐について学ぶ

print "=== 条件分岐 ==="

// =============================================================================
// 基本的な条件分岐
// =============================================================================

print "--- 基本的な条件分岐 ---"

// if-then-else式
let x = 10
let result = if x > 5 then "大きい" else "小さい"
show result  // "大きい"

// ブール値での条件分岐
let isTrue = True
let message = if isTrue then "真です" else "偽です"
show message  // "真です"

let isFalse = False
let message' = if isFalse then "真です" else "偽です"
show message'  // "偽です"

// =============================================================================
// 比較演算子
// =============================================================================

print "--- 比較演算子 ---"

let a = 5
let b = 3

show $ a == b  // False (等しい)
show $ a != b  // True (等しくない)
show $ a > b   // True (より大きい)
show $ a < b   // False (より小さい)
show $ a >= b  // True (以上)
show $ a <= b  // False (以下)

// =============================================================================
// ブール演算子
// =============================================================================

print "--- ブール演算子 ---"

let p = True
let q = False

show $ p && q  // False (AND演算)
show $ p || q  // True (OR演算)
show $ !p      // False (NOT演算)

// =============================================================================
// 複合条件の書き方比較
// =============================================================================

print "--- 複合条件の書き方比較 ---"

// if-then-else チェーン
fn classify age: Int -> String =
  if age < 13 then "子供"
  else if age < 20 then "ティーンエイジャー"
  else if age < 65 then "大人"
  else "高齢者"

// 三項演算子チェーン
fn classify' age: Int -> String =
  age < 13 ? "子供" :
  age < 20 ? "ティーンエイジャー" :
  age < 65 ? "大人" :
  "高齢者"

show $ classify 16   // "ティーンエイジャー"
show $ classify' 16  // "ティーンエイジャー"

show $ classify 10   // "子供"
show $ classify' 30  // "大人"

// =============================================================================
// 実用的な条件分岐例
// =============================================================================

print "--- 実用的な例 ---"

// 偶数・奇数判定
fn isEven x: Int -> Bool = x % 2 == 0
fn isOdd x: Int -> Bool = x % 2 != 0

show $ isEven 4   // True
show $ isOdd 4    // False

// 成績判定
fn grade score: Int -> String =
  score >= 90 ? "A" :
  score >= 80 ? "B" :
  score >= 70 ? "C" :
  score >= 60 ? "D" :
  "F"

show $ grade 95   // "A"
show $ grade 75   // "C"
show $ grade 45   // "F"

// 実行方法:
// seseragi run examples/basics/04-conditionals.ssrg`,

  "intermediate/01-maybe-basics": `// 01-maybe-basics.ssrg
// Maybe型の基本を学ぶ

print "=== Maybe型の基本 ==="

// =============================================================================
// Maybe型とは
// =============================================================================

print "--- Maybe型とは ---"

// Maybe型は「値があるかもしれない」を表現する型
// Just: 値がある場合
// Nothing: 値がない場合

let someValue = Just 42
let nothingValue = Nothing

show someValue    // Just 42
show nothingValue // Nothing

// =============================================================================
// 基本的なMaybe値の操作
// =============================================================================

print "--- 基本的な操作 ---"

// 異なる型のMaybe値
let maybeNumber = Just 100
let maybeString = Just "Hello"
let maybeBoolean = Just True
let empty = Nothing

show maybeNumber   // Just 100
show maybeString   // Just "Hello"
show maybeBoolean  // Just True
show empty         // Nothing

// =============================================================================
// Maybe型を使った安全な操作
// =============================================================================

print "--- 安全な操作 ---"

// 数値を月名に変換
fn numberToMonth n: Int -> Maybe<String> =
  n == 1 ? Just "January" :
  n == 2 ? Just "February" :
  n == 3 ? Just "March" :
  n == 4 ? Just "April" :
  n == 5 ? Just "May" :
  n == 6 ? Just "June" :
  n == 7 ? Just "July" :
  n == 8 ? Just "August" :
  n == 9 ? Just "September" :
  n == 10 ? Just "October" :
  n == 11 ? Just "November" :
  n == 12 ? Just "December" :
  Nothing

show $ numberToMonth 0   // Nothing
show $ numberToMonth 1   // Just "January"
show $ numberToMonth 12  // Just "December"
show $ numberToMonth 13  // Nothing

// 安全な配列インデックスアクセス（概念的な例）
fn safeGet index: Int -> Maybe<Int> =
  index < 0 ? Nothing :
  index >= 10 ? Nothing :
  Just (index * 10)

show $ safeGet 2   // Just 20
show $ safeGet -1  // Nothing
show $ safeGet 15  // Nothing

// =============================================================================
// Maybe型の実用例
// =============================================================================

print "--- 実用例 ---"

// 年齢を文字列から安全に解析
fn parseAge input: String -> Maybe<Int> =
  input == "25" ? Just 25 :
  input == "30" ? Just 30 :
  Nothing

let age1 = parseAge "25"  // Just 25
let age2 = parseAge "abc" // Nothing

show age1  // Just 25
show age2  // Nothing

// 検索結果の表現
fn findUser id: Int -> Maybe<String> =
  id == 1 ? Just "Alice" :
  id == 2 ? Just "Bob" :
  Nothing

let user1 = findUser 1  // Just "Alice"
let user2 = findUser 99 // Nothing

show user1  // Just "Alice"
show user2  // Nothing

// =============================================================================
// Maybe型の利点
// =============================================================================

print "--- Maybe型の利点 ---"

// 1. nullポインタエラーの回避
// 2. 存在しない値を明示的に表現
// 3. 型安全性の向上
// 4. エラーハンドリングの改善

print "Maybe型により、値の存在/非存在を型レベルで表現できます"

// 実行方法:
// seseragi run examples/intermediate/01-maybe-basics.ssrg`,

  "intermediate/02-either-basics": `// 02-either-basics.ssrg
// Either型の基本を学ぶ

print "=== Either型の基本 ==="

// =============================================================================
// Either型とは
// =============================================================================

print "--- Either型とは ---"

// Either型は「成功または失敗」を表現する型
// Right: 成功した場合の値
// Left: 失敗した場合の値（通常はエラー）

let successValue = Right 42
let errorValue = Left "Something went wrong"

show successValue  // Right 42
show errorValue    // Left "Something went wrong"

// =============================================================================
// 基本的なEither値の操作
// =============================================================================

print "--- 基本的な操作 ---"

// 異なる型のEither値
let successNumber = Right 100
let successString = Right "Hello"
let errorMessage = Left "エラーが発生しました"
let anotherError = Left "別のエラー"

show successNumber  // Right 100
show successString  // Right "Hello"
show errorMessage   // Left "エラーが発生しました"
show anotherError   // Left "別のエラー"

// =============================================================================
// Either型を使った安全な操作
// =============================================================================

print "--- 安全な操作 ---"

// 安全な除算関数（エラーメッセージ付き）
fn safeDivide x: Int -> y: Int -> Either<String, Int> =
  y == 0 ? Left "ゼロ除算エラー" : Right (x / y)

let result1 = safeDivide 10 2  // Right 5
let result2 = safeDivide 10 0  // Left "ゼロ除算エラー"

show result1  // Right 5
show result2  // Left "ゼロ除算エラー"

// 安全な文字列解析
fn parseInt input: String -> Either<String, Int> =
  input == "42" ? Right 42 :
  input == "100" ? Right 100 :
  Left \`"\${input}"は数値ではありません\`

show $ parseInt "42"   // Right 42
show $ parseInt "abc"  // Left "abcは数値ではありません"

// =============================================================================
// Either型の実用例
// =============================================================================

print "--- 実用例 ---"

// ユーザー検索（詳細なエラーメッセージ付き）
fn findUser id: Int -> Either<String, String> =
  id == 1 ? Right "Alice" :
  id == 2 ? Right "Bob" :
  Left \`ユーザーID \${id} は存在しません\`

let user1 = findUser 1   // Right "Alice"
let user2 = findUser 99  // Left "ユーザーID 99 は存在しません"

show user1  // Right "Alice"
show user2  // Left "ユーザーID 99 は存在しません"

// ファイル読み込み（概念的な例）
fn readFile filename: String -> Either<String, String> =
  filename == "config.txt" ? Right "設定内容" :
  filename == "data.txt" ? Right "データ内容" :
  Left \`ファイル "\${filename}" が見つかりません\`

show $ readFile "config.txt"   // Right "設定内容"
show $ readFile "missing.txt"  // Left "ファイル \"missing.txt\" が見つかりません"

// =============================================================================
// バリデーション例
// =============================================================================

print "--- バリデーション例 ---"

// 年齢のバリデーション
fn validateAge age: Int -> Either<String, Int> =
  age < 0 ? Left "年齢は負数にできません" :
  age > 150 ? Left "年齢が無効です" :
  Right age

show $ validateAge 25   // Right 25
show $ validateAge -5   // Left "年齢は負数にできません"
show $ validateAge 200  // Left "年齢が無効です"

// =============================================================================
// Either型の利点
// =============================================================================

print "--- Either型の利点 ---"

// 1. エラーハンドリングの改善
// 2. 詳細なエラーメッセージの提供
// 3. 型安全性の向上
// 4. 例外処理の代替手段

print "Either型により、エラーを値として扱えます"

// 実行方法:
// seseragi run examples/intermediate/02-either-basics.ssrg`,

  "intermediate/03-lists-and-arrays": `// 03-lists-and-arrays.ssrg
// ListとArrayの基本を学ぶ

print "=== ListとArrayの基本 ==="

// =============================================================================
// Array型
// =============================================================================

print "--- Array型 ---"

// 基本的な配列
let numbers = [1, 2, 3, 4, 5]
let strings = ["hello", "world", "seseragi"]
let booleans = [True, False, True]
let empty = []

show numbers   // [1, 2, 3, 4, 5]
show strings   // ["hello", "world", "seseragi"]
show booleans  // [True, False, True]
show empty     // []

// 配列のインデックスアクセス
show numbers[0]  // 1
show numbers[2]  // 3
show strings[1]  // "world"

// =============================================================================
// List型
// =============================================================================

print "--- List型 ---"

// 空のリスト
let emptyList = Empty
let emptyList' = \`[]
show emptyList   // Empty
show emptyList'  // \`[]

// 要素を持つリスト（Cons構築子使用）
let singletonList = Cons 42 Empty
show singletonList  // Cons 42 Empty

// cons演算子 : を使用
let singletonList' = 42 : \`[]
show singletonList'  // 42 : \`[]

// シュガー構文
let singletonList'' = \`[42]
show singletonList''  // \`[42]

// 複数要素のリスト
let list1 = Cons 1 (Cons 2 (Cons 3 Empty))
let list2 = 1 : 2 : 3 : \`[]
let list3 = \`[1, 2, 3]
show list1  // Cons 1 (Cons 2 (Cons 3 Empty))
show list2  // 1 : 2 : 3 : \`[]
show list3  // \`[1, 2, 3]

// 文字列のリスト
let stringList = \`["hello", "world", "seseragi"]
show stringList  // \`["hello", "world", "seseragi"]

// headとtailの演算子
let numbers' = \`[10, 20, 30, 40]
show $ head numbers'  // Just 10
show $ tail numbers'  // \`[20, 30, 40]

// 演算子形式
show $ ^numbers'      // Just 10 (head演算子)
show $ >>numbers'     // \`[20, 30, 40] (tail演算子)

// 演算子の連結
show $ ^>>numbers'    // Just 20 (tailのhead)
show $ >>.>>numbers'  // \`[30, 40] (tailのtail)

// より複雑な連結
show $ ^>>.>>numbers' // Just 30 (3番目の要素)
show $ ^>>.>>.>>numbers' // Just 40 (4番目の要素)

// 空リストでのhead/tail
let emptyList'' = \`[]
show $ ^emptyList''   // Nothing
show $ >>emptyList''  // \`[]

// =============================================================================
// Array↔List変換
// =============================================================================

print "--- Array↔List変換 ---"

// 配列をリストに変換
let arr: Array<Int> = [1, 2, 3, 4, 5]
let list: List<Int> = arrayToList arr
show arr   // [1, 2, 3, 4, 5]
show list  // \`[1, 2, 3, 4, 5]

// リストを配列に変換
let backToArray: Array<Int> = listToArray list
show backToArray  // [1, 2, 3, 4, 5]

// 空配列の変換
let emptyArr: Array<Int> = []
let emptyListFromArray: List<Int> = arrayToList emptyArr
let emptyArrFromList: Array<Int> = listToArray emptyListFromArray

show emptyArr            // []
show emptyListFromArray  // \`[]
show emptyArrFromList    // []

// =============================================================================
// 内包表記
// =============================================================================

print "--- Array内包表記 ---"

// 基本的な内包表記
let squares = [x * x | x <- [1, 2, 3, 4, 5]]
show squares  // [1, 4, 9, 16, 25]

// 範囲演算子
let range1 = 1..5   // 末尾含まず
let range2 = 1..=5  // 末尾含む
show range1  // [1, 2, 3, 4]
show range2  // [1, 2, 3, 4, 5]

// 範囲演算子を使った内包表記
let squaresRange = [x * x | x <- 1..=5]
show squaresRange  // [1, 4, 9, 16, 25]

// 条件付き内包表記
let evenSquares = [x * x | x <- 1..=6, x % 2 == 0]
show evenSquares  // [4, 16, 36]

// 複数のジェネレータ
let pairs = [(x, y) | x <- 1..=2, y <- 3..=4]
show pairs  // [(1, 3), (1, 4), (2, 3), (2, 4)]

print "--- List内包表記 ---"

// 基本的な内包表記
let squares' = \`[x * x | x <- [1, 2, 3, 4, 5]]
show squares'  // \`[1, 4, 9, 16, 25]

// 範囲演算子を使った内包表記
let squaresRange' = \`[x * x | x <- 1..=5]
show squaresRange'  // \`[1, 4, 9, 16, 25]

// 条件付き内包表記
let evenSquares' = \`[x * x | x <- 1..=6, x % 2 == 0]
show evenSquares'  // \`[4, 16, 36]

// 文字列の内包表記
let greetings = \`[\`Hello, \${name}!\` | name <- ["Alice", "Bob", "Charlie"]]
show greetings  // \`["Hello, Alice!", "Hello, Bob!", "Hello, Charlie!"]

// ガード付き複数ジェネレータ
let filtered = \`[x + y | x <- 1..=3, y <- 4..=6, x + y > 6]
show filtered  // \`[7, 8, 8, 9]

// =============================================================================
// ArrayとListの使い分け
// =============================================================================

print "--- ArrayとListの使い分け ---"

// Array型の特徴：
// - JavaScriptの配列と互換性がある
// - インデックスアクセスが効率的
// - 可変長

// List型の特徴：
// - 関数プログラミングに適している
// - 再帰的な処理に適している
// - 不変性

print "Array型はJavaScriptとの互換性、List型は関数プログラミングに適しています"


// 実行方法:
// seseragi run examples/intermediate/03-lists-and-arrays.ssrg`,

  "intermediate/04-operators": `// 04-operators.ssrg
// モナド演算子の基本を学ぶ

print "=== モナド演算子の基本 ==="

// =============================================================================
// ファンクター演算子 (<$>)
// =============================================================================

print "--- ファンクター演算子 <$> ---"

// 通常の関数を定義
fn double x: Int -> Int = x * 2
fn increment x: Int -> Int = x + 1

// Maybe型に対するファンクター
let maybeValue = Just 42
let nothingValue = Nothing

let doubled = double <$> maybeValue
let doubled' = double <$> nothingValue

show doubled   // Just 84
show doubled'  // Nothing

// Either型に対するファンクター
let rightValue = Right 10
let leftValue = Left "error"

let incremented = increment <$> rightValue
let incremented' = increment <$> leftValue

show incremented   // Right 11
show incremented'  // Left "error"

// =============================================================================
// アプリカティブ演算子 (<*>)
// =============================================================================

print "--- アプリカティブ演算子 <*> ---"

// 複数引数の関数
fn add x: Int -> y: Int -> Int = x + y
fn multiply x: Int -> y: Int -> Int = x * y

// Maybe型でのアプリカティブ
let value1 = Just 10
let value2 = Just 5
let value3 = Nothing

let sum = Just add <*> value1 <*> value2
let sum' = Just add <*> value1 <*> value3

show sum   // Just 15
show sum'  // Nothing

// Either型でのアプリカティブ
let right1 = Right 20
let right2 = Right 3
let left1 = Left "first error"

let product = Right multiply <*> right1 <*> right2
let product' = Right multiply <*> left1 <*> right2

show product   // Right 60
show product'  // Left "first error"

// =============================================================================
// モナド演算子 (>>=)
// =============================================================================

print "--- モナド演算子 >>= ---"

// Maybe型を返す関数
fn doubleIfEven x: Int -> Maybe<Int> =
  x % 2 == 0 ? Just (x * 2) : Nothing

fn addTen x: Int -> Maybe<Int> =
  x > 0 ? Just (x + 10) : Nothing

// モナド演算子の使用
let result1 = Just 4 >>= doubleIfEven
let result2 = Just 3 >>= doubleIfEven

show result1  // Just 8
show result2  // Nothing

// モナド演算子の連鎖
let chain1 = Just 2
  >>= doubleIfEven
  >>= addTen
let chain2 = Just 3
  >>= doubleIfEven
  >>= addTen

show chain1  // Just 14
show chain2  // Nothing

// Either型でのモナド演算子
fn validatePositive x: Int -> Either<String, Int> =
  x > 0 ? Right x: Left "値は正数である必要があります"

fn validateEven x: Int -> Either<String, Int> =
  x % 2 == 0 ? Right x: Left "値は偶数である必要があります"

let valid1 = Right 8
  >>= validatePositive
  >>= validateEven
let valid2 = Right 7
  >>= validatePositive
  >>= validateEven
let valid3 = Right -2
  >>= validatePositive
  >>= validateEven

show valid1  // Right 8
show valid2  // Left "値は偶数である必要があります"
show valid3  // Left "値は正数である必要があります"

// 配列とリストでのモナド演算子
let arr = [1, 2, 3]
let mapped = (\\x -> x * 2) <$> arr
show mapped  // [2, 4, 6]

let lst = \`[1, 2, 3]
let transformed = (\\x -> x * 3) <$> lst
show transformed  // \`[3, 6, 9]

fn repeat x: Int -> Array<Int> = [x, x]
let flattened = arr >>= repeat
show flattened  // [1, 1, 2, 2, 3, 3]

fn listRepeat x: Int -> List<Int> = \`[x, x]
let listFlattened = lst >>= listRepeat
show listFlattened  // \`[1, 1, 2, 2, 3, 3]

// =============================================================================
// 実用的な例
// =============================================================================

print "--- 実用的な例 ---"

// 変数名にアポストロフィーを使った例
let x' = Just 100
let y' = Just 25

fn subtract x: Int -> y: Int -> Int = x - y

let result' = Just subtract <*> x' <*> y'
show result'  // Just 75

// <$>: モナド値に関数適用
let value = Just 100
let value' = toString <$> value
show value'  // Just "100"

// <*>: 複数のモナド値を受け取って関数適用
fn createEmail name: String -> domain: String -> String = \`\${name}@\${domain}\`
let nameValue = Just "user"
let domainValue = Just "example.com"
let email = Just createEmail <*> nameValue <*> domainValue
show email  // Just "user@example.com"

// >>=: モナド値で処理を連結
fn parsePort input: String -> Maybe<Int> =
  input == "80" ? Just 80 :
  input == "443" ? Just 443 :
  Nothing

fn validatePort port: Int -> Maybe<String> =
  port == 80 ? Just "HTTP" :
  port == 443 ? Just "HTTPS" :
  Nothing

let result = Just "80"
  >>= parsePort
  >>= validatePort
show result  // Just "HTTP"

// =============================================================================
// 演算子の利点
// =============================================================================

print "--- 演算子の利点 ---"

// 1. 関数型プログラミングの表現力向上
// 2. エラーハンドリングの簡潔化
// 3. 型安全性の維持
// 4. 合成可能な操作

print "モナド演算子により、安全で表現力豊かなコードが書けます"

// 実行方法:
// seseragi run examples/intermediate/04-operators.ssrg`,

  "advanced/01-pattern-matching": `// 01-pattern-matching.ssrg
// パターンマッチングの基本を学ぶ

print "=== パターンマッチング ==="

// =============================================================================
// リテラルパターン
// =============================================================================

print "--- リテラルパターン ---"

// 数値リテラルパターン
fn describeNumber n: Int -> String = match n {
  0 -> "ゼロ"
  1 -> "一"
  42 -> "答え"
  _ -> "その他の数"
}

show $ describeNumber 0   // "ゼロ"
show $ describeNumber 42  // "答え"
show $ describeNumber 99  // "その他の数"

// 文字列リテラルパターン
fn greetByLanguage lang: String -> String = match lang {
  "jp" -> "こんにちは"
  "en" -> "Hello"
  "fr" -> "Bonjour"
  _ -> "Unknown language"
}

show $ greetByLanguage "jp"  // "こんにちは"
show $ greetByLanguage "en"  // "Hello"
show $ greetByLanguage "de"  // "Unknown language"

// =============================================================================
// タプルパターン
// =============================================================================

print "--- タプルパターン ---"

// 座標のパターンマッチング
fn describePoint point: (Int, Int) -> String = match point {
  (0, 0) -> "原点"
  (0, _) -> "Y軸上"
  (_, 0) -> "X軸上"
  (x, y) -> \`点(\${x}, \${y})\`
}

show $ describePoint $ (0, 0)  // "原点"
show $ describePoint $ (0, 5)  // "Y軸上"
show $ describePoint $ (3, 0)  // "X軸上"
show $ describePoint $ (2, 3)  // "点(2, 3)"

// 3つ組のパターンマッチング
fn analyzeTriple triple: (Int, Int, Int) -> String = match triple {
  (0, 0, 0) -> "すべてゼロ"
  (x, y, z) when x == y && y == z -> \`すべて同じ値: \${x}\`
  (x, y, z) -> \`異なる値: \${x}, \${y}, \${z}\`
}

show $ analyzeTriple $ (0, 0, 0)  // "すべてゼロ"
show $ analyzeTriple $ (5, 5, 5)  // "すべて同じ値: 5"
show $ analyzeTriple $ (1, 2, 3)  // "異なる値: 1, 2, 3"

// =============================================================================
// 基本的なADT（代数データ型）
// =============================================================================

print "--- 基本的なADT ---"

// 列挙型の定義
type Color =
  | Red
  | Green
  | Blue

let red = Red
let green = Green
let blue = Blue

// パターンマッチング
let colorName = match red {
  Red -> "赤"
  Green -> "緑"
  Blue -> "青"
}

show colorName  // "赤"

// 関数でのパターンマッチング
fn colorToString color: Color -> String = match color {
  Red -> "赤色"
  Green -> "緑色"
  Blue -> "青色"
}

show $ colorToString green  // "緑色"

// =============================================================================
// 引数付きADT
// =============================================================================

print "--- 引数付きADT ---"

// 引数を持つ代数データ型
type Shape =
  | Circle Float
  | Rectangle Float Float

let circle = Circle 5.0
let rectangle = Rectangle 3.0 4.0

// パターンマッチングで計算
let area = match circle {
  Circle radius -> radius * radius * 3.14
  Rectangle width height -> width * height
}

show area  // 78.5

// より複雑なパターンマッチング
fn calculateArea shape: Shape -> Float = match shape {
  Circle radius -> radius * radius * 3.14
  Rectangle width height -> width * height
}

show $ calculateArea circle     // 78.5
show $ calculateArea rectangle  // 12.0

// =============================================================================
// ビルトインMaybe型
// =============================================================================

print "--- ビルトインMaybe型 ---"

let just42 = Just 42
let nothing = Nothing

let maybeValue = match just42 {
  Nothing -> 0
  Just value -> value * 2
}

show maybeValue  // 84

// Maybe型の安全な操作
fn unwrapOr maybe: Maybe<Int> -> defaultValue: Int -> Int = match maybe {
  Nothing -> defaultValue
  Just value -> value
}

show $ unwrapOr just42 0   // 42
show $ unwrapOr nothing 0  // 0

// =============================================================================
// ビルトインEither型
// =============================================================================

print "--- ビルトインEither型 ---"

let right100 = Right 100
let leftError = Left "エラー"

let eitherValue = match right100 {
  Left msg -> 0
  Right value -> value + 10
}

show eitherValue  // 110

// Either型のエラーハンドリング
fn handleResult result: Either<String, Int> -> String = match result {
  Left error -> \`エラー: \${error}\`
  Right value -> \`成功: \${value}\`
}

show $ handleResult right100  // "成功: 100"
show $ handleResult leftError // "エラー: エラー"

// =============================================================================
// ビルトインList型
// =============================================================================

print "--- ビルトインList型 ---"

let myList = Cons 1 (Cons 2 (Cons 3 Empty))
let emptyList = Empty

let listHead = match myList {
  Empty -> -1
  Cons h t -> h
}

show listHead  // 1

// リストの再帰的な処理
fn listLength list: List<Int> -> Int = match list {
  Empty -> 0
  Cons h t -> 1 + listLength t
}

show $ listLength myList     // 3
show $ listLength emptyList  // 0

// =============================================================================
// 複雑なパターンマッチング例
// =============================================================================

print "--- 複雑な例 ---"

// Guardパターン（when）
type Result =
  | Success Int
  | Failure String

fn processResult result: Result -> String = match result {
  Success value when value == 0 -> "ゼロです"
  Success value when value > 0 -> \`正の値: \${value}\`
  Success value -> \`負の値: \${value}\`
  Failure msg -> \`失敗: \${msg}\`
}

let success = Success 42
let zero = Success 0
let failure = Failure "エラーが発生しました"

show $ processResult success  // "正の値: 42"
show $ processResult zero     // "ゼロです"
show $ processResult failure  // "失敗: エラーが発生しました"

// Orパターン（|）
fn classifyCharacter char: String -> String = match char {
  "a" | "e" | "i" | "o" | "u" -> "母音"
  "y" -> "半母音"
  _ -> "子音"
}

show $ classifyCharacter "a"  // "母音"
show $ classifyCharacter "b"  // "子音"
show $ classifyCharacter "y"  // "半母音"

// 配列パターン
fn analyzeArray arr: Array<Int> -> String = match arr {
  [] -> "空の配列"
  [a] -> \`要素1つ: \${a}\`
  [a, b] -> \`要素2つ: \${a}, \${b}\`
  [a, b, c] -> \`要素3つ: \${a}, \${b}, \${c}\`
  _ -> "4つ以上の要素"
}

show $ analyzeArray $ []        // "空の配列"
show $ analyzeArray $ [1]       // "要素1つ: 1"
show $ analyzeArray $ [1, 2]    // "要素2つ: 1, 2"
show $ analyzeArray $ [1, 2, 3] // "要素3つ: 1, 2, 3"
show $ analyzeArray $ [1, 2, 3, 4] // "4つ以上の要素"

// =============================================================================
// パターンマッチングの利点
// =============================================================================

print "--- パターンマッチングの利点 ---"

// 1. 型安全性の向上
// 2. 網羅性のチェック
// 3. 表現力の向上
// 4. 関数型プログラミングの核心機能

print "パターンマッチングにより、型安全で表現力豊かなコードが書けます"

// 実行方法:
// seseragi run examples/advanced/01-pattern-matching.ssrg`,

  "advanced/02-structs-and-methods": `// 02-structs-and-methods.ssrg
// 構造体とメソッドについて学ぶ

print "=== 構造体とメソッド ==="

// =============================================================================
// 構造体の定義
// =============================================================================

print "--- 構造体の定義 ---"

// 基本的な構造体定義
struct Person {
  name: String,
  age: Int
}

struct Point {
  x: Float,
  y: Float
}

// より複雑な構造体
struct Employee {
  id: Int,
  name: String,
  department: String,
  salary: Int,
  isActive: Bool
}

print "--- 構造体のインスタンス化 ---"

// 構造体インスタンスの作成
let alice = Person { name: "Alice", age: 30 }
let bob = Person { name: "Bob", age: 25 }

show alice  // Person { name: "Alice", age: 30 }
show bob    // Person { name: "Bob", age: 25 }

// Point構造体のインスタンス
let origin = Point { x: 0.0, y: 0.0 }
let point1 = Point { x: 3.0, y: 4.0 }
let point2 = Point { x: 1.0, y: 2.0 }

show origin  // Point { x: 0.0, y: 0.0 }
show point1  // Point { x: 3.0, y: 4.0 }
show point2  // Point { x: 1.0, y: 2.0 }

print "--- フィールドアクセス ---"

// ドット記法でフィールドにアクセス
let aliceName = alice.name
let aliceAge = alice.age

show aliceName  // "Alice"
show aliceAge   // 30

// Pointのフィールドアクセス
let x1 = point1.x
let y1 = point1.y

show x1  // 3.0
show y1  // 4.0

// =============================================================================
// 構造体のメソッド実装
// =============================================================================

print "--- 構造体のメソッド実装 ---"

// Point構造体にメソッドを実装
struct Vector {
  x: Int,
  y: Int
}

impl Vector {
  // メソッドの定義
  fn add self -> other: Vector -> Vector {
    let x = self.x + other.x
    let y = self.y + other.y
    Vector { x, y }
  }

  fn subtract self -> other: Vector -> Vector {
    let x = self.x - other.x
    let y = self.y - other.y
    Vector { x, y }
  }

  fn scale self -> factor: Int -> Vector {
    let x = self.x * factor
    let y = self.y * factor
    Vector { x, y }
  }

  fn distanceSquared self -> Int =
    self.x * self.x + self.y * self.y

  // 演算子オーバーロード
  operator + self -> other -> Vector = self add other
  operator - self -> other: Vector -> Vector = self subtract other
  operator * self -> factor: Int -> Vector = self scale factor
}

let v1 = Vector { x: 3, y: 4 }
let v2 = Vector { x: 1, y: 2 }

show v1  // Vector { x: 3, y: 4 }
show v2  // Vector { x: 1, y: 2 }

print "--- メソッドの呼び出し ---"

// メソッドの呼び出し
let sum = v1 add v2
let diff = v1 subtract v2
let scaled = v1 scale 3
let distance = v1 distanceSquared()

show sum       // Vector { x: 4, y: 6 }
show diff      // Vector { x: 2, y: 2 }
show scaled    // Vector { x: 9, y: 12 }
show distance  // 25

// 演算子のオーバーロード
show $ v1 + v2 // Vector { x: 4, y: 6 }
show $ v1 - v2 // Vector { x: 2, y: 2 }
show $ v1 * 3  // Vector { x: 9, y: 12 }

// =============================================================================
// 実用的な構造体例
// =============================================================================

print "--- 実用的な例 ---"

// 銀行口座の構造体
struct Account {
  id: Int,
  owner: String,
  balance: Int
}

impl Account {
  fn getInfo self -> String {
    let { id, owner, balance } = self
    \`口座ID: \${id}, 所有者: \${owner}, 残高: \${balance}\`
  }

  fn deposit self -> amount: Int -> Account {
    Account { ...self, balance: self.balance + amount }
  }

  fn withdraw self -> amount: Int -> Either<String, Account> {
    let balance = self.balance - amount
    if amount <= balance
    then Right (Account { ...self, balance })
    else Left \`残高不足です。現在の残高: \${self.balance}, 出金額: \${amount}\`
  }
}

let account = Account { id: 1001, owner: "Alice", balance: 10000 }
show account  // Account { id: 1001, owner: "Alice", balance: 10000 }

let account' = account deposit 5000
show account'  // Account { id: 1001, owner: "Alice", balance: 15000 }

let withdrawResult = account' withdraw 3000
show withdrawResult  // Right Account { id: 1001, owner: "Alice", balance: 12000 }

let failedWithdraw = account' withdraw 20000
show failedWithdraw  // Left "残高不足です。現在の残高: 15000, 出金額: 20000"

// 成功時の処理
let account'' = match withdrawResult {
  Right acc -> acc
  Left _ -> account'
}
show $ account'' getInfo()     // "口座ID: 1001, 所有者: Alice, 残高: 12000"

// =============================================================================
// 構造体の利点
// =============================================================================

print "--- 構造体の利点 ---"

// 1. データの構造化
// 2. 型安全性
// 3. メソッドによる操作の関連付け
// 4. コードの可読性向上

print "構造体により、関連するデータとメソッドを一箇所にまとめられます"

// 実行方法:
// seseragi run examples/advanced/02-structs-and-methods.ssrg`,

  "advanced/03-monad-composition": `// 03-monad-composition.ssrg
// モナドの合成について学ぶ

print "=== モナドの合成 ==="

// =============================================================================
// 複数のMaybe値の合成
// =============================================================================

print "--- Maybe値の合成 ---"

// Maybe値の基本的な合成
let maybeX = Just 10
let maybeY = Just 20
let maybeZ = Nothing

fn add x: Int -> y: Int -> Int = x + y

// アプリカティブ演算子での合成
let result1 = Just add <*> maybeX <*> maybeY
let result2 = Just add <*> maybeX <*> maybeZ

show result1  // Just 30
show result2  // Nothing

// モナド演算子での合成（処理単位を組み合わせる）
fn doubleIfPositive x: Int -> Maybe<Int> =
  if x > 0 then Just (x * 2) else Nothing

fn addTen x: Int -> Maybe<Int> =
  if x < 100 then Just (x + 10) else Nothing

// 処理単位を組み合わせたパイプライン
let result3 = maybeX
  >>= doubleIfPositive
  >>= addTen

let result4 = maybeZ
  >>= doubleIfPositive
  >>= addTen

show result3  // Just 30
show result4  // Nothing

// =============================================================================
// Either値の合成
// =============================================================================

print "--- Either値の合成 ---"

// Either値の基本的な合成
let rightX = Right 15
let rightY = Right 25
let leftError = Left "エラーが発生しました"

fn multiply x: Int -> y: Int -> Int = x * y

// アプリカティブ演算子での合成
let eitherResult1 = Right multiply <*> rightX <*> rightY
let eitherResult2 = Right multiply <*> rightX <*> leftError

show eitherResult1  // Right 375
show eitherResult2  // Left "エラーが発生しました"

// モナド演算子での合成（処理単位を組み合わせる）
fn multiplyBy2 x: Int -> Either<String, Int> =
  if x > 0 then Right (x * 2) else Left "値は正数である必要があります"

fn subtractFive x: Int -> Either<String, Int> =
  if x >= 5 then Right (x - 5) else Left "値が小さすぎます"

// 処理単位を組み合わせたパイプライン
let eitherResult3 = rightX
  >>= multiplyBy2
  >>= subtractFive

let eitherResult4 = Right 0
  >>= multiplyBy2
  >>= subtractFive

let eitherResult5 = Right 2
  >>= multiplyBy2
  >>= subtractFive

show eitherResult3  // Right 25
show eitherResult4  // Left "値は正数である必要があります"
show eitherResult5  // Left "値が小さすぎます"

// =============================================================================
// 複雑な計算のチェーン
// =============================================================================

print "--- 複雑な計算のチェーン ---"

// 実用的な処理関数群
fn parseUserId input: String -> Maybe<Int> = match input {
  "user123" -> Just 123
  "admin456" -> Just 456
  "guest789" -> Just 789
  _ -> Nothing
}

fn checkPermission userId: Int -> Maybe<String> = match userId {
  123 -> Just "read"
  456 -> Just "admin"
  789 -> Just "guest"
  _ -> Nothing
}

fn accessResource permission: String -> Maybe<String> = match permission {
  "read" -> Just "データを読み取りました"
  "admin" -> Just "管理者権限でアクセスしました"
  "guest" -> Just "ゲストとしてアクセスしました"
  _ -> Nothing
}

// 認証・認可の処理チェーン
let accessResult1 = Just "user123"
  >>= parseUserId
  >>= checkPermission
  >>= accessResource

let accessResult2 = Just "admin456"
  >>= parseUserId
  >>= checkPermission
  >>= accessResource

let accessResult3 = Just "unknown"
  >>= parseUserId
  >>= checkPermission
  >>= accessResource

show accessResult1  // Just "データを読み取りました"
show accessResult2  // Just "管理者権限でアクセスしました"
show accessResult3  // Nothing

// =============================================================================
// バリデーションの合成
// =============================================================================

print "--- バリデーションの合成 ---"

// バリデーション関数群
fn validateAge age: Int -> Either<String, Int> = match age {
  n when n < 0 -> Left "年齢は負数にできません"
  n when n > 150 -> Left "年齢が無効です"
  n -> Right n
}

fn validateName name: String -> Either<String, String> = match name {
  "" -> Left "名前が空です"
  "admin" -> Left "予約語は使用できません"
  "root" -> Left "予約語は使用できません"
  n -> Right n
}

fn validateEmail email: String -> Either<String, String> = match email {
  "" -> Left "メールアドレスが空です"
  e when e == "test@test.com" -> Left "テストアドレスは使用できません"
  e -> Right e
}

// 複数のバリデーションを組み合わせる構造体
struct User {
  name: String,
  age: Int,
  email: String
}

// ユーザー作成関数（カリー化）
fn createUser name: String -> age: Int -> email: String -> User =
  User { name: name, age: age, email: email }

// アプリカティブでバリデーションを並列合成
let userResult1 = Right createUser
 <*> validateName "Alice"
 <*> validateAge 25
 <*> validateEmail "alice@example.com"

let userResult2 = Right createUser
 <*> validateName "admin"  // エラー
 <*> validateAge 25
 <*> validateEmail "alice@example.com"

let userResult3 = Right createUser
 <*> validateName "Bob"
 <*> validateAge -5  // エラー
 <*> validateEmail "test@test.com"  // エラー

show userResult1  // Right User { name: "Alice", age: 25, email: "alice@example.com" }
show userResult2  // Left "予約語は使用できません"
show userResult3  // Left "年齢は負数にできません"  （注：最初のエラーのみ）

// =============================================================================
// 関数の合成
// =============================================================================

print "--- 関数の合成 ---"

// 関数を返す関数
let compose = \\f -> \\g -> \\x -> g x | f

// 基本的な関数
fn double x: Int -> Int = x * 2
fn increment x: Int -> Int = x + 1
fn square x: Int -> Int = x * x

// 関数の合成
let doubleAndIncrement = compose increment double
let squareAndDouble = compose double square

show $ doubleAndIncrement 5  // 11
show $ squareAndDouble 3     // 18

// =============================================================================
// 実践的な例：データ処理パイプライン
// =============================================================================

print "--- データ処理パイプライン ---"

// APIレスポンスの解析パイプライン
fn fetchUserData userId: String -> Either<String, String> = match userId {
  "user123" -> Right \`{"id": "user123", "score": 85}\`
  "user456" -> Right \`{"id": "user456", "score": -10}\`
  _ -> Left \`ユーザー \${userId} が見つかりません\`
}

fn parseScore json: String -> Either<String, Int> = match json {
  s when s == \`{"id": "user123", "score": 85}\` -> Right 85
  s when s == \`{"id": "user456", "score": -10}\` -> Right -10
  _ -> Left "JSONの解析に失敗しました"
}

fn calculateBonus score: Int -> Either<String, Int> = match score {
  s when s >= 80 -> Right (s * 2)  // 高得点はボーナス2倍
  s when s >= 60 -> Right (s + 20)  // 中得点は + 20
  s when s > 0 -> Right s           // 低得点はそのまま
  _ -> Left \`スコア \${score} は無効です（負の値）\`
}

fn generateReport bonus: Int -> Either<String, String> = match bonus {
  b when b >= 150 -> Right \`優秀！ボーナススコア: \${b}\`
  b when b >= 80 -> Right \`良好。ボーナススコア: \${b}\`
  b -> Right \`ボーナススコア: \${b}\`
}

// 前の結果に依存する処理チェーン
let report1 = fetchUserData "user123"
  >>= parseScore
  >>= calculateBonus
  >>= generateReport

let report2 = fetchUserData "user456"
  >>= parseScore
  >>= calculateBonus
  >>= generateReport

let report3 = fetchUserData "unknown"
  >>= parseScore
  >>= calculateBonus
  >>= generateReport

show report1  // Right "優秀！ボーナススコア: 170"
show report2  // Left "スコア -10 は無効です（負の値）"
show report3  // Left "ユーザー unknown が見つかりません"

// =============================================================================
// モナド合成の利点
// =============================================================================

print "--- モナド合成の利点 ---"

// 1. 複雑な計算の表現
// 2. エラーハンドリングの自動化
// 3. 関数の合成可能性
// 4. 型安全性の維持

print "モナドの合成により、複雑な計算を安全かつ簡潔に表現できます"

// 実行方法:
// seseragi run examples/advanced/03-monad-composition.ssrg`,

  "advanced/04-real-world-examples": `// 04-real-world-examples.ssrg
// 実践的な例を学ぶ

print "=== 実践的な例 ==="

// =============================================================================
// FizzBuzz問題
// =============================================================================

print "--- FizzBuzz ---"

// FizzBuzz関数
fn fizzBuzz n: Int -> String = match (n % 3, n % 5) {
  (0, 0) -> "FizzBuzz"
  (0, _) -> "Fizz"
  (_, 0) -> "Buzz"
  _ -> \`\${n}\`
}

// FizzBuzzのテスト
[print $ fizzBuzz n | n <- 1..=100]

// =============================================================================
// 計算器の実装
// =============================================================================

print "--- 計算器 ---"

// 演算の種類
type Operation =
  | Add
  | Subtract
  | Multiply
  | Divide

// 安全な計算関数
fn calculate op: Operation -> x: Int -> y: Int -> Either<String, Int> = match op {
  Add -> Right (x + y)
  Subtract -> Right (x - y)
  Multiply -> Right (x * y)
  Divide -> match y {
    0 -> Left "ゼロ除算エラー"
    _ -> Right (x / y)
  }
}

// 計算例
let result1 = calculate Add 10 5
let result2 = calculate Divide 20 4
let result3 = calculate Divide 10 0

show result1  // Right 15
show result2  // Right 5
show result3  // Left "ゼロ除算エラー"

// =============================================================================
// 簡単な在庫管理システム
// =============================================================================

print "--- 在庫管理システム ---"

// 商品の構造体
struct Product {
  id: Int,
  name: String,
  price: Int,
  stock: Int
}

impl Product {
  fn updateStock self -> newStock: Int -> Product =
    Product { ...self, stock: newStock }

  fn sell self -> quantity: Int -> Either<String, Product> = match quantity {
    q when q <= 0 -> Left "数量は正数である必要があります"
    q when q > self.stock -> Left "在庫が不足しています"
    q -> Right $ self updateStock (self.stock - q)
  }

  fn restock self -> quantity: Int -> Either<String, Product> = match quantity {
    q when q <= 0 -> Left "数量は正数である必要があります"
    q -> Right $ self updateStock (self.stock + q)
  }
}

// 商品の使用例
let apple = Product { id: 1, name: "りんご", price: 100, stock: 50 }
show apple  // Product { id: 1, name: "りんご", price: 100, stock: 50 }

let soldApple = apple sell 10
show soldApple  // Right Product { id: 1, name: "りんご", price: 100, stock: 40 }

let restockedApple = match soldApple {
  Left error -> Left error
  Right product -> product restock 20
}
show restockedApple  // Right Product { id: 1, name: "りんご", price: 100, stock: 60 }

// =============================================================================
// 簡単な銀行システム
// =============================================================================

print "--- 銀行システム ---"

// 取引の種類
type Transaction =
  | Deposit Int
  | Withdraw Int
  | Transfer Int Int

// 口座の構造体
struct BankAccount {
  id: Int,
  owner: String,
  balance: Int
}

impl BankAccount {
  fn deposit self -> amount: Int -> Either<String, BankAccount> = match amount {
    a when a <= 0 -> Left "入金額は正数である必要があります"
    a -> Right (BankAccount { ...self, balance: self.balance + a })
  }

  fn withdraw self -> amount: Int -> Either<String, BankAccount> = match amount {
    a when a <= 0 -> Left "出金額は正数である必要があります"
    a when a > self.balance -> Left "残高が不足しています"
    a -> Right (BankAccount { ...self, balance: self.balance - a })
  }

  fn getStatement self -> String {
    let { id, owner, balance } = self
    \`口座ID: \${id}, 所有者: \${owner}, 残高: \${balance}円\`
  }
}

// 口座の使用例
let account = BankAccount { id: 1001, owner: "田中太郎", balance: 50000 }
show account  // BankAccount { id: 1001, owner: "田中太郎", balance: 50000 }

// 一連の取引
let step1 = account deposit 20000
let step2 = match step1 {
  Left error -> Left error
  Right acc -> acc withdraw 15000
}
let step3 = match step2 {
  Left error -> Left error
  Right acc -> acc deposit 5000
}

show step1  // Right BankAccount { id: 1001, owner: "田中太郎", balance: 70000 }
show step2  // Right BankAccount { id: 1001, owner: "田中太郎", balance: 55000 }
show step3  // Right BankAccount { id: 1001, owner: "田中太郎", balance: 60000 }

// getStatementの使用例
let statement = match step3 {
  Left error -> error
  Right acc -> acc getStatement()
}
show statement  // "口座ID: 1001, 所有者: 田中太郎, 残高: 60000円"

// 失敗例：残高不足による出金失敗
let failedWithdraw = account withdraw 100000
show failedWithdraw  // Left "残高が不足しています"

// 失敗例：無効な入金額
let invalidDeposit = account deposit (-1000)
show invalidDeposit  // Left "入金額は正数である必要があります"

// =============================================================================
// 成績管理システム
// =============================================================================

print "--- 成績管理システム ---"

// 成績の構造体
struct Grade {
  subject: String,
  score: Int
}

// 学生の構造体
struct Student {
  name: String,
  grades: List<Grade>
}

fn calculateAverage grades: List<Grade> -> sum: Int -> count: Int -> Int = match grades {
  Empty -> sum / count
  Cons grade rest -> calculateAverage (rest) (sum + grade.score) (count + 1)
}

impl Student {
  fn addGrade self -> grade: Grade -> Student {
    let grades = grade : self.grades
    Student { ...self, grades }
  }

  // 平均計算
  fn getAverageScore self -> Maybe<Int> = match (self.grades) {
    Empty -> Nothing
    _ -> Just (calculateAverage (self.grades) 0 0)
  }
}

// 学生の使用例
let mathGrade = Grade { subject: "数学", score: 80 }
let englishGrade = Grade { subject: "英語", score: 90 }

let student = Student { name: "佐藤花子", grades: \`[mathGrade] }
show student  // Student { name: "佐藤花子", grades: \`[Grade { subject: "数学", score: 80 }] }

let student' = student addGrade englishGrade
show student'  // Student { name: "佐藤花子", grades: \`[Grade { subject: "英語", score: 90 }, Grade { subject: "数学", score: 80 }] }

let average = student' getAverageScore()
show average  // Just 85

// =============================================================================
// 実践的なプログラミングの原則
// =============================================================================

print "--- 実践的なプログラミングの原則 ---"

// 1. 型安全性を活用する
// 2. エラーハンドリングを適切に行う
// 3. 不変性を保つ
// 4. 関数の合成を活用する
// 5. パターンマッチングを使って表現力を高める

print "実践的なプログラムでは、型安全性とエラーハンドリングが重要です"

// 実行方法:
// seseragi run examples/advanced/04-real-world-examples.ssrg`,
}

// パースしてセクション分割されたデータを生成
export const allCodeSections: CodeSection[] = []

// 各ファイルをパースしてセクションに分割
Object.entries(sampleFiles).forEach(([filename, code]) => {
  const category = filename.startsWith("basics/")
    ? "basics"
    : filename.startsWith("intermediate/")
      ? "intermediate"
      : "advanced"

  console.log(`Processing file: ${filename} (category: ${category})`)
  const parsed = parseCodeSections(code, filename, category)
  console.log(`Parsed sections for ${filename}:`, parsed.sections.length)
  allCodeSections.push(...parsed.sections)
})

console.log(`Total sections parsed: ${allCodeSections.length}`)
console.log(
  "Section titles:",
  allCodeSections.map((s) => s.title)
)

// カテゴリ別に整理
export const sectionsByCategory = {
  basics: allCodeSections.filter((s) => s.category === "basics"),
  intermediate: allCodeSections.filter((s) => s.category === "intermediate"),
  advanced: allCodeSections.filter((s) => s.category === "advanced"),
}

// ファイル別に整理
export const sectionsByFile = allCodeSections.reduce(
  (acc, section) => {
    const fileKey = section.id.split("-").slice(0, -1).join("-")
    if (!acc[fileKey]) acc[fileKey] = []
    acc[fileKey].push(section)
    return acc
  },
  {} as Record<string, CodeSection[]>
)

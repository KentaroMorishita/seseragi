export const samples = [
  {
    name: "Welcome to Seseragi",
    code: `// Welcome to Seseragi Playground!
// Seseragiは簡潔な構文でTypeScriptコードを生成する新しいプログラミング言語です

print "=== Seseragi言語の特徴 ==="

// 🔹 シンプルな関数定義
fn greet name: String -> String = \`こんにちは、\${name}さん！\`
let message = greet "プログラマー" // messageはStringに推論される
show message // "こんにちは、プログラマーさん！"

fn add x: Int -> y: Int -> Int = x + y
show $ add 10 20 // 30

// 🔹 null安全なMaybe型
fn findUser id: Int -> Maybe<String> =
  id == 1 ? Just "Alice" :
  id == 2 ? Just "Bob" :
  Nothing

show $ findUser 1  // Just("Alice")
show $ findUser 99 // Nothing

// 🔹 配列の操作と内包表記
fn double x: Int -> Int = x * 2

let numbers: Array<Int> = 1..=10
let doubled: Array<Int> = [double x | x <- numbers]
show doubled  // [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]

// 🔹 パターンマッチング
fn describe n: Int -> String = match n {
  0 -> "zero"
  1 -> "one"
  2 -> "two"
  _ -> "other"
}

show $ describe 1  // "one"

print "✨ 左のExamplesボタンから更に多くのサンプルを試してみてください！"`,
  },
  {
    name: "Basic Functions",
    code: `// 関数定義と型推論
fn square x: Int -> Int = x * x
fn greet name: String -> String = \`Hello, \${name}!\`

// 関数の使用
show $ square 5
show $ greet "Alice"

// カリー化された関数
fn add x: Int -> y: Int -> Int = x + y
let add5 = add 5
show $ add5 10`,
  },
  {
    name: "Maybe Basics",
    code: `// Maybe型の基本
let someValue = Just 42
let nothingValue = Nothing

show someValue    // Just 42
show nothingValue // Nothing

// 安全な操作
fn numberToMonth n: Int -> Maybe<String> =
  n == 1 ? Just "January" :
  n == 2 ? Just "February" :
  n == 3 ? Just "March" :
  Nothing

show $ numberToMonth 1   // Just "January"
show $ numberToMonth 13  // Nothing`,
  },
  {
    name: "Conditionals",
    code: `// 条件分岐
let x = 10
let result = if x > 5 then "大きい" else "小さい"
show result

// 三項演算子
fn classify age: Int -> String =
  age < 13 ? "子供" :
  age < 20 ? "ティーンエイジャー" :
  age < 65 ? "大人" :
  "高齢者"

show $ classify 16   // "ティーンエイジャー"
show $ classify 30   // "大人"`,
  },
  {
    name: "Template Literals",
    code: `// テンプレートリテラル
let name = "Alice"
let age = 25
let message = \`Hello, \${name}! You are \${age} years old.\`
show message

// 計算結果の埋め込み
let x = 10
let y = 5
let calc = \`\${x} + \${y} = \${x + y}\`
show calc

// 関数結果の埋め込み
fn square n: Int -> Int = n * n
let result = \`The square of 7 is \${square 7}\`
show result`,
  },
]

# Seseragi Language Support for VS Code

このVS Code拡張は、Seseragi関数型プログラミング言語（.ssrg）のシンタックスハイライトと基本的な言語サポートを提供します。

## 機能

- ✅ シンタックスハイライト
- ✅ 括弧の自動補完
- ✅ コメントサポート
- ✅ インデント規則
- ✅ 言語固有の設定

## サポートされる言語機能

### キーワード
- `fn`, `let`, `type`, `impl`, `monoid`
- `effectful`, `match`, `import`, `as`
- `if`, `then`, `else`

### 型システム
- **基本型**: `Int`, `Float`, `Bool`, `String`, `Char`, `Unit`
- **ジェネリック型**: `Maybe`, `Either`, `IO`, `List`, `Array`
- **ユーザー定義型**: 大文字で始まる型名

### 演算子
- **パイプライン**: `|` (左から右への関数合成)
- **逆パイプ**: `~` (部分適用)
- **モナドバインド**: `>>=` (FlatMap操作)
- **畳み込みモノイド**: `>>>` (モノイド操作)
- **関数型**: `->` (型注釈)
- **算術演算子**: `+`, `-`, `*`, `/`, `%`
- **比較演算子**: `==`, `!=`, `<`, `>`, `<=`, `>=`
- **論理演算子**: `&&`, `||`

### 組み込み関数
- **Maybe**: `Just`, `Nothing`
- **Either**: `Left`, `Right`
- **List**: `Cons`, `Nil`

## 使用方法

1. `.ssrg` 拡張子のファイルを作成
2. Seseragiコードを記述
3. 自動的にシンタックスハイライトが適用される

## サンプルコード

```seseragi
// 基本的な関数定義
fn add a :Int -> b :Int -> Int = a + b

// パイプライン演算子
fn processNumber x :Int -> Int = 
  x | double | square

// Maybe型
fn safeDivide a :Int -> b :Int -> Maybe<Int> =
  if b == 0 then Nothing else Just (a / b)

// パターンマッチング
type Color = Red | Green | Blue

fn colorName color :Color -> String = match color
  | Red -> "赤"
  | Green -> "緑" 
  | Blue -> "青"
```

## 設定

VS Codeの設定で以下が自動的に適用されます：

- ファイル関連付け: `*.ssrg` → `seseragi`
- カスタムトークンカラー
- 自動インデント
- 括弧の自動補完

## 今後の予定

- [ ] IntelliSense（自動補完）
- [ ] エラー診断
- [ ] 定義へのジャンプ
- [ ] コードフォーマット
- [ ] デバッガーサポート
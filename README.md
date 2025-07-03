# Seseragi

A programming language that compiles to TypeScript

## 特徴

- **静的型付け** - 型推論による型安全性
- **不変変数** - `let`による再代入不可な変数
- **モナド型** - `Maybe`、`Either`による安全なエラーハンドリング
- **VS Code統合** - 構文ハイライト、LSP、リアルタイム型チェック

## クイックスタート

```bash
# インストール
bun install

# Hello World
echo 'let message = "Hello, Seseragi!"
print message' > hello.ssrg

# 実行
seseragi run hello.ssrg
```

## 基本的な使い方

```bash
# Seseragiファイルを直接実行
seseragi run example.ssrg

# TypeScriptにコンパイル（シンプル版）
seseragi example.ssrg              # example.ts に出力

# TypeScriptにコンパイル（出力先指定）
seseragi example.ssrg -o output.ts

# コードフォーマット（上書き）
seseragi fmt example.ssrg

# コードフォーマット（出力先指定）
seseragi fmt example.ssrg -o formatted.ssrg

# ファイル監視でコンパイル
seseragi example.ssrg --auto

# ダイレクト実行
seseragi run example.ssrg

# ファイル監視で実行
seseragi run example.ssrg --watch
```

## サンプルコード

```seseragi
// Hello World
print "Hello world"

// 基本的な変数・関数定義
let name = "Alice"
fn greet name: String -> String = "Hello " + name
print $ greet name

// 関数の自動カリー化・部分適用
fn add x: Int -> y: Int -> Int = x + y
let addTen = add 10
print $ addTen 20  // 30

// 構造体定義と演算子オーバーロード
struct Point {
  x: Int,
  y: Int
}

impl Point {
  fn square self -> Point {
    let Point { x, y } = self
    Point { x: x * x, y: y * y }
  }

  operator + self -> other -> Point {
    Point { x: self.x + other.x, y: self.y + other.y }
  }
  operator * self -> scalar: Int -> Point {
    Point { x: self.x * scalar, y: self.y * scalar }
  }
}

let p1 = Point { x: 3, y: 4 }
let p2 = Point { x: 1, y: 2 }
show $ p1 square() // Point { x: 9, y: 16 }
show $ p1 + p2 * 3  // Point { x: 6, y: 10 }

// 配列とリストの使い分け
let array = [1, 2, 3, 4, 5]     // Array - インデックスアクセス
let list = `[1, 2, 3, 4, 5]     // List - head/tail操作

show $ [x * x | x <- 1..=5]     // 配列内包表記: [1, 4, 9, 16, 25]
show $ ^list                    // リストhead: Just 1
show $ >>list                   // リストtail: `[2, 3, 4, 5]
```

## VS Code拡張

1. プロジェクトをVS Codeで開く
2. 拡張機能が自動でインストールされる
3. `.ssrg`ファイルで構文ハイライト、型チェックが有効になる

## 開発・ドキュメント

- **開発者向けガイド**: [CLAUDE.md](./CLAUDE.md)
- **サンプルコード**: [examples/](./examples/)

## ライセンス

Apache-2.0

## 技術スタック

- **Runtime**: [Bun](https://bun.sh)
- **Target**: TypeScript/JavaScript
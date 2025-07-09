# 🚀 Seseragi クイックスタート

**5分でSeseragiを試してみよう！**

## 📋 必要なもの

- Seseragi がインストールされていること
- ターミナルまたはコマンドプロンプト

## 🎯 ステップ1: Hello World

まずは最もシンプルなプログラムから始めましょう：

```bash
seseragi run examples/basics/01-hello-world.ssrg
```

**期待される出力:**
```
=== Hello, World! ===
--- 出力の基本 ---
Hello, World!
42
True
--- 関数適用演算子 $ ---
10
10
20
40
```

## 🔢 ステップ2: 基本型と変数

Seseragiの基本的な型を試してみましょう：

```bash
seseragi run examples/basics/02-types-and-variables.ssrg
```

**学習ポイント:**
- Int, Float, String, Bool の基本型
- 算術演算と比較演算
- 文字列補間

## 🎯 ステップ3: 関数を作ってみよう

関数の定義と呼び出しを学びましょう：

```bash
seseragi run examples/basics/03-functions.ssrg
```

**学習ポイント:**
- 関数の定義（`fn`キーワード）
- 引数なしの関数は `()` で呼び出す
- ブロック構文 `{}` では `=` は不要
- カリー化（部分適用）
- 高階関数

## 🔄 ステップ4: 条件分岐を学ぼう

条件分岐の書き方を学びましょう：

```bash
seseragi run examples/basics/04-conditionals.ssrg
```

**学習ポイント:**
- if-then-else式
- 三項演算子 `cond ? A : B`
- Bool値は `True` と `False`
- 比較演算子とブール演算子

## 🌟 ステップ5: Maybe型を試してみよう

Seseragiの特徴的な機能の一つ、Maybe型を試してみましょう：

```bash
seseragi run examples/intermediate/01-maybe-basics.ssrg
```

**学習ポイント:**
- `Just` と `Nothing` の使い方
- 安全な値の表現
- null安全性

## 🎮 ステップ6: 実験してみよう

プレイグラウンドで自由にコードを試してみましょう：

```bash
seseragi run examples/playground.ssrg
```

または、自分でコードを書いてみましょう：

```seseragi
// your_experiment.ssrg
fn greet name: String -> String = `Hello, ${name}!`
show $ greet "World"
```

```bash
seseragi run your_experiment.ssrg
```

## 📚 次のステップ

クイックスタートを完了したら：

1. **基礎レベルを完了する**: `examples/basics/` の全ファイルを実行
2. **中級レベルに進む**: `examples/intermediate/` でモナド型を学習
3. **上級レベルに挑戦**: `examples/advanced/` で実践的な応用を学習

## 🔧 実行方法のバリエーション

### ファイル監視モード
ファイルを編集するたびに自動実行：
```bash
seseragi run examples/basics/01-hello-world.ssrg --watch
```

### コンパイル後実行
TypeScriptファイルに変換してから実行：
```bash
seseragi examples/basics/01-hello-world.ssrg -o output.ts
bun output.ts
```

## 💡 よくある質問

### Q: 関数適用の優先度は？
A: `$` 演算子を使って括弧を減らせます：
```seseragi
show $ double 5  // show(double(5)) と同じ
```

### Q: 変数名にアポストロフィーは使える？
A: はい！ `let x'` や `fn calculate'` のように使えます。

### Q: 引数なしの関数の呼び出し方は？
A: 必ず `()` を付けて呼び出します：
```seseragi
fn hello -> String = "Hello"
show hello()  // "Hello"
show hello    // () => "Hello"
```

### Q: エラーが出たらどうする？
A: エラーメッセージを確認し、型注釈を明示的に書いてみましょう。

## 🎉 おめでとうございます！

Seseragiの基本的な使い方を学びました！

**次は何をする？**
- 📖 [README.md](./README.md) で詳しい学習ガイドを確認
- 🎯 体系的な学習を開始
- 🚀 自分だけのSeseragiプロジェクトを作成

---

**Happy Coding with Seseragi! 🌊**
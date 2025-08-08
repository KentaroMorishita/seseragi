# Seseragi サンプルコード

Seseragiプログラミング言語を体系的に学習するためのサンプルコード集です。

## 🎯 学習の流れ

Seseragiは**基礎** → **中級** → **上級**の3段階で学習できます：

### 📚 基礎レベル（basics/）
プログラミングの基本概念を学びます。

```bash
# 順番に実行して学習してください
seseragi run examples/basics/01-hello-world.ssrg
seseragi run examples/basics/02-types-and-variables.ssrg
seseragi run examples/basics/03-functions.ssrg
seseragi run examples/basics/04-conditionals.ssrg
```

### 🚀 中級レベル（intermediate/）
関数プログラミングの核心概念を学びます。

```bash
seseragi run examples/intermediate/01-maybe-basics.ssrg
seseragi run examples/intermediate/02-either-basics.ssrg
seseragi run examples/intermediate/03-lists-and-arrays.ssrg
seseragi run examples/intermediate/04-operators.ssrg
seseragi run examples/intermediate/05-frp-signals.ssrg
seseragi run examples/intermediate/06-tasks.ssrg
seseragi run examples/intermediate/05-frp-signals.ssrg
```

### ⚡ 上級レベル（advanced/）
実践的な応用と高度な機能を学びます。

```bash
seseragi run examples/advanced/01-pattern-matching.ssrg
seseragi run examples/advanced/02-structs-and-methods.ssrg
seseragi run examples/advanced/03-monad-composition.ssrg
seseragi run examples/advanced/04-real-world-examples.ssrg
```

## 🗂️ ディレクトリ構成

```
examples/
├── README.md                           # このファイル
├── QUICK_START.md                      # 5分でSeseragiを試す
├── playground.ssrg                     # 実験用プレイグラウンド
├── basics/                             # 基礎レベル
│   ├── 01-hello-world.ssrg            # 最初のSeseragiプログラム
│   ├── 02-types-and-variables.ssrg    # 基本型と変数
│   ├── 03-functions.ssrg              # 関数定義と呼び出し
│   └── 04-conditionals.ssrg           # 条件分岐とブール値
├── intermediate/                       # 中級レベル
│   ├── 01-maybe-basics.ssrg           # Maybe型の基本
│   ├── 02-either-basics.ssrg          # Either型の基本
│   ├── 03-lists-and-arrays.ssrg       # ListとArrayの基本
│   ├── 04-operators.ssrg              # モナド演算子の基本
│   ├── 05-frp-signals.ssrg            # FRP: Signalの基本
│   └── 06-tasks.ssrg                  # 非同期計算: Task の基本
└── advanced/                           # 上級レベル
    ├── 01-pattern-matching.ssrg       # パターンマッチング
    ├── 02-structs-and-methods.ssrg    # 構造体とメソッド
    ├── 03-monad-composition.ssrg      # モナドの合成
    └── 04-real-world-examples.ssrg    # 実践的な例
```

## 🎮 実行方法

### 基本的な実行
```bash
seseragi run examples/basics/01-hello-world.ssrg
```

### ファイル監視モード（開発中に便利）
```bash
seseragi run examples/basics/01-hello-world.ssrg --watch
```

### コンパイル後実行
```bash
seseragi examples/basics/01-hello-world.ssrg -o output.ts
bun output.ts
```

## 📖 学習のポイント

### 基礎レベルで学ぶこと
- **基本型**: Int, Float, String, Bool
- **変数宣言**: letの使い方
- **関数**: 定義、呼び出し、カリー化
- **条件分岐**: if-then-else、三項演算子

### 中級レベルで学ぶこと
- **Maybe型**: 安全な値の表現
- **Either型**: エラーハンドリング
- **List/Array**: データ構造の基本
- **モナド演算子**: `<$>`, `<*>`, `>>=`

### 上級レベルで学ぶこと
- **パターンマッチング**: 代数データ型の活用
- **構造体**: データとメソッドの組み合わせ
- **モナド合成**: 複雑な計算の表現
- **実践例**: 実際のプログラムの作成

## 🛠️ Seseragiの特徴

### 関数プログラミング
- **カリー化**: すべての関数は自動的にカリー化される
- **不変性**: データは変更されず、新しい値が作成される
- **型安全性**: 型システムによる安全性

### モナド型システム
- **Maybe型**: null安全性を提供
- **Either型**: エラーハンドリングの改善
- **List型**: 連結リスト

### 実用的な機能
- **構造体**: データとメソッドの組み合わせ
- **パターンマッチング**: データ分解
- **TypeScript出力**: 既存のエコシステムとの統合

## 💡 学習のコツ

1. **順番に学習**: 基礎 → 中級 → 上級の順で進む
2. **手を動かす**: コードを実際に実行して理解を深める
3. **実験**: playground.ssrgで自由にコードを試す
4. **応用**: 学んだ概念を組み合わせて新しいプログラムを作成

## 🆘 困ったときは

### よくあるエラー
- **型エラー**: 型注釈を明示的に書く
- **構文エラー**: 括弧やセミコロンの確認
- **未定義エラー**: 関数や変数の定義を確認

### デバッグのヒント
- **小さく始める**: 複雑な処理を分割して確認
- **show関数**: 値の確認にshow関数を使用
- **段階的構築**: 機能を少しずつ追加して動作確認

## 🚀 次のステップ

サンプルコードを一通り学習したら：

1. **自分でプログラムを書く**: 学んだ概念を使って新しいプログラムを作成
2. **実際のプロジェクトに応用**: より大きなプロジェクトでSeseragiを活用

---

**Happy Coding with Seseragi!**

# Seseragi Programming Language

Seseragiは、TypeScriptにトランスパイルされる純粋関数型プログラミング言語です。

## 特徴

- **純粋関数型プログラミング** - 副作用の明示的管理
- **静的型付け** - 型安全性の保証
- **カリー化された関数** - デフォルトで部分適用をサポート
- **モナド合成** - Maybe、Either、IOモナドによる安全な処理
- **パイプライン演算子** - 関数合成の直感的な記述

## クイックスタート

### 依存関係のインストール

```bash
bun install
```

### Seseragiコンパイラの実行

```bash
# TypeScriptトランスパイラのテスト
bun run src/main.ts

# 全テスト実行
bun test

# 型チェック
bun run typecheck
```

### サンプルコード

Seseragi言語（`.ssrg`）のサンプル：

```seseragi
// 基本的な関数定義
fn add a :Int -> b :Int -> Int = a + b

// パイプライン演算子
fn processNumber x :Int -> Int = 
  x | double | square

// Maybe型による安全な処理
fn safeDivide a :Int -> b :Int -> Maybe<Int> =
  b == 0 is True then Nothing else Just (a / b)
```

## VS Code拡張

Seseragi言語用のVS Code拡張が含まれています：

### 機能
- ✅ シンタックスハイライト
- ✅ 括弧の自動補完
- ✅ コメントサポート
- ✅ 言語固有の設定

### セットアップ
1. `.ssrg` 拡張子のファイルを作成
2. 自動的にシンタックスハイライトが適用される
3. サンプル: `examples/sample.ssrg`

## 開発情報

### 技術スタック
- **言語**: TypeScript/JavaScript
- **パーサー**: 手書き再帰降下パーサー
- **テスト**: Bun Test
- **ビルド**: Bun
- **ランタイム**: [Bun](https://bun.sh)

### プロジェクト構造
```
src/
├── lexer.ts      # 字句解析器
├── parser.ts     # 構文解析器
├── ast.ts        # 抽象構文木定義
├── codegen.ts    # TypeScriptコード生成
└── main.ts       # メインエントリーポイント

tests/            # テストファイル
examples/         # Seseragiサンプルコード
.vscode/          # VS Code拡張とワークスペース設定
```

### 開発コマンド
```bash
# 開発サーバー
bun run dev

# ビルド
bun run build:all

# テスト
bun test
bun run test:watch

# 品質チェック
bun run typecheck
bun run lint
bun run format
```

## コントリビューション

詳細な開発ガイドは `CLAUDE.md` を参照してください。

## ライセンス

MIT License

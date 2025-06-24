# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

Seseragiは、TypeScriptにトランスパイルされる純粋関数型プログラミング言語です。このリポジトリには言語仕様と設計文書が含まれています。現在は実装コードはなく、日本語での設計文書のみが存在します。

## リポジトリ構造

- `design.md` - 構文、型、機能を含む包括的な言語仕様
- `language-design.md` - 代替言語仕様文書

## 言語アーキテクチャ

### 核となる設計原則
- **純粋関数型プログラミング** - `effectful`キーワードによる明示的な副作用管理
- **静的型付け** - 型安全性の保証
- **不変変数** - `let`キーワードを使用、再代入不可
- **デフォルトでカリー化された関数** - すべての関数が部分適用をサポート
- **モナド合成** - オプション値、エラー、副作用の処理

### 主要言語機能

**型システム:**
- 基本型: `Int`, `Float`, `Bool`, `String`, `Char`, `Unit`
- コレクション: `List<T>`, `Array<T>`
- モナド: `Maybe<T>`, `Either<L,R>`, `IO<T>` (副作用管理用)
- `type`キーワードによるユーザー定義型
- `impl`ブロックでのメソッド実装、`monoid`サポート

**関数定義:**
- `fn`キーワードによる関数定義
- ワンライナーとブロック構文の両方をサポート
- 明示的な関数型構文 `(Type -> Type)` による高階関数
- `operator`キーワードで定義された演算子の中置記法

**特殊演算子:**
- パイプライン演算子 `|` - 左から右への関数合成
- 逆パイプ演算子 `~` - 部分適用用
- FlatMap演算子 `>>=` - モナド操作用
- FoldMonoid演算子 `>>>` - モノイド畳み込み操作用

**制御フロー:**
- 従来のif/elseの代わりに`Condition`型と`is`, `then`, `else`
- 代数的データ型用の`match`キーワードによるパターンマッチング
- 従来のループなし - 関数合成と再帰を推奨

### モジュールシステム
- ファイルパスベースのモジュール管理
- インポート構文: `import module::{Type1, Type2}`
- 名前衝突回避のための`as`キーワードによる型エイリアス

## 言語実装プラン

### フェーズ1: 基礎実装 (1-2ヶ月)
1. **字句解析器 (Lexer)**
   - Seseragi構文のトークン化
   - 予約語、演算子、リテラルの認識
   - TypeScript/JavaScriptで実装

2. **構文解析器 (Parser)**
   - 抽象構文木 (AST) の生成
   - 関数定義、型定義、変数定義の解析
   - パターンマッチング構文の解析

3. **基本型システム**
   - プリミティブ型の実装
   - 型推論の基礎
   - 型チェッカーの骨格

### フェーズ2: コア機能 (2-3ヶ月)
1. **関数とカリー化**
   - カリー化された関数の実装
   - 部分適用のサポート
   - 高階関数の処理

2. **演算子システム**
   - 中置記法の実装
   - カスタム演算子の定義
   - 演算子の優先順位処理

3. **基本モナド**
   - `Maybe<T>`の実装
   - `Either<L,R>`の実装
   - モナド演算子 `>>=` の実装

### フェーズ3: 高度な機能 (2-3ヶ月)
1. **パターンマッチング**
   - `match`構文の完全実装
   - 網羅性チェック
   - ガード式のサポート

2. **副作用管理**
   - `IO<T>`モナドの実装
   - `effectful`関数の処理
   - 純粋性の検証

3. **モジュールシステム**
   - ファイルベースモジュール
   - インポート/エクスポート
   - 名前空間管理

### フェーズ4: TypeScriptトランスパイラ (1-2ヶ月)
1. **コード生成**
   - ASTからTypeScriptコードへの変換
   - カリー化の実装
   - モナドのJavaScript表現

2. **最適化**
   - 不要なカリー化の除去
   - インライン化
   - デッドコード除去

### フェーズ5: 開発ツール (1-2ヶ月)
1. **REPL**
   - インタラクティブな開発環境
   - 式の評価
   - 型情報の表示

2. **言語サーバー**
   - エディタサポート
   - シンタックスハイライト
   - エラー診断

## 実装技術スタック

- **言語**: TypeScript/JavaScript (セルフホスティング目標)
- **パーサー**: 手書き再帰降下パーサーまたはPEG.js
- **テスト**: Jest またはVitest
- **ビルド**: esbuildまたはVite
- **CLI**: Commander.js

## 開発時の注意点

1. **言語進化**: 構文や意味論の変更は両方の仕様文書に反映
2. **TypeScript互換性**: TypeScriptにトランスパイルするため、機能拡張時はTypeScript互換性を考慮
3. **関数型純粋性**: 関数型プログラミングパラダイムを維持 - 副作用は`IO`モナドと`effectful`関数で明示的に管理
4. **日本語文書**: 現在の文書はすべて日本語 - この慣例を維持するか並行して英語文書を提供

## 開発コマンド

現在利用可能なコマンド:

```bash
# 開発サーバー（ファイル監視）
bun run dev

# ビルド
bun run build          # JavaScript出力
bun run build:types    # TypeScript型定義
bun run build:all      # 全てのビルド

# テスト
bun test               # 全テスト実行
bun run test:watch     # ファイル監視でテスト
bun run test:coverage  # カバレッジ付きテスト

# 型チェック・品質管理
bun run typecheck      # TypeScript型チェック
bun run lint           # コード品質チェック
bun run format         # コードフォーマット
bun run format:check   # フォーマットチェック（変更なし）

# クリーン
bun run clean         # ビルド成果物削除
```

将来実装予定のコマンド:
```bash
# REPL起動
bun run repl

# トランスパイル実行
seseragi compile input.ses --output output.ts

# 型チェック
seseragi check input.ses
```

## 開発環境セットアップ手順

### 初回セットアップ
```bash
# 依存関係インストール
bun install

# 型チェック確認
bun run typecheck

# 全テスト実行
bun test
```

### コード品質管理
プロジェクトではBiome v2.0.5を使用してコードフォーマットとリンティングを行います:

```bash
# コードを自動フォーマット
bun run format

# リンティングエラーチェック
bun run lint

# 型安全性チェック
bun run typecheck
```

**フォーマット設定（biome.json）:**
- インデント: 2スペース
- 行幅: 80文字
- クォート: ダブルクォート
- セミコロン: 必要に応じて
- 末尾カンマ: ES5準拠

## 開発ワークフロー

### 1. 作業開始時
```bash
# TODOリストの確認
cat TODO.md

# 作業対象のタスクを決定
# TODOリストでタスクを進行中にマーク（手動）
```

### 2. 開発サイクル
```bash
# 機能ブランチ作成（必要に応じて）
git checkout -b feature/task-name

# 開発実行
# - コード実装
# - テスト追加・更新

# 品質チェック
bun run format      # コードフォーマット
bun run typecheck   # 型チェック
bun run lint        # リンティング
bun test            # テスト実行

# ビルド確認
bun run build:all   # 全ビルド実行
```

### 3. 作業完了時
```bash
# TODOリスト更新
# - 完了したタスクにチェックマーク
# - 必要に応じて新しいサブタスク追加

# 変更をコミット
git add .
git commit -m "タスク完了: [タスク名]

詳細な変更内容の説明

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# リモートにプッシュ（必要に応じて）
git push -u origin feature/task-name
```

### 4. プルリクエスト作成
```bash
# GitHub CLIでプルリクエスト作成
gh pr create --title "機能タイトル" --body "詳細説明"
```

### TODOリスト管理ルール
- タスク開始時: `[ ]` → `[進行中]`として手動マーク
- タスク完了時: `[進行中]` → `[x]`に変更
- 新しいサブタスクが発見された場合は追加
- 完了したタスクは必ずコミットと一緒に更新

### TypeScript設定

**開発用（tsconfig.json）:**
- ターゲット: ES2022
- 厳密な型チェック有効
- `src/`ディレクトリのみ対象

**テスト用（tsconfig.test.json）:**
- 開発設定を継承
- `tests/`ディレクトリも含む
- 一部厳密チェックを緩和（開発効率のため）
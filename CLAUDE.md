# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

Seseragiは、TypeScriptにトランスパイルされるプログラミング言語です。字句解析、構文解析、型推論、コード生成、CLI、LSPサーバー、VS Code拡張などの機能が実装されています。

## リポジトリ構造

```
src/
├── lexer.ts              # 字句解析器
├── parser.ts             # 構文解析器
├── ast.ts                # 抽象構文木定義
├── codegen.ts            # TypeScriptコード生成
├── typechecker.ts        # 型チェッカー
├── type-inference.ts     # Hindley-Milner型推論システム
├── usage-analyzer.ts     # 使用量解析
├── cli.ts                # CLIメインエントリーポイント
├── cli/                  # CLI個別コマンド実装
│   ├── compile.ts        # compile コマンド
│   ├── format.ts         # format コマンド
│   └── run.ts            # run コマンド
├── formatter/            # コードフォーマッター（複数実装）
├── lsp/                  # Language Server Protocol実装
│   ├── main.ts           # LSPサーバーエントリーポイント
│   └── server.ts         # LSPサーバー実装
└── runtime/              # Seseragiランタイムライブラリ

tests/                    # 包括的テストスイート
examples/                 # Seseragiサンプルコードと使用例
extensions/seseragi/      # VS Code拡張機能
```

## 言語アーキテクチャ

### 核となる設計原則
- **副作用の明示的管理** - `effectful`キーワードによる安全な副作用処理
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

## 現在の実装状況

### ✅ 完了済み実装

**コア言語機能:**
- ✅ **字句解析器 (Lexer)** - 実装済み (src/lexer.ts)
- ✅ **構文解析器 (Parser)** - 手書き再帰降下パーサー実装済み (src/parser.ts)
- ✅ **抽象構文木 (AST)** - 型安全なAST定義 (src/ast.ts)
- ✅ **型チェッカー** - 実装済み (src/typechecker.ts)
- ✅ **Hindley-Milner型推論システム** - 実装済み (src/type-inference.ts)
- ✅ **TypeScriptコード生成** - 実装済み (src/codegen.ts)

**開発ツール:**
- ✅ **CLI** - compile, run, format コマンド実装済み (src/cli.ts, src/cli/)
- ✅ **Language Server Protocol (LSP)** - VS Code統合対応済み (src/lsp/)
- ✅ **VS Code拡張** - シンタックスハイライト、補完等対応済み (extensions/seseragi/)
- ✅ **コードフォーマッター** - 複数のフォーマッター実装 (src/formatter/)
- ✅ **使用量解析** - 変数使用状況解析 (src/usage-analyzer.ts)

**モナドとランタイム:**
- ✅ **Maybe<T>モナド** - 実装済み
- ✅ **Either<L,R>モナド** - 実装済み
- ✅ **モナド演算子** - `>>=`, `<$>`, `<*>` 実装済み
- ✅ **Seseragiランタイム** - TypeScript/JavaScript実装 (src/runtime/)

**テストと品質:**
- ✅ **テストスイート** - Bunテストランナー使用
- ✅ **型安全性テスト** - 型推論、型チェックテスト
- ✅ **モナド法則テスト** - 数学的正確性検証
- ✅ **パフォーマンステスト** - 大規模コード処理確認

### 🔄 継続開発項目

**高度な機能 (優先度順):**
- 🔄 **パターンマッチングの拡張** - より複雑なパターン対応
- 🔄 **エラーメッセージの改善** - ユーザビリティ向上
- 🔄 **REPL実装** - インタラクティブ開発環境
- 🔄 **モジュールシステム** - インポート/エクスポート機能

## 実装技術スタック

- **言語**: TypeScript/JavaScript
- **ランタイム**: [Bun](https://bun.sh) - 高速JavaScript/TypeScriptランタイム
- **パーサー**: 手書き再帰降下パーサー (src/parser.ts)
- **テスト**: Bunの組み込みテストランナー (`bun test`)
- **ビルド**: Bunビルダー (`bun build`)
- **CLI**: Commander.js - コマンドライン引数解析
- **LSP**: vscode-languageserver - VS Code統合
- **コード品質**: Biome v2.0.5 - リンティング・フォーマット
- **型推論**: Hindley-Milner型推論システム (独自実装)

## 主要コンポーネント

### コンパイルパイプライン
1. **字句解析 (Lexer)**: Seseragiソースコードをトークンに変換
2. **構文解析 (Parser)**: トークンから抽象構文木 (AST) を生成
3. **型推論 (TypeInference)**: Hindley-Milner型推論でASTに型情報を付与
4. **型チェック (TypeChecker)**: 型安全性と型制約の検証
5. **コード生成 (CodeGen)**: ASTからTypeScriptコードを生成

### CLI Architecture
- **メインCLI (cli.ts)**: Commanderベースのコマンド分割
- **compileCommand**: .ssrg → .ts コンパイル（ウォッチモード対応）
- **runCommand**: 一時ファイル経由でのSeseragiコード実行
- **formatCommand**: 複数フォーマッター選択式コード整形

### LSP Server Architecture
- **LSP Server**: VS Code統合対応、診断・補完・ホバー・フォーマット
- **診断機能**: リアルタイムエラー・警告表示
- **型情報表示**: ホバーでHindley-Milner推論結果表示
- **自動フォーマット**: 保存時フォーマット統合

### Type Inference System
- **Hindley-Milner実装**: src/type-inference.ts
- **型変数の単一化**: TypeVariable, TypeConstructor, FunctionType
- **型制約の解決**: Constraint solving algorithm
- **多相型サポート**: Generics and type parameters

## 開発時の注意点

1. **言語進化**: 構文や意味論の変更は両方の仕様文書に反映
2. **TypeScript互換性**: TypeScriptにトランスパイルするため、機能拡張時はTypeScript互換性を考慮
3. **型安全性**: 強力な型システムと型推論により、実行時エラーを最小化
4. **日本語文書**: 現在の文書はすべて日本語 - この慣例を維持するか並行して英語文書を提供
5. **実用性重視**: 理論的純粋性よりも実用性を優先 - 必要に応じて副作用も扱える

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

**Seseragiコマンド:**
```bash
# 直接実行（推奨）
seseragi run examples/tutorial.ssrg

# TypeScriptにトランスパイル
seseragi compile input.ssrg --output output.ts

# ファイル監視でトランスパイル
seseragi compile input.ssrg --output output.ts --watch

# コードフォーマット
seseragi format input.ssrg --in-place
```

将来実装予定のコマンド:
```bash
# REPL起動
seseragi repl

# 型チェック
seseragi check input.ssrg

# パッケージ管理
seseragi init
seseragi install <package>
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
# 機能ブランチ作成（必須）
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

# 変更をコミット（featureブランチで）
git add .
git commit -m "タスク完了: [タスク名]

詳細な変更内容の説明

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# featureブランチをリモートにプッシュ
git push -u origin feature/task-name
```

### 4. プルリクエスト作成
**重要：必ずfeatureブランチから作成すること**

```bash
# 1. featureブランチ作成（作業開始時）
git checkout -b feature/task-name

# 2. 開発・テスト・コミット
# ... 開発作業 ...
git add .
git commit -m "実装内容の説明"

# 3. featureブランチをリモートにプッシュ
git push -u origin feature/task-name

# 4. MCPのGitHub機能でプルリクエスト作成（推奨）
# mcp__github-mcp-server__create_pull_request を使用

# または GitHub CLIでプルリクエスト作成
gh pr create --title "機能タイトル" --body "詳細説明"
```

**プルリクエスト作成の正しい手順：**
1. ✅ 最初にfeatureブランチを作成
2. ✅ featureブランチで開発・テスト
3. ✅ featureブランチにコミット
4. ✅ featureブランチをremoteにpush
5. ✅ MCPまたはGitHub CLIでプルリクエスト作成
6. ✅ レビュー後、MCPでマージ

**❌ 絶対にやってはいけないこと：**
- mainブランチで直接開発してpush
- 開発後にプルリクエストを作ろうとすること

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

## 言語機能追加時の必須手順

**新しい言語機能（トークン、演算子、構文等）を追加する際は、必ず以下の手順を全て実行すること：**

### ✅ 必須チェックリスト

#### 1. コア実装
- [ ] **Lexer更新** - 新しいトークンを `src/lexer.ts` の `TokenType` enum と `nextToken()` に追加
- [ ] **Parser更新** - 新しい構文解析ルールを `src/parser.ts` に追加
- [ ] **AST拡張** - 新しいノード型を `src/ast.ts` に定義
- [ ] **CodeGen更新** - TypeScript出力ルールを `src/codegen.ts` に追加

#### 2. 型推論システム更新（🚨重要🚨）
- [ ] **型推論ルール追加** - `src/type-inference.ts` に新しい演算子・構文の型推論ルールを追加
- [ ] **型制約処理** - 必要に応じて新しい型制約を実装
- [ ] **エラーハンドリング** - 適切な型エラーメッセージを追加

#### 3. VSCode拡張機能修正（🚨重要🚨）
- [ ] **構文ハイライト** - `extensions/seseragi/syntaxes/seseragi.tmLanguage.json` を更新
  - 新しいキーワード、演算子、構文パターンを追加
  - パターンの優先順位を確認
- [ ] **バージョンアップ** - `extensions/seseragi/package.json` のversion番号を上げる

#### 4. VSCode拡張機能再ビルド（🚨重要🚨）
```bash
cd extensions/seseragi

# 拡張機能再ビルド
bun run compile

# 古いパッケージ削除 & 新パッケージ作成
rm -f *.vsix && vsce package

# 古い拡張機能をアンインストール
code --uninstall-extension seseragi-dev.seseragi-language-support

# 新しい拡張機能をインストール
code --install-extension seseragi-language-support-*.vsix

# VSCodeをリロード（必須）
# Command Palette → "Developer: Reload Window"
```

#### 5. テスト・検証
- [ ] **テスト追加** - 新機能のテストケースを `tests/` に追加
- [ ] **既存テスト確認** - `bun test` で全テストが通ることを確認
- [ ] **コンパイル確認** - サンプルコードがコンパイルできることを確認
- [ ] **VSCode動作確認** - 構文ハイライトとエラー表示が正しく動作することを確認

#### 6. ドキュメント更新
- [ ] **サンプルコード** - `examples/` ディレクトリに使用例を追加
- [ ] **README更新** - 必要に応じて機能説明を追加

### 🚨 よくある見落とし

1. **型推論ルールの追加忘れ** → "Unknown binary operator" エラー
2. **VSCode拡張機能の再ビルド忘れ** → 構文エラーが表示され続ける
3. **VSCode拡張機能のバージョンアップ忘れ** → キャッシュにより更新されない
4. **VSCodeのリロード忘れ** → 新しい構文ハイライトが適用されない

### 💡 効率的な開発フロー

```bash
# 1. 実装フェーズ
vi src/lexer.ts          # トークン追加
vi src/parser.ts         # パーサー更新  
vi src/ast.ts            # AST拡張
vi src/type-inference.ts # 型推論追加（忘れやすい🚨）
vi src/codegen.ts        # コード生成追加

# 2. VSCode拡張フェーズ（忘れやすい🚨）
vi extensions/seseragi/syntaxes/seseragi.tmLanguage.json  # 構文更新
vi extensions/seseragi/package.json                       # version up
cd extensions/seseragi && ./rebuild-extension.sh          # 再ビルド

# 3. テスト・検証フェーズ
bun test                 # 全テスト実行
bun run src/cli.ts compile examples/test.ssrg  # コンパイル確認
# VSCode → Command Palette → "Developer: Reload Window"  # 拡張機能確認
```

## 特定開発タスク

### 単一テストファイル実行
```bash
# 特定のテストファイルのみ実行
bun test tests/type-inference.test.ts
bun test tests/parser.test.ts
bun test tests/monad-laws.test.ts

# テスト名パターンマッチング
bun test --grep "type inference"
bun test --grep "Maybe monad"
```

### LSPサーバー開発・デバッグ

**🚨🚨🚨 LSPサーバー修正時の必須手順 🚨🚨🚨**

**LSPサーバー（src/lsp/）を修正した場合は、以下を必ず実行：**

```bash
# 1. 【必須】プロジェクトビルド（LSPサーバーはdist/lsp/main.jsを使用）
bun run build

# 2. 【必須】VS Code完全再起動（アプリケーション自体を終了）
# Command Palette → "Developer: Reload Window" だけでは不十分！

# 3. デバッグ時のみ：LSPサーバーを独立起動
bun run src/lsp/main.ts --stdio
```

**⚠️ 絶対に忘れてはいけないこと：**
- LSPサーバーは `dist/lsp/main.js` を実行している
- `src/lsp/` の変更は `bun run build` しないと反映されない
- TypeScriptソースを修正してもJavaScriptビルドしないと無意味
- VS Codeの "Reload Window" だけでは古いLSPサーバープロセスが残る場合がある

```bash
# VS Code拡張のリロードテスト
cd extensions/seseragi
bun run compile
# VS Code: Ctrl+Shift+P → "Developer: Reload Window"
```

### 型推論システムの動作確認
```bash
# 型推論の詳細ログ付きで実行
DEBUG=type-inference bun run src/main.ts

# 特定のSeseragiファイルで型推論テスト
seseragi compile examples/tutorial.ssrg --output /tmp/test.ts
```

### フォーマッター開発
```bash
# 複数フォーマッター実装のテスト
bun test tests/formatter.test.ts

# 特定フォーマッター実装の確認
seseragi format examples/tutorial.ssrg --check
```